// Data structures for the movement system.
// Field offsets reference the Object/Person struct in popTB.exe.
// All structures verified via Ghidra disassembly and Frida instrumentation.

use super::constants::*;

/// 2D world coordinate (X, Z). Y is height, handled separately.
/// The game world is a toroidal 16-bit coordinate space.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorldCoord {
    pub x: i16,
    pub z: i16,
}

impl WorldCoord {
    pub fn new(x: i16, z: i16) -> Self {
        Self { x, z }
    }

    /// Convert world coordinates to tile coordinates.
    /// Binary: `tile = (world >> 8) & 0xFE`
    /// Each tile is 512 world units (the & 0xFE masks the lowest bit).
    pub fn to_tile(&self) -> TileCoord {
        TileCoord {
            x: ((self.x >> 8) & 0xFEu16 as i16) as u8,
            z: ((self.z >> 8) & 0xFEu16 as i16) as u8,
        }
    }

    pub fn as_array(&self) -> [i16; 2] {
        [self.x, self.z]
    }

    pub fn as_array_mut(&mut self) -> &mut [i16; 2] {
        // Safety: WorldCoord is repr(C)-compatible with [i16; 2]
        // But let's avoid unsafe — just return an array
        unsafe { &mut *(self as *mut WorldCoord as *mut [i16; 2]) }
    }
}

/// Tile coordinate in the 128×128 region map grid.
/// The low bit is always 0 (masked by 0xFE), so effective range is 0-254 step 2.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TileCoord {
    pub x: u8,
    pub z: u8,
}

impl TileCoord {
    pub fn new(x: u8, z: u8) -> Self {
        Self { x, z }
    }

    /// Convert tile back to world coordinates (center of tile).
    /// Binary: `world = ((tile_byte & 0xFE) + 1) << 8`
    pub fn to_world(&self) -> WorldCoord {
        WorldCoord {
            x: ((((self.x & 0xFE) as u16) + 1) << 8) as i16,
            z: ((((self.z & 0xFE) as u16) + 1) << 8) as i16,
        }
    }

    /// Compute the region map cell index from tile coordinates.
    /// Binary: `((tile_x & 0xFE) * 2) | (tile_z >> 1)` → ×16 for byte offset.
    /// Note: the exact indexing varies — this produces a flat 128×128 index.
    pub fn cell_index(&self) -> usize {
        let tx = (self.x & 0xFE) as usize;
        let tz = (self.z & 0xFE) as usize;
        // 128×128 grid — each row is 128 cells
        (tx >> 1) * REGION_GRID_SIZE + (tz >> 1)
    }
}

/// A cell in the 128×128 region map.
/// Binary base: 0x88897C, 16 bytes per cell.
/// Original: struct at RegionMap[cell_index]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct RegionMapCell {
    /// +0x00: Low flags byte
    pub flags_low: u8,
    /// +0x01: High flags byte (bit 1 = has building)
    pub flags_high: u8,
    /// +0x02 .. +0x07: padding/other fields
    pub _pad1: [u8; 6],
    /// +0x08: Region ID word (low 10 bits = region index)
    pub region_id_raw: u16,
    /// +0x0A .. +0x0B: padding
    pub _pad2: [u8; 2],
    /// +0x0C: Terrain type (low nibble → terrain flags at 0x5A3038)
    pub terrain_type: u8,
    /// +0x0D .. +0x0F: padding
    pub _pad3: [u8; 3],
}

impl RegionMapCell {
    /// Extract the 10-bit region ID.
    pub fn region_id(&self) -> u16 {
        self.region_id_raw & REGION_ID_MASK
    }

    /// Check if this cell has a building.
    pub fn has_building(&self) -> bool {
        self.flags_high & CELL_HAS_BUILDING != 0
    }

    /// Get terrain type (low nibble).
    pub fn terrain_class(&self) -> u8 {
        self.terrain_type & 0x0F
    }
}

/// A single waypoint within a route segment.
/// 4 bytes per waypoint in the binary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Waypoint {
    /// Tile X coordinate
    pub tile_x: u8,
    /// Tile Z coordinate
    pub tile_z: u8,
    /// Flags
    pub flags: u8,
    /// Padding
    pub _pad: u8,
}

impl Waypoint {
    /// Convert waypoint tile to world coordinates.
    /// Binary: `world = ((tile_byte & 0xFE) + 1) << 8`
    pub fn to_world(&self) -> WorldCoord {
        TileCoord::new(self.tile_x, self.tile_z).to_world()
    }
}

/// A route segment in the segment pool.
/// Binary base: 0x93E1C1, 109 bytes per segment, 400 slots.
/// Original: RouteSegment[index]
#[derive(Debug, Clone)]
pub struct RouteSegment {
    /// +0x00: Reference count (shared between units)
    pub ref_count: i16,
    /// +0x02: Flags (bit 0=curved, bit 1=bidirectional, bit 2=persistent)
    pub flags: u8,
    /// +0x04: Start tile X
    pub start_tile_x: u8,
    /// +0x05: Start tile Z
    pub start_tile_z: u8,
    /// +0x08: End tile X
    pub end_tile_x: u8,
    /// +0x09: End tile Z
    pub end_tile_z: u8,
    /// +0x0C: Waypoints array (up to MAX_WAYPOINTS)
    pub waypoints: [Waypoint; MAX_WAYPOINTS],
    /// +0x6C: Number of valid waypoints
    pub waypoint_count: u8,
}

impl Default for RouteSegment {
    fn default() -> Self {
        Self {
            ref_count: 0,
            flags: 0,
            start_tile_x: 0,
            start_tile_z: 0,
            end_tile_x: 0,
            end_tile_z: 0,
            waypoints: [Waypoint::default(); MAX_WAYPOINTS],
            waypoint_count: 0,
        }
    }
}

impl RouteSegment {
    /// Check if this segment slot is free (ref_count <= 0).
    pub fn is_free(&self) -> bool {
        self.ref_count <= 0
    }

    /// Check if this segment matches a given start→end tile pair.
    pub fn matches(&self, start: TileCoord, end: TileCoord) -> bool {
        self.start_tile_x == start.x
            && self.start_tile_z == start.z
            && self.end_tile_x == end.x
            && self.end_tile_z == end.z
    }

    /// Check bidirectional match (also matches end→start).
    pub fn matches_bidirectional(&self, start: TileCoord, end: TileCoord) -> bool {
        self.matches(start, end)
            || (self.flags & 0x02 != 0
                && self.start_tile_x == end.x
                && self.start_tile_z == end.z
                && self.end_tile_x == start.x
                && self.end_tile_z == start.z)
    }

    /// Extract waypoint N as world coordinates.
    /// Original: ExtractWaypoint @ 0x4d8560
    pub fn get_waypoint_world(&self, index: u8) -> Option<WorldCoord> {
        if index < self.waypoint_count {
            Some(self.waypoints[index as usize].to_world())
        } else {
            None
        }
    }
}

/// A failure cache entry. Records recently-failed A* searches.
/// Binary: 10 bytes × 8 entries at 0x93E171.
#[derive(Debug, Clone, Copy, Default)]
pub struct FailureCacheEntry {
    /// +0x00: Usage count (0 = empty)
    pub usage_count: i16,
    /// +0x02: Source tile X
    pub src_tile_x: u8,
    /// +0x03: Source tile Z
    pub src_tile_z: u8,
    /// +0x06: Destination tile X
    pub dst_tile_x: u8,
    /// +0x07: Destination tile Z
    pub dst_tile_z: u8,
}

impl FailureCacheEntry {
    pub fn is_empty(&self) -> bool {
        self.usage_count == 0
    }

    pub fn matches(&self, src: TileCoord, dst: TileCoord) -> bool {
        !self.is_empty()
            && self.src_tile_x == src.x
            && self.src_tile_z == src.z
            && self.dst_tile_x == dst.x
            && self.dst_tile_z == dst.z
    }
}

/// Movement-related fields from the Object/Person struct.
/// These are the fields that the movement system reads and writes.
#[derive(Debug, Clone)]
pub struct PersonMovement {
    /// +0x0C: flags1 (0x1000=moving, 0x80=blocked, 0x80000000=needs_bridge)
    pub flags1: u32,
    /// +0x10: flags2 (0x4000000=route_through_building)
    pub flags2: u32,
    /// +0x2B: state (0x03=MOVING, 0x07=GOTO)
    pub state: u8,
    /// +0x30: unit_type (indexes speed table)
    pub unit_type: u8,
    /// +0x3D: position (X, Z)
    pub position: WorldCoord,
    /// +0x4F: target_pos
    pub target_pos: WorldCoord,
    /// +0x53: next_waypoint
    pub next_waypoint: WorldCoord,
    /// +0x57: movement_dest
    pub movement_dest: WorldCoord,
    /// +0x5D: facing_angle (11-bit, 0-2047)
    pub facing_angle: u16,
    /// +0x5F: speed (per tick)
    pub speed: u16,
    /// +0x63: segment_index (0 = direct walk)
    pub segment_index: u16,
    /// +0x65: route_byte
    pub route_byte: u8,
    /// +0x67: waypoint_idx (current position in segment)
    pub waypoint_idx: u8,
    /// +0x68: follower_count
    pub follower_count: u8,
    /// +0x6A: follower_ids (ObjectTable indices)
    pub follower_ids: [u16; MAX_FOLLOWERS],
    /// +0x76: formation_flags (bit 5 = in formation)
    pub formation_flags: u8,
    /// +0x82: formation_offsets (12 × 6 bytes)
    pub formation_offsets: [u8; MAX_FOLLOWERS * 6],
}

impl Default for PersonMovement {
    fn default() -> Self {
        Self {
            flags1: 0,
            flags2: 0,
            state: 0,
            unit_type: 0,
            position: WorldCoord::default(),
            target_pos: WorldCoord::default(),
            next_waypoint: WorldCoord::default(),
            movement_dest: WorldCoord::default(),
            facing_angle: 0,
            speed: 0,
            segment_index: 0,
            route_byte: 0,
            waypoint_idx: 0,
            follower_count: 0,
            follower_ids: [0u16; MAX_FOLLOWERS],
            formation_flags: 0,
            formation_offsets: [0u8; MAX_FOLLOWERS * 6],
        }
    }
}

impl PersonMovement {
    pub fn is_moving(&self) -> bool {
        self.flags1 & FLAG1_MOVING != 0
    }

    pub fn is_blocked(&self) -> bool {
        self.flags1 & FLAG1_BLOCKED != 0
    }

    pub fn in_formation(&self) -> bool {
        self.formation_flags & FORMATION_FLAG_BIT != 0
    }

    /// Set the movement target flags after STATE_GOTO.
    /// Original: flags1 |= 0x1000; flags1 &= ~0x80;
    pub fn set_goto_flags(&mut self) {
        self.flags1 |= FLAG1_MOVING;
        self.flags1 &= !FLAG1_BLOCKED;
    }
}

/// Cache of recently-chosen walkability adjustment targets.
/// Prevents multiple units from snapping to the same walkable tile.
/// Binary: 16 entries at 0x0095232A, counter at 0x00952326.
#[derive(Debug, Clone)]
pub struct UsedTargetsCache {
    entries: [TileCoord; MAX_USED_TARGETS],
    count: usize,
}

impl UsedTargetsCache {
    pub fn new() -> Self {
        Self {
            entries: [TileCoord::default(); MAX_USED_TARGETS],
            count: 0,
        }
    }

    /// Check if a tile was recently used as an adjusted target.
    pub fn contains(&self, tile: TileCoord) -> bool {
        self.entries[..self.count].contains(&tile)
    }

    /// Record a tile as a used target. Wraps around when full.
    pub fn record(&mut self, tile: TileCoord) {
        if self.count < MAX_USED_TARGETS {
            self.entries[self.count] = tile;
            self.count += 1;
        } else {
            // Wrap around — overwrite oldest
            let idx = self.count % MAX_USED_TARGETS;
            self.entries[idx] = tile;
            self.count += 1;
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }
}

impl Default for UsedTargetsCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_tile_conversion() {
        // World (0x0100, 0x0200) → tile ((0x0100 >> 8) & 0xFE, (0x0200 >> 8) & 0xFE)
        // = (0x00 & 0xFE, 0x02 & 0xFE) = (0, 2)
        let w = WorldCoord::new(0x0100, 0x0200);
        let t = w.to_tile();
        assert_eq!(t.x, 0);
        assert_eq!(t.z, 2);
    }

    #[test]
    fn tile_to_world_conversion() {
        // tile (0x04, 0x08) → world (((4 & 0xFE) + 1) << 8, ((8 & 0xFE) + 1) << 8)
        // = ((4+1) << 8, (8+1) << 8) = (0x0500, 0x0900)
        let t = TileCoord::new(0x04, 0x08);
        let w = t.to_world();
        assert_eq!(w.x, 0x0500);
        assert_eq!(w.z, 0x0900);
    }

    #[test]
    fn tile_roundtrip() {
        // Tile centers should roundtrip cleanly
        let t1 = TileCoord::new(0x10, 0x20);
        let w = t1.to_world();
        let t2 = w.to_tile();
        assert_eq!(t1, t2);
    }

    #[test]
    fn region_cell_region_id() {
        let mut cell = RegionMapCell::default();
        cell.region_id_raw = 0xFC23; // low 10 bits = 0x023 = 35
        assert_eq!(cell.region_id(), 35);
    }

    #[test]
    fn region_cell_has_building() {
        let mut cell = RegionMapCell::default();
        cell.flags_high = 0;
        assert!(!cell.has_building());
        cell.flags_high = CELL_HAS_BUILDING;
        assert!(cell.has_building());
    }

    #[test]
    fn segment_match() {
        let mut seg = RouteSegment::default();
        seg.ref_count = 1;
        seg.start_tile_x = 10;
        seg.start_tile_z = 20;
        seg.end_tile_x = 30;
        seg.end_tile_z = 40;

        let start = TileCoord::new(10, 20);
        let end = TileCoord::new(30, 40);
        assert!(seg.matches(start, end));
        assert!(!seg.matches(end, start));
    }

    #[test]
    fn segment_bidirectional_match() {
        let mut seg = RouteSegment::default();
        seg.ref_count = 1;
        seg.flags = 0x02; // bidirectional
        seg.start_tile_x = 10;
        seg.start_tile_z = 20;
        seg.end_tile_x = 30;
        seg.end_tile_z = 40;

        let start = TileCoord::new(10, 20);
        let end = TileCoord::new(30, 40);
        assert!(seg.matches_bidirectional(start, end));
        assert!(seg.matches_bidirectional(end, start)); // reversed
    }

    #[test]
    fn failure_cache_entry() {
        let entry = FailureCacheEntry {
            usage_count: 1,
            src_tile_x: 5,
            src_tile_z: 10,
            dst_tile_x: 50,
            dst_tile_z: 60,
        };
        assert!(!entry.is_empty());
        assert!(entry.matches(TileCoord::new(5, 10), TileCoord::new(50, 60)));
        assert!(!entry.matches(TileCoord::new(50, 60), TileCoord::new(5, 10)));
    }

    #[test]
    fn person_movement_flags() {
        let mut person = PersonMovement::default();
        assert!(!person.is_moving());
        assert!(!person.is_blocked());

        person.set_goto_flags();
        assert!(person.is_moving());
        assert!(!person.is_blocked());

        // If blocked was set before, set_goto_flags clears it
        person.flags1 |= FLAG1_BLOCKED;
        assert!(person.is_blocked());
        person.set_goto_flags();
        assert!(!person.is_blocked());
    }

    #[test]
    fn waypoint_to_world() {
        let wp = Waypoint {
            tile_x: 0x10,
            tile_z: 0x20,
            flags: 0,
            _pad: 0,
        };
        let w = wp.to_world();
        // ((0x10 & 0xFE) + 1) << 8 = (0x10 + 1) << 8 = 0x1100
        // ((0x20 & 0xFE) + 1) << 8 = (0x20 + 1) << 8 = 0x2100
        assert_eq!(w.x, 0x1100);
        assert_eq!(w.z, 0x2100);
    }
}

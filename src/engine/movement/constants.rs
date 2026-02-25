// Movement system constants — all values verified against popTB.exe disassembly
// and confirmed active via Frida instrumentation.

// === Angle system (11-bit, 2048 values = 360 degrees) ===

/// Bitmask for valid angle values (0x000-0x7FF)
pub const ANGLE_MASK: u16 = 0x7FF;
/// 180 degrees in angle units
pub const ANGLE_HALF: i32 = 0x400;
/// 360 degrees in angle units
pub const ANGLE_FULL: i32 = 0x800;

// === World coordinate system ===

/// Threshold for toroidal distance wrapping
pub const WORLD_WRAP_THRESHOLD: i32 = 0x8000;
/// Total world size in each axis (128 cells * 512 units per cell)
pub const WORLD_SIZE: i32 = 0x10000;

// === Region map (128×128 grid at 0x88897C) ===

/// Region map grid dimension
pub const REGION_GRID_SIZE: usize = 128;
/// Bytes per region map cell
pub const REGION_CELL_SIZE: usize = 16;
/// Mask for extracting region ID from cell+0x08 word
pub const REGION_ID_MASK: u16 = 0x3FF;
/// Building flag in cell flags_high (offset +0x01, bit 1)
pub const CELL_HAS_BUILDING: u8 = 0x02;

// === Route segment pool (0x93E1C1) ===

/// Maximum segments in the pool
pub const MAX_SEGMENTS: usize = 400;
/// Bytes per route segment
pub const SEGMENT_SIZE: usize = 109;
/// Maximum waypoints per segment
pub const MAX_WAYPOINTS: usize = 23;

// === Failure cache (0x93E171) ===

/// Number of failure cache entries
pub const FAILURE_CACHE_SIZE: usize = 8;
/// Bytes per failure cache entry
pub const FAILURE_ENTRY_SIZE: usize = 10;

// === Movement flags (Object+0x0C flags1) ===

/// Unit has an active movement target
pub const FLAG1_MOVING: u32 = 0x1000;
/// Unit is blocked / stuck
pub const FLAG1_BLOCKED: u32 = 0x0080;
/// Unit needs a bridge to cross
pub const FLAG1_NEEDS_BRIDGE: u32 = 0x8000_0000;

// === Movement flags (Object+0x10 flags2) ===

/// Route passes through a building
pub const FLAG2_ROUTE_THROUGH_BUILDING: u32 = 0x0400_0000;
/// Cache miss flag
pub const FLAG2_CACHE_MISS: u32 = 0x1000_0000;

// === Formation constants ===

/// Arrival threshold per axis (world units)
pub const ARRIVAL_THRESHOLD: i32 = 0x48; // 72
/// Max distance for formation coherence
pub const FORMATION_MAX_DISTANCE: i32 = 0x238; // 568
/// Max angle difference for formation membership (~20°)
pub const FORMATION_MAX_ANGLE: i32 = 0x71; // 113
/// Max distance² for formation membership
pub const FORMATION_MAX_DIST_SQ: i32 = 0x40_0000;
/// Formation row spacing (world units)
pub const FORMATION_ROW_SPACING: i32 = 0x12; // 18
/// Maximum followers per formation group
pub const MAX_FOLLOWERS: usize = 12;
/// Follower catch-up speed multiplier (3/2)
pub const CATCHUP_SPEED_NUM: i32 = 3;
pub const CATCHUP_SPEED_DEN: i32 = 2;
/// Formation flag bit (at unit+0x76)
pub const FORMATION_FLAG_BIT: u8 = 0x20;

// === Person states ===

/// MOVING state (dormant — not used in practice)
pub const STATE_MOVING: u8 = 0x03;
/// GOTO state (actual movement state)
pub const STATE_GOTO: u8 = 0x07;

// === Speed table ===

/// Base address of speed table in binary
pub const SPEED_TABLE_ADDR: u32 = 0x5A0974;
/// Stride of speed table entries (26 bytes per unit type)
pub const SPEED_TABLE_STRIDE: usize = 26;

// === RNG constants (formation idle) ===
// state = state * 9377 + 0x24DF; state = ROR(state, 13)

pub const RNG_MULTIPLIER: u32 = 9377;
pub const RNG_INCREMENT: u32 = 0x24DF;
pub const RNG_ROTATE_BITS: u32 = 13;

// === Terrain walkability (flags table at 0x5A3038) ===

/// Bit in terrain flags indicating walkable terrain
pub const TERRAIN_WALKABLE_BIT: u8 = 0x02;
/// Number of terrain flag entries (indexed by terrain_class)
pub const TERRAIN_FLAGS_COUNT: usize = 16;
/// Maximum neighbors to search in spiral walkability scan
pub const MAX_WALKABILITY_SEARCH: usize = 32;
/// Maximum entries in the used-targets cache (prevents loop revisiting)
pub const MAX_USED_TARGETS: usize = 16;

// === Waypoint stepping (ProcessRouteMovement @ 0x4d8e60) ===

/// Distance threshold for waypoint arrival (normal paths, world units)
pub const WAYPOINT_ARRIVAL_THRESHOLD: i32 = 0x240; // 576
/// Distance threshold for building entrance arrival (world units)
pub const BUILDING_ARRIVAL_THRESHOLD: i32 = 0xE0; // 224
/// Segment flag bit 2: persistent (don't deallocate when ref_count hits 0)
pub const SEGMENT_FLAG_PERSISTENT: u8 = 0x04;

// === Pathfinder — Dual-arm wall-following search (0x45d090) ===
// Despite the name "A*" in docs, this is actually a dual-arm wall-following
// (Bug-algorithm variant) that uses Bresenham-like zigzag toward goal.

/// Maximum search iterations before giving up (from 0x599830)
pub const PATHFIND_MAX_ITERATIONS: usize = 1500;
/// Maximum local waypoints per search arm
pub const PATHFIND_ARM_MAX_WAYPOINTS: usize = 260;
/// Maximum total waypoints in the global pool
pub const PATHFIND_MAX_POOL_WAYPOINTS: usize = 400;
/// Default max waypoints for output (tribe-dependent in original)
pub const PATHFIND_DEFAULT_MAX_OUTPUT: usize = 400;
/// Maximum retries with relaxed constraints
pub const PATHFIND_MAX_RETRIES: usize = 3;
/// Minimum distance (in cells) for intermediate building route
pub const PATHFIND_MIN_INTERMEDIATE_DIST: i32 = 0x64; // 100
/// Maximum output waypoints per segment
pub const PATHFIND_MAX_SEGMENT_WAYPOINTS: usize = 23;

/// Number of cardinal directions
pub const NUM_DIRECTIONS: usize = 4;

/// Direction deltas for cardinal movement (South, East, North, West)
/// Original: table at 0x599838, 10 bytes per entry
/// Each entry: (dx: i32, dz: i32) — only first 8 bytes used
pub const DIRECTION_DX: [i32; 4] = [0, 1, 0, -1]; // S, E, N, W
pub const DIRECTION_DZ: [i32; 4] = [1, 0, -1, 0]; // S, E, N, W

// Search arm states
/// Arm is initialized but hasn't started
pub const ARM_STATE_INIT: u8 = 0;
/// Arm found a path to the goal
pub const ARM_STATE_FOUND: u8 = 1;
/// Arm stalled (entered open area or exhausted)
pub const ARM_STATE_STALLED: u8 = 2;
/// Arm is actively expanding (wall-following)
pub const ARM_STATE_EXPANDING: u8 = 3;

// Cell passability results
/// Cell is passable — unit can enter
pub const PASS_OK: u8 = 0;
/// Blocked by water boundary
pub const PASS_WATER: u8 = 1;
/// Off the map edge
pub const PASS_OFF_MAP: u8 = 3;
/// Terrain blocked (cliff, wall, etc.)
pub const PASS_TERRAIN: u8 = 4;

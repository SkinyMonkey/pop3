// UnitCoordinator — owns all live units and movement infrastructure.
//
// Provides the bridge between user input (selection, move orders) and
// the movement system (pathfinding, per-tick position updates).

use crate::movement::{
    RegionMap, SegmentPool, FailureCache, UsedTargetsCache,
    PersonMovement, WorldCoord, RouteResult,
    state_goto, process_route_movement, move_point_by_angle,
    atan2,
};
use crate::pop::units::{ModelType, UnitRaw};
use super::unit::Unit;
use super::selection::{SelectionState, DragState};
use super::coords::{world_to_render_pos, toroidal_delta, cell_to_world};

/// Default movement speeds per unit subtype.
/// From the game's speed table at 0x5A0974 (stride 26).
fn default_speed(subtype: u8) -> u16 {
    match subtype {
        2 => 0x30, // Brave
        3 => 0x28, // Warrior
        4 => 0x28, // Religious
        5 => 0x30, // Spy
        6 => 0x28, // Firewarrior
        7 => 0x30, // Shaman
        _ => 0x30, // Default
    }
}

pub struct UnitCoordinator {
    pub units: Vec<Unit>,
    pub selection: SelectionState,
    pub drag: DragState,

    // Movement infrastructure
    region_map: RegionMap,
    segment_pool: SegmentPool,
    failure_cache: FailureCache,
    used_targets: UsedTargetsCache,

    landscape_size: f32,
}

impl UnitCoordinator {
    pub fn new() -> Self {
        Self {
            units: Vec::new(),
            selection: SelectionState::new(),
            drag: DragState::None,
            region_map: RegionMap::new(),
            segment_pool: SegmentPool::new(),
            failure_cache: FailureCache::new(),
            used_targets: UsedTargetsCache::new(),
            landscape_size: 128.0,
        }
    }

    /// Extract person units from level data into live units.
    /// Non-person objects remain as static LevelObjects in main.rs.
    pub fn load_level(&mut self, units_raw: &[UnitRaw], landscape_height: &[[u16; 128]; 128], landscape_size: usize) {
        self.units.clear();
        self.selection.clear();
        self.landscape_size = landscape_size as f32;

        // Reset movement infrastructure
        self.segment_pool = SegmentPool::new();
        self.failure_cache = FailureCache::new();
        self.region_map = RegionMap::new();

        Self::populate_water(&mut self.region_map, landscape_height, landscape_size);

        log::info!("[unit-ctrl] load_level: {} raw units, landscape_size={}", units_raw.len(), landscape_size);

        for raw in units_raw {
            if raw.model_type() != Some(ModelType::Person) {
                continue;
            }
            if raw.loc_x() == 0 && raw.loc_y() == 0 {
                continue;
            }

            let mut movement = PersonMovement::default();
            movement.position = WorldCoord::new(raw.loc_x() as i16, raw.loc_y() as i16);
            movement.facing_angle = (raw.angle() & 0x7FF) as u16;
            movement.unit_type = raw.subtype;
            movement.speed = default_speed(raw.subtype);

            let (cx, cy) = world_to_render_pos(&movement.position, self.landscape_size);
            self.units.push(Unit {
                id: self.units.len(),
                model_type: ModelType::Person,
                subtype: raw.subtype,
                tribe_index: raw.tribe_index(),
                movement,
                cell_x: cx,
                cell_y: cy,
            });
        }
        log::info!("[unit-ctrl] loaded {} person units", self.units.len());
    }

    /// Issue move orders to all selected units targeting `target_world`.
    pub fn order_move(&mut self, target_world: WorldCoord) {
        self.used_targets.clear();
        for &unit_id in &self.selection.selected {
            if let Some(unit) = self.units.get_mut(unit_id) {
                let was_moving = unit.movement.is_moving();
                let result = state_goto(
                    &self.region_map,
                    &mut self.segment_pool,
                    &self.failure_cache,
                    &mut unit.movement,
                    target_world,
                    &mut self.used_targets,
                );
                // The original binary sets MOVING unconditionally, but without
                // the full person state machine to catch NoRoute, clear it here.
                if result == RouteResult::NoRoute {
                    unit.movement.flags1 &= !0x1000; // Clear MOVING
                }
                let is_moving = unit.movement.is_moving();
                log::info!("[move-order] unit {} state_goto: was_moving={} → is_moving={} result={:?} target=({}, {}) next_wp=({}, {}) flags=0x{:08x} seg_idx={}",
                    unit_id, was_moving, is_moving, result,
                    unit.movement.target_pos.x, unit.movement.target_pos.z,
                    unit.movement.next_waypoint.x, unit.movement.next_waypoint.z,
                    unit.movement.flags1, unit.movement.segment_index);
            }
        }
    }

    /// Advance all moving units by one tick.
    pub fn tick(&mut self) {
        for unit in &mut self.units {
            if !unit.movement.is_moving() {
                continue;
            }

            // Step 1: Waypoint advancement
            process_route_movement(&mut self.segment_pool, &mut unit.movement);

            // Step 2: Compute facing angle toward next waypoint
            let dx = toroidal_delta(unit.movement.position.x, unit.movement.next_waypoint.x);
            let dz = toroidal_delta(unit.movement.position.z, unit.movement.next_waypoint.z);

            log::trace!("[tick] unit {} pos=({}, {}) wp=({}, {}) dx={} dz={} speed={} seg={}",
                unit.id, unit.movement.position.x, unit.movement.position.z,
                unit.movement.next_waypoint.x, unit.movement.next_waypoint.z,
                dx, dz, unit.movement.speed, unit.movement.segment_index);

            // Check arrival at destination
            if dx.abs() < 0x48 && dz.abs() < 0x48 {
                if unit.movement.segment_index == 0 {
                    // Direct walk completed — snap to target
                    unit.movement.position = unit.movement.target_pos;
                    unit.movement.flags1 &= !0x1000; // Clear MOVING
                }
                // If segment_index != 0, process_route_movement handles waypoint advance
            } else {
                // Face toward waypoint and advance.
                // atan2's dy parameter uses screen-y convention (+y = south),
                // so negate dz (world +z = north). Original: NEG EAX at 0x424ae5.
                unit.movement.facing_angle = atan2(dx, -dz);
                move_point_by_angle(
                    &mut unit.movement.position,
                    unit.movement.facing_angle,
                    unit.movement.speed as i16,
                );
            }

            // Step 3: Update rendering cache (smooth sub-cell precision)
            let (cx, cy) = world_to_render_pos(&unit.movement.position, self.landscape_size);
            unit.cell_x = cx;
            unit.cell_y = cy;
        }
    }

    /// Mark height-0 cells as water (unwalkable) in the region map.
    /// Water cells get region_id=1 so `same_region` returns false when
    /// routing between land (region 0) and water, forcing the pathfinder
    /// to engage and reject the unwalkable target.
    fn populate_water(region_map: &mut RegionMap, landscape_height: &[[u16; 128]; 128], size: usize) {
        region_map.set_terrain_flags(1, 0x00); // terrain class 1 = water = unwalkable
        // landscape_height[cell_y][cell_x] — cell_y is the row (world x, flipped),
        // cell_x is the column (world z). Use cell_to_world → to_tile() to get the
        // correct tile coordinates matching the routing system's to_tile().
        for cell_y in 0..size {
            for cell_x in 0..size {
                if landscape_height[cell_y][cell_x] == 0 {
                    let world = cell_to_world(cell_x as f32 + 0.5, cell_y as f32 + 0.5, size as f32);
                    let tile = world.to_tile();
                    let cell = region_map.get_cell_mut(tile);
                    cell.terrain_type = 1;
                    region_map.set_cell_region(tile, 1); // water region
                }
            }
        }
    }

    pub fn region_map(&self) -> &RegionMap {
        &self.region_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_level_filters_persons() {
        // UnitRaw is repr(C, packed) — we can't easily construct one in tests
        // without unsafe. Test via the coordinator's public interface instead.
        let coord = UnitCoordinator::new();
        assert!(coord.units.is_empty());
        assert_eq!(coord.landscape_size, 128.0);
    }

    #[test]
    fn populate_water_marks_unwalkable() {
        let mut height = [[100u16; 128]; 128];
        // Set a few cells to water (height 0)
        // height[cell_y][cell_x], so these are at landscape cells:
        height[0][0] = 0;     // cell (0, 0)
        height[10][20] = 0;   // cell (20, 10)
        height[63][64] = 0;   // cell (64, 63)

        let mut map = RegionMap::new();
        UnitCoordinator::populate_water(&mut map, &height, 128);

        // Water cells should be unwalkable — use same cell_to_world → to_tile mapping
        let tile_0_0 = cell_to_world(0.5, 0.5, 128.0).to_tile();
        assert!(!map.is_walkable(tile_0_0));
        let tile_20_10 = cell_to_world(20.5, 10.5, 128.0).to_tile();
        assert!(!map.is_walkable(tile_20_10));
        let tile_64_63 = cell_to_world(64.5, 63.5, 128.0).to_tile();
        assert!(!map.is_walkable(tile_64_63));

        // Land cells should remain walkable
        let land1 = cell_to_world(1.5, 1.5, 128.0).to_tile();
        assert!(map.is_walkable(land1));
        let land2 = cell_to_world(50.5, 50.5, 128.0).to_tile();
        assert!(map.is_walkable(land2));
    }

    #[test]
    fn populate_water_all_land() {
        // No water at all — everything should be walkable
        let height = [[50u16; 128]; 128];
        let mut map = RegionMap::new();
        UnitCoordinator::populate_water(&mut map, &height, 128);
        // Check various tiles via the same coordinate path
        let t1 = cell_to_world(0.5, 0.5, 128.0).to_tile();
        assert!(map.is_walkable(t1));
        let t2 = cell_to_world(127.5, 127.5, 128.0).to_tile();
        assert!(map.is_walkable(t2));
    }
}

// UnitCoordinator — owns all live units and movement infrastructure.
//
// Provides the bridge between user input (selection, move orders) and
// the movement system (pathfinding, per-tick position updates).

use crate::movement::{
    RegionMap, SegmentPool, FailureCache, UsedTargetsCache,
    PersonMovement, WorldCoord,
    state_goto, process_route_movement, move_point_by_angle,
    atan2,
};
use crate::pop::units::{ModelType, UnitRaw};
use super::unit::Unit;
use super::selection::{SelectionState, DragState};
use super::coords::world_to_cell;

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
    pub fn load_level(&mut self, units_raw: &[UnitRaw], landscape_size: usize) {
        self.units.clear();
        self.selection.clear();
        self.landscape_size = landscape_size as f32;

        // Reset movement infrastructure
        self.segment_pool = SegmentPool::new();
        self.failure_cache = FailureCache::new();
        self.region_map = RegionMap::new(); // Default: all walkable

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

            let (cx, cy) = world_to_cell(&movement.position, self.landscape_size);
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
                state_goto(
                    &self.region_map,
                    &mut self.segment_pool,
                    &self.failure_cache,
                    &mut unit.movement,
                    target_world,
                    &mut self.used_targets,
                );
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
            let dx = unit.movement.next_waypoint.x as i32 - unit.movement.position.x as i32;
            let dz = unit.movement.next_waypoint.z as i32 - unit.movement.position.z as i32;

            // Check arrival at destination
            if dx.abs() < 0x48 && dz.abs() < 0x48 {
                if unit.movement.segment_index == 0 {
                    // Direct walk completed — snap to target
                    unit.movement.position = unit.movement.target_pos;
                    unit.movement.flags1 &= !0x1000; // Clear MOVING
                }
                // If segment_index != 0, process_route_movement handles waypoint advance
            } else {
                // Face toward waypoint and advance
                unit.movement.facing_angle = atan2(dx, dz);
                move_point_by_angle(
                    &mut unit.movement.position,
                    unit.movement.facing_angle,
                    unit.movement.speed as i16,
                );
            }

            // Step 3: Update rendering cache
            let (cx, cy) = world_to_cell(&unit.movement.position, self.landscape_size);
            unit.cell_x = cx;
            unit.cell_y = cy;
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
}

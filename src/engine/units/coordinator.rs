// UnitCoordinator — owns all live units and movement infrastructure.
//
// Provides the bridge between user input (selection, move orders) and
// the movement system (pathfinding, per-tick position updates).

use crate::engine::state::rng::GameRng;
use crate::engine::movement::{
    RegionMap, SegmentPool, FailureCache, UsedTargetsCache,
    PersonMovement, WorldCoord, RouteResult,
    state_goto, process_route_movement, move_point_by_angle,
    atan2,
};
use crate::data::units::{ModelType, UnitRaw};
use super::unit::Unit;
use super::person_state::{
    PersonState, person_type_defaults, enter_state, tick_state, TickResult,
    calculate_melee_damage, apply_damage,
    COMBAT_DETECT_RANGE, COMBAT_MELEE_RANGE, COMBAT_ATTACK_INTERVAL,
};
use super::selection::{SelectionState, DragState};
use super::coords::{world_to_render_pos, toroidal_delta, cell_to_world};

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

    // State machine RNG (same LCG as original binary)
    pub rng: GameRng,
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
            rng: GameRng::new(0x1234),
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

            let defaults = person_type_defaults(raw.subtype);
            let mut movement = PersonMovement::default();
            movement.position = WorldCoord::new(raw.loc_x() as i16, raw.loc_y() as i16);
            movement.facing_angle = (raw.angle() & 0x7FF) as u16;
            movement.unit_type = raw.subtype;
            movement.speed = defaults.speed;

            let (cx, cy) = world_to_render_pos(&movement.position, self.landscape_size);
            self.units.push(Unit {
                id: self.units.len(),
                model_type: ModelType::Person,
                subtype: raw.subtype,
                tribe_index: raw.tribe_index(),
                movement,
                cell_x: cx,
                cell_y: cy,
                state: PersonState::Idle,
                prev_state: PersonState::Idle,
                state_timer: 0,
                state_counter: 0,
                health: defaults.max_health,
                max_health: defaults.max_health,
                target_unit: None,
                attacker_unit: None,
                alive: true,
            });
            // Initialize idle state with a random timer (matches Person_Init calling Person_SetState)
            let idx = self.units.len() - 1;
            enter_state(&mut self.units[idx], PersonState::Idle, &mut self.rng);
        }
        log::info!("[unit-ctrl] loaded {} person units", self.units.len());
        for unit in &self.units {
            log::debug!("[unit-ctrl] unit {} sub={} tribe={} state={:?} timer={} pos=({}, {}) hp={}/{}",
                unit.id, unit.subtype, unit.tribe_index, unit.state, unit.state_timer,
                unit.movement.position.x, unit.movement.position.z,
                unit.health, unit.max_health);
        }
    }

    /// Issue move orders to all selected units targeting `target_world`.
    /// Transitions units into GoToPoint state and calls state_goto.
    pub fn order_move(&mut self, target_world: WorldCoord) {
        self.used_targets.clear();
        for &unit_id in &self.selection.selected {
            if let Some(unit) = self.units.get_mut(unit_id) {
                if !unit.alive { continue; }
                let result = state_goto(
                    &self.region_map,
                    &mut self.segment_pool,
                    &self.failure_cache,
                    &mut unit.movement,
                    target_world,
                    &mut self.used_targets,
                );
                if result == RouteResult::NoRoute {
                    unit.movement.flags1 &= !0x1000;
                } else {
                    unit.state = PersonState::GoToPoint;
                    unit.target_unit = None; // Cancel combat
                    // Restore subtype speed (enter_idle sets it to 0)
                    unit.movement.speed = person_type_defaults(unit.subtype).speed;
                }
                log::info!("[move-order] unit {} result={:?} state={:?} target=({}, {})",
                    unit_id, result, unit.state,
                    unit.movement.target_pos.x, unit.movement.target_pos.z);
            }
        }
    }

    /// Advance all units by one tick: state machine + movement + combat + drowning.
    pub fn tick(&mut self) {
        let unit_count = self.units.len();

        // Phase 1: State machine tick + movement for each unit
        for i in 0..unit_count {
            let unit = &mut self.units[i];
            if !unit.alive { continue; }

            // Run state machine tick
            let result = tick_state(unit);
            if let TickResult::Transition(new_state) = result {
                enter_state(unit, new_state, &mut self.rng);
            }

            // Process movement for moving states
            if unit.movement.is_moving() {
                Self::advance_movement(&mut self.segment_pool, unit, self.landscape_size);
            }

            // Update rendering cache
            let (cx, cy) = world_to_render_pos(&unit.movement.position, self.landscape_size);
            unit.cell_x = cx;
            unit.cell_y = cy;
        }

        // Phase 2: Drowning detection
        for i in 0..unit_count {
            let unit = &self.units[i];
            if !unit.alive { continue; }
            if unit.state == PersonState::Drowning || unit.state == PersonState::Dead { continue; }

            let tile = unit.movement.position.to_tile();
            if !self.region_map.is_walkable(tile) {
                let unit = &mut self.units[i];
                enter_state(unit, PersonState::Drowning, &mut self.rng);
            }
        }

        // Phase 3: Combat detection — idle/wander units auto-engage nearby enemies
        self.detect_combat();

        // Phase 4: Process combat damage for fighting units
        self.process_combat();
    }

    /// Move a unit one step along its path (waypoint advancement + position update).
    fn advance_movement(segment_pool: &mut SegmentPool, unit: &mut Unit, _landscape_size: f32) {
        // Waypoint advancement for pathfind-routed movement
        if unit.state == PersonState::GoToPoint || unit.state == PersonState::GoToMarker
            || unit.state == PersonState::Moving
        {
            process_route_movement(segment_pool, &mut unit.movement);
        }

        // Compute facing angle toward next waypoint (for routed movement)
        // or use existing facing_angle (for wander/flee)
        if unit.state == PersonState::GoToPoint || unit.state == PersonState::GoToMarker
            || unit.state == PersonState::Moving
        {
            let dx = toroidal_delta(unit.movement.position.x, unit.movement.next_waypoint.x);
            let dz = toroidal_delta(unit.movement.position.z, unit.movement.next_waypoint.z);

            // Check arrival at destination
            if dx.abs() < 0x48 && dz.abs() < 0x48 {
                if unit.movement.segment_index == 0 {
                    unit.movement.position = unit.movement.target_pos;
                    unit.movement.flags1 &= !0x1000; // Clear MOVING
                }
                return;
            }
            unit.movement.facing_angle = atan2(dx, -dz);
        }

        // Advance position by speed in facing direction
        move_point_by_angle(
            &mut unit.movement.position,
            unit.movement.facing_angle,
            unit.movement.speed as i16,
        );
    }

    /// Detect nearby enemies and enter combat for idle/wandering units.
    fn detect_combat(&mut self) {
        // Collect (unit_index, target_index) pairs to avoid borrow issues
        let mut engagements: Vec<(usize, usize)> = Vec::new();

        for i in 0..self.units.len() {
            let unit = &self.units[i];
            if !unit.alive { continue; }
            // Only idle/wandering units auto-engage
            if unit.state != PersonState::Idle && unit.state != PersonState::Wander { continue; }

            let mut best_dist = COMBAT_DETECT_RANGE as i32 + 1;
            let mut best_target: Option<usize> = None;

            for j in 0..self.units.len() {
                if i == j { continue; }
                let other = &self.units[j];
                if !other.alive { continue; }
                if other.tribe_index == unit.tribe_index { continue; } // Same tribe
                if other.state == PersonState::Dead { continue; }

                let dx = toroidal_delta(unit.movement.position.x, other.movement.position.x) as i32;
                let dz = toroidal_delta(unit.movement.position.z, other.movement.position.z) as i32;
                let dist = dx.abs() + dz.abs(); // Manhattan distance (fast approximation)

                if dist < best_dist {
                    best_dist = dist;
                    best_target = Some(j);
                }
            }

            if let Some(target) = best_target {
                engagements.push((i, target));
            }
        }

        // Apply engagements
        for (attacker_idx, target_idx) in engagements {
            let target_id = self.units[target_idx].id;
            let target_pos = self.units[target_idx].movement.position;
            let unit = &mut self.units[attacker_idx];
            unit.target_unit = Some(target_id);
            enter_state(unit, PersonState::Fighting, &mut self.rng);

            // Face toward target
            let dx = toroidal_delta(unit.movement.position.x, target_pos.x);
            let dz = toroidal_delta(unit.movement.position.z, target_pos.z);
            unit.movement.facing_angle = atan2(dx, -dz);
        }
    }

    /// Process melee damage for fighting units.
    fn process_combat(&mut self) {
        // Collect damage events: (target_index, damage, attacker_tribe)
        let mut damage_events: Vec<(usize, u16, u8)> = Vec::new();

        for i in 0..self.units.len() {
            let unit = &self.units[i];
            if !unit.alive || unit.state != PersonState::Fighting { continue; }

            let target_id = match unit.target_unit {
                Some(id) => id,
                None => continue,
            };

            // Find target by ID
            let target_idx = match self.units.iter().position(|u| u.id == target_id) {
                Some(idx) => idx,
                None => continue,
            };

            let target = &self.units[target_idx];
            if !target.alive || target.health == 0 {
                continue; // Target already dead, tick_fighting will transition to Idle
            }

            // Check melee range
            let dx = toroidal_delta(unit.movement.position.x, target.movement.position.x) as i32;
            let dz = toroidal_delta(unit.movement.position.z, target.movement.position.z) as i32;
            let dist = dx.abs() + dz.abs();

            if dist <= COMBAT_MELEE_RANGE as i32 {
                // Attack rate: only deal damage every COMBAT_ATTACK_INTERVAL ticks
                // Use state_timer as attack cooldown
                if self.units[i].state_timer == 0 {
                    let damage = calculate_melee_damage(&self.units[i]);
                    damage_events.push((target_idx, damage, self.units[i].tribe_index));
                    self.units[i].state_timer = COMBAT_ATTACK_INTERVAL;
                } else {
                    self.units[i].state_timer -= 1;
                }
            }
            // If out of melee range but within detect range, unit stays in Fighting
            // (could add chase behavior later)
        }

        // Apply damage
        for (target_idx, damage, _attacker_tribe) in damage_events {
            let target = &mut self.units[target_idx];
            apply_damage(target, damage);
            if target.health == 0 {
                enter_state(target, PersonState::Dead, &mut self.rng);
            }
        }

        // Clear target for units whose target died
        for i in 0..self.units.len() {
            if self.units[i].state != PersonState::Fighting { continue; }
            if let Some(target_id) = self.units[i].target_unit {
                if let Some(target) = self.units.iter().find(|u| u.id == target_id) {
                    if !target.alive || target.state == PersonState::Dead {
                        self.units[i].target_unit = None;
                        // tick_fighting will transition to Idle on next tick
                    }
                }
            }
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

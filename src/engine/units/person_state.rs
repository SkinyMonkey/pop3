// Person state machine — faithful to Person_SetState (0x004fd5d0).
//
// State values are stored at object offset 0x2C in the original binary.
// All 44 values are defined for binary compatibility, but only core states
// (Idle, Moving, Wander, GoToPoint, Fighting, Fleeing, Drowning, Dead)
// have real implementations in this phase.

use crate::engine::state::rng::GameRng;
use super::unit::Unit;

/// All person states from the original binary's Person_SetState switch.
/// Values match offset 0x2C exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PersonState {
    Idle              = 0x01,
    Dying             = 0x02,
    Moving            = 0x03,
    Wander            = 0x04,
    GoToPoint         = 0x05,
    FollowPath        = 0x06,
    GoToMarker        = 0x07,
    WaitForPath       = 0x08,
    WaitAtMarker      = 0x09,
    EnterBuilding     = 0x0A,
    InsideBuilding    = 0x0B,
    InsideTraining    = 0x0C,
    Building          = 0x0D,
    InTraining        = 0x0E,
    WaitOutside       = 0x0F,
    Training          = 0x10,
    Housing           = 0x11,
    // 0x12 unused
    Gathering         = 0x13,
    // 0x14 unused
    GatheringWood     = 0x15,
    CarryingWood      = 0x16,
    Drowning          = 0x17,
    Dead              = 0x18,
    Fighting          = 0x19,
    Fleeing           = 0x1A,
    Spawning          = 0x1B,
    BeingSacrificed   = 0x1C,
    InShield          = 0x1D,
    InShieldIdle      = 0x1E,
    Preaching         = 0x1F,
    SitDown           = 0x20,
    BeingConverted    = 0x21,
    WaitingAfterConvert = 0x22,
    WaitingForBoat    = 0x23,
    Placeholder       = 0x24,
    GetOffBoat        = 0x25,
    WaitingInWater    = 0x26,
    EnteringVehicle   = 0x27,
    ExitingVehicle    = 0x28,
    Celebrating       = 0x29,
    Teleporting       = 0x2A,
    InternalState     = 0x2B,
    WaitingAtReincPillar = 0x2C,
}

impl Default for PersonState {
    fn default() -> Self {
        PersonState::Idle
    }
}

/// Default stats per person subtype.
/// From the Unit Type Data Table at 0x0059FE44 (stride 0x32).
pub struct PersonTypeDefaults {
    pub max_health: u16,
    pub speed: u16,
    pub fight_damage: u16,
}

/// Returns default stats for a given person subtype.
/// Max health values extracted from binary at 0x0059FE50 + subtype * 0x32.
/// Speed values from 0x5A0974 (stride 26).
pub fn person_type_defaults(subtype: u8) -> PersonTypeDefaults {
    match subtype {
        1 => PersonTypeDefaults { max_health: 32,   speed: 0x30, fight_damage: 64  }, // Wild
        2 => PersonTypeDefaults { max_health: 1400, speed: 0x30, fight_damage: 200 }, // Brave
        3 => PersonTypeDefaults { max_health: 1800, speed: 0x28, fight_damage: 400 }, // Warrior
        4 => PersonTypeDefaults { max_health: 1400, speed: 0x28, fight_damage: 150 }, // Religious
        5 => PersonTypeDefaults { max_health: 1400, speed: 0x30, fight_damage: 200 }, // Spy
        6 => PersonTypeDefaults { max_health: 1200, speed: 0x28, fight_damage: 500 }, // SuperWarrior
        7 => PersonTypeDefaults { max_health: 900,  speed: 0x30, fight_damage: 300 }, // Shaman
        8 => PersonTypeDefaults { max_health: 2000, speed: 0x30, fight_damage: 600 }, // Angel of Death
        _ => PersonTypeDefaults { max_health: 200,  speed: 0x30, fight_damage: 100 }, // Fallback
    }
}

// --- State entry ---

/// Enter a new state, saving the previous state and running entry logic.
/// Mirrors the preamble + switch of Person_SetState (0x004fd5d0).
pub fn enter_state(unit: &mut Unit, new_state: PersonState, rng: &mut GameRng) {
    log::debug!("[state] unit {} {:?} → {:?}", unit.id, unit.state, new_state);
    unit.prev_state = unit.state;
    unit.state = new_state;
    unit.state_counter = 0;

    // Common flag clearing (matches original's preamble):
    // flags1 &= 0xFCDEFDDD — clears MOVING, BLOCKED, and various control bits
    unit.movement.flags1 &= 0xFCDE_FDDD;

    match new_state {
        PersonState::Idle => enter_idle(unit, rng),
        PersonState::Wander => enter_wander(unit, rng),
        PersonState::Moving => { /* movement system handles entry */ }
        PersonState::GoToPoint | PersonState::GoToMarker => { /* state_goto called separately */ }
        PersonState::Fighting => enter_fighting(unit),
        PersonState::Fleeing => enter_fleeing(unit, rng),
        PersonState::Drowning => enter_drowning(unit),
        PersonState::Dead => enter_dead(unit, rng),
        _ => { /* Unimplemented states — no-op */ }
    }
}

/// Idle: speed=0, random timer 50-100 ticks.
/// Original: case '\x01' in Person_SetState.
fn enter_idle(unit: &mut Unit, rng: &mut GameRng) {
    unit.movement.speed = 0;
    unit.state_timer = (rng.next() % 50 + 50) as u16;
}

/// Wander sub-phases stored in `state_counter`.
/// Original: Person_ProcessIdleWanderState uses phase byte at +0x2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WanderPhase {
    /// Walking in random direction (32-63 ticks).
    Walking  = 0,
    /// Paused/idle, looking around (64-127 ticks).
    Pausing  = 1,
    /// Second walking phase (32-63 ticks).
    Walking2 = 2,
    /// Water escape — pathfind away from water, or idle (32-63 ticks).
    WaterEscape = 3,
}

/// Wander: start in Walking phase with random direction.
/// Original: case '\x04' in Person_SetState.
fn enter_wander(unit: &mut Unit, rng: &mut GameRng) {
    unit.state_counter = WanderPhase::Walking as u8;
    enter_wander_walking(unit, rng);
}

/// Set up a walking sub-phase: random direction, timer 32-63 ticks, MOVING flag.
fn enter_wander_walking(unit: &mut Unit, rng: &mut GameRng) {
    unit.state_timer = ((rng.next() & 0x1F) + 0x20) as u16; // 32-63
    let angle = (rng.next() & 0x7FF) as u16;
    unit.movement.facing_angle = angle;
    unit.movement.flags1 |= 0x1080;
    let defaults = person_type_defaults(unit.subtype);
    unit.movement.speed = defaults.speed;
}

/// Set up a pausing sub-phase: stop moving, timer 64-127 ticks.
fn enter_wander_pausing(unit: &mut Unit, rng: &mut GameRng) {
    unit.state_timer = ((rng.next() & 0x3F) + 0x40) as u16; // 64-127
    unit.movement.flags1 &= !0x1000; // Stop moving
    unit.movement.speed = 0;
}

/// Fighting: enter Seek phase, stop moving.
/// Original: case '\x19' → Person_EnterFightingState (0x00437b40).
fn enter_fighting(unit: &mut Unit) {
    unit.movement.speed = 0;
    unit.movement.flags1 &= !0x1000; // Stop moving
    unit.state_counter = CombatPhase::Seek as u8;
    unit.state_timer = 0;
}

/// Fleeing: random direction, speed=0x6E, timer=0x40.
/// Original: case '\x1a' in Person_SetState.
fn enter_fleeing(unit: &mut Unit, rng: &mut GameRng) {
    unit.movement.speed = 0x6E; // Flee speed (faster than normal)
    unit.state_timer = 0x40;     // 64 ticks
    let angle = (rng.next() & 0x7FF) as u16;
    unit.movement.facing_angle = angle;
    // Set MOVING and BLOCKED flags for flee movement
    unit.movement.flags1 |= 0x1080;
}

/// Drowning: set drowning flags.
/// Original: case '\x17' → Person_EnterDrowningState (0x00503190).
fn enter_drowning(unit: &mut Unit) {
    unit.movement.speed = 0;
    unit.movement.flags1 &= !0x1000; // Stop moving
}

/// Dead: speed=0, set dead flags, random counter 0-7.
/// Original: case '\x18' in Person_SetState.
fn enter_dead(unit: &mut Unit, rng: &mut GameRng) {
    unit.movement.speed = 0;
    unit.movement.flags1 &= !0x1000; // Stop moving
    // Original: flags1 |= 0x480, flags2 |= 0x4000
    unit.movement.flags1 |= 0x480;
    unit.state_counter = (rng.next() & 7) as u8;
}

// --- Per-tick state update ---

/// Result of a single tick_state call.
pub enum TickResult {
    /// Stay in current state.
    Continue,
    /// Transition to a new state.
    Transition(PersonState),
}

/// Per-tick state update for a single unit.
/// Called each game tick from the coordinator.
pub fn tick_state(unit: &mut Unit, rng: &mut GameRng) -> TickResult {
    match unit.state {
        PersonState::Idle => tick_idle(unit),
        PersonState::Moving | PersonState::GoToPoint | PersonState::GoToMarker => tick_moving(unit),
        PersonState::Wander => tick_wander(unit, rng),
        PersonState::Fighting => tick_fighting(unit),
        PersonState::Fleeing => tick_fleeing(unit),
        PersonState::Drowning => tick_drowning(unit),
        PersonState::Dead => tick_dead(unit),
        _ => TickResult::Continue, // Unimplemented states hold
    }
}

/// Idle: countdown state_timer, transition to Wander when expired.
/// Original: Object_ProcessPersonState case 0x01 — decrements timer,
/// on expiry looks up default state from DAT_0059fe44[subtype*0x32].
fn tick_idle(unit: &mut Unit) -> TickResult {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
        TickResult::Continue
    } else {
        TickResult::Transition(PersonState::Wander)
    }
}

/// Moving/GoToPoint/GoToMarker: check if movement completed.
/// Movement itself is handled by the existing movement system in coordinator.tick().
fn tick_moving(unit: &mut Unit) -> TickResult {
    if !unit.movement.is_moving() {
        TickResult::Transition(PersonState::Idle)
    } else {
        TickResult::Continue
    }
}

/// Wander: cycle through walking/pausing sub-phases.
/// Original: Person_ProcessIdleWanderState — phase 0=walk, 1=pause, 2=walk, then → Idle.
/// Movement (walking in random direction) is handled by the coordinator.
fn tick_wander(unit: &mut Unit, rng: &mut GameRng) -> TickResult {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
        return TickResult::Continue;
    }

    match unit.state_counter {
        0 => {
            // Walking → Pausing
            unit.state_counter = WanderPhase::Pausing as u8;
            enter_wander_pausing(unit, rng);
            TickResult::Continue
        }
        1 => {
            // Pausing → Walking2
            unit.state_counter = WanderPhase::Walking2 as u8;
            enter_wander_walking(unit, rng);
            TickResult::Continue
        }
        2 => {
            // Walking2 → back to Idle
            unit.movement.flags1 &= !0x1000;
            unit.movement.speed = 0;
            TickResult::Transition(PersonState::Idle)
        }
        _ => {
            // WaterEscape or unknown → Idle
            unit.movement.flags1 &= !0x1000;
            unit.movement.speed = 0;
            TickResult::Transition(PersonState::Idle)
        }
    }
}

/// Fighting: advance through combat sub-phases.
/// Actual damage application and chase movement happen in the coordinator
/// (needs access to both units). This manages the phase state machine.
///
/// Phase flow: Seek → Approach → SwingReady → Strike → LungeBack → LungeFwd → Recovering → Seek
pub fn tick_fighting(unit: &mut Unit) -> TickResult {
    if unit.target_unit.is_none() {
        return TickResult::Transition(PersonState::Idle);
    }

    let phase = CombatPhase::from_counter(unit.state_counter);
    match phase {
        CombatPhase::Seek => {
            // Coordinator will set Approach phase when target is detected
            // (via process_combat chase logic)
            TickResult::Continue
        }
        CombatPhase::Approach => {
            // Coordinator handles movement toward target.
            // When in melee range, coordinator sets SwingReady phase.
            TickResult::Continue
        }
        CombatPhase::SwingReady => {
            // Pre-strike pause
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                unit.state_counter = CombatPhase::Strike as u8;
                TickResult::Continue
            }
        }
        CombatPhase::Strike => {
            // Damage is applied by coordinator when it sees Strike phase.
            // Immediately transition to LungeBack.
            unit.state_counter = CombatPhase::LungeBack as u8;
            unit.state_timer = LUNGE_TICKS;
            TickResult::Continue
        }
        CombatPhase::LungeBack => {
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                unit.state_counter = CombatPhase::LungeFwd as u8;
                unit.state_timer = LUNGE_TICKS;
                TickResult::Continue
            }
        }
        CombatPhase::LungeFwd => {
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                unit.state_counter = CombatPhase::Recovering as u8;
                unit.state_timer = RECOVERING_TICKS;
                TickResult::Continue
            }
        }
        CombatPhase::Recovering => {
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                // Back to Seek for next attack cycle
                unit.state_counter = CombatPhase::Seek as u8;
                TickResult::Continue
            }
        }
    }
}

/// Fleeing: countdown timer, transition to Idle when expired.
fn tick_fleeing(unit: &mut Unit) -> TickResult {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
        TickResult::Continue
    } else {
        unit.movement.flags1 &= !0x1000;
        TickResult::Transition(PersonState::Idle)
    }
}

/// Drowning: lose health each tick, die when health reaches 0.
fn tick_drowning(unit: &mut Unit) -> TickResult {
    // Lose ~2% of max_health per tick (matches original's gradual drowning)
    let damage = (unit.max_health / 50).max(1);
    if unit.health <= damage {
        unit.health = 0;
        TickResult::Transition(PersonState::Dead)
    } else {
        unit.health -= damage;
        TickResult::Continue
    }
}

/// Dead: countdown state_counter, mark not alive when done.
fn tick_dead(unit: &mut Unit) -> TickResult {
    if unit.state_counter > 0 {
        unit.state_counter -= 1;
    } else {
        unit.alive = false;
    }
    TickResult::Continue
}

// --- Combat helpers ---

/// Calculate melee damage from attacker to defender.
/// Original: Combat_ProcessMeleeDamage (0x004c5d20).
/// damage = (fight_damage * health) / max_health, minimum 32.
/// Bloodlust doubles the damage output.
pub fn calculate_melee_damage(attacker: &Unit) -> u16 {
    let defaults = person_type_defaults(attacker.subtype);
    let base = defaults.fight_damage as u32;
    let mut damage = (base * attacker.health as u32) / attacker.max_health.max(1) as u32;
    if attacker.bloodlust {
        damage *= 2;
    }
    damage.max(0x20) as u16 // Min damage = 32
}

/// Apply damage to a unit, accounting for shield protection.
/// Original: Object_ApplyDamage (0x00504f20).
/// Shield halves incoming damage (right-shift by 1).
pub fn apply_damage(unit: &mut Unit, damage: u16) {
    let effective = if unit.shielded { damage >> 1 } else { damage };
    if unit.health <= effective {
        unit.health = 0;
    } else {
        unit.health -= effective;
    }
}

/// Detection range for combat (world coordinate units).
/// Units within this distance will engage each other.
pub const COMBAT_DETECT_RANGE: i32 = 512;

/// Melee attack range (world coordinate units).
/// Units must be this close to deal damage.
pub const COMBAT_MELEE_RANGE: i32 = 72;

/// Ticks between melee attacks (used as fallback; sub-phases have own timers).
pub const COMBAT_ATTACK_INTERVAL: u16 = 8;

/// Combat sub-phases stored in `state_counter` (offset 0x2D).
/// Original: Person_ProcessCombatState uses phase byte to drive micro-states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CombatPhase {
    /// Find/validate target, set initial approach direction.
    Seek        = 0x00,
    /// Walk toward enemy (facing updated each tick by coordinator).
    Approach    = 0x22,
    /// Pre-strike pause (0x10 = 16 ticks).
    SwingReady  = 0x07,
    /// Deliver damage this tick, then transition to lunge/recovery.
    Strike      = 0x26,
    /// Post-strike lunge backward (cosmetic, random angle offset).
    LungeBack   = 0x27,
    /// Post-strike lunge forward.
    LungeFwd    = 0x28,
    /// Post-attack cooldown (8 ticks), then back to Seek.
    Recovering  = 0x0C,
}

/// Timer for SwingReady phase (original: 0x10 ticks).
pub const SWING_READY_TICKS: u16 = 0x10;
/// Timer for Recovering phase (original: 8 ticks).
pub const RECOVERING_TICKS: u16 = 8;
/// Timer for lunge phases (original: ~4 ticks each).
pub const LUNGE_TICKS: u16 = 4;

impl CombatPhase {
    pub fn from_counter(val: u8) -> Self {
        match val {
            0x22 => CombatPhase::Approach,
            0x07 => CombatPhase::SwingReady,
            0x26 => CombatPhase::Strike,
            0x27 => CombatPhase::LungeBack,
            0x28 => CombatPhase::LungeFwd,
            0x0C => CombatPhase::Recovering,
            _    => CombatPhase::Seek,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::movement::PersonMovement;
    use crate::data::units::ModelType;

    fn make_unit(subtype: u8, tribe: u8) -> Unit {
        use crate::engine::movement::WorldCoord;
        let defaults = person_type_defaults(subtype);
        Unit {
            id: 0,
            model_type: ModelType::Person,
            subtype,
            tribe_index: tribe,
            movement: PersonMovement::default(),
            cell_x: 0.0,
            cell_y: 0.0,
            state: PersonState::Idle,
            prev_state: PersonState::Idle,
            state_timer: 0,
            state_counter: 0,
            health: defaults.max_health,
            max_health: defaults.max_health,
            target_unit: None,
            attacker_unit: None,
            alive: true,
            home_pos: WorldCoord::new(0, 0),
            behavior_flags: 0,
            wander_duration: 0,
            wander_range: 0,
            linked_obj_id: None,
            bloodlust: false,
            shielded: false,
        }
    }

    #[test]
    fn person_state_enum_values() {
        assert_eq!(PersonState::Idle as u8, 0x01);
        assert_eq!(PersonState::Moving as u8, 0x03);
        assert_eq!(PersonState::Wander as u8, 0x04);
        assert_eq!(PersonState::GoToPoint as u8, 0x05);
        assert_eq!(PersonState::GoToMarker as u8, 0x07);
        assert_eq!(PersonState::Drowning as u8, 0x17);
        assert_eq!(PersonState::Dead as u8, 0x18);
        assert_eq!(PersonState::Fighting as u8, 0x19);
        assert_eq!(PersonState::Fleeing as u8, 0x1A);
        assert_eq!(PersonState::Celebrating as u8, 0x29);
        assert_eq!(PersonState::WaitingAtReincPillar as u8, 0x2C);
    }

    #[test]
    fn person_type_defaults_health() {
        assert_eq!(person_type_defaults(1).max_health, 32);   // Wild
        assert_eq!(person_type_defaults(2).max_health, 1400); // Brave
        assert_eq!(person_type_defaults(3).max_health, 1800); // Warrior
        assert_eq!(person_type_defaults(7).max_health, 900);  // Shaman
    }

    #[test]
    fn enter_idle_sets_timer_and_zero_speed() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Idle, &mut rng);
        assert_eq!(unit.state, PersonState::Idle);
        assert_eq!(unit.movement.speed, 0);
        assert!(unit.state_timer >= 50 && unit.state_timer <= 99);
    }

    #[test]
    fn enter_wander_sets_flags_and_direction() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Wander, &mut rng);
        assert_eq!(unit.state, PersonState::Wander);
        assert!(unit.state_timer >= 32 && unit.state_timer <= 95);
        assert!(unit.movement.facing_angle <= 2047);
        assert!(unit.movement.flags1 & 0x1000 != 0); // MOVING set
        assert_eq!(unit.movement.speed, 0x30);         // Brave speed
    }

    #[test]
    fn enter_dead_sets_flags() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Dead, &mut rng);
        assert_eq!(unit.state, PersonState::Dead);
        assert_eq!(unit.movement.speed, 0);
        assert!(unit.movement.flags1 & 0x480 != 0);
        assert!(unit.state_counter <= 7);
    }

    #[test]
    fn enter_fleeing_sets_speed_and_timer() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Fleeing, &mut rng);
        assert_eq!(unit.state, PersonState::Fleeing);
        assert_eq!(unit.movement.speed, 0x6E);
        assert_eq!(unit.state_timer, 0x40);
        assert!(unit.movement.flags1 & 0x1000 != 0); // MOVING set
    }

    #[test]
    fn idle_counts_down_to_wander() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Idle;
        unit.state_timer = 2;
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_timer, 1);
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_timer, 0);
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Transition(PersonState::Wander)));
    }

    #[test]
    fn wander_sub_phases_walk_pause_walk_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Wander, &mut rng);
        assert_eq!(unit.state_counter, WanderPhase::Walking as u8);
        assert!(unit.state_timer >= 32 && unit.state_timer <= 63);
        assert!(unit.movement.flags1 & 0x1000 != 0); // MOVING

        // Drain walking timer
        while unit.state_timer > 0 {
            assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        }
        // Timer=0, should transition to Pausing
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_counter, WanderPhase::Pausing as u8);
        assert!(unit.state_timer >= 64 && unit.state_timer <= 127);
        assert_eq!(unit.movement.flags1 & 0x1000, 0); // NOT moving

        // Drain pausing timer
        while unit.state_timer > 0 {
            assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        }
        // Timer=0, should transition to Walking2
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_counter, WanderPhase::Walking2 as u8);
        assert!(unit.state_timer >= 32 && unit.state_timer <= 63);

        // Drain walking2 timer
        while unit.state_timer > 0 {
            assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        }
        // Timer=0, should transition to Idle
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Transition(PersonState::Idle)));
    }

    #[test]
    fn moving_to_idle_when_arrived() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::GoToPoint;
        unit.movement.flags1 |= 0x1000; // Still moving
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        unit.movement.flags1 &= !0x1000; // Arrived
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Transition(PersonState::Idle)));
    }

    #[test]
    fn drowning_drains_health_to_death() {
        let mut unit = make_unit(2, 0); // Brave, HP=1400
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Drowning;
        let initial_hp = unit.health;
        match tick_state(&mut unit, &mut rng) {
            TickResult::Continue => {},
            _ => panic!("Should continue"),
        }
        assert!(unit.health < initial_hp);
        for _ in 0..200 {
            if let TickResult::Transition(PersonState::Dead) = tick_state(&mut unit, &mut rng) {
                assert_eq!(unit.health, 0);
                return;
            }
        }
        panic!("Should have transitioned to Dead");
    }

    #[test]
    fn dead_counts_down_then_not_alive() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Dead;
        unit.state_counter = 3;
        unit.alive = true;
        for _ in 0..3 {
            tick_state(&mut unit, &mut rng);
            assert!(unit.alive);
        }
        tick_state(&mut unit, &mut rng);
        assert!(!unit.alive);
    }

    #[test]
    fn fleeing_counts_down_to_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Fleeing;
        unit.state_timer = 2;
        unit.movement.flags1 |= 0x1000;
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        match tick_state(&mut unit, &mut rng) {
            TickResult::Transition(PersonState::Idle) => {},
            _ => panic!("Expected Idle transition"),
        }
    }

    #[test]
    fn fighting_without_target_goes_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Fighting;
        unit.target_unit = None;
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Transition(PersonState::Idle)));
    }

    #[test]
    fn combat_phase_cycle() {
        let mut unit = make_unit(3, 0); // Warrior
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Fighting, &mut rng);
        unit.target_unit = Some(1);
        assert_eq!(unit.state_counter, CombatPhase::Seek as u8);

        // Seek stays in Seek (coordinator drives Seek→Approach)
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));

        // Simulate coordinator setting SwingReady
        unit.state_counter = CombatPhase::SwingReady as u8;
        unit.state_timer = 2;
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue)); // timer 2→1
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue)); // timer 1→0
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue)); // → Strike
        assert_eq!(unit.state_counter, CombatPhase::Strike as u8);

        // Strike → LungeBack
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_counter, CombatPhase::LungeBack as u8);
        assert_eq!(unit.state_timer, LUNGE_TICKS);

        // Drain LungeBack
        for _ in 0..LUNGE_TICKS {
            assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        }
        // → LungeFwd
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_counter, CombatPhase::LungeFwd as u8);

        // Drain LungeFwd
        for _ in 0..LUNGE_TICKS {
            assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        }
        // → Recovering
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_counter, CombatPhase::Recovering as u8);
        assert_eq!(unit.state_timer, RECOVERING_TICKS);

        // Drain Recovering
        for _ in 0..RECOVERING_TICKS {
            assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        }
        // → Seek
        assert!(matches!(tick_state(&mut unit, &mut rng), TickResult::Continue));
        assert_eq!(unit.state_counter, CombatPhase::Seek as u8);
    }

    #[test]
    fn calculate_melee_damage_scales_with_health() {
        let mut unit = make_unit(3, 0); // Warrior, fight_damage=400
        // Full health: damage = 400 * 1800 / 1800 = 400
        assert_eq!(calculate_melee_damage(&unit), 400);
        // Half health: damage = 400 * 900 / 1800 = 200
        unit.health = 900;
        assert_eq!(calculate_melee_damage(&unit), 200);
        // Very low health: damage = 400 * 10 / 1800 = 2 → clamped to 32
        unit.health = 10;
        assert_eq!(calculate_melee_damage(&unit), 32); // min 0x20
    }

    #[test]
    fn apply_damage_clamps_to_zero() {
        let mut unit = make_unit(2, 0);
        unit.health = 100;
        apply_damage(&mut unit, 50);
        assert_eq!(unit.health, 50);
        apply_damage(&mut unit, 200); // More than remaining
        assert_eq!(unit.health, 0);
    }

    #[test]
    fn bloodlust_doubles_damage() {
        let mut unit = make_unit(3, 0); // Warrior, fight_damage=400
        assert_eq!(calculate_melee_damage(&unit), 400);
        unit.bloodlust = true;
        assert_eq!(calculate_melee_damage(&unit), 800);
    }

    #[test]
    fn shield_halves_incoming_damage() {
        let mut unit = make_unit(2, 0);
        unit.health = 200;
        unit.shielded = true;
        apply_damage(&mut unit, 100); // 100 >> 1 = 50
        assert_eq!(unit.health, 150);
    }

    #[test]
    fn prev_state_saved_on_transition() {
        let mut unit = make_unit(2, 0);
        unit.state = PersonState::Idle;
        let mut rng = GameRng::new(1);
        enter_state(&mut unit, PersonState::Wander, &mut rng);
        assert_eq!(unit.prev_state, PersonState::Idle);
        assert_eq!(unit.state, PersonState::Wander);
    }
}

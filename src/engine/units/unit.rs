// Live unit entity — a person unit with mutable position and movement state.

use crate::engine::movement::PersonMovement;
use crate::data::units::ModelType;
use super::person_state::PersonState;

pub type UnitId = usize;

pub struct Unit {
    pub id: UnitId,
    pub model_type: ModelType,
    pub subtype: u8,
    pub tribe_index: u8,
    pub movement: PersonMovement,
    // Rendering cache — cell-space position, updated from world coords each tick.
    pub cell_x: f32,
    pub cell_y: f32,

    // Person state machine
    pub state: PersonState,
    pub prev_state: PersonState,
    pub state_timer: u16,      // countdown timer (ticks)
    pub state_counter: u8,     // sub-counter for state transitions (offset 0x2D)

    // Combat stats (offsets 0x6C-0x7C in original)
    pub health: u16,           // current HP (offset 0x6E)
    pub max_health: u16,       // max HP (offset 0x6C)
    pub target_unit: Option<UnitId>,   // combat target (offset 0x8A)
    pub attacker_unit: Option<UnitId>, // who's attacking us (offset 0x88)
    pub alive: bool,           // false = dead/removed from game
}

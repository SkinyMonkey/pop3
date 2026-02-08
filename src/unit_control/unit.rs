// Live unit entity — a person unit with mutable position and movement state.

use crate::movement::PersonMovement;
use crate::pop::units::ModelType;

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
}

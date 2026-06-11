// Live unit entity — a person unit with mutable position and movement state.

use crate::engine::movement::{PersonMovement, WorldCoord};
use crate::data::units::ModelType;
use super::person_state::PersonState;
use super::animation::AnimationState;

pub type UnitId = usize;

pub struct Unit {
    pub id: UnitId,
    pub model_type: ModelType,
    pub subtype: u8,
    /// Tribe slot 0..=3 (Blue/Red/Yellow/Green), or **255 = neutral / wild**.
    /// On-disk this is `SBYTE Owner` (-1 = neutral) per `pop.h:1120`; we keep the
    /// u8 sentinel for compact storage. Use [`Unit::owner`] for an `Option<u8>`
    /// view at sites that would otherwise risk a `.min(3)`-style mis-clamp.
    pub tribe_index: u8,
    pub movement: PersonMovement,
    // Rendering cache — cell-space position, updated from world coords each tick.
    pub cell_x: f32,
    pub cell_y: f32,

    // Person state machine
    pub state: PersonState,
    pub prev_state: PersonState,
    pub state_timer: u16,      // countdown timer (ticks)
    pub state_counter: u8,     // sub-counter / phase within state (offset 0x2D)

    // Combat stats (offsets 0x6C-0x7C in original)
    pub health: u16,           // current HP (offset 0x6E)
    pub max_health: u16,       // max HP (offset 0x6C)
    pub target_unit: Option<UnitId>,   // combat target (offset 0x8A)
    pub attacker_unit: Option<UnitId>, // who's attacking us (offset 0x88)
    pub alive: bool,           // false = dead/removed from game

    // Home/spawn position (offset 0x68/0x6A) — used for wander range
    pub home_pos: WorldCoord,
    // Behavior flags (offset 0x76)
    pub behavior_flags: u16,
    // Wander state (offsets 0x7B/0x7C)
    pub wander_duration: u8,   // decrements each tick while wandering
    pub wander_range: u8,      // random walk range (subtype-dependent)
    // Linked object (offset 0x72) — vehicle, effect, etc.
    pub linked_obj_id: Option<UnitId>,
    // Combat modifiers
    pub bloodlust: bool,       // bloodlust spell active — doubles damage
    pub shielded: bool,        // inside shield — halves incoming damage

    // Animation state (offsets +0x33..+0x3a in original binary)
    pub anim: AnimationState,

    // Building association — which building this person is entering/inside/exiting
    pub building_handle: Option<u16>,
    // Wood being carried (for Gathering/CarryingWood states)
    pub wood_carried: u16,
    // Guard position — where this unit should hold (for Guard state)
    pub guard_position: Option<WorldCoord>,
    // Gather target — tree position for Gathering state navigation
    pub gather_target: Option<WorldCoord>,
}

impl Unit {
    /// Decoded owner: `None` for neutral / wild units (sentinel 255 or any
    /// value outside 0..=3), `Some(0..=3)` for a tribe slot. Use this instead
    /// of indexing into tribe arrays with `tribe_index as usize` directly.
    pub fn owner(&self) -> Option<u8> {
        if self.tribe_index < 4 { Some(self.tribe_index) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_with_tribe(t: u8) -> Unit {
        Unit {
            id: 0,
            model_type: ModelType::Person,
            subtype: 2,
            tribe_index: t,
            movement: PersonMovement::default(),
            cell_x: 0.0,
            cell_y: 0.0,
            state: PersonState::Idle,
            prev_state: PersonState::Idle,
            state_timer: 0,
            state_counter: 0,
            health: 0,
            max_health: 0,
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
            anim: AnimationState::default(),
            building_handle: None,
            wood_carried: 0,
            guard_position: None,
            gather_target: None,
        }
    }

    #[test]
    fn owner_returns_some_for_valid_tribes() {
        for t in 0..=3u8 {
            assert_eq!(unit_with_tribe(t).owner(), Some(t));
        }
    }

    #[test]
    fn owner_returns_none_for_neutral_sentinel() {
        assert_eq!(unit_with_tribe(255).owner(), None);
        assert_eq!(unit_with_tribe(4).owner(), None);
        assert_eq!(unit_with_tribe(100).owner(), None);
    }
}

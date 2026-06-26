#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiBuildingPriority {
    DrumTower = 0,     // Highest (defense)
    Training = 1,      // Military
    Housing = 2,       // Population
    Reincarnation = 3, // Shaman respawn (lowest)
}

/// 7-state placement machine for AI building placement.
/// Original: AI_ValidateBuildingPlacements + AI_ExecuteBuildingPriorities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiPlacementState {
    Idle,
    SelectType,
    FindLocation,
    ValidateTerrain,
    CheckResources,
    PlaceBuilding,
    WaitConstruction,
}

pub struct AiBuildingPlacement {
    pub state: AiPlacementState,
    pub selected_type: Option<u8>,
    pub target_x: i32,
    pub target_y: i32,
    pub priority_queue: Vec<(AiBuildingPriority, u8)>,
}

impl AiBuildingPlacement {
    pub fn new() -> Self {
        Self {
            state: AiPlacementState::Idle,
            selected_type: None,
            target_x: 0,
            target_y: 0,
            priority_queue: Vec::new(),
        }
    }

    /// Queue a building for placement, sorted by priority.
    pub fn queue_building(&mut self, building_type: u8) {
        let priority = Self::priority_for_type(building_type);
        self.priority_queue.push((priority, building_type));
        self.priority_queue.sort_by_key(|(p, _)| *p);
    }

    fn priority_for_type(building_type: u8) -> AiBuildingPriority {
        match building_type {
            4 => AiBuildingPriority::DrumTower,        // DRUM_TOWER
            6 | 7 | 8 => AiBuildingPriority::Training, // SPY/WARRIOR/SUPER train
            1 | 2 | 3 => AiBuildingPriority::Housing,  // SMALL/MED/LARGE HUT
            5 => AiBuildingPriority::Reincarnation,    // TEMPLE
            _ => AiBuildingPriority::Housing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_ordering_drum_tower_highest() {
        assert!(AiBuildingPriority::DrumTower < AiBuildingPriority::Training);
        assert!(AiBuildingPriority::Training < AiBuildingPriority::Housing);
        assert!(AiBuildingPriority::Housing < AiBuildingPriority::Reincarnation);
    }

    #[test]
    fn new_placement_is_idle() {
        let p = AiBuildingPlacement::new();
        assert_eq!(p.state, AiPlacementState::Idle);
        assert!(p.priority_queue.is_empty());
    }

    #[test]
    fn queue_building_sorts_by_priority() {
        let mut p = AiBuildingPlacement::new();
        // Queue housing (type 2) then drum tower (type 4)
        p.queue_building(2); // Housing
        p.queue_building(4); // DrumTower
                             // DrumTower should sort first (lower priority value = higher priority)
        assert_eq!(p.priority_queue[0].0, AiBuildingPriority::DrumTower);
        assert_eq!(p.priority_queue[1].0, AiBuildingPriority::Housing);
    }

    #[test]
    fn placement_state_has_7_variants() {
        // Verify all 7 states exist
        let states = [
            AiPlacementState::Idle,
            AiPlacementState::SelectType,
            AiPlacementState::FindLocation,
            AiPlacementState::ValidateTerrain,
            AiPlacementState::CheckResources,
            AiPlacementState::PlaceBuilding,
            AiPlacementState::WaitConstruction,
        ];
        assert_eq!(states.len(), 7);
    }
}

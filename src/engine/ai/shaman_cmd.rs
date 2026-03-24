pub const MAX_SHAMAN_COMMANDS: usize = 10;
pub const SHAMAN_COMMAND_SIZE: usize = 0x52;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShamanCommandType {
    PrimaryAttack = 0,
    SecondaryAttack = 1,
    DefendPosition = 2,
    SpellCasting = 3,
    ArmyMovement = 4,
    BuildingPlacement = 5,
    ResourceGathering = 6,
    Conversion = 7,
}

impl ShamanCommandType {
    pub fn from_raw(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::PrimaryAttack),
            1 => Some(Self::SecondaryAttack),
            2 => Some(Self::DefendPosition),
            3 => Some(Self::SpellCasting),
            4 => Some(Self::ArmyMovement),
            5 => Some(Self::BuildingPlacement),
            6 => Some(Self::ResourceGathering),
            7 => Some(Self::Conversion),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShamanCommand {
    pub active: bool,
    pub cmd_type: ShamanCommandType,
    pub target_x: u16,
    pub target_y: u16,
    pub target_id: u16,
    pub state: u16,
}

impl ShamanCommand {
    pub fn empty() -> Self {
        Self {
            active: false,
            cmd_type: ShamanCommandType::PrimaryAttack,
            target_x: 0,
            target_y: 0,
            target_id: 0,
            state: 0,
        }
    }
}

pub struct ShamanCommandQueue {
    pub slots: [ShamanCommand; MAX_SHAMAN_COMMANDS],
}

impl ShamanCommandQueue {
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| ShamanCommand::empty()),
        }
    }

    pub fn push_command(&mut self, cmd: ShamanCommand) -> bool {
        for slot in &mut self.slots {
            if !slot.active {
                *slot = cmd;
                slot.active = true;
                return true;
            }
        }
        false
    }

    pub fn clear_command(&mut self, index: usize) {
        if index < MAX_SHAMAN_COMMANDS {
            self.slots[index].active = false;
        }
    }

    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_has_10_empty_slots() {
        let queue = ShamanCommandQueue::new();
        assert_eq!(queue.slots.len(), MAX_SHAMAN_COMMANDS);
        assert_eq!(queue.active_count(), 0);
    }

    #[test]
    fn push_command_adds_to_first_slot() {
        let mut queue = ShamanCommandQueue::new();
        let mut cmd = ShamanCommand::empty();
        cmd.cmd_type = ShamanCommandType::SpellCasting;
        cmd.active = true;
        assert!(queue.push_command(cmd));
        assert_eq!(queue.active_count(), 1);
        assert_eq!(queue.slots[0].cmd_type, ShamanCommandType::SpellCasting);
    }

    #[test]
    fn push_command_on_full_queue_returns_false() {
        let mut queue = ShamanCommandQueue::new();
        for _ in 0..MAX_SHAMAN_COMMANDS {
            assert!(queue.push_command(ShamanCommand::empty()));
        }
        assert!(!queue.push_command(ShamanCommand::empty()));
    }

    #[test]
    fn clear_command_frees_slot_for_reuse() {
        let mut queue = ShamanCommandQueue::new();
        queue.push_command(ShamanCommand::empty());
        assert_eq!(queue.active_count(), 1);
        queue.clear_command(0);
        assert_eq!(queue.active_count(), 0);
        // Can push again
        assert!(queue.push_command(ShamanCommand::empty()));
        assert_eq!(queue.active_count(), 1);
    }

    #[test]
    fn from_raw_all_8_types() {
        assert_eq!(ShamanCommandType::from_raw(0), Some(ShamanCommandType::PrimaryAttack));
        assert_eq!(ShamanCommandType::from_raw(1), Some(ShamanCommandType::SecondaryAttack));
        assert_eq!(ShamanCommandType::from_raw(2), Some(ShamanCommandType::DefendPosition));
        assert_eq!(ShamanCommandType::from_raw(3), Some(ShamanCommandType::SpellCasting));
        assert_eq!(ShamanCommandType::from_raw(4), Some(ShamanCommandType::ArmyMovement));
        assert_eq!(ShamanCommandType::from_raw(5), Some(ShamanCommandType::BuildingPlacement));
        assert_eq!(ShamanCommandType::from_raw(6), Some(ShamanCommandType::ResourceGathering));
        assert_eq!(ShamanCommandType::from_raw(7), Some(ShamanCommandType::Conversion));
        assert_eq!(ShamanCommandType::from_raw(8), None);
    }
}

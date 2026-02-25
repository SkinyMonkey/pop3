use super::constants::*;

/// Typed wrapper for g_GameFlags (0x00884bf9).
/// Each method maps to a documented bit in the original bitfield.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameFlags(u32);

impl GameFlags {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn from_raw(v: u32) -> Self {
        Self(v)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }

    // --- Queries ---

    pub fn is_paused(&self) -> bool {
        self.0 & FLAG_PAUSED != 0
    }

    pub fn is_multiplayer(&self) -> bool {
        self.0 & FLAG_MULTIPLAYER != 0
    }

    pub fn is_net_waiting(&self) -> bool {
        self.0 & FLAG_NET_WAITING != 0
    }

    pub fn is_net_paused(&self) -> bool {
        self.0 & FLAG_NET_PAUSED != 0
    }

    /// Victory or defeat state is active.
    /// When set, AI updates are skipped in the tick loop.
    pub fn is_victory_defeat(&self) -> bool {
        self.0 & FLAG_VICTORY_DEFEAT != 0
    }

    pub fn has_won(&self) -> bool {
        self.0 & FLAG_PLAYER_WON != 0
    }

    pub fn has_lost(&self) -> bool {
        self.0 & FLAG_PLAYER_LOST != 0
    }

    // --- Mutations ---

    pub fn set_paused(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_PAUSED;
        } else {
            self.0 &= !FLAG_PAUSED;
        }
    }

    pub fn set_multiplayer(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_MULTIPLAYER;
        } else {
            self.0 &= !FLAG_MULTIPLAYER;
        }
    }

    /// Set the player-won flag. Also sets victory/defeat active.
    /// Original: _DAT_00884bf9 = (_DAT_00884bf9 & 0xf9ffffff) | 0x2000000
    pub fn set_won(&mut self) {
        self.0 &= FLAG_CLEAR_VICTORY_DEFEAT;
        self.0 |= FLAG_PLAYER_WON;
    }

    /// Set the player-lost flag. Also sets victory/defeat active.
    /// Original: _DAT_00884bf9 = (_DAT_00884bf9 & 0xf9ffffff) | 0x4000000
    pub fn set_lost(&mut self) {
        self.0 &= FLAG_CLEAR_VICTORY_DEFEAT;
        self.0 |= FLAG_PLAYER_LOST;
    }

    /// Clear both victory and defeat flags.
    pub fn clear_victory_defeat(&mut self) {
        self.0 &= FLAG_CLEAR_VICTORY_DEFEAT;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_zero() {
        let f = GameFlags::new();
        assert_eq!(f.raw(), 0);
        assert!(!f.is_paused());
        assert!(!f.is_multiplayer());
        assert!(!f.has_won());
        assert!(!f.has_lost());
    }

    #[test]
    fn test_set_won_clears_lost() {
        let mut f = GameFlags::new();
        f.set_lost();
        assert!(f.has_lost());
        f.set_won();
        assert!(f.has_won());
        assert!(!f.has_lost());
    }

    #[test]
    fn test_set_lost_clears_won() {
        let mut f = GameFlags::new();
        f.set_won();
        assert!(f.has_won());
        f.set_lost();
        assert!(f.has_lost());
        assert!(!f.has_won());
    }

    #[test]
    fn test_paused_toggle() {
        let mut f = GameFlags::new();
        f.set_paused(true);
        assert!(f.is_paused());
        f.set_paused(false);
        assert!(!f.is_paused());
    }

    #[test]
    fn test_multiplayer_preserves_other_flags() {
        let mut f = GameFlags::new();
        f.set_paused(true);
        f.set_multiplayer(true);
        assert!(f.is_paused());
        assert!(f.is_multiplayer());
    }

    #[test]
    fn test_from_raw_roundtrip() {
        let f = GameFlags::from_raw(FLAG_PAUSED | FLAG_MULTIPLAYER);
        assert!(f.is_paused());
        assert!(f.is_multiplayer());
        assert!(!f.has_won());
    }
}

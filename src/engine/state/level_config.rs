//! Per-level rules from `LEVELHEADERv2.LevelFlags`.
//!
//! Cross-reference: `pop.h:945-950` (Pop-World-Editor). This struct mirrors
//! the on-disk bitfield as decoded by `crate::data::level::LevelConfig` so
//! engine-state code doesn't need to depend on the `data` layer's type.

use serde::{Deserialize, Serialize};

/// High-level level flags from `pop.h:945-950`.
///
/// Drives gameplay rules that vary per level — fog of war, shaman omnipresence,
/// whether guest spells are allowed, and whether reincarnation timeout applies.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelConfig {
    /// `LEVEL_FLAGS_USE_FOG = 1 << 0`. Currently no consumer in the engine;
    /// future fog-of-war rendering should gate on this.
    pub use_fog: bool,
    /// `LEVEL_FLAGS_SHAMAN_OMNI = 1 << 1`. Shaman is omnipresent (always
    /// accessible to spells regardless of position). No consumer yet.
    pub shaman_omni: bool,
    /// `LEVEL_FLAGS_NO_GUEST = 1 << 4`. Guest spells disabled. No consumer yet.
    pub no_guest: bool,
    /// `LEVEL_NO_REINCARNATE_TIME = 1 << 5`. Disables the reincarnation
    /// timeout path — tribes with zero population never get eliminated by the
    /// timer (consumer: `victory::check_victory_conditions`).
    pub no_reincarnate_timer: bool,
}

impl LevelConfig {
    pub fn from_flags(flags: u8) -> Self {
        Self {
            use_fog:              (flags & (1 << 0)) != 0,
            shaman_omni:          (flags & (1 << 1)) != 0,
            no_guest:             (flags & (1 << 4)) != 0,
            no_reincarnate_timer: (flags & (1 << 5)) != 0,
        }
    }

    /// Helper: whether a guest player is permitted to cast spells on this level.
    /// Returns the opposite of `no_guest`. Convenience for spell-access checks.
    pub fn guest_spells_allowed(&self) -> bool {
        !self.no_guest
    }

    /// Helper: whether the shaman can be addressed from anywhere (omnipresent)
    /// rather than only at its current world position. Consumer: shaman spell /
    /// command-issue path (not yet built).
    pub fn is_shaman_omni(&self) -> bool {
        self.shaman_omni
    }

    /// Helper: whether fog-of-war rendering should hide unexplored cells.
    /// Consumer: render path (not yet built).
    pub fn is_fog_enabled(&self) -> bool {
        self.use_fog
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_flags_decodes_each_bit() {
        assert_eq!(LevelConfig::from_flags(0).no_reincarnate_timer, false);
        assert!(LevelConfig::from_flags(0b0000_0001).use_fog);
        assert!(LevelConfig::from_flags(0b0000_0010).shaman_omni);
        assert!(LevelConfig::from_flags(0b0001_0000).no_guest);
        assert!(LevelConfig::from_flags(0b0010_0000).no_reincarnate_timer);
    }

    #[test]
    fn all_bits_decode_independently() {
        let cfg = LevelConfig::from_flags(0xFF);
        assert!(cfg.use_fog && cfg.shaman_omni && cfg.no_guest && cfg.no_reincarnate_timer);
    }

    #[test]
    fn helper_accessors_match_field_state() {
        let cfg = LevelConfig {
            use_fog: true,
            shaman_omni: true,
            no_guest: true,
            no_reincarnate_timer: false,
        };
        assert!(cfg.is_fog_enabled());
        assert!(cfg.is_shaman_omni());
        assert!(!cfg.guest_spells_allowed(), "no_guest=true must disable guest spells");
        let permissive = LevelConfig::default();
        assert!(permissive.guest_spells_allowed());
        assert!(!permissive.is_fog_enabled());
        assert!(!permissive.is_shaman_omni());
    }
}

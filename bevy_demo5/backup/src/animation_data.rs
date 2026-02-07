//! Animation data extracted from Populous: The Beginning
//!
//! This module contains animation definitions extracted from the game files.
//! The animation system works as follows:
//!
//! 1. VSTART-0.ANI contains animation start indices (8 directions per animation)
//! 2. VFRA-0.ANI contains frame sequences as linked lists
//! 3. Each VFRA entry has: sprite_frame, timing, flags, next_frame
//! 4. Flags 0x0100 marks direction boundaries (single frame or start of direction)
//!
//! For the sprite_demo, we use direct sprite frame indices from HSPR0-0.DAT.

/// Person types in the game (from exe analysis)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PersonType {
    Wildman = 1,
    Brave = 2,
    Warrior = 3,
    Preacher = 4,
    Spy = 5,
    Firewarrior = 6,
    Shaman = 7,
    Angel = 8,
}

impl PersonType {
    pub fn name(&self) -> &'static str {
        match self {
            PersonType::Wildman => "Wildman",
            PersonType::Brave => "Brave",
            PersonType::Warrior => "Warrior",
            PersonType::Preacher => "Preacher",
            PersonType::Spy => "Spy",
            PersonType::Firewarrior => "Firewarrior",
            PersonType::Shaman => "Shaman",
            PersonType::Angel => "Angel of Death",
        }
    }

    pub fn all() -> &'static [PersonType] {
        &[
            PersonType::Wildman,
            PersonType::Brave,
            PersonType::Warrior,
            PersonType::Preacher,
            PersonType::Spy,
            PersonType::Firewarrior,
            PersonType::Shaman,
            PersonType::Angel,
        ]
    }
}

/// Animation states (from exe decompilation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AnimState {
    Idle = 0,
    Walk = 1,
    Special = 2,
    Fight = 3,
    CarryIdle = 4,
    CarryWalk = 5,
    Swim = 12,
    Celebrate = 25,
}

impl AnimState {
    pub fn name(&self) -> &'static str {
        match self {
            AnimState::Idle => "Idle",
            AnimState::Walk => "Walk",
            AnimState::Special => "Special",
            AnimState::Fight => "Fight",
            AnimState::CarryIdle => "Carry (Idle)",
            AnimState::CarryWalk => "Carry (Walk)",
            AnimState::Swim => "Swim",
            AnimState::Celebrate => "Celebrate",
        }
    }
}

/// Animation definition with direct sprite frame references
#[derive(Debug, Clone)]
pub struct AnimationDef {
    /// Starting sprite frame index in HSPR0-0.DAT
    pub sprite_start: u16,
    /// Number of frames per direction
    pub frames_per_dir: u8,
    /// Number of stored directions (usually 5, mirrored to 8)
    pub stored_directions: u8,
    /// Frame timing in game ticks (21 = ~80ms per frame)
    pub timing: u8,
    /// Human-readable name
    pub name: &'static str,
}

/// Known character animations in HSPR0-0.DAT
/// These are verified working animations extracted from sprite analysis
pub const CHARACTER_ANIMATIONS: &[AnimationDef] = &[
    // Shaman animations (verified)
    AnimationDef {
        sprite_start: 7578,
        frames_per_dir: 8,
        stored_directions: 5,
        timing: 21,
        name: "Shaman Idle 1",
    },
    AnimationDef {
        sprite_start: 7618,
        frames_per_dir: 8,
        stored_directions: 5,
        timing: 21,
        name: "Shaman Idle 2",
    },
    AnimationDef {
        sprite_start: 7658,
        frames_per_dir: 8,
        stored_directions: 5,
        timing: 21,
        name: "Shaman Idle 3",
    },
    AnimationDef {
        sprite_start: 7698,
        frames_per_dir: 8,
        stored_directions: 5,
        timing: 21,
        name: "Shaman Idle 4",
    },
    // Additional animations with 3 frames per direction
    AnimationDef {
        sprite_start: 7738,
        frames_per_dir: 3,
        stored_directions: 5,
        timing: 21,
        name: "Animation 5",
    },
    AnimationDef {
        sprite_start: 7753,
        frames_per_dir: 3,
        stored_directions: 5,
        timing: 21,
        name: "Animation 6",
    },
    AnimationDef {
        sprite_start: 7768,
        frames_per_dir: 3,
        stored_directions: 5,
        timing: 21,
        name: "Animation 7",
    },
    AnimationDef {
        sprite_start: 7783,
        frames_per_dir: 3,
        stored_directions: 5,
        timing: 21,
        name: "Animation 8",
    },
    AnimationDef {
        sprite_start: 7798,
        frames_per_dir: 3,
        stored_directions: 5,
        timing: 21,
        name: "Animation 9",
    },
    AnimationDef {
        sprite_start: 7813,
        frames_per_dir: 3,
        stored_directions: 5,
        timing: 21,
        name: "Animation 10",
    },
];

/// VFRA frame entry (from VFRA-0.ANI)
#[derive(Debug, Clone, Copy)]
pub struct VfraEntry {
    /// Sprite frame index in HSPR0-0.DAT
    pub sprite_frame: u16,
    /// Timing value (lower byte is frame delay)
    pub timing: u8,
    /// Flags (0x0100 = direction boundary)
    pub flags: u16,
    /// Next frame index in VFRA (for linked list traversal)
    pub next_frame: u16,
}

impl VfraEntry {
    /// Check if this entry marks a direction boundary
    pub fn is_boundary(&self) -> bool {
        self.flags & 0x0100 != 0
    }

    /// Get timing in seconds (approximate conversion)
    /// Timing value of 21 ≈ 80ms per frame
    pub fn timing_seconds(&self) -> f32 {
        self.timing as f32 * 0.004
    }
}

/// Direction mapping for 8-direction system
/// Directions 5-7 are mirrored from 3, 2, 1
pub const DIRECTION_NAMES: &[&str] = &[
    "South",     // 0
    "Southwest", // 1
    "West",      // 2
    "Northwest", // 3
    "North",     // 4
    "Northeast", // 5 (mirror of 3)
    "East",      // 6 (mirror of 2)
    "Southeast", // 7 (mirror of 1)
];

/// Get the source direction and mirror flag for a display direction
/// Returns (source_direction, is_mirrored)
pub fn get_source_direction(direction: u8) -> (u8, bool) {
    match direction {
        0 => (0, false), // South - no mirror
        1 => (1, false), // SW - no mirror
        2 => (2, false), // West - no mirror
        3 => (3, false), // NW - no mirror
        4 => (4, false), // North - no mirror
        5 => (3, true),  // NE - mirror of NW (dir 3)
        6 => (2, true),  // East - mirror of West (dir 2)
        7 => (1, true),  // SE - mirror of SW (dir 1)
        _ => (0, false),
    }
}

/// Calculate the sprite frame index for a given animation, direction, and frame
pub fn get_sprite_frame(anim: &AnimationDef, direction: u8, frame: u8) -> (u16, bool) {
    let (source_dir, mirrored) = get_source_direction(direction);
    let frame_idx = frame as u16 % anim.frames_per_dir as u16;
    let sprite_offset = source_dir as u16 * anim.frames_per_dir as u16 + frame_idx;
    (anim.sprite_start + sprite_offset, mirrored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shaman_animation_frames() {
        let shaman1 = &CHARACTER_ANIMATIONS[0];
        assert_eq!(shaman1.sprite_start, 7578);
        assert_eq!(shaman1.frames_per_dir, 8);

        // Direction 0, frame 0 should be sprite 7578
        let (sprite, mirrored) = get_sprite_frame(shaman1, 0, 0);
        assert_eq!(sprite, 7578);
        assert!(!mirrored);

        // Direction 2, frame 3 should be sprite 7578 + 2*8 + 3 = 7597
        let (sprite, mirrored) = get_sprite_frame(shaman1, 2, 3);
        assert_eq!(sprite, 7597);
        assert!(!mirrored);

        // Direction 6 (East) should mirror direction 2 (West)
        let (sprite, mirrored) = get_sprite_frame(shaman1, 6, 3);
        assert_eq!(sprite, 7597); // Same as dir 2
        assert!(mirrored);
    }

    #[test]
    fn test_direction_mapping() {
        // Stored directions
        for dir in 0..5 {
            let (src, mirror) = get_source_direction(dir);
            assert_eq!(src, dir);
            assert!(!mirror);
        }

        // Mirrored directions
        assert_eq!(get_source_direction(5), (3, true)); // NE mirrors NW
        assert_eq!(get_source_direction(6), (2, true)); // E mirrors W
        assert_eq!(get_source_direction(7), (1, true)); // SE mirrors SW
    }
}

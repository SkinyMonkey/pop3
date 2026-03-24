pub mod building_ai;
pub mod constants;
pub mod difficulty;
pub mod popscript;
pub mod shaman_cmd;
pub mod target;

use mlua::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::state::traits::AiTick;

/// Data bridge between Rust game state and Lua PopScript functions.
/// Populated from GameWorld before AI tick, written back after.
pub struct AiGameBridge {
    pub game_tick: u32,
    pub current_tribe: u8,
    pub player_tribe: u8,
    // Per-tribe readable state
    pub tribe_populations: [u32; 4],
    pub tribe_mana: [u32; 4],
    pub tribe_active: [bool; 4],
    pub tribe_num_buildings: [u32; 4],
    // Configuration setters (written by scripts, read by game)
    pub defence_radius: [u32; 4],
    pub base_radius: [u32; 4],
    pub attack_variable: [u32; 4],
    pub reincarnation: [bool; 4],
    pub bucket_usage: [bool; 4],
    pub bucket_count_for_spell: [[u32; 21]; 4],
    pub max_building_type: [[u32; 16]; 4],
    pub building_type_enabled: [[bool; 16]; 4],
    pub spell_entry: [[bool; 21]; 4],
    pub marker_entries: Vec<MarkerEntry>,
    // AI output commands (collected during script execution)
    pub pending_attacks: Vec<AiAttackCommand>,
    pub pending_builds: Vec<AiBuildCommand>,
    pub pending_spells: Vec<AiSpellCommand>,
    pub pending_trains: Vec<AiTrainCommand>,
    pub pending_moves: Vec<AiMoveCommand>,
    pub pending_pray: Vec<AiPrayCommand>,
    pub pending_cleanup: Vec<AiCleanupCommand>,
    pub pending_convert: Vec<AiConvertCommand>,
    pub pending_shaman_move: Vec<AiShamanMoveCommand>,
}

impl AiGameBridge {
    pub fn new() -> Self {
        Self {
            game_tick: 0,
            current_tribe: 0,
            player_tribe: 0,
            tribe_populations: [0; 4],
            tribe_mana: [0; 4],
            tribe_active: [false; 4],
            tribe_num_buildings: [0; 4],
            defence_radius: [0; 4],
            base_radius: [0; 4],
            attack_variable: [0; 4],
            reincarnation: [false; 4],
            bucket_usage: [false; 4],
            bucket_count_for_spell: [[0; 21]; 4],
            max_building_type: [[0; 16]; 4],
            building_type_enabled: [[false; 16]; 4],
            spell_entry: [[false; 21]; 4],
            marker_entries: Vec::new(),
            pending_attacks: Vec::new(),
            pending_builds: Vec::new(),
            pending_spells: Vec::new(),
            pending_trains: Vec::new(),
            pending_moves: Vec::new(),
            pending_pray: Vec::new(),
            pending_cleanup: Vec::new(),
            pending_convert: Vec::new(),
            pending_shaman_move: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiAttackCommand {
    pub target_tribe: u8,
    pub num_people: u32,
    pub attack_type: u32,
    pub marker: Option<(i32, i32)>,
}

#[derive(Debug, Clone)]
pub struct AiBuildCommand {
    pub building_type: u8,
    pub marker_x: i32,
    pub marker_y: i32,
}

#[derive(Debug, Clone)]
pub struct AiSpellCommand {
    pub spell_type: u8,
    pub target_x: i32,
    pub target_y: i32,
}

#[derive(Debug, Clone)]
pub struct AiTrainCommand {
    pub unit_type: u8,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct AiMoveCommand {
    pub marker: i32,
    pub num_people: u32,
}

#[derive(Debug, Clone)]
pub struct AiPrayCommand {
    pub head_num: i32,
}

#[derive(Debug, Clone)]
pub struct AiCleanupCommand {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct AiConvertCommand {
    pub marker: i32,
}

#[derive(Debug, Clone)]
pub struct AiShamanMoveCommand {
    pub marker: i32,
}

#[derive(Debug, Clone)]
pub struct MarkerEntry {
    pub marker: i32,
    pub x: i32,
    pub y: i32,
}

/// Per-tribe script execution state.
///
/// Each of the 4 tribes has its own set of script variables and EVERY loop
/// counters. The Lua VM is shared (per D-10), but tribe state is isolated.
pub struct TribeScriptState {
    /// 64 script variables per tribe (INT_ATTR_* reads/writes).
    /// Original: script variable array at tribe script state.
    pub variables: Vec<i32>,

    /// EVERY block tick counters, keyed by block identifier.
    /// Tracks how many ticks since last execution of each EVERY block.
    pub every_counters: HashMap<String, u32>,

    /// Whether this tribe's AI script is active.
    pub active: bool,
}

impl TribeScriptState {
    pub fn new() -> Self {
        Self {
            variables: vec![0; 64],
            every_counters: HashMap::new(),
            active: false,
        }
    }
}

/// AI scripting system owning a Lua 5.4 VM and per-tribe state.
///
/// One shared Lua VM instance (per D-10) with per-tribe state tables.
/// The VM has all PopScript constants registered as globals and stub
/// functions for the 168 PopScript commands.
pub struct AiSystem {
    /// The Lua 5.4 VM instance.
    lua: Lua,

    /// Per-tribe script state (4 tribes: Blue, Red, Yellow, Green).
    pub tribe_states: [TribeScriptState; 4],
}

impl AiSystem {
    /// Create a new AI system with a Lua 5.4 VM and all PopScript constants.
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();
        constants::register_constants(&lua)?;

        let tribe_states = [
            TribeScriptState::new(),
            TribeScriptState::new(),
            TribeScriptState::new(),
            TribeScriptState::new(),
        ];

        Ok(Self { lua, tribe_states })
    }

    /// Check if the given tribe index is the human player.
    pub fn is_player(&self, tribe_idx: usize, player_tribe: u8) -> bool {
        tribe_idx == player_tribe as usize
    }

    /// Get a reference to the Lua VM (for testing and script loading).
    pub fn lua(&self) -> &Lua {
        &self.lua
    }
}

impl AiTick for AiSystem {
    fn tick_update_ai(&mut self) {
        // Empty stub -- wired in plan 06.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_system_new_succeeds() {
        let system = AiSystem::new().expect("AiSystem::new() should not panic");
        assert_eq!(system.tribe_states.len(), 4);
    }

    #[test]
    fn lua_vm_executes_simple_code() {
        let system = AiSystem::new().unwrap();
        let result: i32 = system.lua.load("return 1+1").eval().unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn ai_system_implements_ai_tick() {
        let mut system = AiSystem::new().unwrap();
        // Should be callable without panic
        system.tick_update_ai();
    }

    #[test]
    fn tribe_script_state_defaults() {
        let state = TribeScriptState::new();
        assert_eq!(state.variables.len(), 64);
        assert!(state.every_counters.is_empty());
        assert!(!state.active);
    }

    #[test]
    fn is_player_check() {
        let system = AiSystem::new().unwrap();
        assert!(system.is_player(0, 0));
        assert!(!system.is_player(1, 0));
        assert!(system.is_player(2, 2));
    }
}

pub mod constants;

use mlua::prelude::*;
use std::collections::HashMap;

use super::state::traits::AiTick;

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

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

/// Maximum Lua instructions per tribe per tick before aborting.
/// Prevents infinite loops in AI scripts.
const SCRIPT_INSTRUCTION_LIMIT: u32 = 100_000;

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

    /// Whether scripts have been loaded for current level.
    scripts_loaded: [bool; 4],

    /// Player tribe index (skip AI for this tribe).
    player_tribe: u8,

    /// Shared bridge for Lua <-> Rust communication.
    bridge: Rc<RefCell<AiGameBridge>>,
}

impl AiSystem {
    /// Create a new AI system with a Lua 5.4 VM and all PopScript constants.
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();
        constants::register_constants(&lua)?;

        // Set instruction count hook for safety (per Pitfall 2 from research).
        lua.set_hook(
            mlua::HookTriggers::new().every_nth_instruction(SCRIPT_INSTRUCTION_LIMIT),
            |_lua, _debug| {
                Err(mlua::Error::RuntimeError(
                    "AI script exceeded instruction limit (100000)".into(),
                ))
            },
        )?;

        let bridge = Rc::new(RefCell::new(AiGameBridge::new()));
        popscript::register_popscript_functions(&lua, &bridge)?;
        popscript::register_every(&lua)?;

        let tribe_states = [
            TribeScriptState::new(),
            TribeScriptState::new(),
            TribeScriptState::new(),
            TribeScriptState::new(),
        ];

        Ok(Self {
            lua,
            tribe_states,
            scripts_loaded: [false; 4],
            player_tribe: 0,
            bridge,
        })
    }

    /// Check if the given tribe index is the human player.
    pub fn is_player(&self, tribe_idx: usize, player_tribe: u8) -> bool {
        tribe_idx == player_tribe as usize
    }

    /// Get a reference to the Lua VM (for testing and script loading).
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Get a reference to the bridge (for external state sync).
    pub fn bridge(&self) -> &Rc<RefCell<AiGameBridge>> {
        &self.bridge
    }

    /// Populate the AiGameBridge with current game state before AI tick.
    /// Must be called each tick BEFORE tick_update_ai so PopScript functions
    /// read fresh values.
    pub fn update_bridge(
        &mut self,
        game_tick: u32,
        player_tribe: u8,
        tribe_populations: [u32; 4],
        tribe_mana: [u32; 4],
        tribe_active: [bool; 4],
        tribe_num_buildings: [u32; 4],
    ) {
        let mut bridge = self.bridge.borrow_mut();
        bridge.game_tick = game_tick;
        bridge.player_tribe = player_tribe;
        bridge.tribe_populations = tribe_populations;
        bridge.tribe_mana = tribe_mana;
        bridge.tribe_active = tribe_active;
        bridge.tribe_num_buildings = tribe_num_buildings;
        // Clear pending commands from previous tick
        bridge.pending_attacks.clear();
        bridge.pending_builds.clear();
        bridge.pending_spells.clear();
        bridge.pending_trains.clear();
        bridge.pending_moves.clear();
        bridge.pending_pray.clear();
        bridge.pending_cleanup.clear();
        bridge.pending_convert.clear();
        bridge.pending_shaman_move.clear();
    }

    /// Load scripts for a level. Called when transitioning to InGame.
    /// Finds and loads .lua scripts from the scripts directory for all
    /// non-player tribes.
    pub fn load_level_scripts(
        &mut self,
        level: u32,
        scripts_dir: &std::path::Path,
        player_tribe: u8,
    ) {
        self.player_tribe = player_tribe;
        self.scripts_loaded = [false; 4];

        let scripts = crate::data::scripts::find_scripts_for_level(level, scripts_dir);
        for (tribe, path) in scripts {
            if tribe == player_tribe {
                continue; // Skip player tribe
            }
            match crate::data::scripts::load_tribe_script(&path) {
                Ok(source) => {
                    // Wrap source in a tick function for this tribe.
                    // The script body becomes the function body so it runs
                    // each tick when called.
                    let wrapped = format!(
                        "function _tribe_{}_tick()\n{}\nend",
                        tribe, source
                    );
                    match self.lua.load(&wrapped).set_name(path.to_string_lossy().as_ref()).exec() {
                        Ok(()) => {
                            self.scripts_loaded[tribe as usize] = true;
                            self.tribe_states[tribe as usize].active = true;
                            log::info!(
                                "Loaded AI script for tribe {} from {}",
                                tribe,
                                path.display()
                            );
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to compile AI script for tribe {}: {}",
                                tribe,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    log::warn!("No AI script for tribe {}: {}", tribe, e);
                }
            }
        }
    }
}

impl AiTick for AiSystem {
    /// Mirrors AI_UpdateAllTribes @ 0x0041a7d0.
    /// For each tribe 0-3: if not player and active and script loaded,
    /// set current_tribe in bridge and execute the tribe's tick function.
    fn tick_update_ai(&mut self) {
        for tribe_idx in 0..4u8 {
            if tribe_idx == self.player_tribe {
                continue;
            }
            if !self.tribe_states[tribe_idx as usize].active {
                continue;
            }
            if !self.scripts_loaded[tribe_idx as usize] {
                continue;
            }

            // Set current tribe in bridge for PopScript functions
            self.bridge.borrow_mut().current_tribe = tribe_idx;

            // Reset EVERY IDs for deterministic counter keying
            let _ = self.lua.load("_every_reset_ids()").exec();

            // Execute the tribe's script tick function
            let func_name = format!("_tribe_{}_tick", tribe_idx);
            match self.lua.globals().get::<LuaFunction>(func_name.as_str()) {
                Ok(func) => {
                    if let Err(e) = func.call::<()>(()) {
                        log::error!("AI script error for tribe {}: {}", tribe_idx, e);
                    }
                }
                Err(_) => {
                    // No per-tick function defined; script may have failed to load
                }
            }
        }
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
        // Should be callable without panic (no scripts loaded = no-op)
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

    #[test]
    fn tick_update_ai_no_scripts_no_panic() {
        let mut system = AiSystem::new().unwrap();
        // Activate tribes but don't load scripts
        system.tribe_states[1].active = true;
        system.tribe_states[2].active = true;
        // Should not panic
        system.tick_update_ai();
    }

    #[test]
    fn load_level_scripts_skips_player_tribe() {
        let mut system = AiSystem::new().unwrap();
        let dir = std::env::temp_dir().join("pop3_test_load_skip_player");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Create scripts for all 4 tribes
        for t in 0..4u8 {
            let name = crate::data::scripts::script_filename(1, t);
            std::fs::write(
                dir.join(&name),
                format!("-- tribe {} script\nSET_DEFENSE_RADIUS(100)\n", t),
            )
            .unwrap();
        }

        // Player is tribe 0
        system.load_level_scripts(1, &dir, 0);

        // Player tribe should NOT have script loaded
        assert!(!system.scripts_loaded[0]);
        // Other tribes should have scripts loaded
        assert!(system.scripts_loaded[1]);
        assert!(system.scripts_loaded[2]);
        assert!(system.scripts_loaded[3]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tick_update_ai_skips_player_tribe() {
        let mut system = AiSystem::new().unwrap();
        let dir = std::env::temp_dir().join("pop3_test_skip_player");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Create script for tribe 0 and 1
        for t in 0..2u8 {
            let name = crate::data::scripts::script_filename(1, t);
            std::fs::write(
                dir.join(&name),
                "SET_DEFENSE_RADIUS(999)\n",
            )
            .unwrap();
        }

        // Player is tribe 0 -- tribe 0's script should be skipped
        system.load_level_scripts(1, &dir, 0);

        // Reset bridge defence_radius
        system.bridge.borrow_mut().defence_radius = [0; 4];

        system.tick_update_ai();

        let bridge = system.bridge.borrow();
        // Tribe 0 is player, should NOT have been updated
        assert_eq!(bridge.defence_radius[0], 0);
        // Tribe 1 is AI, should have been updated
        assert_eq!(bridge.defence_radius[1], 999);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tick_update_ai_skips_inactive_tribes() {
        let mut system = AiSystem::new().unwrap();

        // Manually mark tribe 1 as having a script loaded but not active
        system.scripts_loaded[1] = true;
        system.tribe_states[1].active = false;
        system.player_tribe = 0;

        // Define a tick function that would set something
        system
            .lua
            .load("function _tribe_1_tick() SET_DEFENSE_RADIUS(500) end")
            .exec()
            .unwrap();

        system.tick_update_ai();

        // Should NOT have executed (tribe inactive)
        let bridge = system.bridge.borrow();
        assert_eq!(bridge.defence_radius[1], 0);
    }

    #[test]
    fn tick_update_ai_sets_current_tribe_before_execution() {
        let mut system = AiSystem::new().unwrap();
        let dir = std::env::temp_dir().join("pop3_test_current_tribe");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Script for tribe 2 that reads MY_NUM_PEOPLE (depends on current_tribe)
        let name = crate::data::scripts::script_filename(1, 2);
        std::fs::write(
            dir.join(&name),
            "SET_DEFENSE_RADIUS(MY_NUM_PEOPLE())\n",
        )
        .unwrap();

        system.load_level_scripts(1, &dir, 0);

        // Set tribe 2's population to 42
        system.bridge.borrow_mut().tribe_populations[2] = 42;

        system.tick_update_ai();

        // Defence radius for tribe 2 should be 42 (matching its population)
        let bridge = system.bridge.borrow();
        assert_eq!(bridge.defence_radius[2], 42);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_bridge_populates_state() {
        let mut system = AiSystem::new().unwrap();
        system.update_bridge(42, 0, [10, 20, 30, 40], [100, 200, 300, 400], [true, true, false, false], [5, 3, 0, 0]);
        let bridge = system.bridge.borrow();
        assert_eq!(bridge.game_tick, 42);
        assert_eq!(bridge.player_tribe, 0);
        assert_eq!(bridge.tribe_populations, [10, 20, 30, 40]);
        assert_eq!(bridge.tribe_mana, [100, 200, 300, 400]);
    }

    #[test]
    fn update_bridge_clears_pending_commands() {
        let mut system = AiSystem::new().unwrap();
        system.bridge.borrow_mut().pending_attacks.push(AiAttackCommand {
            target_tribe: 1, num_people: 10, attack_type: 0, marker: None,
        });
        assert_eq!(system.bridge.borrow().pending_attacks.len(), 1);
        system.update_bridge(0, 0, [0; 4], [0; 4], [false; 4], [0; 4]);
        assert_eq!(system.bridge.borrow().pending_attacks.len(), 0);
    }

    #[test]
    fn instruction_limit_aborts_runaway_script() {
        let mut system = AiSystem::new().unwrap();

        // Define a tick function with infinite loop
        system
            .lua
            .load("function _tribe_1_tick() while true do end end")
            .exec()
            .unwrap();

        system.scripts_loaded[1] = true;
        system.tribe_states[1].active = true;
        system.player_tribe = 0;

        // Should not hang -- instruction limit will abort it
        system.tick_update_ai();
        // If we get here, the limit worked (the error is logged, not propagated)
    }
}

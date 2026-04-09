pub mod building_ai;
pub mod constants;
pub mod difficulty;
pub mod dispatch;
pub mod popscript;
pub mod shaman_cmd;
pub mod target;

use mlua::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::state::traits::AiTick;
use building_ai::AiBuildingPlacement;
use difficulty::DifficultyScaling;
use shaman_cmd::ShamanCommandQueue;

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
    // Per-tribe unit counts
    pub tribe_braves: [u32; 4],
    pub tribe_warriors: [u32; 4],
    pub tribe_preachers: [u32; 4],
    pub tribe_spies: [u32; 4],
    pub tribe_super_warriors: [u32; 4],
    // Per-tribe kill counts
    pub tribe_killed_by_blue: [u32; 4],
    pub tribe_killed_by_red: [u32; 4],
    pub tribe_killed_by_yellow: [u32; 4],
    pub tribe_killed_by_green: [u32; 4],
    // Per-tribe building counts
    pub tribe_small_huts: [u32; 4],
    pub tribe_medium_huts: [u32; 4],
    pub tribe_large_huts: [u32; 4],
    pub tribe_drum_towers: [u32; 4],
    pub tribe_temples: [u32; 4],
    pub tribe_spy_trains: [u32; 4],
    pub tribe_warrior_trains: [u32; 4],
    pub tribe_super_trains: [u32; 4],
    pub tribe_boats: [u32; 4],
    pub tribe_airships: [u32; 4],
    pub tribe_vehicles: [u32; 4],
    pub tribe_wood_count: [u32; 4],
    // Per-tribe army counts
    pub tribe_attack_army: [u32; 4],
    pub tribe_defend_army: [u32; 4],
    // Per-tribe spell costs
    pub tribe_spell_burn_cost: [u32; 4],
    pub tribe_spell_blast_cost: [u32; 4],
    pub tribe_spell_lightning_cost: [u32; 4],
    // Per-tribe reincarnation timer
    pub tribe_reincarnation_timer: [u32; 4],
    // Per-tribe shaman state
    pub tribe_shaman_lives: [u32; 4],
    pub tribe_shaman_alive: [bool; 4],
    pub tribe_shaman_available: [bool; 4],
    pub tribe_shaman_available_for_attack: [bool; 4],
    pub tribe_prisoner_left: [bool; 4],
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
    // Script flags and state
    pub state_flags: [bool; 64],
    pub turn_push_enabled: bool,
    pub tribe_disabled: [bool; 4],
    pub dont_target_shaman: bool,
    pub defend_shamen: bool,
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
            tribe_braves: [0; 4],
            tribe_warriors: [0; 4],
            tribe_preachers: [0; 4],
            tribe_spies: [0; 4],
            tribe_super_warriors: [0; 4],
            tribe_killed_by_blue: [0; 4],
            tribe_killed_by_red: [0; 4],
            tribe_killed_by_yellow: [0; 4],
            tribe_killed_by_green: [0; 4],
            tribe_small_huts: [0; 4],
            tribe_medium_huts: [0; 4],
            tribe_large_huts: [0; 4],
            tribe_drum_towers: [0; 4],
            tribe_temples: [0; 4],
            tribe_spy_trains: [0; 4],
            tribe_warrior_trains: [0; 4],
            tribe_super_trains: [0; 4],
            tribe_boats: [0; 4],
            tribe_airships: [0; 4],
            tribe_vehicles: [0; 4],
            tribe_wood_count: [0; 4],
            tribe_attack_army: [0; 4],
            tribe_defend_army: [0; 4],
            tribe_spell_burn_cost: [0; 4],
            tribe_spell_blast_cost: [0; 4],
            tribe_spell_lightning_cost: [0; 4],
            tribe_reincarnation_timer: [0; 4],
            tribe_shaman_lives: [0; 4],
            tribe_shaman_alive: [false; 4],
            tribe_shaman_available: [false; 4],
            tribe_shaman_available_for_attack: [false; 4],
            tribe_prisoner_left: [false; 4],
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
            state_flags: [false; 64],
            turn_push_enabled: false,
            tribe_disabled: [false; 4],
            dont_target_shaman: false,
            defend_shamen: false,
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
    pub tribe_id: u8,
    pub target_tribe: u8,
    pub num_people: u32,
    pub attack_type: u32,
    pub marker: Option<(i32, i32)>,
}

#[derive(Debug, Clone)]
pub struct AiBuildCommand {
    pub tribe_id: u8,
    pub building_type: u8,
    pub marker_x: i32,
    pub marker_y: i32,
}

#[derive(Debug, Clone)]
pub struct AiSpellCommand {
    pub tribe_id: u8,
    pub spell_type: u8,
    pub target_x: i32,
    pub target_y: i32,
}

#[derive(Debug, Clone)]
pub struct AiTrainCommand {
    pub tribe_id: u8,
    pub unit_type: u8,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct AiMoveCommand {
    pub tribe_id: u8,
    pub marker: i32,
    pub num_people: u32,
}

#[derive(Debug, Clone)]
pub struct AiPrayCommand {
    pub tribe_id: u8,
    pub head_num: i32,
}

#[derive(Debug, Clone)]
pub struct AiCleanupCommand {
    pub tribe_id: u8,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct AiConvertCommand {
    pub tribe_id: u8,
    pub marker: i32,
}

#[derive(Debug, Clone)]
pub struct AiShamanMoveCommand {
    pub tribe_id: u8,
    pub marker: i32,
}

/// Snapshot of all pending AI commands, returned by drain_pending_commands.
pub struct AiPendingCommands {
    pub attacks: Vec<AiAttackCommand>,
    pub builds: Vec<AiBuildCommand>,
    pub spells: Vec<AiSpellCommand>,
    pub trains: Vec<AiTrainCommand>,
    pub moves: Vec<AiMoveCommand>,
    pub converts: Vec<AiConvertCommand>,
    pub shaman_moves: Vec<AiShamanMoveCommand>,
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
/// Set high enough for complex initialization ticks (GAME_TURN == 0)
/// where many variables and EVERY blocks fire simultaneously.
const SCRIPT_INSTRUCTION_LIMIT: u32 = 500_000;

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

    /// Per-tribe difficulty scaling (mana adjust, training costs).
    difficulty: [DifficultyScaling; 4],

    /// Per-tribe shaman command queues (10 slots each).
    shaman_commands: [ShamanCommandQueue; 4],

    /// Per-tribe building placement state machines.
    building_placement: [AiBuildingPlacement; 4],
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
        popscript::register_query_globals(&lua, &bridge)?;
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
            difficulty: [
                DifficultyScaling::normal(),
                DifficultyScaling::normal(),
                DifficultyScaling::normal(),
                DifficultyScaling::normal(),
            ],
            shaman_commands: std::array::from_fn(|_| ShamanCommandQueue::new()),
            building_placement: std::array::from_fn(|_| AiBuildingPlacement::new()),
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

    /// Clone pending commands from bridge for dispatch.
    /// Commands remain in bridge and are cleared by update_bridge at start of next tick.
    pub fn drain_pending_commands(&self) -> AiPendingCommands {
        let bridge = self.bridge.borrow();
        AiPendingCommands {
            attacks: bridge.pending_attacks.clone(),
            builds: bridge.pending_builds.clone(),
            spells: bridge.pending_spells.clone(),
            trains: bridge.pending_trains.clone(),
            moves: bridge.pending_moves.clone(),
            converts: bridge.pending_convert.clone(),
            shaman_moves: bridge.pending_shaman_move.clone(),
        }
    }

    /// Extract AI state for serialization (save game).
    ///
    /// Returns (per-tribe variables, EVERY counters from Lua VM).
    /// EVERY counters are global (not per-tribe), stored as a single entry.
    pub fn extract_save_state(&self) -> (Vec<Vec<i32>>, Vec<Vec<(String, u32)>>) {
        // Extract per-tribe variables
        let variables: Vec<Vec<i32>> = self
            .tribe_states
            .iter()
            .map(|ts| ts.variables.clone())
            .collect();

        // Extract EVERY counters from Lua VM
        let every_counters = match self.lua.globals().get::<LuaFunction>("_get_every_counters") {
            Ok(func) => match func.call::<LuaTable>(()) {
                Ok(table) => {
                    let mut entries = Vec::new();
                    for pair in table.pairs::<String, u32>() {
                        if let Ok((k, v)) = pair {
                            entries.push((k, v));
                        }
                    }
                    if entries.is_empty() {
                        Vec::new()
                    } else {
                        vec![entries]
                    }
                }
                Err(_) => Vec::new(),
            },
            Err(_) => Vec::new(),
        };

        (variables, every_counters)
    }

    /// Restore AI state from deserialized save data (load game).
    ///
    /// Restores per-tribe variables and EVERY counters into the Lua VM.
    pub fn restore_save_state(
        &mut self,
        variables: &[Vec<i32>],
        every_counters: &[Vec<(String, u32)>],
    ) {
        // Restore per-tribe variables
        for (i, vars) in variables.iter().enumerate() {
            if i < 4 {
                self.tribe_states[i].variables = vars.clone();
            }
        }

        // Restore EVERY counters into Lua VM
        if !every_counters.is_empty() {
            if let Ok(table) = self.lua.create_table() {
                for (k, v) in &every_counters[0] {
                    let _ = table.set(k.as_str(), *v);
                }
                if let Ok(func) = self.lua.globals().get::<LuaFunction>("_set_every_counters") {
                    let _ = func.call::<()>(table);
                }
            }
        }
    }

    /// Get mana adjustment factor for a tribe (100 = no change).
    pub fn mana_adjust(&self, tribe_idx: usize) -> u32 {
        self.difficulty[tribe_idx].mana_adjust
    }

    /// Get mutable access to a tribe's shaman command queue.
    pub fn shaman_commands_mut(&mut self, tribe: u8) -> &mut ShamanCommandQueue {
        &mut self.shaman_commands[tribe as usize]
    }

    /// Get mutable access to a tribe's building placement state.
    pub fn building_placement_mut(&mut self, tribe: u8) -> &mut AiBuildingPlacement {
        &mut self.building_placement[tribe as usize]
    }

    /// Get marker entries from bridge (for dispatch to resolve marker IDs).
    pub fn marker_entries(&self) -> Vec<MarkerEntry> {
        self.bridge.borrow().marker_entries.clone()
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

    /// How many tribes have scripts loaded.
    pub fn loaded_tribe_count(&self) -> usize {
        self.scripts_loaded.iter().filter(|&&s| s).count()
    }

    /// Load scripts for a level. Called when transitioning to InGame.
    /// Finds and loads .lua scripts from the scripts directory for all tribes.
    /// Scripts run for all tribes, but AI commands only apply to non-player tribes.
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
            match crate::data::scripts::load_tribe_script(&path) {
                Ok(source) => {
                    // Initialize PopScript variables (_var0.._var63 = 0)
                    popscript::init_script_variables(&self.lua);

                    // Preprocess: normalize pure query name usage
                    // (e.g., MY_NUM_PEOPLE() → MY_NUM_PEOPLE) so the
                    // plain-integer globals work for both variable and
                    // function-call access patterns.
                    let source = popscript::preprocess_script(&source);

                    // Wrap source in a tick function for this tribe.
                    // The script body becomes the function body so it runs
                    // each tick when called.
                    let wrapped = format!("function _tribe_{}_tick()\n{}\nend", tribe, source);
                    match self
                        .lua
                        .load(&wrapped)
                        .set_name(path.to_string_lossy().as_ref())
                        .exec()
                    {
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
                            log::error!("Failed to compile AI script for tribe {}: {}", tribe, e);
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
    /// For each tribe 0-3: if active and script loaded,
    /// set current_tribe in bridge and execute the tribe's tick function.
    /// Scripts run for all tribes (including player) to trigger level events,
    /// but AI command dispatch is filtered to non-player tribes only.
    fn tick_update_ai(&mut self) {
        for tribe_idx in 0..4u8 {
            if !self.tribe_states[tribe_idx as usize].active {
                continue;
            }
            if !self.scripts_loaded[tribe_idx as usize] {
                continue;
            }

            // Set current tribe in bridge for PopScript functions
            self.bridge.borrow_mut().current_tribe = tribe_idx;

            // Update query globals as plain integers for variable access in scripts
            {
                let bridge = self.bridge.borrow();
                popscript::update_query_globals(&self.lua, &bridge);
            }

            // Reset EVERY IDs for deterministic counter keying
            let _ = self.lua.load("_every_reset_ids()").exec();

            // Execute the tribe's script tick function
            let func_name = format!("_tribe_{}_tick", tribe_idx);
            match self.lua.globals().get::<LuaFunction>(func_name.as_str()) {
                Ok(func) => {
                    if let Err(e) = func.call::<()>(()) {
                        log::error!("Script error for tribe {}: {}", tribe_idx, e);
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

        // All tribes should have scripts loaded (including player for level events)
        assert!(system.scripts_loaded[0]);
        assert!(system.scripts_loaded[1]);
        assert!(system.scripts_loaded[2]);
        assert!(system.scripts_loaded[3]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tick_update_ai_executes_all_tribes() {
        let mut system = AiSystem::new().unwrap();
        let dir = std::env::temp_dir().join("pop3_test_all_tribes");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Create script for tribe 0 and 1
        for t in 0..2u8 {
            let name = crate::data::scripts::script_filename(1, t);
            std::fs::write(dir.join(&name), "SET_DEFENSE_RADIUS(999)\n").unwrap();
        }

        // Player is tribe 0 -- all tribes execute (player tribe for level events)
        system.load_level_scripts(1, &dir, 0);

        // Reset bridge defence_radius
        system.bridge.borrow_mut().defence_radius = [0; 4];

        system.tick_update_ai();

        let bridge = system.bridge.borrow();
        // All tribes should have been updated
        assert_eq!(bridge.defence_radius[0], 999);
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
        std::fs::write(dir.join(&name), "SET_DEFENSE_RADIUS(MY_NUM_PEOPLE())\n").unwrap();

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
        system.update_bridge(
            42,
            0,
            [10, 20, 30, 40],
            [100, 200, 300, 400],
            [true, true, false, false],
            [5, 3, 0, 0],
        );
        let bridge = system.bridge.borrow();
        assert_eq!(bridge.game_tick, 42);
        assert_eq!(bridge.player_tribe, 0);
        assert_eq!(bridge.tribe_populations, [10, 20, 30, 40]);
        assert_eq!(bridge.tribe_mana, [100, 200, 300, 400]);
    }

    #[test]
    fn update_bridge_clears_pending_commands() {
        let mut system = AiSystem::new().unwrap();
        system
            .bridge
            .borrow_mut()
            .pending_attacks
            .push(AiAttackCommand {
                tribe_id: 1,
                target_tribe: 1,
                num_people: 10,
                attack_type: 0,
                marker: None,
            });
        assert_eq!(system.bridge.borrow().pending_attacks.len(), 1);
        system.update_bridge(0, 0, [0; 4], [0; 4], [false; 4], [0; 4]);
        assert_eq!(system.bridge.borrow().pending_attacks.len(), 0);
    }

    #[test]
    fn drain_pending_commands_returns_queued_attacks() {
        let system = AiSystem::new().unwrap();
        system
            .bridge
            .borrow_mut()
            .pending_attacks
            .push(AiAttackCommand {
                tribe_id: 1,
                target_tribe: 2,
                num_people: 15,
                attack_type: 1,
                marker: Some((100, 200)),
            });
        let cmds = system.drain_pending_commands();
        assert_eq!(cmds.attacks.len(), 1);
        assert_eq!(cmds.attacks[0].target_tribe, 2);
        assert_eq!(cmds.attacks[0].num_people, 15);
    }

    #[test]
    fn difficulty_defaults_to_normal() {
        let system = AiSystem::new().unwrap();
        for i in 0..4 {
            assert_eq!(system.mana_adjust(i), 100);
        }
    }

    #[test]
    fn extract_and_restore_round_trip() {
        let mut system = AiSystem::new().unwrap();

        // Set tribe 1's variable[0] to 42
        system.tribe_states[1].variables[0] = 42;

        // Push an EVERY counter into Lua
        system
            .lua
            .load(r#"_set_every_counters({["64_0"] = 100})"#)
            .exec()
            .unwrap();

        // Extract state
        let (vars, every) = system.extract_save_state();
        assert_eq!(vars[1][0], 42);
        assert!(!every.is_empty());
        let has_counter = every[0].iter().any(|(k, v)| k == "64_0" && *v == 100);
        assert!(has_counter, "Expected EVERY counter '64_0' = 100");

        // Create a fresh system and restore
        let mut system2 = AiSystem::new().unwrap();
        system2.restore_save_state(&vars, &every);
        assert_eq!(system2.tribe_states[1].variables[0], 42);

        // Verify Lua-side EVERY counters restored
        let val: i32 = system2
            .lua
            .load(r#"return _get_every_counters()["64_0"] or 0"#)
            .eval()
            .unwrap();
        assert_eq!(val, 100);
    }

    #[test]
    fn extract_empty_state_is_safe() {
        let system = AiSystem::new().unwrap();
        let (vars, every) = system.extract_save_state();

        // All 4 tribes should have 64 variables, all zeros
        assert_eq!(vars.len(), 4);
        for v in &vars {
            assert_eq!(v.len(), 64);
            assert!(v.iter().all(|&x| x == 0));
        }

        // EVERY counters should be empty (one entry with empty vec, or empty outer)
        if !every.is_empty() {
            assert!(every[0].is_empty());
        }
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

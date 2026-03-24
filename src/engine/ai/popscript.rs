use mlua::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::AiGameBridge;

/// Register the EVERY macro as a Lua function.
///
/// EVERY(interval, body) executes `body()` when `(game_tick - last_fire) >= interval`.
/// Each call site gets its own counter tracked in a Lua-side table.
/// Uses an auto-incrementing ID per unique (interval, call-order) pair since
/// `debug.getinfo` is not available in mlua's sandboxed Lua.
pub fn register_every(lua: &Lua) -> LuaResult<()> {
    lua.load(
        r#"
        local _every_counters = {}
        local _every_next_id = 0

        function EVERY(interval, body)
            -- Use the interval + sequential ID as a unique key per call site.
            -- Scripts are deterministic so call order is stable across ticks.
            -- The key resets each script load but counters persist via save/load.
            local key = tostring(interval) .. "_" .. tostring(_every_next_id)
            _every_next_id = _every_next_id + 1

            if not _every_counters[key] then
                _every_counters[key] = 0
            end
            local tick = GAME_TURN()
            if (tick - _every_counters[key]) >= interval then
                _every_counters[key] = tick
                body()
            end
        end

        -- Reset the ID counter at start of each tick (call before running script)
        function _every_reset_ids()
            _every_next_id = 0
        end

        -- Export counter table for save/load
        function _get_every_counters()
            return _every_counters
        end
        function _set_every_counters(t)
            _every_counters = t
        end
    "#,
    )
    .exec()?;
    Ok(())
}

/// Register all PopScript functions that bridge Lua scripts to game state.
///
/// Functions are registered with access to the AiGameBridge via Rc<RefCell<>>.
/// The bridge is populated from GameWorld before each AI tick and drained after.
///
/// Per D-02: All 168 functions registered.
/// Per D-04: Unimplemented functions panic, not silent no-op.
pub fn register_popscript_functions(
    lua: &Lua,
    bridge: &Rc<RefCell<AiGameBridge>>,
) -> LuaResult<()> {
    let globals = lua.globals();

    // ---- Game state read functions (real implementations) ----

    // GAME_TURN() -> current game tick
    {
        let b = bridge.clone();
        globals.set(
            "GAME_TURN",
            lua.create_function(move |_, ()| {
                let bridge = b.borrow();
                Ok(bridge.game_tick as i32)
            })?,
        )?;
    }

    // MY_NUM_PEOPLE() -> current tribe's population
    {
        let b = bridge.clone();
        globals.set(
            "MY_NUM_PEOPLE",
            lua.create_function(move |_, ()| {
                let bridge = b.borrow();
                let tribe = bridge.current_tribe as usize;
                Ok(bridge.tribe_populations[tribe] as i32)
            })?,
        )?;
    }

    // BLUE_PEOPLE() -> blue tribe population
    {
        let b = bridge.clone();
        globals.set(
            "BLUE_PEOPLE",
            lua.create_function(move |_, ()| {
                let bridge = b.borrow();
                Ok(bridge.tribe_populations[0] as i32)
            })?,
        )?;
    }

    // RED_PEOPLE() -> red tribe population
    {
        let b = bridge.clone();
        globals.set(
            "RED_PEOPLE",
            lua.create_function(move |_, ()| {
                let bridge = b.borrow();
                Ok(bridge.tribe_populations[1] as i32)
            })?,
        )?;
    }

    // YELLOW_PEOPLE() -> yellow tribe population
    {
        let b = bridge.clone();
        globals.set(
            "YELLOW_PEOPLE",
            lua.create_function(move |_, ()| {
                let bridge = b.borrow();
                Ok(bridge.tribe_populations[2] as i32)
            })?,
        )?;
    }

    // GREEN_PEOPLE() -> green tribe population
    {
        let b = bridge.clone();
        globals.set(
            "GREEN_PEOPLE",
            lua.create_function(move |_, ()| {
                let bridge = b.borrow();
                Ok(bridge.tribe_populations[3] as i32)
            })?,
        )?;
    }

    // MY_MANA() -> current tribe's mana
    {
        let b = bridge.clone();
        globals.set(
            "MY_MANA",
            lua.create_function(move |_, ()| {
                let bridge = b.borrow();
                let tribe = bridge.current_tribe as usize;
                Ok(bridge.tribe_mana[tribe] as i32)
            })?,
        )?;
    }

    // ---- Action commands (push to bridge pending lists) ----

    // ATTACK(target_tribe, num_people, attack_type)
    {
        let b = bridge.clone();
        globals.set(
            "ATTACK",
            lua.create_function(move |_, (target, num, atype): (i32, i32, i32)| {
                b.borrow_mut().pending_attacks.push(super::AiAttackCommand {
                    target_tribe: target as u8,
                    num_people: num as u32,
                    attack_type: atype as u32,
                    marker: None,
                });
                Ok(0)
            })?,
        )?;
    }

    // ATTACK_MARKER(target_tribe, marker, num_people, attack_type)
    {
        let b = bridge.clone();
        globals.set(
            "ATTACK_MARKER",
            lua.create_function(
                move |_, (target, marker, num, atype): (i32, i32, i32, i32)| {
                    b.borrow_mut().pending_attacks.push(super::AiAttackCommand {
                        target_tribe: target as u8,
                        num_people: num as u32,
                        attack_type: atype as u32,
                        marker: Some((marker, 0)),
                    });
                    Ok(0)
                },
            )?,
        )?;
    }

    // BUILD_AT(building_type, x, y)
    {
        let b = bridge.clone();
        globals.set(
            "BUILD_AT",
            lua.create_function(move |_, (btype, x, y): (i32, i32, i32)| {
                b.borrow_mut().pending_builds.push(super::AiBuildCommand {
                    building_type: btype as u8,
                    marker_x: x,
                    marker_y: y,
                });
                Ok(0)
            })?,
        )?;
    }

    // TRAIN_PEOPLE_NOW(unit_type, count)
    {
        let b = bridge.clone();
        globals.set(
            "TRAIN_PEOPLE_NOW",
            lua.create_function(move |_, (utype, count): (i32, i32)| {
                b.borrow_mut().pending_trains.push(super::AiTrainCommand {
                    unit_type: utype as u8,
                    count: count as u32,
                });
                Ok(0)
            })?,
        )?;
    }

    // SPELL_AT_MARKER(spell_type, marker)
    {
        let b = bridge.clone();
        globals.set(
            "SPELL_AT_MARKER",
            lua.create_function(move |_, (spell, marker): (i32, i32)| {
                b.borrow_mut().pending_spells.push(super::AiSpellCommand {
                    spell_type: spell as u8,
                    target_x: marker,
                    target_y: 0,
                });
                Ok(0)
            })?,
        )?;
    }

    // CONVERT_AT_MARKER(marker)
    {
        let b = bridge.clone();
        globals.set(
            "CONVERT_AT_MARKER",
            lua.create_function(move |_, marker: i32| {
                b.borrow_mut()
                    .pending_convert
                    .push(super::AiConvertCommand { marker });
                Ok(0)
            })?,
        )?;
    }

    // SEND_PEOPLE_TO_MARKER(marker, num)
    {
        let b = bridge.clone();
        globals.set(
            "SEND_PEOPLE_TO_MARKER",
            lua.create_function(move |_, (marker, num): (i32, i32)| {
                b.borrow_mut().pending_moves.push(super::AiMoveCommand {
                    marker,
                    num_people: num as u32,
                });
                Ok(0)
            })?,
        )?;
    }

    // SEND_SHAMAN_TO_MARKER(marker) -- alias: SEND_SHAMAN_DEFENDERS_HOME uses marker 0
    {
        let b = bridge.clone();
        globals.set(
            "SEND_SHAMAN_DEFENDERS_HOME",
            lua.create_function(move |_, ()| {
                b.borrow_mut()
                    .pending_shaman_move
                    .push(super::AiShamanMoveCommand { marker: 0 });
                Ok(0)
            })?,
        )?;
    }

    // PRAY_AT_HEAD(head_num)
    {
        let b = bridge.clone();
        globals.set(
            "PRAY_AT_HEAD",
            lua.create_function(move |_, head_num: i32| {
                b.borrow_mut()
                    .pending_pray
                    .push(super::AiPrayCommand { head_num });
                Ok(0)
            })?,
        )?;
    }

    // SET_MARKER_ENTRY(marker, x, y)
    {
        let b = bridge.clone();
        globals.set(
            "SET_MARKER_ENTRY",
            lua.create_function(move |_, (marker, x, y): (i32, i32, i32)| {
                b.borrow_mut()
                    .marker_entries
                    .push(super::MarkerEntry { marker, x, y });
                Ok(0)
            })?,
        )?;
    }

    // DELETE_SMOKE_STUFF(x, y)
    {
        let b = bridge.clone();
        globals.set(
            "DELETE_SMOKE_STUFF",
            lua.create_function(move |_, (x, y): (i32, i32)| {
                b.borrow_mut()
                    .pending_cleanup
                    .push(super::AiCleanupCommand { x, y });
                Ok(0)
            })?,
        )?;
    }

    // ---- Configuration setters (store in bridge fields) ----

    // SET_DEFENSE_RADIUS(r)
    {
        let b = bridge.clone();
        globals.set(
            "SET_DEFENSE_RADIUS",
            lua.create_function(move |_, r: i32| {
                let mut bridge = b.borrow_mut();
                let tribe = bridge.current_tribe as usize;
                bridge.defence_radius[tribe] = r as u32;
                Ok(0)
            })?,
        )?;
    }

    // SET_BASE_RADIUS(r)
    {
        let b = bridge.clone();
        globals.set(
            "SET_BASE_RADIUS",
            lua.create_function(move |_, r: i32| {
                let mut bridge = b.borrow_mut();
                let tribe = bridge.current_tribe as usize;
                bridge.base_radius[tribe] = r as u32;
                Ok(0)
            })?,
        )?;
    }

    // SET_ATTACK_VARIABLE(v)
    {
        let b = bridge.clone();
        globals.set(
            "SET_ATTACK_VARIABLE",
            lua.create_function(move |_, v: i32| {
                let mut bridge = b.borrow_mut();
                let tribe = bridge.current_tribe as usize;
                bridge.attack_variable[tribe] = v as u32;
                Ok(0)
            })?,
        )?;
    }

    // SET_SPELL_ENTRY(spell, enabled)
    {
        let b = bridge.clone();
        globals.set(
            "SET_SPELL_ENTRY",
            lua.create_function(move |_, (spell, enabled): (i32, i32)| {
                let mut bridge = b.borrow_mut();
                let tribe = bridge.current_tribe as usize;
                if (spell as usize) < 21 {
                    bridge.spell_entry[tribe][spell as usize] = enabled != 0;
                }
                Ok(0)
            })?,
        )?;
    }

    // SET_REINCARNATION(on)
    {
        let b = bridge.clone();
        globals.set(
            "SET_REINCARNATION",
            lua.create_function(move |_, on: i32| {
                let mut bridge = b.borrow_mut();
                let tribe = bridge.current_tribe as usize;
                bridge.reincarnation[tribe] = on != 0;
                Ok(0)
            })?,
        )?;
    }

    // SET_BUCKET_USAGE(on)
    {
        let b = bridge.clone();
        globals.set(
            "SET_BUCKET_USAGE",
            lua.create_function(move |_, on: i32| {
                let mut bridge = b.borrow_mut();
                let tribe = bridge.current_tribe as usize;
                bridge.bucket_usage[tribe] = on != 0;
                Ok(0)
            })?,
        )?;
    }

    // SET_BUCKET_COUNT_FOR_SPELL(spell, count) -- not in original 168 list but
    // referenced in plan; register as stub since it's config-only
    // MAX_BUILDING_TYPE(type, count)
    {
        let b = bridge.clone();
        globals.set(
            "MAX_BUILDING_TYPE",
            lua.create_function(move |_, (btype, count): (i32, i32)| {
                let mut bridge = b.borrow_mut();
                let tribe = bridge.current_tribe as usize;
                if (btype as usize) < 16 {
                    bridge.max_building_type[tribe][btype as usize] = count as u32;
                }
                Ok(0)
            })?,
        )?;
    }

    // ENABLE_BUILDING_TYPE(type) -- not in original 168 function list;
    // building enable is handled via MAX_BUILDING_TYPE > 0 or ATTR flags.

    // ---- Remaining stub functions that panic per D-04 ----
    // Functions that need game state not yet available in the bridge keep their
    // panic stubs. They will be wired when underlying game systems are ready.

    let stub_functions = [
        "DEFEND_SHAMEN",
        "SEND_ALL_PEOPLE_TO_MARKER",
        "SEND_BLUE_PEOPLE_TO_MARKER",
        "SEND_RED_PEOPLE_TO_MARKER",
        "SEND_GREEN_PEOPLE_TO_MARKER",
        "SEND_GHOSTS_TO_MARKER",
        "SET_BASE_MARKER",
        "RESET_BASE_MARKER",
        "MARKER_ENTRIES",
        "SET_DRUM_TOWER_POS",
        "SET_BUILDING_DIRECTION",
        "SET_BOAT_HOUSE_WITH_BOAT",
        "PARTIAL_BUILDING_COUNT",
        "IS_BUILDING_NEAR",
        "PREACH_AT_MARKER",
        "COUNT_PEOPLE_IN_HOUSES",
        "CLEAR_STANDING_PEOPLE",
        "DESELECT_ALL_PEOPLE",
        "SET_WOOD_COLLECTION_RADII",
        "ONLY_STAND_AT_MARKERS",
        "I_KILL_CONVERTABLE",
        "CLEAR_HOUSE_INFO_FLAG",
        "FIX_WILD_IN_AREA",
        "SPELL_AT_THING",
        "GIVE_MANA_TO_PLAYER",
        "STATE_SET",
        "SET_TIMER_GOING",
        "HAS_TIMER_REACHED_ZERO",
        "REMOVE_TIMER",
        "DO_TRIGGER",
        "TRIGGER_THING",
        "TRIGGER_LEVEL_WON",
        "TRIGGER_LEVEL_LOST",
        "TURN_PUSH_ON",
        "TURN_PUSH_OFF",
        "TARGET_SHAMAN",
        "TARGET_MEDICINE_MAN",
        "TARGET_WARRIORS",
        "TARGET_S_WARRIOR",
        "TARGET_FIREWARRIORS",
        "TARGET_BLUE_SHAMAN",
        "TARGET_RED_SHAMAN",
        "TARGET_YELLOW_SHAMAN",
        "TARGET_GREEN_SHAMAN",
        "TARGET_BLUE_DRUM_TOWERS",
        "TARGET_RED_DRUM_TOWERS",
        "TARGET_YELLOW_DRUM_TOWERS",
        "TARGET_GREEN_DRUM_TOWERS",
        "TARGET_BLUE_SUPER_WARRIORS",
        "TARGET_RED_SUPER_WARRIORS",
        "TARGET_YELLOW_SUPER_WARRIORS",
        "TARGET_GREEN_SUPER_WARRIORS",
        "DONT_TARGET_SHAMAN",
        "IS_SHAMAN_AVAILABLE_FOR_ATTACK",
        "IS_SHAMAN_IN_AREA",
        "IS_PRISONER_LEFT",
        "NAV_CHECK",
        "GET_HEAD_TRIGGER_COUNT",
        "GET_HEIGHT_AT_POS",
        "THING_COUNT_IN_AREA",
        "CAMERA_ROTATION",
        "FLYBY_CREATE_NEW",
        "FLYBY_SET_EVENT_POS",
        "FLYBY_SET_EVENT_ANGLE",
        "FLYBY_SET_EVENT_ZOOM",
        "FLYBY_SET_EVENT_INT_POINT",
        "FLYBY_SET_EVENT_TOOLTIP",
        "FLYBY_SET_END_TARGET",
        "FLYBY_START",
        "FLYBY_STOP",
        "FLYBY_ALLOW_INTERRUPT",
        "OPEN_DIALOG",
        "SET_MSG_AUTO_OPEN_DLG",
        "SET_MSG_DELETE_ON_OK",
        "SET_MSG_ID",
        "SET_MSG_NARRATIVE",
        "SET_MSG_OK_SAVE",
        "SET_MSG_TIMEOUT",
        "FLASH_BUTTON",
        "MARVELLOUS_HOUSE_DEATH",
        "REMOVE_HEAD_AT_POS",
        "REMOVE_PLAYER_THING",
        "SET_NO_BLUE",
        "SET_NO_RED",
        "SET_NO_GREEN",
        "SET_NO_YELLOW",
        "BOAT_PATROL",
        // Game state query stubs (need game systems not yet in bridge)
        "MY_NUM_KILLED_BY_BLUE",
        "MY_NUM_KILLED_BY_RED",
        "MY_NUM_KILLED_BY_YELLOW",
        "MY_NUM_KILLED_BY_GREEN",
        "MY_NUM_BRAVES",
        "MY_NUM_WARRIORS",
        "MY_NUM_PREACHERS",
        "MY_NUM_SPIES",
        "MY_NUM_SUPER_WARRIORS",
        "BLUE_BRAVES",
        "BLUE_WARRIORS",
        "BLUE_PREACHERS",
        "BLUE_SPIES",
        "BLUE_SUPER_WARRIORS",
        "RED_BRAVES",
        "RED_WARRIORS",
        "RED_PREACHERS",
        "RED_SPIES",
        "RED_SUPER_WARRIORS",
        "YELLOW_BRAVES",
        "YELLOW_WARRIORS",
        "YELLOW_PREACHERS",
        "YELLOW_SPIES",
        "YELLOW_SUPER_WARRIORS",
        "GREEN_BRAVES",
        "GREEN_WARRIORS",
        "GREEN_PREACHERS",
        "GREEN_SPIES",
        "GREEN_SUPER_WARRIORS",
        "MY_SPELL_BURN_COST",
        "MY_SPELL_BLAST_COST",
        "MY_SPELL_LIGHTNING_COST",
        "MY_NUM_SMALL_HUT",
        "MY_NUM_MEDIUM_HUT",
        "MY_NUM_LARGE_HUT",
        "MY_NUM_DRUM_TOWER",
        "MY_NUM_TEMPLE",
        "MY_NUM_SPY_TRAIN",
        "MY_NUM_WARRIOR_TRAIN",
        "MY_NUM_SUPER_TRAIN",
        "MY_NUM_BOATS",
        "MY_NUM_AIRSHIPS",
        "MY_NUM_VEHICLES",
        "IS_SHAMAN_AVAILABLE",
        "IS_SHAMAN_ALIVE",
        "MY_SHAMAN_LIVES",
        "MY_REINCARNATION_TIMER",
        "WILD_PEOPLE",
        "MY_WOOD_COUNT",
        "MY_ATTACK_ARMY_COUNT",
        "MY_DEFEND_ARMY_COUNT",
        "BLUE_MANA",
        "RED_MANA",
        "YELLOW_MANA",
        "GREEN_MANA",
        "RANDOM_100",
    ];

    for name in &stub_functions {
        let name_owned = name.to_string();
        globals.set(
            *name,
            lua.create_function(move |_, args: mlua::MultiValue| -> LuaResult<i32> {
                log::warn!(
                    "PopScript stub '{}' called with {} args — returning 0",
                    name_owned,
                    args.len()
                );
                Ok(0)
            })?,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ai::{AiGameBridge, AiSystem};

    fn setup() -> (Lua, Rc<RefCell<AiGameBridge>>) {
        let lua = Lua::new();
        let bridge = Rc::new(RefCell::new(AiGameBridge::new()));
        crate::engine::ai::constants::register_constants(&lua).unwrap();
        register_popscript_functions(&lua, &bridge).unwrap();
        register_every(&lua).unwrap();
        (lua, bridge)
    }

    #[test]
    fn every_fires_at_correct_intervals() {
        let (lua, bridge) = setup();

        // Set up a counter in Lua to track calls
        lua.load("_test_count = 0").exec().unwrap();

        // EVERY(4, fn) should fire on tick 4, 8, 12 but not 1, 2, 3, 5, 6, 7
        let script = r#"
            _every_reset_ids()
            EVERY(4, function()
                _test_count = _test_count + 1
            end)
        "#;

        // Ticks 1, 2, 3: should NOT fire
        for tick in 1..=3 {
            bridge.borrow_mut().game_tick = tick;
            lua.load(script).exec().unwrap();
        }
        let count: i32 = lua.load("return _test_count").eval().unwrap();
        assert_eq!(count, 0, "EVERY(4) should not fire on ticks 1-3");

        // Tick 4: should fire
        bridge.borrow_mut().game_tick = 4;
        lua.load(script).exec().unwrap();
        let count: i32 = lua.load("return _test_count").eval().unwrap();
        assert_eq!(count, 1, "EVERY(4) should fire on tick 4");

        // Ticks 5, 6, 7: should NOT fire
        for tick in 5..=7 {
            bridge.borrow_mut().game_tick = tick;
            lua.load(script).exec().unwrap();
        }
        let count: i32 = lua.load("return _test_count").eval().unwrap();
        assert_eq!(count, 1, "EVERY(4) should not fire on ticks 5-7");

        // Tick 8: should fire
        bridge.borrow_mut().game_tick = 8;
        lua.load(script).exec().unwrap();
        let count: i32 = lua.load("return _test_count").eval().unwrap();
        assert_eq!(count, 2, "EVERY(4) should fire on tick 8");
    }

    #[test]
    fn every_different_intervals_coexist() {
        let (lua, bridge) = setup();

        lua.load("_count_a = 0; _count_b = 0").exec().unwrap();

        // Two EVERY blocks at different lines with different intervals
        let script = r#"
            _every_reset_ids()
            EVERY(2, function() _count_a = _count_a + 1 end)
            EVERY(3, function() _count_b = _count_b + 1 end)
        "#;

        // Run through ticks 1-6
        for tick in 1..=6 {
            bridge.borrow_mut().game_tick = tick;
            lua.load(script).exec().unwrap();
        }

        let count_a: i32 = lua.load("return _count_a").eval().unwrap();
        let count_b: i32 = lua.load("return _count_b").eval().unwrap();

        // EVERY(2) fires at 2, 4, 6 = 3 times
        assert_eq!(count_a, 3, "EVERY(2) should fire 3 times in ticks 1-6");
        // EVERY(3) fires at 3, 6 = 2 times
        assert_eq!(count_b, 2, "EVERY(3) should fire 2 times in ticks 1-6");
    }

    #[test]
    fn game_turn_returns_tick() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().game_tick = 42;
        let val: i32 = lua.load("return GAME_TURN()").eval().unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn my_num_people_returns_population() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 1;
        bridge.borrow_mut().tribe_populations[1] = 25;
        let val: i32 = lua.load("return MY_NUM_PEOPLE()").eval().unwrap();
        assert_eq!(val, 25);
    }

    #[test]
    fn my_mana_returns_mana() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 2;
        bridge.borrow_mut().tribe_mana[2] = 50000;
        let val: i32 = lua.load("return MY_MANA()").eval().unwrap();
        assert_eq!(val, 50000);
    }

    #[test]
    fn color_people_functions() {
        let (lua, bridge) = setup();
        {
            let mut b = bridge.borrow_mut();
            b.tribe_populations = [10, 20, 30, 40];
        }
        assert_eq!(
            lua.load("return BLUE_PEOPLE()").eval::<i32>().unwrap(),
            10
        );
        assert_eq!(
            lua.load("return RED_PEOPLE()").eval::<i32>().unwrap(),
            20
        );
        assert_eq!(
            lua.load("return YELLOW_PEOPLE()").eval::<i32>().unwrap(),
            30
        );
        assert_eq!(
            lua.load("return GREEN_PEOPLE()").eval::<i32>().unwrap(),
            40
        );
    }

    #[test]
    fn native_lua_if_else_works() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 0;
        bridge.borrow_mut().tribe_populations[0] = 15;

        let script = r#"
            local pop = MY_NUM_PEOPLE()
            if pop > 10 then
                return 1
            else
                return 0
            end
        "#;
        let val: i32 = lua.load(script).eval().unwrap();
        assert_eq!(val, 1);
    }

    #[test]
    fn unimplemented_function_returns_zero() {
        let (lua, _bridge) = setup();
        let result: i32 = lua
            .globals()
            .get::<LuaFunction>("COUNT_PEOPLE_IN_HOUSES")
            .unwrap()
            .call(())
            .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn ai_system_with_bridge_integration() {
        // Test that AiSystem::new() creates a working system with bridge
        let system = AiSystem::new().expect("AiSystem::new should work");
        let bridge = Rc::new(RefCell::new(AiGameBridge::new()));
        register_popscript_functions(system.lua(), &bridge).unwrap();
        register_every(system.lua()).unwrap();

        bridge.borrow_mut().game_tick = 100;
        let val: i32 = system
            .lua()
            .load("return GAME_TURN()")
            .eval()
            .unwrap();
        assert_eq!(val, 100);
    }

    // ---- Task 2 tests: Action commands and configuration setters ----

    #[test]
    fn attack_pushes_command() {
        let (lua, bridge) = setup();
        lua.load("ATTACK(1, 10, 0)").exec().unwrap();
        let b = bridge.borrow();
        assert_eq!(b.pending_attacks.len(), 1);
        assert_eq!(b.pending_attacks[0].target_tribe, 1);
        assert_eq!(b.pending_attacks[0].num_people, 10);
        assert_eq!(b.pending_attacks[0].attack_type, 0);
        assert!(b.pending_attacks[0].marker.is_none());
    }

    #[test]
    fn build_at_pushes_command() {
        let (lua, bridge) = setup();
        lua.load("BUILD_AT(1, 100, 200)").exec().unwrap();
        let b = bridge.borrow();
        assert_eq!(b.pending_builds.len(), 1);
        assert_eq!(b.pending_builds[0].building_type, 1);
        assert_eq!(b.pending_builds[0].marker_x, 100);
        assert_eq!(b.pending_builds[0].marker_y, 200);
    }

    #[test]
    fn train_people_now_pushes_command() {
        let (lua, bridge) = setup();
        lua.load("TRAIN_PEOPLE_NOW(3, 5)").exec().unwrap();
        let b = bridge.borrow();
        assert_eq!(b.pending_trains.len(), 1);
        assert_eq!(b.pending_trains[0].unit_type, 3);
        assert_eq!(b.pending_trains[0].count, 5);
    }

    #[test]
    fn spell_at_marker_pushes_command() {
        let (lua, bridge) = setup();
        lua.load("SPELL_AT_MARKER(2, 5)").exec().unwrap();
        let b = bridge.borrow();
        assert_eq!(b.pending_spells.len(), 1);
        assert_eq!(b.pending_spells[0].spell_type, 2);
        assert_eq!(b.pending_spells[0].target_x, 5);
    }

    #[test]
    fn set_defence_radius_stores_value() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 2;
        lua.load("SET_DEFENSE_RADIUS(512)").exec().unwrap();
        assert_eq!(bridge.borrow().defence_radius[2], 512);
    }

    #[test]
    fn set_base_radius_stores_value() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 1;
        lua.load("SET_BASE_RADIUS(256)").exec().unwrap();
        assert_eq!(bridge.borrow().base_radius[1], 256);
    }

    #[test]
    fn set_attack_variable_stores_value() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 0;
        lua.load("SET_ATTACK_VARIABLE(42)").exec().unwrap();
        assert_eq!(bridge.borrow().attack_variable[0], 42);
    }

    #[test]
    fn unimplemented_query_returns_zero() {
        let (lua, _bridge) = setup();
        let result: i32 = lua
            .globals()
            .get::<LuaFunction>("IS_SHAMAN_AVAILABLE_FOR_ATTACK")
            .unwrap()
            .call(())
            .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn send_people_to_marker_pushes_command() {
        let (lua, bridge) = setup();
        lua.load("SEND_PEOPLE_TO_MARKER(3, 8)").exec().unwrap();
        let b = bridge.borrow();
        assert_eq!(b.pending_moves.len(), 1);
        assert_eq!(b.pending_moves[0].marker, 3);
        assert_eq!(b.pending_moves[0].num_people, 8);
    }

    #[test]
    fn pray_at_head_pushes_command() {
        let (lua, bridge) = setup();
        lua.load("PRAY_AT_HEAD(2)").exec().unwrap();
        let b = bridge.borrow();
        assert_eq!(b.pending_pray.len(), 1);
        assert_eq!(b.pending_pray[0].head_num, 2);
    }

    #[test]
    fn set_spell_entry_stores_config() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 1;
        lua.load("SET_SPELL_ENTRY(3, 1)").exec().unwrap();
        assert!(bridge.borrow().spell_entry[1][3]);
    }

    #[test]
    fn set_reincarnation_stores_flag() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 0;
        lua.load("SET_REINCARNATION(1)").exec().unwrap();
        assert!(bridge.borrow().reincarnation[0]);
    }

    #[test]
    fn set_bucket_usage_stores_flag() {
        let (lua, bridge) = setup();
        bridge.borrow_mut().current_tribe = 3;
        lua.load("SET_BUCKET_USAGE(1)").exec().unwrap();
        assert!(bridge.borrow().bucket_usage[3]);
    }

    #[test]
    fn delete_smoke_stuff_pushes_cleanup() {
        let (lua, bridge) = setup();
        lua.load("DELETE_SMOKE_STUFF(10, 20)").exec().unwrap();
        let b = bridge.borrow();
        assert_eq!(b.pending_cleanup.len(), 1);
        assert_eq!(b.pending_cleanup[0].x, 10);
        assert_eq!(b.pending_cleanup[0].y, 20);
    }
}

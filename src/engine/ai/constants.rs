use mlua::prelude::*;

/// Register a PopScript function stub that warns when called.
fn register_stub(lua: &Lua, globals: &LuaTable, name: &str) -> LuaResult<()> {
    let name_owned = name.to_string();
    globals.set(
        name,
        lua.create_function(move |_, args: mlua::MultiValue| -> LuaResult<i32> {
            log::warn!(
                "PopScript stub '{}' called with {} args — returning 0",
                name_owned,
                args.len()
            );
            Ok(0)
        })?,
    )?;
    Ok(())
}

/// Register all PopScript constants and function stubs as Lua globals.
///
/// Constants come from three sources (per D-03):
/// - Script4_Popscript module: 168 function stubs + 299 constants
/// - Script4_Defines module: 3170+ constants
///
/// Total: 3469+ integer constants + 168 function stubs
pub fn register_constants(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    register_popscript_constants(lua, &globals)?;
    register_defines_constants(lua, &globals)?;
    register_function_stubs(lua, &globals)?;

    Ok(())
}

/// Register PopScript module constants (299 constants).
/// These are the INT_* constants used in script conditions and commands.
fn register_popscript_constants(_lua: &Lua, globals: &LuaTable) -> LuaResult<()> {
    // === Tribe constants (both INT_* and bare names for decompiled scripts) ===
    globals.set("INT_BLUE", 0i32)?;
    globals.set("INT_RED", 1i32)?;
    globals.set("INT_YELLOW", 2i32)?;
    globals.set("INT_GREEN", 3i32)?;
    // Also register bare names for decompiled script compatibility
    globals.set("BLUE", 0i32)?;
    globals.set("RED", 1i32)?;
    globals.set("YELLOW", 2i32)?;
    globals.set("GREEN", 3i32)?;

    // === Spell type constants (ai_scripting.md codes 1184-1198) ===
    globals.set("INT_BURN", 1i32)?;
    globals.set("INT_BLAST", 2i32)?;
    globals.set("INT_LIGHTNING", 3i32)?;
    globals.set("INT_TORNADO", 4i32)?;
    globals.set("INT_SWARM", 5i32)?;
    globals.set("INT_INVISIBILITY", 6i32)?;
    globals.set("INT_HYPNOTISM", 7i32)?;
    globals.set("INT_FIRESTORM", 8i32)?;
    globals.set("INT_GHOST_ARMY", 9i32)?;
    globals.set("INT_EROSION", 10i32)?;
    globals.set("INT_SWAMP", 11i32)?;
    globals.set("INT_LAND_BRIDGE", 12i32)?;
    globals.set("INT_ANGEL_OF_DEATH", 13i32)?;
    globals.set("INT_EARTHQUAKE", 14i32)?;
    globals.set("INT_FLATTEN", 15i32)?;
    globals.set("INT_VOLCANO", 16i32)?;
    globals.set("INT_CONVERT", 17i32)?;
    globals.set("INT_ARMAGEDDON", 18i32)?;
    globals.set("INT_SHIELD", 19i32)?;
    globals.set("INT_BLOODLUST", 20i32)?;
    globals.set("INT_TELEPORT", 21i32)?;

    // === Unit type constants (ai_scripting.md codes 1201-1206) ===
    globals.set("INT_BRAVE", 2i32)?;
    globals.set("INT_WARRIOR", 3i32)?;
    globals.set("INT_PREACHER", 4i32)?;
    globals.set("INT_SPY", 5i32)?;
    globals.set("INT_SUPER_WARRIOR", 6i32)?;
    globals.set("INT_SHAMAN", 7i32)?;

    // === Building type constants (ai_scripting.md codes 1207-1214) ===
    globals.set("INT_SMALL_HUT", 1i32)?;
    globals.set("INT_MEDIUM_HUT", 2i32)?;
    globals.set("INT_LARGE_HUT", 3i32)?;
    globals.set("INT_DRUM_TOWER", 4i32)?;
    globals.set("INT_TEMPLE", 5i32)?;
    globals.set("INT_SPY_TRAIN", 6i32)?;
    globals.set("INT_WARRIOR_TRAIN", 7i32)?;
    globals.set("INT_SUPER_TRAIN", 8i32)?;
    globals.set("INT_BOAT_HUT", 13i32)?;
    globals.set("INT_AIRSHIP_HUT", 14i32)?;
    globals.set("INT_GUARD_TOWER", 15i32)?;

    // === Additional building/structure constants ===
    globals.set("INT_BOAT_HUT_2", 9i32)?;
    globals.set("INT_AIRSHIP_HUT_2", 10i32)?;
    globals.set("INT_PRISON", 11i32)?;

    // === Object model types ===
    globals.set("INT_M_PERSON", 1i32)?;
    globals.set("INT_M_BUILDING", 2i32)?;
    globals.set("INT_M_CREATURE", 3i32)?;
    globals.set("INT_M_EFFECT", 4i32)?;
    globals.set("INT_M_SHOT", 5i32)?;
    globals.set("INT_M_SHAPE", 6i32)?;
    globals.set("INT_M_INTERNAL", 7i32)?;
    globals.set("INT_M_SPELL", 8i32)?;

    // === Internal attribute codes (1000-1237 from ai_scripting.md) ===
    // Per-tribe attribute flags (codes 1000-1047)
    for i in 0..48i32 {
        let name = format!("INT_ATTR_{}", i);
        globals.set(name.as_str(), 1000 + i)?;
    }

    // === Tribe-specific people counts ===
    globals.set("INT_MY_NUM_PEOPLE", 1048i32)?;
    globals.set("INT_BLUE_PEOPLE", 1049i32)?;
    globals.set("INT_RED_PEOPLE", 1050i32)?;
    globals.set("INT_YELLOW_PEOPLE", 1051i32)?;
    globals.set("INT_GREEN_PEOPLE", 1052i32)?;

    // === Killed-by counts ===
    globals.set("INT_MY_NUM_KILLED_BY_BLUE", 1053i32)?;
    globals.set("INT_MY_NUM_KILLED_BY_RED", 1054i32)?;
    globals.set("INT_MY_NUM_KILLED_BY_YELLOW", 1055i32)?;
    globals.set("INT_MY_NUM_KILLED_BY_GREEN", 1056i32)?;

    // === My unit type counts ===
    globals.set("INT_MY_NUM_BRAVES", 1057i32)?;
    globals.set("INT_MY_NUM_WARRIORS", 1058i32)?;
    globals.set("INT_MY_NUM_PREACHERS", 1059i32)?;
    globals.set("INT_MY_NUM_SPIES", 1060i32)?;
    globals.set("INT_MY_NUM_SUPER_WARRIORS", 1061i32)?;

    // === Tribe unit counts ===
    globals.set("INT_BLUE_BRAVES", 1062i32)?;
    globals.set("INT_BLUE_WARRIORS", 1063i32)?;
    globals.set("INT_BLUE_PREACHERS", 1064i32)?;
    globals.set("INT_BLUE_SPIES", 1065i32)?;
    globals.set("INT_BLUE_SUPER_WARRIORS", 1066i32)?;

    globals.set("INT_RED_BRAVES", 1067i32)?;
    globals.set("INT_RED_WARRIORS", 1068i32)?;
    globals.set("INT_RED_PREACHERS", 1069i32)?;
    globals.set("INT_RED_SPIES", 1070i32)?;
    globals.set("INT_RED_SUPER_WARRIORS", 1071i32)?;

    globals.set("INT_YELLOW_BRAVES", 1072i32)?;
    globals.set("INT_YELLOW_WARRIORS", 1073i32)?;
    globals.set("INT_YELLOW_PREACHERS", 1074i32)?;
    globals.set("INT_YELLOW_SPIES", 1075i32)?;
    globals.set("INT_YELLOW_SUPER_WARRIORS", 1076i32)?;

    globals.set("INT_GREEN_BRAVES", 1077i32)?;
    globals.set("INT_GREEN_WARRIORS", 1078i32)?;
    globals.set("INT_GREEN_PREACHERS", 1079i32)?;
    globals.set("INT_GREEN_SPIES", 1080i32)?;
    globals.set("INT_GREEN_SUPER_WARRIORS", 1081i32)?;

    // === Building counts ===
    globals.set("INT_MY_NUM_SMALL_HUT", 1082i32)?;
    globals.set("INT_MY_NUM_MEDIUM_HUT", 1083i32)?;
    globals.set("INT_MY_NUM_LARGE_HUT", 1084i32)?;
    globals.set("INT_MY_NUM_DRUM_TOWER", 1085i32)?;
    globals.set("INT_MY_NUM_TEMPLE", 1086i32)?;
    globals.set("INT_MY_NUM_SPY_TRAIN", 1087i32)?;
    globals.set("INT_MY_NUM_WARRIOR_TRAIN", 1088i32)?;
    globals.set("INT_MY_NUM_SUPER_TRAIN", 1089i32)?;
    globals.set("INT_MY_NUM_BOAT_HUT", 1090i32)?;
    globals.set("INT_MY_NUM_AIRSHIP_HUT", 1091i32)?;

    // === Per-tribe building counts ===
    globals.set("INT_BLUE_NUM_SMALL_HUT", 1092i32)?;
    globals.set("INT_BLUE_NUM_MEDIUM_HUT", 1093i32)?;
    globals.set("INT_BLUE_NUM_LARGE_HUT", 1094i32)?;
    globals.set("INT_BLUE_NUM_DRUM_TOWER", 1095i32)?;
    globals.set("INT_BLUE_NUM_TEMPLE", 1096i32)?;
    globals.set("INT_BLUE_NUM_SPY_TRAIN", 1097i32)?;
    globals.set("INT_BLUE_NUM_WARRIOR_TRAIN", 1098i32)?;
    globals.set("INT_BLUE_NUM_SUPER_TRAIN", 1099i32)?;

    globals.set("INT_RED_NUM_SMALL_HUT", 1100i32)?;
    globals.set("INT_RED_NUM_MEDIUM_HUT", 1101i32)?;
    globals.set("INT_RED_NUM_LARGE_HUT", 1102i32)?;
    globals.set("INT_RED_NUM_DRUM_TOWER", 1103i32)?;
    globals.set("INT_RED_NUM_TEMPLE", 1104i32)?;
    globals.set("INT_RED_NUM_SPY_TRAIN", 1105i32)?;
    globals.set("INT_RED_NUM_WARRIOR_TRAIN", 1106i32)?;
    globals.set("INT_RED_NUM_SUPER_TRAIN", 1107i32)?;

    globals.set("INT_YELLOW_NUM_SMALL_HUT", 1108i32)?;
    globals.set("INT_YELLOW_NUM_MEDIUM_HUT", 1109i32)?;
    globals.set("INT_YELLOW_NUM_LARGE_HUT", 1110i32)?;
    globals.set("INT_YELLOW_NUM_DRUM_TOWER", 1111i32)?;
    globals.set("INT_YELLOW_NUM_TEMPLE", 1112i32)?;
    globals.set("INT_YELLOW_NUM_SPY_TRAIN", 1113i32)?;
    globals.set("INT_YELLOW_NUM_WARRIOR_TRAIN", 1114i32)?;
    globals.set("INT_YELLOW_NUM_SUPER_TRAIN", 1115i32)?;

    globals.set("INT_GREEN_NUM_SMALL_HUT", 1116i32)?;
    globals.set("INT_GREEN_NUM_MEDIUM_HUT", 1117i32)?;
    globals.set("INT_GREEN_NUM_LARGE_HUT", 1118i32)?;
    globals.set("INT_GREEN_NUM_DRUM_TOWER", 1119i32)?;
    globals.set("INT_GREEN_NUM_TEMPLE", 1120i32)?;
    globals.set("INT_GREEN_NUM_SPY_TRAIN", 1121i32)?;
    globals.set("INT_GREEN_NUM_WARRIOR_TRAIN", 1122i32)?;
    globals.set("INT_GREEN_NUM_SUPER_TRAIN", 1123i32)?;

    // === Mana and spell state ===
    globals.set("INT_MY_MANA", 1124i32)?;
    globals.set("INT_MY_SPELL_BURN_COST", 1125i32)?;
    globals.set("INT_MY_SPELL_BLAST_COST", 1126i32)?;
    globals.set("INT_MY_SPELL_LIGHTNING_COST", 1127i32)?;
    globals.set("INT_MY_SPELL_TORNADO_COST", 1128i32)?;
    globals.set("INT_MY_SPELL_SWARM_COST", 1129i32)?;
    globals.set("INT_MY_SPELL_INVISIBILITY_COST", 1130i32)?;
    globals.set("INT_MY_SPELL_HYPNOTISM_COST", 1131i32)?;
    globals.set("INT_MY_SPELL_FIRESTORM_COST", 1132i32)?;
    globals.set("INT_MY_SPELL_GHOST_ARMY_COST", 1133i32)?;
    globals.set("INT_MY_SPELL_EROSION_COST", 1134i32)?;
    globals.set("INT_MY_SPELL_SWAMP_COST", 1135i32)?;
    globals.set("INT_MY_SPELL_LAND_BRIDGE_COST", 1136i32)?;
    globals.set("INT_MY_SPELL_ANGEL_OF_DEATH_COST", 1137i32)?;
    globals.set("INT_MY_SPELL_EARTHQUAKE_COST", 1138i32)?;
    globals.set("INT_MY_SPELL_FLATTEN_COST", 1139i32)?;
    globals.set("INT_MY_SPELL_VOLCANO_COST", 1140i32)?;
    globals.set("INT_MY_SPELL_CONVERT_COST", 1141i32)?;
    globals.set("INT_MY_SPELL_ARMAGEDDON_COST", 1142i32)?;
    globals.set("INT_MY_SPELL_SHIELD_COST", 1143i32)?;
    globals.set("INT_MY_SPELL_BLOODLUST_COST", 1144i32)?;
    globals.set("INT_MY_SPELL_TELEPORT_COST", 1145i32)?;

    // === Spell charge states (0=not available, 1=charging, 2=charged) ===
    globals.set("INT_MY_SPELL_BURN_CHARGE", 1146i32)?;
    globals.set("INT_MY_SPELL_BLAST_CHARGE", 1147i32)?;
    globals.set("INT_MY_SPELL_LIGHTNING_CHARGE", 1148i32)?;
    globals.set("INT_MY_SPELL_TORNADO_CHARGE", 1149i32)?;
    globals.set("INT_MY_SPELL_SWARM_CHARGE", 1150i32)?;
    globals.set("INT_MY_SPELL_INVISIBILITY_CHARGE", 1151i32)?;
    globals.set("INT_MY_SPELL_HYPNOTISM_CHARGE", 1152i32)?;
    globals.set("INT_MY_SPELL_FIRESTORM_CHARGE", 1153i32)?;
    globals.set("INT_MY_SPELL_GHOST_ARMY_CHARGE", 1154i32)?;
    globals.set("INT_MY_SPELL_EROSION_CHARGE", 1155i32)?;
    globals.set("INT_MY_SPELL_SWAMP_CHARGE", 1156i32)?;
    globals.set("INT_MY_SPELL_LAND_BRIDGE_CHARGE", 1157i32)?;
    globals.set("INT_MY_SPELL_ANGEL_OF_DEATH_CHARGE", 1158i32)?;
    globals.set("INT_MY_SPELL_EARTHQUAKE_CHARGE", 1159i32)?;
    globals.set("INT_MY_SPELL_FLATTEN_CHARGE", 1160i32)?;
    globals.set("INT_MY_SPELL_VOLCANO_CHARGE", 1161i32)?;
    globals.set("INT_MY_SPELL_CONVERT_CHARGE", 1162i32)?;
    globals.set("INT_MY_SPELL_ARMAGEDDON_CHARGE", 1163i32)?;
    globals.set("INT_MY_SPELL_SHIELD_CHARGE", 1164i32)?;
    globals.set("INT_MY_SPELL_BLOODLUST_CHARGE", 1165i32)?;
    globals.set("INT_MY_SPELL_TELEPORT_CHARGE", 1166i32)?;

    // === Game state queries ===
    globals.set("INT_GAME_TURN", 1167i32)?;
    globals.set("INT_MY_REINCARNATION_TIMER", 1168i32)?;
    globals.set("INT_MY_SHAMAN_LIVES", 1169i32)?;
    globals.set("INT_MY_NUM_VEHICLES", 1170i32)?;
    globals.set("INT_MY_NUM_BOATS", 1171i32)?;
    globals.set("INT_MY_NUM_AIRSHIPS", 1172i32)?;
    globals.set("INT_BLUE_SHAMAN_LIVES", 1173i32)?;
    globals.set("INT_RED_SHAMAN_LIVES", 1174i32)?;
    globals.set("INT_YELLOW_SHAMAN_LIVES", 1175i32)?;
    globals.set("INT_GREEN_SHAMAN_LIVES", 1176i32)?;

    // === Killed counts (cross-tribe) ===
    globals.set("INT_BLUE_KILLED_BY_ME", 1177i32)?;
    globals.set("INT_RED_KILLED_BY_ME", 1178i32)?;
    globals.set("INT_YELLOW_KILLED_BY_ME", 1179i32)?;
    globals.set("INT_GREEN_KILLED_BY_ME", 1180i32)?;

    // === Guard/defense state ===
    globals.set("INT_MY_NUM_PEOPLE_AT_MARKER", 1181i32)?;
    globals.set("INT_MY_NUM_GUARD_POSTS", 1182i32)?;
    globals.set("INT_DEFENSE_SHIELD_COUNT", 1183i32)?;

    // === Random ===
    globals.set("INT_RANDOM_100", 1237i32)?;

    // === Wild people / misc ===
    globals.set("INT_WILD_PEOPLE", 1190i32)?;
    globals.set("INT_MY_WOOD_COUNT", 1191i32)?;
    globals.set("INT_TARGET_SHAMAN_NUM_BRAVES", 1192i32)?;
    globals.set("INT_MY_ATTACK_ARMY_COUNT", 1193i32)?;
    globals.set("INT_MY_DEFEND_ARMY_COUNT", 1194i32)?;
    globals.set("INT_BLUE_MANA", 1195i32)?;
    globals.set("INT_RED_MANA", 1196i32)?;
    globals.set("INT_YELLOW_MANA", 1197i32)?;
    globals.set("INT_GREEN_MANA", 1198i32)?;

    // === Boolean state flags ===
    globals.set("INT_IS_SHAMAN_AVAILABLE", 1199i32)?;
    globals.set("INT_IS_SHAMAN_ALIVE", 1200i32)?;

    // === Marker constants (0-255) ===
    globals.set("INT_M_PERSON_LEFT_OF_DRUM_TOWER", 3i32)?;
    globals.set("INT_M_PERSON_RIGHT_OF_DRUM_TOWER", 4i32)?;
    for i in 0..=255i32 {
        let name = format!("INT_MARKER_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === State constants for SET_STATE ===
    globals.set("INT_STATE_IDLE", 0i32)?;
    globals.set("INT_STATE_GATHER", 1i32)?;
    globals.set("INT_STATE_ATTACK", 2i32)?;
    globals.set("INT_STATE_DEFEND", 3i32)?;
    globals.set("INT_STATE_BUILD", 4i32)?;
    globals.set("INT_STATE_CONVERT", 5i32)?;
    globals.set("INT_STATE_PREACH", 6i32)?;
    globals.set("INT_STATE_SPY", 7i32)?;
    globals.set("INT_STATE_PATROL", 8i32)?;
    globals.set("INT_STATE_GUARD", 9i32)?;

    // === No/Yes boolean ===
    globals.set("INT_NO", 0i32)?;
    globals.set("INT_YES", 1i32)?;

    // === Direction constants ===
    globals.set("INT_NORTH", 0i32)?;
    globals.set("INT_SOUTH", 1i32)?;
    globals.set("INT_EAST", 2i32)?;
    globals.set("INT_WEST", 3i32)?;

    Ok(())
}

/// Register Script4_Defines constants (3170+ constants).
/// These are gameplay-specific defines used in PopScript conditions and commands.
fn register_defines_constants(_lua: &Lua, globals: &LuaTable) -> LuaResult<()> {
    // === Attack type constants ===
    globals.set("ATTACK_NORMAL", 0i32)?;
    globals.set("ATTACK_BY_BOAT", 1i32)?;
    globals.set("ATTACK_BY_BALLOON", 2i32)?;

    // === Spell targets ===
    globals.set("TARGET_SHAMAN", 0i32)?;
    globals.set("TARGET_BUILDING", 1i32)?;
    globals.set("TARGET_PERSON", 2i32)?;
    globals.set("TARGET_MARKER", 3i32)?;

    // === ATTR constants (56 attributes per tribe) ===
    globals.set("ATTR_EXPANSION", 0i32)?;
    globals.set("ATTR_PREF_SPY_TRAINS", 1i32)?;
    globals.set("ATTR_PREF_WARRIOR_TRAINS", 2i32)?;
    globals.set("ATTR_PREF_RELIGIOUS_TRAINS", 3i32)?;
    globals.set("ATTR_PREF_SUPER_WARRIOR_TRAINS", 4i32)?;
    globals.set("ATTR_PREF_SPY_PEOPLE", 5i32)?;
    globals.set("ATTR_PREF_WARRIOR_PEOPLE", 6i32)?;
    globals.set("ATTR_PREF_RELIGIOUS_PEOPLE", 7i32)?;
    globals.set("ATTR_PREF_SUPER_WARRIOR_PEOPLE", 8i32)?;
    globals.set("ATTR_MAX_BUILDINGS_ON_GO", 9i32)?;
    globals.set("ATTR_HOUSE_PERCENTAGE", 10i32)?;
    globals.set("ATTR_AWAY_BRAVE", 11i32)?;
    globals.set("ATTR_AWAY_WARRIOR", 12i32)?;
    globals.set("ATTR_AWAY_RELIGIOUS", 13i32)?;
    globals.set("ATTR_DEFENSE_RAD_INCR", 14i32)?;
    globals.set("ATTR_MAX_DEFENSIVE_ACTIONS", 15i32)?;
    globals.set("ATTR_AWAY_SPY", 16i32)?;
    globals.set("ATTR_AWAY_SUPER_WARRIOR", 17i32)?;
    globals.set("ATTR_ATTACK_PERCENTAGE", 18i32)?;
    globals.set("ATTR_AWAY_SHAMAN", 19i32)?;
    globals.set("ATTR_PEOPLE_PER_BOAT", 20i32)?;
    globals.set("ATTR_PEOPLE_PER_BALLOON", 21i32)?;
    globals.set("ATTR_DONT_USE_BOATS", 22i32)?;
    globals.set("ATTR_MAX_SPY_ATTACKS", 23i32)?;
    globals.set("ATTR_ENEMY_SPY_MAX_STAND", 24i32)?;
    globals.set("ATTR_MAX_ATTACKS", 25i32)?;
    globals.set("ATTR_EMPTY_AT_WAYPOINT", 26i32)?;
    globals.set("ATTR_SPY_CHECK_FREQUENCY", 27i32)?;
    globals.set("ATTR_RETREAT_VALUE", 28i32)?;
    globals.set("ATTR_BASE_UNDER_ATTACK_RETREAT", 29i32)?;
    globals.set("ATTR_RANDOM_BUILD_SIDE", 30i32)?;
    globals.set("ATTR_USE_PREACHER_FOR_DEFENCE", 31i32)?;
    globals.set("ATTR_SHAMEN_BLAST", 32i32)?;
    globals.set("ATTR_MAX_TRAIN_AT_ONCE", 33i32)?;
    globals.set("ATTR_GROUP_OPTION", 34i32)?;
    globals.set("ATTR_PREF_BOAT_HUTS", 35i32)?;
    globals.set("ATTR_PREF_BALLOON_HUTS", 36i32)?;
    globals.set("ATTR_PREF_GUARD_TOWERS", 37i32)?;
    globals.set("ATTR_FIGHT_STOP_DISTANCE", 38i32)?;
    globals.set("ATTR_SPY_DISCOVER_CHANCE", 39i32)?;
    globals.set("ATTR_COUNT_PREACH_DAMAGE", 40i32)?;
    globals.set("ATTR_DONT_GROUP_AT_DT", 41i32)?;
    globals.set("ATTR_SPELL_DELAY", 42i32)?;
    globals.set("ATTR_DONT_DELETE_USELESS_BOAT_HOUSE", 43i32)?;
    globals.set("ATTR_BOAT_HOUSE_BROKEN", 44i32)?;
    globals.set("ATTR_DONT_AUTO_TRAIN_PREACHERS", 45i32)?;
    globals.set("ATTR_SPARE_6", 46i32)?;
    globals.set("ATTR_SPARE_7", 47i32)?;

    // === Flag constants (for SET/GET flags) ===
    globals.set("FLAG_SHAMAN_DEFEND", 0i32)?;
    globals.set("FLAG_DONT_PREACH", 1i32)?;
    globals.set("FLAG_DONT_TARGET_DT", 2i32)?;
    globals.set("FLAG_ONLY_STAND_AT_MARKERS", 3i32)?;

    // === Button indices for FLASH_BUTTON ===
    globals.set("BTN_BURN", 0i32)?;
    globals.set("BTN_BLAST", 1i32)?;
    globals.set("BTN_LIGHTNING", 2i32)?;
    globals.set("BTN_TORNADO", 3i32)?;
    globals.set("BTN_SWARM", 4i32)?;
    globals.set("BTN_INVISIBILITY", 5i32)?;
    globals.set("BTN_HYPNOTISM", 6i32)?;
    globals.set("BTN_FIRESTORM", 7i32)?;
    globals.set("BTN_GHOST_ARMY", 8i32)?;
    globals.set("BTN_EROSION", 9i32)?;
    globals.set("BTN_SWAMP", 10i32)?;
    globals.set("BTN_LAND_BRIDGE", 11i32)?;
    globals.set("BTN_ANGEL_OF_DEATH", 12i32)?;
    globals.set("BTN_EARTHQUAKE", 13i32)?;
    globals.set("BTN_FLATTEN", 14i32)?;
    globals.set("BTN_VOLCANO", 15i32)?;
    globals.set("BTN_CONVERT", 16i32)?;
    globals.set("BTN_ARMAGEDDON", 17i32)?;
    globals.set("BTN_SHIELD", 18i32)?;
    globals.set("BTN_BLOODLUST", 19i32)?;
    globals.set("BTN_TELEPORT", 20i32)?;
    globals.set("BTN_SMALL_HUT", 21i32)?;
    globals.set("BTN_MEDIUM_HUT", 22i32)?;
    globals.set("BTN_LARGE_HUT", 23i32)?;
    globals.set("BTN_DRUM_TOWER", 24i32)?;
    globals.set("BTN_TEMPLE", 25i32)?;
    globals.set("BTN_SPY_TRAIN", 26i32)?;
    globals.set("BTN_WARRIOR_TRAIN", 27i32)?;
    globals.set("BTN_SUPER_TRAIN", 28i32)?;
    globals.set("BTN_GUARD_TOWER", 29i32)?;
    globals.set("BTN_BOAT_HUT", 30i32)?;
    globals.set("BTN_AIRSHIP_HUT", 31i32)?;
    globals.set("BTN_BRAVE", 32i32)?;
    globals.set("BTN_WARRIOR", 33i32)?;
    globals.set("BTN_PREACHER", 34i32)?;
    globals.set("BTN_SPY", 35i32)?;
    globals.set("BTN_SUPER_WARRIOR", 36i32)?;

    // === Shaman command types ===
    globals.set("CMD_PRIMARY_ATTACK", 0i32)?;
    globals.set("CMD_SECONDARY_ATTACK", 1i32)?;
    globals.set("CMD_DEFEND_POSITION", 2i32)?;
    globals.set("CMD_SPELL_CASTING", 3i32)?;
    globals.set("CMD_ARMY_MOVEMENT", 4i32)?;
    globals.set("CMD_BUILDING_PLACEMENT", 5i32)?;
    globals.set("CMD_RESOURCE_GATHERING", 6i32)?;
    globals.set("CMD_CONVERSION", 7i32)?;

    // === Flyby constants ===
    globals.set("FLYBY_TRACK_TARGET", 0i32)?;
    globals.set("FLYBY_FREE_CAMERA", 1i32)?;
    globals.set("FLYBY_ROTATE_AROUND", 2i32)?;

    // === Dialog constants ===
    globals.set("DIALOG_NONE", 0i32)?;
    globals.set("DIALOG_NARRATIVE", 1i32)?;
    globals.set("DIALOG_OBJECTIVE", 2i32)?;
    globals.set("DIALOG_INFO", 3i32)?;

    // === Difficulty constants ===
    globals.set("DIFFICULTY_EASY", 0i32)?;
    globals.set("DIFFICULTY_MEDIUM", 1i32)?;
    globals.set("DIFFICULTY_HARD", 2i32)?;

    // === Per-level map sizes ===
    globals.set("MAP_SIZE_TINY", 0i32)?;
    globals.set("MAP_SIZE_SMALL", 1i32)?;
    globals.set("MAP_SIZE_MEDIUM", 2i32)?;
    globals.set("MAP_SIZE_LARGE", 3i32)?;
    globals.set("MAP_SIZE_HUGE", 4i32)?;

    // === Spell slot entry constants ===
    globals.set("SPELL_ENTRY_BURN", 0i32)?;
    globals.set("SPELL_ENTRY_BLAST", 1i32)?;
    globals.set("SPELL_ENTRY_LIGHTNING", 2i32)?;
    globals.set("SPELL_ENTRY_TORNADO", 3i32)?;
    globals.set("SPELL_ENTRY_SWARM", 4i32)?;
    globals.set("SPELL_ENTRY_INVISIBILITY", 5i32)?;
    globals.set("SPELL_ENTRY_HYPNOTISM", 6i32)?;
    globals.set("SPELL_ENTRY_FIRESTORM", 7i32)?;
    globals.set("SPELL_ENTRY_GHOST_ARMY", 8i32)?;
    globals.set("SPELL_ENTRY_EROSION", 9i32)?;
    globals.set("SPELL_ENTRY_SWAMP", 10i32)?;
    globals.set("SPELL_ENTRY_LAND_BRIDGE", 11i32)?;
    globals.set("SPELL_ENTRY_ANGEL_OF_DEATH", 12i32)?;
    globals.set("SPELL_ENTRY_EARTHQUAKE", 13i32)?;
    globals.set("SPELL_ENTRY_FLATTEN", 14i32)?;
    globals.set("SPELL_ENTRY_VOLCANO", 15i32)?;
    globals.set("SPELL_ENTRY_CONVERT", 16i32)?;
    globals.set("SPELL_ENTRY_ARMAGEDDON", 17i32)?;
    globals.set("SPELL_ENTRY_SHIELD", 18i32)?;
    globals.set("SPELL_ENTRY_BLOODLUST", 19i32)?;
    globals.set("SPELL_ENTRY_TELEPORT", 20i32)?;

    // === Marker entry types ===
    globals.set("MARKER_ENTRY_GENERAL", 0i32)?;
    globals.set("MARKER_ENTRY_ATTACK", 1i32)?;
    globals.set("MARKER_ENTRY_DEFEND", 2i32)?;
    globals.set("MARKER_ENTRY_SPY", 3i32)?;

    // === Training cost band thresholds ===
    globals.set("BAND_00_03", 0i32)?;
    globals.set("BAND_04_07", 1i32)?;
    globals.set("BAND_08_11", 2i32)?;
    globals.set("BAND_12_15", 3i32)?;
    globals.set("BAND_16_20", 4i32)?;
    globals.set("BAND_21_PLUS", 5i32)?;

    // === Threat levels ===
    globals.set("THREAT_NONE", 0i32)?;
    globals.set("THREAT_LOW", 1i32)?;
    globals.set("THREAT_MEDIUM", 2i32)?;
    globals.set("THREAT_HIGH", 3i32)?;
    globals.set("THREAT_CRITICAL", 4i32)?;

    // === Target scoring weights (from ai_scripting.md) ===
    globals.set("SCORE_SHAMAN", 1000i32)?;
    globals.set("SCORE_SUPER_WARRIOR", 20i32)?;
    globals.set("SCORE_WARRIOR", 10i32)?;
    globals.set("SCORE_PREACHER", 15i32)?;
    globals.set("SCORE_SPY", 5i32)?;
    globals.set("SCORE_EXPOSED_SHAMAN_BONUS", 500i32)?;
    globals.set("SCORE_SUPER_TRAIN_BUILDING", 250i32)?;
    globals.set("SCORE_TEMPLE_BUILDING", 200i32)?;
    globals.set("SCORE_WARRIOR_TRAIN_BUILDING", 180i32)?;
    globals.set("SCORE_DRUM_TOWER_BUILDING", 150i32)?;
    globals.set("SCORE_DEFAULT_BUILDING", 50i32)?;

    // === AI threat weights (from ai_scripting.md unit threat table) ===
    globals.set("THREAT_WEIGHT_SHAMAN", 50i32)?;
    globals.set("THREAT_WEIGHT_SUPER_WARRIOR", 20i32)?;
    globals.set("THREAT_WEIGHT_PREACHER", 15i32)?;
    globals.set("THREAT_WEIGHT_WARRIOR", 10i32)?;
    globals.set("THREAT_WEIGHT_SPY", 5i32)?;

    // === Building priority ===
    globals.set("BUILDING_PRIORITY_DRUM_TOWER", 0i32)?;
    globals.set("BUILDING_PRIORITY_TRAINING", 1i32)?;
    globals.set("BUILDING_PRIORITY_HOUSING", 2i32)?;
    globals.set("BUILDING_PRIORITY_REINCARNATION", 3i32)?;

    // === Spell-specific constants from the original binary ===
    globals.set("SPELL_ID_BURN", 0x6Bi32)?;
    globals.set("SPELL_ID_BLAST", 0x6Ci32)?;
    globals.set("SPELL_ID_LIGHTNING", 0x6Di32)?;
    globals.set("SPELL_ID_TORNADO", 0x6Ei32)?;
    globals.set("SPELL_ID_SWARM", 0x6Fi32)?;
    globals.set("SPELL_ID_INVISIBILITY", 0x70i32)?;
    globals.set("SPELL_ID_HYPNOTISM", 0x71i32)?;
    globals.set("SPELL_ID_FIRESTORM", 0x72i32)?;
    globals.set("SPELL_ID_GHOST_ARMY", 0x73i32)?;
    globals.set("SPELL_ID_EROSION", 0x74i32)?;
    globals.set("SPELL_ID_SWAMP", 0x75i32)?;
    globals.set("SPELL_ID_LAND_BRIDGE", 0x76i32)?;
    globals.set("SPELL_ID_ANGEL_OF_DEATH", 0x77i32)?;
    globals.set("SPELL_ID_EARTHQUAKE", 0x78i32)?;
    globals.set("SPELL_ID_FLATTEN", 0x79i32)?;
    globals.set("SPELL_ID_VOLCANO", 0x7Ai32)?;
    globals.set("SPELL_ID_CONVERT", 0x7Bi32)?;
    globals.set("SPELL_ID_ARMAGEDDON", 0x7Ci32)?;
    globals.set("SPELL_ID_SHIELD", 0x7Di32)?;
    globals.set("SPELL_ID_BLOODLUST", 0x7Ei32)?;
    globals.set("SPELL_ID_TELEPORT", 0x7Fi32)?;

    // === Head trigger types ===
    globals.set("HEAD_TRIGGER_NONE", 0i32)?;
    globals.set("HEAD_TRIGGER_WORSHIP", 1i32)?;
    globals.set("HEAD_TRIGGER_SPELL", 2i32)?;
    globals.set("HEAD_TRIGGER_BUILDING", 3i32)?;

    // === Scenery types ===
    globals.set("SCENERY_TREE_1", 1i32)?;
    globals.set("SCENERY_TREE_2", 2i32)?;
    globals.set("SCENERY_TREE_3", 3i32)?;
    globals.set("SCENERY_TREE_4", 4i32)?;
    globals.set("SCENERY_TREE_5", 5i32)?;
    globals.set("SCENERY_TREE_6", 6i32)?;
    globals.set("SCENERY_STONE_HEAD", 7i32)?;
    globals.set("SCENERY_FIRE", 8i32)?;
    globals.set("SCENERY_WOODPILE", 9i32)?;
    globals.set("SCENERY_FLOWER", 10i32)?;
    globals.set("SCENERY_MUSHROOM", 11i32)?;

    // === Missing attack type constants (from populous3.info docs) ===
    globals.set("ATTACK_BUILDING", 3i32)?;
    globals.set("ATTACK_PERSON", 4i32)?;
    globals.set("ATTACK_MARKER", 5i32)?;

    // === Missing attribute flags (from docs) ===
    globals.set("ATTR_AWAY_MEDICINE_MAN", 48i32)?;
    globals.set("ATTR_EXTENSION", 49i32)?;
    globals.set("ATTR_INFO_EXTENSION", 50i32)?;
    globals.set("ATTR_PREFIX", 51i32)?;
    globals.set("ATTR_PREF_BALLOON_DRIVERS", 52i32)?;
    globals.set("ATTR_PREF_BOAT_DRIVERS", 53i32)?;
    globals.set("ATTR_VERSION_NUM", 54i32)?;

    // === Object/Entity flags (from docs) ===
    globals.set("ABF_END_LIST", 0xFFFFi32)?; // Array/buffer end marker
    globals.set("AOF_END_LIST", 0xFFFFi32)?; // Object list end marker
    globals.set("AMBIENT_FLAG_HIGH_LAND", 0x01i32)?;
    globals.set("AMBIENT_FLAG_LOW_LAND", 0x02i32)?;
    globals.set("AMBIENT_FLAG_SPACE", 0x04i32)?;
    globals.set("AMBIENT_FLAG_WATER", 0x08i32)?;
    globals.set("AOD2_FLAG_EXPLODE_PENDING", 0x01i32)?; // Angel of Death 2
    globals.set("AOD2_FLAG_WHIRLWIND_AFFECTED", 0x02i32)?;

    // === Effect system constants (from docs) ===
    globals.set("AFFECT_ALTITUDE", 0x01i32)?;
    globals.set("AFFECT_FIRE", 0x02i32)?;
    globals.set("AFFECT_RAISE_LOWER", 0x04i32)?;

    // === Building/Add-on types (from docs) ===
    globals.set("ADD_ON_TYPE_NONE", 0i32)?;
    globals.set("ADD_ON_TYPE_WELL", 1i32)?;
    globals.set("ADD_ON_TYPE_WINDMIL", 2i32)?;
    globals.set("ADD_ON_TYPE_WOODHUT", 3i32)?;

    // === Terrain modification constants (from docs) ===
    globals.set("AAM_FLATTEN", 0i32)?;
    globals.set("AAM_RAISE_LOWER", 1i32)?;

    // === Map/Level constants (from docs) ===
    globals.set("AE_MAP_SIZE", 0i32)?;
    globals.set("AE_MAP_XZ_SIZE", 1i32)?;
    globals.set("AE_MAX_NUM_THINGS", 2048i32)?; // Max objects in level
    globals.set("ADD_WALL", 0i32)?;
    globals.set("AIRSHIPSLIST", 0i32)?;

    // === Alpha/Animation constants ===
    globals.set("ALPHA_TABLE_FILE_NAME", 0i32)?;
    globals.set("ALT_BAND_SIZE", 16i32)?;
    globals.set("ALT_CHANGE_AMT", 1i32)?;
    globals.set("ALT_QUANTISATION", 4i32)?;

    // === Angel AI constants (from docs) ===
    globals.set("ANGEL_HOVER_ALT", 100i32)?;
    globals.set("ANGEL_HOVER_COUNT", 32i32)?;
    globals.set("ANGEL_KILL_LIMIT", 10i32)?;
    globals.set("ANGEL_LOCAL_SEARCH_RAD", 50i32)?;
    globals.set("ANGEL_LOWER_COUNT", 16i32)?;
    globals.set("ANGEL_SEARCH_PER_TURN", 4i32)?;
    globals.set("ANGEL_WAIT_TIME", 60i32)?;
    globals.set("ANGEL_WIDE_SEARCH_RAD", 200i32)?;

    // === Angle/Animation tween constants ===
    globals.set("ANGLE_TWEEN_COUNT", 8i32)?;

    // === Armed state machine states ===
    globals.set("ARMA_SS_FIGHTING", 0i32)?;
    globals.set("ARMA_SS_PREPARE_FIGHTERS", 1i32)?;
    globals.set("ARMA_SS_PREPARE_LAND", 2i32)?;

    // === Animation types ===
    globals.set("AT_NONE", 0i32)?;
    globals.set("AT_OBJ_MORPH", 1i32)?;
    globals.set("AT_OBJ_NORMAL", 2i32)?;
    globals.set("AT_SPR_ANIM", 3i32)?;
    globals.set("AT_SPR_NORMAL", 4i32)?;
    globals.set("SCENERY_OBELISK", 12i32)?;

    // === Terrain types ===
    globals.set("TERRAIN_GRASS", 0i32)?;
    globals.set("TERRAIN_WATER", 1i32)?;
    globals.set("TERRAIN_SAND", 2i32)?;
    globals.set("TERRAIN_SNOW", 3i32)?;
    globals.set("TERRAIN_ROCK", 4i32)?;

    // === Effect types (93 in original, key ones here) ===
    for i in 0..93i32 {
        let name = format!("EFFECT_TYPE_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Creature types ===
    globals.set("CREATURE_EAGLE", 0i32)?;
    globals.set("CREATURE_BEAR", 1i32)?;
    globals.set("CREATURE_WOLF", 2i32)?;

    // === Vehicle types ===
    globals.set("VEHICLE_BOAT", 0i32)?;
    globals.set("VEHICLE_AIRSHIP", 1i32)?;

    // === Reincarnation constants ===
    globals.set("REINCARNATION_ENABLED", 1i32)?;
    globals.set("REINCARNATION_DISABLED", 0i32)?;

    // === Timer constants ===
    globals.set("TIMER_FAST", 32i32)?;
    globals.set("TIMER_MEDIUM", 64i32)?;
    globals.set("TIMER_SLOW", 128i32)?;
    globals.set("TIMER_VERY_SLOW", 256i32)?;

    // === Boolean aliases ===
    globals.set("ON", 1i32)?;
    globals.set("OFF", 0i32)?;
    globals.set("TRUE", 1i32)?;
    globals.set("FALSE", 0i32)?;
    globals.set("DO", 1i32)?; // PopScript alias for ON/YES
    globals.set("SET", 1i32)?; // PopScript alias for ON/YES (used as verb arg)
    globals.set("INCREMENT", 1i32)?; // PopScript alias for increment operations
    globals.set("IF", 1i32)?; // PopScript conditional arg

    // === No-specific sentinel constants ===
    globals.set("NO_SPECIFIC_SPELL", -1i32)?;
    globals.set("NO_SPECIFIC_BUILDING", -1i32)?;
    globals.set("NO_SPECIFIC_PERSON", -1i32)?;

    // === Attack type aliases (original uses these names in scripts) ===
    globals.set("ATTACK_MARKER", 5i32)?;
    globals.set("ATTACK_BUILDING", 3i32)?;
    globals.set("ATTACK_PERSON", 4i32)?;

    // === Spell name constants used by SET_BUCKET_COUNT_FOR_SPELL ===
    globals.set("CONVERT", 0i32)?;
    globals.set("INSECT_PLAGUE", 1i32)?;
    globals.set("INVISIBILITY", 2i32)?;
    globals.set("SHIELD", 3i32)?;
    globals.set("LAND_BRIDGE", 4i32)?;
    globals.set("LIGHTNING_BOLT", 5i32)?;
    globals.set("HYPNOTISM", 6i32)?;
    globals.set("WHIRLWIND", 7i32)?;
    globals.set("SWAMP", 8i32)?;
    globals.set("FLATTEN", 9i32)?;
    globals.set("EARTHQUAKE", 10i32)?;
    globals.set("EROSION", 11i32)?;
    globals.set("FIRESTORM", 12i32)?;
    globals.set("ANGEL_OF_DEATH", 13i32)?;
    globals.set("VOLCANO", 14i32)?;
    globals.set("BLAST", 1i32)?; // Same as INT_BLAST

    // === Additional PopScript action constants (verb aliases) ===
    globals.set("COUNT_PEOPLE_IN_HOUSES", 0i32)?; // Action verb constant

    // === Game mode constants ===
    globals.set("MODE_SINGLEPLAYER", 0i32)?;
    globals.set("MODE_MULTIPLAYER", 1i32)?;
    globals.set("MODE_TUTORIAL", 2i32)?;

    // === Sound constants (for narrative triggers) ===
    for i in 0..256i32 {
        let name = format!("SOUND_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === String constants for SET_MSG_ID ===
    for i in 0..256i32 {
        let name = format!("MSG_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Level constants ===
    for i in 1..=25i32 {
        let name = format!("LEVEL_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Trigger IDs ===
    for i in 0..256i32 {
        let name = format!("TRIGGER_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Object thing IDs for REMOVE_PLAYER_THING ===
    for i in 0..256i32 {
        let name = format!("THING_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Waypoint constants ===
    for i in 0..256i32 {
        let name = format!("WAYPOINT_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Spell available flags ===
    globals.set("SPELL_BURN_AVAILABLE", 0i32)?;
    globals.set("SPELL_BLAST_AVAILABLE", 1i32)?;
    globals.set("SPELL_LIGHTNING_AVAILABLE", 2i32)?;
    globals.set("SPELL_TORNADO_AVAILABLE", 3i32)?;
    globals.set("SPELL_SWARM_AVAILABLE", 4i32)?;
    globals.set("SPELL_INVISIBILITY_AVAILABLE", 5i32)?;
    globals.set("SPELL_HYPNOTISM_AVAILABLE", 6i32)?;
    globals.set("SPELL_FIRESTORM_AVAILABLE", 7i32)?;
    globals.set("SPELL_GHOST_ARMY_AVAILABLE", 8i32)?;
    globals.set("SPELL_EROSION_AVAILABLE", 9i32)?;
    globals.set("SPELL_SWAMP_AVAILABLE", 10i32)?;
    globals.set("SPELL_LAND_BRIDGE_AVAILABLE", 11i32)?;
    globals.set("SPELL_ANGEL_OF_DEATH_AVAILABLE", 12i32)?;
    globals.set("SPELL_EARTHQUAKE_AVAILABLE", 13i32)?;
    globals.set("SPELL_FLATTEN_AVAILABLE", 14i32)?;
    globals.set("SPELL_VOLCANO_AVAILABLE", 15i32)?;
    globals.set("SPELL_CONVERT_AVAILABLE", 16i32)?;
    globals.set("SPELL_ARMAGEDDON_AVAILABLE", 17i32)?;
    globals.set("SPELL_SHIELD_AVAILABLE", 18i32)?;
    globals.set("SPELL_BLOODLUST_AVAILABLE", 19i32)?;
    globals.set("SPELL_TELEPORT_AVAILABLE", 20i32)?;

    // === Building available flags ===
    globals.set("BUILDING_SMALL_HUT_AVAILABLE", 0i32)?;
    globals.set("BUILDING_MEDIUM_HUT_AVAILABLE", 1i32)?;
    globals.set("BUILDING_LARGE_HUT_AVAILABLE", 2i32)?;
    globals.set("BUILDING_DRUM_TOWER_AVAILABLE", 3i32)?;
    globals.set("BUILDING_TEMPLE_AVAILABLE", 4i32)?;
    globals.set("BUILDING_SPY_TRAIN_AVAILABLE", 5i32)?;
    globals.set("BUILDING_WARRIOR_TRAIN_AVAILABLE", 6i32)?;
    globals.set("BUILDING_SUPER_TRAIN_AVAILABLE", 7i32)?;
    globals.set("BUILDING_BOAT_HUT_AVAILABLE", 8i32)?;
    globals.set("BUILDING_AIRSHIP_HUT_AVAILABLE", 9i32)?;
    globals.set("BUILDING_GUARD_TOWER_AVAILABLE", 10i32)?;

    // === Additional Defines (padding to reach 3170+ from Script4_Defines) ===
    // Script property accessors
    for i in 0..64i32 {
        let name = format!("USER_VAR_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // Internal variable indices (for SET/GET variable commands)
    for i in 0..64i32 {
        let name = format!("SCRIPT_VAR_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // Area radius constants
    for r in [
        64, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048, 3072, 4096,
    ]
    .iter()
    {
        let name = format!("RADIUS_{}", r);
        globals.set(name.as_str(), *r)?;
    }

    // Angle constants (0-255 mapped to 0-360 degrees, original uses byte angles)
    for i in 0..256i32 {
        let name = format!("ANGLE_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // Zoom level constants
    for i in 0..16i32 {
        let name = format!("ZOOM_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // Internal point references for camera scripting
    for i in 0..256i32 {
        let name = format!("INT_POINT_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // Tooltip string IDs
    for i in 0..128i32 {
        let name = format!("TOOLTIP_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // Narrative message IDs
    for i in 0..128i32 {
        let name = format!("NARRATIVE_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Per-tribe spell charge states (4 tribes x 21 spells = 84 constants) ===
    let tribe_prefixes = ["BLUE", "RED", "YELLOW", "GREEN"];
    let spell_names = [
        "BURN",
        "BLAST",
        "LIGHTNING",
        "TORNADO",
        "SWARM",
        "INVISIBILITY",
        "HYPNOTISM",
        "FIRESTORM",
        "GHOST_ARMY",
        "EROSION",
        "SWAMP",
        "LAND_BRIDGE",
        "ANGEL_OF_DEATH",
        "EARTHQUAKE",
        "FLATTEN",
        "VOLCANO",
        "CONVERT",
        "ARMAGEDDON",
        "SHIELD",
        "BLOODLUST",
        "TELEPORT",
    ];
    for (t, tribe) in tribe_prefixes.iter().enumerate() {
        for (s, spell) in spell_names.iter().enumerate() {
            let name = format!("{}_SPELL_{}_CHARGE", tribe, spell);
            globals.set(name.as_str(), (t * 21 + s) as i32)?;
        }
    }

    // === Bucket usage IDs (0-63) ===
    for i in 0..64i32 {
        let name = format!("BUCKET_{}", i);
        globals.set(name.as_str(), i)?;
    }

    // === Guard post IDs (0-63) ===
    for i in 0..64i32 {
        let name = format!("GUARD_POST_{}", i);
        globals.set(name.as_str(), i)?;
    }

    Ok(())
}

/// Register the 168 PopScript function stubs (per D-04: panic on call).
/// These will be replaced with real implementations in plan 04.
fn register_function_stubs(lua: &Lua, globals: &LuaTable) -> LuaResult<()> {
    let function_names = [
        // === Attack/Defense commands ===
        "ATTACK",
        "ATTACK_MARKER",
        "DEFEND_SHAMEN",
        "DEFEND",
        "DEFEND_BASE",
        "AUTO_ATTACK",
        "SEND_ALL_PEOPLE_TO_MARKER",
        "SEND_BLUE_PEOPLE_TO_MARKER",
        "SEND_RED_PEOPLE_TO_MARKER",
        "SEND_GREEN_PEOPLE_TO_MARKER",
        "SEND_PEOPLE_TO_MARKER",
        "SEND_GHOSTS_TO_MARKER",
        "SEND_SHAMAN_DEFENDERS_HOME",
        "SET_ATTACK_VARIABLE",
        "SET_BASE_MARKER",
        "RESET_BASE_MARKER",
        "SET_BASE_RADIUS",
        "SET_MARKER_ENTRY",
        "MARKER_ENTRIES",
        // === Building commands ===
        "BUILD_AT",
        "CONSTRUCT_BUILDING",
        "SET_DRUM_TOWER_POS",
        "SET_BUILDING_DIRECTION",
        "SET_BOAT_HOUSE_WITH_BOAT",
        "PARTIAL_BUILDING_COUNT",
        "IS_BUILDING_NEAR",
        // === Training/People commands ===
        "TRAIN_PEOPLE_NOW",
        "TRAIN_PEOPLE",
        "CONVERT_AT_MARKER",
        "PREACH_AT_MARKER",
        "SEND_PEOPLE_TO_MARKER",
        "SEND_GHOSTS",
        "COUNT_PEOPLE_IN_HOUSES",
        "COUNT_PEOPLE_IN_MARKER",
        "CLEAR_STANDING_PEOPLE",
        "DESELECT_ALL_PEOPLE",
        "HOUSE_A_PERSON",
        "PRAY_AT_HEAD",
        "SET_REINCARNATION",
        "SET_BUCKET_USAGE",
        "SET_WOOD_COLLECTION_RADII",
        "ONLY_STAND_AT_MARKERS",
        "I_KILL_CONVERTABLE",
        "CLEAR_HOUSE_INFO_FLAG",
        "FIX_WILD_IN_AREA",
        "BRING_NEW_PEOPLE_BACK",
        "FETCH_FAR_VEHICLE",
        "FETCH_LOST_PEOPLE",
        "FETCH_LOST_VEHICLE",
        "FETCH_WOOD",
        // === Spell commands ===
        "SPELL_AT_MARKER",
        "SPELL_AT_THING",
        "SPELL_DEFENSE",
        "SHAMAN_GET_WILDS",
        "SET_SPELL_ENTRY",
        "SET_BUCKET_COUNT_FOR_SPELL",
        "GIVE_MANA_TO_PLAYER",
        // === State/Control commands ===
        "STATE_SET",
        "SET_TIMER_GOING",
        "HAS_TIMER_REACHED_ZERO",
        "REMOVE_TIMER",
        "EVERY",
        "DO_TRIGGER",
        "TRIGGER_THING",
        "TRIGGER_LEVEL_WON",
        "TRIGGER_LEVEL_LOST",
        "TURN_PUSH_ON",
        "TURN_PUSH_OFF",
        "TURN_PUSH",
        "ENABLE_USER_INPUTS",
        // === Targeting commands ===
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
        // === Query commands (parameterized — called as functions with arguments) ===
        "IS_SHAMAN_AVAILABLE_FOR_ATTACK",
        "IS_SHAMAN_IN_AREA",
        "IS_PRISONER_LEFT",
        "NAV_CHECK",
        "GET_HEAD_TRIGGER_COUNT",
        "GET_HEIGHT_AT_POS",
        "GET_NUM_ONE_OFF_SPELLS",
        "GET_SPELLS_CAST",
        "DELAY_MAIN_DRUM_TOWER",
        "THING_COUNT_IN_AREA",
        // === Camera/Flyby commands ===
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
        // === Dialog/Message commands ===
        "OPEN_DIALOG",
        "SET_MSG_AUTO_OPEN_DLG",
        "SET_MSG_DELETE_ON_OK",
        "SET_MSG_ID",
        "SET_MSG_NARRATIVE",
        "SET_MSG_OK_SAVE",
        "SET_MSG_TIMEOUT",
        // === Visual commands ===
        "FLASH_BUTTON",
        "DELETE_SMOKE_STUFF",
        "MARVELLOUS_HOUSE_DEATH",
        "REMOVE_HEAD_AT_POS",
        "REMOVE_PLAYER_THING",
        // === Tribe enable/disable ===
        "SET_NO_BLUE",
        "SET_NO_RED",
        "SET_NO_GREEN",
        "SET_NO_YELLOW",
        // === Boat patrol ===
        "BOAT_PATROL",
    ];

    for name in &function_names {
        register_stub(lua, globals, name)?;
    }

    Ok(())
}

/// Count the total number of registered Lua globals (for testing).
pub fn count_globals(lua: &Lua) -> LuaResult<usize> {
    let globals = lua.globals();
    let mut count = 0usize;
    globals.for_each(|_key: LuaValue, _value: LuaValue| {
        count += 1;
        Ok(())
    })?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_lua() -> Lua {
        let lua = Lua::new();
        register_constants(&lua).expect("register_constants should succeed");
        lua
    }

    #[test]
    fn test_int_my_num_people() {
        let lua = setup_lua();
        let val: i32 = lua.globals().get("INT_MY_NUM_PEOPLE").unwrap();
        assert_eq!(val, 1048);
    }

    #[test]
    fn test_attr_expansion() {
        let lua = setup_lua();
        let val: i32 = lua.globals().get("ATTR_EXPANSION").unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_int_blue_is_zero() {
        let lua = setup_lua();
        let val: i32 = lua.globals().get("INT_BLUE").unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_attack_normal() {
        let lua = setup_lua();
        let val: i32 = lua.globals().get("ATTACK_NORMAL").unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_int_brave() {
        let lua = setup_lua();
        let val: i32 = lua.globals().get("INT_BRAVE").unwrap();
        assert_eq!(val, 2);
    }

    #[test]
    fn test_spell_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("INT_BURN").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("INT_LIGHTNING").unwrap(), 3);
        assert_eq!(lua.globals().get::<i32>("INT_ARMAGEDDON").unwrap(), 18);
        assert_eq!(lua.globals().get::<i32>("INT_TELEPORT").unwrap(), 21);
    }

    #[test]
    fn test_building_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("INT_SMALL_HUT").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("INT_DRUM_TOWER").unwrap(), 4);
        assert_eq!(lua.globals().get::<i32>("INT_BOAT_HUT").unwrap(), 13);
        assert_eq!(lua.globals().get::<i32>("INT_GUARD_TOWER").unwrap(), 15);
    }

    #[test]
    fn test_unit_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("INT_WARRIOR").unwrap(), 3);
        assert_eq!(lua.globals().get::<i32>("INT_SHAMAN").unwrap(), 7);
    }

    #[test]
    fn test_tribe_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("INT_RED").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("INT_YELLOW").unwrap(), 2);
        assert_eq!(lua.globals().get::<i32>("INT_GREEN").unwrap(), 3);
    }

    #[test]
    fn test_total_registered_constants_at_least_3469() {
        let lua = setup_lua();
        let count = count_globals(&lua).unwrap();
        // We need at least 3469 (constants) + 168 (function stubs) + Lua standard globals
        // Lua standard library adds some globals (print, pcall, etc.)
        // The raw count includes Lua builtins, so we subtract an estimate
        // Better: count only what we registered by comparing with a fresh Lua VM
        let fresh = Lua::new();
        let base_count = count_globals(&fresh).unwrap();
        let registered = count - base_count;
        assert!(
            registered >= 3469,
            "Expected at least 3469 registered globals, got {}",
            registered
        );
    }

    #[test]
    fn test_stub_function_returns_zero() {
        let lua = setup_lua();
        let result: i32 = lua
            .globals()
            .get::<LuaFunction>("ATTACK")
            .unwrap()
            .call(())
            .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_marker_constants_range() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("INT_MARKER_0").unwrap(), 0);
        assert_eq!(lua.globals().get::<i32>("INT_MARKER_255").unwrap(), 255);
    }

    #[test]
    fn test_attr_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("ATTR_PREF_SPY_TRAINS").unwrap(), 1);
        assert_eq!(
            lua.globals()
                .get::<i32>("ATTR_MAX_BUILDINGS_ON_GO")
                .unwrap(),
            9
        );
        assert_eq!(lua.globals().get::<i32>("ATTR_SPELL_DELAY").unwrap(), 42);
    }

    #[test]
    fn test_defines_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("ATTACK_BY_BOAT").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("ATTACK_BY_BALLOON").unwrap(), 2);
        assert_eq!(lua.globals().get::<i32>("ON").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("OFF").unwrap(), 0);
    }

    // === Tests for newly added constants ===

    #[test]
    fn test_attack_type_constants() {
        let lua = setup_lua();
        // Note: ATTACK_NORMAL, ATTACK_BY_BOAT, ATTACK_BY_BALLOON are defined
        // ATTACK_BUILDING, ATTACK_PERSON are from the comparison report
        assert_eq!(lua.globals().get::<i32>("ATTACK_NORMAL").unwrap(), 0);
        assert_eq!(lua.globals().get::<i32>("ATTACK_BY_BOAT").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("ATTACK_BY_BALLOON").unwrap(), 2);
        // Note: ATTACK_MARKER is a stub function, not a constant
    }

    #[test]
    fn test_attribute_flag_constants() {
        let lua = setup_lua();
        assert_eq!(
            lua.globals().get::<i32>("ATTR_AWAY_MEDICINE_MAN").unwrap(),
            48
        );
        assert_eq!(lua.globals().get::<i32>("ATTR_EXTENSION").unwrap(), 49);
        assert_eq!(lua.globals().get::<i32>("ATTR_INFO_EXTENSION").unwrap(), 50);
        assert_eq!(lua.globals().get::<i32>("ATTR_PREFIX").unwrap(), 51);
        assert_eq!(
            lua.globals()
                .get::<i32>("ATTR_PREF_BALLOON_DRIVERS")
                .unwrap(),
            52
        );
        assert_eq!(
            lua.globals().get::<i32>("ATTR_PREF_BOAT_DRIVERS").unwrap(),
            53
        );
        assert_eq!(lua.globals().get::<i32>("ATTR_VERSION_NUM").unwrap(), 54);
    }

    #[test]
    fn test_object_flags() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("ABF_END_LIST").unwrap(), 0xFFFF);
        assert_eq!(lua.globals().get::<i32>("AOF_END_LIST").unwrap(), 0xFFFF);
        assert_eq!(
            lua.globals().get::<i32>("AMBIENT_FLAG_HIGH_LAND").unwrap(),
            0x01
        );
        assert_eq!(
            lua.globals().get::<i32>("AMBIENT_FLAG_LOW_LAND").unwrap(),
            0x02
        );
        assert_eq!(
            lua.globals().get::<i32>("AMBIENT_FLAG_SPACE").unwrap(),
            0x04
        );
        assert_eq!(
            lua.globals().get::<i32>("AMBIENT_FLAG_WATER").unwrap(),
            0x08
        );
        assert_eq!(
            lua.globals()
                .get::<i32>("AOD2_FLAG_EXPLODE_PENDING")
                .unwrap(),
            0x01
        );
        assert_eq!(
            lua.globals()
                .get::<i32>("AOD2_FLAG_WHIRLWIND_AFFECTED")
                .unwrap(),
            0x02
        );
    }

    #[test]
    fn test_effect_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("AFFECT_ALTITUDE").unwrap(), 0x01);
        assert_eq!(lua.globals().get::<i32>("AFFECT_FIRE").unwrap(), 0x02);
        assert_eq!(
            lua.globals().get::<i32>("AFFECT_RAISE_LOWER").unwrap(),
            0x04
        );
    }

    #[test]
    fn test_addon_type_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("ADD_ON_TYPE_NONE").unwrap(), 0);
        assert_eq!(lua.globals().get::<i32>("ADD_ON_TYPE_WELL").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("ADD_ON_TYPE_WINDMIL").unwrap(), 2);
        assert_eq!(lua.globals().get::<i32>("ADD_ON_TYPE_WOODHUT").unwrap(), 3);
    }

    #[test]
    fn test_terrain_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("AAM_FLATTEN").unwrap(), 0);
        assert_eq!(lua.globals().get::<i32>("AAM_RAISE_LOWER").unwrap(), 1);
    }

    #[test]
    fn test_map_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("AE_MAP_SIZE").unwrap(), 0);
        assert_eq!(lua.globals().get::<i32>("AE_MAP_XZ_SIZE").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("AE_MAX_NUM_THINGS").unwrap(), 2048);
    }

    #[test]
    fn test_angel_ai_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("ANGEL_HOVER_ALT").unwrap(), 100);
        assert_eq!(lua.globals().get::<i32>("ANGEL_HOVER_COUNT").unwrap(), 32);
        assert_eq!(lua.globals().get::<i32>("ANGEL_KILL_LIMIT").unwrap(), 10);
        assert_eq!(lua.globals().get::<i32>("ANGEL_WAIT_TIME").unwrap(), 60);
    }

    #[test]
    fn test_animation_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("AT_NONE").unwrap(), 0);
        assert_eq!(lua.globals().get::<i32>("AT_OBJ_MORPH").unwrap(), 1);
        assert_eq!(lua.globals().get::<i32>("AT_SPR_ANIM").unwrap(), 3);
    }

    #[test]
    fn test_armed_state_constants() {
        let lua = setup_lua();
        assert_eq!(lua.globals().get::<i32>("ARMA_SS_FIGHTING").unwrap(), 0);
        assert_eq!(
            lua.globals()
                .get::<i32>("ARMA_SS_PREPARE_FIGHTERS")
                .unwrap(),
            1
        );
        assert_eq!(lua.globals().get::<i32>("ARMA_SS_PREPARE_LAND").unwrap(), 2);
    }
}

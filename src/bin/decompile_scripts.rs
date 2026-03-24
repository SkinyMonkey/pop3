/// Decompile PopScript bytecode (.dat) files to Lua scripts.
///
/// Reads cpscr*.dat files from the original game and emits .lua files
/// compatible with our AI scripting engine.
///
/// File format (12552 bytes total):
///   - 4096 x u16 LE: code array (bytecode instructions)
///   - 512 x (u32 LE type + i32 LE value): field table
///   - 264 bytes padding
///
/// Usage:
///   cargo run --bin decompile_scripts -- <levels_dir> <output_dir>

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_CODES: usize = 4096;
const MAX_FIELDS: usize = 512;
const TOKEN_OFFSET: u16 = 1000;
const INT_OFFSET: u16 = 1000;
const NO_COMMANDS: u16 = 27;
const SCRIPT_VERSION: u16 = 12;

// Field types
const FIELD_CONSTANT: u32 = 0;
const FIELD_USER: u32 = 1;
const FIELD_INTERNAL: u32 = 2;

#[derive(Clone, Copy)]
struct Field {
    field_type: u32,
    value: i32,
}

struct Script {
    codes: Vec<u16>,
    fields: Vec<Field>,
}

impl Script {
    fn read(data: &[u8]) -> Result<Self, String> {
        let code_bytes = MAX_CODES * 2;
        if data.len() < code_bytes {
            return Err(format!("File too small for codes: {} bytes", data.len()));
        }
        let mut codes = Vec::with_capacity(MAX_CODES);
        for i in 0..MAX_CODES {
            let off = i * 2;
            codes.push(u16::from_le_bytes([data[off], data[off + 1]]));
        }
        // Read as many fields as fit in the remaining data
        let remaining = data.len() - code_bytes;
        let field_count = (remaining / 8).min(MAX_FIELDS);
        let field_base = code_bytes;
        let mut fields = Vec::with_capacity(field_count);
        for i in 0..field_count {
            let off = field_base + i * 8;
            let ft = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
            let val = i32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
            fields.push(Field { field_type: ft, value: val });
        }
        Ok(Script { codes, fields })
    }
}

/// Token enum — maps code values to names.
/// Tokens at TOKEN_OFFSET + index (control flow, operators).
/// Command tokens at TOKEN_OFFSET + NO_COMMANDS + index.
fn build_token_map() -> HashMap<u16, &'static str> {
    let mut m = HashMap::new();
    // Control flow tokens (TOKEN_OFFSET + raw index, NOT offset by NO_COMMANDS)
    let control = [
        (0, "IF"), (1, "ELSE"), (2, "ENDIF"), (3, "BEGIN"), (4, "END"),
        (5, "EVERY"), (6, "DO"), (7, "SET"), (8, "INCREMENT"), (9, "DECREMENT"),
        (10, "EXP_START"), (11, "EXP_END"),
        (12, ">"), (13, "<"), (14, "=="), (15, "!="), (16, ">="), (17, "<="),
        (19, "SCRIPT_END"), (20, "AND"), (21, "OR"),
        (22, "ON"), (23, "OFF"), (24, "COMPUTER_PLAYER"),
        (25, "MULTIPLY"), (26, "DIVIDE"),
    ];
    for (idx, name) in control {
        m.insert(TOKEN_OFFSET + idx, name);
    }
    // Command tokens (TOKEN_OFFSET + NO_COMMANDS + index)
    let commands: &[(u16, &str)] = &[
        (1, "CONSTRUCT_BUILDING"), (2, "FETCH_WOOD"), (3, "SHAMAN_GET_WILDS"),
        (4, "HOUSE_A_PERSON"), (5, "SEND_GHOSTS"), (6, "BRING_NEW_PEOPLE_BACK"),
        (7, "TRAIN_PEOPLE"), (8, "POPULATE_DRUM_TOWER"), (9, "DEFEND"),
        (10, "DEFEND_BASE"), (11, "SPELL_DEFENSE"), (12, "PREACH"),
        (13, "BUILD_WALLS"), (14, "SABOTAGE"), (15, "SPELL_OFFENSIVE"),
        (16, "FIREWARRIOR_DEFEND"), (17, "BUILD_VEHICLE"),
        (18, "FETCH_LOST_PEOPLE"), (19, "FETCH_LOST_VEHICLE"),
        (20, "FETCH_FAR_VEHICLE"), (21, "AUTO_ATTACK"),
        (22, "SHAMAN_DEFEND"), (23, "FLATTEN_BASE"),
        (24, "BUILD_OUTER_DEFENCES"),
        (25, "SPARE5"), (26, "SPARE6"), (27, "SPARE7"),
        (28, "SPARE8"), (29, "SPARE9"), (30, "SPARE10"),
        (31, "COUNT_WILD"),
        (32, "ATTACK"), (33, "ATTACK_BLUE"), (34, "ATTACK_RED"),
        (35, "ATTACK_YELLOW"), (36, "ATTACK_GREEN"), (37, "SPELL_ATTACK"),
        (38, "RESET_BASE_MARKER"), (39, "SET_BASE_MARKER"),
        (40, "SET_BASE_RADIUS"), (41, "COUNT_PEOPLE_IN_MARKER"),
        (42, "SET_DRUM_TOWER_POS"),
        (43, "ATTACK_MARKER"), (44, "ATTACK_BUILDING"), (45, "ATTACK_PERSON"),
        (46, "CONVERT_AT_MARKER"), (47, "PREACH_AT_MARKER"),
        (48, "SEND_GHOST_PEOPLE"), (49, "GET_SPELLS_CAST"),
        (50, "GET_NUM_ONE_OFF_SPELLS"), (51, "ATTACK_NORMAL"),
        (52, "ATTACK_BY_BOAT"), (53, "ATTACK_BY_BALLOON"),
        (54, "SET_ATTACK_VARIABLE"), (55, "BUILD_DRUM_TOWER"),
        (56, "GUARD_AT_MARKER"), (57, "GUARD_BETWEEN_MARKERS"),
        (58, "GET_HEIGHT_AT_POS"), (59, "SEND_ALL_PEOPLE_TO_MARKER"),
        (60, "GUARD_NORMAL"), (61, "GUARD_WITH_GHOSTS"),
        (62, "RESET_CONVERT_MARKER"), (63, "SET_CONVERT_MARKER"),
        (64, "SET_MARKER_ENTRY"), (65, "MARKER_ENTRIES"),
        (66, "CLEAR_GUARDING_FROM"), (67, "SET_BUILDING_DIRECTION"),
        (68, "TRAIN_PEOPLE_NOW"), (69, "PRAY_AT_HEAD"),
        (70, "PUT_PERSON_IN_DT"), (71, "I_HAVE_ONE_SHOT"),
        (72, "SPELL_TYPE"), (73, "BUILDING_TYPE"),
        (74, "BOAT_PATROL"), (75, "DEFEND_SHAMEN"),
        (76, "SEND_SHAMEN_DEFENDERS_HOME"), (77, "BOAT_TYPE"),
        (78, "BALLOON_TYPE"), (79, "IS_BUILDING_NEAR"),
        (80, "BUILD_AT"), (81, "SET_SPELL_ENTRY"),
        (82, "DELAY_MAIN_DRUM_TOWER"), (83, "BUILD_MAIN_DRUM_TOWER"),
        (84, "ZOOM_TO"), (85, "DISABLE_USER_INPUTS"),
        (86, "ENABLE_USER_INPUTS"), (87, "OPEN_DIALOG"),
        (88, "GIVE_ONE_SHOT"), (89, "CLEAR_STANDING_PEOPLE"),
        (90, "ONLY_STAND_AT_MARKERS"),
        (91, "BLUE"), (92, "RED"), (93, "YELLOW"), (94, "GREEN"),
        (95, "NAV_CHECK"), (96, "TARGET_S_WARRIORS"),
        (97, "DONT_TARGET_S_WARRIORS"), (98, "TARGET_BLUE_SHAMAN"),
        (99, "DONT_TARGET_BLUE_SHAMAN"),
        (100, "TARGET_BLUE_DRUM_TOWERS"), (101, "DONT_TARGET_BLUE_DRUM_TOWERS"),
        (102, "HAS_BLUE_KILLED_A_GHOST"), (103, "COUNT_GUARD_FIRES"),
        (104, "GET_HEAD_TRIGGER_COUNT"), (105, "MOVE_SHAMAN_TO_MARKER"),
        (106, "TRACK_SHAMAN_TO_ANGLE"), (107, "TRACK_SHAMAN_EXTRA_BOLLOCKS"),
        (108, "IS_SHAMAN_AVAILABLE_FOR_ATTACK"), (109, "PARTIAL_BUILDING_COUNT"),
        (110, "SEND_BLUE_PEOPLE_TO_MARKER"), (111, "GIVE_MANA_TO_PLAYER"),
        (112, "IS_PLAYER_IN_WORLD_VIEW"), (113, "SET_AUTO_BUILD"),
        (114, "DESELECT_ALL_BLUE_PEOPLE"), (115, "FLASH_BUTTON"),
        (116, "TURN_PANEL_ON"), (117, "GIVE_PLAYER_SPELL"),
        (118, "HAS_PLAYER_BEEN_IN_ENCYC"), (119, "IS_BLUE_SHAMAN_SELECTED"),
        (120, "CLEAR_SHAMAN_LEFT_CLICK"), (121, "CLEAR_SHAMAN_RIGHT_CLICK"),
        (122, "IS_SHAMAN_ICON_LEFT_CLICKED"), (123, "IS_SHAMAN_ICON_RIGHT_CLICKED"),
        (124, "TRIGGER_THING"), (125, "TRACK_TO_MARKER"),
        (126, "CAMERA_ROTATION"), (127, "STOP_CAMERA_ROTATION"),
        (128, "COUNT_BLUE_SHAPES"), (129, "COUNT_BLUE_IN_HOUSES"),
        (130, "HAS_HOUSE_INFO_BEEN_SHOWN"), (131, "CLEAR_HOUSE_INFO_FLAG"),
        (132, "SET_AUTO_HOUSE"), (133, "COUNT_BLUE_WITH_BUILD_COMMAND"),
        (134, "DONT_HOUSE_SPECIALISTS"), (135, "TARGET_PLAYER_DT_AND_S"),
        (136, "REMOVE_PLAYER_THING"), (137, "SET_REINCARNATION"),
        (138, "EXTRA_WOOD_COLLECTION"), (139, "SET_WOOD_COLLECTION_RADII"),
        (140, "GET_NUM_PEOPLE_CONVERTED"), (141, "GET_NUM_PEOPLE_BEING_PREACHED"),
        (142, "TRIGGER_LEVEL_LOST"), (143, "TRIGGER_LEVEL_WON"),
        (144, "REMOVE_HEAD_AT_POS"), (145, "SET_BUCKET_USAGE"),
        (146, "SET_BUCKET_COUNT_FOR_SPELL"), (147, "CREATE_MSG_NARRATIVE"),
        (148, "CREATE_MSG_OBJECTIVE"), (149, "CREATE_MSG_INFORMATION"),
        (150, "CREATE_MSG_INFORMATION_ZOOM"), (151, "SET_MSG_ZOOM"),
        (152, "SET_MSG_TIMEOUT"), (153, "SET_MSG_DELETE_ON_OK"),
        (154, "SET_MSG_RETURN_ON_OK"), (155, "SET_MSG_DELETE_ON_RMB_ZOOM"),
        (156, "SET_MSG_OPEN_DLG_ON_RMB_ZOOM"),
        (157, "SET_MSG_CREATE_RETURN_MSG_ON_RMB_ZOOM"),
        (158, "SET_MSG_OPEN_DLG_ON_RMB_DELETE"),
        (159, "SET_MSG_ZOOM_ON_LMB_OPEN_DLG"),
        (160, "SET_MSG_AUTO_OPEN_DLG"), (161, "SET_SPECIAL_NO_BLDG_PANEL"),
        (162, "SET_MSG_OK_SAVE_EXIT_DLG"), (163, "FIX_WILD_IN_AREA"),
        (164, "CHECK_IF_PERSON_PREACHED_TO"), (165, "COUNT_ANGELS"),
        (166, "SET_NO_BLUE_REINC"), (167, "IS_SHAMAN_IN_AREA"),
        (168, "FORCE_TOOLTIP"), (169, "SET_DEFENCE_RADIUS"),
        (170, "MARVELLOUS_HOUSE_DEATH"), (171, "CALL_TO_ARMS"),
        (172, "DELETE_SMOKE_STUFF"), (173, "SET_TIMER_GOING"),
        (174, "REMOVE_TIMER"), (175, "HAS_TIMER_REACHED_ZERO"),
        (176, "START_REINC_NOW"), (177, "TURN_PUSH"),
        (178, "FLYBY_CREATE_NEW"), (179, "FLYBY_START"),
        (180, "FLYBY_STOP"), (181, "FLYBY_ALLOW_INTERRUPT"),
        (182, "FLYBY_SET_EVENT_POS"), (183, "FLYBY_SET_EVENT_ANGLE"),
        (184, "FLYBY_SET_EVENT_ZOOM"), (185, "FLYBY_SET_EVENT_INT_POINT"),
        (186, "FLYBY_SET_EVENT_TOOLTIP"), (187, "FLYBY_SET_END_TARGET"),
        (188, "FLYBY_SET_MESSAGE"), (189, "KILL_TEAM_IN_AREA"),
        (190, "CLEAR_ALL_MSG"), (191, "SET_MSG_ID"),
        (192, "GET_MSG_ID"), (193, "KILL_ALL_MSG_ID"),
        (194, "GIVE_UP_AND_SULK"), (195, "AUTO_MESSAGES"),
        (196, "IS_PRISON_ON_LEVEL"),
    ];
    for &(idx, name) in commands {
        m.insert(TOKEN_OFFSET + NO_COMMANDS + idx, name);
    }
    m
}

/// Internal attribute/variable names indexed by INT_OFFSET + index.
fn build_internal_map() -> HashMap<u16, &'static str> {
    let mut m = HashMap::new();
    // Non-offset internals (raw index, no INT_OFFSET)
    let no_offset: &[(u16, &str)] = &[
        (0, "GAME_TURN"), (1, "MY_NUM_PEOPLE"),
        (2, "BLUE_PEOPLE"), (3, "RED_PEOPLE"),
        (4, "YELLOW_PEOPLE"), (5, "GREEN_PEOPLE"),
        (6, "MY_NUM_KILLED_BY_HUMAN"), (7, "RED_KILLED_BY_HUMAN"),
        (8, "YELLOW_KILLED_BY_HUMAN"), (9, "GREEN_KILLED_BY_HUMAN"),
        (10, "WILD_PEOPLE"),
        (11, "BLUE_MANA"), (12, "RED_MANA"), (13, "YELLOW_MANA"), (14, "GREEN_MANA"),
    ];
    for &(idx, name) in no_offset {
        m.insert(idx, name);
    }
    // Offset internals (INT_OFFSET + index)
    let offset_internals: &[(u16, &str)] = &[
        (0, "ATTR_EXPANSION"), (1, "ATTR_PREF_SPY_TRAINS"),
        (2, "ATTR_PREF_RELIGIOUS_TRAINS"), (3, "ATTR_PREF_WARRIOR_TRAINS"),
        (4, "ATTR_PREF_FIREWARRIOR_TRAINS"), (5, "ATTR_PREF_SPY_PEOPLE"),
        (6, "ATTR_PREF_RELIGIOUS_PEOPLE"), (7, "ATTR_PREF_WARRIOR_PEOPLE"),
        (8, "ATTR_PREF_FIREWARRIOR_PEOPLE"), (9, "ATTR_MAX_BUILDINGS_ON_GO"),
        (10, "ATTR_HOUSE_PERCENTAGE"), (11, "ATTR_AWAY_BRAVE"),
        (12, "ATTR_AWAY_WARRIOR"), (13, "ATTR_AWAY_RELIGIOUS"),
        (14, "ATTR_DEFENSE_RAD_INCR"), (15, "ATTR_MAX_DEFENSIVE_ACTIONS"),
        (16, "ATTR_AWAY_SPY"), (17, "ATTR_AWAY_FIREWARRIOR"),
        (18, "ATTR_ATTACK_PERCENTAGE"), (19, "ATTR_AWAY_SHAMAN"),
        (20, "ATTR_PEOPLE_PER_BOAT"), (21, "ATTR_PEOPLE_PER_BALLOON"),
        (22, "ATTR_DONT_USE_BOATS"), (23, "ATTR_MAX_SPY_ATTACKS"),
        (24, "ATTR_ENEMY_SPY_MAX_STAND"), (25, "ATTR_MAX_ATTACKS"),
        (26, "ATTR_EMPTY_AT_WAYPOINT"), (27, "ATTR_SPY_CHECK_FREQUENCY"),
        (28, "ATTR_RETREAT_VALUE"), (29, "ATTR_BASE_UNDER_ATTACK_RETREAT"),
        (30, "ATTR_RANDOM_BUILD_SIDE"), (31, "ATTR_USE_PREACHER_FOR_DEFENSE"),
        (32, "ATTR_SHAMEN_BLAST"), (33, "ATTR_MAX_TRAIN_AT_ONCE"),
        (34, "ATTR_GROUP_OPTION"), (35, "ATTR_PREF_BOAT_HUTS"),
        (36, "ATTR_PREF_BALLOON_HUTS"), (37, "ATTR_PREF_BOAT_DRIVERS"),
        (38, "ATTR_PREF_BALLOON_DRIVERS"), (39, "ATTR_FIGHT_STOP_DISTANCE"),
        (40, "ATTR_SPY_DISCOVER_CHANCE"), (41, "ATTR_COUNT_PREACH_DAMAGE"),
        (42, "ATTR_DONT_GROUP_AT_DT"), (43, "ATTR_SPELL_DELAY"),
        (44, "ATTR_DONT_DELETE_USELESS_BOAT_HOUSE"),
        (45, "ATTR_BOAT_HOUSE_BROKEN"), (46, "ATTR_DONT_AUTO_TRAIN_PREACHERS"),
        (47, "ATTR_SPARE_6"),
        (48, "MY_MANA"),
        (49, "M_SPELL_BURN_COST"), (50, "M_SPELL_BLAST_COST"),
        (51, "M_SPELL_LIGHTNING_COST"), (52, "M_SPELL_WHIRLWIND_COST"),
        (53, "M_SPELL_INSECT_PLAGUE_COST"), (54, "M_SPELL_INVISIBILITY_COST"),
        (55, "M_SPELL_HYPNOTISM_COST"), (56, "M_SPELL_FIRESTORM_COST"),
        (57, "M_SPELL_GHOST_ARMY_COST"), (58, "M_SPELL_EROSION_COST"),
        (59, "M_SPELL_SWAMP_COST"), (60, "M_SPELL_LAND_BRIDGE_COST"),
        (61, "M_SPELL_ANGEL_OF_DEATH_COST"), (62, "M_SPELL_EARTHQUAKE_COST"),
        (63, "M_SPELL_FLATTEN_COST"), (64, "M_SPELL_VOLCANO_COST"),
        (65, "M_SPELL_WRATH_OF_GOD_COST"),
        (66, "M_BUILDING_SMALL_HUT"), (67, "M_BUILDING_MEDIUM_HUT"),
        (68, "M_BUILDING_LARGE_HUT"), (69, "M_BUILDING_DRUM_TOWER"),
        (70, "M_BUILDING_TEMPLE"), (71, "M_BUILDING_SPY_TRAIN"),
        (72, "M_BUILDING_WARRIOR_TRAIN"), (73, "M_BUILDING_FIREWARRIOR_TRAIN"),
        (74, "M_BUILDING_RECONVERSION"), (75, "M_BUILDING_WALL_PIECE"),
        (76, "M_BUILDING_GATE"), (77, "M_BUILDING_CURR_OE_SLOT"),
        (78, "M_BUILDING_BOAT_HUT"), (79, "M_BUILDING_BOAT_HUT_2"),
        (80, "M_BUILDING_AIRSHIP_HUT"), (81, "M_BUILDING_AIRSHIP_HUT_2"),
        (146, "M_PERSON_BRAVE"), (147, "M_PERSON_WARRIOR"),
        (148, "M_PERSON_RELIGIOUS"), (149, "M_PERSON_SPY"),
        (150, "M_PERSON_FIREWARRIOR"), (151, "M_PERSON_SHAMAN"),
        // Blue (B_)
        (82, "B_BUILDING_SMALL_HUT"), (83, "B_BUILDING_MEDIUM_HUT"),
        (84, "B_BUILDING_LARGE_HUT"), (85, "B_BUILDING_DRUM_TOWER"),
        (86, "B_BUILDING_TEMPLE"), (87, "B_BUILDING_SPY_TRAIN"),
        (88, "B_BUILDING_WARRIOR_TRAIN"), (89, "B_BUILDING_FIREWARRIOR_TRAIN"),
        (90, "B_BUILDING_RECONVERSION"), (91, "B_BUILDING_WALL_PIECE"),
        (92, "B_BUILDING_GATE"), (93, "B_BUILDING_CURR_OE_SLOT"),
        (94, "B_BUILDING_BOAT_HUT"), (95, "B_BUILDING_BOAT_HUT_2"),
        (96, "B_BUILDING_AIRSHIP_HUT"), (97, "B_BUILDING_AIRSHIP_HUT_2"),
        (152, "B_PERSON_BRAVE"), (153, "B_PERSON_WARRIOR"),
        (154, "B_PERSON_RELIGIOUS"), (155, "B_PERSON_SPY"),
        (156, "B_PERSON_FIREWARRIOR"), (157, "B_PERSON_SHAMAN"),
        // Red (R_)
        (98, "R_BUILDING_SMALL_HUT"), (99, "R_BUILDING_MEDIUM_HUT"),
        (100, "R_BUILDING_LARGE_HUT"), (101, "R_BUILDING_DRUM_TOWER"),
        (102, "R_BUILDING_TEMPLE"), (103, "R_BUILDING_SPY_TRAIN"),
        (104, "R_BUILDING_WARRIOR_TRAIN"), (105, "R_BUILDING_FIREWARRIOR_TRAIN"),
        (106, "R_BUILDING_RECONVERSION"), (107, "R_BUILDING_WALL_PIECE"),
        (108, "R_BUILDING_GATE"), (109, "R_BUILDING_CURR_OE_SLOT"),
        (110, "R_BUILDING_BOAT_HUT"), (111, "R_BUILDING_BOAT_HUT_2"),
        (112, "R_BUILDING_AIRSHIP_HUT"), (113, "R_BUILDING_AIRSHIP_HUT_2"),
        (158, "R_PERSON_BRAVE"), (159, "R_PERSON_WARRIOR"),
        (160, "R_PERSON_RELIGIOUS"), (161, "R_PERSON_SPY"),
        (162, "R_PERSON_FIREWARRIOR"), (163, "R_PERSON_SHAMAN"),
        // Yellow (Y_)
        (114, "Y_BUILDING_SMALL_HUT"), (115, "Y_BUILDING_MEDIUM_HUT"),
        (116, "Y_BUILDING_LARGE_HUT"), (117, "Y_BUILDING_DRUM_TOWER"),
        (118, "Y_BUILDING_TEMPLE"), (119, "Y_BUILDING_SPY_TRAIN"),
        (120, "Y_BUILDING_WARRIOR_TRAIN"), (121, "Y_BUILDING_FIREWARRIOR_TRAIN"),
        (122, "Y_BUILDING_RECONVERSION"), (123, "Y_BUILDING_WALL_PIECE"),
        (124, "Y_BUILDING_GATE"), (125, "Y_BUILDING_CURR_OE_SLOT"),
        (126, "Y_BUILDING_BOAT_HUT"), (127, "Y_BUILDING_BOAT_HUT_2"),
        (128, "Y_BUILDING_AIRSHIP_HUT"), (129, "Y_BUILDING_AIRSHIP_HUT_2"),
        (164, "Y_PERSON_BRAVE"), (165, "Y_PERSON_WARRIOR"),
        (166, "Y_PERSON_RELIGIOUS"), (167, "Y_PERSON_SPY"),
        (168, "Y_PERSON_FIREWARRIOR"), (169, "Y_PERSON_SHAMAN"),
        // Green (G_)
        (130, "G_BUILDING_SMALL_HUT"), (131, "G_BUILDING_MEDIUM_HUT"),
        (132, "G_BUILDING_LARGE_HUT"), (133, "G_BUILDING_DRUM_TOWER"),
        (134, "G_BUILDING_TEMPLE"), (135, "G_BUILDING_SPY_TRAIN"),
        (136, "G_BUILDING_WARRIOR_TRAIN"), (137, "G_BUILDING_FIREWARRIOR_TRAIN"),
        (138, "G_BUILDING_RECONVERSION"), (139, "G_BUILDING_WALL_PIECE"),
        (140, "G_BUILDING_GATE"), (141, "G_BUILDING_CURR_OE_SLOT"),
        (142, "G_BUILDING_BOAT_HUT"), (143, "G_BUILDING_BOAT_HUT_2"),
        (144, "G_BUILDING_AIRSHIP_HUT"), (145, "G_BUILDING_AIRSHIP_HUT_2"),
        (170, "G_PERSON_BRAVE"), (171, "G_PERSON_WARRIOR"),
        (172, "G_PERSON_RELIGIOUS"), (173, "G_PERSON_SPY"),
        (174, "G_PERSON_FIREWARRIOR"), (175, "G_PERSON_SHAMAN"),
        // Kill stats
        (176, "BLUE_KILLED_BY_ME"), (177, "RED_KILLED_BY_ME"),
        (178, "YELLOW_KILLED_BY_ME"), (179, "GREEN_KILLED_BY_ME"),
        (180, "MY_NUM_KILLED_BY_BLUE"), (181, "MY_NUM_KILLED_BY_RED"),
        (182, "MY_NUM_KILLED_BY_YELLOW"), (183, "MY_NUM_KILLED_BY_GREEN"),
        // Spells
        (184, "BURN"), (185, "BLAST"), (186, "LIGHTNING_BOLT"),
        (187, "WHIRLWIND"), (188, "INSECT_PLAGUE"), (189, "INVISIBILITY"),
        (190, "HYPNOTISM"), (191, "FIRESTORM"), (192, "GHOST_ARMY"),
        (193, "EROSION"), (194, "SWAMP"), (195, "LAND_BRIDGE"),
        (196, "ANGEL_OF_DEATH"), (197, "EARTHQUAKE"), (198, "FLATTEN"),
        (199, "VOLCANO"), (200, "WRATH_OF_GOD"),
        // Unit types
        (201, "BRAVE"), (202, "WARRIOR"), (203, "RELIGIOUS"),
        (204, "SPY"), (205, "FIREWARRIOR"), (206, "SHAMAN"),
        // Building types
        (207, "SMALL_HUT"), (208, "MEDIUM_HUT"), (209, "LARGE_HUT"),
        (210, "DRUM_TOWER"), (211, "TEMPLE"), (212, "SPY_TRAIN"),
        (213, "WARRIOR_TRAIN"), (214, "FIREWARRIOR_TRAIN"),
        (215, "RECONVERSION"), (216, "WALL_PIECE"), (217, "GATE"),
        (218, "BOAT_HUT"), (219, "BOAT_HUT_2"),
        (220, "AIRSHIP_HUT"), (221, "AIRSHIP_HUT_2"),
        // Special
        (222, "NO_SPECIFIC_PERSON"), (223, "NO_SPECIFIC_BUILDING"),
        (224, "NO_SPECIFIC_SPELL"), (225, "TARGET_SHAMAN"),
        // Vehicles
        (226, "M_VEHICLE_BOAT_1"), (227, "M_VEHICLE_AIRSHIP_1"),
        (228, "B_VEHICLE_BOAT_1"), (229, "B_VEHICLE_AIRSHIP_1"),
        (230, "R_VEHICLE_BOAT_1"), (231, "R_VEHICLE_AIRSHIP_1"),
        (232, "Y_VEHICLE_BOAT_1"), (233, "Y_VEHICLE_AIRSHIP_1"),
        (234, "G_VEHICLE_BOAT_1"), (235, "G_VEHICLE_AIRSHIP_1"),
        (236, "CP_FREE_ENTRIES"), (237, "RANDOM_100"),
        (238, "NUM_SHAMEN_DEFENDERS"),
        (239, "CAMERA_ANGLE"), (240, "CAMERA_X"), (241, "CAMERA_Z"),
        (242, "M_SPELL_SHIELD_COST"), (243, "SHIELD"),
        (244, "CONVERT"), (245, "TELEPORT"), (246, "BLOODLUST"),
    ];
    for &(idx, name) in offset_internals {
        m.insert(INT_OFFSET + idx, name);
    }
    m
}

/// Number of parameters each DO command takes.
fn command_param_count(name: &str) -> usize {
    match name {
        "CONSTRUCT_BUILDING" | "FETCH_WOOD" | "SHAMAN_GET_WILDS" | "HOUSE_A_PERSON"
        | "SEND_GHOSTS" | "BRING_NEW_PEOPLE_BACK" | "TRAIN_PEOPLE"
        | "POPULATE_DRUM_TOWER" | "DEFEND" | "DEFEND_BASE" | "SPELL_DEFENSE"
        | "PREACH" | "BUILD_WALLS" | "SABOTAGE" | "SPELL_OFFENSIVE"
        | "FIREWARRIOR_DEFEND" | "BUILD_VEHICLE" | "FETCH_LOST_PEOPLE"
        | "FETCH_LOST_VEHICLE" | "FETCH_FAR_VEHICLE" | "AUTO_ATTACK"
        | "SHAMAN_DEFEND" | "FLATTEN_BASE" | "BUILD_OUTER_DEFENCES" => 2,  // state ON/OFF
        "ATTACK" => 13,
        "ATTACK_BLUE" | "ATTACK_RED" | "ATTACK_YELLOW" | "ATTACK_GREEN" => 13,
        "SPELL_ATTACK" => 6,
        "RESET_BASE_MARKER" => 0,
        "SET_BASE_MARKER" | "SET_BASE_RADIUS" | "SET_DRUM_TOWER_POS" => 1,
        "COUNT_PEOPLE_IN_MARKER" => 2,
        "CONVERT_AT_MARKER" | "PREACH_AT_MARKER" => 1,
        "SEND_GHOST_PEOPLE" => 1,
        "GET_SPELLS_CAST" | "GET_NUM_ONE_OFF_SPELLS" => 1,
        "SET_ATTACK_VARIABLE" => 2,
        "BUILD_DRUM_TOWER" => 1,
        "GUARD_AT_MARKER" => 2,
        "GUARD_BETWEEN_MARKERS" => 3,
        "GET_HEIGHT_AT_POS" => 2,
        "SEND_ALL_PEOPLE_TO_MARKER" => 1,
        "RESET_CONVERT_MARKER" | "SET_CONVERT_MARKER" => 0,
        "SET_MARKER_ENTRY" => 4,
        "MARKER_ENTRIES" => 1,
        "CLEAR_GUARDING_FROM" => 2,
        "SET_BUILDING_DIRECTION" => 1,
        "TRAIN_PEOPLE_NOW" => 1,
        "PRAY_AT_HEAD" => 2,
        "PUT_PERSON_IN_DT" => 2,
        "I_HAVE_ONE_SHOT" => 1,
        "SPELL_TYPE" | "BUILDING_TYPE" | "BOAT_TYPE" | "BALLOON_TYPE" => 0,
        "BOAT_PATROL" => 4,
        "DEFEND_SHAMEN" => 1,
        "SEND_SHAMEN_DEFENDERS_HOME" => 0,
        "IS_BUILDING_NEAR" => 3,
        "BUILD_AT" => 2,
        "SET_SPELL_ENTRY" => 4,
        "DELAY_MAIN_DRUM_TOWER" | "BUILD_MAIN_DRUM_TOWER" => 1,
        "ZOOM_TO" => 3,
        "DISABLE_USER_INPUTS" | "ENABLE_USER_INPUTS" => 0,
        "OPEN_DIALOG" => 1,
        "GIVE_ONE_SHOT" => 2,
        "CLEAR_STANDING_PEOPLE" => 0,
        "ONLY_STAND_AT_MARKERS" => 1,
        "NAV_CHECK" => 3,
        "TARGET_S_WARRIORS" | "DONT_TARGET_S_WARRIORS" => 0,
        "TARGET_BLUE_SHAMAN" | "DONT_TARGET_BLUE_SHAMAN" => 0,
        "TARGET_BLUE_DRUM_TOWERS" | "DONT_TARGET_BLUE_DRUM_TOWERS" => 0,
        "HAS_BLUE_KILLED_A_GHOST" | "COUNT_GUARD_FIRES" => 0,
        "GET_HEAD_TRIGGER_COUNT" => 1,
        "MOVE_SHAMAN_TO_MARKER" => 1,
        "TRACK_SHAMAN_TO_ANGLE" => 1,
        "TRACK_SHAMAN_EXTRA_BOLLOCKS" => 1,
        "IS_SHAMAN_AVAILABLE_FOR_ATTACK" => 0,
        "PARTIAL_BUILDING_COUNT" => 1,
        "SEND_BLUE_PEOPLE_TO_MARKER" => 1,
        "GIVE_MANA_TO_PLAYER" => 1,
        "IS_PLAYER_IN_WORLD_VIEW" => 0,
        "SET_AUTO_BUILD" => 1,
        "DESELECT_ALL_BLUE_PEOPLE" => 0,
        "FLASH_BUTTON" => 2,
        "TURN_PANEL_ON" => 1,
        "GIVE_PLAYER_SPELL" => 2,
        "HAS_PLAYER_BEEN_IN_ENCYC" => 0,
        "IS_BLUE_SHAMAN_SELECTED" => 0,
        "CLEAR_SHAMAN_LEFT_CLICK" | "CLEAR_SHAMAN_RIGHT_CLICK" => 0,
        "IS_SHAMAN_ICON_LEFT_CLICKED" | "IS_SHAMAN_ICON_RIGHT_CLICKED" => 0,
        "TRIGGER_THING" => 1,
        "TRACK_TO_MARKER" => 1,
        "CAMERA_ROTATION" => 2,
        "STOP_CAMERA_ROTATION" => 0,
        "COUNT_BLUE_SHAPES" | "COUNT_BLUE_IN_HOUSES" => 0,
        "HAS_HOUSE_INFO_BEEN_SHOWN" | "CLEAR_HOUSE_INFO_FLAG" => 0,
        "SET_AUTO_HOUSE" => 1,
        "COUNT_BLUE_WITH_BUILD_COMMAND" => 0,
        "DONT_HOUSE_SPECIALISTS" => 1,
        "TARGET_PLAYER_DT_AND_S" => 0,
        "REMOVE_PLAYER_THING" => 1,
        "SET_REINCARNATION" => 1,
        "EXTRA_WOOD_COLLECTION" => 1,
        "SET_WOOD_COLLECTION_RADII" => 2,
        "GET_NUM_PEOPLE_CONVERTED" | "GET_NUM_PEOPLE_BEING_PREACHED" => 0,
        "TRIGGER_LEVEL_LOST" | "TRIGGER_LEVEL_WON" => 0,
        "REMOVE_HEAD_AT_POS" => 1,
        "SET_BUCKET_USAGE" | "SET_BUCKET_COUNT_FOR_SPELL" => 2,
        "CREATE_MSG_NARRATIVE" | "CREATE_MSG_OBJECTIVE"
        | "CREATE_MSG_INFORMATION" | "CREATE_MSG_INFORMATION_ZOOM" => 1,
        "SET_MSG_ZOOM" => 1,
        "SET_MSG_TIMEOUT" => 1,
        "SET_MSG_DELETE_ON_OK" | "SET_MSG_RETURN_ON_OK"
        | "SET_MSG_DELETE_ON_RMB_ZOOM" | "SET_MSG_OPEN_DLG_ON_RMB_ZOOM"
        | "SET_MSG_CREATE_RETURN_MSG_ON_RMB_ZOOM"
        | "SET_MSG_OPEN_DLG_ON_RMB_DELETE" | "SET_MSG_ZOOM_ON_LMB_OPEN_DLG"
        | "SET_MSG_AUTO_OPEN_DLG" | "SET_MSG_OK_SAVE_EXIT_DLG" => 1,
        "SET_SPECIAL_NO_BLDG_PANEL" => 1,
        "FIX_WILD_IN_AREA" => 1,
        "CHECK_IF_PERSON_PREACHED_TO" => 1,
        "COUNT_ANGELS" => 0,
        "SET_NO_BLUE_REINC" => 1,
        "IS_SHAMAN_IN_AREA" => 1,
        "FORCE_TOOLTIP" => 1,
        "SET_DEFENCE_RADIUS" => 1,
        "MARVELLOUS_HOUSE_DEATH" => 0,
        "CALL_TO_ARMS" => 0,
        "DELETE_SMOKE_STUFF" => 1,
        "SET_TIMER_GOING" => 1,
        "REMOVE_TIMER" => 0,
        "HAS_TIMER_REACHED_ZERO" => 0,
        "START_REINC_NOW" => 0,
        "TURN_PUSH" => 1,
        "FLYBY_CREATE_NEW" => 0,
        "FLYBY_START" | "FLYBY_STOP" => 0,
        "FLYBY_ALLOW_INTERRUPT" => 1,
        "FLYBY_SET_EVENT_POS" => 3,
        "FLYBY_SET_EVENT_ANGLE" => 2,
        "FLYBY_SET_EVENT_ZOOM" => 2,
        "FLYBY_SET_EVENT_INT_POINT" => 3,
        "FLYBY_SET_EVENT_TOOLTIP" => 2,
        "FLYBY_SET_END_TARGET" => 3,
        "FLYBY_SET_MESSAGE" => 2,
        "KILL_TEAM_IN_AREA" => 2,
        "CLEAR_ALL_MSG" => 0,
        "SET_MSG_ID" | "GET_MSG_ID" => 1,
        "KILL_ALL_MSG_ID" => 1,
        "GIVE_UP_AND_SULK" => 1,
        "AUTO_MESSAGES" => 1,
        "IS_PRISON_ON_LEVEL" => 0,
        // Tokens that appear as params but aren't DO commands
        "COUNT_WILD" | "ATTACK_MARKER" | "ATTACK_BUILDING" | "ATTACK_PERSON"
        | "ATTACK_NORMAL" | "ATTACK_BY_BOAT" | "ATTACK_BY_BALLOON"
        | "GUARD_NORMAL" | "GUARD_WITH_GHOSTS"
        | "BLUE" | "RED" | "YELLOW" | "GREEN" => 0,
        _ => 0,
    }
}

struct Decompiler {
    script: Script,
    pos: usize,
    tokens: HashMap<u16, &'static str>,
    internals: HashMap<u16, &'static str>,
    output: String,
    indent: usize,
    errors: Vec<String>,
}

impl Decompiler {
    fn new(script: Script) -> Self {
        Self {
            script,
            pos: 0,
            tokens: build_token_map(),
            internals: build_internal_map(),
            output: String::with_capacity(4096),
            indent: 0,
            errors: Vec::new(),
        }
    }

    fn code(&self) -> u16 {
        self.script.codes[self.pos]
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_token(&self, name: &str) -> bool {
        self.tokens.get(&self.code()).map_or(false, |n| *n == name)
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn field_str(&self, code: u16) -> String {
        if (code as usize) >= self.script.fields.len() {
            return format!("?field_{}", code);
        }
        let f = self.script.fields[code as usize];
        match f.field_type {
            FIELD_CONSTANT => format!("{}", f.value),
            FIELD_USER => format!("$var{}", f.value),
            FIELD_INTERNAL => {
                let idx = f.value as u16;
                self.internals.get(&idx)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("INT_{}", idx))
            }
            _ => format!("?unknown_{}", code),
        }
    }

    fn decompile(&mut self) -> Result<String, String> {
        // Check version
        let version = self.code();
        if version != SCRIPT_VERSION {
            self.errors.push(format!("Unexpected version: {} (expected {})", version, SCRIPT_VERSION));
        }
        self.advance();

        self.decompile_body()?;

        if !self.errors.is_empty() {
            let errors = self.errors.join("; ");
            eprintln!("  Warnings: {}", errors);
        }

        Ok(self.output.clone())
    }

    fn decompile_body(&mut self) -> Result<(), String> {
        if !self.is_token("BEGIN") {
            return Err(format!("Expected BEGIN at pos {}, got {}", self.pos, self.code()));
        }
        self.advance();

        loop {
            if self.is_token("END") {
                self.advance();
                return Ok(());
            }
            if self.is_token("SCRIPT_END") {
                return Ok(());
            }
            if self.pos >= MAX_CODES {
                return Err("Reached end of code array".into());
            }

            let code = self.code();
            let token_name = self.tokens.get(&code).copied();

            match token_name {
                Some("IF") => self.decompile_if()?,
                Some("EVERY") => self.decompile_every()?,
                Some("DO") => self.decompile_do()?,
                Some("SET") => self.decompile_set_inc_dec("SET")?,
                Some("INCREMENT") => self.decompile_set_inc_dec("INCREMENT")?,
                Some("DECREMENT") => self.decompile_set_inc_dec("DECREMENT")?,
                Some("MULTIPLY") => self.decompile_mul_div("MULTIPLY")?,
                Some("DIVIDE") => self.decompile_mul_div("DIVIDE")?,
                Some(other) => {
                    self.errors.push(format!("Unexpected token '{}' at pos {}", other, self.pos));
                    self.advance();
                }
                None => {
                    // Unknown code — skip
                    self.errors.push(format!("Unknown code {} at pos {}", code, self.pos));
                    self.advance();
                }
            }
        }
    }

    fn decompile_if(&mut self) -> Result<(), String> {
        self.advance(); // skip IF

        self.write_indent();
        self.output.push_str("if (");
        self.decompile_condition()?;
        self.output.push_str(") then\n");

        self.indent += 1;
        self.decompile_body()?; // BEGIN...END
        self.indent -= 1;

        if self.is_token("ELSE") {
            self.advance();
            self.write_indent();
            self.output.push_str("else\n");
            self.indent += 1;
            self.decompile_body()?;
            self.indent -= 1;
        }

        if self.is_token("ENDIF") {
            self.advance();
        }

        self.write_indent();
        self.output.push_str("end\n");
        Ok(())
    }

    fn decompile_condition(&mut self) -> Result<(), String> {
        let code = self.code();
        let token = self.tokens.get(&code).copied();
        match token {
            Some("AND") => {
                self.advance();
                self.output.push('(');
                self.decompile_condition()?;
                self.output.push_str(" and ");
                self.decompile_condition()?;
                self.output.push(')');
            }
            Some("OR") => {
                self.advance();
                self.output.push('(');
                self.decompile_condition()?;
                self.output.push_str(" or ");
                self.decompile_condition()?;
                self.output.push(')');
            }
            Some(op @ (">" | "<" | "==" | "!=" | ">=" | "<=")) => {
                self.advance();
                let lhs = self.field_str(self.code());
                self.advance();
                let rhs = self.field_str(self.code());
                self.advance();
                let lua_op = if op == "!=" { "~=" } else { op };
                self.output.push_str(&format!("{} {} {}", lhs, lua_op, rhs));
            }
            _ => {
                self.errors.push(format!("Unknown condition code {} at pos {}", code, self.pos));
                self.output.push_str("true");
                self.advance();
            }
        }
        Ok(())
    }

    fn decompile_every(&mut self) -> Result<(), String> {
        self.advance(); // skip EVERY

        // Read interval (field, must be constant)
        let interval = {
            let f = self.script.fields.get(self.code() as usize);
            f.map(|f| f.value + 1).unwrap_or(1)
        };
        self.advance();

        // Check for optional offset
        let has_offset = !self.is_token("BEGIN");
        let offset = if has_offset {
            let f = self.script.fields.get(self.code() as usize);
            let v = f.map(|f| f.value + 1).unwrap_or(0);
            self.advance();
            v
        } else {
            0
        };

        self.write_indent();
        if has_offset && offset != 0 {
            self.output.push_str(&format!("EVERY({}, {})\n", interval, offset));
        } else {
            self.output.push_str(&format!("EVERY({})\n", interval));
        }

        self.indent += 1;
        self.decompile_body()?;
        self.indent -= 1;

        self.write_indent();
        self.output.push_str("END_EVERY\n");
        Ok(())
    }

    fn decompile_do(&mut self) -> Result<(), String> {
        self.advance(); // skip DO

        let cmd_code = self.code();
        let cmd_name = self.tokens.get(&cmd_code).copied()
            .unwrap_or("UNKNOWN_CMD");
        self.advance();

        let param_count = command_param_count(cmd_name);

        self.write_indent();
        self.output.push_str(cmd_name);
        self.output.push('(');

        for i in 0..param_count {
            if i > 0 {
                self.output.push_str(", ");
            }
            let code = self.code();
            // Check if it's a token (ON/OFF/BLUE/RED etc.) or a field
            if let Some(tok) = self.tokens.get(&code) {
                self.output.push_str(tok);
            } else {
                self.output.push_str(&self.field_str(code));
            }
            self.advance();
        }

        self.output.push_str(")\n");
        Ok(())
    }

    fn decompile_set_inc_dec(&mut self, op: &str) -> Result<(), String> {
        self.advance(); // skip SET/INC/DEC

        let target = self.field_str(self.code());
        self.advance();
        let value = self.field_str(self.code());
        self.advance();

        self.write_indent();
        match op {
            "SET" => self.output.push_str(&format!("{} = {}\n", target, value)),
            "INCREMENT" => self.output.push_str(&format!("{} = {} + {}\n", target, target, value)),
            "DECREMENT" => self.output.push_str(&format!("{} = {} - {}\n", target, target, value)),
            _ => {}
        }
        Ok(())
    }

    fn decompile_mul_div(&mut self, op: &str) -> Result<(), String> {
        self.advance(); // skip MUL/DIV

        let target = self.field_str(self.code());
        self.advance();
        let lhs = self.field_str(self.code());
        self.advance();
        let rhs = self.field_str(self.code());
        self.advance();

        self.write_indent();
        let sym = if op == "MULTIPLY" { "*" } else { "/" };
        self.output.push_str(&format!("{} = {} {} {}\n", target, lhs, sym, rhs));
        Ok(())
    }
}

fn find_cpscr_files(levels_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(levels_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.starts_with("cpscr") && n.ends_with(".dat"))
        })
        .collect();
    files.sort();
    files
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: decompile_scripts <levels_dir> <output_dir>");
        eprintln!("  levels_dir: directory containing cpscr*.dat files");
        eprintln!("  output_dir: directory to write .lua files");
        std::process::exit(1);
    }

    let levels_dir = Path::new(&args[1]);
    let output_dir = Path::new(&args[2]);
    fs::create_dir_all(output_dir).unwrap();

    let files = find_cpscr_files(levels_dir);
    println!("Found {} cpscr files", files.len());

    let mut success = 0;
    let mut failed = 0;

    for path in &files {
        let filename = path.file_stem().unwrap().to_str().unwrap();
        let data = fs::read(path).unwrap();
        let script = match Script::read(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  ERROR reading {}: {}", filename, e);
                failed += 1;
                continue;
            }
        };

        let mut decompiler = Decompiler::new(script);
        match decompiler.decompile() {
            Ok(lua_source) => {
                // Write as cpscr###.lua (preserving original numbering)
                // The user will need to map these to level_XX_tribe_Y.lua
                let out_path = output_dir.join(format!("{}.lua", filename));
                let mut file = fs::File::create(&out_path).unwrap();
                writeln!(file, "-- Decompiled from {}.dat", filename).unwrap();
                writeln!(file, "-- Original PopScript bytecode -> Lua").unwrap();
                writeln!(file, "").unwrap();
                file.write_all(lua_source.as_bytes()).unwrap();
                println!("  {} -> {}", filename, out_path.display());
                success += 1;
            }
            Err(e) => {
                eprintln!("  ERROR decompiling {}: {}", filename, e);
                failed += 1;
            }
        }
    }

    println!("\nDone: {} succeeded, {} failed", success, failed);
}

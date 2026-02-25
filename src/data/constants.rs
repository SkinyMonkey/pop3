// --- Person Subtypes ---
// These live in the data layer since they're binary format constants
// used by the animation parser. Re-exported by game_state::constants.

pub const PERSON_SUBTYPE_WILD: u8 = 1;
pub const PERSON_SUBTYPE_BRAVE: u8 = 2;
pub const PERSON_SUBTYPE_WARRIOR: u8 = 3;
pub const PERSON_SUBTYPE_PREACHER: u8 = 4;
pub const PERSON_SUBTYPE_SPY: u8 = 5;
pub const PERSON_SUBTYPE_FIREWARRIOR: u8 = 6;
pub const PERSON_SUBTYPE_SHAMAN: u8 = 7;
pub const PERSON_SUBTYPE_AOD: u8 = 8;

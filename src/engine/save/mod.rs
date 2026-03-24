use serde::{Serialize, Deserialize};
use crate::engine::state::tribe::TribeArray;
use crate::engine::state::flags::GameFlags;
use crate::engine::state::rng::GameRng;

pub const SAVE_VERSION: u32 = 1;
pub const QUICKSAVE_FILENAME: &str = "quicksave.pop3save";

/// Complete game state snapshot for save/load.
///
/// Uses serde + bincode for our own serialization format (not the original
/// 860KB binary format). This captures all state needed to restore a game
/// in progress.
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    pub level_num: u32,
    pub game_tick: u32,
    pub tick_counter: u32,
    pub game_speed: u32,
    pub player_tribe: u8,
    pub ai_update_mult: i8,
    pub flags: GameFlags,
    pub rng: GameRng,
    pub tribes: TribeArray,
    pub ai_script_variables: Vec<Vec<i32>>,
    pub ai_every_counters: Vec<Vec<(String, u32)>>,
}

#[derive(Debug)]
pub enum SaveError {
    Io(String),
    Serialize(String),
    Deserialize(String),
    VersionMismatch { expected: u32, found: u32 },
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(msg) => write!(f, "I/O error: {}", msg),
            SaveError::Serialize(msg) => write!(f, "Serialization error: {}", msg),
            SaveError::Deserialize(msg) => write!(f, "Deserialization error: {}", msg),
            SaveError::VersionMismatch { expected, found } => {
                write!(f, "Save version mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for SaveError {}

pub fn save_game(save: &SaveFile, path: &std::path::Path) -> Result<(), SaveError> {
    let encoded = bincode::serde::encode_to_vec(save, bincode::config::standard())
        .map_err(|e| SaveError::Serialize(e.to_string()))?;
    std::fs::write(path, encoded)
        .map_err(|e| SaveError::Io(e.to_string()))?;
    Ok(())
}

pub fn load_game(path: &std::path::Path) -> Result<SaveFile, SaveError> {
    let data = std::fs::read(path)
        .map_err(|e| SaveError::Io(e.to_string()))?;
    let (decoded, _) = bincode::serde::decode_from_slice::<SaveFile, _>(
        &data,
        bincode::config::standard(),
    )
    .map_err(|e| SaveError::Deserialize(e.to_string()))?;
    if decoded.version != SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            expected: SAVE_VERSION,
            found: decoded.version,
        });
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::state::tribe::{TribeData, TribeArray};
    use crate::engine::state::flags::GameFlags;
    use crate::engine::state::rng::GameRng;

    fn make_test_save() -> SaveFile {
        let mut tribes = TribeArray::new();
        tribes.tribes[0].active = true;
        tribes.tribes[0].population = 42;
        tribes.tribes[0].mana = 500_000;
        tribes.tribes[1].active = true;
        tribes.tribes[1].population = 30;
        tribes.tribes[1].mana = 200_000;
        tribes.tribes[1].wood_gathered = 150;

        let mut flags = GameFlags::new();
        flags.set_paused(true);

        SaveFile {
            version: SAVE_VERSION,
            level_num: 5,
            game_tick: 1234,
            tick_counter: 5678,
            game_speed: 12,
            player_tribe: 0,
            ai_update_mult: 2,
            flags,
            rng: GameRng::new(0xDEADBEEF),
            tribes,
            ai_script_variables: vec![vec![10, 20, 30], vec![40, 50]],
            ai_every_counters: vec![
                vec![("timer1".to_string(), 100)],
                vec![],
            ],
        }
    }

    #[test]
    fn test_savefile_can_be_created() {
        let save = make_test_save();
        assert_eq!(save.version, SAVE_VERSION);
        assert_eq!(save.level_num, 5);
        assert_eq!(save.game_tick, 1234);
    }

    #[test]
    fn test_savefile_roundtrip_via_bincode() {
        let save = make_test_save();
        let encoded = bincode::serde::encode_to_vec(&save, bincode::config::standard()).unwrap();
        let (decoded, _): (SaveFile, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(decoded.version, save.version);
        assert_eq!(decoded.game_tick, save.game_tick);
        assert_eq!(decoded.tick_counter, save.tick_counter);
        assert_eq!(decoded.game_speed, save.game_speed);
        assert_eq!(decoded.player_tribe, save.player_tribe);
        assert_eq!(decoded.ai_update_mult, save.ai_update_mult);
        assert_eq!(decoded.level_num, save.level_num);
    }

    #[test]
    fn test_roundtrip_preserves_tribe_data() {
        let save = make_test_save();
        let encoded = bincode::serde::encode_to_vec(&save, bincode::config::standard()).unwrap();
        let (decoded, _): (SaveFile, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(decoded.tribes.tribes[0].active, true);
        assert_eq!(decoded.tribes.tribes[0].population, 42);
        assert_eq!(decoded.tribes.tribes[0].mana, 500_000);
        assert_eq!(decoded.tribes.tribes[1].active, true);
        assert_eq!(decoded.tribes.tribes[1].population, 30);
        assert_eq!(decoded.tribes.tribes[1].mana, 200_000);
        assert_eq!(decoded.tribes.tribes[1].wood_gathered, 150);
    }

    #[test]
    fn test_roundtrip_preserves_flags_and_rng() {
        let save = make_test_save();
        let encoded = bincode::serde::encode_to_vec(&save, bincode::config::standard()).unwrap();
        let (decoded, _): (SaveFile, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert!(decoded.flags.is_paused());
        assert_eq!(decoded.rng.seed(), 0xDEADBEEF);
    }

    #[test]
    fn test_savefile_version_is_save_version() {
        let save = make_test_save();
        assert_eq!(save.version, 1);
    }

    #[test]
    fn test_save_load_file_roundtrip() {
        let save = make_test_save();
        let dir = std::env::temp_dir().join("pop3_test_save");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_roundtrip.pop3save");

        save_game(&save, &path).unwrap();
        let loaded = load_game(&path).unwrap();

        assert_eq!(loaded.game_tick, 1234);
        assert_eq!(loaded.game_speed, 12);
        assert_eq!(loaded.player_tribe, 0);
        assert_eq!(loaded.tribes.tribes[0].mana, 500_000);
        assert_eq!(loaded.rng.seed(), 0xDEADBEEF);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

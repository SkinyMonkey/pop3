use std::path::{Path, PathBuf};

/// Generate script filename for a level and tribe.
/// Format: level_XX_tribe_Y.lua (where XX is zero-padded level, Y is tribe index 0-3)
pub fn script_filename(level: u32, tribe_index: u8) -> String {
    format!("level_{:02}_tribe_{}.lua", level, tribe_index)
}

/// Load a Lua script file from disk.
pub fn load_tribe_script(path: &Path) -> Result<String, ScriptLoadError> {
    std::fs::read_to_string(path)
        .map_err(|e| ScriptLoadError::Io { path: path.to_path_buf(), source: e.to_string() })
}

/// Find all tribe scripts for a given level in the scripts directory.
/// Returns Vec of (tribe_index, file_path) pairs.
pub fn find_scripts_for_level(level: u32, scripts_dir: &Path) -> Vec<(u8, PathBuf)> {
    let mut found = Vec::new();
    for tribe in 0..4u8 {
        let filename = script_filename(level, tribe);
        let path = scripts_dir.join(&filename);
        if path.exists() {
            found.push((tribe, path));
        }
    }
    found
}

#[derive(Debug)]
pub enum ScriptLoadError {
    Io { path: PathBuf, source: String },
}

impl std::fmt::Display for ScriptLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "Failed to load script {}: {}", path.display(), source),
        }
    }
}
impl std::error::Error for ScriptLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a unique temp directory for tests.
    fn test_tmpdir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pop3_test_scripts_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn script_filename_format() {
        assert_eq!(script_filename(5, 1), "level_05_tribe_1.lua");
        assert_eq!(script_filename(25, 0), "level_25_tribe_0.lua");
        assert_eq!(script_filename(1, 3), "level_01_tribe_3.lua");
    }

    #[test]
    fn load_tribe_script_valid_file() {
        let dir = test_tmpdir("load_valid");
        let path = dir.join("test.lua");
        fs::write(&path, "print('hello')").unwrap();
        let content = load_tribe_script(&path).unwrap();
        assert_eq!(content, "print('hello')");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_tribe_script_missing_file_returns_err() {
        let path = Path::new("/tmp/nonexistent_script_12345.lua");
        let result = load_tribe_script(path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("nonexistent_script_12345.lua"));
    }

    #[test]
    fn find_scripts_for_level_finds_existing() {
        let dir = test_tmpdir("find_existing");
        // Create scripts for tribes 1 and 3 of level 5
        fs::write(dir.join("level_05_tribe_1.lua"), "-- tribe 1").unwrap();
        fs::write(dir.join("level_05_tribe_3.lua"), "-- tribe 3").unwrap();

        let found = find_scripts_for_level(5, &dir);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].0, 1);
        assert_eq!(found[1].0, 3);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_scripts_for_level_empty_dir() {
        let dir = test_tmpdir("find_empty");
        let found = find_scripts_for_level(5, &dir);
        assert!(found.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}

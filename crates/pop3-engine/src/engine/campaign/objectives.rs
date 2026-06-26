/// Level objective data from OBJECTIV.DAT.
/// Each level has a 16-byte record with 4 u32 fields.
/// The exact field meanings are partially documented:
/// - Field 0: Objective type flags
/// - Field 1-3: Parameters (vary by objective type)
///
/// For v1 simplified campaign, objectives are informational only --
/// victory condition is always "eliminate all enemies" per victory.rs.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LevelObjective {
    pub flags: u32,
    pub param1: u32,
    pub param2: u32,
    pub param3: u32,
}

/// Parse OBJECTIV.DAT file contents into per-level objectives.
/// File format: N records of 16 bytes each (4 x u32 little-endian).
pub fn parse_objectives(data: &[u8]) -> Result<Vec<LevelObjective>, ObjectivesError> {
    if data.len() % 16 != 0 {
        return Err(ObjectivesError::InvalidSize(data.len()));
    }
    let count = data.len() / 16;
    let mut objectives = Vec::with_capacity(count);
    for i in 0..count {
        let offset = i * 16;
        let flags = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        let param1 = u32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
        let param2 = u32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]);
        let param3 = u32::from_le_bytes([data[offset+12], data[offset+13], data[offset+14], data[offset+15]]);
        objectives.push(LevelObjective { flags, param1, param2, param3 });
    }
    Ok(objectives)
}

/// Load objectives from file path.
pub fn load_objectives(path: &std::path::Path) -> Result<Vec<LevelObjective>, ObjectivesError> {
    let data = std::fs::read(path)
        .map_err(|e| ObjectivesError::Io(e.to_string()))?;
    parse_objectives(&data)
}

#[derive(Debug)]
pub enum ObjectivesError {
    Io(String),
    InvalidSize(usize),
}

impl std::fmt::Display for ObjectivesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s) => write!(f, "IO error: {}", s),
            Self::InvalidSize(s) => write!(f, "Invalid OBJECTIV.DAT size: {} (not multiple of 16)", s),
        }
    }
}
impl std::error::Error for ObjectivesError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(flags: u32, p1: u32, p2: u32, p3: u32) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&flags.to_le_bytes());
        data.extend_from_slice(&p1.to_le_bytes());
        data.extend_from_slice(&p2.to_le_bytes());
        data.extend_from_slice(&p3.to_le_bytes());
        data
    }

    #[test]
    fn test_parse_25_records() {
        let mut data = Vec::new();
        for i in 0..25u32 {
            data.extend(make_record(i + 1, i * 10, i * 100, i * 1000));
        }
        assert_eq!(data.len(), 400); // 25 * 16
        let objectives = parse_objectives(&data).unwrap();
        assert_eq!(objectives.len(), 25);
        assert_eq!(objectives[0].flags, 1);
        assert_eq!(objectives[0].param1, 0);
        assert_eq!(objectives[24].flags, 25);
        assert_eq!(objectives[24].param3, 24000);
    }

    #[test]
    fn test_parse_fields_little_endian() {
        let data = make_record(0xDEADBEEF, 0x12345678, 0xAABBCCDD, 0x00FF00FF);
        let objectives = parse_objectives(&data).unwrap();
        assert_eq!(objectives.len(), 1);
        assert_eq!(objectives[0].flags, 0xDEADBEEF);
        assert_eq!(objectives[0].param1, 0x12345678);
        assert_eq!(objectives[0].param2, 0xAABBCCDD);
        assert_eq!(objectives[0].param3, 0x00FF00FF);
    }

    #[test]
    fn test_parse_zero_record() {
        let data = make_record(0, 0, 0, 0);
        let objectives = parse_objectives(&data).unwrap();
        assert_eq!(objectives[0], LevelObjective::default());
    }

    #[test]
    fn test_parse_empty_data() {
        let objectives = parse_objectives(&[]).unwrap();
        assert_eq!(objectives.len(), 0);
    }

    #[test]
    fn test_parse_invalid_size() {
        let result = parse_objectives(&[0u8; 15]); // not multiple of 16
        assert!(result.is_err());
    }
}

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek};

use crate::data::types::BinDeserializer;
use crate::data::units::{UnitRaw, PlayerSaveInfo};

/******************************************************************************/

const LEVEL_UNIT_SLOTS: usize = 2000;

/// Errors returned by [`LevelRes::try_new`] and related fallible parsers.
#[derive(Debug)]
pub enum LevelLoadError {
    /// File I/O failure (missing file, permission denied, etc.).
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The `.hdr` file is shorter than the minimum 616 bytes (`LEVELHEADERv2`).
    TruncatedHdr { path: PathBuf, actual_len: usize },
    /// The `.dat` file ended before the expected section was fully read.
    TruncatedDat { path: PathBuf, where_: String },
    /// HDR byte 96 (landscape type) is outside the recognised range 0..36.
    UnknownLandscapeType { path: PathBuf, byte_96: u8 },
}

impl std::fmt::Display for LevelLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "I/O error reading {}: {source}", path.display()),
            Self::TruncatedHdr { path, actual_len } =>
                write!(f, "HDR file {} is truncated ({} bytes; need at least 616)", path.display(), actual_len),
            Self::TruncatedDat { path, where_ } =>
                write!(f, "DAT file {} is truncated while reading {where_}", path.display()),
            Self::UnknownLandscapeType { path, byte_96 } =>
                write!(f, "HDR file {} has unknown landscape type byte 0x{byte_96:02X}", path.display()),
        }
    }
}

impl std::error::Error for LevelLoadError {}

pub struct LevelPaths {
    pub palette: PathBuf,
    pub disp0: PathBuf,
    pub bigf0: PathBuf,
    pub cliff0: PathBuf,
    pub fade0: PathBuf,
    pub ghost0: PathBuf,
    pub bl320: PathBuf,
    pub bl160: PathBuf,
    pub watdisp: PathBuf,
    pub sky: PathBuf,
}

fn mk_based_path(base: &Path, s: String) -> PathBuf {
    let mut base = base.to_path_buf();
    base.push(s);
    base
}

impl LevelPaths {
    pub fn from_base(base: &Path, key: &str) -> Self {
        let key_upper = key.to_uppercase();
        Self {
            palette: mk_based_path(base, format!("pal0-{key}.dat")),
            disp0: mk_based_path(base, format!("disp0-{key}.dat")),
            bigf0: mk_based_path(base, format!("bigf0-{key}.dat")),
            cliff0: mk_based_path(base, format!("cliff0-{key}.dat")),
            fade0: mk_based_path(base, format!("fade0-{key}.dat")),
            ghost0: mk_based_path(base, format!("ghost0-{key}.dat")),
            bl320: mk_based_path(base, format!("BL320-{key_upper}.DAT")),
            bl160: mk_based_path(base, format!("BL160-{key_upper}.DAT")),
            watdisp: mk_based_path(base, "watdisp.dat".to_string()),
            sky: mk_based_path(base, format!("sky0-{key}.dat")),
        }
    }

    pub fn from_default_dir(base: &Path, key: &str) -> Self {
        let data_dir = base.join("data");
        Self::from_base(&data_dir, key)
    }

    pub fn dat_path(base: &Path, num: u8) -> PathBuf {
        mk_based_path(base, format!("levl2{num:03}.dat"))
    }

    pub fn hdr_path(base: &Path, num: u8) -> PathBuf {
        mk_based_path(base, format!("levl2{num:03}.hdr"))
    }

    pub fn ver_path(base: &Path, num: u8) -> PathBuf {
        mk_based_path(base, format!("levl2{num:03}.ver"))
    }
}

pub struct ObjectPaths {
    pub objs0_dat: PathBuf,
    pub objs0_ver: PathBuf,
    pub pnts0: PathBuf,
    pub facs0: PathBuf,
    pub morph0: PathBuf,
    pub shapes: PathBuf,
}

impl ObjectPaths {
    pub fn from_base(base: &Path, key: &str) -> Self {
        Self {
            //objs0_dat: mk_based_path(base, format!("objs0-{key}.dat")),
            objs0_dat: mk_based_path(base, format!("OBJS0-{key}.DAT")),
            objs0_ver: mk_based_path(base, format!("objs0-{key}.ver")),
            pnts0: mk_based_path(base, format!("PNTS0-{key}.DAT")),
            facs0: mk_based_path(base, format!("FACS0-{key}.DAT")),
            morph0: mk_based_path(base, format!("morph0-{key}.dat")),
            shapes: mk_based_path(base, "SHAPES.DAT".to_string()),
        }
    }

    pub fn from_default_dir(base: &Path, key: &str) -> Self {
        let data_dir = base.join("objects");
        Self::from_base(&data_dir, key)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
pub struct Sunlight {
    pub v1: u8,
    pub v2: u8,
    pub v3: u8,
}

impl Sunlight {
    pub fn new(v1: u8, v2: u8, v3: u8) -> Self {
        Sunlight {v1, v2, v3}
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Self {
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf).unwrap();
        Self::new(buf[0], buf[1], buf[2])
    }
}

/******************************************************************************/

pub struct LevelRes {
    pub paths: LevelPaths,
    pub params: GlobeTextureParams,
    pub landscape: Landscape<128>,
    /// Per-player starting positions read from `.dat` (`PLAYERSAVEINFO psi[4]`,
    /// `pop.h:1062`). Index = tribe slot (0=Blue … 3=Green).
    pub player_starts: [PlayerSaveInfo; 4],
    pub sunlight: Sunlight,
    pub units: Vec<UnitRaw>,
    /// OBJS bank number (HDR byte 97). Selects which objs0-{N}.dat to load.
    pub obj_bank: u8,
    /// Parsed level header (`.hdr` file). Carries level flags, markers, default abilities.
    pub header: LevelHeader,
}

impl LevelRes {
    /// Load a level (`.dat` + `.hdr`). Panics on I/O failure or corrupted input.
    /// Prefer [`LevelRes::try_new`] when the caller wants to handle errors.
    pub fn new(base: &Path, level_num: u8, level_type_opt: Option<&str>) -> LevelRes {
        Self::try_new(base, level_num, level_type_opt)
            .unwrap_or_else(|e| panic!("LevelRes::new failed: {e}"))
    }

    /// Fallible load. Returns [`LevelLoadError`] for I/O errors, truncated
    /// files, or unrecognised header bytes.
    pub fn try_new(
        base: &Path,
        level_num: u8,
        level_type_opt: Option<&str>,
    ) -> Result<LevelRes, LevelLoadError> {
        let level_dir = base.join("levels");
        let (level_path, level_type, obj_bank, header) = read_level_result(&level_dir, level_num)?;

        let paths = match level_type_opt {
            Some(v) => LevelPaths::from_default_dir(base, v),
            None => LevelPaths::from_default_dir(base, &level_type),
        };

        let mut file = File::options()
            .read(true)
            .open(&level_path)
            .map_err(|e| LevelLoadError::Io {
                path: level_path.clone(),
                source: e,
            })?;
        let landscape = Landscape::from_reader(&mut file);
        seek_or(&mut file, std::io::SeekFrom::Start(0x8000), &level_path)?;
        //read 0x4000 (LandBlocks in v2)
        seek_or(&mut file, std::io::SeekFrom::Current(0x4000), &level_path)?;
        //read 0x4000 (LandOrients in v2)
        seek_or(&mut file, std::io::SeekFrom::Current(0x4000), &level_path)?;
        //read 0x4000 (NoAccessSquares)
        seek_or(&mut file, std::io::SeekFrom::Current(0x4000), &level_path)?;
        // PLAYERSAVEINFO psi[4] — per-player starting positions (16 bytes each).
        let mut player_starts: [PlayerSaveInfo; 4] = [PlayerSaveInfo {
            start_pos_x: 0, start_pos_y: 0,
            _future1: 0, _future2: 0, _future3: 0,
        }; 4];
        for slot in player_starts.iter_mut() {
            *slot = PlayerSaveInfo::from_reader(&mut file).ok_or_else(|| {
                LevelLoadError::TruncatedDat {
                    path: level_path.clone(),
                    where_: "PLAYERSAVEINFO".into(),
                }
            })?;
        }
        let sunlight = Sunlight::from_reader(&mut file);
        // DAT unit section is fixed-size: 2000 slots * 55 bytes each.
        // Do not read UnitRaw entries until EOF, because trailing non-unit bytes
        // in the DAT file can be misinterpreted as extra bogus units.
        let units = try_read_fixed_unit_slots(&mut file, LEVEL_UNIT_SLOTS, &level_path)?;
        let params = GlobeTextureParams::from_level(&paths);
        Ok(LevelRes {
            paths,
            params,
            landscape,
            player_starts,
            sunlight,
            units,
            obj_bank,
            header,
        })
    }
}

fn seek_or<S: Seek>(seekable: &mut S, pos: std::io::SeekFrom, path: &Path) -> Result<(), LevelLoadError> {
    seekable.seek(pos).map(|_| ()).map_err(|e| LevelLoadError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

fn try_read_fixed_unit_slots<R: Read>(
    reader: &mut R,
    count: usize,
    path: &Path,
) -> Result<Vec<UnitRaw>, LevelLoadError> {
    let mut units = Vec::with_capacity(count);
    for idx in 0..count {
        let unit = UnitRaw::from_reader(reader).ok_or_else(|| LevelLoadError::TruncatedDat {
            path: path.to_path_buf(),
            where_: format!("unit slot {}/{}", idx + 1, count),
        })?;
        units.push(unit);
    }
    Ok(units)
}

fn read_fixed_unit_slots<R: Read>(reader: &mut R, count: usize) -> Vec<UnitRaw> {
    let mut units = Vec::with_capacity(count);
    for idx in 0..count {
        let unit = UnitRaw::from_reader(reader).unwrap_or_else(|| {
            panic!(
                "Level DAT is truncated while reading unit slot {}/{}",
                idx + 1,
                count
            )
        });
        units.push(unit);
    }
    units
}

fn read_level_result(base: &Path, num: u8) -> Result<(PathBuf, String, u8, LevelHeader), LevelLoadError> {
    let dat_path = LevelPaths::dat_path(base, num);
    let hdr_path = LevelPaths::hdr_path(base, num);
    let hdr_data = std::fs::read(&hdr_path).map_err(|e| LevelLoadError::Io {
        path: hdr_path.clone(),
        source: e,
    })?;
    let landscape_type = try_read_landscape_type(&hdr_data, &hdr_path)?;
    let obj_bank = if hdr_data.len() > 97 { hdr_data[97] } else { 0 };
    let header = LevelHeader::from_bytes(&hdr_data).ok_or_else(|| LevelLoadError::TruncatedHdr {
        path: hdr_path.clone(),
        actual_len: hdr_data.len(),
    })?;
    Ok((dat_path, landscape_type, obj_bank, header))
}

pub fn read_level(base: &Path, num: u8) -> (PathBuf, String, u8, LevelHeader) {
    let dat_path = LevelPaths::dat_path(base, num);
    let hdr_path = LevelPaths::hdr_path(base, num);
    let hdr_data = read_bin(&hdr_path);
    let landscape_type = read_landscape_type_from_bytes(&hdr_data);
    let obj_bank = if hdr_data.len() > 97 { hdr_data[97] } else { 0 };
    let header = LevelHeader::from_bytes(&hdr_data).unwrap_or_default();
    (dat_path, landscape_type, obj_bank, header)
}

/******************************************************************************/

pub fn read_landscape_type(hdr_path: &Path) -> String {
    let hdr_data = read_bin(hdr_path);
    read_landscape_type_from_bytes(&hdr_data)
}

fn read_landscape_type_from_bytes(hdr_data: &[u8]) -> String {
    if hdr_data.len() < 97 {
        panic!("Hdr is too small {}", hdr_data.len())
    }
    let type_int = hdr_data[96];
    match type_int {
        0 ..= 9 => {
            let v = 0x30 + type_int;
            std::char::from_u32(v as u32).unwrap().to_string().to_lowercase()
        },
        i if i < 36 => {
            let v = 0x41 + (type_int - 10);
            std::char::from_u32(v as u32).unwrap().to_string().to_lowercase()
        },
        _ => panic!("Wrong landscape type {type_int:?}")
    }
}

fn try_read_landscape_type(hdr_data: &[u8], path: &Path) -> Result<String, LevelLoadError> {
    if hdr_data.len() < 97 {
        return Err(LevelLoadError::TruncatedHdr {
            path: path.to_path_buf(),
            actual_len: hdr_data.len(),
        });
    }
    let type_int = hdr_data[96];
    match type_int {
        0..=9 => {
            let v = 0x30 + type_int;
            Ok(std::char::from_u32(v as u32).unwrap().to_string().to_lowercase())
        }
        i if i < 36 => {
            let v = 0x41 + (type_int - 10);
            Ok(std::char::from_u32(v as u32).unwrap().to_string().to_lowercase())
        }
        _ => Err(LevelLoadError::UnknownLandscapeType {
            path: path.to_path_buf(),
            byte_96: type_int,
        }),
    }
}

/******************************************************************************/

pub fn read_bin(path: &Path) -> Vec<u8> {
    let mut f = OpenOptions::new().read(true).open(path).unwrap();
    let mut vec = Vec::new();
    f.read_to_end(&mut vec).unwrap();
    vec
}

#[allow(dead_code)]
fn read_bin16(path: &Path) -> Vec<u16> {
    let buf = read_bin(path);
    let mut vec = vec![0; buf.len() / 2];
    for (i, n) in (0..).zip(buf.chunks(2).take(vec.len())) {
        if n.len() == 2 {
            vec[i] = u16::from_le_bytes([n[0], n[1]]);
        }
    }
    vec
}

fn read_bin_i8(path: &Path) -> Vec<i8> {
    let buf = read_bin(path);
    let mut v = std::mem::ManuallyDrop::new(buf);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

fn read_disp(path: &Path) -> Vec<i8> {
    // Mirror each row of the 256×256 displacement map left↔right.
    // Same off-by-one fix as `Landscape::flip`: the previous bound
    // `width/2 - 1` left the middle pair (cols 127↔128) un-swapped.
    let mut disp = read_bin_i8(path);
    let width = 256;
    for i in 0..width {
        for j in 0..(width / 2) {
            let n = i * width + j;
            let n1 = i * width + (width - 1) - j;
            disp.swap(n, n1);
        }
    }
    disp
}

#[cfg(test)]
mod tests {
    use super::read_fixed_unit_slots;
    use crate::data::types::BinDeserializer;
    use crate::data::units::UnitRaw;
    use std::io::Cursor;

    fn unit_raw_bytes(subtype: u8, model: u8, tribe_index: u8, loc_x: u16, loc_y: u16, angle: u32) -> [u8; 55] {
        let mut bytes = [0u8; 55];
        bytes[0] = subtype;
        bytes[1] = model;
        bytes[2] = tribe_index;
        bytes[3..5].copy_from_slice(&loc_x.to_le_bytes());
        bytes[5..7].copy_from_slice(&loc_y.to_le_bytes());
        bytes[7..11].copy_from_slice(&angle.to_le_bytes());
        bytes
    }

    #[test]
    fn landscape_flip_swaps_every_row_pair() {
        // Regression for an off-by-one in `Landscape::flip` that previously
        // skipped the middle pair of rows (63 ↔ 64 for N=128), leaving a
        // 2-row seam at the centre of the rendered terrain.
        use super::Landscape;
        // Build a 4×4 landscape where each row holds its own row index in every
        // column. After a correct horizontal flip we expect row r → 3-r.
        let mut ls: Landscape<4> = Landscape::new();
        for r in 0..4 {
            for c in 0..4 {
                ls.height[r][c] = r as u16;
            }
        }
        // The `flip` method is private; exercise it through a small wrapper.
        // We use the public `from_reader` path with synthetic bytes that
        // recreate the same pre-flip layout, then assert the post-flip values.
        // Construct bytes such that height[i%N][i/N] = i%N for all flat
        // positions — i.e. a flat array where element index i has value i%N.
        let n = 4usize;
        let mut buf = Vec::with_capacity(n * n * 2);
        for i in 0..(n * n) {
            buf.extend_from_slice(&((i % n) as u16).to_le_bytes());
        }
        let ls: Landscape<4> = Landscape::from_reader(&mut std::io::Cursor::new(buf));
        // After a correct flip every row r holds (n - 1 - r). Critically, the
        // middle pair (rows 1 and 2 for n=4) must have been swapped.
        for c in 0..n {
            assert_eq!(ls.height[0][c], 3);
            assert_eq!(ls.height[1][c], 2, "middle pair must swap (was the off-by-one bug)");
            assert_eq!(ls.height[2][c], 1, "middle pair must swap (was the off-by-one bug)");
            assert_eq!(ls.height[3][c], 0);
        }
    }

    #[test]
    fn read_fixed_unit_slots_reads_exact_slot_count() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&unit_raw_bytes(7, 1, 0, 0x1234, 0x5678, 0x9ABC_DEF0));
        bytes.extend_from_slice(&unit_raw_bytes(2, 1, 1, 0x0102, 0x0304, 0x0506_0708));
        // Extra valid UnitRaw + trailing noise should not be consumed.
        bytes.extend_from_slice(&unit_raw_bytes(5, 1, 2, 0x2222, 0x3333, 0x4444_5555));
        bytes.extend_from_slice(&[0xAA; 13]);

        let mut cursor = Cursor::new(bytes);
        let units = read_fixed_unit_slots(&mut cursor, 2);
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].subtype, 7);
        assert_eq!(units[1].subtype, 2);

        // The next entry is still available in the stream.
        let next = UnitRaw::from_reader(&mut cursor).expect("expected unread third unit");
        assert_eq!(next.subtype, 5);
    }
}

pub fn read_pal(paths: &LevelPaths) -> Vec<u8> {
    read_bin(&paths.palette)
}

/// Build the 256-entry sky palette interpolation table per FUN_004dc3f0.
///
/// The game sorts 13 sky colors (pal[0x71..0x7E]) by perceived luminance,
/// then builds a table mapping each brightness level 0..255 to the palette
/// index of the closest sky color. This produces smooth gradients.
pub fn build_sky_interp_table(pal: &[u8]) -> [u8; 256] {
    let mut table = [0x70u8; 256];

    struct Entry { pal_idx: u8, lum: u32 }
    let mut entries: Vec<Entry> = Vec::with_capacity(13);
    for i in 1..=13u8 {
        let p = (0x70 + i) as usize * 4;
        let r = pal[p] as u32;
        let g = pal[p + 1] as u32;
        let b = pal[p + 2] as u32;
        entries.push(Entry { pal_idx: 0x70 + i, lum: r * 66 + g * 129 + b * 25 });
    }

    // Sort by luminance (ascending) — game uses selection sort, result is identical
    entries.sort_by_key(|e| e.lum);

    // Normalized luminance: (raw_lum >> 8), clamped to 255
    let norm_lums: Vec<u8> = entries.iter()
        .map(|e| (e.lum >> 8).min(255) as u8)
        .collect();

    let min_lum = *norm_lums.iter().min().unwrap() as i32;
    let max_lum = *norm_lums.iter().max().unwrap() as i32;
    let range = max_lum - min_lum;

    // Build 256-entry table: linearly sweep [min_lum, max_lum],
    // for each target find the sorted entry with closest luminance
    let mut acc: i32 = 0;
    for i in 0..256 {
        let target = min_lum + (acc >> 8);

        let mut best_idx = 0;
        let mut best_dist = i32::MAX;
        for (j, &lum) in norm_lums.iter().enumerate() {
            let dist = (lum as i32 - target).abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = j;
            }
        }

        table[i] = entries[best_idx].pal_idx;
        acc += range;
    }

    table
}

/******************************************************************************/

pub struct GlobeTextureParams {
    pub disp0: Vec<i8>,
    pub cliff0: Vec<u8>,
    pub bigf0: Vec<u8>,
    pub fade0: Vec<u8>,
    pub ghost0: Vec<u8>,
    pub static_landscape_array: Vec<u16>,
    pub palette: Vec<u8>,
    pub watdisp: Vec<u8>,
}

impl GlobeTextureParams {
    pub fn from_level(paths: &LevelPaths) -> Self {
        Self {
            bigf0: read_bin(&paths.bigf0),
            cliff0: read_bin(&paths.cliff0),
            disp0: read_disp(&paths.disp0),
            fade0: read_bin(&paths.fade0),
            ghost0: read_bin(&paths.ghost0),
            static_landscape_array: Self::make_static_array(),
            palette: read_bin(&paths.palette),
            watdisp: read_bin(&paths.watdisp),
        }
    }

    pub fn make_static_array() -> Vec<u16> {
        let mut v = vec![0; 1152];
        for (i, elem) in v.iter_mut().enumerate() {
            if i < 128 {
                *elem = 0x140;
            } else if i < 362 {
                *elem = (0xd3d - (1152 - i) * 3) as u16;
            } else {
                *elem = 0x400;
            }
        }
        v
    }
}

/******************************************************************************/

pub struct Landscape<const N: usize> {
    pub height: [[u16; N]; N],
}

impl<const N: usize> Landscape<N> {
    pub fn new() -> Self {
        Self{height: [[0u16; N]; N]}
    }

    pub fn land_size(&self) -> usize {
        N
    }

    fn flip(&mut self) {
        // Mirror rows: swap pairs (0, N-1), (1, N-2), …, (N/2 - 1, N/2).
        // The previous bound was `width/2 - 1`, which skipped the middle pair
        // (rows 63 ↔ 64 for N=128) and left a 2-row seam at the centre of the
        // rendered terrain.
        let width = N;
        for i in 0..width {
            for j in 0..(width / 2) {
                let n1 = (width - 1) - j;
                let v = self.height[j][i];
                self.height[j][i] = self.height[n1][i];
                self.height[n1][i] = v;
            }
        }
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Self {
        let mut s = Self::new();
        let mut buf = Vec::new();
        let _file_size = reader.read_to_end(&mut buf);
        for (i, n) in (0..).zip(buf.chunks(2).take(N*N)) {
            if n.len() == 2 {
                let val = u16::from_le_bytes([n[0], n[1]]);
                s.height[i%N][i/N] = val;
            }
        }
        s.flip();
        s
    }

    pub fn from_file(path: &Path) -> Self {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader(&mut file)
    }

    pub fn is_land_adj(&self, i: usize, j: usize) -> bool {
        if self.height[i][j] > 0 {
            return false;
        }
        let i_u = (i+1) % N;
        let j_u = (j+1) % N;
        let i_d = if i == 0 { N-1 } else { i - 1 };
        let j_d = if j == 0 { N-1 } else { j - 1 };
        (self.height[i][j_d] > 0) ||
               (self.height[i][j_u] > 0) ||
               (self.height[i_d][j] > 0) ||
               (self.height[i_u][j] > 0) ||
               (self.height[i_u][j_d] > 0) ||
               (self.height[i_u][j_u] > 0) ||
               (self.height[i_d][j_d] > 0) ||
               (self.height[i_d][j_u] > 0)
    }

    pub fn make_shores(&self) -> Self{
        let mut output = Self{height: self.height};
        for i in 0..N {
            for j in 0..N {
                if self.height[i][j] == 0 && self.is_land_adj(i, j) {
                    output.height[i][j] = 1;
                }
            }
        }
        output
    }

    pub fn to_vec(&self) -> Vec<u32> {
        let mut vec = vec![0u32; N*N];
        for i in 0..N {
            for j in 0..N {
                vec[i*N + j] = self.height[i][j] as u32;
            }
        }
        vec
    }
}

impl<const N: usize> Default for Landscape<N> {
    fn default() -> Self {
        Self::new()
    }
}

/******************************************************************************/

// Level header (`.hdr` file). Cross-reference: `pop.h:1017-1043` LEVELHEADERv2/v3.
//
// Layout (LEVELHEADERv2, packed):
//   offset 0..56   PLAYERTHINGS DefaultThings  (starting spells/buildings)
//   offset 56..88  CHAR Name[32]
//   offset 88      UBYTE NumPlayers
//   offset 89..92  UBYTE ComputerPlayerIndex[3]
//   offset 92..96  UBYTE DefaultAllies[4]
//   offset 96      UBYTE LevelType                (already parsed via read_landscape_type)
//   offset 97      UBYTE ObjectsBankNum           (already parsed via obj_bank)
//   offset 98      UBYTE LevelFlags
//   offset 99      UBYTE Pad[1]
//   offset 100..612 UWORD Markers[256]            (((bz*2)<<8) | (bx*2))
//   offset 612     UWORD StartPos
//   offset 614     UWORD StartAngle
//   offset 616     UBYTE Version (v3)
//   offset 617..   v3 extras (MaxAltPoints/Objects/Players, Script2[10][32])

/// `pop.h:1001` `struct PLAYERTHINGS`. Per-player starting abilities (56 bytes).
///
/// Drives initial spell / building / vehicle availability at level start. Bitmasks
/// index into the spell and building model tables (`pop.h:706-786` and `pop.h:574-700`).
#[derive(Debug, Copy, Clone, Default)]
pub struct PlayerThings {
    pub spells_available: u32,
    pub buildings_available: u32,
    pub buildings_available_level: u32,
    pub buildings_available_once: u32,
    /// Union of `SpellsAvailableLevel` / `SpellsNotCharging` (`pop.h:1007-1010`).
    pub spells_available_level: u32,
    pub spells_available_once: [u8; 32],
    pub vehicles_available: u16,
    pub training_mana_off: u8,
    pub flags: u8,
}

impl PlayerThings {
    fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 56 { return None; }
        let mut spells_available_once = [0u8; 32];
        spells_available_once.copy_from_slice(&buf[20..52]);
        Some(Self {
            spells_available:           u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            buildings_available:        u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            buildings_available_level:  u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            buildings_available_once:   u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            spells_available_level:     u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            spells_available_once,
            vehicles_available:         u16::from_le_bytes(buf[52..54].try_into().unwrap()),
            training_mana_off:          buf[54],
            flags:                      buf[55],
        })
    }

    /// Returns `true` if the given spell type (`pop.h:706-786` `M_SPELL_*`,
    /// 1-based) is available at level start. Checks both the permanent and
    /// per-level bitmasks.
    pub fn is_spell_available(&self, spell_id: u8) -> bool {
        bit_set(self.spells_available, spell_id) ||
        bit_set(self.spells_available_level, spell_id)
    }

    /// Returns the count of one-shot uses remaining for a spell.
    /// `spells_available_once[N]` is decremented as the spell is cast.
    pub fn one_shot_count(&self, spell_id: u8) -> u8 {
        let idx = spell_id as usize;
        if idx == 0 || idx > self.spells_available_once.len() { return 0; }
        // Layout: 1-based spell ids, but `spells_available_once[0]` is "spell 1".
        self.spells_available_once[idx - 1]
    }

    /// Returns `true` if the building type (`pop.h:574-700` `M_BUILDING_*`,
    /// 1-based) is available. Combines permanent + level-scoped + once flags.
    pub fn is_building_available(&self, building_id: u8) -> bool {
        bit_set(self.buildings_available, building_id) ||
        bit_set(self.buildings_available_level, building_id) ||
        bit_set(self.buildings_available_once, building_id)
    }

    /// Returns `true` if the given vehicle type (`pop.h:701-704` 1-based) is
    /// available at level start.
    pub fn is_vehicle_available(&self, vehicle_id: u8) -> bool {
        bit_set(self.vehicles_available as u32, vehicle_id)
    }

    /// Iterator over spell ids (1..=21) that are available.
    pub fn available_spells(&self) -> impl Iterator<Item = u8> + '_ {
        (1u8..=21).filter(move |&id| self.is_spell_available(id))
    }

    /// Iterator over building ids (1..=31) that are available.
    pub fn available_buildings(&self) -> impl Iterator<Item = u8> + '_ {
        (1u8..=31).filter(move |&id| self.is_building_available(id))
    }
}

/// Returns `true` when bit at position `(id - 1)` is set in `mask`.
/// Spell / building / vehicle ids are 1-based per `pop.h`.
fn bit_set(mask: u32, id: u8) -> bool {
    if id == 0 || id > 32 { return false; }
    (mask & (1u32 << (id - 1))) != 0
}

/// Decoded marker: `(cell_x, cell_y)` in 0..=255 grid coordinates.
/// `None` means "marker slot unused" (raw on-disk value `(0, 0)`,
/// matching `engine.cpp:2640` "if Markers[i] is (0.5, 0.5) it's unset").
pub type MarkerCell = Option<(u8, u8)>;

/// `pop.h:1017` LEVELHEADERv2 / `pop.h:1035` LEVELHEADERv3.
#[derive(Debug, Clone)]
pub struct LevelHeader {
    pub default_things: PlayerThings,
    pub name: String,
    pub num_players: u8,
    pub computer_player_index: [u8; 3],
    pub default_allies: [u8; 4],
    pub level_type: u8,
    pub objects_bank_num: u8,
    pub level_flags: u8,
    /// Decoded marker positions (256 slots). Index = marker id used by AI script
    /// commands (ATTACK_MARKER, SET_BASE_MARKER, etc.).
    pub markers: [MarkerCell; 256],
    pub start_pos: u16,
    pub start_angle: u16,
    /// `0` for legacy v2 headers (without the v3 extension trailer).
    pub version: u8,
    pub max_alt_points: u32,
    pub max_num_objects: u32,
    pub max_num_players: u32,
    /// Up to 10 secondary script names (v3 only, empty for v2 headers).
    pub script2: Vec<String>,
}

impl Default for LevelHeader {
    fn default() -> Self {
        Self {
            default_things: PlayerThings::default(),
            name: String::new(),
            num_players: 0,
            computer_player_index: [0; 3],
            default_allies: [0; 4],
            level_type: 0,
            objects_bank_num: 0,
            level_flags: 0,
            markers: [None; 256],
            start_pos: 0,
            start_angle: 0,
            version: 0,
            max_alt_points: 0,
            max_num_objects: 0,
            max_num_players: 0,
            script2: Vec::new(),
        }
    }
}

impl LevelHeader {
    /// Parse a `.hdr` file's bytes. Returns `None` if the buffer is shorter than the
    /// v2 fixed-size region (616 bytes). Optional v3 trailer is parsed when present.
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 616 { return None; }

        let default_things = PlayerThings::from_bytes(&buf[0..56])?;

        // Null-terminated CHAR Name[32]
        let name_bytes = &buf[56..88];
        let nlen = name_bytes.iter().position(|&b| b == 0).unwrap_or(32);
        let name = String::from_utf8_lossy(&name_bytes[..nlen]).into_owned();

        let num_players = buf[88];
        let computer_player_index = [buf[89], buf[90], buf[91]];
        let default_allies = [buf[92], buf[93], buf[94], buf[95]];
        let level_type = buf[96];
        let objects_bank_num = buf[97];
        let level_flags = buf[98];

        let mut markers = [None; 256];
        for (i, slot) in markers.iter_mut().enumerate() {
            let o = 100 + i * 2;
            let raw = u16::from_le_bytes([buf[o], buf[o + 1]]);
            *slot = decode_marker(raw);
        }

        let start_pos = u16::from_le_bytes([buf[612], buf[613]]);
        let start_angle = u16::from_le_bytes([buf[614], buf[615]]);

        // v3 trailer is optional.
        let (version, max_alt_points, max_num_objects, max_num_players, script2) =
            if buf.len() >= 616 + 1 + 12 {
                let version = buf[616];
                let max_alt_points  = u32::from_le_bytes(buf[617..621].try_into().unwrap());
                let max_num_objects = u32::from_le_bytes(buf[621..625].try_into().unwrap());
                let max_num_players = u32::from_le_bytes(buf[625..629].try_into().unwrap());
                let mut s2 = Vec::new();
                for i in 0..10 {
                    let o = 629 + i * 32;
                    if o + 32 > buf.len() { break; }
                    let bytes = &buf[o..o+32];
                    let nlen = bytes.iter().position(|&b| b == 0).unwrap_or(32);
                    if nlen == 0 { continue; }
                    s2.push(String::from_utf8_lossy(&bytes[..nlen]).into_owned());
                }
                (version, max_alt_points, max_num_objects, max_num_players, s2)
            } else {
                (0, 0, 0, 0, Vec::new())
            };

        Some(Self {
            default_things,
            name,
            num_players,
            computer_player_index,
            default_allies,
            level_type,
            objects_bank_num,
            level_flags,
            markers,
            start_pos,
            start_angle,
            version,
            max_alt_points,
            max_num_objects,
            max_num_players,
            script2,
        })
    }

    /// Returns the high-level level configuration flags decoded from `level_flags`.
    pub fn config(&self) -> LevelConfig {
        LevelConfig::from_flags(self.level_flags)
    }
}

/// Per-`engine.cpp:2693`, a marker word packs `((bz*2) << 8) | (bx*2)`.
/// The high byte is `z*2`, the low byte is `x*2`. After dividing by 2 we get a
/// cell coordinate in 0..=127 (the on-disk encoding uses 0..=255 for finer
/// rotation freedom, but markers are cell-aligned).
///
/// A raw value of `0` indicates an unused slot (matches `engine.cpp:2640`'s
/// "uninitialised at (0.5, 0.5)" check after the runtime offset).
fn decode_marker(raw: u16) -> MarkerCell {
    if raw == 0 { return None; }
    let lo = (raw & 0x00FF) as u8;
    let hi = ((raw >> 8) & 0x00FF) as u8;
    Some((lo / 2, hi / 2))
}

/// High-level level flags from `pop.h:945-950`.
///
/// Drives gameplay rules that vary per level — fog of war, shaman omnipresence,
/// whether guest spells are allowed, and whether reincarnation timeout applies.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct LevelConfig {
    /// `LEVEL_FLAGS_USE_FOG = 1 << 0`
    pub use_fog: bool,
    /// `LEVEL_FLAGS_SHAMAN_OMNI = 1 << 1`
    pub shaman_omni: bool,
    /// `LEVEL_FLAGS_NO_GUEST = 1 << 4`
    pub no_guest: bool,
    /// `LEVEL_NO_REINCARNATE_TIME = 1 << 5` — disables the reincarnation-timeout
    /// path that would normally eliminate a tribe whose population has been zero
    /// for too long.
    pub no_reincarnate_timer: bool,
}

impl LevelConfig {
    pub fn from_flags(flags: u8) -> Self {
        Self {
            use_fog:              (flags & (1 << 0)) != 0,
            shaman_omni:          (flags & (1 << 1)) != 0,
            no_guest:             (flags & (1 << 4)) != 0,
            no_reincarnate_timer: (flags & (1 << 5)) != 0,
        }
    }
}

#[cfg(test)]
mod hdr_tests {
    use super::*;

    fn synth_hdr() -> Vec<u8> {
        let mut buf = vec![0u8; 616];
        // PLAYERTHINGS @ 0..56
        buf[0..4].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes()); // spells_available
        buf[4..8].copy_from_slice(&0x1234_5678u32.to_le_bytes()); // buildings_available
        buf[52..54].copy_from_slice(&0x0AAAu16.to_le_bytes());    // vehicles_available
        buf[54] = 1;                                              // training_mana_off
        buf[55] = 7;                                              // flags
        // Name @ 56
        let name = b"Level Test\0";
        buf[56..56+name.len()].copy_from_slice(name);
        // NumPlayers, ComputerPlayerIndex, DefaultAllies
        buf[88] = 4;
        buf[89..92].copy_from_slice(&[1, 2, 3]);
        buf[92..96].copy_from_slice(&[0b0001, 0b0010, 0b0100, 0b1000]);
        // LevelType / ObjBank / Flags
        buf[96] = 0;
        buf[97] = 2;
        buf[98] = (1 << 0) | (1 << 5); // use_fog + no_reincarnate_timer
        // Markers: slot 1 = cell (10, 20)
        let bx = 10u8;
        let bz = 20u8;
        let marker_word = (((bz as u16) * 2) << 8) | ((bx as u16) * 2);
        buf[100..102].copy_from_slice(&0u16.to_le_bytes());           // slot 0 unused
        buf[102..104].copy_from_slice(&marker_word.to_le_bytes());    // slot 1 = (10, 20)
        // StartPos / StartAngle
        buf[612..614].copy_from_slice(&123u16.to_le_bytes());
        buf[614..616].copy_from_slice(&0x0100u16.to_le_bytes());      // 45°
        buf
    }

    #[test]
    fn parses_player_things_and_name() {
        let h = LevelHeader::from_bytes(&synth_hdr()).expect("parse");
        assert_eq!(h.default_things.spells_available, 0xDEAD_BEEF);
        assert_eq!(h.default_things.buildings_available, 0x1234_5678);
        assert_eq!(h.default_things.vehicles_available, 0x0AAA);
        assert_eq!(h.default_things.training_mana_off, 1);
        assert_eq!(h.default_things.flags, 7);
        assert_eq!(h.name, "Level Test");
        assert_eq!(h.num_players, 4);
        assert_eq!(h.computer_player_index, [1, 2, 3]);
        assert_eq!(h.default_allies, [0b0001, 0b0010, 0b0100, 0b1000]);
    }

    #[test]
    fn parses_markers_with_unused_slots() {
        let h = LevelHeader::from_bytes(&synth_hdr()).expect("parse");
        assert_eq!(h.markers[0], None, "slot 0 (raw 0) must be unused");
        assert_eq!(h.markers[1], Some((10, 20)), "slot 1 must decode to (10, 20)");
        assert_eq!(h.markers[255], None);
    }

    #[test]
    fn parses_start_pos_and_angle() {
        let h = LevelHeader::from_bytes(&synth_hdr()).expect("parse");
        assert_eq!(h.start_pos, 123);
        assert_eq!(h.start_angle, 0x0100);
    }

    #[test]
    fn level_config_decodes_all_flag_bits() {
        let h = LevelHeader::from_bytes(&synth_hdr()).expect("parse");
        let cfg = h.config();
        assert!(cfg.use_fog);
        assert!(cfg.no_reincarnate_timer);
        assert!(!cfg.shaman_omni);
        assert!(!cfg.no_guest);

        // Exhaustive check of each bit.
        let all = LevelConfig::from_flags(0xFF);
        assert!(all.use_fog && all.shaman_omni && all.no_guest && all.no_reincarnate_timer);
        let none = LevelConfig::from_flags(0);
        assert!(!none.use_fog && !none.shaman_omni && !none.no_guest && !none.no_reincarnate_timer);
    }

    #[test]
    fn short_buffer_returns_none() {
        let buf = vec![0u8; 100];
        assert!(LevelHeader::from_bytes(&buf).is_none());
    }

    #[test]
    fn marker_encoding_roundtrip() {
        // Per engine.cpp:2693, marker word = ((bz*2) << 8) | (bx*2).
        for &(x, z) in &[(0, 0), (1, 1), (64, 64), (127, 127), (10, 100)] {
            // (0, 0) sentinel-collides with "unused", so skip the round-trip for (0, 0).
            let raw = (((z as u16) * 2) << 8) | ((x as u16) * 2);
            let decoded = decode_marker(raw);
            if x == 0 && z == 0 {
                assert_eq!(decoded, None);
            } else {
                assert_eq!(decoded, Some((x as u8, z as u8)));
            }
        }
    }
}

/******************************************************************************/

/// `.ver` file contents. `pop.h:1156` `struct LEVELVERSION` — 68 bytes total.
///
/// Written by the original game (and by Pop-World-Editor) alongside the
/// `.dat`/`.hdr` pair. The game itself does not require this for gameplay;
/// it carries creation metadata for the level editor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LevelVersion {
    /// `VersionNum: SLONG` — author-supplied version (PopEdt writes 0x0B).
    pub version_num: i32,
    /// `CreatedBy[32]: CHAR` — null-terminated name string.
    pub created_by: String,
    /// `CreatedOn[28]: CHAR` — null-terminated build-date string
    /// (PopEdt writes `__DATE__ ", " __TIME__`).
    pub created_on: String,
    /// `CheckSum: SLONG` — level integrity checksum. Original behaviour unverified.
    pub checksum: i32,
}

impl LevelVersion {
    pub const ON_DISK_SIZE: usize = 68;

    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::ON_DISK_SIZE { return None; }
        fn read_cstr(bytes: &[u8]) -> String {
            let nlen = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            String::from_utf8_lossy(&bytes[..nlen]).into_owned()
        }
        Some(Self {
            version_num: i32::from_le_bytes(buf[0..4].try_into().unwrap()),
            created_by:  read_cstr(&buf[4..36]),
            created_on:  read_cstr(&buf[36..64]),
            checksum:    i32::from_le_bytes(buf[64..68].try_into().unwrap()),
        })
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        let buf = std::fs::read(path).ok()?;
        Self::from_bytes(&buf)
    }
}

#[cfg(test)]
mod ver_tests {
    use super::*;

    fn synth_ver() -> Vec<u8> {
        let mut buf = vec![0u8; LevelVersion::ON_DISK_SIZE];
        buf[0..4].copy_from_slice(&0x0Bi32.to_le_bytes());
        let by = b"Author Name\0";
        buf[4..4+by.len()].copy_from_slice(by);
        let on = b"2025-01-01, 12:00:00\0";
        buf[36..36+on.len()].copy_from_slice(on);
        buf[64..68].copy_from_slice(&0xCAFEBABEu32.to_le_bytes());
        buf
    }

    #[test]
    fn parses_version_fields() {
        let v = LevelVersion::from_bytes(&synth_ver()).expect("parse");
        assert_eq!(v.version_num, 0x0B);
        assert_eq!(v.created_by, "Author Name");
        assert_eq!(v.created_on, "2025-01-01, 12:00:00");
        assert_eq!(v.checksum as u32, 0xCAFEBABE);
    }

    #[test]
    fn short_buffer_returns_none() {
        assert!(LevelVersion::from_bytes(&[0u8; 10]).is_none());
    }

    #[test]
    fn empty_strings_decode_cleanly() {
        let buf = vec![0u8; LevelVersion::ON_DISK_SIZE];
        let v = LevelVersion::from_bytes(&buf).expect("parse");
        assert_eq!(v.created_by, "");
        assert_eq!(v.created_on, "");
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn try_read_landscape_type_rejects_short_hdr() {
        let buf = vec![0u8; 50];
        let err = try_read_landscape_type(&buf, Path::new("test.hdr")).unwrap_err();
        match err {
            LevelLoadError::TruncatedHdr { actual_len, .. } => assert_eq!(actual_len, 50),
            other => panic!("expected TruncatedHdr, got {other:?}"),
        }
    }

    #[test]
    fn try_read_landscape_type_rejects_unknown_byte_96() {
        let mut buf = vec![0u8; 97];
        buf[96] = 200; // out of valid 0..36 range
        let err = try_read_landscape_type(&buf, Path::new("test.hdr")).unwrap_err();
        match err {
            LevelLoadError::UnknownLandscapeType { byte_96, .. } => assert_eq!(byte_96, 200),
            other => panic!("expected UnknownLandscapeType, got {other:?}"),
        }
    }

    #[test]
    fn try_read_landscape_type_accepts_valid_digits_and_letters() {
        let mut buf = vec![0u8; 97];
        buf[96] = 0;
        assert_eq!(try_read_landscape_type(&buf, Path::new(".")).unwrap(), "0");
        buf[96] = 10;
        assert_eq!(try_read_landscape_type(&buf, Path::new(".")).unwrap(), "a");
        buf[96] = 35;
        assert_eq!(try_read_landscape_type(&buf, Path::new(".")).unwrap(), "z");
    }

    #[test]
    fn level_load_error_displays_human_readable_messages() {
        let e = LevelLoadError::TruncatedHdr {
            path: PathBuf::from("/x.hdr"),
            actual_len: 100,
        };
        let s = format!("{e}");
        assert!(s.contains("/x.hdr") && s.contains("100"));
    }
}

#[cfg(test)]
mod player_things_tests {
    use super::*;

    fn make_things(spells: u32, buildings: u32, level_spells: u32, vehicles: u16) -> PlayerThings {
        PlayerThings {
            spells_available: spells,
            buildings_available: buildings,
            buildings_available_level: 0,
            buildings_available_once: 0,
            spells_available_level: level_spells,
            spells_available_once: [0; 32],
            vehicles_available: vehicles,
            training_mana_off: 0,
            flags: 0,
        }
    }

    #[test]
    fn spell_availability_checks_both_masks() {
        // M_SPELL_BURN = 1, M_SPELL_BLAST = 2, M_SPELL_LIGHTNING = 3
        // Permanent: Burn + Blast; Level: Lightning
        let t = make_things(0b0011, 0, 0b0100, 0);
        assert!(t.is_spell_available(1));  // Burn — permanent
        assert!(t.is_spell_available(2));  // Blast — permanent
        assert!(t.is_spell_available(3));  // Lightning — level
        assert!(!t.is_spell_available(4)); // Tornado — neither
    }

    #[test]
    fn one_shot_count_indexes_by_spell_id() {
        let mut t = make_things(0, 0, 0, 0);
        t.spells_available_once[0] = 3;  // 1 use of spell id 1 -> stored at index 0
        t.spells_available_once[17] = 1; // spell id 18 -> Armageddon
        assert_eq!(t.one_shot_count(1), 3);
        assert_eq!(t.one_shot_count(18), 1);
        assert_eq!(t.one_shot_count(0), 0);
        assert_eq!(t.one_shot_count(99), 0); // out of range
    }

    #[test]
    fn building_availability_combines_all_three_masks() {
        let t = PlayerThings {
            buildings_available:        0b0001, // bit 0 -> id 1
            buildings_available_level:  0b0010, // bit 1 -> id 2
            buildings_available_once:   0b0100, // bit 2 -> id 3
            ..PlayerThings::default()
        };
        assert!(t.is_building_available(1));
        assert!(t.is_building_available(2));
        assert!(t.is_building_available(3));
        assert!(!t.is_building_available(4));
    }

    #[test]
    fn vehicle_availability_uses_u16_mask() {
        // M_VEHICLE_BOAT_1 = 1, M_VEHICLE_AIRSHIP_1 = 3
        let t = make_things(0, 0, 0, 0b0000_0101);
        assert!(t.is_vehicle_available(1));
        assert!(!t.is_vehicle_available(2));
        assert!(t.is_vehicle_available(3));
    }

    #[test]
    fn available_spells_yields_only_set_ids() {
        let t = make_things(0b1010_1, 0, 0b0_1010_0000, 0);
        // bits 0, 2, 4 (spells 1, 3, 5) and bits 5, 7 (spells 6, 8)
        let ids: Vec<u8> = t.available_spells().collect();
        assert_eq!(ids, vec![1, 3, 5, 6, 8]);
    }

    #[test]
    fn available_buildings_yields_only_set_ids() {
        let t = PlayerThings {
            buildings_available: 0b1_0001, // bits 0, 4 -> ids 1, 5
            ..PlayerThings::default()
        };
        let ids: Vec<u8> = t.available_buildings().collect();
        assert_eq!(ids, vec![1, 5]);
    }

    #[test]
    fn bit_set_handles_edge_cases() {
        assert!(!bit_set(0xFFFF_FFFF, 0));   // id 0 invalid
        assert!(!bit_set(0xFFFF_FFFF, 33));  // id > 32 invalid
        assert!(bit_set(0x8000_0000, 32));   // top bit
    }
}


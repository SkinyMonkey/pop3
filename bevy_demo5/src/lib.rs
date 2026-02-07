//! Populous: The Beginning - Authentic Rendering Library
//!
//! This crate provides tools for rendering assets from Populous: The Beginning.

pub mod animation_data;

// =============================================================================
// SHARED CONSTANTS AND UTILITIES
// =============================================================================

pub const GAME_PATH: &str = "/Users/adriencandiotti/Library/Containers/com.isaacmarovitz.Whisky/Bottles/74820C9D-5F8C-4BFE-B5DB-90E1DE818D3F/drive_c/GOG Games/Populous - The Beginning";

/// Decrypt palette data using XOR rotating mask (from faithful pls::decode)
pub fn palette_decrypt(data: &mut [u8]) {
    let mut m: u8 = 3;
    for v in data.iter_mut() {
        *v = !(*v ^ (1 << (m.wrapping_sub(3) & 7)));
        m = m.wrapping_add(1);
    }
}

/// Load and decrypt a palette file. Returns 256 RGBA colors.
/// Alpha convention: file alpha > 0 = transparent → inverted to standard RGBA.
pub fn load_palette(path: &str) -> Option<Vec<[u8; 4]>> {
    let mut data = std::fs::read(path).ok()?;
    if data.len() < 1024 {
        return None;
    }
    palette_decrypt(&mut data);
    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let off = i * 4;
        let a = data[off + 3];
        palette.push([data[off], data[off + 1], data[off + 2], if a > 0 { 0 } else { 255 }]);
    }
    Some(palette)
}

/// Load a raw (non-encrypted) palette file. Returns 256 RGBA colors, all opaque.
pub fn load_palette_raw(path: &str) -> Option<Vec<[u8; 4]>> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 1024 {
        return None;
    }
    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let off = i * 4;
        palette.push([data[off], data[off + 1], data[off + 2], 255]);
    }
    Some(palette)
}

// =============================================================================
// PSFB SPRITE FORMAT
// =============================================================================

pub struct SpriteFrame {
    pub width: u16,
    pub height: u16,
    /// Row-major pixel indices (255 = transparent)
    pub pixels: Vec<Vec<u8>>,
}

/// Get the number of sprites in a PSFB file
pub fn psfb_sprite_count(data: &[u8]) -> Option<usize> {
    if data.len() < 8 || &data[0..4] != b"PSFB" {
        return None;
    }
    Some(u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize)
}

/// Load a single sprite frame from PSFB file data (already read into memory)
pub fn psfb_load_frame(data: &[u8], frame_index: usize) -> Option<SpriteFrame> {
    let frame_count = psfb_sprite_count(data)?;
    if frame_index >= frame_count {
        return None;
    }

    let header_offset = 8 + frame_index * 8;
    let width = u16::from_le_bytes([data[header_offset], data[header_offset + 1]]);
    let height = u16::from_le_bytes([data[header_offset + 2], data[header_offset + 3]]);
    let pixel_offset = u32::from_le_bytes([
        data[header_offset + 4], data[header_offset + 5],
        data[header_offset + 6], data[header_offset + 7],
    ]) as usize;

    let mut pos = pixel_offset;
    let mut pixels = vec![vec![255u8; width as usize]; height as usize];

    for row in 0..height as usize {
        let mut col = 0usize;
        while pos < data.len() {
            let val = data[pos] as i8;
            pos += 1;

            if val == 0 {
                break;
            } else if val > 0 {
                for _ in 0..val {
                    if col < width as usize && pos < data.len() {
                        pixels[row][col] = data[pos];
                        pos += 1;
                        col += 1;
                    }
                }
            } else {
                col += (-val) as usize;
            }
        }
    }

    Some(SpriteFrame { width, height, pixels })
}

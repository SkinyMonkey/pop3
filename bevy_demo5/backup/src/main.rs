//! Populous: The Beginning - Authentic Rendering Demo
//!
//! This demo uses exact constants and algorithms from the original game,
//! extracted via Ghidra reverse engineering:
//!
//! Key constants from Ghidra:
//! - Curvature: 0xB3B0 (46000) at DAT_007b8fb4
//! - Near plane: 0x1964 (6500) at DAT_007b8fbc
//! - FOV exponent: 0x0B (11) at DAT_007b8fc0
//! - Distance threshold: ~0x2000 at DAT_008853dd
//! - Attenuation multiplier: ~0x10 at DAT_008853d9
//!
//! The curvature formula: curvature = ((cam_z*2)² + (cam_x*2)²) × 0xB3B0 >> 32

use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::PresentMode;
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};

// =============================================================================
// CONSTANTS FROM POPWORLDEDITOR AND GHIDRA REVERSE ENGINEERING
// =============================================================================

// World dimensions
const WORLD_CELLS: usize = 128;
const CELL_SIZE: f32 = 128.0;
const WORLD_EXTENT: f32 = WORLD_CELLS as f32 * CELL_SIZE;

// Level file loading constants
const LEVEL_BASE_PATH: &str = "/Users/adriencandiotti/Pop/Original/levels";
const DATA_BASE_PATH: &str = "/Users/adriencandiotti/Pop/Original/data";
const HEIGHTMAP_FILE_OFFSET: usize = 0x0000;
const HEIGHTMAP_BYTES: usize = WORLD_CELLS * WORLD_CELLS * 2;  // 128×128×2 = 32,768 bytes
const CELLFLAGS_FILE_OFFSET: usize = 0x8000;
const CELLFLAGS_BYTES: usize = WORLD_CELLS * WORLD_CELLS * 4;  // 128×128×4 = 65,536 bytes

// Water cell flag (from Ghidra: g_CellFlags & 0x200)
const CELL_FLAG_WATER: u32 = 0x200;

// Circular viewport radius (from PopWorldEditor: VIEW_RANGE = 30 cells)
// The editor uses 30 cells but we can extend it for better visibility
const VIEW_RANGE: f32 = 50.0;  // Visible terrain range in cells
const VIEWPORT_RADIUS: f32 = VIEW_RANGE * CELL_SIZE;

// Spherical curvature constants (from PopWorldEditor engine.h)
// SPHERE_RATIO = 0.01f - controls how much terrain curves down with distance
// MAX_SPHERICAL_POS_Y = 20.0f - camera height threshold for spherical mode
const SPHERE_RATIO: f32 = 0.01;
const MAX_SPHERICAL_POS_Y: f32 = 20.0;

// Projection constants (from Ghidra - Camera_WorldToScreen @ 0x0046ea30)
const CURVATURE_SCALE: i64 = 0xB3B0;        // DAT_007b8fb4 = 46000
const NEAR_PLANE: i32 = 0x1964;              // DAT_007b8fbc = 6500
const FOV_EXPONENT: u32 = 0x0B;              // DAT_007b8fc0 = 11

// Shading constants (from Terrain_CreateTriangleCommand @ 0x0046f6f0)
const DISTANCE_THRESHOLD: i32 = 0x2000;      // DAT_008853dd (approx)
const ATTENUATION_MULT: i32 = 0x10;          // DAT_008853d9 (approx)

// Fixed-point constants
const FIXED_SHIFT: i32 = 14;
const FIXED_ONE: i32 = 0x4000;

// Height scaling (from PopWorldEditor: GROUND_BLOCK_HEIGHT = 0.005f)
const GROUND_BLOCK_HEIGHT: f32 = 0.005;
const GROUND_HEIGHT_MAX: u16 = 0x0700;  // 1792 - max walkable height

// =============================================================================
// LANDSCAPE_COLORS PALETTE FROM POPWORLDEDITOR
// 256 colors: dark blue (water) -> green -> brown -> tan -> white (peaks)
// =============================================================================
const LANDSCAPE_COLORS: [u32; 256] = [
    0x2D2D96, 0x028000, 0x048000, 0x078000, 0x088000, 0x0A8000, 0x0C8000, 0x0E8000,
    0x108000, 0x128000, 0x158000, 0x168000, 0x188000, 0x1A8000, 0x1C8000, 0x1E8000,
    0x208000, 0x228000, 0x248000, 0x278000, 0x288000, 0x2A8000, 0x2C8000, 0x2E8000,
    0x308000, 0x328000, 0x348000, 0x378000, 0x388000, 0x3A8000, 0x3C8000, 0x3E8000,
    0x408000, 0x428000, 0x458000, 0x468000, 0x488000, 0x4A8000, 0x4D8000, 0x4E8000,
    0x508000, 0x528000, 0x548000, 0x568000, 0x588000, 0x5A8000, 0x5C8000, 0x5E8000,
    0x608100, 0x628100, 0x648100, 0x668100, 0x688100, 0x6A8100, 0x6C8100, 0x6E8100,
    0x708100, 0x728100, 0x748100, 0x768100, 0x788100, 0x7A8100, 0x7C8100, 0x7E8100,
    0x808100, 0x828100, 0x848100, 0x868100, 0x888100, 0x8A8100, 0x8C8100, 0x8E8100,
    0x908200, 0x928200, 0x948200, 0x968200, 0x988200, 0x9A8200, 0x9C8200, 0x9E8200,
    0xA08200, 0xA28200, 0xA48200, 0xA68200, 0xA88200, 0xAA8200, 0xAC8200, 0xAE8200,
    0xB08200, 0xB28200, 0xB48200, 0xB68200, 0xB88200, 0xBA8200, 0xBC8200, 0xBE8200,
    0xC08300, 0xC08402, 0xC18504, 0xC18606, 0xC28709, 0xC2880A, 0xC3890D, 0xC3890E,
    0xC38A0F, 0xC48B10, 0xC48B11, 0xC48C12, 0xC58D14, 0xC58D15, 0xC58E16, 0xC58E17,
    0xC68F18, 0xC68F1A, 0xC6901B, 0xC6901C, 0xC7911E, 0xC7921F, 0xC79220, 0xC89321,
    0xC89422, 0xC89423, 0xC99524, 0xC99526, 0xC99627, 0xCA9728, 0xCA972A, 0xCA982B,
    0xCA982C, 0xCB992D, 0xCB992E, 0xCB9A2F, 0xCC9B30, 0xCC9B32, 0xCC9C33, 0xCC9C34,
    0xCD9D35, 0xCD9E37, 0xCD9E38, 0xCE9F39, 0xCE9F3A, 0xCEA03C, 0xCFA13D, 0xCFA13E,
    0xCFA23F, 0xCFA240, 0xD0A341, 0xD0A342, 0xD0A444, 0xD1A545, 0xD1A546, 0xD1A648,
    0xD2A749, 0xD2A74A, 0xD2A84B, 0xD2A84C, 0xD3A94D, 0xD3AA4F, 0xD3AA50, 0xD4AB51,
    0xD4AB52, 0xD4AC54, 0xD4AC55, 0xD5AD56, 0xD5AE57, 0xD5AE58, 0xD6AF5A, 0xD6B05C,
    0xD7B15E, 0xD7B25F, 0xD7B261, 0xD8B362, 0xD8B463, 0xD8B465, 0xD9B566, 0xD9B668,
    0xDAB76A, 0xDAB86B, 0xDAB86D, 0xDBB96E, 0xDBBA6F, 0xDBBA70, 0xDCBB72, 0xDCBC74,
    0xDDBD77, 0xDDBE78, 0xDEBF7A, 0xDEC07C, 0xDFC17E, 0xDFC282, 0xE0C384, 0xE0C486,
    0xE1C689, 0xE2C68B, 0xE2C78D, 0xE3C88E, 0xE3C991, 0xE4CA93, 0xE4CB94, 0xE5CC96,
    0xE5CD98, 0xE5CE9A, 0xE6CE9B, 0xE6CF9D, 0xE7D1A0, 0xE7D1A2, 0xE8D2A3, 0xE8D3A5,
    0xE9D4A7, 0xE9D5A9, 0xEAD6AA, 0xEAD7AC, 0xEBD8AF, 0xEBD9B0, 0xECD9B2, 0xECDAB4,
    0xEDDBB6, 0xEDDCB7, 0xEDDDB9, 0xEEDEBB, 0xEEDFBD, 0xEFE0BF, 0xEFE1C1, 0xF0E2C3,
    0xF0E2C5, 0xF1E3C6, 0xF1E4C8, 0xF2E5CA, 0xF2E6CC, 0xF3E7CE, 0xF3E8D0, 0xF4E9D2,
    0xF4EAD3, 0xF5EAD5, 0xF5EBD7, 0xF5ECD9, 0xF6EDDA, 0xF7EEDD, 0xF7EFDF, 0xF7F0E1,
    0xF8F1E2, 0xF8F2E4, 0xF9F2E6, 0xF9F3E8, 0xFAF4E9, 0xFAF5EC, 0xFBF6EE, 0xFCF8F1,
    0xFCF9F3, 0xFDFAF5, 0xFDFBF6, 0xFDFBF8, 0xFEFCFA, 0xFFFEFD, 0xFFFEFE, 0xFFFFFF,
];

// =============================================================================
// AUTHENTIC CURVATURE ALGORITHM
// From Vertex_ApplyTransform @ 0x0046ebd0 and Camera_WorldToScreen @ 0x0046ea30
// =============================================================================

/// Apply the authentic curvature formula from the original game.
/// The key insight is that X and Z are DOUBLED before squaring, making
/// the curvature 4x stronger than a naive implementation.
fn apply_authentic_curvature(cam_x: i32, cam_y: i32, cam_z: i32, curvature_scale: i64) -> i32 {
    // Original code: dist_sq = (cam_z * 2)² + (cam_x * 2)²
    let dx2 = (cam_x * 2) as i64;
    let dz2 = (cam_z * 2) as i64;
    let dist_sq = dx2 * dx2 + dz2 * dz2;

    // Original code: curvature = (dist_sq * DAT_007b8fb4) >> 32
    let curvature = ((dist_sq * curvature_scale) >> 32) as i32;

    // Original code: cam_y = cam_y - curvature
    cam_y - curvature
}

// =============================================================================
// AUTHENTIC DISTANCE ATTENUATION
// From Terrain_CreateTriangleCommand @ 0x0046f6f0
// =============================================================================

/// Calculate distance-based brightness attenuation exactly as in the original.
fn calculate_distance_attenuation(camera_depth: i32, base_brightness: i32) -> i32 {
    let distance = camera_depth - DISTANCE_THRESHOLD;

    if distance <= 0 {
        return base_brightness;
    }

    // Original: dist_sq = (distance * distance) >> 16
    let dist_sq = ((distance as i64 * distance as i64) >> 16) as i32;

    // Original: attenuation = (dist_sq * (DAT_008853d9 << 8)) >> 16
    let attenuation = ((dist_sq as i64 * ((ATTENUATION_MULT as i64) << 8)) >> 16) as i32;

    // Original: clamp(attenuation, 0, 0x20)
    let attenuation = attenuation.clamp(0, 0x20);

    // Original: brightness = brightness - attenuation
    (base_brightness - attenuation).max(0)
}

/// Convert LANDSCAPE_COLORS palette entry (0xRRGGBB) to RGB floats (0.0-1.0)
fn palette_to_rgb(color: u32) -> (f32, f32, f32) {
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    (r, g, b)
}

/// Get terrain color from height using LANDSCAPE_COLORS palette
/// From PopWorldEditor: if(w > 8) w /= 8; else if(w > 0) w = 1; if(w > 0xFF) w = 0xFF;
fn get_terrain_color(height: u16) -> (f32, f32, f32) {
    let index = if height > 8 {
        (height / 8).min(255) as usize
    } else if height > 0 {
        1
    } else {
        0  // Water (height 0) uses first color
    };
    palette_to_rgb(LANDSCAPE_COLORS[index])
}

/// Apply PopWorldEditor-style spherical curvature
/// From engine.cpp: GroundHeight[ax][az].height -= f * f * SPHERE_RATIO;
/// where f = distance from camera
fn apply_sphere_curvature(height: f32, distance: f32) -> f32 {
    // Convert distance from world units to cell units for proper scaling
    let dist_cells = distance / CELL_SIZE;
    height - dist_cells * dist_cells * SPHERE_RATIO * CELL_SIZE
}

// =============================================================================
// SKY AND TERRAIN TEXTURE RENDERING (from PopResourceEditor file format analysis)
// =============================================================================

const SKY_DATA_PATH: &str = "/Users/adriencandiotti/Pop/Original/data";
const SKY_WIDTH: usize = 512;

// BigFade terrain texture dimensions (from PopResourceEditor BigFade.h)
// This is the primary landscape texture - a vertical strip containing terrain patterns
const BIGFADE_WIDTH: usize = 256;
const BIGFADE_HEIGHT: usize = 1152;
const BIGFADE_SIZE: usize = BIGFADE_WIDTH * BIGFADE_HEIGHT;  // 294,912 bytes

/// Load a 256-color palette from pal0-*.dat file
/// Format: 256 colors × 4 bytes (RGB + padding)
fn load_palette(palette_id: &str) -> Option<Vec<[u8; 4]>> {
    let path = format!("{}/pal0-{}.dat", SKY_DATA_PATH, palette_id);
    let data = std::fs::read(&path).ok()?;

    if data.len() < 1024 {
        eprintln!("Palette file too small: {} bytes", data.len());
        return None;
    }

    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let offset = i * 4;
        palette.push([
            data[offset],      // R
            data[offset + 1],  // G
            data[offset + 2],  // B
            255,               // A (ignore padding byte)
        ]);
    }
    Some(palette)
}

/// Load sky texture from sky0-*.dat file
/// Format: Raw 8-bit palette indices, 512 pixels wide, variable height
fn load_sky_texture(sky_id: &str, palette: &[[u8; 4]]) -> Option<Image> {
    let path = format!("{}/sky0-{}.dat", SKY_DATA_PATH, sky_id);
    let data = std::fs::read(&path).ok()?;

    // Determine height from file size (width is always 512)
    let height = data.len() / SKY_WIDTH;
    if height == 0 || data.len() % SKY_WIDTH != 0 {
        eprintln!("Invalid sky file size: {} bytes", data.len());
        return None;
    }

    println!("Loading sky texture: {}x{}", SKY_WIDTH, height);

    // Convert indexed to RGBA
    let mut pixels = Vec::with_capacity(SKY_WIDTH * height * 4);
    for &index in &data {
        let color = palette[index as usize];
        pixels.extend_from_slice(&color);
    }

    Some(Image::new(
        bevy::render::render_resource::Extent3d {
            width: SKY_WIDTH as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

/// Load BigFade terrain texture from bigf0-*.dat file
/// Format: Raw 8-bit palette indices, 256×1152 pixels
fn load_bigfade_texture(style_id: &str, palette: &[[u8; 4]]) -> Option<Image> {
    use bevy::image::{ImageSampler, ImageSamplerDescriptor, ImageAddressMode, ImageFilterMode};

    let path = format!("{}/bigf0-{}.dat", SKY_DATA_PATH, style_id);
    let data = std::fs::read(&path).ok()?;

    if data.len() != BIGFADE_SIZE {
        eprintln!("BigFade size mismatch: {} bytes (expected {})", data.len(), BIGFADE_SIZE);
        return None;
    }

    println!("Loading BigFade terrain texture: {}x{}", BIGFADE_WIDTH, BIGFADE_HEIGHT);

    // Convert indexed to RGBA using palette
    let mut pixels = Vec::with_capacity(BIGFADE_SIZE * 4);
    for &index in &data {
        let color = palette[index as usize];
        pixels.extend_from_slice(&color);
    }

    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: BIGFADE_WIDTH as u32,
            height: BIGFADE_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    // Set sampler to repeat/tile the texture
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,  // Pixel-art style, no interpolation
        min_filter: ImageFilterMode::Nearest,
        ..default()
    });

    Some(image)
}

// =============================================================================
// SPRITE BANK LOADING (from HSPR0-0.DAT format analysis)
// =============================================================================

/// Path to game data files
const SPRITE_DATA_PATH: &str = "/Users/adriencandiotti/Library/Containers/com.isaacmarovitz.Whisky/Bottles/74820C9D-5F8C-4BFE-B5DB-90E1DE818D3F/drive_c/GOG Games/Populous - The Beginning/data";

/// Follower sprite files: f<anim>t<tribe>-0.dat
/// Tribe numbers: 0-3 for Blue/Red/Yellow/Green, higher numbers for other variants
/// The number after 't' appears to be tribe, after 'f' is animation set
/// f00t6-0.dat has 225 frames with small sprites (6x10)
/// f00t1-0.dat has 224 frames with larger sprites (10x27) - likely walking braves
///
/// For shaman, we'll use HSPR0-0.DAT which has all sprites including shamans
const SHAMAN_IDLE_FRAME: usize = 97;

/// Use HSPR0-0.DAT main sprite bank (contains all humanoid sprites)
const SPRITE_FILE: &str = "HSPR0-0.DAT";

/// Shaman animation: 8 directions × 8 frames = 64 frames total
/// Starting at frame 7578, each direction has 8 consecutive frames
const SHAMAN_ANIM_START: usize = 7578;
const SHAMAN_FRAMES_PER_DIR: usize = 8;
const SHAMAN_NUM_DIRECTIONS: usize = 8;
const SHAMAN_ANIM_SPEED: f32 = 0.08;  // Seconds per frame (faster)

/// Frame to use for static sprite (first frame of first direction)
const SPRITE_FRAME: usize = SHAMAN_ANIM_START;

/// Scan sprite bank and print frames that could be shamans
#[allow(dead_code)]
fn scan_sprite_bank_for_shamans() {
    let path = format!("{}/HSPR0-0.DAT", SPRITE_DATA_PATH);
    let Ok(data) = std::fs::read(&path) else {
        eprintln!("Could not read sprite bank");
        return;
    };

    if data.len() < 8 || &data[0..4] != b"PSFB" {
        eprintln!("Invalid sprite bank");
        return;
    }

    let frame_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    println!("Sprite bank has {} frames", frame_count);

    // Find larger character sprites (shamans might have robes)
    println!("\nLarger human sprites (25-50 wide, 30-55 tall) across ALL frames:");
    for i in 0..frame_count {
        let header_offset = 8 + i * 8;
        let width = u16::from_le_bytes([data[header_offset], data[header_offset + 1]]);
        let height = u16::from_le_bytes([data[header_offset + 2], data[header_offset + 3]]);

        if width >= 25 && width <= 50 && height >= 30 && height <= 55 {
            println!("  Frame {}: {}x{}", i, width, height);
        }
    }

    // Shaman/warrior sized sprites in first 500 frames
    println!("\nMedium sprites (frames 72-500, 16-24 wide, 24-32 tall):");
    for i in 72..frame_count.min(500) {
        let header_offset = 8 + i * 8;
        let width = u16::from_le_bytes([data[header_offset], data[header_offset + 1]]);
        let height = u16::from_le_bytes([data[header_offset + 2], data[header_offset + 3]]);

        if width >= 16 && width <= 24 && height >= 24 && height <= 32 {
            println!("  Frame {}: {}x{}", i, width, height);
        }
    }
}

/// A single sprite frame decoded from the sprite bank
struct SpriteFrame {
    width: u16,
    height: u16,
    pixels: Vec<Vec<u8>>,  // 2D array of palette indices (row-major)
}

/// Decode a single sprite frame from a PSFB sprite bank file
/// Format: PSFB header, then 8-byte frame headers (w, h, offset), then RLE pixel data
fn load_sprite_frame_from_file(file_name: &str, frame_index: usize) -> Option<SpriteFrame> {
    let path = format!("{}/{}", SPRITE_DATA_PATH, file_name);
    let data = std::fs::read(&path).ok()?;

    // Check PSFB magic
    if data.len() < 8 || &data[0..4] != b"PSFB" {
        eprintln!("Invalid sprite bank: missing PSFB header");
        return None;
    }

    // Read frame count
    let frame_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    if frame_index >= frame_count {
        eprintln!("Frame index {} out of range (max {})", frame_index, frame_count);
        return None;
    }

    // Read frame header (8 bytes: width u16, height u16, offset u32)
    let header_offset = 8 + frame_index * 8;
    let width = u16::from_le_bytes([data[header_offset], data[header_offset + 1]]);
    let height = u16::from_le_bytes([data[header_offset + 2], data[header_offset + 3]]);
    let pixel_offset = u32::from_le_bytes([
        data[header_offset + 4],
        data[header_offset + 5],
        data[header_offset + 6],
        data[header_offset + 7],
    ]) as usize;

    // Pixel offset is absolute from file start (per PopSpriteEditor)
    let mut pos = pixel_offset;

    // Decode RLE pixel data - initialize with 255 (transparent color key)
    let mut pixels = vec![vec![255u8; width as usize]; height as usize];

    for row in 0..height as usize {
        let mut col = 0usize;
        while pos < data.len() {
            let val = data[pos] as i8;
            pos += 1;

            if val == 0 {
                // End of row - remaining pixels stay transparent (255)
                break;
            } else if val > 0 {
                // Read N literal pixels
                for _ in 0..val {
                    if col < width as usize && pos < data.len() {
                        pixels[row][col] = data[pos];
                        pos += 1;
                        col += 1;
                    }
                }
            } else {
                // Skip N transparent pixels - already 255 from initialization
                col += (-val) as usize;
            }
        }
    }

    Some(SpriteFrame { width, height, pixels })
}

/// Convert a sprite frame to a Bevy Image using a palette
/// Color keys (transparent): indices 0 and 255 (from PopSpriteEditor Palette.cpp)
fn sprite_frame_to_image(frame: &SpriteFrame, palette: &[[u8; 4]]) -> Image {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let mut rgba = Vec::with_capacity(width * height * 4);

    // Debug: print some palette indices used
    let mut index_counts: std::collections::HashMap<u8, usize> = std::collections::HashMap::new();
    for row in &frame.pixels {
        for &index in row {
            *index_counts.entry(index).or_insert(0) += 1;
        }
    }
    let mut indices: Vec<_> = index_counts.iter().collect();
    indices.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    println!("Top palette indices used: {:?}", &indices[..indices.len().min(10)]);

    for row in &frame.pixels {
        for &index in row {
            if index == 255 {
                // Transparent pixel (color key index 255 only - index 0 is dark brown)
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let color = palette.get(index as usize).unwrap_or(&[255, 0, 255, 255]);
                rgba.extend_from_slice(&[color[0], color[1], color[2], 255]);
            }
        }
    }

    Image::new(
        bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Load the game palette
fn load_game_palette() -> Option<Vec<[u8; 4]>> {
    let palette_path = format!("{}/pal0-0.dat", SPRITE_DATA_PATH);
    let palette_data = std::fs::read(&palette_path).ok()?;

    if palette_data.len() < 1024 {
        eprintln!("Palette file too small");
        return None;
    }

    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let offset = i * 4;
        palette.push([
            palette_data[offset],
            palette_data[offset + 1],
            palette_data[offset + 2],
            255,
        ]);
    }
    Some(palette)
}

/// Load all shaman animation frames (8 directions × 8 frames = 64 total)
fn load_shaman_animation_frames(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<ShamanAnimFrames> {
    let palette = load_game_palette()?;

    let mut all_frames: Vec<Vec<Handle<StandardMaterial>>> = Vec::with_capacity(SHAMAN_NUM_DIRECTIONS);
    let mut frame_width = 0.0f32;
    let mut frame_height = 0.0f32;

    for dir in 0..SHAMAN_NUM_DIRECTIONS {
        let mut dir_frames: Vec<Handle<StandardMaterial>> = Vec::with_capacity(SHAMAN_FRAMES_PER_DIR);

        for frame_idx in 0..SHAMAN_FRAMES_PER_DIR {
            let global_frame = SHAMAN_ANIM_START + dir * SHAMAN_FRAMES_PER_DIR + frame_idx;
            let frame = load_sprite_frame_from_file(SPRITE_FILE, global_frame)?;

            // Store dimensions from first frame
            if dir == 0 && frame_idx == 0 {
                frame_width = frame.width as f32;
                frame_height = frame.height as f32;
                println!("Loaded shaman sprite: {}x{} from {} frame {}",
                         frame.width, frame.height, SPRITE_FILE, global_frame);
            }

            // Convert to image (suppress debug output for subsequent frames)
            let image = sprite_frame_to_image_quiet(&frame, &palette);
            let image_handle = images.add(image);

            let material = materials.add(StandardMaterial {
                base_color_texture: Some(image_handle),
                base_color: Color::WHITE,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,  // Double-sided for billboard
                ..default()
            });

            dir_frames.push(material);
        }

        all_frames.push(dir_frames);
    }

    println!("Loaded {} shaman animation frames ({} directions × {} frames)",
             SHAMAN_NUM_DIRECTIONS * SHAMAN_FRAMES_PER_DIR,
             SHAMAN_NUM_DIRECTIONS, SHAMAN_FRAMES_PER_DIR);

    Some(ShamanAnimFrames {
        frames: all_frames,
        frame_width,
        frame_height,
        loaded: true,
    })
}

/// Convert a sprite frame to a Bevy Image (quiet version without debug output)
fn sprite_frame_to_image_quiet(frame: &SpriteFrame, palette: &[[u8; 4]]) -> Image {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let mut rgba = Vec::with_capacity(width * height * 4);

    for row in &frame.pixels {
        for &index in row {
            if index == 255 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let color = palette.get(index as usize).unwrap_or(&[255, 0, 255, 255]);
                rgba.extend_from_slice(&[color[0], color[1], color[2], 255]);
            }
        }
    }

    Image::new(
        bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Load shaman sprite and create a textured material (single frame, for fallback)
fn load_shaman_sprite(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<(Handle<Mesh>, Handle<StandardMaterial>, f32, f32)> {
    let palette = load_game_palette()?;

    // Load sprite frame from main sprite bank
    let frame = load_sprite_frame_from_file(SPRITE_FILE, SPRITE_FRAME)?;
    println!("Loaded shaman sprite: {}x{} from {} frame {}",
             frame.width, frame.height, SPRITE_FILE, SPRITE_FRAME);

    // Convert to image
    let image = sprite_frame_to_image(&frame, &palette);
    let image_handle = images.add(image);

    // Create material with texture
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        base_color: Color::WHITE,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,  // Double-sided for billboard
        ..default()
    });

    Some((Handle::default(), material, frame.width as f32, frame.height as f32))
}

/// Generate an inverted sphere mesh for sky rendering (viewed from inside)
fn generate_sky_sphere(radius: f32, h_segments: u32, v_segments: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for j in 0..=v_segments {
        let v = j as f32 / v_segments as f32;
        let theta = v * std::f32::consts::PI;  // 0 to PI (top to bottom)

        for i in 0..=h_segments {
            let u = i as f32 / h_segments as f32;
            let phi = u * std::f32::consts::TAU;  // 0 to 2PI (around)

            // Spherical coordinates
            let x = radius * theta.sin() * phi.cos();
            let y = radius * theta.cos();  // Y up
            let z = radius * theta.sin() * phi.sin();

            positions.push([x, y, z]);
            // Normals point inward for inside rendering
            normals.push([-x / radius, -y / radius, -z / radius]);
            uvs.push([u, v]);
        }
    }

    // Generate triangle indices (reversed winding for inside view)
    for j in 0..v_segments {
        for i in 0..h_segments {
            let row = h_segments + 1;
            let a = j * row + i;
            let b = a + 1;
            let c = a + row;
            let d = c + 1;

            // Reversed winding order for inside rendering
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Component to identify the sky sphere entity
#[derive(Component)]
struct SkySphere;

// =============================================================================
// RESOURCES
// =============================================================================

// Original game angle system: 0-2047 steps = 0-360 degrees
// Conversion: angle_radians = angle_steps * (2π / 2048)
const ANGLE_STEPS_TO_RADIANS: f32 = std::f32::consts::TAU / 2048.0;

/// Convert original game angle (0-2047) to radians
fn angle_to_radians(steps: u16) -> f32 {
    (steps & 0x7FF) as f32 * ANGLE_STEPS_TO_RADIANS
}

/// Convert radians to original game angle (0-2047)
fn radians_to_angle(radians: f32) -> u16 {
    ((radians / ANGLE_STEPS_TO_RADIANS) as i32 & 0x7FF) as u16
}

#[derive(Resource)]
struct SphericalCamera {
    focus: Vec3,
    world_rotation: u16,   // Y-axis rotation in original game units (0-2047)
    curvature_scale: i64,  // Can be adjusted at runtime
    distance: f32,         // Distance from focus point
    pitch: u16,            // X-axis rotation in original game units (0-2047)
    dirty: bool,           // Flag to indicate terrain needs regeneration
}

impl Default for SphericalCamera {
    fn default() -> Self {
        // Original game uses 0-2047 angle system (2048 steps = 360 degrees)
        // Pitch of ~239 steps ≈ 42 degrees (closer to original game view)
        Self {
            focus: Vec3::new(WORLD_EXTENT / 2.0, 0.0, WORLD_EXTENT / 2.0),
            world_rotation: 0,
            curvature_scale: CURVATURE_SCALE,
            distance: 6000.0,
            pitch: 239,  // ~42 degrees in original game units
            dirty: true, // Start dirty to generate initial mesh
        }
    }
}

#[derive(Resource)]
struct CurrentLevel {
    number: u32,
    loaded_from_file: bool,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self { number: 1, loaded_from_file: true }
    }
}

#[derive(Resource)]
struct Heightmap {
    heights: Vec<Vec<u16>>,
    cell_flags: Vec<Vec<u32>>,
    /// Spawn positions for each tribe (up to 4 tribes, coordinates in world units)
    spawn_positions: Vec<(f32, f32)>,
}

impl Default for Heightmap {
    fn default() -> Self {
        // Load level 1 by default, fall back to empty terrain if file not found
        Self::load_from_level_file(1).unwrap_or_else(|| {
            eprintln!("Failed to load level 1, using empty terrain");
            Self {
                heights: vec![vec![0u16; WORLD_CELLS + 1]; WORLD_CELLS + 1],
                cell_flags: vec![vec![0u32; WORLD_CELLS]; WORLD_CELLS],
                spawn_positions: vec![],
            }
        })
    }
}

impl Heightmap {
    /// Generate procedural terrain (fallback when no level file loaded)
    fn generate_procedural() -> Self {
        let size = WORLD_CELLS + 1;
        let mut heights = vec![vec![0u16; size]; size];
        let mut cell_flags = vec![vec![0u32; WORLD_CELLS]; WORLD_CELLS];

        // Generate interesting terrain
        for z in 0..size {
            for x in 0..size {
                let fx = x as f32 / WORLD_CELLS as f32;
                let fz = z as f32 / WORLD_CELLS as f32;

                // Central plateau with surrounding valleys
                let center_dist = ((fx - 0.5).powi(2) + (fz - 0.5).powi(2)).sqrt();
                let plateau = (1.0 - (center_dist * 2.5).min(1.0)).powf(0.5) * 500.0;

                // Rolling hills
                let h1 = (fx * std::f32::consts::TAU * 4.0).sin() * 120.0;
                let h2 = (fz * std::f32::consts::TAU * 4.0).sin() * 120.0;
                let h3 = ((fx + fz) * std::f32::consts::TAU * 2.0).cos() * 80.0;

                // Mountain ridges
                let ridge1 = ((fx - 0.3).abs() * 10.0).min(1.0);
                let ridge2 = ((fz - 0.7).abs() * 10.0).min(1.0);
                let mountains = (1.0 - ridge1.min(ridge2)) * 400.0;

                let height = (plateau + h1 + h2 + h3 + mountains + 200.0).max(50.0);
                heights[z][x] = height as u16;
            }
        }

        // Triangle split directions (alternating)
        for z in 0..WORLD_CELLS {
            for x in 0..WORLD_CELLS {
                cell_flags[z][x] = if (x + z) % 2 == 0 { 1 } else { 0 };
            }
        }

        // Print height stats
        let min_height = heights.iter().flatten().min().copied().unwrap_or(0);
        let max_height = heights.iter().flatten().max().copied().unwrap_or(0);
        println!("Procedural terrain: heights range {}-{}", min_height, max_height);

        // Default spawn positions for procedural terrain (4 corners)
        let spawn_positions = vec![
            (WORLD_EXTENT * 0.25, WORLD_EXTENT * 0.25),
            (WORLD_EXTENT * 0.75, WORLD_EXTENT * 0.25),
            (WORLD_EXTENT * 0.75, WORLD_EXTENT * 0.75),
            (WORLD_EXTENT * 0.25, WORLD_EXTENT * 0.75),
        ];

        Self { heights, cell_flags, spawn_positions }
    }

    /// Load heightmap from original game DAT file
    fn load_from_level_file(level_number: u32) -> Option<Self> {
        let path = format!("{}/levl2{:03}.dat", LEVEL_BASE_PATH, level_number);
        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to read level file {}: {}", path, e);
                return None;
            }
        };

        let min_size = CELLFLAGS_FILE_OFFSET + CELLFLAGS_BYTES;
        if data.len() < min_size {
            eprintln!("Level file too small: {} bytes (need at least {})", data.len(), min_size);
            return None;
        }

        // Load heightmap (128×128 u16 little-endian at offset 0)
        let size = WORLD_CELLS + 1;
        let mut heights = vec![vec![0u16; size]; size];

        for z in 0..WORLD_CELLS {
            for x in 0..WORLD_CELLS {
                let offset = HEIGHTMAP_FILE_OFFSET + (z * WORLD_CELLS + x) * 2;
                let height = u16::from_le_bytes([data[offset], data[offset + 1]]);
                heights[z][x] = height;
            }
        }

        // Edge vertices - copy from nearest cell for wrapping
        for z in 0..=WORLD_CELLS {
            heights[z][WORLD_CELLS] = heights[z][0];
        }
        for x in 0..=WORLD_CELLS {
            heights[WORLD_CELLS][x] = heights[0][x];
        }

        // Load cell flags (128×128 u32 little-endian at offset 0x8000)
        let mut cell_flags = vec![vec![0u32; WORLD_CELLS]; WORLD_CELLS];

        for z in 0..WORLD_CELLS {
            for x in 0..WORLD_CELLS {
                let offset = CELLFLAGS_FILE_OFFSET + (z * WORLD_CELLS + x) * 4;
                let flag = u32::from_le_bytes([
                    data[offset], data[offset + 1],
                    data[offset + 2], data[offset + 3]
                ]);
                cell_flags[z][x] = flag;
            }
        }

        // Print some stats
        let min_height = heights.iter().flatten().min().copied().unwrap_or(0);
        let max_height = heights.iter().flatten().max().copied().unwrap_or(0);
        println!("Loaded level {}: heights range {}-{}", level_number, min_height, max_height);

        // Load spawn positions from header file
        let spawn_positions = Self::load_spawn_positions(level_number);
        println!("Loaded {} spawn positions", spawn_positions.len());

        Some(Self { heights, cell_flags, spawn_positions })
    }

    /// Load spawn positions from level .dat file by finding "reincarnation site" objects (type 2)
    /// Each tribe has one reincarnation site which is where shamans spawn
    fn load_spawn_positions(level_number: u32) -> Vec<(f32, f32)> {
        let path = format!("{}/levl2{:03}.dat", LEVEL_BASE_PATH, level_number);
        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to read level file {}: {}", path, e);
                return Self::default_spawn_positions();
            }
        };

        // Object data starts after:
        // - 0x8000 bytes (heightmap)
        // - 0x4000 bytes (flags)
        // - 0x4000 bytes (more data)
        // - 0x4000 bytes (more data)
        // - 0x40 bytes (tribe data)
        // - 3 bytes
        let object_start: usize = 0x14043;
        let entry_size: usize = 0x37;  // 55 bytes per entry
        let entries_per_block: usize = 100;
        let num_blocks: usize = 20;

        // Find reincarnation sites (type 2 objects) for each tribe
        // Object entry format: marker(1), type(1), owner(1), X(2), Z(2), ...
        // Take first type 2 object found for each tribe
        let mut tribes_found = [false; 4];
        let mut spawn_positions: Vec<(u8, f32, f32)> = Vec::new();

        for block in 0..num_blocks {
            let block_offset = object_start + block * 0x157c;
            for entry in 0..entries_per_block {
                let offset = block_offset + entry * entry_size;
                if offset + entry_size > data.len() {
                    break;
                }

                let obj_type = data[offset + 1];
                let owner = data[offset + 2];

                // Skip empty entries
                if obj_type == 0 {
                    continue;
                }

                // Look for reincarnation site (type 2) for each tribe
                // Skip owner 255 (neutral/wildmen) and already found tribes
                if obj_type == 2 && owner < 4 && !tribes_found[owner as usize] {
                    tribes_found[owner as usize] = true;

                    // Coordinates are stored as unsigned 16-bit little-endian
                    let x_raw = u16::from_le_bytes([data[offset + 3], data[offset + 4]]);
                    let z_raw = u16::from_le_bytes([data[offset + 5], data[offset + 6]]);

                    // From PopWorldEditor source code (engine.cpp):
                    // thing->x = (float)((thing->Thing.PosX >> 8) / 2) + 0.5f;
                    // This extracts cell index: high byte >> 8, then / 2, plus 0.5 to center
                    let cell_x = ((x_raw >> 8) / 2) as f32 + 0.5;
                    let cell_z = ((z_raw >> 8) / 2) as f32 + 0.5;

                    // Convert to our world coordinates (CELL_SIZE = 128)
                    let world_x = cell_x * CELL_SIZE;
                    let world_z = cell_z * CELL_SIZE;

                    println!("  Found spawn for tribe {}: raw ({}, {}) -> cell ({:.1}, {:.1}) -> world ({:.1}, {:.1})",
                             owner, x_raw, z_raw, cell_x, cell_z, world_x, world_z);
                    spawn_positions.push((owner, world_x, world_z));
                }
            }
        }

        // Sort by tribe ID and extract just positions
        spawn_positions.sort_by_key(|(owner, _, _)| *owner);
        let positions: Vec<(f32, f32)> = spawn_positions.iter()
            .map(|(_, x, z)| (*x, *z))
            .collect();

        if positions.is_empty() {
            eprintln!("No spawn positions found, using defaults");
            return Self::default_spawn_positions();
        }

        println!("Loaded {} spawn positions", positions.len());
        positions
    }

    fn default_spawn_positions() -> Vec<(f32, f32)> {
        vec![
            (WORLD_EXTENT * 0.25, WORLD_EXTENT * 0.25),
            (WORLD_EXTENT * 0.75, WORLD_EXTENT * 0.25),
            (WORLD_EXTENT * 0.75, WORLD_EXTENT * 0.75),
            (WORLD_EXTENT * 0.25, WORLD_EXTENT * 0.75),
        ]
    }
}

impl Heightmap {
    fn get_height_at(&self, world_x: f32, world_z: f32) -> f32 {
        let grid_x = (world_x / CELL_SIZE).clamp(0.0, (WORLD_CELLS - 1) as f32);
        let grid_z = (world_z / CELL_SIZE).clamp(0.0, (WORLD_CELLS - 1) as f32);

        let x0 = grid_x.floor() as usize;
        let z0 = grid_z.floor() as usize;
        let x1 = (x0 + 1).min(WORLD_CELLS);
        let z1 = (z0 + 1).min(WORLD_CELLS);

        let fx = grid_x.fract();
        let fz = grid_z.fract();

        let h00 = self.heights[z0][x0] as f32;
        let h10 = self.heights[z0][x1] as f32;
        let h01 = self.heights[z1][x0] as f32;
        let h11 = self.heights[z1][x1] as f32;

        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;
        h0 * (1.0 - fz) + h1 * fz
    }
}

// =============================================================================
// TERRAIN MESH GENERATION WITH AUTHENTIC ALGORITHMS
// =============================================================================

/// Calculate wrapped distance (shortest path on torus)
fn wrap_distance(d: f32) -> f32 {
    let half_world = WORLD_EXTENT / 2.0;
    if d > half_world {
        d - WORLD_EXTENT
    } else if d < -half_world {
        d + WORLD_EXTENT
    } else {
        d
    }
}

fn generate_curved_terrain_mesh(
    heightmap: &Heightmap,
    camera: &SphericalCamera,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let grid_size = WORLD_CELLS + 1;

    // Generate vertices with authentic curvature
    for z in 0..grid_size {
        for x in 0..grid_size {
            let world_x = x as f32 * CELL_SIZE;
            let world_z = z as f32 * CELL_SIZE;
            let height = heightmap.heights[z][x] as f32;

            // Step 1: Calculate wrapped distance from camera (toroidal topology)
            // This allows terrain to wrap around the world edges
            let dx = wrap_distance(world_x - camera.focus.x);
            let dz = wrap_distance(world_z - camera.focus.z);

            // Step 2: Apply world rotation around camera focus
            let rotation_rad = angle_to_radians(camera.world_rotation);
            let cos_r = rotation_rad.cos();
            let sin_r = rotation_rad.sin();
            let rotated_dx = dx * cos_r - dz * sin_r;
            let rotated_dz = dx * sin_r + dz * cos_r;

            // Position relative to camera focus (for rendering)
            let rotated_x = rotated_dx + camera.focus.x;
            let rotated_z = rotated_dz + camera.focus.z;

            // Calculate distance from camera focus for curvature and viewport
            let dist_from_focus = (dx * dx + dz * dz).sqrt();

            // Step 3: Apply PopWorldEditor-style spherical curvature
            // From engine.cpp: GroundHeight[ax][az].height -= f * f * SPHERE_RATIO
            // This creates a parabolic depression where distant terrain drops away
            let curved_height = apply_sphere_curvature(height, dist_from_focus);

            positions.push([rotated_x, curved_height, rotated_z]);

            // Calculate lighting based on terrain slope
            let slope_x = if x > 0 && x < WORLD_CELLS {
                (heightmap.heights[z][x + 1] as f32 - heightmap.heights[z][x.saturating_sub(1)] as f32) / (2.0 * CELL_SIZE)
            } else { 0.0 };
            let slope_z = if z > 0 && z < WORLD_CELLS {
                (heightmap.heights[z + 1][x] as f32 - heightmap.heights[z.saturating_sub(1)][x] as f32) / (2.0 * CELL_SIZE)
            } else { 0.0 };

            // Sun from northeast - same direction as original game
            let sun_factor = (-slope_x * 0.7 - slope_z * 0.7 + 0.5).clamp(0.0, 1.0);
            let brightness = 0.4 + sun_factor * 0.6;  // Range 0.4 to 1.0

            // Apply circular viewport fade - terrain beyond VIEWPORT_RADIUS fades out
            let viewport_fade = if dist_from_focus < VIEWPORT_RADIUS * 0.85 {
                1.0
            } else if dist_from_focus < VIEWPORT_RADIUS {
                1.0 - ((dist_from_focus - VIEWPORT_RADIUS * 0.85) / (VIEWPORT_RADIUS * 0.15))
            } else {
                0.0
            };

            // Get terrain color from LANDSCAPE_COLORS palette (from PopWorldEditor)
            // This uses the authentic color gradient from the editor
            let raw_height = heightmap.heights[z][x];
            let (base_r, base_g, base_b) = get_terrain_color(raw_height);

            // Apply brightness and viewport fade
            let final_brightness = brightness * viewport_fade;
            let color = [
                base_r * final_brightness,
                base_g * final_brightness,
                base_b * final_brightness,
                1.0
            ];
            colors.push(color);
            normals.push([0.0, 1.0, 0.0]);  // Will recalculate

            // UV coordinates not used - terrain uses vertex colors
            // The original game pre-generates terrain textures from BigFade + cliff + displacement
            uvs.push([0.0, 0.0]);
        }
    }

    // Recalculate normals from curved positions
    let row_size = grid_size;
    for z in 0..grid_size {
        for x in 0..grid_size {
            let idx = z * row_size + x;
            let pos = Vec3::from_array(positions[idx]);

            let left = if x > 0 { Vec3::from_array(positions[idx - 1]) } else { pos };
            let right = if x < grid_size - 1 { Vec3::from_array(positions[idx + 1]) } else { pos };
            let up = if z > 0 { Vec3::from_array(positions[idx - row_size]) } else { pos };
            let down = if z < grid_size - 1 { Vec3::from_array(positions[idx + row_size]) } else { pos };

            let normal = (right - left).cross(down - up).normalize_or_zero();
            normals[idx] = if normal.y < 0.0 { (-normal).to_array() } else { normal.to_array() };
        }
    }

    // Generate indices with triangle split direction
    // Only include triangles within the circular viewport
    let row_size = grid_size as u32;
    let viewport_radius_sq = VIEWPORT_RADIUS * VIEWPORT_RADIUS;

    for z in 0..WORLD_CELLS {
        for x in 0..WORLD_CELLS {
            // Check if cell center is within viewport radius (with toroidal wrapping)
            let cell_center_x = (x as f32 + 0.5) * CELL_SIZE;
            let cell_center_z = (z as f32 + 0.5) * CELL_SIZE;
            let dx = wrap_distance(cell_center_x - camera.focus.x);
            let dz = wrap_distance(cell_center_z - camera.focus.z);
            let dist_sq = dx * dx + dz * dz;

            // Skip cells outside the viewport circle (with some margin for edge triangles)
            if dist_sq > viewport_radius_sq * 1.1 {
                continue;
            }

            let top_left = (z as u32) * row_size + (x as u32);
            let top_right = top_left + 1;
            let bottom_left = top_left + row_size;
            let bottom_right = bottom_left + 1;

            // Use cell flag bit 0 for split direction (from original)
            let split_dir = heightmap.cell_flags[z][x] & 1;

            if split_dir == 0 {
                indices.extend_from_slice(&[
                    top_left, bottom_left, top_right,
                    bottom_left, bottom_right, top_right,
                ]);
            } else {
                indices.extend_from_slice(&[
                    top_left, bottom_left, bottom_right,
                    top_left, bottom_right, top_right,
                ]);
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// =============================================================================
// COMPONENTS
// =============================================================================

#[derive(Component)]
struct TerrainMesh;

#[derive(Component)]
struct UnitSprite {
    world_x: f32,
    world_z: f32,
    tribe: u8,
    direction: u8,        // 0-7 for 8 directions
    current_frame: u8,    // 0-7 for animation frame
    anim_timer: f32,      // Time accumulator for animation
}

impl UnitSprite {
    fn new(world_x: f32, world_z: f32, tribe: u8) -> Self {
        Self {
            world_x,
            world_z,
            tribe,
            direction: 0,
            current_frame: 0,
            anim_timer: 0.0,
        }
    }
}

/// Resource to hold all shaman animation frames as textures
#[derive(Resource, Default)]
struct ShamanAnimFrames {
    /// frames[direction][frame] = material handle
    frames: Vec<Vec<Handle<StandardMaterial>>>,
    frame_width: f32,
    frame_height: f32,
    loaded: bool,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct InfoText;



// =============================================================================
// SYSTEMS
// =============================================================================

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    heightmap: Res<Heightmap>,
    mut camera: ResMut<SphericalCamera>,
    mut shaman_anim: ResMut<ShamanAnimFrames>,
) {
    // Load palette for textures (used by both terrain and sky)
    let palette = load_palette("0");

    // Load shaman animation frames
    if let Some(anim_frames) = load_shaman_animation_frames(&mut images, &mut materials) {
        *shaman_anim = ShamanAnimFrames {
            frames: anim_frames.frames,
            frame_width: anim_frames.frame_width,
            frame_height: anim_frames.frame_height,
            loaded: true,
        };
    } else {
        eprintln!("Failed to load shaman animation frames");
    }

    // Generate terrain with BigFade texture
    let terrain_mesh = generate_curved_terrain_mesh(&heightmap, &camera);

    // Use vertex colors for terrain (the original game pre-generates terrain textures
    // by combining BigFade, cliff texture, and displacement maps - complex to replicate)
    // Vertex colors already provide height-based coloring similar to the game
    let terrain_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,  // Vertex colors provide the color
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(terrain_mesh)),
        MeshMaterial3d(terrain_material),
        Transform::default(),
        TerrainMesh,
    ));

    // Sky sphere disabled for now - was not rendering correctly
    // TODO: Fix sky rendering later

    // Spawn one unit per tribe at their spawn position (shaman location)
    spawn_units(&mut commands, &mut meshes, &mut materials, &mut images, &heightmap, &shaman_anim);

    // Focus camera on blue shaman (tribe 0) spawn position
    if !heightmap.spawn_positions.is_empty() {
        let (spawn_x, spawn_z) = heightmap.spawn_positions[0];
        camera.focus = Vec3::new(spawn_x, 0.0, spawn_z);
    }

    // Camera - positioned in an orbit around the focus point
    // The original game uses an isometric-style projection looking down at the terrain
    let terrain_height = heightmap.get_height_at(camera.focus.x, camera.focus.z);

    // Camera position: orbit around focus at distance, with pitch angle
    // Higher pitch = more top-down, lower pitch = more side view
    let pitch_rad = angle_to_radians(camera.pitch);
    let camera_offset = Vec3::new(
        0.0,
        camera.distance * pitch_rad.sin(),  // Height based on pitch
        camera.distance * pitch_rad.cos(),  // Horizontal distance based on pitch
    );

    let focus_point = Vec3::new(camera.focus.x, terrain_height, camera.focus.z);
    let camera_pos = focus_point + camera_offset;

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),  // Narrower FOV for more isometric feel
            near: 10.0,
            far: 80000.0,
            aspect_ratio: 16.0 / 9.0,
        }),
        Transform::from_translation(camera_pos)
            .looking_at(focus_point, Vec3::Y),
        MainCamera,
    ));

    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 600.0,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.3, 0.0)),
    ));

    // Info text
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        InfoText,
    ));
}

fn update_camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    time: Res<Time>,
    mut camera: ResMut<SphericalCamera>,
) {
    // Rotation speed in original game angle units (0-2047)
    let rotate_speed = (200.0 * time.delta_secs()) as i16;
    let move_speed = 2000.0 * time.delta_secs();
    let height_speed = 1000.0 * time.delta_secs();
    let curvature_step: i64 = (5000.0 * time.delta_secs()) as i64;

    let old_rotation = camera.world_rotation;
    let old_focus = camera.focus;
    let old_curvature = camera.curvature_scale;

    // Q/E rotates the world (in original game units)
    if keyboard.pressed(KeyCode::KeyQ) {
        camera.world_rotation = (camera.world_rotation as i16 + rotate_speed) as u16 & 0x7FF;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        camera.world_rotation = (camera.world_rotation as i16 - rotate_speed) as u16 & 0x7FF;
    }

    // WASD moves focus - with toroidal wrapping (can go around the sphere)
    let mut move_dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { move_dir.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { move_dir.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }

    if move_dir != Vec2::ZERO {
        // Negate rotation so WASD moves in screen-relative directions
        let rotation_rad = -angle_to_radians(camera.world_rotation);
        let cos_r = rotation_rad.cos();
        let sin_r = rotation_rad.sin();
        let rotated = Vec2::new(
            move_dir.x * cos_r - move_dir.y * sin_r,
            move_dir.x * sin_r + move_dir.y * cos_r,
        ) * move_speed;

        // Toroidal wrapping - wrap around the world like a sphere
        camera.focus.x = (camera.focus.x + rotated.x).rem_euclid(WORLD_EXTENT);
        camera.focus.z = (camera.focus.z + rotated.y).rem_euclid(WORLD_EXTENT);
    }

    // R/F distance (zoom) - doesn't affect terrain mesh, only camera position
    if keyboard.pressed(KeyCode::KeyR) {
        camera.distance = (camera.distance + height_speed).min(15000.0);
    }
    if keyboard.pressed(KeyCode::KeyF) {
        camera.distance = (camera.distance - height_speed).max(1500.0);
    }

    // Z/X pitch (camera tilt angle) - doesn't affect terrain mesh
    let pitch_speed = (100.0 * time.delta_secs()) as i16;
    if keyboard.pressed(KeyCode::KeyZ) {
        camera.pitch = (camera.pitch as i16 + pitch_speed).min(483) as u16;
    }
    if keyboard.pressed(KeyCode::KeyX) {
        camera.pitch = (camera.pitch as i16 - pitch_speed).max(85) as u16;
    }

    // T/G curvature (to compare with original)
    if keyboard.pressed(KeyCode::KeyT) {
        camera.curvature_scale = (camera.curvature_scale + curvature_step).min(150000);
    }
    if keyboard.pressed(KeyCode::KeyG) {
        camera.curvature_scale = (camera.curvature_scale - curvature_step).max(0);
    }

    // Mouse wheel zoom - doesn't affect terrain mesh
    for event in scroll_events.read() {
        camera.distance = (camera.distance - event.y * 300.0).clamp(1500.0, 15000.0);
    }

    // Mark dirty only if parameters affecting terrain mesh changed
    if camera.world_rotation != old_rotation
        || camera.focus != old_focus
        || camera.curvature_scale != old_curvature
    {
        camera.dirty = true;
    }
}

fn update_terrain_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    heightmap: Res<Heightmap>,
    mut camera: ResMut<SphericalCamera>,
    terrain_query: Query<&Mesh3d, With<TerrainMesh>>,
) {
    // Only regenerate mesh if camera parameters changed
    if !camera.dirty {
        return;
    }
    camera.dirty = false;

    for mesh_handle in terrain_query.iter() {
        let new_mesh = generate_curved_terrain_mesh(&heightmap, &camera);
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = new_mesh;
        }
    }
}

fn update_camera_transform(
    camera: Res<SphericalCamera>,
    heightmap: Res<Heightmap>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_query.get_single_mut() else { return };

    let terrain_height = heightmap.get_height_at(camera.focus.x, camera.focus.z);
    let focus_point = Vec3::new(camera.focus.x, terrain_height, camera.focus.z);

    // Camera stays at a fixed offset from focus point (looking from +Z direction)
    // World rotation (Q/E) rotates the terrain, not the camera
    // This creates the effect of the world spinning under a stationary viewpoint
    let pitch_rad = angle_to_radians(camera.pitch);
    let camera_offset = Vec3::new(
        0.0,                                    // No horizontal offset
        camera.distance * pitch_rad.sin(),     // Height based on pitch
        camera.distance * pitch_rad.cos(),     // Distance based on pitch
    );

    let camera_pos = focus_point + camera_offset;

    *transform = Transform::from_translation(camera_pos)
        .looking_at(focus_point, Vec3::Y);
}

fn update_unit_sprites(
    camera: Res<SphericalCamera>,
    heightmap: Res<Heightmap>,
    shaman_anim: Res<ShamanAnimFrames>,
    mut sprites: Query<(&mut Transform, &mut UnitSprite)>,
    camera_query: Query<&Transform, (With<MainCamera>, Without<UnitSprite>)>,
) {
    let Ok(cam_transform) = camera_query.get_single() else { return };

    let scale = 4.0;
    let sprite_height = if shaman_anim.loaded {
        shaman_anim.frame_height * scale
    } else {
        180.0
    };

    for (mut transform, mut unit) in sprites.iter_mut() {
        // Apply toroidal wrapping and world rotation
        let dx = wrap_distance(unit.world_x - camera.focus.x);
        let dz = wrap_distance(unit.world_z - camera.focus.z);
        let rotation_rad = angle_to_radians(camera.world_rotation);
        let cos_r = rotation_rad.cos();
        let sin_r = rotation_rad.sin();
        let rotated_dx = dx * cos_r - dz * sin_r;
        let rotated_dz = dx * sin_r + dz * cos_r;
        let rotated_x = rotated_dx + camera.focus.x;
        let rotated_z = rotated_dz + camera.focus.z;

        let height = heightmap.get_height_at(unit.world_x, unit.world_z);

        // Apply same curvature as terrain uses
        let dist_from_focus = (rotated_dx * rotated_dx + rotated_dz * rotated_dz).sqrt();
        let curved_height = apply_sphere_curvature(height, dist_from_focus);

        // Sprite offset by half height to stand on ground
        transform.translation = Vec3::new(rotated_x, curved_height + sprite_height / 2.0, rotated_z);

        // Calculate direction from camera to unit (on XZ plane)
        // This determines which sprite direction to show
        let to_unit = Vec3::new(
            transform.translation.x - cam_transform.translation.x,
            0.0,
            transform.translation.z - cam_transform.translation.z,
        );

        // Convert angle to 8 directions (0-7)
        // atan2 gives angle in radians, convert to 0-7 direction index
        // Direction 0 = facing camera (south), going clockwise
        let angle = to_unit.x.atan2(to_unit.z);  // -PI to PI
        // Normalize to 0-2PI, then divide into 8 sectors
        let normalized = (angle + std::f32::consts::PI) / std::f32::consts::TAU;  // 0.0 to 1.0
        // Add 0.5/8 offset so direction 0 is centered on "facing south"
        let offset_normalized = (normalized + 0.0625) % 1.0;
        unit.direction = (offset_normalized * 8.0) as u8 % 8;

        // Billboard - sprite always faces camera
        let direction = cam_transform.translation - transform.translation;
        let yaw = direction.x.atan2(direction.z);
        transform.rotation = Quat::from_rotation_y(yaw);
    }
}

/// Update sprite animation frames over time
fn update_sprite_animation(
    time: Res<Time>,
    shaman_anim: Res<ShamanAnimFrames>,
    mut sprites: Query<(&mut UnitSprite, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    if !shaman_anim.loaded {
        return;
    }

    for (mut unit, mut material) in sprites.iter_mut() {
        // Update animation timer
        unit.anim_timer += time.delta_secs();

        // Check if we should advance to next frame
        if unit.anim_timer >= SHAMAN_ANIM_SPEED {
            unit.anim_timer -= SHAMAN_ANIM_SPEED;
            unit.current_frame = (unit.current_frame + 1) % SHAMAN_FRAMES_PER_DIR as u8;

            // Update material to the new frame
            let dir = unit.direction as usize % SHAMAN_NUM_DIRECTIONS;
            let frame = unit.current_frame as usize % SHAMAN_FRAMES_PER_DIR;
            material.0 = shaman_anim.frames[dir][frame].clone();
        }
    }
}

fn update_info_text(
    camera: Res<SphericalCamera>,
    level: Res<CurrentLevel>,
    mut text_query: Query<&mut Text, With<InfoText>>,
) {
    let Ok(mut text) = text_query.get_single_mut() else { return };

    let level_info = format!("Level {}", level.number);

    *text = Text::new(format!(
        "Populous Authentic Rendering Demo\n\
         --------------------------------\n\
         {}\n\
         Curvature: {} (original: {})\n\
         Pitch: {} ({:.1}°) | Distance: {:.0}\n\
         \n\
         Controls:\n\
         1-9, 0 - Load level (0=10)\n\
         WASD   - Move focus\n\
         Q/E    - Rotate world\n\
         R/F    - Camera distance\n\
         Z/X    - Camera pitch\n\
         T/G    - Adjust curvature\n\
         Scroll - Zoom\n\
         F12    - Screenshot",
        level_info,
        camera.curvature_scale, CURVATURE_SCALE,
        camera.pitch, angle_to_radians(camera.pitch).to_degrees(), camera.distance
    ));
}

/// Take a screenshot when F12 is pressed (for debugging)
fn screenshot_on_key(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut counter: Local<u32>,
) {
    if input.just_pressed(KeyCode::F12) {
        let path = format!("./screenshot-{}.png", *counter);
        *counter += 1;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
        println!("Screenshot saved: {}", path);
    }
}

fn handle_level_loading(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut heightmap: ResMut<Heightmap>,
    mut level: ResMut<CurrentLevel>,
    mut camera: ResMut<SphericalCamera>,
    mut windows: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    unit_query: Query<Entity, With<UnitSprite>>,
    shaman_anim: Res<ShamanAnimFrames>,
) {
    // Number keys 1-9 load levels 1-9, 0 loads level 10
    let level_key = if keyboard.just_pressed(KeyCode::Digit1) { Some(1) }
        else if keyboard.just_pressed(KeyCode::Digit2) { Some(2) }
        else if keyboard.just_pressed(KeyCode::Digit3) { Some(3) }
        else if keyboard.just_pressed(KeyCode::Digit4) { Some(4) }
        else if keyboard.just_pressed(KeyCode::Digit5) { Some(5) }
        else if keyboard.just_pressed(KeyCode::Digit6) { Some(6) }
        else if keyboard.just_pressed(KeyCode::Digit7) { Some(7) }
        else if keyboard.just_pressed(KeyCode::Digit8) { Some(8) }
        else if keyboard.just_pressed(KeyCode::Digit9) { Some(9) }
        else if keyboard.just_pressed(KeyCode::Digit0) { Some(10) }
        else { None };

    if let Some(num) = level_key {
        if let Some(loaded) = Heightmap::load_from_level_file(num) {
            // Despawn old units
            for entity in unit_query.iter() {
                commands.entity(entity).despawn();
            }

            // Spawn new units at spawn positions
            spawn_units(&mut commands, &mut meshes, &mut materials, &mut images, &loaded, &shaman_anim);

            // Focus camera on blue shaman (tribe 0) spawn position
            if !loaded.spawn_positions.is_empty() {
                let (spawn_x, spawn_z) = loaded.spawn_positions[0];
                camera.focus = Vec3::new(spawn_x, 0.0, spawn_z);
            }

            *heightmap = loaded;
            level.number = num;
            level.loaded_from_file = true;
            camera.dirty = true;  // Trigger mesh regeneration

            // Update window title
            if let Ok(mut window) = windows.get_single_mut() {
                window.title = format!("Populous - Level {}", num);
            }
        }
    }

}

fn spawn_units(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    heightmap: &Heightmap,
    shaman_anim: &ShamanAnimFrames,
) {
    let scale = 4.0;  // Scale sprite for 3D world

    // Use animation frames if loaded, otherwise fallback
    let (sprite_mesh, sprite_material, sprite_height) = if shaman_anim.loaded {
        let mesh = meshes.add(Rectangle::new(
            shaman_anim.frame_width * scale,
            shaman_anim.frame_height * scale,
        ));
        // Use first frame of first direction as initial material
        let mat = shaman_anim.frames[0][0].clone();
        (mesh, Some(mat), shaman_anim.frame_height * scale)
    } else if let Some((_, mat, w, h)) = load_shaman_sprite(images, materials) {
        let mesh = meshes.add(Rectangle::new(w * scale, h * scale));
        (mesh, Some(mat), h * scale)
    } else {
        eprintln!("Failed to load shaman sprite, using fallback");
        (meshes.add(Rectangle::new(120.0, 180.0)), None, 180.0)
    };

    let tribe_colors = [
        Color::srgb(0.2, 0.5, 1.0),   // Blue (player)
        Color::srgb(1.0, 0.2, 0.2),   // Red
        Color::srgb(1.0, 0.9, 0.2),   // Yellow
        Color::srgb(0.2, 0.9, 0.3),   // Green
    ];

    for (tribe, &(world_x, world_z)) in heightmap.spawn_positions.iter().enumerate() {
        if tribe >= 4 {
            break;
        }
        let height = heightmap.get_height_at(world_x, world_z);

        // Use sprite material if loaded, otherwise fallback to colored material
        let material_handle = if let Some(ref mat) = sprite_material {
            mat.clone()
        } else {
            materials.add(StandardMaterial {
                base_color: tribe_colors[tribe],
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })
        };

        commands.spawn((
            Mesh3d(sprite_mesh.clone()),
            MeshMaterial3d(material_handle),
            Transform::from_xyz(world_x, height + sprite_height / 2.0, world_z),
            UnitSprite::new(world_x, world_z, tribe as u8),
        ));
    }
}

/// Update sky sphere position to follow camera focus
fn update_sky_sphere(
    camera: Res<SphericalCamera>,
    mut sky_query: Query<&mut Transform, With<SkySphere>>,
) {
    for mut transform in &mut sky_query {
        transform.translation = camera.focus;
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    // Debug: uncomment to scan sprite bank for shaman candidates
    // scan_sprite_bank_for_shamans();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Populous Authentic Rendering - Ghidra Constants".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        // Bevy Remote Protocol for MCP debugger
        .add_plugins(RemotePlugin::default())
        .add_plugins(RemoteHttpPlugin::default())
        .init_resource::<Heightmap>()
        .init_resource::<SphericalCamera>()
        .init_resource::<CurrentLevel>()
        .init_resource::<ShamanAnimFrames>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_level_loading,
            update_camera_controls,
            update_terrain_mesh,
            update_camera_transform,
            update_unit_sprites,
            update_sprite_animation,
            update_sky_sphere,
            update_info_text,
            screenshot_on_key,
        ).chain())
        .run();
}

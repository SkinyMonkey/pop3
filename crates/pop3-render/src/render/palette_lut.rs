/*! GPU Look-Up Tables for paletted blend effects.

Translates the original PopTB 8-bit CPU blending systems (ghost tables,
fade tables, remap tables) to GPU textures that fragment shaders sample
instead of doing CPU byte-level lookups.

Original C++ behaviour (8bpp only):
  - Ghost table: ghost[256*src + dst] → blended palette index
  - Invert-glass: ghost[256*dst + src] → blended palette index
  - Fade table: fade[(step<<8) + idx] → faded palette index
  - Remap table: remap[pixel] → remapped palette index

Data files on disk (per level, indexed by hex digit):
  - ghost0-X.dat: 65536 bytes (256×256) — transparency blend table
  - fade0-X.dat:  16384 bytes (64×256)  — 64-step fade table
  - fepalX.dat:   1024 bytes  (256×4)   — level palette

In-game naming: X = level index 0–9 as '0'–'9', 10+ as 'W'+index.
Frontend naming: X = tribe index 0–4.

GPU approach:
  - Ghost/fade tables uploaded as R8Uint textures
  - Palette as R8Uint texture or storage buffer
  - Fragment shader does the same LUT lookups but on the GPU
*/

use crate::render::gpu::texture::GpuTexture;

// ---------------------------------------------------------------------------
// Palette entry (matches C++ TbPaletteEntry layout)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct PaletteEntry {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PaletteEntry {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

// ---------------------------------------------------------------------------
// Palette (256 entries, matches C++ TbPalette)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Palette {
    pub entries: [PaletteEntry; 256],
}

impl Palette {
    pub fn from_rgba_bytes(raw: &[u8]) -> Self {
        let mut entries = [PaletteEntry { r: 0, g: 0, b: 0 }; 256];
        for i in 0..256 {
            let off = i * 4;
            if off + 2 < raw.len() {
                entries[i] = PaletteEntry::new(raw[off], raw[off + 1], raw[off + 2]);
            }
        }
        Self { entries }
    }

    pub fn from_rgb_triples(raw: &[u8]) -> Self {
        let mut entries = [PaletteEntry { r: 0, g: 0, b: 0 }; 256];
        let count = (raw.len() / 3).min(256);
        for i in 0..count {
            entries[i] = PaletteEntry::new(raw[i * 3], raw[i * 3 + 1], raw[i * 3 + 2]);
        }
        Self { entries }
    }
}

// ---------------------------------------------------------------------------
// Nearest-palette-colour search (C++ LbPalette_FindColour, 3-phase)
// ---------------------------------------------------------------------------

/// 3-phase nearest-colour search matching the original C++ algorithm:
///   Phase 1: min squared Euclidean distance (dr²+dg²+db²)
///   Phase 2: min Manhattan distance tiebreaker (|dr|+|dg|+|db|)
///   Phase 3: min weighted brightness tiebreaker (B² + 2G² + 2R²)
pub fn find_colour(palette: &Palette, r: u8, g: u8, b: u8) -> u8 {
    let mut best_dist: u32 = 0x1000000;
    let mut candidates = Vec::new();

    for (i, e) in palette.entries.iter().enumerate() {
        let dr = (r as i32 - e.r as i32).abs() as u32;
        let dg = (g as i32 - e.g as i32).abs() as u32;
        let db = (b as i32 - e.b as i32).abs() as u32;
        let dist = dr * dr + dg * dg + db * db;
        if dist == 0 {
            return i as u8;
        }
        if dist < best_dist {
            best_dist = dist;
            candidates.clear();
            candidates.push(i);
        } else if dist == best_dist {
            candidates.push(i);
        }
    }

    if candidates.len() == 1 {
        return candidates[0] as u8;
    }

    let mut best_man: u32 = u32::MAX;
    let mut man_candidates = Vec::new();
    for &idx in &candidates {
        let e = &palette.entries[idx];
        let dr = (r as i32 - e.r as i32).abs() as u32;
        let dg = (g as i32 - e.g as i32).abs() as u32;
        let db = (b as i32 - e.b as i32).abs() as u32;
        let man = dr + dg + db;
        if man < best_man {
            best_man = man;
            man_candidates.clear();
            man_candidates.push(idx);
        } else if man == best_man {
            man_candidates.push(idx);
        }
    }

    if man_candidates.len() == 1 {
        return man_candidates[0] as u8;
    }

    let mut best_bright: u32 = u32::MAX;
    let mut result: u8 = 0;
    for &idx in &man_candidates {
        let e = &palette.entries[idx];
        let dr = (r as i32 - e.r as i32).abs() as u32;
        let dg = (g as i32 - e.g as i32).abs() as u32;
        let db = (b as i32 - e.b as i32).abs() as u32;
        let bright = db * db + 2 * dg * dg + 2 * dr * dr;
        if bright < best_bright {
            best_bright = bright;
            result = idx as u8;
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Ghost table generation (C++ LbPalette_GenerateGhostTable)
// ---------------------------------------------------------------------------

/// Generate a 256×256 ghost (transparency blend) table.
///
/// Table layout: `table[256 * src_idx + dst_idx]` → blended palette index.
///
/// Blend formula per channel (integer division, matching original):
///   result_ch = (src_ch * percent + dst_ch * (100 - percent)) / 100
///
/// The result is then mapped back to the nearest palette index.
pub fn generate_ghost_table(palette: &Palette, percent: u32) -> [u8; 65536] {
    let mut table = [0u8; 65536];
    let inv = 100 - percent;
    for src in 0..256u32 {
        let se = &palette.entries[src as usize];
        for dst in 0..256u32 {
            let de = &palette.entries[dst as usize];
            let r = ((se.r as u32 * percent + de.r as u32 * inv) / 100) as u8;
            let g = ((se.g as u32 * percent + de.g as u32 * inv) / 100) as u8;
            let b = ((se.b as u32 * percent + de.b as u32 * inv) / 100) as u8;
            table[(src * 256 + dst) as usize] = find_colour(palette, r, g, b);
        }
    }
    table
}

// ---------------------------------------------------------------------------
// Fade table generation
// ---------------------------------------------------------------------------

/// Generate a 64-step fade table (64×256 bytes), matching the original
/// game's `fade0-X.dat` file format (16384 bytes = 64 rows × 256 columns).
///
/// Row `step` (0..64) maps each palette index to its faded version.
/// Step 0 = black, step 63 = original colours.
pub fn generate_fade_table(palette: &Palette) -> [u8; 16384] {
    let mut table = [0u8; 16384];
    for step in 0..64u32 {
        for idx in 0..256u32 {
            let e = &palette.entries[idx as usize];
            let r = ((e.r as u32 * step) / 63) as u8;
            let g = ((e.g as u32 * step) / 63) as u8;
            let b = ((e.b as u32 * step) / 63) as u8;
            table[(step * 256 + idx) as usize] = find_colour(palette, r, g, b);
        }
    }
    table
}

// ---------------------------------------------------------------------------
// GPU LUT resources
// ---------------------------------------------------------------------------

pub const GHOST_TABLE_SIZE: u32 = 256;
pub const FADE_TABLE_HEIGHT: u32 = 64;
pub const FADE_TABLE_WIDTH: u32 = 256;

fn nearest_sampler(device: &wgpu::Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

/// GPU resources for palette-based blend effects.
///
/// Holds the ghost table and fade table as GPU textures that fragment
/// shaders sample to replicate the original 8-bit palette blending
/// pipeline on the GPU.
///
/// Ghost table layout (from `ghost0-X.dat`, 65536 bytes):
///   `table[256 * src_idx + dst_idx]` → blended palette index
///   Uploaded as 256×256 R8Uint texture.
///
/// Fade table layout (from `fade0-X.dat`, 16384 bytes):
///   `table[256 * step + idx]` → faded palette index
///   64 steps (rows), 256 palette indices (columns).
///   Uploaded as 256×64 R8Uint texture.
pub struct PaletteLut {
    /// 256×256 R8Uint ghost/transparency table.
    /// Sample as: `ghost_tex[vec2(src_idx / 256.0, dst_idx / 256.0)]`
    pub ghost_texture: GpuTexture,
    pub ghost_sampler: wgpu::Sampler,

    /// 256×64 R8Uint fade table (64 steps, 256 palette entries per step).
    /// Sample as: `fade_tex[vec2(palette_idx / 256.0, fade_step / 64.0)]`
    pub fade_texture: GpuTexture,
    pub fade_sampler: wgpu::Sampler,

    /// 256×1 R8Uint palette-to-RGB lookup (for indexed sprite rendering).
    /// Not used for RGBA paths — only when rendering from palette-index textures.
    pub _palette_texture: GpuTexture,
}

impl PaletteLut {
    /// Build LUT textures by loading `ghost0-X.dat` and `fade0-X.dat` from
    /// game data, as the original binary does (Palette_InitFromSystemAndFile).
    ///
    /// Falls back to procedural generation if files are missing.
    ///
    /// `ghost_data` = raw bytes from `ghost0-X.dat` (65536 bytes, or empty for fallback).
    /// `fade_data`  = raw bytes from `fade0-X.dat`  (16384 bytes, or empty for fallback).
    /// `palette_rgba` = RGBA palette (1024 bytes, 256 entries × 4 channels).
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ghost_data: &[u8],
        fade_data: &[u8],
        palette_rgba: &[u8],
    ) -> Self {
        let palette = Palette::from_rgba_bytes(palette_rgba);

        let ghost_bytes: Vec<u8> = if ghost_data.len() == 65536 {
            ghost_data.to_vec()
        } else {
            log::warn!(
                "ghost data is {} bytes (expected 65536), generating 50% ghost table",
                ghost_data.len()
            );
            let generated = generate_ghost_table(&palette, 50);
            generated.to_vec()
        };
        let ghost_texture = GpuTexture::new_2d(
            device, queue,
            GHOST_TABLE_SIZE, GHOST_TABLE_SIZE,
            wgpu::TextureFormat::R8Uint,
            &ghost_bytes,
            "ghost_table",
        );
        let ghost_sampler = nearest_sampler(device, "ghost_sampler");

        let fade_bytes: Vec<u8> = if fade_data.len() == 16384 {
            fade_data.to_vec()
        } else {
            log::warn!(
                "fade data is {} bytes (expected 16384), generating procedural fade table",
                fade_data.len()
            );
            let generated = generate_fade_table(&palette);
            generated.to_vec()
        };
        let fade_texture = GpuTexture::new_2d(
            device, queue,
            FADE_TABLE_WIDTH, FADE_TABLE_HEIGHT,
            wgpu::TextureFormat::R8Uint,
            &fade_bytes,
            "fade_table",
        );
        let fade_sampler = nearest_sampler(device, "fade_sampler");

        let palette_index_data: Vec<u8> = (0u8..=255).collect::<Vec<_>>();
        let palette_texture = GpuTexture::new_2d_height1(
            device, queue,
            256,
            wgpu::TextureFormat::R8Uint,
            &palette_index_data,
            "palette_index_map",
        );

        Self {
            ghost_texture,
            ghost_sampler,
            fade_texture,
            fade_sampler,
            _palette_texture: palette_texture,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_palette() -> Palette {
        let mut entries = [PaletteEntry { r: 0, g: 0, b: 0 }; 256];
        entries[0] = PaletteEntry::new(0, 0, 0);
        entries[1] = PaletteEntry::new(255, 0, 0);
        entries[2] = PaletteEntry::new(0, 255, 0);
        entries[3] = PaletteEntry::new(0, 0, 255);
        entries[4] = PaletteEntry::new(255, 255, 255);
        Palette { entries }
    }

    fn greyscale_palette() -> Palette {
        let mut entries = [PaletteEntry { r: 0, g: 0, b: 0 }; 256];
        for i in 0..256 {
            entries[i] = PaletteEntry::new(i as u8, i as u8, i as u8);
        }
        Palette { entries }
    }

    #[test]
    fn find_colour_exact_match() {
        let pal = test_palette();
        assert_eq!(find_colour(&pal, 255, 0, 0), 1);
        assert_eq!(find_colour(&pal, 0, 255, 0), 2);
        assert_eq!(find_colour(&pal, 0, 0, 255), 3);
        assert_eq!(find_colour(&pal, 255, 255, 255), 4);
    }

    #[test]
    fn find_colour_nearest() {
        let pal = test_palette();
        assert_eq!(find_colour(&pal, 200, 0, 0), 1);
        assert_eq!(find_colour(&pal, 0, 200, 0), 2);
    }

    #[test]
    fn ghost_table_dimensions() {
        let pal = test_palette();
        let table = generate_ghost_table(&pal, 50);
        assert_eq!(table.len(), 65536);
    }

    #[test]
    fn ghost_table_glass_lookup() {
        let pal = greyscale_palette();
        let table = generate_ghost_table(&pal, 50);
        let blended = table[256 * 255 + 0];
        assert_eq!(blended, 127);
    }

    #[test]
    fn ghost_table_invert_glass_lookup() {
        let pal = greyscale_palette();
        let table = generate_ghost_table(&pal, 50);
        let blended = table[256 * 0 + 255];
        assert_eq!(blended, 127);
    }

    #[test]
    fn fade_table_dimensions() {
        let pal = test_palette();
        let table = generate_fade_table(&pal);
        assert_eq!(table.len(), 16384);
    }

    #[test]
    fn fade_table_step_63_is_identity() {
        let pal = test_palette();
        let table = generate_fade_table(&pal);
        for idx in 0..4u32 {
            let faded = table[63 * 256 + idx as usize];
            let e_orig = &pal.entries[idx as usize];
            let e_faded = &pal.entries[faded as usize];
            assert!(
                (e_orig.r as i32 - e_faded.r as i32).abs() <= 1,
                "idx={} r mismatch: {} vs {}",
                idx, e_orig.r, e_faded.r
            );
        }
    }

    #[test]
    fn fade_table_step_0_is_black() {
        let pal = test_palette();
        let table = generate_fade_table(&pal);
        for idx in 0..4u32 {
            let faded = table[0 * 256 + idx as usize];
            let e = &pal.entries[faded as usize];
            assert_eq!(e.r, 0, "step 0 idx {} should fade to black (r)", idx);
            assert_eq!(e.g, 0, "step 0 idx {} should fade to black (g)", idx);
            assert_eq!(e.b, 0, "step 0 idx {} should fade to black (b)", idx);
        }
    }
}
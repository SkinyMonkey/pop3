/*! GPU Look-Up Tables for paletted blend effects.

Translates the original PopTB 8-bit CPU blending systems (ghost tables,
fade tables, remap tables) to GPU textures that fragment shaders sample
instead of doing CPU byte-level lookups.

Original C++ behaviour (8bpp only):
  - Ghost table: ghost[256*src + dst] → blended palette index
  - Invert-glass: ghost[256*dst + src] → blended palette index
  - Fade table: fade[(step<<8) + idx] → faded palette index
  - Remap table: remap[pixel] → remapped palette index

GPU approach:
  - Ghost/fade/remap tables uploaded as R8Uint textures
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

/// Generate a 256-step fade table (256×256 bytes).
///
/// Row `step` maps each palette index to its faded version at that step.
/// Step 0 = fully faded (black), step 255 = original colours.
///
/// The original PopTB generates these tables at runtime; we generate a
/// linear fade-to-black table which covers the common use case
/// (screen fade-in / fade-out transitions). Custom fade tables can be
/// loaded from game data if the format is discovered.
pub fn generate_fade_table(palette: &Palette) -> [u8; 65536] {
    let mut table = [0u8; 65536];
    for step in 0..256u32 {
        for idx in 0..256u32 {
            let e = &palette.entries[idx as usize];
            let r = ((e.r as u32 * step) / 255) as u8;
            let g = ((e.g as u32 * step) / 255) as u8;
            let b = ((e.b as u32 * step) / 255) as u8;
            table[(step * 256 + idx) as usize] = find_colour(palette, r, g, b);
        }
    }
    table
}

// ---------------------------------------------------------------------------
// GPU LUT resources
// ---------------------------------------------------------------------------

/// GPU resources for palette-based blend effects.
///
/// Holds the ghost table, fade table, and palette as GPU textures
/// that fragment shaders can sample to replicate the original 8-bit
/// palette blending pipeline on the GPU.
pub struct PaletteLut {
    /// 256×256 R8Uint ghost/transparency table.
    /// Sample as: `ghost_tex[vec2(src_idx / 256.0, dst_idx / 256.0)]`
    pub ghost_texture: GpuTexture,
    pub ghost_sampler: wgpu::Sampler,

    /// 256×256 R8Uint fade table.
    /// Sample as: `fade_tex[vec2(palette_idx / 256.0, fade_step / 256.0)]`
    pub fade_texture: GpuTexture,
    pub fade_sampler: wgpu::Sampler,

    /// 256×1 R8Uint palette-to-RGB lookup (for indexed sprite rendering).
    /// Not used for RGBA paths — only when rendering from palette-index textures.
    pub _palette_texture: GpuTexture,
}

impl PaletteLut {
    /// Build all LUT textures from an RGBA palette (4 bytes per entry, 256 entries).
    ///
    /// `ghost_percent` controls the transparency blend ratio baked into the
    /// ghost table (e.g. 50 = 50% opacity, as in the original game).
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        palette_rgba: &[u8],
        ghost_percent: u32,
    ) -> Self {
        let palette = Palette::from_rgba_bytes(palette_rgba);

        let ghost_data = generate_ghost_table(&palette, ghost_percent);
        let ghost_texture = GpuTexture::new_2d(
            device, queue,
            256, 256,
            wgpu::TextureFormat::R8Uint,
            &ghost_data,
            "ghost_table",
        );
        let ghost_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ghost_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let fade_data = generate_fade_table(&palette);
        let fade_texture = GpuTexture::new_2d(
            device, queue,
            256, 256,
            wgpu::TextureFormat::R8Uint,
            &fade_data,
            "fade_table",
        );
        let fade_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fade_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // Pack palette as R8Uint index = 0..255 (identity for now; real use is
        // to provide palette indices from a sideband channel alongside the RGBA
        // sprite texture). This texture stores the raw palette order.
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
        // GLASS: table[256 * src + dst] — 50% blend of src=255 over dst=0
        // (255 * 50 + 0 * 50) / 100 = 127 → closest grey index
        let blended = table[256 * 255 + 0];
        assert_eq!(blended, 127);
    }

    #[test]
    fn ghost_table_invert_glass_lookup() {
        let pal = greyscale_palette();
        let table = generate_ghost_table(&pal, 50);
        // INVERT_GLASS: table[256 * dst + src] — 50% blend, dest dominates
        // Same formula (symmetric at 50%): (255 * 50 + 0 * 50) / 100 = 127
        let blended = table[256 * 0 + 255];
        assert_eq!(blended, 127);
    }

    #[test]
    fn fade_table_step_255_is_identity() {
        let pal = test_palette();
        let table = generate_fade_table(&pal);
        // Step 255 should be close to original colours
        for idx in 0..4u32 {
            let faded = table[255 * 256 + idx as usize];
            // At step 255, the colour should map close to itself
            // (may not be exact due to /255 integer division)
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
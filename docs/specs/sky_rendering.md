# Sky Rendering System

## Overview

The sky in Populous: The Beginning is rendered as a **textured hemisphere** projected
onto the framebuffer before terrain drawing. The system uses a 512×512 paletted
texture sampled through a pre-computed lens distortion table (`skylens.dat`), with
camera yaw influencing sky alignment/parallax. The entire sky fills the screen each
frame; terrain is then drawn on top, overdrawing sky pixels below the horizon.

The sky renderer supports **4 modes** selected at runtime based on hardware flags
and screen resolution. Mode 0 (tiled perspective) is the primary renderer used on
most configurations.

## Data Files

### sky0-{c}.dat — Sky Texture (two formats)

Files named `sky0-{c}.dat` exist in **two formats** distinguished by file size:

**Format A — 640×480 (sky0-0.dat only)**

| Property | Value |
|----------|-------|
| Size | 307,200 bytes (640 × 480) |
| Format | Raw 8-bit **absolute** palette indices |
| Value range | 100–127 (0x64–0x7F), 28 unique values |
| Palette offset | None — indices are direct palette lookups |

Used by **Mode 1** (simple scanline renderer) for direct 640×480 framebuffer fill.
Only `sky0-0.dat` uses this format.

**Format B — 512×512 (sky0-1.dat through sky0-z.dat)**

| Property | Value |
|----------|-------|
| Size | 262,144 bytes (512 × 512) |
| Format | Raw 8-bit **relative** indices |
| Value range | 1–14 (relative indices into sky sub-palette) |
| Palette offset | **+0x70** → palette entries 0x71–0x7E |

Same format as MSKY files. Used by **Mode 0** (tiled perspective renderer). The
values are relative indices into the sky sub-palette at palette positions 0x70–0x7F,
mapped by `Sky_BuildPaletteLUT`.

### MSKY0-{c}.DAT — Sky Texture (512×512)

| Property | Value |
|----------|-------|
| Size | 262,144 bytes (512 × 512) |
| Format | Raw 8-bit relative indices (same encoding as sky0 Format B) |
| Value range | 1–14 in observed data (0 = transparent/background, 15 unused) |
| Variants | 0–9, a, b (per level theme) |
| Referenced via | `DAT_00599ed4` (sky texture pointer) |

Same format as sky0 Format B. Used by **Mode 0** (tiled perspective renderer).

### skylens.dat — Hemisphere Lens Distortion Table

| Property | Value |
|----------|-------|
| Size | 16,848 bytes (0x41D0) |
| Format | 2,106 pairs of `i32` (little-endian) |
| Structure | 26 groups × 81 entries × 8 bytes |
| Loaded to | `0x007f9880` |
| Flag | `DAT_00599ed8` (set to 1 after load) |

**Structure detail:** Each entry is an `(i32, i32)` pair representing
`(u_distortion, v_distortion)` in fixed-point. Within each group, the first
element sweeps symmetrically from `+N` to `-N` (81 entries), representing the
horizontal lens correction across the viewport. The 26 groups represent vertical
rows, with distortion magnitude increasing for rows further from the horizon
(top of screen = more distortion).

| Group | First entry | Last entry | V-distortion |
|-------|------------|------------|--------------|
| 0 (horizon) | +20,720 | -20,720 | -34,248 |
| 12 (mid) | +31,483 | -31,483 | -53,468 |
| 25 (zenith) | +72,000 | -72,000 | -125,820 |

## Global Variables

### Sky State

| Address | Name | Type | Description |
|---------|------|------|-------------|
| 0x00599ed4 | g_sky_texture_ptr | `u8*` | Pointer to loaded MSKY texture (512×512) |
| 0x00599ed8 | g_skylens_loaded | `i32` | Flag: 1 if skylens.dat has been loaded |
| 0x00599ed0 | g_sky_palette_extra | `i32` | External palette offset for parallax mode |
| 0x00599ecc | g_sky_use_palette_remap | `char` | If non-zero, use palette remapping in tile renderer |
| 0x00599820 | g_camera_yaw | `i32` | Camera yaw rotation (read by `Camera_GetYawRotation`) |
| 0x00865134 | g_sky_render_mode | `i32` | 0=tiled, 1=simple, 2=parallax, 3=flat fill |
| 0x0087e33d | g_render_flags | `u8` | Bit 7: if set, mode=0 (tiled); if clear, mode=2 (parallax) |

### Sky Render Context (struct at 0x8646c8, size ≥ 0x834)

Passed as `this` pointer to all sky render functions.

| Offset | Type | Name | Description |
|--------|------|------|-------------|
| +0x00 | `u8*` | texture_ptr | Sky texture data (from `g_sky_texture_ptr`) |
| +0x04 | `u8*` | framebuffer | Target pixel buffer |
| +0x08 | `i32` | max_x | Screen width |
| +0x0C | `i32` | max_y | Screen height |
| +0x10 | `i32` | stride | Bytes per row in framebuffer |
| +0x14 | `i32` | clip_left | Left clipping boundary |
| +0x18 | `i32` | clip_top | Top clipping boundary |
| +0x1C | `i32` | clip_right | Right clipping boundary |
| +0x20 | `i32` | clip_bottom | Bottom clipping boundary |
| +0x24 | `i32` | tile_col_start | `clip_left >> 4` |
| +0x28 | `i32` | tile_row_start | `clip_top >> 4` |
| +0x2C | `i32` | tile_col_end | `(clip_right + 15) >> 4` |
| +0x30 | `i32` | tile_row_end | `(clip_bottom + 15) >> 4` |
| +0x34 | `i32[0x200]` | tile_uv_top | UV coordinates for current tile row (top edge) |
| +0x834 | `i32[0x200]` | tile_uv_bottom | UV coordinates for next tile row (bottom edge) |

### Sky Rotation State (struct at 0x8646b0, size 0x18)

Tracks cumulative camera-driven sky rotation.

| Offset | Type | Name | Description |
|--------|------|------|-------------|
| +0x00 | `i32` | target_yaw | Last camera yaw value |
| +0x04 | `i32` | target_u | Last camera U offset |
| +0x08 | `i32` | target_v | Last camera V offset |
| +0x0C | `u32` | angle | Cumulative rotation angle (& 0x7FF for 2048-entry trig table) |
| +0x10 | `i32` | u_accumulator | Accumulated U panning |
| +0x14 | `i32` | v_accumulator | Accumulated V panning |

### Sky Render Params (struct at 0x865138, size 0x14)

Derived from rotation state, consumed by tile renderer.

| Offset | Type | Name | Description |
|--------|------|------|-------------|
| +0x00 | `i32` | u_origin | `u_accumulator << 9` |
| +0x04 | `i32` | v_origin | `v_accumulator << 9` |
| +0x08 | `i32` | angle | Copy of rotation angle |
| +0x0C | `i32` | cos_angle | `g_cos_table_2048[angle]` |
| +0x10 | `i32` | sin_angle | `g_sin_table_2048[angle]` |

### Parallax Scaling Globals

| Address | Type | Name | Description |
|---------|------|------|-------------|
| 0x0086514C | `i32` | screen_width_copy | Copy of screen width |
| 0x00865150 | `i32` | screen_height_copy | Copy of screen height |
| 0x00865154 | `i32` | sky_center_x | `camera_yaw + (screen_width - camera_yaw) / 2` |
| 0x00865158 | `i32` | sky_center_y | Always 0 |
| 0x0086515C | `i32` | u_scale | `0x2800000 / visible_width` |
| 0x00865160 | `i32` | v_scale | `0xC80000 / visible_height` |
| 0x00865164 | `i32` | combined_scale | Fixed-point `0xC8 * 0x15999 >> 16` |

### Palette/Dither Data

| Address | Type | Name | Description |
|---------|------|------|-------------|
| 0x0056d2f0 | `u8[16]` | g_dither_matrix | 4×4 ordered dither pattern: `00 20 08 28 30 10 38 18 0C 2C 04 24 3C 1C 34 14` |

## Functions

### Call Graph

```
UI_ClearScreenBuffer (0x00494280)
  ├─ [loads skylens.dat if not loaded]
  └─ Sky_RenderOrchestrator (0x004dc0e0)
       ├─ Camera_GetYawRotation (0x0045ae50)
       ├─ Sky_SetViewport (0x004dc890)
       ├─ Sky_UpdateRotation (0x004dc710)
       ├─ Sky_ComputeParams (0x004dc850)
       ├─ Sky_BuildPaletteLUT (0x004dc3f0)
       └─ [dispatch by g_sky_render_mode]
            ├─ mode 0: Sky_RenderTiled (0x004dcc30)
            │           └─ Sky_RasterizeScanline (0x004dc930)
            ├─ mode 1: Sky_RenderSimple (0x004dd710)
            ├─ mode 2: Sky_RenderParallax (0x004dd790)
            └─ mode 3: Sky_RenderFlatFill (0x004dd880)
```

### UI_ClearScreenBuffer @ 0x00494280

Entry point called before each frame's terrain render. Dispatches based on a
mode parameter:

- **param == 0**: Render sky texture. Loads `skylens.dat` on first call (into
  `0x007f9880`, 0x41D0 bytes). Then calls `Sky_RenderOrchestrator`.
- **param == 1 or 2**: Fill framebuffer with a solid color (palette index from
  `DAT_008853ca`, replicated across 4 bytes for fast 32-bit writes).

After sky/clear, optionally overlays a sprite element (weather, lens flare)
via `Sprite_SetupScanline8bpp` / `Sprite_SetupScanline16bpp`.

### Sky_RenderOrchestrator @ 0x004dc0e0

Main sky rendering coordinator. Sets up all state and dispatches to a mode-specific
renderer.

**Sequence:**

1. **Mode selection**: Reads `g_render_flags` bit 7. If set, `g_sky_render_mode = 0`
   (tiled). If clear, `g_sky_render_mode = 2` (parallax).

2. **Resolution check**: If screen is exactly 512×384 and a specific flag is set,
   uses a special framebuffer offset (`screen_ptr + stride * 3 * 16 + 0x40`);
   otherwise uses the base framebuffer pointer directly.

3. **Camera yaw**: Calls `Camera_GetYawRotation()` → returns `DAT_00599820`.
   Masks result with `& 0xFFFFFFFC` (align to 4). This yaw drives the sky parallax.

4. **Viewport setup**: Calls `Sky_SetViewport` to clip the rendering region and
   compute tile grid boundaries.

5. **Parallax scale computation**:
   - `u_scale = 0x2800000 / visible_width`
   - `v_scale = 0xC80000 / visible_height`
   - `combined_scale = (0xC8 * 0x15999) >> 16`

6. **Texture pointer**: Copies `g_sky_texture_ptr` to render context.

7. **Camera rotation feed**: Reads per-level values from spell data table
   `0x885784` and passes them to `Sky_UpdateRotation`. These values are constant
   per level; after first-frame target initialization, frame-to-frame deltas are
   typically zero, so rotation accumulators usually remain unchanged.

8. **Derive params**: Calls `Sky_ComputeParams` to convert rotation state into
   the UV origin + sin/cos values used by the tile renderer.

9. **Build palette LUT**: Calls `Sky_BuildPaletteLUT` with the current palette
   pointer to create a luminance-sorted 256-entry mapping for the parallax
   fade renderer.

10. **Dispatch**: Based on `g_sky_render_mode`:
    - 0 → `Sky_RenderTiled`
    - 1 → `Sky_RenderSimple`
    - 2 → `Sky_RenderParallax`
    - 3 → `Sky_RenderFlatFill`

### Sky_SetViewport @ 0x004dc890

Clips two corner coordinates to the render context bounds and computes tile-grid
boundaries.

**Input**: `(x1, y1, x2, y2)` — viewport corners.

**Output** (stored in render context):
- `clip_left`, `clip_top`, `clip_right`, `clip_bottom` — pixel bounds clamped to `[0, max]`
- `tile_col_start = clip_left >> 4`
- `tile_row_start = clip_top >> 4`
- `tile_col_end = (clip_right + 15) >> 4`
- `tile_row_end = (clip_bottom + 15) >> 4`

### Sky_UpdateRotation @ 0x004dc710

Updates the sky rotation state based on camera movement delta. Uses wrap-around
arithmetic for toroidal panning.

**Algorithm:**
1. Compute deltas from previous target: `dx = new_yaw - old_yaw`, `du = new_u - old_u`,
   `dv = new_v - old_v`
2. Wrap deltas to prevent jumps: yaw wraps at ±0x400 (±1024), U/V wrap at ±0x8000
3. Rotate the U/V deltas using sin/cos lookup at `lookup = (angle - new_yaw) & 0x7FF`:
   - `u_acc += du * cos(lookup) + dv * sin(lookup)`
   - `v_acc += dv * cos(lookup) - du * sin(lookup)`
   (64-bit multiply, result >> 16 for fixed-point)
4. Update angle: `angle = (angle + dx/2) & 0x7FF`

### Sky_ComputeParams @ 0x004dc850

Converts rotation state into render params:
- `u_origin = u_accumulator << 9`
- `v_origin = v_accumulator << 9`
- `angle = rotation.angle`
- `cos_angle = g_cos_table_2048[angle]`
- `sin_angle = g_sin_table_2048[angle]`

### Sky_BuildPaletteLUT @ 0x004dc3f0

Builds a 256-entry palette lookup table for the parallax renderer by sorting the
16 sky sub-palette entries (indices 0x70–0x7F) by luminance.

**Algorithm:**
1. If `palette_ptr == 0`: default mapping (indices 0x70+i for positions 0..15).
2. Otherwise:
   - Compute luminance for each palette entry 0x70–0x7E (14 entries) using
     weighted RGB: `lum = R * 0x42 + G * 0x81 + B * 0x19` (roughly BT.601 Y)
   - Sort by luminance (darkest first)
   - Store sorted palette indices at `+0x18` (14 entries + 0x70 and 0x7F at ends)
   - Create inverse lookup at `+0x28`
   - Compute per-entry luminance values at `+0x08`
   - Build a 256-entry quantization table at `+0x38`: maps any 0–255 brightness
     to the nearest palette entry in the sorted order

### Sky_RenderTiled (Mode 0) @ 0x004dcc30

**Primary sky renderer.** Renders the sky using a 16×16 tile grid with perspective-correct
texture mapping. This is the most complex and visually accurate mode.

**Algorithm:**

For each tile in the grid (iterated row-major):

1. **Perspective projection**: For each tile corner `(col * 16, row * 16)`:
   - Compute view-space offset: `dx = col*16 - param.u_center`, `dy = row*16 - param.v_center`
   - Scale by lens factors: `sx = dx * param.u_scale >> 16`, `sy = dy * param.v_scale >> 16`
   - Apply hemisphere projection: `depth = param.horizon - (sx² >> 12)`
   - Compute perspective divisor: `perspective = horizon << 16 / (horizon - depth)`
     (clamped to 0x10000 when depth == horizon)
   - Rotate UV by camera angle:
     ```
     half_persp = sx * perspective >> 1
     u = origin_u + (half_persp * cos + (perspective << 8) * sin) >> 16
     v = origin_v - (half_persp * sin) + ((perspective << 8) * cos) >> 16
     ```

2. **UV delta computation**: For each tile, compute incremental UV deltas
   between adjacent corners (divided by 16 for per-pixel stepping).

3. **Vertical interpolation**: Between top and bottom tile rows, interpolate
   UV values linearly across 16 scanlines.

4. **Pixel write**: For each pixel in the 16×16 tile:
   - Extract texture coordinate: `tex_offset = (v & 0x3FE00) + ((u & 0x3FE00) >> 9)`
   - This masks bits 9–17 of the fixed-point UV, giving a 0–511 integer index
   - Read palette index from `texture[tex_offset]`
   - Write to framebuffer (packed 4 bytes at a time for performance)

5. **Palette remapping** (optional): If `g_sky_use_palette_remap` is set, pixels
   are run through `Sky_RasterizeScanline` which applies an additional palette
   indirection via `g_sky_palette_extra + 0x957078`.

**Fixed-point format**: UV coordinates use approximately 16.16 fixed-point.
The mask `0x3FE00` extracts bits 9–17, giving 9 bits of integer (0–511) for the
512×512 texture.

### Sky_RenderSimple (Mode 1) @ 0x004dd710

Fast fallback for exactly 512×384 resolution. Directly copies from the sky texture
without perspective correction.

**Algorithm:** For each scanline `y` in `[clip_top, clip_bottom)`:
- For each pixel `x` in `[clip_right-1, clip_left]` (right to left):
  - `tex_u = (origin_u >> 16) + angle * 2 + x`
  - `tex_v = (origin_v >> 18) + y * 2`
  - `pixel = texture[(tex_u & 0x1FF) + (tex_v & 0x1FF) * 512]`
  - Write pixel to `framebuffer[y * stride + x]`

`angle` is loaded from render params offset `+0x08` (raw rotation angle), not from
`v_origin`.

In original runtime behavior, `origin_u` and `angle` often remain 0 (constant
spell inputs, no accumulator drift), so Mode 1 effectively samples
`tex_u = x & 0x1FF`, `tex_v = y * 2` (static sky).

### Sky_RenderParallax (Mode 2) @ 0x004dd790

Renders a gradient sky using the palette LUT for smooth vertical fading.
Uses the dither matrix and palette luminance curve for banding reduction.

**Algorithm:** For each scanline `y` in `[clip_top, clip_bottom)`:
1. Compute vertical progress: `t = (y - clip_top) * 0xF00000 / total_height >> 16`
2. Apply 4×4 dither: `brightness = t + dither_matrix[(y & 3) * 4 + (iteration & 3)]`
3. Clamp to 255
4. Look up palette entry: `color = palette_lut[brightness + 0x38]`
5. If palette remapping active: `color = remap_table[color]`
6. Fill entire scanline with this color (replicated as 4-byte `i32` writes)

### Sky_RenderFlatFill (Mode 3) @ 0x004dd880

Debug/fallback mode. Fills the sky region with solid palette index `0x7E` (126).

**Algorithm:** For each scanline, memset the visible pixel range with `0x7E7E7E7E`.

### Sky_RasterizeScanline @ 0x004dc930

Per-scanline texture sampler with palette remapping, used by Mode 0 when
`g_sky_use_palette_remap` is set.

**Algorithm:** For each of 16 pixels per tile column:
- Sample texture at `(u, v)` using the `0x3FE00` mask (same as Mode 0)
- Remap through `palette_extra + 0x957078` indirection table
- Write pixel to framebuffer
- Advance UV by per-pixel deltas

## Rendering Pipeline Integration

```
Per frame:
  1. DDraw_FlipAndClear
       └─ UI_ClearScreenBuffer(mode=0)
            └─ Sky_RenderOrchestrator
                 └─ Sky_RenderTiled (fills entire framebuffer with sky)

  2. Terrain_RenderOrchestrator
       └─ [draws terrain on top, overwriting sky below horizon]

  3. Game_RenderWorld
       └─ [sprites, 3D models, effects, UI]
```

The sky is always rendered to the **full screen** first. Terrain rendering then
overwrites all pixels below the horizon line. This means sky is only visible in
the upper portion of the screen (above the terrain horizon), but the full-screen
render avoids needing to track the exact horizon boundary.

## Key Constants

| Value | Meaning |
|-------|---------|
| `0x2800000` | U-scale numerator (41,943,040) |
| `0xC80000` | V-scale numerator (13,107,200) |
| `0x15999` | Combined scale factor |
| `0x3FE00` | UV mask for 512×512 texture (bits 9–17, 9-bit integer) |
| `0x7FF` | Angle mask for 2048-entry trig table |
| `0x7E` | Flat fill palette index (126) |
| `16` | Tile size in pixels |
| `0x41D0` | Skylens data size (16,848 bytes) |

## Current Rust Implementation Status

Current sky work is centered on the standalone dome demo:

- **`shaders/sky_dome.wgsl`** — 4-mode shader approximation (dome/simple/parallax/flat).
- **`src/bin/sky_dome_demo.rs` (`SkyCamera`)** — rotation state with
  `update_rotation()` and `to_params()`.
- **`src/bin/sky_dome_demo.rs` (`SkyDomeParams`)** — 32-byte GPU uniform matching
  shader layout.
- **`src/bin/sky_dome_demo.rs`** — standalone viewer with mode switching and
  theme cycling.
- **Format auto-detection by file size** — `307200` → 640×480 (absolute palette
  indices), `262144` → 512×512 (relative indices + `0x70` offset).
- **Continuous drift + yaw-driven panning** are enabled for visual motion
  (enhancement relative to original behavior).

### Not yet implemented

- `skylens.dat` integration for true Mode 0 vertical distortion
- Original viewport-center Mode 0 parallax (`sky_center_x`-driven center shift)
- Viewport-computed fixed-point scales (`0x2800000/visible_width`,
  `0xC80000/visible_height`) instead of hardcoded demo scales

## Implementation Notes

These notes document intentional differences between the current demo and
original game behavior:

1. **Sky panning is an enhancement**:
   Original per-level spell values are constant; after first-frame initialization,
   `u_accumulator`, `v_accumulator`, and `angle` usually stay at 0.
   The demo intentionally adds continuous drift (`rate = 0.003`) and yaw-driven
   panning for visual motion.
2. **Mode 0 parallax mechanism differs**:
   Original Mode 0 parallax primarily comes from shifting projection center
   (`sky_center_x`) from camera yaw. The demo pans UVs via `SkyCamera.u_acc`.
3. **Dome projection is an approximation**:
   Original uses horizontal depth term plus `skylens.dat` vertical distortion.
   The demo uses radial `sx² + sy²` hemisphere curvature.
4. **Scale values are currently hardcoded**:
   Demo uses `u_scale = 2.5`, `v_scale = 1.8`, `horizon = 1.5` rather than
   viewport-derived fixed-point values from original code.

## Renamed Functions

| Address | Old Name | New Name |
|---------|----------|----------|
| 0x004dc0e0 | FUN_004dc0e0 | Sky_RenderOrchestrator |
| 0x004dcc30 | FUN_004dcc30 | Sky_RenderTiled |
| 0x004dd710 | FUN_004dd710 | Sky_RenderSimple |
| 0x004dd790 | FUN_004dd790 | Sky_RenderParallax |
| 0x004dd880 | FUN_004dd880 | Sky_RenderFlatFill |
| 0x004dc890 | FUN_004dc890 | Sky_SetViewport |
| 0x004dc710 | FUN_004dc710 | Sky_UpdateRotation |
| 0x004dc850 | FUN_004dc850 | Sky_ComputeParams |
| 0x004dc3f0 | FUN_004dc3f0 | Sky_BuildPaletteLUT |
| 0x004dc930 | FUN_004dc930 | Sky_RasterizeScanline |
| 0x0045ae50 | FUN_0045ae50 | Camera_GetYawRotation |

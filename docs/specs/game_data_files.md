# Populous: The Beginning — Game Data File Reference

This document describes all game data files used by Pop3, their binary formats, and how they are parsed/decrypted.

## Level Files

### Level Header (`levels/levl2{NNN}.hdr`)

Binary header for each level. At least 70 bytes.

| Offset | Type | Description |
|--------|------|-------------|
| 96 | u8 | Landscape type (0-9 = `'0'`-`'9'`, 10-35 = `'a'`-`'z'`) |

**Parser**: `read_landscape_type()` in `src/pop/level.rs`

The landscape type determines which theme-keyed data files to load (palette, terrain textures, objects, etc.).

### Level Data (`levels/levl2{NNN}.dat`)

Main level data containing terrain, units, and tribe configuration.

| Offset | Size | Description |
|--------|------|-------------|
| 0x0000 | 0x8000 | Heightmap (`Landscape<128>` struct — 128x128 grid of `u16` heights) |
| 0x8000 | 0x4000 | Skipped (unknown) |
| 0xC000 | 0x4000 | Skipped (unknown) |
| 0x10000 | 0x4000 | Land flags (128x128 grid) |
| after flags | 64 bytes | 4 x `TribeConfigRaw` (16 bytes each) |
| after tribes | 3 bytes | Sunlight data (R, G, B) |
| after sun | 110000 bytes | 2000 x `UnitRaw` (55 bytes each) — fixed unit slot array |

**Parser**: `LevelRes::new()` in `src/pop/level.rs`

**Note**: The unit slot array is always exactly 2000 entries. Empty slots have `subtype == 0`. Parsing stops after reading all 2000 slots to avoid trailing garbage data.

#### UnitRaw (55 bytes)

| Offset | Type | Field |
|--------|------|-------|
| 0 | u8 | subtype (unit type: 1=Brave, 2=Warrior, etc.) |
| 1 | u8 | tribe index |
| 2-5 | i16, i16 | world position (x, y) — 16-bit signed toroidal |
| ... | ... | Movement state, health, flags |

**Parser**: `UnitRaw` in `src/pop/level.rs`

## Palette Files

### Color Palette (`data/pal0-{key}.dat`)

256-color palette in BGRA format.

| Size | Format |
|------|--------|
| 1024 bytes | 256 entries x 4 bytes (B, G, R, A) |

**Parser**: `read_pal()` / `read_bin()` in `src/pop/level.rs`

Used for all sprite and terrain texture color mapping. Palette index 255 (0xFF) is typically used as the transparent color in sprites.

## Terrain & Environment Files

All files use the theme key derived from the level header's landscape type.

### Displacement Map (`data/disp0-{key}.dat`)

| Size | Format | Notes |
|------|--------|-------|
| 65536 bytes | 256x256 signed 8-bit | Flipped horizontally during load |

**Parser**: `read_disp()` in `src/pop/level.rs`

### Sky Texture (`data/sky0-{key}.dat`)

| Size | Format |
|------|--------|
| 262144 bytes | 512x512, 8-bit palette-indexed |

**Parser**: Uses `build_sky_interp_table()` in `src/pop/level.rs`

Sky colors are extracted from palette indices 0x71-0x7D (13 colors), sorted by luminance, and interpolated into a 256-entry lookup table for smooth gradients.

### Other Terrain Textures

| File | Size | Description |
|------|------|-------------|
| `data/bigf0-{key}.dat` | Variable | Big font textures (raw u8) |
| `data/cliff0-{key}.dat` | Variable | Cliff/edge textures (raw u8) |
| `data/fade0-{key}.dat` | Variable | Fade effect textures (raw u8) |
| `data/watdisp.dat` | 65536 bytes | Water displacement map (256x256) |

## Blend/Lighting Textures

### BL320 (`data/BL320-{KEY}.DAT`)

**Note**: Key is UPPERCASE in filename.

| Format | Description |
|--------|-------------|
| Raw 8-bit indexed | Multiple 256x256 pixel blocks, sequential layout |

**Parser**: `read_bl320()` in `src/pop/bl320.rs`

### BL160 (`data/BL160-{KEY}.DAT`)

Same format as BL320 but smaller dimensions.

**Parser**: `read_bl160()` in `src/pop/bl320.rs`

## 3D Object Files

Located in `objects/` subdirectory.

### Object Definitions (`objects/OBJS0-{key}.DAT`)

| Record Size | Description |
|------------|-------------|
| 52 bytes (packed) | One `ObjectRaw` per 3D model |

**Fields**:
| Offset | Type | Field |
|--------|------|-------|
| 0 | u16 | flags |
| 2 | u16 | face_count |
| 4 | u16 | point_count |
| 6 | u8 | morph_index |
| 7 | u32 | coordinate_scale |
| ... | u32 pairs | face/point data pointers |
| 0x2C | u8 | shapes_index (rotation 0 footprint) |
| 0x2D | u8 | rotation 1 footprint index |
| 0x2E-0x2F | u8, u8 | rotation 2-3 footprint indices |

**Parser**: `ObjectRaw::from_reader()` in `src/pop/objects.rs`

### Point Data (`objects/PNTS0-{key}.DAT`)

| Record Size | Format |
|------------|--------|
| 6 bytes | `PointRaw` — X (i16), Y (i16), Z (i16) |

### Face Data (`objects/FACS0-{key}.DAT`)

| Record Size | Format |
|------------|--------|
| 54 bytes | `FaceRaw` — texture index, flags, up to 4 vertex refs with UV coords |

**Parser**: `FaceRaw::from_reader()` in `src/pop/objects.rs`

### Shape Footprints (`objects/SHAPES.DAT`)

| Entries | Record Size | Total |
|---------|------------|-------|
| 95 | 48 bytes | 4560 bytes |

**Fields per entry**:
| Offset | Type | Field |
|--------|------|-------|
| 0 | u8 | width (cells) |
| 1 | u8 | height (cells) |
| 2 | u8 | origin_x |
| 3 | u8 | origin_z |
| 4-43 | [u8; 40] | cell_mask (bit 0 = occupied) |
| 44-47 | u32 | shape_ref |

**Parser**: `Shape` in `src/pop/objects.rs`

### Morph Animation (`objects/morph0-{key}.dat`)

Binary morph animation data. Loaded as raw bytes.

## Sprite Files (PSFB Format)

### Format Overview

PSFB (Populous Sprite File Binary) is a container format for RLE-compressed indexed-color sprites.

```
+-- Header (8 bytes) ---------+
| Magic: 0x42465350 ("PSFB")  |  4 bytes, little-endian
| Sprite count                 |  4 bytes, u32 LE
+-- Index (8 bytes x count) ---+
| Width                        |  2 bytes, u16 LE
| Height                       |  2 bytes, u16 LE
| Data offset (absolute)       |  4 bytes, u32 LE
+-- Sprite Data ---------------+
| RLE-compressed pixel rows    |
+------------------------------+
```

**RLE Encoding** (per row):
- Read a control byte:
  - If `0x00` (zero): End of row, advance to next row
  - If negative (i8 < 0): Skip `-value` pixels (transparent)
  - If positive (i8 > 0): Read next `value` bytes as literal pixel data (palette indices)
- Leading `0x00` bytes before first non-zero data indicate fully empty rows (skip)

**Output**: Palette-indexed pixels (u8 per pixel). Index 255 = transparent. Must be converted to RGBA using the level palette for display.

**Parser**: `ContainerPSFB` and `SpritePSFB` in `src/pop/psfb.rs`

### Sprite Files

| File | Description |
|------|-------------|
| `data/hspr0-{key}.dat` | All animated unit sprites (braves, warriors, shamans, etc.) |
| `data/plspanel.spr` | HUD panel sprites (buttons, backgrounds, icons) |

## Animation Files

### VSTART (`data/VSTART-0.ANI`)

| Record Size | Fields |
|------------|--------|
| 4 bytes | `VstartRaw`: index (u16), f1 (u8), f2 (u8) |

Animation start frame definitions.

### VFRA (`data/VFRA-0.ANI`)

| Record Size | Fields |
|------------|--------|
| 8 bytes | `VfraRaw`: index, width, height, next_vfra link |

Animation frame sequences — defines which sprite frames to play.

### VELE (`data/VELE-0.ANI`)

| Record Size | Fields |
|------------|--------|
| 10 bytes | `VeleRaw`: sprite_index (u16), X (i16), Y (i16), flags (u16), next_index (u16) |

Animation velocity/element data — per-frame sprite positioning.

**Parsers**: `src/pop/animation.rs`

## Encrypted Files

### Encryption Algorithm

Used by `plssphr.dat` and `plsdata.dat`.

**Cipher**: XOR with rotating 8-byte mask, then bitwise NOT.

```
Mask table: [0x20, 0x40, 0x80, 0x01, 0x02, 0x04, 0x08, 0x10]

For each byte at position i:
    k = (i - 3) & 7            // Rotating index (wraps 0-7)
    mask = 1 << k               // = mask_table[(i-3) & 7]
    decrypted = !(byte ^ mask)  // XOR then bitwise NOT
```

**Implementation** (`src/pop/pls.rs`):
```rust
pub fn decode(data: &mut [u8]) {
    let mut m: u8 = 0;
    for v in &mut data.iter_mut() {
        let k = m.wrapping_sub(3) & 7;
        let mask: u8 = 1 << k;
        *v = !(*v ^ mask);
        m = m.wrapping_add(1);
    }
}
```

### Sphere Mesh (`data/plssphr.dat`)

After decryption, contains a text-format sphere mesh:
```
Tri_mesh: <vertex_count> <face_count>
Vertex list:
<id> X: <x> Y: <y> Z: <z>
...
Face list:
<id> A:<v1> B:<v2> C:<v3> ...
```

### Planet Data (`data/plsdata.dat`)

After decryption, contains planet orbital system configuration (for the space background). Tab-separated text with planet parameters (name, orbit radius, rotation speed, etc.).

## Coordinate Systems

| System | Range | Usage |
|--------|-------|-------|
| World | 16-bit signed, toroidal (wraps at 65536) | Unit positions in level data |
| Tile | 0-254, steps of 2 | 128x128 terrain grid |
| Cell | 0-127 | 128x128 cell grid |

Conversions: `src/unit_control/coords.rs`

## Key Parsing Trait

All binary structures use the `BinDeserializer` trait (`src/pop/types.rs`):
```rust
trait BinDeserializer {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self>;
    fn from_reader_vec<R: Read>(reader: &mut R) -> Vec<Self>;
    fn from_file(path: &Path) -> Option<Self>;
    fn from_file_vec(path: &Path) -> Vec<Self>;
}
```

Packed binary structures use `#[repr(C, packed)]` with fixed `size_of::<T>()` bytes per record.

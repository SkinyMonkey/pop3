# Textures, Palettes and 3D Models

## Appendix AM: Data Tables Reference

### Unit Stats Table

**Base Address:** 0x0059fe44
**Entry Size:** 0x32 (50) bytes per unit type

```c
struct UnitStats {
    uint8_t idle_animation;    // +0x00
    uint8_t flag;              // +0x02
    uint8_t health_min_rand;   // +0x03
    uint8_t health_max_rand;   // +0x04
    uint32_t base_health;      // +0x0C (@ 0x59fe50)
    // ...
    uint8_t flags;             // +0x2C (@ 0x59fe70)
};

// Access pattern:
unit_health = (&DAT_0059fe50 + unit_type * 0x32);
unit_flags = (&DAT_0059fe70 + unit_type * 0x32);
```

**Unit Type Indices:**
| Index | Unit Type |
|-------|-----------|
| 0 | Wild |
| 1 | Brave |
| 2 | Warrior |
| 3 | Preacher |
| 4 | Spy |
| 5 | Super Warrior |
| 6 | Shaman |
| 7 | Angel of Death |

### Spell Data Table

**Base Address:** 0x00885760
**Entry Size:** 0xc65 (3173) bytes per spell

```c
struct SpellData {
    // ... many fields
    uint16_t blast_radius;      // +0x72
    uint16_t blast_radius2;     // +0x74
    // ...
    uint8_t effect_flags[...];  // +0x93d
};

// Access pattern:
spell_data = (&DAT_00885760 + spell_type * 0xc65);
```

### Building Data Table

**Base Address:** 0x005a0014
**Entry Size:** 0x4c (76) bytes per building

```c
struct BuildingData {
    // ... building parameters
    uint8_t worker_flags;  // +0x3c (@ 0x005a0050)
    // Bit 0x400: builds worker/harvester
    // Bit 0x40: different cost structure
};

// Access pattern:
building_data = (&DAT_005a0014 + building_type * 0x4c);
```

### Scenery Data Table

**Base Address:** 0x005a07a0
**Entry Size:** 0x18 (24) bytes per scenery type

```c
struct SceneryData {
    uint8_t default_state;   // +0x01
    uint8_t growth_state;    // +0x02
    uint32_t flags;          // +0x04
    uint16_t min_height;     // +0x10
    uint16_t max_height;     // +0x1A
};
```

### Constant.dat Parameters

**Loader:** `LoadConstantsDat` @ 0x0041eb50

**File Format:**
- Encrypted with XOR + bit-rotation (header: 0x40 0x7e)
- Each line: `P3CONST_<NAME> <VALUE>`
- Parsed with sscanf("%s %ld")

**Constants Table Entry (0x1f bytes):**
```c
struct ConstantEntry {
    char name[24];       // +0x00 - Parameter name
    uint8_t type;        // +0x19 - Type (1=byte, 2=short, 4=int)
    uint8_t flags;       // +0x1a - Flags
    void* valuePtr;      // +0x1b - Pointer to value
};
// Flags: bit2=loaded, bit0=percent, bit1=percentage
```

**Known Parameter Strings:**

*Unit Health:*
| Parameter | Address |
|-----------|---------|
| LIFE_BRAVE | 0x005a3f1c |
| LIFE_WARR | 0x005a3f3b |
| LIFE_SPY | 0x005a3f5a |
| LIFE_PREACH | 0x005a3f79 |
| LIFE_SWARR | 0x005a3f98 |
| LIFE_SHAMEN | 0x005a3fb7 |

*Unit Speed:*
- BRAVE_SPEED, WARRIOR_SPEED, SPY_SPEED
- RELIGIOUS_SPEED, SUPER_WARRIOR_SPEED
- MEDICINE_MAN_SPEED

*Combat Damage:*
| Parameter | Address |
|-----------|---------|
| FIGHT_DAMAGE_BRAVE | 0x005a4c30 |
| FIGHT_DAMAGE_WARR | 0x005a4c4f |
| SW_BLAST_DAMAGE_BRAVE | 0x005a3ff5 |
| SW_BLAST_DAMAGE_WARR | 0x005a4014 |
| SW_BLAST_DAMAGE_SPY | 0x005a4033 |
| SW_BLAST_DAMAGE_PREACH | 0x005a4052 |
| SW_BLAST_DAMAGE_SWARR | 0x005a4071 |
| SW_BLAST_DAMAGE_SHAMEN | 0x005a4090 |
| SW_BLAST_DAMAGE_AOD | 0x005a40af |
| SWARM_PERSON_DAMAGE | 0x005a4f56 |

*Mana Generation:*
| Parameter | Address |
|-----------|---------|
| MAX_MANA | 0x005a339b |
| START_MANA | 0x005a33ba |
| CONVERT_MANA | 0x005a33d9 |
| MANA_F_BRAVE | 0x005a36e0 |
| MANA_F_WARR | 0x005a36ff |
| MANA_F_SPY | 0x005a371e |
| MANA_F_PREACH | 0x005a373d |
| MANA_F_SWARR | 0x005a375c |
| MANA_F_SHAMEN | 0x005a377b |
| MANA_F_TRAINING | 0x005a4432 |
| MANA_F_HOUSED | 0x005a4451 |
| MANA_F_WORKING | 0x005a4470 |

*Building Costs:*
- RAISE_LOWER_COST @ 0x005a448f

*Spell Costs (base names):*
- SPELL_BURN, SPELL_BLAST, SPELL_BOLT, SPELL_WWIND
- SPELL_PLAGUE, SPELL_INVIS, SPELL_FIREST, SPELL_HYPNO
- SPELL_GARMY, SPELL_EROSION, SPELL_SWAMP, SPELL_LBRIDGE
- SPELL_AOD, SPELL_QUAKE, SPELL_FLATTEN, SPELL_VOLCANO
- SPELL_ARMAGEDDON, SPELL_CONVERT_WILD, SPELL_SHIELD
- SPELL_BLOODLUST, SPELL_TELEPORT
- Plus _OPT_S variants for each (optimization multiplier)

*Spell Altitude Bands:*
- ALT_BAND_0_SPELL_INCR through ALT_BAND_7_SPELL_INCR (@ 0x005a506d-0x005a5146)

*Combat Damage:*
- SW_BLAST_DAMAGE_SH_IN_V% @ 0x005a3300 (Shaman in vehicle)
- SW_BLAST_DAMAGE @ 0x005a331f (base)
- SW_BLAST_DAMAGE_TOWER @ 0x005a335d
- SW_BLDG_DAMAGE_DELAY @ 0x005a3493
- BLAST_DAMAGE_BLDG @ 0x005a34f0
- BLAST_DAMAGE_PERSON @ 0x005a350f
- SWARM_PERSON_DAMAGE @ 0x005a4f56
- FALL_OUT_OF_WW_DAMAGE @ 0x005a504e
- BLOODLUST_DAMAGE_X @ 0x005a5ab8

*Training Costs (Human vs AI):*
- HUMAN_TRAIN_MANA_WARR/SPY/PREACH/SWARR (@ 0x005a58c8-0x005a5925)
- CP_TRAIN_MANA_WARR/SPY/PREACH/SWARR (@ 0x005a5944-0x005a59a1)
- TRAIN_MANA_BAND_00_03 through TRAIN_MANA_BAND_21+ (@ 0x005a580e-0x005a58a9)

*Unit Speed:*
- BRAVE_SPEED @ 0x005a59df
- WARRIOR_SPEED @ 0x005a59fe
- RELIGIOUS_SPEED @ 0x005a5a1d
- SPY_SPEED @ 0x005a5a3c
- SUPER_WARRIOR_SPEED @ 0x005a5a5b
- MEDICINE_MAN_SPEED @ 0x005a5a7a

*Vehicle Health:*
- VEHICLE_LIFE_BOAT @ 0x005a5bcf
- VEHICLE_LIFE_BALLOON @ 0x005a5c0d

*Mana Generation:*
- HUMAN_MANA_ADJUST @ 0x005a35aa
- COMPUTER_MANA_ADJUST @ 0x005a35c9
- SHAMEN_DEAD_MANA_%_LOST @ 0x005a35e8
- SHAMEN_DEAD_MANA_%_GAIN @ 0x005a3607
- MANA_F_HUT_LEVEL_1/2/3 (@ 0x005a44ae-0x005a44ec)
- MANA_IDLE_BRAVES @ 0x005a5b34
- MANA_IDLE_SPECIALS @ 0x005a5b53
- MANA_BUSY_BRAVES @ 0x005a5b72
- MANA_BUSY_SPECIALS @ 0x005a5b91

*Building Maximums:*
- BLDG_MAX_BUILD_TEEPEE1/2/3 @ 0x005a41c6-0x005a4204
- BLDG_MAX_BUILD_DTOWER @ 0x005a4223
- BLDG_MAX_BUILD_TEMPLE @ 0x005a4242
- BLDG_MAX_BUILD_SPY @ 0x005a4261
- BLDG_MAX_BUILD_WARR @ 0x005a4280
- BLDG_MAX_BUILD_SWARR @ 0x005a429f
- BLDG_MAX_BUILD_BOAT @ 0x005a42be
- BLDG_MAX_BUILD_BALLOON @ 0x005a42dd

*Population Caps:*
- MAX_POP_VALUE__HUT_1 @ 0x005a4afa
- MAX_POP_VALUE__HUT_2 @ 0x005a4b19
- MAX_POP_VALUE__HUT_3 @ 0x005a4b38

*Tree/Wood Parameters:*
- TREE1-6_WOOD_VALUE @ 0x005a46dc-0x005a4777 (wood yield)
- TREE1-6_WOOD_GROW @ 0x005a4796-0x005a4831 (growth rate)
- TREE1-6_DORMANT_TIME @ 0x005a4850-0x005a48eb (dormancy)

*Wood Costs (Units):*
- WOOD_BRAVE @ 0x005a490a
- WOOD_WARR @ 0x005a4929
- WOOD_PREACH @ 0x005a4948
- WOOD_SWARR @ 0x005a4967
- WOOD_SHAMEN @ 0x005a4986

*Wood Costs (Buildings):*
- WOOD_HUT_1/2/3 @ 0x005a49a5-0x005a49e3
- WOOD_DRUM_TOWER @ 0x005a4a02
- WOOD_TEMPLE @ 0x005a4a21
- WOOD_SPY_HUT @ 0x005a4a40
- WOOD_WARRIOR @ 0x005a4a5f
- WOOD_SUPER @ 0x005a4a7e
- WOOD_RECONV @ 0x005a4a9d
- WOOD_BOAT_1 @ 0x005a4abc
- WOOD_AIR_1 @ 0x005a4adb

*Vehicle Wood Costs:*
- WOOD_VEHICLE_BOAT1 @ 0x005a4bf2
- WOOD_VEHICLE_AIRSHIP_1 @ 0x005a4c11

*Spell Max Effects (SP_1_OFF_MAX_*):*
All at 0x005a527c-0x005a54e8 for each spell type

*Level Editor Maximums (LSME_1_OFF_MAX_*):*
- EROSION, LBRIDGE, FLATTEN, HILL, VALLEY, RISE, DIP, TREES, WILDS

### Global Data Addresses

| Address | Size | Description |
|---------|------|-------------|
| 0x00888984 | 128×128×2 | Heightmap (g_Heightmap) |
| 0x00888987 | 128×128×4 | Cell flags (g_CellFlags) |
| 0x00888982 | varies | Object spatial index |
| 0x00973640 | varies | Color palette (BGRA) |
| g_CosTable | 2048×2 | Cosine lookup |
| g_SinTable | 2048×2 | Sine lookup |
| 0x005a7d48 | varies | File loading buffer |
| 0x005a7d80 | varies | Animation elements |
| 0x005a7d84 | varies | Frame data |
| 0x005a7d90 | varies | Sprite indices |

---

## Appendix AN: Texture and Level Loading

### Level Textures

**Loader:** `LoadLevelTextures` @ 0x00421320

Loads terrain texture files based on level theme:
```c
void LoadLevelTextures(int levelId) {
    // Get theme from level header
    int theme = LoadLevelHeader(levelId, ...);
    char themeChar = (theme < 10) ? '0' + theme : 'W' + (theme - 10);

    // Load files:
    // data/plspl0_%c.dat - Palette data
    // data/plsft0_%c.dat - Font data
    // data/plscv0_%c.dat - Curve data
    // data/plstx_%03d.dat - Texture data

    File_ReadEntire(palettePath, DAT_00867590, 0x400, ...);
    File_ReadEntire(fontPath, &DAT_00957078, 0x4000, ...);
    File_ReadEntire(curvePath, &DAT_00802398, 0x6000, ...);
    File_ReadEntire(texturePath, &DAT_007b9178, 0x40000, ...);
}
```

### Sprite File References

Frontend sprites (data/fenew/):
- `fettru.spr`, `fettee.spr`, `fettwe.spr` - Tribe UI elements
- `fesd*.spr` - Screen decoration
- `felo*.spr`, `fehi*.spr` - Logo elements
- `feft*.spr` - Font sprites
- `felgs*.spr` - Language-specific sprites
- `igmslidr.spr`, `feslider.spr` - Sliders
- `feboxes.spr`, `fepointe.spr`, `fecursor.spr` - UI elements

Game sprites:
- `data/plspanel.spr` - In-game panel
- `data/plsspace.spr` - Space/background

---

## Appendix AY: 3D Shape/Model System

### Shape Loading Functions

| Function | Address | Purpose |
|----------|---------|---------|
| FUN_0049bba0 | 0x0049bba0 | shapes.dat loader |
| FUN_0049bc30 | 0x0049bc30 | Shape unloader/cleanup |
| FUN_00410870 | 0x00410870 | Sphere file parser |
| Shape_Init | 0x0048f8d0 | Shape object init |

### Shape Data Files

- `objects/shapes.dat` (0x0059f3d0) - 3D model data
- `objects/shapes.ver` (0x0059a040) - Version file

### 3D Model Format (Sphere Text Format)

**Tri-mesh Section:**
```
Tri-mesh, [vertex_count], [face_count]
```

Memory allocation:
- Vertex storage: `vertex_count * 0x1c` bytes (28 bytes/vertex)
- Face storage: `face_count * 0xc` bytes (12 bytes/face)

### Vertex Structure (28 bytes / 0x1c)

```c
struct Vertex3D {
    float x;         // +0x00: X coordinate
    float y;         // +0x04: Y coordinate
    float z;         // +0x08: Z coordinate
    float normal_x;  // +0x0c: Normal X (normalized)
    float normal_y;  // +0x10: Normal Y (normalized)
    float normal_z;  // +0x14: Normal Z (normalized)
    float x_copy;    // +0x18: Copy for normalization
};
```

### Face Structure (12 bytes / 0xc)

```c
struct Face3D {
    uint32_t vertex_a;  // +0x00: Vertex A index
    uint32_t vertex_b;  // +0x04: Vertex B index
    uint32_t vertex_c;  // +0x08: Vertex C index
};
```

### Shape Object Fields

When object type = 9 (Shape):
- Offset +0x2b: Render state (=1)
- Offset +0x2c: Current state
- Offset +0x9b: Shape index
- Offset +0x68: Animation frame
- Offset +0x9e: Y-rotation
- Offset +0x9f: X-rotation

### 3D Transformation Pipeline

**Camera Functions:**
| Function | Address | Purpose |
|----------|---------|---------|
| Camera_WorldToScreen | 0x0046ea30 | 3D→2D projection |
| Math_RotationMatrix | 0x004bc360 | Rotation matrix calc |
| Camera_SetupProjection | 0x0046edb0 | Camera parameter init |
| Camera_ApplyRotation | 0x0046f2a0 | Apply camera rotation |
| Camera_GenerateProjectionLUT | 0x0046f1e0 | Pre-computed LUTs |

**Isometric Projection Formula:**
```c
screen_x = (world_x * matrix_ac + world_z * matrix_b4) >> 14;
screen_y = (world_y * matrix_bc + world_x * matrix_b8 + world_z * matrix_c0) >> 14;
```

### Terrain Vertex Integration

**Terrain Vertex (32 bytes):**
- Position X, Y, Z (indices 0-2)
- Texture/height data (index 1, 5)
- Flags (index 6)
- Lighting value (index 5)

**Triangle Command (0x46 bytes):**
- 3 vertex references (offsets 0x6, 0x1a, 0x2e)
- Screen coordinates (offsets 0xc, 0x10)
- Shading values (offsets 0x16, 0x2a, 0x3e)
- Command flags (offset 0x45)

### Global Shape Data

| Address | Purpose |
|---------|---------|
| 0x005a7d78 | Shape data buffer |
| 0x00598170 | Shape count |
| 0x007f919a-0x007f91ba | Water mesh vertices (0x2d×0x20) |
| 0x007b8f78-0x007b8fb0 | Camera/rendering state |

---

## Appendix BC: Palette System

### Palette Data

**Primary Palette:** 0x00973640 (256 × 4-byte BGRA)

### Palette Files (data/palX-0.dat)

| File | Purpose |
|------|---------|
| pal0-0.dat | Main game palette |
| sky0-0.dat | Sky/background |
| fade0-0.dat | Fade effects |
| ghost0-0.dat | Transparency |
| bl320-0.dat | Blue colors |
| anibl0-0.dat | Animated blue |
| baclr0-0.dat | Background color |
| bsclr0-0.dat | Background special |
| bigf0-0.dat | Big font |
| cliff0-0.dat | Cliff textures |
| disp0-0.dat | Display |
| al0-0.dat | Alignment |

### Frontend Palettes (data/fenew/)

11 variants (0-B) for each:
- `fepalX.dat` - Frontend palette
- `fefadeX.dat` - Fade palette
- `feghostX.dat` - Ghost/transparency
- `febackgX.dat` - Background

### Palette Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Palette_IndexToRGBA | 0x00402800 | Convert index to RGBA |
| FUN_004503f0 | 0x004503f0 | Master palette init |
| FUN_00450790 | 0x00450790 | Level palette loading |
| FUN_004506b0 | 0x004506b0 | Animation object palette |
| FUN_005102e0 | 0x005102e0 | DirectDraw palette init |

### Palette Globals

| Address | Purpose |
|---------|---------|
| 0x00973640 | Primary palette (256×BGRA) |
| 0x0087acab | Palette file path |
| 0x0087accb | Fade palette path |
| 0x0087aceb | Ghost palette path |
| 0x0087ad2b | Blue palette path |
| 0x0087e365 | Current level palette index |

---

## Appendix CK: Data File Formats

### Level-Specific Data Files

| Pattern | Size | Purpose |
|---------|------|---------|
| `data/plstx%03d.dat` | 0x40000 | Terrain textures (256KB) |
| `data/plspl0-%c.dat` | 0x400 | Palette data |
| `data/plsft0-%c.dat` | 0x4000 | Font/text data |
| `data/plscv0-%c.dat` | 0x6000 | Curve/conversion data |

**File selection:** Character `%c` is derived from level header parameter:
```c
if (level_param < 10) {
    c = '0' + level_param;  // '0'-'9'
} else {
    c = 'W' + level_param;  // 'a'-'z' for 10+
}
```

### Global Data Files

| File | Address | Size | Purpose |
|------|---------|------|---------|
| `data\watdisp.dat` | DAT_00599ac8 | 0x10000 | Water displacement/shading lookup |
| `objects/shapes.dat` | DAT_005a7d78 | Variable | 3D shape definitions |
| `objects/shapes.ver` | - | - | Shape version file |
| `plssphr.dat` | DAT_005a7d48 | Variable | Sphere mesh data (XOR encrypted) |
| `data\plsbackg.dat` | - | - | Background/skybox data |
| `data\skylens.dat` | - | - | Sky lens flare data |
| `data\alavaa.dat` | - | - | Lava animation data |

### Frontend Data (fenew folder)

Each level type (0-9, a, b) has:
- `febackgX.dat` - Background image
- `fepalX.dat` - Palette
- `fefadeX.dat` - Fade transition data
- `feghostX.dat` - Ghost/transparency overlay

### Object Animation Files

| Pattern | Purpose |
|---------|---------|
| `objects/objs0-%d.dat` | Object sprite sheets |
| `objects/aniob0-0.dat` | Object animation data |
| `objects/morph0-0.dat` | Morph animation data |

### Terrain Effect Files

| File | Purpose |
|------|---------|
| `data/pal0-0.dat` | Base palette |
| `data/sky0-0.dat` | Sky gradient |
| `data/fade0-0.dat` | Fade effects |
| `data/ghost0-0.dat` | Ghost transparency |
| `data/bl320-0.dat` | Blend/lighting |
| `data/anibl0-0.dat` | Animated blend |
| `data/baclr0-0.dat` | Background color |
| `data/bsclr0-0.dat` | Base color |
| `data/bigf0-0.dat` | Big font |
| `data/cliff0-0.dat` | Cliff textures |
| `data/disp0-0.dat` | Displacement maps |
| `data/al0-0.dat` | Alpha/transparency |

### Footprint Files

Files `data\f00t0-0.dat` through `data\f00t11-0.dat` contain unit footprint/shadow data.

---

## Appendix CL: Palette and Color Lookup System

### Palette Structure

| Address | Size | Purpose |
|---------|------|---------|
| DAT_00973640 | 1024 bytes | Main palette (256 × 4 bytes RGBA) |
| DAT_00867590 | 0x400 | Level palette source |

### Color Lookup Function @ 0x0050f7f0

Finds closest palette index for RGB color using distance calculation:

```c
uint Palette_FindClosestColor(byte* palette, byte r, byte g, byte b) {
    int best_dist = 0x1000000;  // Very large initial distance
    uint best_index = 0;

    // First pass: find minimum squared distance
    for (int i = 0; i < 256; i++) {
        int dr = r - palette[i*4];
        int dg = g - palette[i*4+1];
        int db = b - palette[i*4+2];
        int dist = dr*dr + dg*dg + db*db;

        if (dist < best_dist) {
            best_dist = dist;
            if (dist == 0) return i;  // Exact match
        }
    }

    // Second pass: collect all matches at minimum distance
    byte matches[256];
    int match_count = 0;
    for (int i = 0; i < 256; i++) {
        // ... same distance calculation ...
        if (dist == best_dist) {
            matches[match_count++] = i;
        }
    }

    if (match_count == 1) return matches[0];

    // Third pass: use Manhattan distance to break ties
    int best_manhattan = 0x1000000;
    for (int i = 0; i < match_count; i++) {
        int m = abs(r - palette[matches[i]*4]) +
                abs(g - palette[matches[i]*4+1]) +
                abs(b - palette[matches[i]*4+2]);
        if (m < best_manhattan) {
            best_manhattan = m;
            best_index = matches[i];
        }
    }

    // Fourth pass: tie-break by luminance
    // Prefers darker colors (2*r² + 2*g² + b²)
    ...

    return best_index;
}
```

### Special Color Indices (from FUN_0044ff20)

Pre-computed palette indices for common colors:

| Address | RGB | Purpose |
|---------|-----|---------|
| DAT_005a17a8 | (0, 0, 191) | Blue medium |
| DAT_005a17a9 | (0, 0, 255) | Blue bright |
| DAT_005a17aa | (0, 0, 127) | Blue dark |
| DAT_005a17ad | (191, 0, 0) | Red medium |
| DAT_005a17ae | (255, 0, 0) | Red bright |
| DAT_005a17af | (127, 0, 0) | Red dark |
| DAT_005a17b2 | (191, 191, 0) | Yellow medium |
| DAT_005a17b3 | (255, 255, 0) | Yellow bright |
| DAT_005a17b4 | (127, 127, 0) | Yellow dark |
| DAT_005a17b7 | (0, 191, 0) | Green medium |
| DAT_005a17b8 | (0, 255, 0) | Green bright |
| DAT_005a17b9 | (0, 127, 0) | Green dark |

These are used for player/tribe color indicators.

---

## Appendix CN: Shape/Sphere File Format

### plssphr.dat Format

Text-based format (XOR encrypted), parsed by Shape_ParseSphereFile @ 0x00410870:

```
Tri_mesh: <vertex_count> <face_count>
Vertex list:
<id> X: <x> Y: <y> Z: <z>
<id> X: <x> Y: <y> Z: <z>
...
Face list:
<id> A:<v1> B:<v2> C:<v3> AB:<edge1> BC:<edge2> CA:<edge3>
...
```

### Parsing Details

1. **Header**: "Tri_mesh:" specifies vertex and face counts
2. **Vertices**: Stored as 28 bytes each (0x1C):
   - Bytes 0-12: Position (X, Y, Z as floats)
   - Bytes 12-24: Normalized direction (computed: pos / |pos|)
3. **Faces**: Stored as 12 bytes each (0x0C):
   - 3 vertex indices (A, B, C)
   - Edge data stored but rearranged: [A, C, B] order

### Shape Structure (0x60 = 96 bytes)

| Offset | Field |
|--------|-------|
| +0x00 | Type (3 = sphere mesh) |
| +0x04 | Vertex count |
| +0x08 | Vertex buffer pointer |
| +0x0C | Normal buffer pointer |
| +0x10 | Face count |
| +0x14 | Face buffer pointer |
| +0x1C | Flags (bit 0-2: buffers allocated) |
| +0x20-0x28 | Position offset (X, Y, Z) |
| +0x38-0x48 | Transform matrix |
| +0x50 | Scale X (default 0.5) |
| +0x54 | Scale Y (default 0.1) |
| +0x58 | Scale Z (default 1.0) |
| +0x5C | Render mode (1) |

### shapes.dat Format

Binary format loaded by Shape_LoadDatFile @ 0x0049bba0:
- Header at DAT_005a7d78
- Shape count at DAT_00598170
- Each entry is 0x30 (48) bytes
- Pointers are relative and fixed up after load

---

## Appendix DH: Texture and UV System

### UV Rotation Tables

The terrain uses pre-computed UV rotation tables for texture mapping on terrain cells.

**Initialization Function:** `Terrain_InitializeUVRotationTables @ 0x00451110`

**UV Table Structure at DAT_0059bf50:**
```c
// Each entry is 0x18 (24) bytes containing 6 UV coordinates (3 vertices × 2 components)
// Table has 4 rotation states (0°, 90°, 180°, 270°)

struct UVRotationEntry {
    int32_t u1, v1;    // Vertex 1 UV (offset +0x00, +0x04)
    int32_t u2, v2;    // Vertex 2 UV (offset +0x08, +0x0C)
    int32_t u3, v3;    // Vertex 3 UV (offset +0x10, +0x14)
};

// Access: rotation_index * 0x18 + DAT_0059bf50
```

**Triangle UV Application (from Triangle_CreateWithRotatedUVs @ 0x0046fb40):**
```c
// Copy UV coordinates based on texture rotation
iVar2 = (uint)*(byte *)(param_1 + 0x45) * 0x18;  // texture_id * 24
*(int*)(triangle + 0x0e) = *(int*)(DAT_0059bf50 + iVar2);      // U1
*(int*)(triangle + 0x12) = *(int*)(DAT_0059bf54 + iVar2);      // V1
*(int*)(triangle + 0x22) = *(int*)(DAT_0059bf58 + iVar2);      // U2
*(int*)(triangle + 0x26) = *(int*)(DAT_0059bf5c + iVar2);      // V2
*(int*)(triangle + 0x36) = *(int*)(DAT_0059bf60 + iVar2);      // U3
*(int*)(triangle + 0x3a) = *(int*)(DAT_0059bf64 + iVar2);      // V3
```

### Texture Loading

**Function:** `LoadLevelTextures @ 0x00421320`

**Texture Files Loaded:**
| File Pattern | Destination | Size | Purpose |
|--------------|-------------|------|---------|
| `data/plspl0_X.dat` | DAT_00867590 | 0x400 | Sprite palette |
| `data/plsft0_X.dat` | DAT_00957078 | 0x4000 | Font textures |
| `data/plscv0_X.dat` | DAT_00802398 | 0x6000 | Cave/special textures |
| `data/plstx_NNN.dat` | DAT_007b9178 | 0x40000 | Main terrain textures |

Where X = level type character (0-9, a-f) and NNN = level number.

### Palette System

**Function:** `Palette_IndexToRGBA @ 0x00402800`

**Global Palette Location:** `DAT_00973640`

```c
// Convert 8-bit palette index to RGBA
undefined1* Palette_IndexToRGBA(undefined1* out, byte index) {
    int offset = index * 4;
    out[2] = DAT_00973640[index];          // R
    out[1] = DAT_00973640[offset + 1];     // G
    out[0] = DAT_00973640[offset + 2];     // B
    out[3] = 0xFF;                          // A (always opaque)
    out[4] = index;                         // Original index
    return out;
}
```

---

## Appendix DT: Shading Lookup Tables

### Initialization

**Function:** `Shading_InitializeLookupTables @ 0x00486a20`

Initializes terrain shading lookup tables based on render mode (2D vs 3D).

**Mode 2 (Software 2D):**
- Loads water displacement from `data/watdisp.dat` (64KB)
- Initializes UV rotation tables
- Sets up basic shading tables

**Mode 3 (Hardware-accelerated 3D):**
- Allocates tile cache: `DAT_007b9128` = 512KB, `DAT_007b916c` = 160KB
- Initializes 64K tile lookup entries
- Generates 129×129 shading gradient grid
- Loads water displacement data

**Water Displacement Data:**
- File: `data/watdisp.dat`
- Size: 0x10000 (65536) bytes
- Storage: `DAT_00599ac8`
- Used for water wave animation UV distortion

### Shading Gradient Generation

For 3D mode, generates a 129×129 (0x81 × 0x81) gradient:
```c
uint gradient = 0x80808080;  // Starting gray
for (y = 0; y < 129; y++) {
    for (x = 0; x < 129; x++) {
        FUN_00487b00(gradient);
        gradient = (gradient & 0xFFFFFF00) | ((gradient + 2) & 0xFF);
    }
    gradient = ((gradient >> 8) + 2) << 8 | (gradient & 0xFF);
}
```

---

## Appendix EE: Palette System (Basic Rendering)

### Overview

Populous: The Beginning uses an 8-bit indexed color system with 256-color palettes. The palette is stored in BGRA format and converted to RGBA during rendering.

### Core Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Palette_IndexToRGBA | 0x00402800 | Converts 8-bit palette index to RGBA |
| Palette_InitializePaths | 0x004503f0 | Sets up file paths for all palette files |
| Palette_LoadForLevel | 0x00450790 | Loads palettes for current level |

### Primary Data Structures

**Main Palette:** `DAT_00973640`
- Size: 1024 bytes (256 entries × 4 bytes)
- Format: BGRA (Blue, Green, Red, Alpha)

**Palette File Path Globals:**

| Address | Purpose |
|---------|---------|
| DAT_0087acab | Main palette path (`pal0-0.dat`) |
| DAT_0087accb | Fade palette path (`fade0-0.dat`) |
| DAT_0087aceb | Ghost palette path (`ghost0-0.dat`) |
| DAT_0087ad2b | Blend palette path (`bl320-0.dat`) |
| DAT_0087ad4b | Animated blend path (`anibl0-0.dat`) |
| DAT_0087adab | Background color path (`baclr0-0.dat`) |
| DAT_0087adcb | Base color path (`bsclr0-0.dat`) |
| DAT_0087adeb | Big font path (`bigf0-0.dat`) |
| DAT_0087ae0b | Cliff palette path (`cliff0-0.dat`) |
| DAT_0087ae2b | Display palette path (`disp0-0.dat`) |
| DAT_0087ae4b | Alignment palette path (`al0-0.dat`) |
| DAT_0059ebf0 | Sky palette path (`sky0-0.dat`) |

### Palette Files

| File | Purpose |
|------|---------|
| `pal0-0.dat` | Main 256-color palette (1KB) |
| `sky0-0.dat` | Sky gradient colors |
| `fade0-0.dat` | Fade transition lookup table |
| `ghost0-0.dat` | Transparency/alpha blend lookup table |
| `bl320-0.dat` | 32-level blend table for software alpha |
| `anibl0-0.dat` | Animated water/lava color cycling |
| `baclr0-0.dat` | Background fill color |
| `bsclr0-0.dat` | Base/default color |
| `bigf0-0.dat` | Big font palette entries |
| `cliff0-0.dat` | Cliff texture color ramp |
| `disp0-0.dat` | Display effect colors |
| `al0-0.dat` | Alignment/grid colors |

### Palette_IndexToRGBA Algorithm

```c
// @ 0x00402800
// Converts 8-bit palette index to 5-byte RGBA output
// Note: Swaps R and B channels (palette is BGRA, output is RGBA)

void Palette_IndexToRGBA(byte* output, byte index) {
    int offset = index * 4;

    // BGRA to RGBA conversion
    output[0] = DAT_00973640[offset + 2];  // Red (from B position)
    output[1] = DAT_00973640[offset + 1];  // Green
    output[2] = DAT_00973640[offset + 0];  // Blue (from R position)
    output[3] = 0xFF;                       // Alpha (always opaque)
    output[4] = index;                      // Original palette index

    return output;
}
```

### Level-Specific Palettes

The game supports 16 level themes (0-9, a-f). Each theme has its own palette variant:

```c
// Palette_InitializePaths @ 0x004503f0
void Palette_InitializePaths(char level_type) {
    // Convert level type to character
    char suffix;
    if (level_type < 10) {
        suffix = level_type + '0';  // '0'-'9'
    } else {
        suffix = level_type + 'W';  // 'a'-'f' (87 = 'W', so 10+'W' = 'a')
    }

    // Update all palette path strings with new suffix
    // e.g., "data\\pal0-0.dat" becomes "data\\pal0-3.dat" for level type 3

    DAT_0059ebfa = suffix;  // sky path
    DAT_0087acb5 = suffix;  // pal path
    DAT_0087acd6 = suffix;  // fade path
    DAT_0087acf7 = suffix;  // ghost path
    // ... etc for all palette files

    DAT_0087e365 = level_type;  // Store current level type
}
```

### Palette Loading Flow

```
Game Startup / Level Load
    │
    ├─→ Palette_InitializePaths(level_type)
    │   └─→ Updates all palette file paths for level theme
    │
    └─→ Palette_LoadForLevel()
        ├─→ Palette_InitializePaths(DAT_008853d5)
        ├─→ LoadLevelSpecialData()
        ├─→ LoadObjectivesData()
        └─→ Sprite_LoadBank()
```

### Usage in Rendering

The palette is accessed throughout the rendering pipeline:

1. **Sprite Rendering**: Sprite pixel values (0-255) are indices into the palette
2. **Terrain Textures**: Texture pixels are palette indices
3. **Font Rendering**: `Render_DrawCharacter` calls `Palette_IndexToRGBA` for each pixel
4. **UI Elements**: Health bars, selection rings use palette colors
5. **Minimap**: Terrain and object colors from palette

### Special Palette Indices

Pre-computed palette indices for common colors (from initialization):

| Address | RGB Value | Purpose |
|---------|-----------|---------|
| DAT_005a178d | (0, 0, 0) | Black |
| DAT_005a1791 | (255, 255, 255) | White |
| DAT_005a1795 | (255, 0, 0) | Red |
| DAT_005a1799 | (0, 255, 0) | Green |
| DAT_005a179d | (0, 0, 255) | Blue |
| DAT_005a17a1 | (255, 255, 0) | Yellow |
| DAT_005a17a5 | (255, 0, 255) | Magenta |
| DAT_005a17a9 | (0, 255, 255) | Cyan |
| DAT_005a17ad | (127, 127, 127) | Gray |
| DAT_005a17b1 | (127, 0, 0) | Dark red |
| DAT_005a17b5 | (0, 0, 127) | Dark blue |
| DAT_005a17b9 | (0, 127, 0) | Dark green |

These are used for tribe/player color indicators and UI elements.

---


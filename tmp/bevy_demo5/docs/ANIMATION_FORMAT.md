# Populous: The Beginning - Animation Format Documentation

## Overview

The game uses multiple layers of animation data:
1. **Sprite Banks** (HSPR0-0.DAT) - Raw sprite frames with RLE compression
2. **Animation Definition Files** (VSTART, VFRA, VELE) - Animation sequences and timing
3. **Hardcoded Tables** in the executable - Person type to animation mappings

## Bullfrog Format Lineage

Populous: The Beginning uses animation formats derived from earlier Bullfrog games,
particularly **Syndicate Wars** (1996). The formats are nearly identical:

| Syndicate Wars | Populous TB | Description |
|----------------|-------------|-------------|
| HSTA-0.ANI | VSTART-0.ANI | Animation start indices |
| HFRA-0.ANI | VFRA-0.ANI | Frame linked list |
| HELE-0.ANI | VELE-0.ANI | Sprite elements with offsets |

**Key difference**: Syndicate Wars uses 2-byte VSTART entries, while Populous uses
4-byte entries (adding a mirror reference field).

### Online Documentation Sources
- [libsyndicate](https://github.com/CeRiAl/libsyndicate) - Documented Bullfrog animation format
- [PopSpriteEditor](https://github.com/Toksisitee/PopSpriteEditor) - PSFB sprite format
- [pop3-rev](https://github.com/hrttf111/pop3-rev) - Ghidra reverse engineering project
- [Popre.net Forum](https://www.popre.net/forum/) - Community documentation

## File Formats

### HSPR0-0.DAT - Sprite Bank

```
Header:
  bytes 0-3:   Magic "PSFB" (0x42465350)
  bytes 4-7:   Frame count (u32 little-endian)

Frame Headers (8 bytes each, starting at offset 8):
  bytes 0-1:   Width (u16)
  bytes 2-3:   Height (u16)
  bytes 4-7:   Pixel data offset (u32)

Pixel Data:
  RLE encoded:
  - byte > 0: read N literal pixels
  - byte < 0: skip |N| transparent pixels (palette index 255)
  - byte == 0: end of row
```

### VSTART-0.ANI - Animation Start Indices

**Verified from Ghidra decompilation** (`Animation_LoadAllData` at 0x00452530):
The code uses `>> 2` (divide by 4) to calculate entry count, confirming 4-byte entries.

```
Total size: 3168 bytes
Records: 792 entries (4 bytes each)
Organization: 8 entries per animation (8 directions)

Record format (4 bytes):
  bytes 0-1:  VFRA frame index (u16) - index into VFRA linked list
  bytes 2-3:  Mirror reference (u16)
              - For directions 0-4: typically 0x0000
              - For directions 5-7: source direction index to mirror from

Total animations: 99 (792 / 8)

Note: Unlike Syndicate Wars (2 bytes/entry), Populous uses 4 bytes per entry.
```

**Animation Chain**:
```
VSTART[anim_id * 8 + direction] → VFRA index
       ↓
VFRA[index] → {sprite_frame, timing, flags, next_frame}
       ↓                                      ↓
  sprite in HSPR bank                    loops back to create animation
```

### VFRA-0.ANI - Frame Sequences (Linked List)

**Verified from Ghidra decompilation** (`Animation_LoadAllData` at 0x00452530):
The code uses `>> 3` (divide by 8) to calculate entry count, confirming 8-byte entries.

```
Total size: 31688 bytes
Records: 3961 entries (8 bytes each)

Record format (8 bytes) - matches libsyndicate SpriteFrame:
  bytes 0-1:  first_element (u16) - sprite frame index or VELE element index
  bytes 2:    width (u8) - frame width
  bytes 3:    height (u8) - frame height
  bytes 4-5:  flags (u16)
              - 0x0100: Animation start frame marker
  bytes 6-7:  next_frame (u16) - next VFRA index (creates linked list loop)

Note: When first_element > HSPR sprite count, it references VELE elements.
```

### VELE-0.ANI - Sprite Elements

Based on libsyndicate documentation (SpriteElement format):

```
Total size: 160760 bytes
Records: 16076 entries (10 bytes each)

Record format (10 bytes) - from libsyndicate:
  bytes 0-1:  sprite (u16) - index into HSPR sprite bank
  bytes 2-3:  x_offset (i16) - horizontal position offset
  bytes 4-5:  y_offset (i16) - vertical position offset
  bytes 6-7:  flipped (u16) - horizontal flip flag
  bytes 8-9:  next_element (u16) - next element index (0 = end of list)

Purpose: Allows complex sprites composed of multiple sub-sprites with offsets.
VFRA entries with high indices (>7953) reference VELE elements rather than
direct HSPR sprites.
```

## Direction System

The game uses 8 directions but only stores 5 in the sprite bank:

```
Stored directions (0-4):
  0: South
  1: Southwest
  2: West
  3: Northwest
  4: North

Mirrored directions (5-7):
  5: Northeast = Mirror of Dir 3 (Northwest) with X-flip
  6: East = Mirror of Dir 2 (West) with X-flip
  7: Southeast = Mirror of Dir 1 (Southwest) with X-flip
```

## Known Animation Locations in HSPR0-0.DAT

### Shaman Animations (Frame 7578+)

| Start Frame | Description | Frames/Dir | Total Frames |
|-------------|-------------|------------|--------------|
| 7578 | Shaman 1 | 8 | 40 |
| 7618 | Shaman 2 | 8 | ~40 |
| 7658 | Shaman 3 | 8 | ~40 |
| 7698 | Shaman 4 | 8 | ~40 |
| 7738+ | Various spell effects | Variable | Variable |

### Frame Layout for 8-frame Animations

```
For a 40-frame animation (8 frames/dir × 5 dirs):
  Direction 0 (South):     frames 0-7
  Direction 1 (Southwest): frames 8-15
  Direction 2 (West):      frames 16-23
  Direction 3 (Northwest): frames 24-31
  Direction 4 (North):     frames 32-39
```

## Animation Timing

From VFRA analysis, timing values follow this pattern:

### Timing Value Structure (Word1)
- **Low byte**: Frame delay value (most common: 21)
- **High byte**: Appears to be sprite height or rendering flags (26-37 range common)

### Frame Delay Distribution
| Value | Count | Interpretation |
|-------|-------|----------------|
| 21 | 1460 | Base timing (most common) |
| 22 | 290 | Slightly slower |
| 23 | 255 | Slower still |
| 24-27 | ~600 | Medium timing |
| 28+ | ~500 | Slow animations |

### Timing Interpretation

The game likely uses a fixed timestep. Common interpretations:

1. **If timing = game ticks at 15 FPS**:
   - 21 ticks = 1.4 seconds per frame (too slow for walking)

2. **If timing = 256ths of a second** (most likely):
   - 21/256 = 0.082 seconds = 82ms per frame
   - 12.2 frames/second animation rate

3. **Recommended for implementation**:
   - Use ~0.08 seconds per frame for base timing (value 21)
   - Scale proportionally for other values: `delay_seconds = timing_value * 0.004`

### Animation Chain Example
```
Animation at frame 1: [1, 2, 3, 4] with timing [21, 21, 21, 21]
  -> 4 frames at 82ms each = 328ms total loop
  -> ~3 loops per second
```

## Executable Data Tables

The game executable contains hardcoded animation lookup tables:

### Animation Table at 0x0059f8d9
- 11 bytes per animation entry
- Contains: frame_count, flags, timing data, sprite bank indices

### Person Animation Lookup at 0x0059fb30
- 9 animation types × N person types
- Maps (person_type, animation_state) → animation_index

### Animation States
From decompiled Person_SelectAnimation:
- 0: Idle
- 1: Walking
- 2: Special
- 3: Fighting
- 4: Carrying (idle)
- 5: Carrying (walking)
- 0x0c: Swimming
- 0x19: Celebrating

## Implementation Notes

### Animation System Complexity

The full game animation system is multi-layered:
1. **VFRA** uses indices that may exceed HSPR0-0.DAT frame count (up to 16074)
2. **VELE** contains animation elements with sprite references, offsets, and linked lists
3. Multiple sprite banks exist (HSPR0-0.DAT, HSPR0-1.DAT, etc.)

For simplicity, the demo directly uses HSPR0-0.DAT sprite indices for known animations.

### Loading Animations (Simplified Approach)

1. Load sprite bank (HSPR0-0.DAT)
2. Use known frame ranges for specific characters:
   - Shaman 1: frames 7578-7617 (40 frames = 8/dir × 5 dirs)
   - Shaman 2: frames 7618-7657
   - Shaman 3: frames 7658-7697
   - Shaman 4: frames 7698-7737
3. For each frame: decode RLE data, apply palette
4. For mirrored directions (5-7): use source direction with X-flip

### Rendering

1. Get current animation frame from timer
2. Look up direction (0-7)
3. If direction >= 5: map to source direction and set mirror flag
4. Apply X-scale of -1 for mirrored sprites
5. Billboard sprite to face camera

### Known Frame Ranges in HSPR0-0.DAT

| Description | Start Frame | Frames/Dir | Total | Notes |
|-------------|-------------|------------|-------|-------|
| Shaman 1 | 7578 | 8 | 40 | 19x33 pixels |
| Shaman 2 | 7618 | 8 | 40 | 27x31 pixels |
| Shaman 3 | 7658 | 8 | 40 | 23x36 pixels |
| Shaman 4 | 7698 | 8 | 40 | 15x35 pixels |

### Person Type Mapping (from exe analysis)

The exe contains lookup tables mapping (person_type, animation_state) → animation_index:
- `0x0059fb30`: Main animation lookup (9 types × 26 states)
- `0x0059fcf2`: Preaching animations
- `0x0059f8d9`: Animation definitions (11 bytes each)

Person Types:
- 1: Wildman
- 2: Brave
- 3: Warrior
- 4: Preacher
- 5: Spy
- 6: Firewarrior
- 7: Shaman
- 8: Angel of Death

### Known Issues

- VFRA sprite indices >7953 reference elements beyond HSPR0-0.DAT
- The VELE system adds complexity with element offsets and linked lists
- Full animation system requires parsing multiple files and exe tables
- Dimension analysis shows varying frame sizes within same animation

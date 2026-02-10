# Rendering System

## Phase 3: Rendering System

### Game Entry Points

| Address    | Name              | Description                              |
|------------|-------------------|------------------------------------------|
| 0x00562a00 | WinMain           | Windows entry point, calls GameMain      |
| 0x004bc730 | GameMain          | Main game initialization                 |
| 0x004ba520 | GameLoop          | Main game loop with state machine        |
| 0x004ba370 | InitConfigFromRegistry | Loads config from Windows registry  |

### DirectDraw System (0x00510xxx)

| Address    | Name                    | Description                          |
|------------|-------------------------|--------------------------------------|
| 0x00510c70 | DDraw_Create            | Creates DirectDraw object            |
| 0x00510ca0 | DDraw_Initialize        | Initializes DDraw with window        |
| 0x00510e10 | DDraw_RegisterWindowClass | Registers "Bullfrog" window class  |
| 0x00510210 | DDraw_IsInitialized     | Returns if DDraw is ready            |
| 0x00510940 | DDraw_Flip              | Flips front/back buffers             |
| 0x00510a90 | DDraw_BlitRect          | Blits rectangle to surface           |
| 0x00510b70 | DDraw_FlipAndClear      | Flips and clears back buffer         |
| 0x00511e50 | DDraw_RestoreSurface    | Restores lost surface                |
| 0x00511e80 | DDraw_ClearSurface      | Clears surface with color            |
| 0x0041f500 | DDraw_EnumerateDevices  | Enumerates DDraw devices             |

### Texture Loading

| Address    | Name               | Description                              |
|------------|--------------------|------------------------------------------|
| 0x00421320 | LoadLevelTextures  | Loads level terrain textures             |

**Texture Files:**
- `data/plspl0X.dat` - Splash/palette data (X = level theme 0-9, a-b)
- `data/plsft0X.dat` - Font texture data
- `data/plscv0X.dat` - Cover/overlay textures (0x6000 bytes)
- `data/plstxXXX.dat` - Level terrain textures (0x40000 bytes)

### Sky Rendering

See [sky_rendering.md](sky_rendering.md) for full documentation.

| Address    | Name                    | Description                              |
|------------|-------------------------|------------------------------------------|
| 0x00494280 | UI_ClearScreenBuffer    | Entry point — loads skylens, dispatches sky render |
| 0x004dc0e0 | Sky_RenderOrchestrator  | Sets up camera, parallax, dispatches to mode |
| 0x004dcc30 | Sky_RenderTiled         | Mode 0: 16×16 tile perspective renderer  |
| 0x004dd710 | Sky_RenderSimple        | Mode 1: direct scanline copy (512×384)   |
| 0x004dd790 | Sky_RenderParallax      | Mode 2: gradient with dithered palette   |
| 0x004dd880 | Sky_RenderFlatFill      | Mode 3: solid color debug fill           |

### Frame Rendering

| Address    | Name              | Description                              |
|------------|-------------------|------------------------------------------|
| 0x004a6bf0 | DrawFrameRate     | Draws FPS counter on screen              |

### Game State Machine

The main game loop (`GameLoop`) uses `DAT_00877598` as state:
- `0x02` - Frontend/Menu state
- `0x07` - Unknown state
- `0x0A` - Unknown state
- `0x0B` - Unknown state
- `0x0C` - Unknown state

### Key Rendering Globals

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x005aeeb4 | g_DDrawInitialized| DDraw initialization flag      |
| 0x00973ae4 | g_PrimarySurface  | Primary DDraw surface          |
| 0x00973c4c | g_BackBuffer      | Back buffer surface            |
| 0x00973a94 | g_SecondarySurface| Secondary surface              |
| 0x00973adc | g_RenderFlags     | Rendering mode flags           |
| 0x008537b0 | g_FrameTargetTime | Target time for next frame     |
| 0x008537c8 | g_FrameRateLimit  | Frame rate limit timing        |
| 0x008853fa | g_FrameRateValue  | Current frame rate limit (12-60)|
| 0x00877598 | g_GameState       | Current game state             |

### Renamed Functions (Phase 3)

| Address    | Original        | New Name                    |
|------------|-----------------|-----------------------------|
| 0x00562a00 | FUN_00562a00    | WinMain                     |
| 0x004bc730 | FUN_004bc730    | GameMain                    |
| 0x004ba520 | FUN_004ba520    | GameLoop                    |
| 0x004ba370 | FUN_004ba370    | InitConfigFromRegistry      |
| 0x00510c70 | FUN_00510c70    | DDraw_Create                |
| 0x00510ca0 | FUN_00510ca0    | DDraw_Initialize            |
| 0x00510e10 | FUN_00510e10    | DDraw_RegisterWindowClass   |
| 0x00510210 | FUN_00510210    | DDraw_IsInitialized         |
| 0x00510940 | FUN_00510940    | DDraw_Flip                  |
| 0x00510a90 | FUN_00510a90    | DDraw_BlitRect              |
| 0x00510b70 | FUN_00510b70    | DDraw_FlipAndClear          |
| 0x00511e50 | FUN_00511e50    | DDraw_RestoreSurface        |
| 0x00511e80 | FUN_00511e80    | DDraw_ClearSurface          |
| 0x0041f500 | FUN_0041f500    | DDraw_EnumerateDevices      |
| 0x00421320 | FUN_00421320    | LoadLevelTextures           |
| 0x004a6bf0 | FUN_004a6bf0    | DrawFrameRate               |

---

## Extended Analysis

### Combat System

| Address    | Name                          | Description                              |
|------------|-------------------------------|------------------------------------------|
| 0x004c5d20 | Combat_ProcessMeleeDamage     | Calculates and applies melee damage      |
| 0x00438610 | Building_ProcessFightingPersons | Updates units fighting at a building   |
| 0x00423c60 | Game_CheckVictoryConditions   | Checks win/lose conditions each tick     |

**Melee Damage Formula:**
```
damage = (FIGHT_DAMAGE[subtype] * current_health) / max_health
if damage < 0x21: damage = 0x20
if has_bloodlust: damage *= DAT_005a32bc (bloodlust multiplier)
```

### Simulation Tick System

Each game tick processes these subsystems in order:

| Address    | Name                       | Description                              |
|------------|----------------------------|------------------------------------------|
| 0x004a76b0 | Tick_ProcessNetworkMessages | Process incoming network packets         |
| 0x004a6f60 | Tick_ProcessPendingActions  | Process queued player actions            |
| 0x004a7ac0 | Tick_UpdateGameTime         | Advance game time counter                |
| 0x0048bda0 | Tick_UpdateTerrain          | Update terrain modifications             |
| 0x004a7550 | Tick_UpdateObjects          | Update all game objects                  |
| 0x0048bf10 | Tick_UpdateWater            | Update water level and flooding          |
| 0x0041a7d0 | AI_UpdateAllTribes          | Run AI scripts for all tribes            |
| 0x004198f0 | Tick_UpdatePopulation       | Update population spawning               |
| 0x004aeac0 | Tick_UpdateMana             | Update mana generation                   |
| 0x00456500 | Tick_UpdateSinglePlayer     | Single player specific updates           |
| 0x00469320 | Tick_UpdateTutorial         | Tutorial mode specific updates           |

### Network System

| Address    | Name                   | Description                              |
|------------|------------------------|------------------------------------------|
| 0x004e6c40 | Network_SendPacket     | Send network packet via MLDPlay          |
| 0x004e5ad0 | Network_WriteSyncLog   | Write to sync.log for debugging          |
| 0x004e57a0 | Network_OpenSyncLog    | Open sync.log file                       |

**Network Packet Types (byte 0):**
- 0x01-0x05: Compressed packets (RLE compression applied)
- 0x06: Game state sync packet
- 0x07: Heartbeat packet
- 0x0D: Time sync packet
- 0x0E: Pause sync packet

### Renamed Functions (Extended)

| Address    | Original        | New Name                          |
|------------|-----------------|-----------------------------------|
| 0x004c5d20 | FUN_004c5d20    | Combat_ProcessMeleeDamage         |
| 0x00438610 | FUN_00438610    | Building_ProcessFightingPersons   |
| 0x00423c60 | FUN_00423c60    | Game_CheckVictoryConditions       |
| 0x004a76b0 | FUN_004a76b0    | Tick_ProcessNetworkMessages       |
| 0x004a6f60 | FUN_004a6f60    | Tick_ProcessPendingActions        |
| 0x004a7ac0 | FUN_004a7ac0    | Tick_UpdateGameTime               |
| 0x0048bda0 | FUN_0048bda0    | Tick_UpdateTerrain                |
| 0x004a7550 | FUN_004a7550    | Tick_UpdateObjects                |
| 0x0048bf10 | FUN_0048bf10    | Tick_UpdateWater                  |
| 0x004198f0 | FUN_004198f0    | Tick_UpdatePopulation             |
| 0x004aeac0 | FUN_004aeac0    | Tick_UpdateMana                   |
| 0x00456500 | FUN_00456500    | Tick_UpdateSinglePlayer           |
| 0x00469320 | FUN_00469320    | Tick_UpdateTutorial               |
| 0x004e6c40 | FUN_004e6c40    | Network_SendPacket                |
| 0x0042e5f0 | FUN_0042e5f0    | Building_Update                   |
| 0x0049e110 | FUN_0049e110    | Effect_Update                     |
| 0x00458800 | FUN_00458800    | Shot_Update                       |
| 0x004eedc0 | FUN_004eedc0    | Internal_Update                   |
| 0x004afad0 | FUN_004afad0    | Object_UpdateState                |
| 0x004ed510 | FUN_004ed510    | Object_UpdateMovement             |
| 0x0041eb50 | FUN_0041eb50    | LoadConstantsDat                  |

---

## Appendix AL: Rendering System Deep Dive

### Rendering Pipeline Architecture

**Main Entry Points:**

| Function | Address | Purpose |
|----------|---------|---------|
| GameLoop | 0x004ba520 | Main loop with state machine |
| Game_RenderWorld | 0x0048c070 | World rendering entry |
| Render_ProcessCommandBuffer | 0x005125d0 | Command dispatcher |
| DDraw_Flip | 0x00510940 | Page flip/present |
| DDraw_BlitRect | 0x00510a90 | Rectangle blit |

**Rendering Call Hierarchy:**
```
GameLoop() [when g_GameState == 0x07]
├── Game_RenderWorld()
│   ├── FUN_004c3cf0() [Get camera/viewport coords]
│   │   └── Stores to DAT_0096cb18, DAT_0096cb1c
│   └── Render_ProcessCommandBuffer(&PTR_LAB_00599b80, 0, 0)
│       └── Dispatches via function pointer table:
│           - Type 0x01: Standard render ops
│           - Type 0x02: Special effects
│           - Type 0x03: Custom callbacks
│           - Types 0xF0-0xF6: Extended ops
├── Game_RenderEffects() [stub]
├── Game_ProcessInput()
├── Game_UpdateUI()
└── DDraw_Flip()
```

### Bit-Depth Rendering Vtable

**Setup Function:** `Render_SetBitDepthVtable` @ 0x0050f520

```c
void Render_SetBitDepthVtable(surface_info* param_1) {
    DAT_009735d0 = param_1;
    switch(param_1->bitDepth) {  // offset +0x20
        case 8:  DAT_009735b8 = &DAT_009735d8; break;
        case 16: DAT_009735b8 = &DAT_009735e0; break;
        case 24: DAT_009735b8 = &DAT_009735e4; break;
        case 32: DAT_009735b8 = &DAT_009735e8; break;
    }
    FUN_0052b9e0(param_1 + 4);  // Setup color masks
}
```

**Vtable Functions:**
- `*DAT_009735b8[0]`: Pixel plot
- `*DAT_009735b8[0x0C]`: Line draw
- `*DAT_009735b8[0x3C]`: Sprite blit (via Sprite_BlitWithVtable)

### Sprite System

**Sprite Loading:** `Sprite_LoadBank` @ 0x00450990

Loads sprite data from multiple files:
- `data/hspr0-0.dat` - High-res sprites
- `DATA/VSTART-0.ANI` - Animation start frames
- `DATA/VFRA-0.ANI` - Animation frame data
- `DATA/VELE-0.ANI` - Animation element data
- `DATA/VSPR-0.INF` - Sprite info

**Animation Loading:** `Animation_LoadAllData` @ 0x00452530

Parses animation data into runtime structures:
- `DAT_005a7d80` - Animation element array
- `DAT_005a7d84` - Frame data array
- `DAT_005a7d90` - Sprite index array
- `DAT_005a7d88` - Element link array

**Sprite Blitting:** `Sprite_BlitWithVtable` @ 0x005123c0

```c
void Sprite_BlitWithVtable(x, y, sprite_ptr, width, height) {
    // Calls through vtable at offset 0x3C
    (**(code **)(*DAT_009735b8 + 0x3C))(x, y, sprite_ptr, width, height);
}
```

### Animation System

**Person Animation:** `Person_SetAnimation` @ 0x004feed0

Selects animation based on:
- Unit type (offset 0x2b)
- Current state (offset 0x2c)
- Owner tribe (offset 0x2f)
- Vehicle attachment (offset 0x9f)

Animation data tables:
- `DAT_0059f638` - Animation X offset table
- `DAT_0059f63a` - Animation Y offset table
- `DAT_0059fce0` - Mounted unit animations
- `DAT_0059fbae` - Shaman-specific animations

### Coordinate Transformation

**Camera to Screen:**
```c
// From exploration findings
screen_x = (world_x - 0x80 - camera_offset_x) * viewport_width >> 8;
screen_y = viewport_height - ((world_y - camera_offset_y + 0x86) * viewport_height >> 8);
```

**Rotation Transform (fixed-point):**
```c
newX = (x * sin_angle - z * cos_angle) >> 16;
newY = (z * sin_angle + x * cos_angle) >> 16;
```

**Angle System:** 0-2047 values = 0-360° (~0.176° per step)

### DirectDraw Integration

**Surface Globals:**
| Address | Purpose |
|---------|---------|
| DAT_00973ae4 | Primary DDraw surface |
| DAT_00973c4c | Backbuffer surface |
| DAT_009735b8 | Bit-depth vtable pointer |
| DAT_009735d0 | Surface info structure |
| DAT_00973adc | Lock/error status flags |

**DDraw Operations:**
- `DDraw_Flip` @ 0x00510940 - Page flipping
- `DDraw_BlitRect` @ 0x00510a90 - Rectangle copy
- `DDraw_ClearSurface` @ 0x00511e80 - Clear surface
- `DDraw_RestoreSurface` @ 0x00511e50 - Restore lost surface

---

## Appendix AR: Z-Sorting and Draw Order

### Layer-Based Rendering (NOT pure painter's algorithm)

**Core function:** FUN_0047cc80 @ 0x0047cc80

**Layer Priority Order:**
| Priority | Layer ID | Content |
|----------|----------|---------|
| 0 | 0x03 | Terrain/ground base |
| 1 | 0x0b | Ground objects |
| 2 | 0x13 | Buildings/structures |
| 3 | 0x16 | Special structures |
| 4 | 0x10 | Curse/magic effects |
| 5 | 0x1b | Moving creatures/units |
| 6 | 0x21 | Flying effects |
| 7 | 0x22 | Special effects |
| 8 | 0x14 | Spell effects |
| 9 | 0x1e | Highest priority |

### Object Type to Layer Mapping

| Type | Offset +0x2a | Layer |
|------|--------------|-------|
| 0x01 | Warrior/Person | 0x13 (0x1b if moving) |
| 0x02 | Building | 0x03/0x08/0x0b/0x13 |
| 0x04 | Vehicle/Creature | 0x13 |
| 0x05 | Effect/Spell | 0x22/0x07/0x14 |
| 0x06 | Scenery/Trees | 0x16 |
| 0x09 | Conversion | 0x1b |

### Object Linked List

**Traversal (Minimap_RenderObjects):**
```c
for (obj = g_PersonListHead; obj != NULL; obj = obj[1]) {
    // Next pointer at offset +0x20
}
```

### Flying Units

**Detection:** Terrain flags at cell coordinates
**Flag:** DAT_0087e412 |= 0x100000
**Layer:** 0x21 (draws on top)

---

## Appendix AS: Shadow and Lighting System

### Shadow Rendering: Sprite Overlay

**Shadow sprites loaded as "*SHADOW_DUMMY" resources:**
- Address: 0x0059db10, 0x0059e1d0
- Index 0xe in sprite resource table
- Loaded by FUN_0041db20()

### Shadow Offset Calculation (FUN_00416410)

| Object Type | Shadow Offset |
|-------------|---------------|
| Buildings (0x02, subtype 0x13) | 0x140 |
| Creatures (0x09) | 0x40 |
| Others | Based on terrain height |

### No Dynamic Lighting

- No day/night cycle
- No directional lighting
- Static sprite-based shadows
- Shadow length = object type + terrain height

### Terrain Shading (Minimap only)

```c
brightness = terrain_brightness - 0x80;
```

---

## Appendix AT: Render Command Buffer System

### Ring Buffer Structure

**Base:** DAT_00973e98 (~0xF8 bytes per instance)

```c
struct RenderCommandBuffer {
    void*    data_ptr;       // +0x04: Buffer base
    uint32_t capacity;       // +0x08: Ring buffer size
    uint32_t read_pos;       // +0x0C: Read pointer (mod capacity)
    uint32_t pending_count;  // +0x14: Commands waiting
    HANDLE   semaphore;      // +0x1C: Sync handle
};
```

### Command Types

| Type | Code | Size | Purpose |
|------|------|------|---------|
| 1 | 0x01 | 8 bytes | Sprite render with position |
| 2 | 0x02 | 12 bytes | Extended sprite |
| 3 | 0x03 | - | Flush/sync |
| 4 | 0x04 | - | Boundary marker |
| 5 | 0x05 | 11 bytes | Extended render |
| F0-F6 | 0xF0-0xF6 | - | Special operations |

### Command Flow

```
Game Objects
    ↓
Object_SelectForRendering() [0x00411040]
    ↓
Sprite_RenderObject() [0x00411c90]
    ↓
FUN_00512930/005129e0/00512b50() [submit]
    ↓
DAT_00973e98 (ring buffer)
    ↓
Render_ProcessCommandBuffer() [0x005125d0]
    ↓
RenderCmd_ReadNext() [0x00512760]
    ↓
DirectDraw operations
```

### Key Addresses

| Address | Purpose |
|---------|---------|
| 0x00973e98 | Primary command buffer |
| 0x00973e8c | Command count |
| 0x00973a98 | Secondary buffer |
| 0x00599b80 | Processor vtable |

---

---

## Appendix BS: 3D Matrix System

The game uses a **16.14 fixed-point** 3x3 matrix system for all 3D transformations.

### Fixed-Point Format

| Value | Decimal | Description |
|-------|---------|-------------|
| 0x4000 | 1.0 | Unit value |
| 0x2000 | 0.5 | Half |
| 0x8000 | 2.0 | Double |
| >> 14 | ÷16384 | Shift to convert |

### Matrix Storage

The projection matrix is stored at global address range `DAT_006868ac` - `DAT_006868cc`:

```
Matrix Layout (row-major, 9 values × 4 bytes = 36 bytes):
┌────────────────┬────────────────┬────────────────┐
│ DAT_006868ac   │ DAT_006868b0   │ DAT_006868b4   │  Row 0
│ [0,0]          │ [0,1]          │ [0,2]          │
├────────────────┼────────────────┼────────────────┤
│ DAT_006868b8   │ DAT_006868bc   │ DAT_006868c0   │  Row 1
│ [1,0]          │ [1,1]          │ [1,2]          │
├────────────────┼────────────────┼────────────────┤
│ DAT_006868c4   │ DAT_006868c8   │ DAT_006868cc   │  Row 2
│ [2,0]          │ [2,1]          │ [2,2]          │
└────────────────┴────────────────┴────────────────┘
```

### Matrix_SetIdentity @ 0x00450320

Sets matrix to identity (diagonal = 0x4000, rest = 0):

```c
void Matrix_SetIdentity(int* matrix) {
    matrix[0] = 0x4000;  // [0,0] = 1.0
    matrix[1] = 0;       // [0,1] = 0
    matrix[2] = 0;       // [0,2] = 0
    matrix[3] = 0;       // [1,0] = 0
    matrix[4] = 0x4000;  // [1,1] = 1.0
    matrix[5] = 0;       // [1,2] = 0
    matrix[6] = 0;       // [2,0] = 0
    matrix[7] = 0;       // [2,1] = 0
    matrix[8] = 0x4000;  // [2,2] = 1.0
}
```

### Matrix_Multiply3x3 @ 0x004bc060

Multiplies two 3x3 matrices with fixed-point normalization:

```c
void Matrix_Multiply3x3(int* result, int* A, int* B) {
    // Standard 3x3 matrix multiplication
    // Each element: sum of row(A) * col(B)
    // Final normalization: >> 14 (divide by 16384)

    // result[0] = (A[0]*B[0] + A[1]*B[3] + A[2]*B[6]) >> 14
    // result[1] = (A[0]*B[1] + A[1]*B[4] + A[2]*B[7]) >> 14
    // ... etc for all 9 elements
}
```

### Vector_TransformByMatrix @ 0x004bc000

Transforms a 3D point by the matrix (no normalization):

```c
void Vector_TransformByMatrix(int* result, int* matrix, int* point) {
    result[0] = matrix[0]*point[0] + matrix[1]*point[1] + matrix[2]*point[2];
    result[1] = matrix[3]*point[0] + matrix[4]*point[1] + matrix[5]*point[2];
    result[2] = matrix[6]*point[0] + matrix[7]*point[1] + matrix[8]*point[2];
}
```

### Matrix_ApplyYRotation @ 0x004bc1e0

Applies Y-axis rotation to existing matrix:

```c
void Matrix_ApplyYRotation(ushort angle, int* matrix) {
    int cosVal = g_CosTable[angle & 0x7FF] >> 2;  // Scale to 16.14
    int sinVal = g_SinTable[angle & 0x7FF] >> 2;

    // Build Y-rotation matrix:
    // ┌  1     0     0  ┐
    // │  0   cos   sin  │
    // └  0  -cos   sin  ┘
    int rotMatrix[9] = {
        0x4000, 0,      0,       // Row 0 (X unchanged)
        0,      cosVal, sinVal,  // Row 1
        0,     -cosVal, sinVal   // Row 2
    };

    // Save current matrix
    int saved[9];
    memcpy(saved, matrix, 36);

    // matrix = rotMatrix × saved
    Matrix_Multiply3x3(matrix, rotMatrix, saved);
}
```

### Math_RotationMatrix @ 0x004bc360

Builds arbitrary rotation matrix around specified axis:

```c
void Math_RotationMatrix(int* matrix, ushort angle, char axis) {
    // axis: 1=X, 2=Y, 3=Z

    int cosVal = g_CosTable[angle & 0x7FF] >> 2;
    int sinVal = g_SinTable[angle & 0x7FF] >> 2;

    // Rodrigues rotation formula implementation
    // Uses cross-product and normalization
    // Final result is orthonormalized via Math_SqrtApprox
}
```

### g_CameraTarget Structure

The camera state is stored in the global `g_CameraTarget`:

| Offset | Size | Description |
|--------|------|-------------|
| +0x00 | 36 | 3x3 rotation matrix (9 ints) |
| +0x24 | 2 | X-axis rotation angle |
| +0x26 | 2 | Z-axis rotation angle |
| +0x2A | 4 | FOV/scale factor |
| +0x32 | 2 | Y-axis rotation angle |

### Matrix Pipeline

```
1. Matrix_SetIdentity()
      ↓
2. Math_RotationMatrix() - apply X rotation
      ↓
3. Math_RotationMatrix() - apply Z rotation
      ↓
4. Matrix_ApplyYRotation() - apply Y rotation
      ↓
5. Matrix_Multiply3x3() - combine matrices
      ↓
6. Copy to DAT_006868ac (projection matrix)
      ↓
7. Use in Camera_WorldToScreen()
```

---

## Appendix BT: Perspective Projection System

### Camera_WorldToScreen @ 0x0046ea30

Transforms world coordinates to screen space with perspective projection.

**Input/Output:**
- param_1[0-2]: World X, Y, Z (input) → Camera X, Y, Z (output)
- param_1[3-4]: Screen X, Y (output)
- param_1[6]: Clipping flags (output)

### Transformation Algorithm

```c
void Camera_WorldToScreen(int* vertex) {
    int worldX = vertex[0];
    int worldY = vertex[1];
    int worldZ = vertex[2];

    // Step 1: Transform to camera space via matrix multiplication
    // Uses projection matrix at DAT_006868ac
    int camX = (worldX * DAT_006868ac + worldZ * DAT_006868b4) >> 14;
    int camY = (worldY * DAT_006868bc + worldX * DAT_006868b8 + worldZ * DAT_006868c0) >> 14;
    int camZ = (worldY * DAT_006868c8 + worldX * DAT_006868c4 + worldZ * DAT_006868cc) >> 14;

    vertex[0] = camX;
    vertex[1] = camY;
    vertex[2] = camZ;

    // Step 2: Apply curvature correction (spherical world effect)
    // Adjusts Y based on X² + Z² distance from center
    int64_t curvature = (int64_t)(camZ*2*camZ*2 + camX*2*camX*2) * DAT_007b8fb4;
    camY = camY - (int)(curvature >> 32);
    vertex[1] = camY;

    // Step 3: Perspective division
    int depth = DAT_007b8fbc + camZ;  // Add near plane offset

    if (depth < 1) {
        // Behind camera - use large negative values
        screenX = DAT_007b8ffc * -16;
        screenY = DAT_007b8ffe * -16;
    } else {
        // scale = (1 << (FOV_exponent + 16)) / depth
        int scale = (1 << (DAT_007b8fc0 + 16)) / depth;

        // Apply FOV factor from g_CameraTarget
        int fov = *(int*)(g_CameraTarget + 0x2A);

        screenX = ((fov * camX) >> 16) * scale >> 16;
        screenY = ((fov * camY) >> 16) * scale >> 16;
    }

    // Step 4: Convert to screen coordinates
    vertex[3] = DAT_007b8ffc + screenX;  // Center X + offset
    vertex[4] = DAT_007b8ffe - screenY;  // Center Y - offset (Y flipped)

    // Step 5: Set clipping flags
    if (vertex[3] < 0) {
        vertex[6] |= 0x02;  // Clip left
    } else if (vertex[3] >= DAT_007b8fe8) {
        vertex[6] |= 0x04;  // Clip right
    } else if (vertex[4] < 0) {
        vertex[6] |= 0x08;  // Clip top
    } else if (vertex[4] >= DAT_007b8fea) {
        vertex[6] |= 0x10;  // Clip bottom
    }
}
```

### Perspective Globals

| Global | Value | Description |
|--------|-------|-------------|
| DAT_007b8fc0 | 0x0B | FOV exponent (affects perspective strength) |
| DAT_007b8fbc | 0x1964 | Near plane offset (6500 units) |
| DAT_007b8fb4 | 0xB3B0 | Curvature scale (46000) |
| DAT_007b8ffc | varies | Screen center X |
| DAT_007b8ffe | varies | Screen center Y |
| DAT_007b8fe8 | varies | Screen width |
| DAT_007b8fea | varies | Screen height |

### Clipping Flags

| Flag | Value | Meaning |
|------|-------|---------|
| CLIP_LEFT | 0x02 | Vertex left of screen |
| CLIP_RIGHT | 0x04 | Vertex right of screen |
| CLIP_TOP | 0x08 | Vertex above screen |
| CLIP_BOTTOM | 0x10 | Vertex below screen |

### Perspective Division Formula

```
scale = 2^(FOV_exponent + 16) / (near_plane + camera_z)

screen_x = center_x + (fov_factor * camera_x / 65536) * scale / 65536
screen_y = center_y - (fov_factor * camera_y / 65536) * scale / 65536
```

The division by depth creates the perspective effect where distant objects appear smaller.

---

## Appendix BU: Vertex Lighting System

### Overview

The terrain uses per-vertex lighting with two components:
1. **Base brightness** - calculated from heightmap data
2. **Distance attenuation** - fades brightness with depth

### Terrain Vertex Structure

| Offset | Size | Description |
|--------|------|-------------|
| +0x00 | 4 | World X position |
| +0x04 | 4 | World Y (height) |
| +0x08 | 4 | World Z position |
| +0x0C | 4 | Screen X |
| +0x10 | 4 | Screen Y |
| +0x14 | 4 | Brightness (0-63) |
| +0x18 | 4 | Flags |

### Vertex Flags

| Flag | Value | Meaning |
|------|-------|---------|
| SPECIAL_TERRAIN | 0x40 | Water/special terrain cell |
| HEIGHT_GENERATED | 0x80 | Height from lookup, not heightmap |

### Base Brightness Calculation

In `Terrain_GenerateVertices @ 0x0046dc10`:

```c
// Get height index from cell position
int heightIndex = CalculateHeightIndex(cellX, cellY);

// Lookup brightness from two tables (at DAT_00599ac8)
// Uses sun direction factor (DAT_00885720)
int sunDir = DAT_00885720 & 0xFF;
int tableIndex = heightIndex * 8;

int brightness1 = LookupTable[sunDir * 0x101 + tableIndex];
int brightness2 = LookupTable[sunDir * -0x101 + tableIndex + 0x4C];

// Combine and normalize
int rawBrightness = brightness1 + brightness2;
vertex->height = rawBrightness >> 3;
vertex->brightness = (rawBrightness >> 4) + 0x10;

// Clamp to valid range
if (vertex->brightness == 0) vertex->brightness = 1;
if (vertex->brightness > 0x3F) vertex->brightness = 0x3F;
```

### Distance Attenuation

In `Terrain_CreateTriangleCommand @ 0x0046f6f0`:

```c
// For each vertex of the triangle:
int distance = vertex->depth - DAT_008853dd;  // Shading threshold

if (distance > 0) {
    // Quadratic falloff: attenuation = distance² × multiplier
    int64_t distSq = (int64_t)distance * distance;
    int attenuation = (distSq >> 16) * (DAT_008853d9 << 8);
    attenuation = attenuation >> 16;

    // Clamp attenuation
    if (attenuation < 0) attenuation = 0;
    if (attenuation > 0x20) attenuation = 0x20;

    // Apply to brightness
    finalBrightness = vertex->brightness - attenuation;
}
```

### Shading Constants

| Global | Address | Purpose |
|--------|---------|---------|
| DAT_008853d9 | 0x008853d9 | Attenuation multiplier |
| DAT_008853da | 0x008853da | Secondary attenuation (for generated heights) |
| DAT_008853dd | 0x008853dd | Distance threshold (shading starts here) |
| DAT_008853e1 | 0x008853e1 | Secondary threshold |
| DAT_00885720 | 0x00885720 | Sun direction index |
| DAT_00599ac8 | 0x00599ac8 | Brightness lookup table |

### Special Height Handling

Vertices with flag 0x80 (HEIGHT_GENERATED) use a different attenuation formula:

```c
if (vertex->flags & 0x80) {
    int distance = vertex->depth - DAT_008853e1;

    if (distance > 0 && vertex->brightness >= 0x21) {
        int64_t distSq = (int64_t)distance * distance;
        int factor = (distSq >> 16) * (DAT_008853da << 5);
        factor = factor >> 16;

        // Blend toward base brightness
        finalBrightness = (vertex->brightness - 0x20) * factor + 0x20;

        // Clamp
        if (finalBrightness < 0) finalBrightness = 0;
        if (finalBrightness > 0x3F) finalBrightness = 0x3F;
    }
}
```

### Triangle Command Shading Storage

The final brightness values are stored in the triangle command at:
- Offset 0x16: Vertex 1 shading (left-shifted by 16)
- Offset 0x2A: Vertex 2 shading (left-shifted by 16)
- Offset 0x3E: Vertex 3 shading (left-shifted by 16)

---

## Appendix BV: Hybrid Rendering Architecture

### Overview

Populous: The Beginning uses a **hybrid rendering architecture**:

| Component | Technique | Why |
|-----------|-----------|-----|
| Terrain | 3D mesh with perspective | Smooth height transitions, proper depth |
| Units/Buildings | 2D pre-rendered sprites | Performance, consistent visual style |
| Effects | 2D sprites with particles | Simplicity, artistic control |
| Water | Animated 3D mesh | Matches terrain seamlessly |

### Why Hybrid?

1. **Performance**: True 3D models for hundreds of units would be prohibitive on 1998 hardware
2. **Visual Consistency**: Pre-rendered sprites ensure consistent quality
3. **Animation**: Frame-based sprite animation is cheaper than skeletal animation
4. **LOD for free**: Sprites work at any distance without LOD systems

### Terrain Pipeline (True 3D)

```
Heightmap Data (g_Heightmap)
        ↓
Terrain_GenerateVertices() - create vertex grid
        ↓
Calculate per-vertex brightness from lookup tables
        ↓
Vertex_ApplyTransform() - world → camera space
        ↓
Camera_WorldToScreen() - camera → screen with perspective
        ↓
Terrain_CreateTriangleCommand() - build triangle with shading
        ↓
Insert into depth bucket (linked list)
        ↓
Render front-to-back via rasterizer
```

### Object Pipeline (2D Sprites)

```
Object_SelectForRendering() @ 0x00411040
        ↓
Check object type (person, building, shape, effect)
        ↓
Get world position → simple Y-based depth
        ↓
Sprite_RenderObject() @ 0x00411c90
        ↓
Look up animation frame from bank
        ↓
Animation_RenderFrameSequence() @ 0x004e7190
        ↓
Sprite_BlitStandard() or Sprite_BlitScaled()
        ↓
Draw to screen buffer (no 3D transform)
```

### Depth Sorting

**Terrain triangles**: Use camera-space Z depth, sorted into 0xE00 (3584) buckets

```c
// From Terrain_CreateTriangleCommand:
int depth = (v1.z + v2.z + v3.z + 0x15000) * 0x55 >> 8;
if (any_vertex_is_special) depth += 0x100;  // Water/special boost
int bucket = depth >> 4;  // Clamp to 0-0xE00
```

**Sprites**: Use world Y position converted to screen Y for ordering

### Object Types and Renderers

| Type Value | Type | Renderer |
|------------|------|----------|
| 0x01 | Person (unit) | Sprite animation |
| 0x02 | Building | Static sprite |
| 0x03 | Tree/scenery | Static sprite |
| 0x04 | Shot/projectile | Sprite animation |
| 0x05 | Spell effect | Particle sprite |
| 0x09 | Shape (3D model) | shapes.dat mesh |

Note: Type 0x09 (Shape) is one of the few object types that uses actual 3D model data from shapes.dat, similar to terrain. These are used for landmarks and special objects.

### Rendering Order

1. Clear depth buckets
2. Generate terrain vertices
3. Build terrain triangle commands
4. Insert triangles into depth buckets
5. For each bucket (back to front):
   - Render terrain triangles
   - Render sprites at this depth
6. Render UI overlay

### Implications for Modern Reimplementation

A Bevy/modern engine reimplementation should:

1. **Terrain**: Use a 3D mesh with vertex colors for lighting
   - Apply the same matrix transforms
   - Use perspective camera matching original FOV

2. **Units/Buildings**: Two valid approaches:
   - **Faithful**: Extract original sprite sheets, render as billboards sorted by depth
   - **Enhanced**: Create 3D models but match original camera angles

3. **Depth Management**:
   - Terrain uses actual Z-buffer
   - Sprites need Y-based depth sorting or material-based sorting

4. **Lighting**:
   - Terrain: Bake into vertex colors using original lookup tables
   - Sprites: Original game has pre-baked lighting in sprite art

### Key Functions Summary

| Function | Address | Purpose |
|----------|---------|---------|
| Matrix_SetIdentity | 0x00450320 | Initialize identity matrix |
| Matrix_Multiply3x3 | 0x004bc060 | Multiply two 3x3 matrices |
| Vector_TransformByMatrix | 0x004bc000 | Transform point by matrix |
| Matrix_ApplyYRotation | 0x004bc1e0 | Apply Y-axis rotation |
| Math_RotationMatrix | 0x004bc360 | Build rotation matrix |
| Camera_WorldToScreen | 0x0046ea30 | Full 3D projection |
| Terrain_GenerateVertices | 0x0046dc10 | Create terrain vertex grid |
| Terrain_CreateTriangleCommand | 0x0046f6f0 | Build depth-sorted triangle |
| Terrain_RenderWithMatrix | 0x00473bd0 | Render terrain with matrix |
| Terrain_CheckCellFlags | 0x0046e040 | Check cell properties |
| Vertex_ApplyTransform | 0x0046ebd0 | Transform single vertex |
| Object_SelectForRendering | 0x00411040 | Select objects to draw |
| Sprite_RenderObject | 0x00411c90 | Render sprite for object |
| Animation_RenderFrameSequence | 0x004e7190 | Animate sprite sequence |

---

## Appendix BW: Spherical World Projection

### The "Planet" Illusion

The game appears to show a spherical planet with a curved horizon. **This is an optical illusion.** The actual world is:

| Perceived | Actual |
|-----------|--------|
| Spherical planet | Flat 256×256 heightmap |
| Curved horizon | Vertices displaced by distance² |
| Camera orbiting | World rotates around viewer |
| Continuous surface | Toroidal topology (edges wrap) |

### Toroidal World Topology

The world wraps at boundaries (256 cells in each direction):

**Math_CalculateWrappedDistanceSquared** calculates distances on the torus:

```c
// If difference > 128, use shorter wrapped path
if (diff > 0x80) {
    diff = 0x100 - diff;  // Wrap to other side
}
```

Walking east continuously brings you back to where you started. Same for north/south. The world topology is a **torus** (donut shape), not a sphere.

### Curvature Distortion

In `Camera_WorldToScreen @ 0x0046ea30`:

```c
// Calculate distance² from screen center
dist_sq = (cam_z * 2) * (cam_z * 2) + (cam_x * 2) * (cam_x * 2);

// Apply curvature using constant 0xB3B0
curvature = (dist_sq * DAT_007b8fb4) >> 32;  // DAT_007b8fb4 = 0xB3B0

// Pull vertices DOWN based on distance
adjusted_cam_y = cam_y - curvature;
```

**Effect:**
- Vertices at screen center: unchanged
- Vertices at edges: pulled downward
- Creates appearance of terrain curving away (like a horizon)

### Projection Constants

Initialized in `Projection_InitializeDefaults @ 0x0046ed30`:

| Global | Address | Default Value | Purpose |
|--------|---------|---------------|---------|
| DAT_007b8fb4 | 0x007b8fb4 | **46000** | Curvature scale (spherical distortion strength) |
| DAT_007b8fbc | 0x007b8fbc | **0x1964 (6500)** | Near plane / Z offset |
| DAT_007b8fc0 | 0x007b8fc0 | **0x0B (11)** | FOV exponent (perspective strength) |
| DAT_007b8fb8 | 0x007b8fb8 | **0x50 (80)** | Clip distance factor |
| DAT_007b8fcc | 0x007b8fcc | **0x20 (32)** | Projection LUT parameter |

**Note**: These can be overridden by `Projection_SetFromParams @ 0x0046f490` which reads values from a parameter structure.

The curvature constant (46000) controls how strongly the spherical effect is applied. Higher values = more curved horizon.

### World Rotation vs Camera Orbit

When Q/E is pressed to rotate the view:

**Camera_ApplyRotation** transforms world coordinates:

```c
angle = *(short*)(g_CameraTarget + 0x32);  // Y rotation angle
cos_val = g_CosTable[angle & 0x7FF];
sin_val = g_SinTable[angle & 0x7FF];

// World coordinates are rotated, not camera
x' = x * cos - z * sin;
z' = x * sin + z * cos;
```

The camera stays fixed; the entire world rotates around the viewer's position.

### Circular Viewport

After rotation, coordinates are clamped to range [1, 220] from center, creating a **circular visible region** (radius ~110 cells). This reinforces the planetary sphere illusion.

### Implementation Summary

To recreate this effect in a modern engine:

1. Use flat 256×256 heightmap terrain
2. Implement toroidal coordinate wrapping for edge cases
3. Apply vertex shader: `y -= (x² + z²) × curvature_scale`
4. Rotate world coordinates (not camera) for view rotation
5. Apply circular viewport mask

---

## Appendix BX: Complete Terrain Rendering Pipeline

### Pipeline Overview

The terrain rendering system processes terrain data through several stages:

```
Terrain_RenderOrchestrator @ 0x0046ac90
    │
    ├─→ Camera_SetupProjection @ 0x0046edb0
    │   ├─→ Copy g_CameraTarget to projection matrix (DAT_006868ac)
    │   ├─→ Clear 0xE01 depth buckets (DAT_00699a64)
    │   └─→ Initialize viewport bounds
    │
    ├─→ Terrain_GenerateVertices @ 0x0046dc10 (multiple passes)
    │   └─→ For each visible cell:
    │       ├─→ Read height from g_Heightmap
    │       ├─→ Calculate brightness from lookup table (DAT_00599ac8)
    │       ├─→ Set vertex flags (0x40=water, 0x80=generated)
    │       └─→ Call Vertex_ApplyTransform (applies curvature!)
    │
    ├─→ Terrain_GenerateTriangles @ 0x0046e0f0
    │   └─→ For each cell quad:
    │       ├─→ Check g_CellFlags bit 0 for split direction
    │       ├─→ Call Terrain_CheckBackfaceCull for each triangle
    │       └─→ Call Terrain_CreateTriangleCommand for visible triangles
    │
    └─→ FUN_0046af00 (Depth Bucket Renderer)
        └─→ For each bucket (0xE00 down to 0):
            ├─→ Process terrain triangles (type 0x00)
            ├─→ Process sprite objects (type 0x01, 0x0D, 0x1A)
            ├─→ Process 3D model faces (type 0x06)
            └─→ Process special effects (types 0x04-0x1F)
```

### Terrain Vertex Structure (32 bytes)

From `Terrain_GenerateVertices @ 0x0046dc10`:

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| +0x00 | 4 | world_x | World X position (fixed-point) |
| +0x04 | 4 | world_y | World Y / Height |
| +0x08 | 4 | world_z | World Z position |
| +0x0C | 4 | screen_x | Projected screen X |
| +0x10 | 4 | screen_y | Projected screen Y |
| +0x14 | 4 | brightness | Vertex brightness (0-63) |
| +0x18 | 4 | flags | Vertex flags |

**Flag values:**
- `0x40` - Water surface vertex
- `0x80` - Height was procedurally generated (not from heightmap)
- `0x02` - Left clip flag
- `0x04` - Right clip flag
- `0x08` - Top clip flag
- `0x10` - Bottom clip flag

### Triangle Command Structure (0x46 = 70 bytes)

From `Terrain_CreateTriangleCommand @ 0x0046f6f0`:

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| +0x00 | 1 | type | Command type (0=terrain, 6=3D face) |
| +0x01 | 1 | subtype | Subtype/flags |
| +0x02 | 4 | next | Linked list pointer to next command |
| +0x06 | 4 | v1_x | Vertex 1 screen X |
| +0x0A | 4 | v1_y | Vertex 1 screen Y |
| +0x0E | 4 | v1_u | Vertex 1 texture U |
| +0x12 | 4 | v1_v | Vertex 1 texture V |
| +0x16 | 4 | v1_shade | Vertex 1 shading value (<<16) |
| +0x1A | 4 | v2_x | Vertex 2 screen X |
| +0x1E | 4 | v2_y | Vertex 2 screen Y |
| +0x22 | 4 | v2_u | Vertex 2 texture U |
| +0x26 | 4 | v2_v | Vertex 2 texture V |
| +0x2A | 4 | v2_shade | Vertex 2 shading value (<<16) |
| +0x2E | 4 | v3_x | Vertex 3 screen X |
| +0x32 | 4 | v3_y | Vertex 3 screen Y |
| +0x36 | 4 | v3_u | Vertex 3 texture U |
| +0x3A | 4 | v3_v | Vertex 3 texture V |
| +0x3E | 4 | v3_shade | Vertex 3 shading value (<<16) |
| +0x42 | 2 | cell_index | Cell index for texture lookup |
| +0x44 | 1 | tri_type | Triangle orientation (0-3) |
| +0x45 | 1 | material | Material/texture ID |

### Backface Culling

From `Terrain_CheckBackfaceCull @ 0x0046e870`:

```c
// 2D cross product determines facing direction
int Terrain_CheckBackfaceCull(int* v1, int* v2, int* v3) {
    // First check if any vertex is fully clipped
    // (returns 0 if all 3 vertices share any clip flag)

    // Then calculate signed area (2D cross product)
    int result = (v3[4] - v2[4]) * (v2[3] - v1[3]) +  // (v3.y - v2.y) * (v2.x - v1.x)
                 (v3[3] - v2[3]) * (v1[4] - v2[4]);   // (v3.x - v2.x) * (v1.y - v2.y)

    return result;  // Positive = front-facing
}
```

### Depth Bucket System

Depth bucket calculation from `Terrain_CreateTriangleCommand`:

```c
// Average depth of 3 vertices + offset
int depth = (v1.z + v2.z + v3.z + 0x15000) * 0x55 >> 8;

// Water triangles get boosted depth
if ((v1.flags & 0x40) || (v2.flags & 0x40) || (v3.flags & 0x40)) {
    depth += 0x100;
}

// Clamp to valid bucket range
int bucket = depth >> 4;
if (bucket < 0) bucket = 0;
if (bucket > 0xE00) bucket = 0xE00;

// Insert into linked list (painter's algorithm)
triangle->next = depth_buckets[bucket];
depth_buckets[bucket] = triangle;
```

**Bucket range:** 0x000 to 0xE00 (0 to 3584)
**Total buckets:** 0xE01 (3585)

### Per-Vertex Lighting

**Brightness calculation from heightmap:**
```c
// From Terrain_GenerateVertices
int height_index = ((cell_x >> 4) | ((cell_z >> 4) << 8)) * 8;
int lookup1 = DAT_00599ac8[height_index + (DAT_00885720 & 0xFF) * 0x101];
int lookup2 = DAT_00599ac8[height_index + 0x4C + (DAT_00885720 & 0xFF) * -0x101];
int brightness = (lookup1 + lookup2) >> 4 + 0x10;

// Clamp to valid range [1, 63]
if (brightness == 0) brightness = 1;
if (brightness > 0x3F) brightness = 0x3F;
```

**Distance-based shading attenuation:**
```c
// From Terrain_CreateTriangleCommand
int dist_factor = vertex_depth - DAT_008853dd;  // Threshold distance
if (dist_factor > 0) {
    // Square falloff
    int attenuation = ((dist_factor * dist_factor) >> 16) * (DAT_008853d9 << 8) >> 16;

    // Clamp attenuation
    if (attenuation < 0) attenuation = 0;
    if (attenuation > 0x20) attenuation = 0x20;

    final_shade = vertex_brightness - attenuation;
}
```

### Texture UV Coordinate Tables

UV coordinates are pre-computed in lookup tables:

| Table | Address | Purpose |
|-------|---------|---------|
| DAT_0059c190 | 0x0059c190 | Standard terrain UVs (24 bytes per material) |
| DAT_0059c010 | 0x0059c010 | Alternative terrain UVs |
| DAT_0059c0d0 | 0x0059c0d0 | Special terrain UVs |

Each material entry contains 6 UV pairs (for 2 triangles):
```c
struct TerrainUVs {
    int u1, v1;  // Vertex 1
    int u2, v2;  // Vertex 2
    int u3, v3;  // Vertex 3
};
```

### Triangle Split Direction

Cell flag bit 0 determines how the quad is split into triangles:

```
Bit 0 = 0:             Bit 0 = 1:
+---+                  +---+
|\ 1|                  |1 /|
| \ |                  | / |
|0 \|                  |/ 0|
+---+                  +---+
```

Triangle indices:
- Bit 0 = 0: Tri0 = (v0, v1, v3), Tri1 = (v3, v2, v0)
- Bit 0 = 1: Tri0 = (v2, v0, v1), Tri1 = (v1, v3, v2)

---

## Appendix BY: 3x3 Matrix System

### Matrix Storage

The 3x3 rotation matrix is stored at `DAT_006868ac`:

```c
// 9 values, 4 bytes each = 36 bytes total
int matrix[3][3] = {
    { DAT_006868ac, DAT_006868b0, DAT_006868b4 },  // Row 0
    { DAT_006868b8, DAT_006868bc, DAT_006868c0 },  // Row 1
    { DAT_006868c4, DAT_006868c8, DAT_006868cc }   // Row 2
};
```

**Fixed-point format:** 16.14 (0x4000 = 1.0)

### Matrix_SetIdentity @ 0x00450320

```c
void Matrix_SetIdentity(int* m) {
    m[0] = 0x4000;  m[1] = 0;       m[2] = 0;
    m[3] = 0;       m[4] = 0x4000;  m[5] = 0;
    m[6] = 0;       m[7] = 0;       m[8] = 0x4000;
}
```

### Matrix_Multiply3x3 @ 0x004bc060

```c
void Matrix_Multiply3x3(int* result, int* a, int* b) {
    // Standard 3x3 matrix multiplication
    result[0] = (a[0]*b[0] + a[1]*b[3] + a[2]*b[6]) >> 14;
    result[1] = (a[0]*b[1] + a[1]*b[4] + a[2]*b[7]) >> 14;
    result[2] = (a[0]*b[2] + a[1]*b[5] + a[2]*b[8]) >> 14;
    result[3] = (a[3]*b[0] + a[4]*b[3] + a[5]*b[6]) >> 14;
    result[4] = (a[3]*b[1] + a[4]*b[4] + a[5]*b[7]) >> 14;
    result[5] = (a[3]*b[2] + a[4]*b[5] + a[5]*b[8]) >> 14;
    result[6] = (a[6]*b[0] + a[7]*b[3] + a[8]*b[6]) >> 14;
    result[7] = (a[6]*b[1] + a[7]*b[4] + a[8]*b[7]) >> 14;
    result[8] = (a[6]*b[2] + a[7]*b[5] + a[8]*b[8]) >> 14;
}
```

### Vector_TransformByMatrix @ 0x004bc000

```c
void Vector_TransformByMatrix(int* result, int* matrix, int* vector) {
    result[0] = matrix[0]*vector[0] + matrix[1]*vector[1] + matrix[2]*vector[2];
    result[1] = matrix[3]*vector[0] + matrix[4]*vector[1] + matrix[5]*vector[2];
    result[2] = matrix[6]*vector[0] + matrix[7]*vector[1] + matrix[8]*vector[2];
}
```

### Matrix_ApplyYRotation @ 0x004bc1e0

```c
void Matrix_ApplyYRotation(ushort angle, int* matrix) {
    int temp[9];
    // Copy matrix to temp

    int cos_val = g_CosTable[angle & 0x7FF] >> 2;
    int sin_val = g_SinTable[angle & 0x7FF] >> 2;

    // Build Y rotation matrix
    int rot[9] = {
        0x4000, 0,        0,
        0,      cos_val,  0,
        0,      -cos_val, sin_val
    };

    Matrix_Multiply3x3(matrix, rot, temp);
}
```

### Math_RotationMatrix @ 0x004bc360

Builds a rotation matrix around an arbitrary axis:

```c
void Math_RotationMatrix(int* matrix, short angle, char axis) {
    // axis: 1=X, 2=Y, 3=Z
    int cos_val = g_CosTable[angle & 0x7FF] >> 2;
    int sin_val = g_SinTable[angle & 0x7FF] >> 2;

    // Apply Rodrigues' rotation formula
    // (Complex calculation involving axis alignment and re-orthogonalization)
}
```

---

## Appendix BZ: Vertex Transformation Pipeline

### Vertex_ApplyTransform @ 0x0046ebd0

This is the core vertex transformation function:

```c
void Vertex_ApplyTransform(int* vertex) {
    int world_x = vertex[0];
    int world_y = vertex[1];
    int world_z = vertex[2];

    // 1. Apply 3x3 rotation matrix (16.14 fixed-point)
    int cam_x = (world_x * DAT_006868ac + world_z * DAT_006868b4) >> 14;
    int cam_y = (world_y * DAT_006868bc + world_x * DAT_006868b8 + world_z * DAT_006868c0) >> 14;
    int cam_z = (world_y * DAT_006868c8 + world_x * DAT_006868c4 + world_z * DAT_006868cc) >> 14;

    vertex[0] = cam_x;
    vertex[1] = cam_y;
    vertex[2] = cam_z;

    // 2. Apply spherical curvature (THE KEY FORMULA)
    long long dist_sq = (long long)(cam_z * 2) * (cam_z * 2) +
                        (long long)(cam_x * 2) * (cam_x * 2);
    long long curvature = dist_sq * (long long)DAT_007b8fb4;  // 46000
    int curved_y = cam_y - (int)(curvature >> 32);

    vertex[1] = curved_y;

    // 3. Perspective projection
    if (cam_z + DAT_007b8fbc < 1) {
        // Behind camera - clip to offscreen
        vertex[3] = DAT_007b8ffc * -16;
        vertex[4] = DAT_007b8ffe * -16;
    } else {
        // Perspective division
        int scale = (1 << (DAT_007b8fc0 + 16)) / (cam_z + DAT_007b8fbc);
        int fov = *(int*)(g_CameraTarget + 0x2A);

        int screen_x = ((fov * cam_x >> 16) * scale) >> 16;
        int screen_y = ((fov * curved_y >> 16) * scale) >> 16;

        vertex[3] = DAT_007b8ffc + screen_x;  // Center X + offset
        vertex[4] = DAT_007b8ffe - screen_y;  // Center Y - offset (Y inverted)
    }
}
```

### Camera_WorldToScreen @ 0x0046ea30

Same as Vertex_ApplyTransform but also sets clip flags:

```c
void Camera_WorldToScreen(int* vertex) {
    // ... same transformation as above ...

    // Set clipping flags
    if (screen_x < 0) {
        vertex[6] |= 0x02;  // Left clip
    } else if (screen_x >= DAT_007b8fe8) {
        vertex[6] |= 0x04;  // Right clip
    }

    if (screen_y < 0) {
        vertex[6] |= 0x08;  // Top clip
    } else if (screen_y >= DAT_007b8fea) {
        vertex[6] |= 0x10;  // Bottom clip
    }
}
```

---

## Appendix CA: Render Command Types

The depth bucket renderer processes various command types:

| Type | Name | Description |
|------|------|-------------|
| 0x00 | TERRAIN | Terrain triangle with texture mapping |
| 0x01 | SPRITE_OBJECT | Game object sprite (units, buildings) |
| 0x04 | SPRITE_SIMPLE | Simple positioned sprite |
| 0x05 | SPRITE_POSITIONED | Sprite at specific position |
| 0x06 | 3D_FACE | 3D model face (textured triangle) |
| 0x08 | FLAT_SHADED | Flat shaded triangle |
| 0x09 | BUILDING_GHOST | Building placement ghost |
| 0x0A | HEALTH_BAR | Unit health bar |
| 0x0B | SHADOW | Shadow sprite |
| 0x0C | SELECTION_BOX | Unit selection indicator |
| 0x0D | UNIT_FULL | Full unit with animations |
| 0x0E | LINE | 2D line primitive |
| 0x0F | EFFECT_A | Effect type A |
| 0x10 | EFFECT_B | Effect type B |
| 0x11 | MINIMAP_DOT | Minimap unit dot |
| 0x12 | SCALED_SPRITE | Scaled sprite (depth-based) |
| 0x13 | LIGHTNING | Lightning bolt effect |
| 0x15 | CLICKABLE_RECT | Clickable region for picking |
| 0x16 | UI_ELEMENT_A | UI element type A |
| 0x17 | UI_ELEMENT_B | UI element type B |
| 0x18 | SPELL_EFFECT | Spell visual effect |
| 0x19 | EFFECT_C | Effect type C |
| 0x1A | BUILDING_RENDER | Building rendering |
| 0x1B | WIREFRAME | Wireframe triangle |
| 0x1C | PATH_ARROW | Pathfinding arrow |
| 0x1E | ICON_A | Icon type A |
| 0x1F | ICON_B | Icon type B |

---

## Appendix CD: Global Data Addresses Summary

### Rendering Globals

| Address | Name | Purpose |
|---------|------|---------|
| 0x006868ac | g_ViewMatrix | 3x3 rotation matrix (36 bytes) |
| 0x00686858 | g_CameraTarget | Camera state structure |
| 0x00699a58 | g_TriangleCmdPool | Triangle command memory pool |
| 0x00699a60 | g_TriangleCmdNext | Next free triangle command |
| 0x00699a64 | g_DepthBuckets | Depth bucket linked lists (0xE01 entries) |
| 0x007b8fb4 | g_CurvatureScale | Spherical curvature constant (46000) |
| 0x007b8fbc | g_NearPlane | Near plane distance (6500) |
| 0x007b8fc0 | g_FovShift | FOV bit shift (11) |
| 0x007b8fe8 | g_ViewportWidth | Screen width |
| 0x007b8fea | g_ViewportHeight | Screen height |
| 0x007b8ffc | g_ScreenCenterX | Screen center X |
| 0x007b8ffe | g_ScreenCenterY | Screen center Y |
| 0x007b9110 | g_MinDepthBucket | Minimum used bucket |
| 0x007b911c | g_MaxDepthBucket | Maximum used bucket |

### Terrain Globals

| Address | Name | Purpose |
|---------|------|---------|
| 0x00888978 | g_CellFlags | Cell flags array (256x256) |
| 0x00888980 | g_Heightmap | Terrain height data |
| 0x00599ac8 | g_BrightnessLUT | Brightness lookup table |
| 0x008853d9 | g_ShadeAttenMul | Shading attenuation multiplier |
| 0x008853dd | g_ShadeAttenDist | Shading attenuation distance |

### Texture Globals

| Address | Name | Purpose |
|---------|------|---------|
| 0x00599abc | g_TerrainTexBase | Terrain texture base address |
| 0x00599ac0 | g_OverlayTexBase | Overlay texture base address |
| 0x0059c010 | g_TerrainUV_Alt | Alternative UV table |
| 0x0059c190 | g_TerrainUV_Std | Standard UV table |
| 0x00973640 | g_Palette | 256-color palette (RGBA) |

### Trigonometry Tables

| Address | Name | Size | Format |
|---------|------|------|--------|
| 0x00597980 | g_SinTable | 2048 entries | 16.16 fixed-point |
| 0x00599180 | g_CosTable | 2048 entries | 16.16 fixed-point |

---

## Appendix CE: g_CameraTarget Complete Structure

The camera state structure (accessed via g_CameraTarget pointer) contains all camera/projection parameters:

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| +0x00 | 4 | curvature | Curvature parameter (default 46000) |
| +0x04 | 4 | lut_param | Projection LUT parameter (default 0x20) |
| +0x08 | 4 | reserved1 | Reserved |
| +0x0C | 4 | clip_factor | Clip distance factor (default 0x50) |
| +0x10 | 4 | near_plane | Near plane distance (default 0x1964) |
| +0x14 | 4 | fov_shift | FOV bit shift (default 0x0B) |
| +0x22 | 2 | viewport_x | Viewport X offset |
| +0x24 | 2 | viewport_y | Viewport Y offset |
| +0x26 | 2 | viewport_w | Viewport width |
| +0x28 | 2 | viewport_h | Viewport height |
| +0x2A | 4 | fov_factor | FOV multiplier for perspective |
| +0x2C | 2 | center_x_offset | Screen center X offset |
| +0x2E | 2 | center_y_offset | Screen center Y offset |
| +0x30 | 2 | screen_width | Total screen width |
| +0x32 | 2 | rotation_angle | Camera Y rotation (0-2047) |
| +0x44 | 2 | quad_v0_x | Rotation quad vertex 0 X |
| +0x46 | 2 | quad_v0_y | Rotation quad vertex 0 Y |
| +0x48 | 2 | quad_v1_x | Rotation quad vertex 1 X |
| +0x4A | 2 | quad_v1_y | Rotation quad vertex 1 Y |
| +0x4C | 2 | quad_v2_x | Rotation quad vertex 2 X |
| +0x4E | 2 | quad_v2_y | Rotation quad vertex 2 Y |
| +0x50 | 2 | quad_v3_x | Rotation quad vertex 3 X |
| +0x52 | 2 | quad_v3_y | Rotation quad vertex 3 Y |
| +0x54 | 1 | is_initialized | Camera initialized flag |

### Resolution-Specific Quad Offsets

From `Camera_SetViewportOffsets @ 0x00421c70`:

**800x600:**
```c
quad_v0 = (-14, -16)
quad_v1 = (14, -16)
quad_v2 = (25, 44)
quad_v3 = (-25, 44)
```

**1024x768:**
```c
quad_v0 = (-16, -18)
quad_v1 = (16, -18)
quad_v2 = (30, 49)
quad_v3 = (-30, 49)
```

**1280x1024:**
```c
quad_v0 = (-18, -20)
quad_v1 = (18, -20)
quad_v2 = (32, 54)
quad_v3 = (-32, 54)
```

**Default/512x384:**
```c
quad_v0 = (-10, -16)
quad_v1 = (10, -16)
quad_v2 = (16, 39)
quad_v3 = (-16, 39)
```

### Projection Parameters Initialization

From `Projection_InitializeDefaults @ 0x0046ed30`:

```c
DAT_007b8fb4 = 46000;      // Curvature scale
DAT_007b8fbc = 0x1964;     // Near plane (6500)
DAT_007b8fc0 = 0x0B;       // FOV shift (11)
DAT_007b8fb8 = 0x50;       // Clip factor (80)
DAT_007b8fcc = 0x20;       // LUT parameter (32)
```

### Dynamic Parameter Loading

From `Projection_SetFromParams @ 0x0046f490`:

The function reads parameters from a structure and updates globals:

```c
void Projection_SetFromParams(int* params) {
    DAT_007b8fb4 = params[0];  // Curvature
    DAT_007b8fcc = params[1];  // LUT param
    DAT_007b8fc4 = params[3];  // Additional param
    DAT_007b8fbc = params[4];  // Near plane
    DAT_007b8fc0 = params[5];  // FOV shift

    // Viewport setup
    DAT_007b8fe4 = params[0x22/4];  // Viewport X
    DAT_007b8fe6 = params[0x24/4];  // Viewport Y
    DAT_007b8fe8 = params[0x26/4];  // Viewport W
    DAT_007b8fea = params[0x28/4];  // Viewport H

    // Screen center calculation
    DAT_007b8ffc = params[0x2A/4] + viewport_w / 2;
    DAT_007b8ffe = params[0x2C/4] + viewport_h / 2;
}
```

---

## Appendix CF: Rasterizer Entry Point

### Function Pointer Setup

The rasterizer function pointer is stored at `DAT_00686898` and is set during `Camera_SetupProjection`:

```c
// From Camera_SetupProjection @ 0x0046edb0
if ((DAT_0087e33c & 0x40) != 0) {
    DAT_00686898 = &LAB_00473bb0;  // Alternative rasterizer
} else {
    DAT_00686898 = FUN_0097c000;   // Main software rasterizer
}
```

### Main Rasterizer (FUN_0097c000)

Location: 0x0097c000 - 0x00988fe7 (51,176 bytes)

This is a massive hand-optimized software rasterizer that:
1. Reads triangle vertices from the command buffer
2. Performs edge setup for scanline conversion
3. Applies Gouraud shading interpolation
4. Samples textures with perspective correction
5. Writes pixels to the frame buffer

### Rasterizer Globals

| Address | Name | Purpose |
|---------|------|---------|
| 0x0098a004 | g_TextureBase | Current texture base address |
| 0x0098a008 | g_ShadeIndex | Current shading lookup index |
| 0x0098a00c | g_RasterFlags | Rasterizer state flags |

### Triangle Call Convention

The rasterizer is called with 3-4 parameters:

```c
(*DAT_00686898)(vertex1, vertex2, vertex3, flags);
```

Where each vertex pointer points to a triangle command vertex:
- +0x00: Screen X (4 bytes)
- +0x04: Screen Y (4 bytes)
- +0x08: Texture U (4 bytes)
- +0x0C: Texture V (4 bytes)
- +0x10: Shade value (4 bytes, <<16 format)

---

## Appendix CI: DirectDraw Integration

### Surface Structure

The game uses DirectDraw for display:

| Global | Purpose |
|--------|---------|
| DAT_00973ae4 | Primary surface |
| DAT_00973c4c | Back buffer surface |
| DAT_00973a94 | Intermediate surface (windowed mode) |
| DAT_00973adc | Display mode flags |

### DDraw_Flip @ 0x00510940

Presents the rendered frame:

```c
int DDraw_Flip(uint flags) {
    flags ^= 1;  // Toggle buffer

    if ((display_flags & 2) == 0) {
        // Exclusive fullscreen mode
        if (windowed_mode == 0) {
            // Direct flip
            result = primary_surface->Flip(NULL, NULL, back_buffer, 0, flags << 4);
        } else {
            // Windowed: blit then flip
            intermediate->Flip(NULL, NULL, back_buffer, 0, flags << 4);
            result = primary_surface->Blt(NULL, intermediate, flags);
        }
    } else {
        // Windowed mode with clipping
        GetWindowRect(hwnd, &rect);
        result = primary_surface->Blt(&rect, back_buffer, NULL, DDBLT_WAIT, NULL);
    }

    // Handle lost surfaces
    if (result == DDERR_SURFACELOST) {
        DDraw_RestoreSurface(&surface1);
        DDraw_RestoreSurface(&surface2);
    }

    return (result == DD_OK) ? 0 : -1;
}
```

### Display Mode Flags

| Bit | Value | Meaning |
|-----|-------|---------|
| 1 | 0x02 | Windowed mode |
| 4 | 0x10 | Use GDI blitting |

---

## Appendix CJ: Summary of Key Rendering Constants

### Fixed-Point Formats

| Value | Format | Description |
|-------|--------|-------------|
| 0x4000 | 16.14 | Matrix element 1.0 |
| >> 14 | 16.14 | Matrix/vector normalization |
| >> 16 | 16.16 | General fixed-point |
| >> 8 | 24.8 | UV coordinates |

### Depth Bucket System

| Constant | Value | Purpose |
|----------|-------|---------|
| BUCKET_COUNT | 0xE01 (3585) | Total depth buckets |
| BUCKET_OFFSET | 0x15000 | Base depth offset |
| BUCKET_SCALE | 0x55 | Depth scaling factor |
| WATER_BOOST | 0x100 | Water surface depth boost |

### Viewport Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| VIEWPORT_MIN | 1 | Minimum coordinate after rotation |
| VIEWPORT_MAX | 0xDC (220) | Maximum coordinate |
| CENTER_OFFSET | 0x6E (110) | Rotation center offset |
| WORLD_SIZE | 256 | World cells per axis |

### Curvature System

| Constant | Value | Purpose |
|----------|-------|---------|
| CURVATURE | 46000 | Spherical distortion strength |
| NEAR_PLANE | 0x1964 (6500) | Perspective near plane |
| FOV_SHIFT | 0x0B (11) | Perspective FOV exponent |
| CLIP_FACTOR | 0x50 (80) | Clipping distance |

### Shading System

| Constant | Range | Purpose |
|----------|-------|---------|
| Brightness | 1-63 | Per-vertex brightness value |
| Attenuation | 0-32 | Distance-based dimming |
| Shade output | << 16 | Final shade in 16.16 format |

---

## Appendix CO: Render Command Buffer System

### Command Types

| Function | Purpose |
|----------|---------|
| RenderCmd_AllocateBuffer | Allocate command memory |
| RenderCmd_CheckSpace | Verify buffer has room |
| RenderCmd_WriteData | Write raw data to buffer |
| RenderCmd_WriteSpriteData | Write sprite render data |
| RenderCmd_SubmitSimple | Submit simple command |
| RenderCmd_SubmitComplex | Submit with viewport bounds |
| RenderCmd_SubmitSprite | Submit sprite for rendering |

### RenderCmd_SubmitComplex @ 0x005129e0

```c
int RenderCmd_SubmitComplex(int param1, uint param2, uint param3) {
    RenderCmd_LockBuffer(&cmd_buffer_lock, 0);

    if (RenderCmd_CheckSpace(1) == 0) {
        RenderCmd_UnlockBuffer();
        return -1;
    }

    if (param1 == 0) {
        // Clear command buffer
        RenderCmd_DestroyBuffer();
        DAT_00973ee8 = 0;
        RenderCmd_UnlockBuffer();
        return 0;
    }

    // Clamp parameters to sprite bounds
    uint x = min(param2, *(ushort*)(param1+4) - 1);
    uint y = min(param3, *(ushort*)(param1+6) - 1);

    RenderCmd_AllocateBuffer(&cmd_buffer);

    // Write viewport bounds and data
    FUN_0052df50(&viewport);
    uint bounds = RenderCmd_GetViewportBounds(0);
    RenderCmd_WriteData(bounds);

    RenderCmd_UnlockBuffer();
    return 0;
}
```

### Thread Safety

Commands use critical section locking:
- DAT_00973e98: Command buffer mutex
- RenderCmd_LockBuffer/UnlockBuffer for synchronization

---

## Appendix CR: Software Rasterizer

### Main Rasterizer Function

| Function | Address | Size | Purpose |
|----------|---------|------|---------|
| Rasterizer_Main | 0x0097c000 | ~52KB | Core software triangle rasterizer |
| Terrain_RenderWithMatrix | 0x00473bd0 | ~1KB | Alternative 3D model rasterizer |

The rasterizer at 0x0097c000 spans from 0x0097c000 to 0x00988fe7 - one of the largest functions in the codebase.

### Rasterizer Selection

```c
// From Render_SetupRasterizerCallback @ 0x00429a50
void Render_SetupRasterizerCallback(void) {
    // Default: use main software rasterizer
    DAT_00686898 = Rasterizer_Main;

    // If hardware flag set, use matrix-based rasterizer
    if ((DAT_0087e33c & 0x40) != 0) {
        DAT_00686898 = Terrain_RenderWithMatrix;
    }

    // Initialize render state
    DAT_00699a60 = DAT_00699a58;
    // Clear 10 dwords at DAT_00699a64
    memset(&DAT_00699a64, 0, 40);

    // Set screen center
    DAT_007b8ffc = screen_width / 2;
    DAT_007b8ffe = screen_height / 2;
}
```

### Rasterizer Invocation

From Render_DrawRotatedQuad @ 0x0040a560:

```c
void Render_DrawRotatedQuad(int param) {
    // ... setup vertices ...

    // Set texture base address
    DAT_0098a004 = DAT_005a7d44 + 0x400;
    DAT_0098a008 = 0;  // Shade offset

    // Rasterize two triangles forming a quad
    Rasterizer_Main(v0, v1, v2, 3);  // First triangle
    Rasterizer_Main(v0, v2, v3, 3);  // Second triangle
}
```

### Rasterizer Input Format

Each vertex passed to the rasterizer (20 bytes):

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 4 | Screen X (fixed-point) |
| +0x04 | 4 | Screen Y (fixed-point) |
| +0x08 | 4 | Depth Z (for perspective) |
| +0x0C | 4 | Texture U coordinate |
| +0x10 | 4 | Texture V coordinate |

The 4th parameter is the render mode:
- Mode 3: Standard textured triangle

### Texture/Shading Global State

| Address | Purpose |
|---------|---------|
| DAT_0098a004 | Texture data base pointer |
| DAT_0098a008 | Shade table index offset |

These must be set before calling the rasterizer.

### Terrain_RenderWithMatrix @ 0x00473bd0

Alternative rasterizer for 3D shapes:

```c
void Terrain_RenderWithMatrix(RenderParams* params) {
    // Store rotation angles
    DAT_007b9000 = params->angle_x;
    DAT_007b9002 = params->angle_y;
    DAT_007b8fb8 = params->scale;

    // Initialize identity matrix at 0x6868ac
    Matrix_SetIdentity(0x6868ac);

    // Apply X rotation (negated)
    Math_RotationMatrix(0x6868ac, -params->rotation_x, 2);

    // Apply Y rotation
    Matrix_ApplyYRotation(0x6868ac, params->rotation_y);

    // Transform all vertices
    for (int i = 0; i < vertex_count; i++) {
        // Matrix multiplication (3x3 @ 16.14 fixed-point)
        cam_x = (v.x * m[0] + v.y * m[1] + v.z * m[2]) >> 14;
        cam_y = (v.x * m[3] + v.y * m[4] + v.z * m[5]) >> 14;
        cam_z = (v.x * m[6] + v.y * m[7] + v.z * m[8]) >> 14;

        // Scale and project
        screen_x = (cam_x * scale >> 8) + center_x;
        screen_y = center_y - (cam_y * scale >> 8);
    }

    // Build triangles with backface culling
    for each face {
        if (Terrain_CheckBackfaceCull(v0, v1, v2) > 0) {
            // Calculate depth bucket
            depth = (v0.z + v1.z + v2.z + 0xC00);
            bucket = (depth * 0x55 >> 8);

            // Create triangle command (type 0x06)
            cmd->type = 0x06;
            // ... fill vertex data ...

            // Insert into depth bucket
            cmd->next = buckets[bucket];
            buckets[bucket] = cmd;
        }
    }

    // Process all depth buckets
    Render_ProcessDepthBuckets_3DModels();
}
```

### Animation Data Files

| File | Purpose |
|------|---------|
| DATA/VELE_0.ANI | Animation element data |
| DATA/VSPR_0.INF | Sprite information |
| DATA/VSTART_0.ANI | Animation start frames |
| DATA/VFRA_0.ANI | Animation frame data |

### Animation Buffer Structure

| Address | Purpose |
|---------|---------|
| DAT_005a7d80 | Start frame pointers |
| DAT_005a7d84 | Frame data |
| DAT_005a7d88 | Element data |
| DAT_005a7d8c | Sprite info |
| DAT_005a7d90 | Frame indices |

---

## Summary: Complete Rendering Architecture

### Pipeline Overview

```
1. Game Loop
   ├── Update game state
   ├── Camera_SetupProjection() - Initialize matrices
   │
2. Terrain Rendering
   ├── Terrain_GenerateVertices() - Create vertex grid
   │   └── For each cell: height, brightness, flags
   │       └── Vertex_ApplyTransform() - Apply curvature!
   │
   ├── Terrain_GenerateTriangles() - Build triangle mesh
   │   └── For each quad: 2 triangles
   │       └── Terrain_CreateTriangleCommand()
   │           └── Insert into depth bucket
   │
3. Object Rendering
   ├── For 3D shapes:
   │   └── Terrain_RenderWithMatrix() - Transform and rasterize
   │
   ├── For 2D sprites:
   │   └── Sprite_RenderObject() - Screen-space positioning
   │       └── Animation_RenderFrameSequence()
   │           └── Sprite_BlitStandard/Scaled()
   │
4. Depth Bucket Processing
   ├── Render_ProcessDepthBuckets_Main() - Process all buckets
   │   └── For each bucket (back to front):
   │       └── Rasterizer_Main() - Draw triangles
   │
5. Post-Processing
   ├── Render_PostProcessEffects() - Special effects
   ├── UI rendering
   │
6. Display
   └── DDraw_Flip() - Present frame
```

### Key Insight: Hybrid Renderer

Populous: The Beginning uses a **hybrid rendering approach**:

1. **Terrain**: Full 3D software-rasterized mesh with per-vertex Gouraud shading
2. **3D Objects** (shapes): Matrix-transformed and software-rasterized
3. **2D Sprites**: Pre-rendered isometric sprites with depth sorting
4. **Spherical Illusion**: Vertex Y-displacement creates curved horizon effect

This hybrid design allowed the game to achieve impressive visual results on 1998 hardware while maintaining good performance through extensive use of pre-rendered sprites for game objects.

---

## Appendix CT: Object Render Type Dispatch System

### Object Type Table (DAT_0059f8d8)

Each object has a render type byte at offset +0x3A. This indexes into an 11-byte structure array:

| Render Type | Handler | Description |
|-------------|---------|-------------|
| 1 | Object_SubmitToDepthBucket | Standard animated sprite |
| 2 | Set flag 0x01 | Mark as rendered |
| 3 | Full render + shadow | Unit with shadow |
| 4 | DAT_006868a4 callback | Special render callback |
| 5 | FUN_00470d20 | Building render |
| 7 | FUN_00470e30 | Effect render |
| 8 | FUN_00470f40 | Projectile render |
| 9 | FUN_00470a50 | Spell effect (unless state 8) |
| 10 | Object_SubmitToDepthBucket | Mode 0x0D |
| 11 | FUN_004737c0 | Land formation (state 1 only) |
| 12 | FUN_00471040 | Tree/vegetation render |
| 13 | FUN_00474d60 | Multi-part object (6 parts) |
| 14 | Render parent object | Linked object |
| 15 | FUN_00472330 | Particle system |
| 16 | FUN_00474e80 | Cell boundary overlay |
| 17 | FUN_00470930 | Positioned sprite |
| 18 | FUN_00470b60 | Special effect |
| 19 | Render_SubmitGrassPatches | Grass/foliage patches |

### Render_DispatchObjectsByType @ 0x0046fc30

```c
void Render_DispatchObjectsByType(uint* cell_flags) {
    for (int pass = 0; pass < 2; pass++) {
        for (obj = cell_object_list; obj != NULL; obj = obj->next) {
            if (obj->flags & 0x10) continue;  // Skip if already rendered

            byte render_type = obj->render_type;  // offset 0x3A
            byte dispatch = object_type_table[render_type * 11];

            if (pass == 0) {
                // First pass: distance check for selection
                if ((obj->flags2 & 0x80) != 0) {
                    int dist = GetDistanceToCamera(obj);
                    if (dist < closest_selectable) {
                        closest_selectable = dist;
                    }
                }
            }

            // Dispatch based on render type
            switch (dispatch) {
                case 1: Object_SubmitToDepthBucket(cell_flags, obj); break;
                case 3: RenderWithShadow(obj); break;
                // ... etc
            }

            obj->flags |= 0x01;  // Mark as rendered
        }
    }
}
```

### Object_SubmitToDepthBucket @ 0x00470030

Calculates screen position and inserts object into appropriate depth bucket:

```c
void Object_SubmitToDepthBucket(cell_flags, obj) {
    // Get world position with interpolation
    ushort x = obj->world_x - obj->interp_x;
    ushort y = obj->world_y - obj->interp_y;
    short z = obj->world_z - obj->interp_z;

    // Apply motion interpolation if enabled
    if ((obj->flags2 & 1) && !(game_flags & 2)) {
        int frame_delta = current_frame - obj->last_frame;
        if (frame_delta != 0 && frame_rate != 0) {
            int scale = frame_delta * interpolation_factor;
            x += (obj->interp_x * scale) / frame_rate;
            y += (obj->interp_y * scale) / frame_rate;
            z += (obj->interp_z * scale) / frame_rate;
        }
    }

    // Transform to camera-relative coordinates (with toroidal wrapping)
    int cam_x = WrapCoordinate(x - camera_x) >> 1;
    int cam_y = WrapCoordinate(y - camera_y) >> 1;
    int cam_z = obj->height_offset + z;

    // Project to screen
    Camera_WorldToScreen(&cam_x, &cam_y, &cam_z, &screen_x, &screen_y, &depth);

    if (depth >= 0) {
        // Calculate depth bucket
        int bucket = (cam_y + 0x7000 + z_offset) >> 4;
        bucket = clamp(bucket, 0, 0xE00);

        // Create render command
        cmd = AllocRenderCommand(14 bytes);
        cmd->type = render_mode;  // From DAT_007b903e
        cmd->screen_x = screen_x;
        cmd->screen_y = screen_y;
        cmd->object_ptr = obj;

        // Insert into bucket
        cmd->next = depth_buckets[bucket];
        depth_buckets[bucket] = cmd;
    }
}
```

---

## Appendix CW: Shadow Calculation

### Shadow_CalculateOffset @ 0x00416410

Calculates shadow offset based on object type:

```c
void Shadow_CalculateOffset(int* shadow_params, int object) {
    shadow_params[2] = 0;  // Default no shadow

    if (object->type == PERSON) {
        if (object->subtype == 0x13) {  // Flying unit
            shadow_params[2] = 0x140;   // Large shadow offset
            return;
        }
    }
    else if (object->type == SHAPE) {
        shadow_params[2] = 0x40;  // Fixed shadow for shapes
        return;
    }

    byte render_type = object_type_table[object->render_type * 11];

    if (render_type == 1) {  // Standard sprite
        // Shadow from sprite height
        shadow_params[2] = sprite_data[object->sprite_id].height << 3;
    }
    else if (render_type == 3) {  // Unit with animation
        if (object->flags3 & 0x02) {
            // Animated: scale shadow by animation frame
            int anim_data = object->anim_id * 0x36 + anim_base;
            int scale = *(int*)(anim_data + 0x0C);
            shadow_params[2] = (*(short*)(anim_data + 0x28) / 2) * scale / scale;
        } else {
            // Static: half of sprite height
            shadow_params[2] = *(short*)(object->anim_id * 0x36 + 0x28 + anim_base) / 2;
        }
    }
    else if (render_type == 10) {  // Special animated
        int frame_data = Animation_GetFrameData(object->sprite_id);
        shadow_params[2] = *(short*)(frame_data + 6) << 3;
    }
}
```

---

## Appendix CX: Render Command Types Summary

### Complete Command Type Table

| Type | Size | Description |
|------|------|-------------|
| 0x00 | - | End marker |
| 0x01 | 14 | Flat-shaded triangle |
| 0x02 | 14 | Point sprite |
| 0x03 | 14 | Line |
| 0x04 | 14 | Gouraud-shaded triangle |
| 0x05 | 14 | Textured triangle (affine) |
| 0x06 | 70 | Terrain triangle (full data) |
| 0x07 | 14 | Transparent sprite |
| 0x08 | 68 | Textured triangle with UVs |
| 0x09 | 14 | Particle |
| 0x0A-0x0D | - | Reserved |
| 0x0E | 14 | Object reference |
| 0x0F | 10 | Health bar (type 0) |
| 0x10 | 10 | Health bar (type 1) |
| 0x11-0x19 | 14 | Sprite variants |
| 0x1A | 14 | Selected object highlight |
| 0x1B-0x1D | 14 | UI elements |
| 0x1E | 12 | Grass patch |
| 0x1F | 12 | Ground detail |

### Render Command Base Structure

```c
struct RenderCommand {
    byte type;           // +0x00: Command type
    byte flags;          // +0x01: Render flags
    void* next;          // +0x02: Next command in bucket (4 bytes)
    // Type-specific data follows...
};
```

### Terrain Triangle Command (Type 0x06)

```c
struct TerrainTriangleCmd {
    byte type;           // 0x06
    byte flags;
    void* next;
    // Vertex 0
    int v0_screen_x;     // +0x06
    int v0_screen_y;     // +0x0A
    int v0_u;            // +0x0E
    int v0_v;            // +0x12
    byte v0_shade;       // +0x16
    // Vertex 1
    int v1_screen_x;     // +0x1A
    int v1_screen_y;     // +0x1E
    int v1_u;            // +0x22
    int v1_v;            // +0x26
    byte v1_shade;       // +0x2A
    // Vertex 2
    int v2_screen_x;     // +0x2E
    int v2_screen_y;     // +0x32
    int v2_u;            // +0x36
    int v2_v;            // +0x3A
    byte v2_shade;       // +0x3E
    // Metadata
    short triangle_id;   // +0x42
    byte rotation;       // +0x44
    byte texture_id;     // +0x45
};  // Total: 70 bytes (0x46)
```

---

## Appendix CY: Render Dispatch and Depth Bucket Processing

### Render_ProcessDepthBuckets_Main @ 0x0046af00

This is the **core render command dispatcher** - the heart of the rendering pipeline. It processes all depth buckets from back to front (0xE01 → 0), dispatching each command to the appropriate handler.

#### Main Loop Structure

```c
void Render_ProcessDepthBuckets_Main(void) {
    // Clear selection state
    DAT_007b9026 = 0;
    if ((DAT_00884bf9 & 0x40) == 0) {
        DAT_007b901a = 0;  // No object under cursor
    }

    // Process depth buckets back-to-front
    for (bucket = 0xE01; bucket > 0; bucket--) {
        RenderCmd* cmd = depth_buckets[bucket];

        while (cmd != NULL) {
            switch (cmd->type) {
                case 0x00: ProcessTerrainTriangle(cmd); break;
                case 0x01: ProcessSprite(cmd); break;
                case 0x04: ProcessBuildingSprite(cmd); break;
                case 0x05: ProcessEffectSprite(cmd); break;
                case 0x06: ProcessGouraudTriangle(cmd); break;
                case 0x08: ProcessTexturedTriangle(cmd); break;
                case 0x09: ProcessHealthBar(cmd); break;
                case 0x0A: ProcessMinimapSprite(cmd); break;
                case 0x0D: ProcessUnitSprite(cmd); break;
                // ... 30+ command types
            }
            cmd = cmd->next;
        }
    }
}
```

### Command Type 0x00: Terrain Triangle

The most complex case - handles textured terrain with:
- Texture coordinate lookup from UV tables
- Water animation (sin/cos displacement)
- Terrain type-specific texturing (grass, sand, lava, ice)
- Backface culling test
- Cursor hit-testing

```c
case 0x00:  // Terrain triangle
    DAT_0098a008 = 0x43;  // Shader mode

    // Get triangle cell coordinates
    triangle_id = cmd->triangle_id;  // offset 0x42
    cell_x = (triangle_table[triangle_id * 8] >> 1);
    cell_y = (triangle_table[triangle_id * 8 + 1] >> 1);

    // Calculate texture address based on cell flags
    cell_index = (cell_x & 0xFE) * 2 + (cell_y & 0xFE);
    cell_flags = g_CellFlags[cell_index];

    if (cell_flags & 0x04) {
        // Water cell: animated texture
        texture_addr = DAT_00599acc +
            ((cell_x & 2) + (cell_y & 2) * 4) * 16 +
            (g_GameTick & 1) * 0x80 +
            (g_GameTick & 6) * 0x4000;

        // Apply sin/cos displacement to UVs
        int phase = (g_GameTick & 0x3F) * 0x80;
        cmd->v0_u += g_CosTable[(cell_x * 0x200 + phase) & 0x7FF] * 8 + 0x200000;
        cmd->v0_v += g_SinTable[(cell_y * 0x200 + phase) & 0x7FF] * 8 + 0x200000;
        // Similar for v1, v2...
    }
    else {
        // Standard terrain texture
        texture_addr = DAT_00599abc +
            (cell_x & 0xE0) * 0x800 +
            (cell_y & 0xE0) * 0x2000 +
            (cell_x & 0x1F) * 8 +
            (cell_y & 0x1F) * 0x800;
    }

    // Call rasterizer
    DAT_00686898(cmd + 6, cmd + 0x1A, cmd + 0x2E);

    // Backface test for cursor picking
    if (PointInTriangle(cursor_x, cursor_y, v0, v1, v2)) {
        // Store terrain info for cursor
        DAT_007b9010 = cell_x;
        DAT_007b9011 = cell_y;
        DAT_007b9042 = cmd->rotation;
    }
    break;
```

### Command Type 0x01/0x0D/0x1A: Sprite Commands

Handles animated sprites for units, buildings, and effects:

```c
case 0x01:  // Standard sprite
case 0x0D:  // Unit sprite
case 0x1A:  // Selection highlight
    object_ptr = *(cmd + 6);  // Object pointer

    // Get sprite animation frame
    if (object_render_table[object->render_type * 11 + 1] < 2) {
        frame_index = object->sprite_id;
    } else {
        frame_index = object->sprite_id + (object->anim_progress >> 2);
    }

    // Get sprite dimensions from animation data
    sprite_data = sprite_table + frame_index * 8;
    width = sprite_data[4];
    height = sprite_data[6];

    // Screen position
    screen_x = cmd->screen_x - width / 2;
    screen_y = cmd->screen_y - height;

    // Distance scaling (for zoom)
    if ((DAT_00884c01 & 0x380) != 0) {
        FUN_00477420(bucket);  // Calculate scale factor
    }

    // Handle special effects
    if (object->effect_color >= -15) {
        DAT_00973610 = effect_palettes + object->effect_color * 0x1000;
        DAT_009735dc |= 8;  // Enable palette effect
    }

    // Blit sprite
    if (scaled) {
        Sprite_BlitScaled(screen_x, screen_y, sprite_data, width, height);
    } else {
        Sprite_BlitStandard(screen_x, screen_y, sprite_data);
    }

    // Cursor hit test
    if (cursor within sprite bounds) {
        DAT_007b901a = object->id;  // Object under cursor
    }
    break;
```

### Command Type 0x06: Gouraud-Shaded Triangle (3D Models)

For 3D model triangles:

```c
case 0x06:
    switch (cmd->shade_mode) {  // offset 0x45
        case 0x00:  // Skip
            break;

        case 0x01:  // Flat shaded
            DAT_0098a008 = palette_lookup[cmd->shade_value * 256 + cmd->material_id];
            break;

        default:  // Gouraud shaded
            // Shift brightness values to upper 16 bits
            cmd->v0_shade <<= 16;
            cmd->v1_shade <<= 16;
            cmd->v2_shade <<= 16;

            // Get texture from material
            texture_addr = material_textures[cmd->material_id * 4];
            DAT_0098a008 = cmd->material_id - 1;
            break;

        case 0x07:  // Textured Gouraud
            // Similar but with full texture mapping
            break;

        case 0x08:
        case 0x09:  // Environment mapped
            texture_addr = material_textures[cmd->material_id * 4];
            DAT_0098a008 = cmd->shade_value;
            break;
    }

    // Rasterize (2 or 5 iterations based on quality)
    iterations = (cmd->v0_shade == 0x200000 &&
                  cmd->v1_shade == 0x200000 &&
                  cmd->v2_shade == 0x200000) ? 2 : 5;
    DAT_00686898(cmd + 6, cmd + 0x1A, cmd + 0x2E, iterations);
    break;
```

### Special Command Types

```c
case 0x09:  // Health bar
    screen_x = cmd->v0_x;
    screen_y = cmd->v0_y;
    owner_color = player_colors[object->owner];
    Palette_IndexToRGBA(owner_color);
    FUN_0050ef90(screen_x - 5, screen_y - 10, screen_x + 5, screen_y);
    break;

case 0x0B:  // Selection circle
    object_index = object->sort_index;
    shadow_x = DAT_0087e55f + object_index * 0x9E;
    shadow_y = DAT_0087e561 + object_index * 0x9E;
    Sprite_RenderWithShadow();
    break;

case 0x0E:  // Line
    Palette_IndexToRGBA(cmd->color);
    FUN_0050f050(cmd->x0, cmd->y0, cmd->x1, cmd->y1);
    break;

case 0x0F:
case 0x10:
case 0x19:  // Marker sprites
    sprite_index = (cmd->type == 0x0F) ? 0x16 :
                   (cmd->type == 0x10) ? 0x47 : 0x46;
    sprite_data = sprite_table + sprite_index * 8;
    width = sprite_data[4];
    height = sprite_data[6];
    screen_x = cmd->x - width / 2;
    screen_y = cmd->y - height + 2;
    DAT_009735dc |= 8;  // Enable effects
    Sprite_BlitStandard(screen_x, screen_y, sprite_data);
    DAT_009735dc &= ~8;
    break;
```

---

## Appendix DC: Render Layer System

### Render_BuildLayerOrder @ 0x0047cc80

Determines the rendering order for UI layers based on game state:

```c
void Render_BuildLayerOrder(void) {
    byte layer_order[24] = {0xFF, ...};  // Initialize with 0xFF
    int layer_count = 0;

    // Check current player's research state
    int player_offset = DAT_00884c88 * 0xC65;
    byte research_flags = *(player_data + player_offset + 0x383);

    // Determine available layers based on research
    byte available[24] = {0};
    int available_count = 0;

    if (research_flags & 0x80) {  // Has shaman
        available[available_count++] = 7;
    }
    if (research_flags & 0x04) {  // Has blast
        available[available_count++] = 2;
    }
    if (research_flags & 0x08) {  // Has convert
        available[available_count++] = 3;
    }
    // ... etc for all spell types

    // Select active layer based on UI state
    uint ui_flags = DAT_0087e412;
    uint selected_layer = 0;

    if (ui_flags & 0x4000) {  // Spell panel open
        selected_layer = 0x18;
    }
    else if (ui_flags & 0x40000) {  // Building panel
        selected_layer = 0x1E;
    }
    else if (ui_flags & 0x04) {  // Attack mode
        selected_layer = 0x1C;
    }
    // ... priority-based layer selection

    // Verify layer is available
    bool layer_valid = false;
    for (int i = 0; i < available_count; i++) {
        if ((layer_mask_table[selected_layer * 0x16] &
             (1 << available[i])) != 0) {
            layer_valid = true;
            break;
        }
    }

    if (!layer_valid) {
        selected_layer = 3;  // Default layer
    }

    // Store final layer order
    DAT_0087e437 = layer_count;
    memcpy(&DAT_0087e438, layer_order, 24);

    // Calculate layer rotation step
    if (layer_count != 0) {
        DAT_0087e42a = (0x800 / layer_count) & 0x7FF;
    }
}
```

### Layer IDs

| ID | Layer Purpose |
|----|---------------|
| 0x03 | Default/neutral |
| 0x07 | Shaman spells |
| 0x08 | Building menu |
| 0x0A | Patrol mode |
| 0x0B | Convert spell |
| 0x10 | Training menu |
| 0x13 | Unit selection |
| 0x16 | Guard mode |
| 0x18 | Spell panel |
| 0x1B | Special abilities |
| 0x1C | Attack mode |
| 0x1D | Defend mode |
| 0x1E | Building panel |
| 0x21 | Dismount |
| 0x22 | Vehicle menu |

---

## Appendix DD: Complete Projection System

### Projection_InitializeDefaults @ 0x0046ed30

Sets up the default projection parameters:

```c
void Projection_InitializeDefaults(void) {
    DAT_00699a50 = &DAT_006868d0;  // Triangle command buffer
    DAT_00699a54 = &DAT_00688490;  // Vertex buffer

    // Projection constants
    DAT_007b8fb4 = 46000;          // CURVATURE constant
    DAT_007b8fbc = 0x1964;         // NEAR_PLANE (6500)
    DAT_007b8fc0 = 0x0B;           // FOV_SHIFT (11)
    DAT_007b8fb8 = 0x50;           // CLIP distance (80)
    DAT_007b8fcc = 0x20;           // ZOOM factor (32)

    DAT_00699a58 = 0;              // Command buffer start
    DAT_00699a5c = 0;              // Command count

    DAT_007b8f74 |= 0x80;          // Enable projection flag

    Camera_GenerateProjectionLUT(0x20);
    FUN_0049c100(&DAT_007b905b);   // Initialize camera state
}
```

### Projection_SetFromParams @ 0x0046f490

Configures projection from a parameter structure (g_CameraTarget):

```c
void Projection_SetFromParams(ProjectionParams* params) {
    // Viewport bounds
    DAT_007b8fe4 = params->viewport_x;      // +0x22
    DAT_007b8fe6 = params->viewport_y;      // +0x24
    DAT_007b8fe8 = params->viewport_width;  // +0x26
    DAT_007b8fea = params->viewport_height; // +0x28

    // Projection parameters
    DAT_007b8fb4 = params->curvature;       // +0x00 (can be dynamic!)
    DAT_007b8fcc = params->zoom;            // +0x04
    DAT_007b8fc0 = params->fov_shift;       // +0x14
    DAT_007b8fbc = params->near_plane;      // +0x10
    DAT_007b8fc4 = params->far_plane;       // +0x0C
    DAT_007b8fc8 = params->z_scale;         // +0x34

    // Screen center calculation
    DAT_007b8ffc = params->offset_x + viewport_width / 2;   // +0x2A
    DAT_007b8ffe = params->offset_y + viewport_height / 2;  // +0x2C

    // Cursor position relative to viewport
    DAT_007b8ff8 = cursor_x - viewport_x;
    DAT_007b8ffa = cursor_y - viewport_y;

    // Handle minimap region (if enabled)
    if (DAT_00884bf9 & 0x01) {
        // Calculate minimap clip region (±0x78 from cursor)
        // Clamp to viewport bounds
    }

    // Setup clipping region
    FUN_004c3bb0(&viewport_bounds);

    // Apply camera rotation if enabled
    if ((DAT_007b8f74 & 0x80) && params->rotation_enabled) {
        Camera_ApplyRotation(params);
    }
}
```

### Vertex_ApplyTransform @ 0x0046ebd0 (Complete Analysis)

The critical vertex transformation function:

```c
void Vertex_ApplyTransform(TerrainVertex* vertex) {
    int world_x = vertex->world_x;
    int world_y = vertex->world_y;  // Height
    int world_z = vertex->world_z;

    // Step 1: 3x3 Matrix Rotation (16.14 fixed-point)
    // Matrix stored at DAT_006868ac (9 values)
    int cam_x = (world_x * matrix[0] + world_z * matrix[2]) >> 14;
    int cam_y = (world_x * matrix[3] + world_y * matrix[4] + world_z * matrix[5]) >> 14;
    int cam_z = (world_x * matrix[6] + world_y * matrix[7] + world_z * matrix[8]) >> 14;

    vertex->cam_x = cam_x;
    vertex->cam_z = cam_z;

    // Step 2: Spherical Curvature Distortion
    // THE KEY FORMULA for the curved planet effect
    int64_t dist_sq = (int64_t)(cam_x * 2) * (cam_x * 2) +
                      (int64_t)(cam_z * 2) * (cam_z * 2);
    int64_t curvature = dist_sq * (int64_t)DAT_007b8fb4;  // 46000

    // Complex bit manipulation = curvature >> 32
    int curvature_offset = (int)((curvature >> 16) | ((curvature >> 32) << 16)) >> 16;

    cam_y = cam_y - curvature_offset;
    vertex->cam_y = cam_y;

    // Step 3: Perspective Projection
    int depth = cam_z + DAT_007b8fbc;  // Add near plane offset

    if (depth < 1) {
        // Behind camera - clip to extreme values
        vertex->screen_x = DAT_007b8ffc * -16;
        vertex->screen_y = DAT_007b8ffe * -16;
    }
    else {
        // Perspective division
        int scale = (1 << (DAT_007b8fc0 + 16)) / depth;  // (1 << 27) / depth

        // Apply FOV factor from g_CameraTarget+0x2A
        int fov_factor = *(int*)(g_CameraTarget + 0x2A);

        int proj_x = (fov_factor * cam_x >> 16) * scale;
        int proj_y = (fov_factor * cam_y >> 16) * scale;

        // Final screen coordinates
        vertex->screen_x = DAT_007b8ffc + (proj_x >> 16);
        vertex->screen_y = DAT_007b8ffe - (proj_y >> 16);
    }
}
```

### Mathematical Summary

**Curvature Formula (Exact):**
```
curvature_offset = floor((4 * (cam_x² + cam_z²) * 46000) / 2^32)
cam_y_final = cam_y - curvature_offset
```

**Perspective Formula:**
```
scale = 2^27 / (cam_z + 6500)
screen_x = center_x + (fov * cam_x * scale) >> 32
screen_y = center_y - (fov * cam_y_curved * scale) >> 32
```

### Projection Constants Table

| Address | Name | Default | Purpose |
|---------|------|---------|---------|
| 0x007b8fb4 | CURVATURE | 46000 | Spherical distortion strength |
| 0x007b8fbc | NEAR_PLANE | 0x1964 (6500) | Z offset for perspective |
| 0x007b8fc0 | FOV_SHIFT | 0x0B (11) | Bit shift for scale calc |
| 0x007b8fb8 | CLIP | 0x50 (80) | Clip distance factor |
| 0x007b8fcc | ZOOM | 0x20 (32) | Zoom/scale parameter |
| 0x007b8ffc | CENTER_X | dynamic | Screen center X |
| 0x007b8ffe | CENTER_Y | dynamic | Screen center Y |
| g_CameraTarget+0x2A | FOV_FACTOR | varies | Field of view multiplier |

---

## Appendix DE: Backface Culling and Clipping

### Terrain_CheckBackfaceCull @ 0x0046e870

Performs combined frustum clipping test and backface culling:

```c
int Terrain_CheckBackfaceCull(Vertex* v0, Vertex* v1, Vertex* v2) {
    // Step 1: Frustum clipping flags
    uint clip0 = 0, clip1 = 0, clip2 = 0;

    // Check X bounds
    if (v0->screen_x < 0) clip0 = 2;
    else if (v0->screen_x >= DAT_007b8fe8) clip0 = 4;

    // Check Y bounds
    if (v0->screen_y >= DAT_007b8fea) clip0 |= 0x10;

    // Repeat for v1, v2...
    if (v1->screen_x < 0) clip1 = 2;
    else if (v1->screen_x >= DAT_007b8fe8) clip1 = 4;
    if (v1->screen_y >= DAT_007b8fea) clip1 |= 0x10;

    if (v2->screen_x < 0) clip2 = 2;
    else if (v2->screen_x >= DAT_007b8fe8) clip2 = 4;
    if (v2->screen_y >= DAT_007b8fea) clip2 |= 0x10;

    // Step 2: Trivial reject - all vertices outside same edge
    if (clip0 != 0 && (clip0 & clip1) != 0 && (clip0 & clip1 & clip2) != 0) {
        return 0;  // Completely outside frustum
    }

    // Step 3: Backface culling via cross product (2D)
    // Returns > 0 for front-facing, <= 0 for back-facing
    int cross = (v2->screen_y - v1->screen_y) * (v1->screen_x - v0->screen_x) +
                (v2->screen_x - v1->screen_x) * (v0->screen_y - v1->screen_y);

    return cross;
}
```

### Clip Flag Encoding

| Bit | Meaning |
|-----|---------|
| 0x02 | Left of viewport (x < 0) |
| 0x04 | Right of viewport (x >= width) |
| 0x10 | Below viewport (y >= height) |

Note: Top clipping (y < 0) appears to be handled elsewhere or assumed within bounds.

---

## Appendix DF: Global Render State Variables

### Projection State (0x007b8f** region)

| Address | Type | Purpose |
|---------|------|---------|
| 0x007b8f74 | uint | Render flags (bit 0x80=projection init) |
| 0x007b8fac | int | Vertex row count |
| 0x007b8fb4 | int | Curvature constant (46000) |
| 0x007b8fb8 | int | Clip distance (0x50) |
| 0x007b8fbc | int | Near plane (0x1964) |
| 0x007b8fc0 | int | FOV shift (0x0B) |
| 0x007b8fc4 | int | Far plane |
| 0x007b8fc8 | int | Z scale |
| 0x007b8fcc | int | Zoom factor (0x20) |
| 0x007b8fd0 | int | Current world X |
| 0x007b8fd4 | int | Current world Z |
| 0x007b8fd8 | int | World X base |
| 0x007b8fe0 | short | Cell Y index |
| 0x007b8fe2 | short | Cell X index |
| 0x007b8fe4 | short | Viewport X |
| 0x007b8fe6 | short | Viewport Y |
| 0x007b8fe8 | short | Viewport width |
| 0x007b8fea | short | Viewport height |
| 0x007b8fec | short | Clip region X |
| 0x007b8fee | short | Clip region Y |
| 0x007b8ff0 | short | Clip region width |
| 0x007b8ff2 | short | Clip region height |
| 0x007b8ff4 | short | Cursor X (absolute) |
| 0x007b8ff6 | short | Cursor Y (absolute) |
| 0x007b8ff8 | short | Cursor X (viewport relative) |
| 0x007b8ffa | short | Cursor Y (viewport relative) |
| 0x007b8ffc | short | Screen center X |
| 0x007b8ffe | short | Screen center Y |

### Cursor/Selection State (0x007b90** region)

| Address | Type | Purpose |
|---------|------|---------|
| 0x007b9004 | short | Selected terrain X |
| 0x007b9006 | short | Selected terrain Y |
| 0x007b9010 | byte | Terrain cell X under cursor |
| 0x007b9011 | byte | Terrain cell Y under cursor |
| 0x007b901a | ushort | Object ID under cursor |
| 0x007b901c | short | Selection box X |
| 0x007b901e | short | Selection box Y |
| 0x007b9020 | short | Selection box width |
| 0x007b9022 | short | Selection box height |
| 0x007b9026 | short | Selection state |
| 0x007b9028 | short | Selection param 1 |
| 0x007b902a | short | Selection param 2 |
| 0x007b902c | short | Selection param 3 |
| 0x007b902e | short | Selection param 4 |
| 0x007b9041 | byte | Debug wireframe mode |
| 0x007b9042 | byte | Terrain rotation under cursor |

### Sprite/Render State (0x009735** region)

| Address | Type | Purpose |
|---------|------|---------|
| 0x009735a0 | int | Sprite palette secondary |
| 0x009735a8 | int | Viewport clip X |
| 0x009735ac | int | Viewport clip Y |
| 0x009735b8 | ptr | Current bit-depth vtable |
| 0x009735c0 | int | Color table offset 0 |
| 0x009735c4 | int | Color table offset 1 |
| 0x009735c8 | int | Color table offset 2 |
| 0x009735cc | int | Color table offset 3 |
| 0x009735d0 | ptr | Display info structure |
| 0x009735d4 | int | Sprite palette primary |
| 0x009735d8 | ptr | 8-bit vtable |
| 0x009735dc | uint | Render flags (bit 8=palette effect) |
| 0x009735e0 | ptr | 16-bit vtable |
| 0x009735e4 | ptr | 24-bit vtable |
| 0x009735e8 | ptr | 32-bit vtable |
| 0x009735ec | ptr | Frame buffer pointer |
| 0x00973610 | ptr | Current palette/effect table |
| 0x00973640 | array | Main palette (256 × 4 bytes RGBA) |

### Rasterizer State (0x0098a0** region)

| Address | Type | Purpose |
|---------|------|---------|
| 0x0098a004 | int | Current texture address |
| 0x0098a008 | byte | Current shade/material mode |

---

## Appendix DG: Complete Rendering Pipeline Summary

### High-Level Pipeline Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GAME LOOP                                   │
│  GameLoop @ 0x004ba520                                              │
│  - Process input                                                    │
│  - Update game state                                                │
│  - Trigger render                                                   │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    CAMERA SETUP                                     │
│  Render_SetupRasterizerCallback @ 0x00429a50                        │
│  - Set Rasterizer_Main as draw callback                             │
│  - Clear depth buckets (0xE01 entries)                              │
│  - Initialize viewport and cursor state                             │
│                                                                     │
│  Camera_SetupProjection @ 0x0046edb0                                │
│  - Copy camera matrix to global state                               │
│  - Apply rotation parameters                                        │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   TERRAIN GENERATION                                │
│  Terrain_GenerateVertices @ 0x0046dc10                              │
│  For each visible cell row:                                         │
│    - Read height from g_Heightmap                                   │
│    - Calculate brightness from lookup table                         │
│    - Set vertex flags (water=0x40, generated=0x80)                  │
│    - Call Vertex_ApplyTransform()                                   │
│                                                                     │
│  Vertex_ApplyTransform @ 0x0046ebd0                                 │
│    1. Matrix multiply (3x3 rotation)                                │
│    2. Apply curvature: y -= (4*(x²+z²)*46000) >> 32                 │
│    3. Perspective divide: scale = 2^27 / (z + 6500)                 │
│    4. Screen position: x = center + proj_x, y = center - proj_y    │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  TRIANGLE GENERATION                                │
│  Terrain_GenerateTriangles @ 0x0046e0f0                             │
│  For each cell quad:                                                │
│    - Check cell flags for diagonal split direction                  │
│    - Call Terrain_CreateTriangleCommand for 2 triangles             │
│                                                                     │
│  Terrain_CreateTriangleCommand @ 0x0046f6f0                         │
│    - Calculate depth: (v0.z + v1.z + v2.z + 0x15000) * 0x55 >> 8    │
│    - Apply distance shading attenuation                             │
│    - Create 70-byte triangle command (type 0x06)                    │
│    - Insert into depth bucket linked list                           │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   OBJECT SUBMISSION                                 │
│  Render_DispatchObjectsByType @ 0x0046fc30                          │
│  For each visible object:                                           │
│    - Get render type from object_type_table[obj->render_type]       │
│    - Dispatch to appropriate handler:                               │
│        Type 1: Object_SubmitToDepthBucket (standard sprite)         │
│        Type 3: Render with shadow                                   │
│        Type 5: Building render                                      │
│        Type 9: Spell effect                                         │
│        Type 13: Multi-part object                                   │
│        etc.                                                         │
│                                                                     │
│  Object_SubmitToDepthBucket @ 0x00470030                            │
│    - Transform world position to screen                             │
│    - Calculate depth bucket                                         │
│    - Create sprite render command                                   │
│    - Insert into depth bucket                                       │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│               DEPTH BUCKET PROCESSING                               │
│  Render_ProcessDepthBuckets_Main @ 0x0046af00                       │
│  For bucket = 0xE01 down to 0 (back to front):                      │
│    For each command in bucket:                                      │
│      Switch on command type:                                        │
│        0x00: Terrain triangle → texture lookup, water anim, raster  │
│        0x01: Standard sprite → Animation_RenderFrameSequence        │
│        0x06: 3D model triangle → Gouraud shading, rasterize         │
│        0x08: Textured triangle → UV coords, rasterize               │
│        0x09: Health bar → Draw_FilledRect                           │
│        0x0D: Unit sprite → Animation_RenderFrameSequence            │
│        0x0E: Line → Draw_Line                                       │
│        etc. (30+ command types)                                     │
│                                                                     │
│      During processing:                                             │
│        - Perform cursor hit-testing for selection                   │
│        - Update DAT_007b901a (object under cursor)                  │
│        - Update DAT_007b9010/11 (terrain cell under cursor)         │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    RASTERIZATION                                    │
│  Rasterizer_Main @ 0x0097c000 (52KB function)                       │
│  Called via callback pointer DAT_00686898                           │
│                                                                     │
│  For triangles:                                                     │
│    - Edge walking scanline rasterizer                               │
│    - Per-pixel texture sampling                                     │
│    - Gouraud shading interpolation                                  │
│    - Write to frame buffer via vtable                               │
│                                                                     │
│  For sprites:                                                       │
│    Sprite_BlitStandard @ 0x0050edd0                                 │
│    - Call through bit-depth vtable (8/16/24/32 bit)                 │
│    - Apply palette effects if DAT_009735dc & 0x08                   │
│                                                                     │
│    Sprite_BlitScaled @ 0x0050f6e0                                   │
│    - Scale sprite dimensions                                        │
│    - Call scaled blit routine                                       │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  POST-PROCESSING                                    │
│  Render_PostProcessEffects @ 0x00467890                             │
│  - Handle screen-space effects based on game state                  │
│  - Process spell visuals                                            │
│  - Update selection indicators                                      │
│  - Handle camera edge scrolling                                     │
│                                                                     │
│  Water_AnimateMesh @ 0x0048e210                                     │
│  - Update water wave phases                                         │
│  - Cull water meshes by distance                                    │
│                                                                     │
│  UI Rendering                                                       │
│  - UI_RenderGamePanel                                               │
│  - UI_RenderResourceDisplay                                         │
│  - Minimap rendering                                                │
│  - Font rendering                                                   │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      DISPLAY                                        │
│  DDraw_Flip @ 0x00510940                                            │
│  - Flip/blit DirectDraw surfaces                                    │
│  - Handle windowed vs fullscreen                                    │
│  - Restore surfaces on device loss                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Architectural Insights

1. **Hybrid 2D/3D Renderer**: The game uses true 3D geometry for terrain with software rasterization, but pre-rendered 2D sprites for all game objects (units, buildings, effects).

2. **Spherical Illusion**: The curved planet effect is achieved through a simple vertex Y-displacement formula applied during transformation, not through actual spherical projection.

3. **Depth Bucket Sorting**: All renderable elements are submitted to depth buckets (0-3584), then processed back-to-front for correct transparency and overlap.

4. **Bit-Depth Agnostic**: The renderer supports 8/16/24/32-bit color through vtable-based dispatch, allowing the same high-level code to work across different display modes.

5. **Integrated Selection**: Object selection via cursor is performed during render processing, using bounding box tests against screen coordinates.

6. **Fixed-Point Math**: All 3D math uses 16.14 fixed-point format (shift >> 14) for integer-only computation on 1998 hardware.

### Functions Renamed in This Session

| Address | New Name |
|---------|----------|
| 0x0050f720 | Sprite_SetupScaledBlit |
| 0x00477420 | Render_CalculateDistanceScale |
| 0x0050f390 | Render_SetupViewportClipping |
| 0x005643c0 | Sprite_BlitScaledInternal |
| 0x00566438 | Sprite_SetupScaleParams |
| 0x0052b950 | Render_CalculateBufferOffset |
| 0x0052b990 | Render_SetupBitMask |
| 0x004c3bb0 | Render_SetClipRegion |
| 0x0052b8c0 | Render_GetClipX |
| 0x0052b8d0 | Render_GetClipY |
| 0x0050ef90 | Draw_FilledRect |
| 0x0050ef70 | Draw_Pixel |
| 0x0050f050 | Draw_Line |
| 0x0048e990 | Water_UpdateWavePhase |
| 0x0048ebb0 | Terrain_SetupWaterWaveParams |
| 0x00473a70 | Terrain_FinalizeRender |
| 0x0046efe0 | Terrain_PostRenderCleanup |
| 0x00475530 | Terrain_RenderSpecialCells |
| 0x0042c170 | Terrain_ProcessVisibleObjects |
| 0x00476770 | Object_RenderHighlight |
| 0x00476e40 | Render_ProcessSelectionHighlights |
| 0x004771a0 | Render_ProcessUnitMarkers |
| 0x00487e30 | Render_Process3DModels |

---

## Appendix DI: Distance-Based Scaling and Fog

### Distance Scale Calculation

**Function:** `Render_CalculateDistanceScale @ 0x00477420`

This function applies distance-based scaling to sprites/objects for perspective and fog effects.

**Key Constants:**
| Constant | Address | Value | Purpose |
|----------|---------|-------|---------|
| FOV Multiplier | g_CameraTarget + 0x2A | Variable | Field of view scale |
| Scale Denominator | DAT_007b8fc4 | Variable | Perspective denominator |
| Distance Threshold | 0x6ff | 1791 | Fog start distance |
| Attenuation Factor | 0x0E | 14 | 14/16 = 87.5% scale per step |

**Algorithm:**
```c
void Render_CalculateDistanceScale(int depth, int* x, int* y) {
    int abs_depth = abs(depth);

    // Apply FOV-based scaling
    *x = (*x * 0x10 * g_CameraTarget[0x2A]) / DAT_007b8fc4;
    *y = (*y * 0x10 * g_CameraTarget[0x2A]) / DAT_007b8fc4;

    // Distance-based attenuation (fog effect)
    if (abs_depth > 0x6ff) {  // Beyond 1791 units
        int attenuation = ((*x * 14) >> 4) * (abs_depth - 0x700) / -0x700;
        *x += attenuation;
        *y += /* similar calculation */;
    }

    // Clamp to [-256, 256] range
    *x = clamp(*x, -0x100, 0x100);
    *y = clamp(*y, -0x100, 0x100);
}
```

---

## Appendix DL: Viewport and Resolution System

### Resolution-Specific Viewport Offsets

**Function:** `Camera_SetViewportOffsets @ 0x00421c70`

The game has hardcoded viewport adjustments for different resolutions:

**800×600:**
```c
offset[0x44] = 0xFFF2 (-14);   offset[0x46] = 0xFFF0 (-16);
offset[0x48] = 0x000E (+14);   offset[0x4A] = 0xFFF0 (-16);
offset[0x4C] = 0x0019 (+25);   offset[0x4E] = 0x002C (+44);
offset[0x50] = 0xFFE7 (-25);   offset[0x52] = 0x002C (+44);
```

**1024×768:**
```c
offset[0x44] = 0xFFF0 (-16);   offset[0x46] = 0xFFEE (-18);
offset[0x48] = 0x0010 (+16);   offset[0x4A] = 0xFFEE (-18);
offset[0x4C] = 0x001E (+30);   offset[0x4E] = 0x0031 (+49);
offset[0x50] = 0xFFE2 (-30);   offset[0x52] = 0x0031 (+49);
```

**1280×1024:**
```c
offset[0x44] = 0xFFEE (-18);   offset[0x46] = 0xFFEC (-20);
offset[0x48] = 0x0012 (+18);   offset[0x4A] = 0xFFEC (-20);
offset[0x4C] = 0x0020 (+32);   offset[0x4E] = 0x0036 (+54);
offset[0x50] = 0xFFE0 (-32);   offset[0x52] = 0x0036 (+54);
```

**Default (640×480):**
```c
offset[0x44] = 0xFFF6 (-10);   offset[0x46] = 0xFFF0 (-16);
offset[0x48] = 0x000A (+10);   offset[0x4A] = 0xFFF0 (-16);
offset[0x4C] = 0x0010 (+16);   offset[0x4E] = 0x0027 (+39);
offset[0x50] = 0xFFF0 (-16);   offset[0x52] = 0x0027 (+39);
```

### Viewport Clipping Setup

**Function:** `Render_SetupViewportClipping @ 0x0050f390`

**Clipping Rectangle Globals:**
| Address | Purpose |
|---------|---------|
| DAT_009735a8 | Clip min X (negated screen origin) |
| DAT_009735ac | Clip min Y |
| DAT_009735b0 | Clip max X |
| DAT_009735b4 | Clip max Y |
| DAT_009735c0 | Current viewport X |
| DAT_009735c4 | Current viewport Y |
| DAT_009735c8 | Current viewport width |
| DAT_009735cc | Current viewport height |

---

## Appendix DM: 3D Model Tile Rendering System

### Model Tile Cache

**Function:** `Render_Process3DModels @ 0x00487e30`

The 3D terrain uses a tile-based caching system where terrain chunks are pre-rendered to texture tiles.

**Cache Structure:**
- `DAT_007b9170` - Tile cache entries (8 bytes each, 512 entries = 0x200)
- `DAT_007b9160` - Tile lookup table (128×128 grid → tile index)
- `DAT_007b913c` - Pending tile buffer
- `DAT_00599ac0` - Rendered tile texture storage

**Tile Entry Format (8 bytes):**
```c
struct TileEntry {
    byte cell_x;        // +0x00: Cell X coordinate (0-127)
    byte cell_z;        // +0x01: Cell Z coordinate (0-127)
    byte flags;         // +0x02: Bit 0=active, Bit 1=rendered, Bit 2=dirty
    byte age;           // +0x03: Age counter for LRU eviction
    short prev_index;   // +0x04: Previous in LRU list
    short next_index;   // +0x06: Next in LRU list
};
```

**Tile Rendering Limits:**
- Maximum 150 (0x96) tiles rendered per frame
- Tiles sized 32×32 pixels (0x20)
- Uses LRU (Least Recently Used) eviction when cache full

### 3D Model Depth Buckets

**Function:** `Render_ProcessDepthBuckets_3DModels @ 0x0046d9a0`

Processes depth buckets 0x00-0x101 for 3D model rendering, applying texture coordinates from pre-rendered tiles.

**Texture Coordinate Sources:**
- Cached tiles: `DAT_0059c010[texture_id * 0x18]`
- Fallback: `DAT_0059c190[texture_id * 0x18]`

---

## Appendix DN: Render Command Buffer System

### Command Buffer Structure

**Key Functions:**
| Function | Address | Purpose |
|----------|---------|---------|
| RenderCmd_SubmitSimple | 0x00512930 | Submit cursor/simple command |
| RenderCmd_SubmitSprite | 0x00512b50 | Submit sprite render command |
| RenderCmd_SubmitComplex | 0x005129e0 | Submit complex viewport command |
| RenderCmd_WriteData | 0x0052d380 | Write command to buffer |
| RenderCmd_ReadNext | 0x00512760 | Read next command from buffer |
| RenderCmd_ProcessType2 | 0x00513000 | Process type 2 commands |

**Command Buffer Global:** `DAT_00973e98`

### Buffer Processing

**Function:** `Render_ProcessCommandBuffer @ 0x005125d0`

Processes commands from the circular buffer:

**Command Types:**
| Type | Code | Purpose |
|------|------|---------|
| 0x01 | Default | Standard render commands |
| 0x02 | Special | Viewport/state changes |
| 0x03 | Callback | Custom callback with parameters |
| 0xF0-0xF6 | Extended | Extended sprite/effect commands |

**Processing Flow:**
```c
while (commands_remaining > 0) {
    RenderCmd_ReadNext(&cmd);

    switch (cmd.type) {
    case 0x01:
        // Dispatch based on sub-type
        if (cmd.subtype >= 0xF0 && cmd.subtype <= 0xF6) {
            callbacks[4](subtype, params...);  // Extended handler
        } else {
            callbacks[0](subtype, params...);  // Default handler
        }
        break;
    case 0x02:
        RenderCmd_ProcessType2(cmd.flag != 0);
        break;
    case 0x03:
        callbacks[2](params...);  // Custom callback
        break;
    }
}
```

---

## Appendix DQ: Shadow Rendering System

### Shadow Offset Calculation

**Function:** `Shadow_CalculateOffset @ 0x00416410`

Calculates shadow offset based on object type and animation state.

**Shadow Offset by Object Type:**
| Type | Sub-type | Shadow Size |
|------|----------|-------------|
| Building (0x02) | Vault (0x13) | 0x140 (320) |
| Shape (0x09) | Any | 0x40 (64) |
| Standard animation | type 1 | `sprite_height << 3` |
| Bone animation | type 3 | Half of bounding box |
| Sequence animation | type 10 | From sequence data |

### Shadow Rendering

**Function:** `Sprite_RenderWithShadow @ 0x00411b70`

Renders object with optional shadow:

**Shadow Modes (from param_1 + 0x16 + mode_offset * 4):**
| Mode | Description |
|------|-------------|
| 0x00 | No shadow - direct render |
| 0x01 | Shadow enabled - render shadow first, then object |

**Shadow Rendering Pipeline:**
1. Setup shadow surface (`FUN_00512310`)
2. Apply shadow color masks (`Render_SetupColorMasks`)
3. Render shadow sprite (`FUN_00416000`)
4. Render main object (`Sprite_RenderObject`)
5. Restore color masks
6. Apply shadow blend (`FUN_00416110`)
7. Restore viewport clip region

---

## Appendix DS: Projection LUT Generation

### Circular Viewport LUT

**Function:** `Camera_GenerateProjectionLUT @ 0x0046f1e0`

Generates lookup table for circular viewport clipping.

**Algorithm:**
```c
void Camera_GenerateProjectionLUT(ushort radius) {
    // Clear LUT (0xDE = 222 entries)
    memset(DAT_0069d268, 0, 222 * 4);

    short* left = &DAT_0069d420;   // Left edge table
    short* right = &DAT_0069d420;  // Right edge table

    for (int y = 0; y < radius/2; y++) {
        // Calculate circle width at this Y
        int x = sqrt((radius/2)² - y²);
        if (x < 3) x = 0;

        short left_edge = 0x6E - x;   // 110 - x
        short right_edge = 0x6E + x;  // 110 + x

        // Store symmetrically
        *left = left_edge;
        left[1] = right_edge;
        *right = left_edge;
        right[1] = right_edge;

        left -= 2;   // Move up
        right += 2;  // Move down
    }
}
```

**LUT Storage:**
- `DAT_0069d268` - Main viewport bounds (222 × 4 bytes)
- `DAT_0069d420` - Edge table center (indexed +/- from here)

**Center Offset:** 0x6E (110) - viewport center in cells

### Projection Defaults

**Function:** `Projection_InitializeDefaults @ 0x0046ed30`

**Default Values Set:**
| Global | Value | Purpose |
|--------|-------|---------|
| DAT_007b8fb4 | 46000 | Curvature constant |
| DAT_007b8fbc | 0x1964 (6500) | Near plane Z offset |
| DAT_007b8fc0 | 0x0B (11) | FOV bit shift |
| DAT_007b8fb8 | 0x50 (80) | Clip distance |
| DAT_007b8fcc | 0x20 (32) | Initial zoom/LUT radius |

---

## Appendix DV: Post-Processing Effects

### Function: `Render_PostProcessEffects @ 0x00467890`

Applies post-render effects based on game state.

**Edge Detection (Cursor near screen edge):**
```c
if (cursor_x < 1) FUN_0048b0e0(2, 4, 0);      // Left edge
if (cursor_x >= screen_width - 1) FUN_0048b0e0(2, 8, 0);  // Right edge
if (cursor_y < 1) FUN_0048b0e0(2, 1, 0);      // Top edge
if (cursor_y >= screen_height - 1) FUN_0048b0e0(2, 2, 0); // Bottom edge
```

**Game Mode Effects (DAT_00884c7f):**
| Mode | Effect |
|------|--------|
| 4 | Periodic sound triggers (every 50ms) |
| 9 | Fog of war overlay |
| 12 | Multiplayer status display |
| 14 | Victory/defeat effects |
| 15 | Object tracking highlight |
| 16 | Camera movement effects |

**Screen Shake Formula:**
```c
intensity = DAT_0087e345 * 8 + 0x78;  // Base 120 + 8 per shake level
intensity = clamp(intensity, 0, 255);
FUN_004abb50(delta_y * 12, intensity);
FUN_004abab0(delta_x * 12, intensity);
```

---

## Appendix DW: Software Rasterizer (Rasterizer_Main)

### Overview

**Function:** `Rasterizer_Main @ 0x0097c000`
**Size:** 52,199 bytes (0x0097c000 - 0x00988fe7)
**Type:** Hand-optimized x86 assembly

This is the core software triangle rasterizer - a massive hand-written assembly function optimized for 1998 Pentium/Pentium II processors.

### Entry Point Analysis

```asm
0097c000: PUSHAD                          ; Save all registers
0097c001: MOV EAX,[ESP+0x24]              ; Vertex 1 pointer
0097c005: MOV EDX,[ESP+0x28]              ; Vertex 2 pointer
0097c009: MOV EBX,[ESP+0x2c]              ; Vertex 3 pointer
0097c00d: MOV ECX,[ESP+0x30]              ; Render mode
0097c011: SUB ESP,0x190                   ; Allocate 400 bytes local stack
```

**Parameters:**
- EAX = Vertex 1 pointer (screen coords + UV + shade)
- EDX = Vertex 2 pointer
- EBX = Vertex 3 pointer
- ECX = Render mode (texture/shade flags)

### Vertex Sorting (Y-axis)

```asm
; Sort vertices by Y coordinate (top to bottom)
0097c070: MOV ECX,[EAX+0x4]               ; V1.y
0097c073: CMP ECX,[EDX+0x4]               ; Compare with V2.y
0097c076: JLE 0x0097c07c
0097c078: MOV ECX,[EDX+0x4]
0097c07b: XCHG EAX,EDX                    ; Swap V1 and V2
0097c07c: CMP ECX,[EBX+0x4]               ; Compare with V3.y
...
; Result: EAX=top, EDX=middle, EBX=bottom vertex
```

### Degenerate Triangle Rejection

```asm
0097c08f: MOV ECX,[EAX+0x4]               ; Top Y
0097c092: CMP ECX,[EBX+0x4]               ; Bottom Y
0097c095: JZ 0x00988fe0                   ; Skip if zero height (degenerate)
```

### Edge Gradient Calculation

Uses a lookup table for small deltas (optimization):
```asm
; For delta_y <= 32 and -32 <= delta_x <= 31
0097c165: MOV EBX,ECX
0097c167: SHL EBX,0x8                     ; delta_y * 256
0097c16a: MOV EAX,[EBX+EAX*4+0x987060]    ; LUT: gradient[dy][dx]

; For larger deltas, use division
0097c1f0: SHL EAX,0x10                    ; delta_x << 16
0097c1f3: CDQ                             ; Sign extend
0097c1f4: IDIV ECX                        ; (dx << 16) / dy = 16.16 gradient
```

### Gradient LUT Location

**Address:** `0x00987060`
**Size:** 256 × 64 × 4 = 65,536 bytes
**Format:** Pre-computed 16.16 fixed-point gradients for small triangles

### Render Mode Dispatch

```asm
0097c238: MOV ECX,[0x0098a009]            ; Render mode
0097c23e: JMP [ECX*4+0x986660]            ; Jump table dispatch
```

**Jump Table:** `0x00986660`
Contains entry points for different rendering modes:
- Flat shaded
- Gouraud shaded
- Textured (affine)
- Textured + shaded
- Perspective-correct textured
- etc.

### Key Rasterizer Globals

| Address | Purpose |
|---------|---------|
| 0x0098a004 | Texture pointer |
| 0x0098a008 | Shade/blend value |
| 0x0098a009 | Render mode |
| 0x0098a00d | Max screen coordinate |
| 0x0098a01d | Scanline buffer offset |
| 0x00987060 | Gradient lookup table |
| 0x00986660 | Render mode jump table |

### Local Stack Layout (0x190 = 400 bytes)

| Offset | Size | Purpose |
|--------|------|---------|
| +0x00 | 4 | Edge gradient (top-bottom) |
| +0x04 | 4 | Edge gradient (top-middle) |
| +0x08 | 4 | Edge gradient (middle-bottom) |
| +0x34 | 4 | V1.y |
| +0x38 | 4 | V1.x |
| +0x3c | 4 | V1.x << 16 |
| +0x40 | 4 | V1 shade |
| +0x44 | 4 | V1.u |
| +0x48 | 4 | V1.v |
| +0x4c | 4 | V2.y |
| +0x50 | 4 | V2.x |
| +0x54 | 4 | V2.x << 16 |
| +0x58 | 4 | V2 shade |
| +0x5c | 4 | V2.u |
| +0x60 | 4 | V2.v |
| +0x64 | 4 | V3.y |
| +0x68 | 4 | V3.x |
| +0x6c | 4 | V3.x << 16 |
| +0x70 | 4 | V3 shade |
| +0x74 | 4 | V3.u |
| +0x78 | 4 | V3.v |
| +0x7c | 4 | Interpolated X delta |
| +0x170 | 4 | Clipping flag |

### Optimization Techniques Used

1. **Gradient LUT** - Pre-computed edge gradients for small triangles
2. **Fixed-point math** - 16.16 format throughout
3. **Register allocation** - All critical values in registers
4. **Jump table dispatch** - Fast mode selection
5. **Unrolled loops** - Scanline inner loops are unrolled
6. **Pentium pairing** - Instructions ordered for U/V pipe pairing

### Performance Characteristics

- Handles ~50,000 triangles/frame on target hardware (P200)
- Texture mapping uses affine interpolation (not perspective-correct for speed)
- Gouraud shading with 6-bit precision (64 levels)
- No subpixel precision (causes texture swimming on distant terrain)

---

## Complete Rendering System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        GAME LOOP                                 │
│  GameLoop @ 0x004ba520                                          │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                   RENDER ORCHESTRATION                           │
│  Terrain_RenderOrchestrator @ 0x0046ac90                        │
│  ├── Camera_SetupProjection @ 0x0046edb0                        │
│  ├── Terrain_GenerateVertices @ 0x0046dc10                      │
│  ├── Terrain_GenerateTriangles @ 0x0046e0f0                     │
│  └── Render_ProcessDepthBuckets_Main @ 0x0046af00               │
└─────────────────────┬───────────────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
┌───────────────┐ ┌───────────┐ ┌───────────────┐
│   TERRAIN     │ │  OBJECTS  │ │    EFFECTS    │
│               │ │           │ │               │
│ Vertex_Apply  │ │ Sprite_   │ │ Effect_Queue  │
│ Transform     │ │ Render    │ │ Visual        │
│ @ 0x0046ebd0  │ │ Object    │ │ @ 0x00453780  │
│               │ │ @ 411c90  │ │               │
│ Curvature:    │ │           │ │ Animation_    │
│ y -= (x²+z²)  │ │ Shadow_   │ │ RenderFrame   │
│     × 46000   │ │ Calculate │ │ Sequence      │
│               │ │ @ 416410  │ │ @ 0x004e7190  │
└───────┬───────┘ └─────┬─────┘ └───────┬───────┘
        │               │               │
        └───────────────┴───────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                    DEPTH BUCKET SORTING                          │
│  3585 buckets (0x00 - 0xE00)                                    │
│  Back-to-front processing                                        │
│  Triangle commands inserted via Terrain_CreateTriangleCommand   │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RASTERIZATION                                 │
│  Rasterizer_Main @ 0x0097c000 (52KB hand-optimized ASM)         │
│  ├── Vertex sorting                                             │
│  ├── Edge gradient calculation                                  │
│  ├── Scanline interpolation                                     │
│  └── Pixel output (via vtable for 8/16/24/32-bit)              │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SPRITE BLITTING                               │
│  Sprite_BlitStandard @ 0x0050edd0                               │
│  Sprite_BlitScaled @ 0x0050f6e0                                 │
│  Uses vtable at DAT_009735b8 for bit-depth dispatch             │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    FRAME BUFFER                                  │
│  DDraw_Flip @ 0x00510940                                        │
│  Double-buffered DirectDraw surfaces                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Appendix DZ: DirectDraw Display Pipeline

### Key Functions

| Function | Address | Purpose |
|----------|---------|---------|
| DDraw_Initialize | 0x00510ca0 | Initialize DirectDraw |
| DDraw_Create | 0x00510c70 | Create DirectDraw object |
| DDraw_Flip | 0x00510940 | Flip front/back buffers |
| DDraw_FlipAndClear | 0x00510b70 | Flip and clear back buffer |
| DDraw_BlitRect | 0x00510a90 | Blit rectangle to surface |
| DDraw_ClearSurface | 0x00511e80 | Clear surface to color |
| DDraw_RestoreSurface | 0x00511e50 | Restore lost surfaces |
| DDraw_IsInitialized | 0x00510210 | Check initialization state |
| DDraw_RegisterWindowClass | 0x00510e10 | Register window class |
| DDraw_EnumerateDevices | 0x0041f500 | Enumerate display devices |

### DDraw Global State

| Address | Variable | Purpose |
|---------|----------|---------|
| DAT_00973adc | DDraw flags | Mode flags |
| DAT_00973ae4 | Primary surface | Front buffer |
| DAT_00973c4c | Back buffer | Render target |
| DAT_00973a94 | Offscreen surface | For windowed mode |
| DAT_005aeec0 | Windowed mode flag | 0=fullscreen, 1=windowed |

### DDraw_Flip Algorithm

```c
int DDraw_Flip(uint flip_flag) {
    flip_flag ^= 1;  // Toggle buffer index

    if ((DDraw_flags & 2) == 0) {
        // Normal mode
        FUN_0052bb60(0, 0);  // Pre-flip callback

        if ((DDraw_flags & 0x10) == 0) {
            // Hardware flip available
            if (windowed_mode == 0) {
                // Fullscreen: Direct flip
                result = primary->Flip(0, 0, back_buffer, 0, flip_flag << 4);
            } else {
                // Windowed: Blit then flip
                offscreen->Flip(0, 0, back_buffer, 0, flip_flag << 4);
                result = primary->Blt(offscreen, flip_flag);
            }
        } else {
            // Software blit mode (compatibility)
            GetWindowRect(hwnd, &rect);
            result = primary->Blt(&rect, back_buffer, 0, 0, 0x1000000, NULL);
        }
    } else {
        // Alternate mode (triple buffering?)
        FUN_0052bb60(2, 0);
        result = primary->Blt(back_buffer, flip_flag);
    }

    // Handle surface loss (Alt+Tab, etc.)
    if (result == DDERR_SURFACELOST) {
        DDraw_RestoreSurface(&primary);
        DDraw_RestoreSurface(&back_buffer);
    }

    return (result == 0) ? 0 : -1;
}
```

### DDraw Flags (DAT_00973adc)

| Bit | Meaning |
|-----|---------|
| 0x02 | Use alternate flip method |
| 0x10 | Software blit mode (no hardware flip) |

---

## Appendix EA: Layer Rendering System

### Render_DrawLayer @ 0x004a0230

Renders a linked list of UI/overlay elements filtered by layer and visibility:

```c
void Render_DrawLayer(layer_context, visibility_table, layer_id) {
    for (element = layer_context->first_element; element != NULL; element = element->next) {
        // Layer filter: element must match layer_id or be '@' (all layers)
        if (element->layer != layer_id && element->layer != '@')
            continue;

        // Visibility check from table
        if (visibility_table[element->visibility_index] == 0)
            continue;

        // Conditional rendering checks
        if ((global_flags & 8) && (player_flags & 4) && !(element->flags & 1))
            continue;  // Skip non-essential in minimal mode

        if ((element->flags & 2) && !(global_flags & 0x8000))
            continue;  // Skip debug elements unless debug mode

        // Call element's render function
        if (element->render_func != NULL) {
            element->render_func(element->data);
        }
    }
}
```

### Layer Element Structure (0x10 bytes)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 4 | Element data pointer |
| +0x04 | 1 | Visibility table index |
| +0x05 | 1 | Layer ID ('0'-'9', '@'=all) |
| +0x06 | 4 | Render function pointer |
| +0x0A | 1 | Element flags |
| +0x0B | 4 | Next element pointer |

### Render_DrawCharacter @ 0x004a0570

Renders a single character using bitmap font:

```c
// For Japanese (locale 0xB) - 16x16 double-byte
if (locale == 0x0B) {
    // Shift-JIS to internal index conversion
    index = (high_byte * 94 + low_byte) * 32;

    for (row = 0; row < 16; row++) {
        bits = font_data[index + row*2] << 8 | font_data[index + row*2 + 1];
        for (col = 15; col >= 0; col--) {
            if (bits & (1 << col)) {
                Palette_IndexToRGBA(color);
                (*vtable->draw_pixel)(x + (15-col), y + row);
            }
        }
    }
}
// For Western - 8x16 single-byte
else {
    for (row = 0; row < 16; row++) {
        bits = font_data[char_index * 32 + row * 2];
        for (col = 0; col < 16; col++) {
            if (bits & (1 << (col % 8))) {
                Palette_IndexToRGBA(color);
                (*vtable->draw_pixel)(x + col, y + row);
            }
        }
    }
}
```

---

## Appendix EF: Faithful Project — 3D Object Pipeline

*Source: [faithful](https://github.com/hrttf111/faithful) Rust OpenGL renderer + [pop3-rev](https://github.com/hrttf111/pop3-rev) Ghidra RE project. Cross-referenced with our own Ghidra analysis.*

### 3D Object Binary Formats

#### ObjectRaw (54 bytes) — `objects/OBJS{bank}-0.DAT`

```
+0x00  u16   flags
+0x02  u16   facs_num          (face count)
+0x04  u16   pnts_num          (vertex count)
+0x06  u8    f1
+0x07  u8    morph_index
+0x08  u32   f2
+0x0C  u32   coord_scale       scale = (coord_scale / 300.0) * 0.5
+0x10  u32   facs_ptr          1-based index into FACS array
+0x14  u32   facs_ptr_end      exclusive end (1-based)
+0x18  u32   pnts_ptr          1-based index into PNTS array
+0x1C  u32   pnts_ptr_end      exclusive end (1-based)
+0x20  [22]  bounding box / misc fields
```

- 194 objects in bank 0, 141 have valid vertex/face data within bank 0 range
- **Pointers are 1-based**: subtract 1 for array access
- **9 banks** (0-8), each with own OBJS/PNTS/FACS files — tribal variants

#### PointRaw (6 bytes) — `objects/PNTS{bank}-0.DAT`

```
+0x00  i16   x
+0x02  i16   y
+0x04  i16   z
```

- **XYZ_SCALE = 1.0 / 300.0** (from faithful constants)
- Bank 0: 66,784 points total

#### FaceRaw (60 bytes) — `objects/FACS{bank}-0.DAT`

```
+0x00  u16   color_index       (palette color for flat-shaded faces)
+0x02  i16   tex_index         (-1 / 0xFFFF = flat-shaded, ≥0 = texture page)
+0x04  i16   flags1
+0x06  u8    num_points        (3 = triangle, 4 = quad)
+0x07  u8    render_flags      (0x00=flat, 0x07=textured, 0x20=decal)
+0x08  u32×8 UV coordinates    (4 vertices × 2 UVs, fixed-point)
+0x28  u16×4 vertex indices    (0-based LOCAL to object's point range)
+0x30  u16×4 per-vertex colors (brightness/color values)
+0x38  u16   tex_sub_flags
+0x3A  u16   padding
```

- Bank 0: 16,144 faces total (8,738 textured, 3,047 flat-shaded)
- 126 unique tex_index values used (0-249 range)
- Face vertex indices are **local** to the object's point slice
- Global PNTS index = `(pnts_ptr - 1) + local_index`
- **Quad triangulation winding**: `[0,1,2]` + `[2,3,0]` (from faithful)

#### UV Coordinate System

```
UV_SCALE = 4.768372e-7  (≈ 1/2^21, faithful constant)

local_u = raw_u32 * UV_SCALE    → [0.0 .. 1.0] within tile
local_v = raw_u32 * UV_SCALE    → [0.0 .. 1.0] within tile
```

### Texture Atlas System

#### Atlas Layout (from faithful fragment shader)

```
8 columns × 32 rows of 256×256 pixel tiles = 2048×8192 total

UV mapping to atlas coordinates:
  row = tex_id / 8
  col = tex_id % 8
  atlas_u = (col + local_u) / 8.0
  atlas_v = (row + local_v) / 32.0

tex_id is passed flat (no interpolation) — glVertexAttribIPointer in faithful
```

#### BL320 Files — Base Terrain Textures

```
36 files: data/bl320-{0..9,A..Z}.dat
Each file: 4 tiles × 256×256 pixels × 1 byte = 262,144 bytes (paletted, 8bpp)
Total: 144 tiles (sequential loading: file 0→tiles 0-3, file 1→tiles 4-7, etc.)

Each BL320 file has its own palette: data/pal0-{0..9,A..Z}.dat
```

#### plstx Files — Level-Specific Object Textures

```
Per-level file: data/plstx_%03d.dat (e.g., plstx_001.dat for level 1)
Format: Same as BL320 (4 tiles of 256×256 paletted, 0x40000 bytes)
Loaded at: DAT_007b9178 (from Ghidra LoadLevelTextures @ 0x00421320)

Level palette: data/plspl0-{c}.dat (1024 bytes = 256 × 4 RGBA)
  Where c = theme character: '0'-'9' for themes 0-9, 'a'-'z' for themes 10+
```

#### Palette Decryption (from faithful `pls::decode`)

All palette files (pal0-*.dat, plspl0-*.dat) use XOR-based encryption:

```rust
fn palette_decrypt(data: &mut [u8]) {
    let mut m: u8 = 3;
    for v in data.iter_mut() {
        *v = !(*v ^ (1 << ((m.wrapping_sub(3)) & 7)));
        m = m.wrapping_add(1);
    }
}
```

**Alpha convention**: In palette files, alpha > 0 means **transparent** (inverted from standard RGBA). When building RGBA textures, invert: `alpha = 255 - file_alpha`.

### GPU Rendering Approach (faithful)

```
TexVertex struct (per-vertex, SoA buffer layout):
  coord: vec3  (world position, pre-scaled by XYZ_SCALE)
  uv:    vec2  (per-tile UV, pre-multiplied by UV_SCALE)
  tex_id: i16  (texture page index, flat/no interpolation)

Drawing: Non-indexed (glDrawArrays), vertices duplicated per face
  - Each triangle = 3 TexVertex entries
  - Each quad = 6 TexVertex entries (2 triangles)
```

### PSFB Sprite Format (from faithful `psfb.rs`)

Used for 2D sprite animations (HSPR0-0.DAT etc.):

```
Header:
  +0x00  u32   magic ("PSFB" / 0x42465350)
  +0x04  u32   num_sprites
  +0x08  u32   data_size (total pixel data)

Per-sprite descriptor (8 bytes each, num_sprites entries):
  +0x00  u32   data_offset (into pixel data section)
  +0x04  u16   width
  +0x06  u16   height

Pixel data section (after all descriptors):
  RLE-encoded, palette-indexed

RLE decompression:
  - Read byte N
  - If N > 0: next N bytes are literal pixel palette indices
  - If N == 0: read count byte, emit that many transparent pixels (index 0)
  - Repeat until row_width pixels emitted, then next row
  - Continue for height rows
```

### Animation Pipeline (Cross-Reference with Appendix AZ)

faithful confirms the animation chain system documented in Appendix AZ:

```
VSTART-0.ANI  (4 bytes/entry): animation_id → vfra_index + mirror_ref
     ↓
VFRA-0.ANI    (8 bytes/entry): sprite_index, w, h, flags, next_frame (linked list)
     ↓
VELE-0.ANI    (10 bytes/entry): sprite_idx, x_offset, y_offset, flip_flags, next_element
     ↓
HSPR0-0.DAT   (PSFB sprites): actual rendered pixel data

Traversal: VSTART[anim_id] → follow VFRA chain → for each frame, follow VELE elements
5 directions stored (0-4), 3 mirrored (5-7) from directions 3-1
```

### faithful Architecture Reference

```
Module layout:
  game/       — object parsing (obj.rs), mesh building, animation loading
  opengl/     — GL rendering, shader management, texture atlas binding
  pls/        — file I/O, palette decode, BL320/plstx loading, PSFB decompression

Key source files:
  game/obj.rs    — ObjectRaw, FaceRaw, PointRaw structs, mesh builder
  pls/mod.rs     — decode(), BL320 loader, palette handler
  pls/psfb.rs    — PSFB header parsing, RLE decompression
  opengl/draw.rs — TexVertex, atlas UV mapping, draw calls

Requires OpenGL 4.6 (gl46 crate) — does NOT run on macOS (max GL 4.1)
```

---

## Appendix R: Rendering System (Ghidra Disassembly Analysis)

### R.1 Binary Overview

Segments:
- `.text`: 0x00401000–0x00563FFF (main code)
- `CSEG`: 0x00564000–0x0056C7FF (additional code, possibly MPEG/video)
- `.rdata`: 0x0056D000–0x005741FF (read-only data, strings)
- `.data`: 0x00575000–0x0097992F (mutable globals — very large, ~4MB)
- `IDCT_DAT`/`UVA_DATA`: 0x0098C000–0x00992BFF (MPEG decoder tables)

Imports: DirectDraw (DDRAW.dll), QSWaveMix (3D audio), WINMM (MIDI/timers), standard Win32

### R.2 Game State Machine

The game uses a state machine with these states:

| Function | Address | Purpose |
|----------|---------|---------|
| `GameState_Loading` | 0x0041FAB0 | Level loading |
| `GameState_Frontend` | 0x004BAA40 | Main menu / level select |
| `GameState_InGame` | 0x004DDD20 | Active gameplay |
| `GameState_Multiplayer` | 0x004C03D0 | Multiplayer lobby/game |
| `GameState_Outro` | 0x004BAE70 | End sequence |
| `GameState_InitiateTransition` | 0x004ABC80 | Transition start |
| `GameState_CompleteTransition` | 0x004ABCD0 | Transition end |

State byte at `0x0087759A` controls which state runs. `GameState_InGame` switches on values 0x1, 0x4, 0x5 for sub-states (loading, paused, etc).

### R.3 DirectDraw Display System

#### Initialization: `ddraw_init_display` @ 0x005102E0

The display is initialized through DirectDraw 1.0 API (not DirectDraw2+):

1. Calls `DDraw_Create` (0x00510C70) → stores IDirectDraw* at `g_pDirectDraw` (0x005AEEA8)
2. Gets or creates window → `g_hWnd` (0x005AEEA0)
3. Calls `ShowWindow(hWnd, SW_SHOW)` then pumps messages via `PeekMessageA`/`DefWindowProcA`
4. Sets cooperative level via IDirectDraw::SetCooperativeLevel (vtable+0x50)
   - Fullscreen: flags 0x51 (DDSCL_FULLSCREEN | DDSCL_EXCLUSIVE | DDSCL_ALLOWREBOOT)
   - Windowed: flags 0x08 (DDSCL_NORMAL)
5. In windowed mode, creates a clipper via IDirectDraw::CreateClipper (vtable+0x10)
6. Creates primary surface → `g_pPrimarySurface` (0x00973AE4)
   - Fullscreen with back buffer: DDSD_CAPS | DDSD_BACKBUFFERCOUNT, caps = DDSCAPS_PRIMARYSURFACE | DDSCAPS_FLIP | DDSCAPS_COMPLEX
   - Windowed: simple primary surface
7. Creates back buffer surface → `g_pBackSurface` (0x00973C4C)
8. If 8bpp: Creates DDPalette → `g_pDDPalette` (0x005AEEAC) from `g_palette_entries` (0x00973640)
   - If no custom palette provided, generates identity palette (R=G=B=index)
   - Gets system palette via `GetSystemPaletteEntries` and applies it
9. Sets display mode via IDirectDraw::SetDisplayMode (vtable+0x54) with requested resolution/bpp

#### Key Globals

| Address | Name | Type | Purpose |
|---------|------|------|---------|
| 0x005AEEA8 | `g_pDirectDraw` | IDirectDraw* | DirectDraw interface |
| 0x005AEEA0 | `g_hWnd` | HWND | Rendering window |
| 0x005AEEA4 | (unnamed) | HWND | Optional external window handle |
| 0x00973AE4 | `g_pPrimarySurface` | IDirectDrawSurface* | Primary (front) surface |
| 0x00973C4C | `g_pBackSurface` | IDirectDrawSurface* | Back buffer surface |
| 0x005AEEAC | `g_pDDPalette` | IDirectDrawPalette* | 8bpp palette |
| 0x00973640 | `g_palette_entries` | PALETTEENTRY[256] | 1024 bytes of palette data |
| 0x005AEEB4 | (unnamed) | DWORD | Display width (set to 1 during init) |
| 0x005AEEB8 | (unnamed) | DWORD | Display height |
| 0x005AEEC0 | (unnamed) | DWORD | Triple-buffer flag |
| 0x005AF228 | `g_hMainWnd` | HWND | App main window |
| 0x00884C67 | `g_screen_width` | WORD | Current screen width (e.g. 640, 512) |
| 0x00884C69 | `g_screen_height` | WORD | Current screen height (e.g. 480, 384) |

#### Surface Flipping

`DDraw_Flip` @ 0x00510940:
- Handles 3 modes: double-buffer flip, windowed blit, triple-buffer blit
- Flag byte at 0x00973ADC controls behavior
  - Bit 1: Uses IDirectDrawSurface::Flip (vtable+0x2C) for hardware flip
  - Bit 4: Windowed mode — calls GetWindowRect, then IDirectDrawSurface::Blt (vtable+0x14) from back to primary
  - Default: Uses IDirectDrawSurface::BltFast (vtable+0x1C) with DDBLTFAST_WAIT
- On DDERR_SURFACELOST (0x887601C2): calls `DDraw_RestoreSurface` (0x00511E50) for both primary and back surfaces

`DDraw_FlipAndClear` @ 0x00510B70: Flip + clear the back buffer

#### Cleanup: `ddraw_cleanup` @ 0x00510110

1. IDirectDraw::RestoreDisplayMode (vtable+0x4C)
2. IDirectDraw::Release (vtable+0x08)
3. Nullifies g_pDirectDraw

### R.4 Render Pipeline (Per-Frame)

The main in-game frame flow from `GameState_InGame` (0x004DDD20):

```
GameState_InGame
  ├── 0x004DEE90 — input processing / game tick
  ├── 0x004E10D0 — game simulation update (param = turn number)
  ├── 0x00417BB0 — pre-render update
  │
  ├── Lock back surface: ddraw_lock_surface(g_pBackSurface, &pitch)
  │     → stores pixel ptr at g_render_buffer_ptr (0x0096CEF8)
  │     → stores pitch at g_render_buffer_pitch (0x0096CEFC)
  │
  ├── render_frame @ 0x004DF3C0:
  │     ├── Copy software framebuffer → locked surface (memcpy rows)
  │     │     Source: g_software_framebuffer (0x0096CF10)
  │     │     Dest: g_render_buffer_ptr, stride = g_render_buffer_pitch
  │     │     Copy 0xA0 dwords (640 bytes) per scanline, up to 480 lines
  │     │
  │     ├── 0x0040B060 — clear/reset render state
  │     ├── 0x004E1980 — MAIN SCENE RENDERER (giant function)
  │     │     Processes render command buffer for the current turn
  │     │     Dispatches draw commands by type (sprites, 3D objects, terrain overlays)
  │     │
  │     ├── 0x0040AEA0 — post-render processing
  │     ├── draw_sprite_rle — cursor/overlay sprite rendering
  │     │     Uses RLE-encoded sprite data with color lookup via g_color_lookup_table
  │     │
  │     ├── 0x00510210 (ddraw_get_display_width) — check display mode changes
  │     ├── RenderCmd_SubmitComplex / Render_InitScreenBuffer — display buffer management
  │     │
  │     ├── Sprite_BlitStandard @ 0x0050EDD0 — blit sprite banks to screen
  │     └── 0x0040B060 — final cleanup
  │
  ├── Unlock surface: 0x005123A0
  └── Return
```

### R.5 Software Rendering Pipeline

The game uses a **software renderer** with a command buffer architecture:

#### Render Command Buffer System

| Function | Address | Purpose |
|----------|---------|---------|
| `RenderCmd_AllocateBuffer` | 0x0052D430 | Allocate command buffer memory |
| `RenderCmd_WriteData` | 0x0052D380 | Write raw data to buffer |
| `RenderCmd_WriteSpriteData` | 0x0052D550 | Write sprite draw command |
| `RenderCmd_ReadNext` | 0x00512760 | Read next command from buffer |
| `RenderCmd_GetCount` | 0x00512860 | Get number of pending commands |
| `RenderCmd_SubmitSimple` | 0x00512930 | Submit simple draw command |
| `RenderCmd_SubmitComplex` | 0x005129E0 | Submit complex draw command (terrain etc) |
| `RenderCmd_SubmitSprite` | 0x00512B50 | Submit sprite draw command |
| `RenderCmd_ProcessType2` | 0x00513000 | Process type-2 commands |
| `Render_ProcessCommandBuffer` | 0x005125D0 | Main command dispatcher |

The command buffer (`Render_ProcessCommandBuffer` @ 0x005125D0) reads commands and dispatches them:
- **Type 1**: Standard draw — dispatches through a vtable of draw callbacks (offset 0x00, 0x04, 0x08, 0x0C, 0x10)
  - Callback selection based on sub-type (0x10..0x16 range handled via jump table)
  - Callbacks at `[EDI]`, `[EDI+4]`, `[EDI+8]`, `[EDI+0xC]`, `[EDI+0x10]` for different rendering modes
- **Type 2**: Complex rendering (`RenderCmd_ProcessType2`)
- **Type 3**: Sprite rendering — dispatches through `[EDI+8]` callback

The vtable pointer at `[ESP+0x20]` is set by the caller (e.g. `Game_RenderWorld`), allowing different rendering backends.

### R.6 Terrain Rendering

#### Orchestrator: `Terrain_RenderOrchestrator` @ 0x0046AC90

This is the main terrain rendering entry point, called each frame:

```
Terrain_RenderOrchestrator
  ├── Record frame timestamp → g_frame_start_time (0x00686850)
  ├── Camera_SetupProjection @ 0x0046EDB0 — setup view matrix, clip planes
  ├── Scanline loop (up to 0xDC = 220 scanlines):
  │     ├── Terrain_GenerateVertices @ 0x0046DC10 — transform terrain vertices
  │     ├── Terrain_GenerateTriangles @ 0x0046E0F0 — triangulate visible cells
  │     └── Advance scanline counters
  ├── Terrain_ProcessVisibleObjects @ 0x0042C170 — find objects in view
  ├── Terrain_RenderSpecialCells @ 0x00475530 — render water/lava/special terrain
  ├── Object_RenderHighlight @ 0x00476770 — selected object highlight
  ├── Render_ProcessSelectionHighlights @ 0x00476E40
  ├── Render_ProcessUnitMarkers @ 0x004771A0
  ├── Render_Process3DModels @ 0x00487E30 — 3D mesh rendering (buildings/units)
  ├── Render_ProcessDepthBuckets_Main @ 0x0046AF00 — depth-sorted rendering
  ├── Terrain_FinalizeRender @ 0x00473A70
  └── Terrain_PostRenderCleanup @ 0x0046EFE0
```

#### Terrain Cell System

The terrain is divided into a grid of cells, each stored as an 8-byte record in `g_terrain_cell_array` (0x007B9170):

```
Offset 0: byte — cell X coordinate (half-resolution: multiply by 2 for world coords)
Offset 1: byte — cell Z coordinate (half-resolution)
Offset 2: byte — flags (bit 0: active, bit 1: rendered, bit 2: dirty)
Offset 3: byte — age counter (for progressive rendering priority)
Offset 4: word — prev link (doubly-linked list)
Offset 6: word — next link (doubly-linked list)
```

The cell grid is 128x128 half-cells = 256x256 world cells. Cells are linked into a free list for allocation.

An index map at `g_terrain_cell_index_map` (0x007B9160) maps (x,z) → cell index. Value -1 = no cell, -2 = cell marked as boundary.

#### Terrain Rendering (Per-Cell)

The main cell renderer at `Render_Process3DModels` (0x00487E30) handles cells marked for rendering:

For each active cell:
1. Read cell coordinates, multiply by 2 for world position
2. Look up heightmap data from `g_terrain_heightmap_ptr` (0x00599AC0)
   - Heightmap organized in 8x8 blocks (256 bytes per block for high-res, or 16x16 for low-res)
   - Cell index → block: `(idx & 7) << 5` for X, `(idx & ~7) << 10` for Z offset
3. Load 33x33 height samples (32x32 quad + 1 border) into local buffer on stack
4. Compute height gradients and normals per-vertex
5. For each 32x32 pixel of the cell:
   - Bilinear interpolate height values
   - Look up terrain texture via `g_terrain_texturemap_ptr` (0x00599ADC)
   - Apply shade lookup: `g_terrain_shade_table[height_gradient + shade_offset]`
   - Apply fog lookup: `g_terrain_fog_table[(depth >> 18) << 7 + shaded_color]`
   - Write final pixel to output buffer (0x100 = 256 bytes stride per cell row)
6. Overlay objects on cell using object-per-cell lookup table at `0x007B916C`

The terrain uses **two rendering paths**:
- Flag 0x80 at 0x0087E33F: High-resolution mode (8x8 blocks, `0x00489360`)
- Default: Low-resolution mode (16x16 blocks, `0x00489EA0`)

#### Topmap (Pre-rendered Texture Map)

`set_topmap_onoff` @ 0x00486FF0:
- Loads `data\topmap.wad` — a WAD file containing pre-rendered terrain textures
- Two WAD handles stored at 0x007B9130 and 0x007B9148 (hi-res/lo-res versions)
- Topmap textures are stored as count at `[WAD+4]`, shifted right by 10: `count = WAD_total_size >> 10`
- Flag at 0x00599AB8 tracks state: bit 0 = loaded, bit 1 = hi-res variant loaded

### R.7 Depth-Sorted Rendering

#### Depth Bucket System

Objects and sprites are sorted into depth buckets for correct painter's-algorithm rendering:

- `g_depth_buckets` @ 0x00699A64: Array of 0xE01 (3585) bucket slots, each a linked list head pointer
- `g_depth_bucket_alloc_ptr` @ 0x00699A60: Current allocation pointer within bucket entries
- Each bucket entry is 14 (0x0E) bytes:
  ```
  Offset 0: byte — render type/command ID
  Offset 2: dword — next entry pointer (linked list)
  Offset 6: dword — object pointer
  Offset A: word — screen X
  Offset C: word — screen Y
  ```

`Object_SubmitToDepthBucket` @ 0x00470030:
1. Takes object pointer and terrain cell info
2. Reads object world position (offsets +0x3D, +0x3F, +0x41) and size (offsets +0x43, +0x45, +0x47)
3. Computes position relative to camera: subtracts camera angles at `[g_camera_data_ptr + 0x24]` and `[g_camera_data_ptr + 0x26]`
4. Applies interpolation for smooth movement between game ticks
5. Calls `Camera_WorldToScreen` (0x0046EA30) to project to screen coords
6. Computes depth key: `(worldY + 0x7000 - 0x12C) >> 4`, clamped to [0, 0xE00]
7. Inserts entry into `g_depth_buckets[depth_key]` as linked list head

`Render_ProcessDepthBuckets_Main` @ 0x0046AF00 (very large function):
- Iterates buckets front-to-back (painter's algorithm)
- For each bucket entry, dispatches based on render type byte

#### Object Dispatch: `Render_DispatchObjectsByType` @ 0x0046FC30

This is the core dispatcher for rendering objects found on terrain cells:

For each object in the cell's linked list (at `g_object_pool[obj_idx]`):
1. Check visibility flags at object+0x35 (bit 4: skip, bit 0: already rendered)
2. Read object type from lookup table at `0x0059F8D8` (indexed by object+0x3A byte)
3. **Type dispatch** (two passes — 0 and 1):

Pass 0 (pre-rendered objects):
| Type | Action |
|------|--------|
| 2 (BUILDING) | Mark visible, call `[0x006868A0]` (configurable renderer), submit health bar |
| 3 (CREATURE) | Process projectile/shot rendering |
| 4 (VEHICLE) | Mark visible, call `[0x006868A4]` (vehicle renderer) |
| 5 (SCENERY) | If tree subtype: `0x004737C0` (tree renderer) |
| 6 (GENERAL) | `0x00471040` (general object renderer) + health bar |
| 7 (EFFECT) | `0x00472330` (effect renderer) |
| 8/10 (SHOT/INTERNAL) | `0x00474E80` with terrain flags |
| 9 (SHAPE) | `0x00470B60` (shape renderer) |
| 11 (SPELL) | `0x00470210` → `Render_SubmitGrassPatches` (grass/ground effects) |

Pass 1 (terrain-level objects):
| Type | Action |
|------|--------|
| 1 (PERSON) | `Object_SubmitToDepthBucket` (for sorted rendering) + health bar |
| 2..6 | Various handlers for different object categories |
| 0xD | `Object_SubmitToDepthBucket` for special objects |
| 0xE..0x13 | Fire trail/particle, tree variations, scenery |

The function pointers at `0x006868A0` and `0x006868A4` are configurable — set by `Camera_SetupProjection` based on view mode flags. They can point to:
- `0x00471730` — standard 3D model renderer
- `0x00472A80` — alternative renderer (possibly LOD)
- `0x00473BC0` — no-op/skip renderer

### R.8 Camera System

#### Key Functions

| Function | Address | Purpose |
|----------|---------|---------|
| `Camera_WorldToScreen` | 0x0046EA30 | 3D world → 2D screen projection |
| `Camera_SetupProjection` | 0x0046EDB0 | Configure projection parameters per frame |
| `Camera_ApplyRotation` | 0x0046F2A0 | Apply camera rotation to 4-corner frustum |
| `Camera_GenerateProjectionLUT` | 0x0046F1E0 | Generate projection lookup tables |
| `Camera_Initialize` | 0x00422130 | Initial camera setup |
| `Camera_SetViewportOffsets` | 0x00421C70 | Set viewport center offsets |
| `Camera_UpdateZoom` | 0x004227A0 | Handle zoom changes |

#### World-to-Screen Projection: `Camera_WorldToScreen` @ 0x0046EA30

Input: struct at ESI with {X, Y, Z} as 32-bit ints (world coords)
Output: overwrites struct with {rotX, rotY, rotZ, screenX, screenY, ...flags}

Algorithm:
1. **3x3 rotation**: Apply camera rotation matrix stored at `g_camera_matrix` (0x006868AC, 9 ints = 36 bytes):
   ```
   rotX = X * M[0][0] + Z * M[0][2]    >> 14
   rotY = Y * M[1][0] + X * M[1][1] + Z * M[1][2]    >> 14
   rotZ = Y * M[2][0] + X * M[2][1] + Z * M[2][2]    >> 14
   ```
   (14-bit fixed-point matrix entries)

2. **Perspective correction**:
   ```
   dist² = (2*rotX)² + (2*rotZ)²
   correction = dist² * g_camera_perspective_correction (0x007B8FB4) >> 16
   rotY -= correction
   ```
   This creates the isometric-ish perspective distortion.

3. **Perspective divide**:
   ```
   depth = g_camera_near_plane (0x007B8FBC) + rotZ
   if depth > 0:
     scale = (1 << (g_camera_projection_shift + 16)) / depth
     screenX = g_screen_center_x + (rotX * zoom * scale >> 16)
     screenY = g_screen_center_y - (rotY * zoom * scale >> 16)
   ```
   The zoom factor is at `[g_camera_data_ptr + 0x2A]` (16-bit fixed point).

4. **Clip flags** (stored at output+0x18):
   - Bit 1: screenX < 0 (left clip)
   - Bit 2: screenX >= viewport_right (right clip)
   - Bit 3: screenY < 0 (top clip)
   - Bit 4: screenY >= viewport_bottom (bottom clip)

#### Camera Setup: `Camera_SetupProjection` @ 0x0046EDB0

1. Selects rendering function pointers based on mode flags:
   - Flag 0x40 at 0x0087E33C → sets table pointer to 0x00473BB0 (optimized path)
   - Flag 0x80 → sets all renderers to 0x00473BC0 (skip/no-op)
2. Reads camera angles from `[g_camera_data_ptr + 0x24]` and `[g_camera_data_ptr + 0x26]`
3. Computes vertical clip ranges from angles (shifted, masked to 0x1FE, divided by 2)
4. Copies 36 bytes of camera matrix data from `g_camera_data_ptr` to `g_camera_matrix`
5. Sets up scanline rendering parameters (scanline count = 0xDC = 220)
6. Configures fog-of-war rendering flag at 0x00686854/0x00686848

#### Camera Rotation: `Camera_ApplyRotation` @ 0x0046F2A0

1. Reads 4 corner points from camera struct at offset +0x44 (4 x 2 words = 16 bytes)
2. If camera has Y-rotation (at `[g_camera_data_ptr + 0x32]`):
   - Computes sin/cos from lookup table at 0x005AC6A0 / 0x005ACEA0 (2048-entry tables, 0x800 entries)
   - Angle = `-rotation & 0x7FF`, used as index
   - Applies 2D rotation to each corner: `x' = x*cos - y*sin`, `y' = x*sin + y*cos`
   - Adds 0x6E (110) bias to each coordinate
3. Clamps all corners to [1, 0xDC] range (1 to 220)
4. Rasterizes the 4 edges into a scanline span buffer at 0x0069D268 (220 entries × 4 bytes each)
   - Each entry: {min_x: word, max_x: word} — the left/right terrain column for that scanline

#### Key Camera Globals

| Address | Name | Purpose |
|---------|------|---------|
| 0x006868A8 | `g_camera_data_ptr` | Pointer to active camera struct |
| 0x006868AC | `g_camera_matrix` | 3x3 rotation matrix (9 ints, 14-bit fixed point) |
| 0x007B8FBC | `g_camera_near_plane` | Near plane distance for perspective divide |
| 0x007B8FC0 | `g_camera_projection_shift` | Bit shift for projection scale |
| 0x007B8FB4 | `g_camera_perspective_correction` | Quadratic perspective distortion factor |
| 0x007B8FFC | `g_screen_center_x` | Screen center X (typically width/2) |
| 0x007B8FFE | `g_screen_center_y` | Screen center Y (typically height/2) |
| 0x007B8FE8 | `g_viewport_right` | Right clip boundary |
| 0x007B8FEA | `g_viewport_bottom` | Bottom clip boundary |
| 0x005AC6A0 | (sin_table) | 2048-entry sine LUT (16-bit fixed) |
| 0x005ACEA0 | (cos_table) | 2048-entry cosine LUT (16-bit fixed) |
| 0x0069D268 | (scanline_spans) | 220-entry span buffer for terrain rasterization |

### R.9 Sprite System

| Function | Address | Purpose |
|----------|---------|---------|
| `Sprite_LoadResources` | 0x0041DB20 | Load all sprite banks |
| `Sprite_LoadBank` | 0x00450990 | Load one sprite bank from disk |
| `Sprite_LoadFromDisk` | 0x0041E790 | Low-level disk read |
| `Sprite_InitAnimationTables` | 0x00451B50 | Setup anim frame tables |
| `Sprite_RenderObject` | 0x00411C90 | Render a game object as sprite (very large) |
| `Sprite_RenderWithShadow` | 0x00411B70 | Render sprite + shadow |
| `Sprite_BlitStandard` | 0x0050EDD0 | Standard blit (no scaling) |
| `Sprite_BlitScaled` | 0x0050F6E0 | Scaled blit |
| `Sprite_SetResolutionParams` | 0x00451FF0 | Adjust for screen resolution |
| `draw_sprite_rle` | 0x004E0000 | RLE sprite decoder + renderer |
| `Shadow_CalculateOffset` | 0x00416410 | Compute shadow position |

#### RLE Sprite Renderer: `draw_sprite_rle` @ 0x004E0000

RLE format:
- Byte stream in sprite data
- If byte < 0: **skip** (-byte) pixels (transparent run)
- If byte > 0: **draw** (byte) pixels of data from following bytes
- If byte == 0: **newline** — advance to next scanline

For each opaque pixel:
1. Read color index from sprite data
2. Look up through a user callback (`[ESP+0x18]`, typically a shade/color transform function)
3. Shift result left by 8, add the existing screen pixel value
4. Index into `g_color_lookup_table` (0x0096CF04) for the final blended color
5. Write to screen buffer

This implements **alpha blending/shading** via lookup tables — the sprite color and the background are combined through a 64KB blend table.

Two rendering paths exist:
- `0x004E0100`: Standard render callback
- `0x004E00F0`: Alternate callback (for highlight/selection rendering)

### R.10 Render Target / Bit Depth System

| Function | Address | Purpose |
|----------|---------|---------|
| `Render_SetupColorMasks` | 0x0050F5F0 | Setup pixel format for render target |
| `Render_SetBitDepthVtable` | 0x0050F520 | Select rendering routines by bpp |
| `Render_InitColorTable` | 0x0050F300 | Initialize clip region + pixel pointer |
| `Render_SetupViewportClipping` | 0x0050F390 | Set clipping rectangle |
| `Render_CalculateBufferOffset` | 0x0052B950 | Compute pixel address from coords |
| `Render_SetupBitMasks` | 0x0052B9E0 | Compute RGB bitmasks |

`Render_SetupColorMasks` @ 0x0050F5F0:
- Reads surface descriptor from DirectDraw lock result
- Stores pixel pointer, pitch, width, height, and bpp
- Selects pixel write function pointer at `g_pixel_write_func` (0x009735B8) based on bit depth:
  - 8bpp: function table at 0x009735D8
  - 16bpp: function table at 0x009735E0
  - 24bpp: function table at 0x009735E4
  - 32bpp: function table at 0x009735E8
- Sets up RGB bitmasks from the surface's pixel format descriptor

### R.11 Font System

| Function | Address | Purpose |
|----------|---------|---------|
| `Font_LoadFiles` | 0x004A20B0 | Load font data files |
| `Font_SetCurrentSize` | 0x004A02B0 | Select font size |
| `Font_RenderString` | 0x004A0310 | Render a complete string |
| `Font_RenderSmallChar` | 0x004A0420 | Render one char (small font) |
| `Font_RenderLargeChar` | 0x004A07C0 | Render one char (large font) |
| `Font_Render8bit` | 0x0050FC20 | 8bpp font renderer |
| `Font_DrawAtPosition8bit` | 0x0050FAE0 | Draw string at x,y in 8bpp mode |
| `Font_GetWidth8bit` | 0x0050FCC0 | Measure string width (8bpp) |
| `Font_GetWidth16bit` | 0x004A0D60 | Measure string width (16bpp) |

`DrawFrameRate` @ 0x004A6BF0:
- Formats "Frame Rate: %03ld (%03ld)" using values from 0x0057C17C (current FPS) and 0x0057C180 (average FPS)
- Renders at top-left corner (position 1,1) using current font

### R.12 Water System

| Function | Address | Purpose |
|----------|---------|---------|
| `Water_SetupMesh` | 0x0048E730 | Initialize water mesh vertices |
| `Water_AnimateMesh` | 0x0048E210 | Animate water vertex positions |
| `Water_UpdateWavePhase` | 0x0048E990 | Advance wave phase for animation |
| `Water_RenderObjects` | 0x004A75F0 | Render objects on/in water |
| `Terrain_SetupWaterWaveParams` | 0x0048EBB0 | Configure wave amplitude/frequency |
| `Tick_UpdateWater` | 0x0048BF10 | Per-tick water state update |

### R.13 Minimap System

| Function | Address | Purpose |
|----------|---------|---------|
| `Minimap_Update` | 0x0042B950 | Full minimap update |
| `Minimap_RenderTerrain` | 0x0042BA10 | Render terrain to minimap |
| `Minimap_RenderObjects` | 0x0042BBE0 | Render objects on minimap |
| `Minimap_DrawSprite` | 0x00494CF0 | Draw sprite on minimap surface |
| `Minimap_UpdateDirtyRegion` | 0x0042BFF0 | Incremental minimap update |
| `Minimap_GetBounds` | 0x0045AA50 | Get minimap screen rect |

### R.14 UI Rendering

| Function | Address | Purpose |
|----------|---------|---------|
| `UI_RenderGamePanel` | 0x00492390 | Main game panel (bottom bar) |
| `UI_RenderBuildingInfo` | 0x004937F0 | Building info popup |
| `UI_RenderResourceDisplay` | 0x00493350 | Mana/follower counts |
| `UI_RenderStatusText` | 0x00493560 | Status text messages |
| `UI_RenderObjectiveDisplay` | 0x00492E30 | Level objectives |
| `UI_RenderPanelBackground` | 0x004C3B40 | Panel backdrop |
| `UI_RenderMultiplayerStatus` | 0x004AE700 | MP game status |

### R.15 3D Drawing Primitives

| Function | Address | Purpose |
|----------|---------|---------|
| `Draw_Pixel` | 0x0050EF70 | Single pixel |
| `Draw_FilledRect` | 0x0050EF90 | Filled rectangle |
| `Draw_Line` | 0x0050F050 | Line drawing |
| `Render_DrawRotatedQuad` | 0x0040A560 | Textured rotated quad |
| `Render_DrawLayer` | 0x004A0230 | Draw a rendering layer |
| `Render_DrawCharacter` | 0x004A0570 | Draw a character model |

### R.16 Effect System (Visual FX)

| Function | Address | Purpose |
|----------|---------|---------|
| `Effect_Init` | 0x004A6F50, 0x004F0E20 | Initialize effect (two overloads) |
| `Effect_InitBlast` | 0x004F3170 | Init blast spell visual |
| `Effect_InitBurn` | 0x004F2840 | Init burn spell visual |
| `Effect_InitConversion` | 0x004F3590 | Init conversion visual |
| `Effect_QueueVisual` | 0x00453780 | Add visual effect to render queue |
| `Effect_SortQueue` | 0x00453A10 | Sort effects by depth |
| `Effect_SetState` | 0x004F1950 | Update effect state |
| `Effect_Update` | 0x0049E110 | Per-tick effect update |
| `Game_RenderEffects` | 0x004A6BE0 | Entry point for effect rendering |
| `Render_PostProcessEffects` | 0x00467890 | Post-process (glow, etc) |

### R.17 Summary of Rendering Architecture

Populous: The Beginning uses a **fully software-rendered** pipeline with DirectDraw only for page-flipping:

1. **Double-buffered display**: Primary surface + back buffer via DirectDraw. All rendering happens to a software framebuffer (`g_software_framebuffer` @ 0x0096CF10), which is then memcpy'd to the locked DirectDraw surface.

2. **Command buffer architecture**: The game builds a list of render commands during the terrain traversal phase, then processes them during the render phase. This allows depth sorting and batching.

3. **Painter's algorithm**: Objects are sorted into 3585 depth buckets (0 to 0xE00), then rendered front-to-back within the terrain. Each bucket is a linked list of 14-byte draw entries.

4. **8bpp and 16bpp support**: The renderer has configurable function pointers for different bit depths. The pixel write function, blit routines, and font renderers all have per-bpp variants.

5. **Terrain**: Rendered in 32×32 pixel cells. Each cell samples a heightmap, applies texture mapping from a separate texture map, then composites shade and fog via lookup tables. Two LOD levels exist (8x8 and 16x16 block modes).

6. **Sprites**: RLE-encoded with LUT-based alpha blending. The color of each sprite pixel is combined with the background through a 64KB color blend table.

7. **3D Models**: Processed per-cell, projected via the camera system, and submitted to depth buckets for sorted rendering. Building/unit models go through configurable render function pointers.

8. **Fixed-point math**: Extensive use of 14-bit and 16-bit fixed-point arithmetic throughout. Trig lookups via 2048-entry sin/cos tables.

---

### R.18 Terrain Triangle Pipeline (Detailed)

#### Triangle_CullAndCrossProduct @ 0x0046E870

Combines **Cohen-Sutherland viewport clipping** with a **backface test** for terrain triangles.

**Parameters**: 3 vertex pointers (each a terrain vertex struct).

**Outcode computation** (per vertex):
- Bit 0x02: screen_x < 0 (left clip)
- Bit 0x04: screen_x >= `g_viewport_right` (0x007B8FE8)
- Bit 0x10: screen_y >= `g_viewport_bottom` (0x007B8FEA)

**Trivial reject**: If `outcode_A & outcode_B & outcode_C != 0`, all vertices lie on the same side of a clip boundary → return 0 (culled).

**Cross product** (2D signed area):
```
cross = (v2.screen_x - v1.screen_x) * (v0.screen_y - v1.screen_y)
      - (v2.screen_y - v1.screen_y) * (v0.screen_x - v1.screen_x)
```
Returns the cross product value. Positive = front-facing, negative/zero = back-facing.

#### Terrain Vertex Structure (32 bytes, stride 0x20)

| Offset | Size  | Field |
|--------|-------|-------|
| +0x00  | dword | world X (view-space, after rotation) |
| +0x04  | dword | world Y (view-space) |
| +0x08  | dword | world Z (view-space, used for depth) |
| +0x0C  | dword | screen X (after projection) |
| +0x10  | dword | screen Y (after projection) |
| +0x14  | dword | shade value (brightness, pre-fog) |
| +0x18  | dword | flags (bit 0x40 = water, bit 0x80 = fog-affected) |

#### Terrain_CreateTriangleCommand @ 0x0046F6F0

Creates a **0x46-byte render command** (type 0) and inserts it into the depth bucket linked list.

**Depth bucket index calculation**:
```
sum = vertex0.z + vertex1.z + vertex2.z + 0x15000
index = (sum + sum*21 + sum*84) >> 8   // nonlinear depth mapping
if (any vertex flag & 0x40)  index += 0x100   // water pushed back
index = clamp(index / 16, 0, 0xE00)
```

**Fog computation** (per vertex, if vertex flag bit 0x80 is set):
```
dist = vertex.z - g_fog_start_distance        // 0x008853DD
if (dist > 0):
    fog = (dist² * g_fog_density) >> 16        // g_fog_density @ 0x008853D9
    fog = clamp(fog, 0, 0x20)
    shade -= fog
```

When flag 0x80 is NOT set, uses alternate fog path:
```
dist = vertex.z - g_fog_start_distance_far     // 0x008853E1
if (dist > 0 && shade > 0x20):
    fog = (dist² * g_fog_density_alt) >> 16    // g_fog_density_alt @ 0x008853DA
    shade = (shade - 0x20) * fog + 0x20
    shade = clamp(shade, 0, 0x3F)
```

Final shade values are shifted left 16 for 16.16 fixed-point interpolation during rasterization.

**Depth bucket tracking**: Updates `g_depth_bucket_min_used` (0x007B9110) and `g_depth_bucket_max_used` (0x007B911C) for efficient traversal.

---

### R.19 Camera Projection (Detailed)

#### Camera_ProjectVertex @ 0x0046EBD0

Transforms a world-space vertex to screen coordinates via the camera.

**Step 1: 3x3 Matrix Rotation** (14-bit fixed-point)
```
Camera matrix at g_camera_matrix (0x006868AC), 9 dwords:
  [0] [3] [6]     Row-major 3x3
  [1] [4] [7]
  [2] [5] [8]

out_x = (mat[2]*z + mat[0]*x) >> 14
out_y = (mat[5]*z + mat[4]*y + mat[3]*x) >> 14
out_z = (mat[8]*z + mat[7]*y + mat[6]*x) >> 14
```

**Step 2: Barrel Distortion Correction**
```
r² = (2*out_x)² + (2*out_z)²
correction = (r² * g_camera_perspective_correction) >> 32   // 0x007B8FB4
out_y -= correction
```
This reduces the "fish-eye" appearance at screen edges.

**Step 3: Perspective Divide**
```
depth = out_z + g_camera_near_plane                // 0x007B8FBC
if (depth > 0):
    scale = (1 << (g_camera_projection_shift + 16)) / depth  // 0x007B8FC0
    // Aspect ratio from camera data at [g_camera_data_ptr + 0x2A]
    proj_x = (aspect * out_x) >> 16
    proj_y = (aspect * out_y) >> 16
    // 16.16 fixed-point perspective multiply
    screen_x = g_screen_center_x + (proj_x * scale) >> 16
    screen_y = g_screen_center_y - (proj_y * scale) >> 16
else:
    // Behind camera — push off-screen
    screen_x = -g_screen_center_x * 100
    screen_y = -g_screen_center_y * 100
```

**Output** written to vertex struct:
- `+0x00, +0x04, +0x08`: view-space x, y, z
- `+0x0C`: screen X
- `+0x10`: screen Y

---

### R.20 3D Model Rendering Pipeline

#### Model3D_RenderObject @ 0x00471730

The complete pipeline for rendering a single 3D mesh object. ~1600 bytes of code.

**Phase 1: Vertex Transform**

1. Read animation deltas from object at `+0x3D/+0x43/+0x45/+0x47` (bone positions)
2. If animation flag (bit 0x100 at `+0x14`): apply time-based animation blending
   - `blend = (g_current_game_turn - object.timestamp) * g_anim_blend_factor / g_anim_blend_duration`
   - Applied to all 3 axis deltas
3. Per-vertex loop (vertex count from object descriptor `+0x04`):
   - Read 3 signed words from point data (6-byte stride)
   - Scale by `object.scale` at `+0x18`, shift right 8
   - Store to `g_transformed_vertex_buffer` (0x0068A050), 32 bytes per vertex

**Phase 2: Optional Rotation**

If rotation flag set, applies 3x3 matrix rotation:
- Calls `Matrix3x3_Identity` (0x00450320) to initialize
- Applies X/Y/Z rotations via `Matrix3x3_RotateX/Y/Z`
- Multiplies each vertex by the rotation matrix (14-bit fixed-point `>> 14`)

**Phase 3: World Positioning**

Adds camera-relative offset with **world wrapping** (torus topology):
```
dx = object.world_x - camera.world_x
if (abs(dx) > 0x8000):   // wraps around 64K world
    dx = 0x10000 - abs(dx)  (or negative equivalent)
// Same for dz
vertex.x += dx/2
vertex.z += dz/2
```

**Phase 4: Height Sampling**

For vertices touching terrain (flag bit 0x2 at `g_terrain_cell_data[cell*4 + 2]`):
- Calls terrain height lookup at 0x004E8E50
Otherwise uses static Y offset from `+0x24` (word).

**Phase 5: Projection**

Calls `Camera_ProjectVertex` (0x0046EBD0) for each vertex.

**Phase 6: Wind Effect**

If object has wind flag at `+0x65`, calls `Model3D_ApplyVertexWind` (0x00477640):
- Radial outward push proportional to vertex height
- Uses atan2 + sin/cos for direction
- Sway counter cycles 0-4 for animation

**Phase 7: Face Rendering**

Face records are 60 bytes (0x3C) each. For each face:
- Look up 4 vertex pointers: `g_transformed_vertex_buffer + index * 0x20`
- Look up color index from `g_face_color_index_table` (0x00884226)
- Call `Triangle_CullAndCrossProduct` — skip if back-facing
- Submit triangle to depth buckets:
  - Normal faces: `Model3D_SubmitTriangle` (0x00472720) → type 0x06
  - Tribal-colored faces: `Model3D_SubmitTriangle_Tribal` (0x004728D0) → type 0x06 with tribal palette offset
- Quads: second triangle [v2, v3, v0] submitted separately

**Phase 8: Extras**

- Shadow casting (flag `+0x35` bit 0x80): `Model3D_SubmitShadow` (0x00476330) → type 0x15
- Selection highlight: `Model3D_SetupSelectionBuffer` + `Model3D_DrawSelectionEdge` + `Model3D_SubmitSelectionHighlight` → type 0x16

---

### R.21 Render Command Types

All render commands are allocated from a linear allocator (`g_depth_bucket_alloc_ptr` @ 0x00699A60, limit at `g_depth_bucket_alloc_limit` @ 0x00699A5C) and inserted into depth bucket linked lists (`g_depth_buckets` @ 0x00699A64, 3585 slots).

| Type | Size | Name | Description |
|------|------|------|-------------|
| 0x00 | 0x46 | Terrain Triangle | Gouraud-shaded terrain tri with per-vertex fog |
| 0x06 | 0x46 | Model Triangle | Textured/shaded 3D model face |
| 0x15 | 0x12 | Shadow | Ground shadow bounding box |
| 0x16 | 0x0A | Selection Highlight | Selection circle composite |

#### Model Triangle Command (0x46 bytes, type 0x06)

| Offset | Size  | Field |
|--------|-------|-------|
| +0x00  | byte  | type = 6 |
| +0x01  | byte  | sub-type = 0 |
| +0x02  | dword | next pointer (linked list) |
| +0x06  | dword | vertex 0 screen X |
| +0x0A  | dword | vertex 0 screen Y |
| +0x0E  | dword | vertex 0 UV U |
| +0x12  | dword | vertex 0 UV V |
| +0x16  | dword | shade value (16.16 FP) |
| +0x1A  | dword | vertex 1 screen X |
| +0x1E  | dword | vertex 1 screen Y |
| +0x22  | dword | vertex 1 UV U |
| +0x26  | dword | vertex 1 UV V |
| +0x2A  | dword | shade value (16.16 FP) |
| +0x2E  | dword | vertex 2 screen X |
| +0x32  | dword | vertex 2 screen Y |
| +0x36  | dword | vertex 2 UV U |
| +0x3A  | dword | vertex 2 UV V |
| +0x3E  | dword | shade value (16.16 FP) |
| +0x42  | word  | texture index |
| +0x44  | byte  | material palette index |
| +0x45  | byte  | face flags |

#### Shadow Command (0x12 bytes, type 0x15)

| Offset | Size  | Field |
|--------|-------|-------|
| +0x00  | byte  | type = 0x15 |
| +0x02  | dword | next pointer |
| +0x06  | dword | object pointer |
| +0x0A  | word  | bounding min X |
| +0x0C  | word  | bounding min Y |
| +0x0E  | word  | bounding max X |
| +0x10  | word  | bounding max Y |

---

### R.22 3x3 Matrix Library

All matrices use **14-bit fixed-point** (1.0 = 0x4000 = 16384).

| Function | Address | Description |
|----------|---------|-------------|
| Matrix3x3_Identity | 0x00450320 | Set to identity (diagonal = 0x4000) |
| Matrix3x3_Multiply | 0x004BC060 | C = A × B (9 dot products, >> 14) |
| Matrix3x3_RotateX | 0x004BC1E0 | Rotate around X axis by angle |
| Matrix3x3_RotateY | 0x004BC260 | Rotate around Y axis by angle |
| Matrix3x3_RotateZ | 0x004BC2E0 | Rotate around Z axis by angle |
| Matrix3x3_RotateArbitrary | 0x004BC360 | Rodrigues rotation + re-orthogonalization |
| Vector3_TransformByRow | 0x004BC000 | Transform vector by matrix row |
| Math_IntegerSqrt | 0x00564000 | Integer square root |
| Math_Atan2 | 0x00564074 | Atan2 (returns 2048-based angle) |

**Angle system**: 2048 units = 360°. Sin/cos tables:
- `g_sin_table_2048` @ 0x005AC6A0 (2048 dwords)
- `g_cos_table_2048` @ 0x005ACEA0 (2048 dwords)

**Matrix3x3_RotateArbitrary** uses Rodrigues' formula and performs **re-orthogonalization** after rotation:
1. Computes R = cos·I + (1-cos)·axis⊗axis + sin·[axis]×
2. Cross product of rows to regenerate orthogonal row
3. Normalizes each row via `Math_IntegerSqrt` to maintain unit length

---

### R.23 Command Buffer Dispatch

#### Render_ProcessCommandBuffer @ 0x005125D0

The central command dispatcher for the software renderer.

**Dispatch by type byte** (`+0x00`):
- **Type 1**: Polygon rendering — further dispatched by sub-type:
  - Sub-types 0xF0–0xF6: jump table at 0x00512740 (special renderers)
  - Other sub-types: indirect call through **renderer vtable** at EDI:
    - `[vtable+0x00]`: flat-shaded triangle
    - `[vtable+0x04]`: textured triangle
    - `[vtable+0x08]`: type 3 renderer
    - `[vtable+0x0C]`: gouraud-shaded triangle
    - `[vtable+0x10]`: textured + gouraud-shaded triangle
- **Type 2**: Special blit operation (calls 0x00513000)
- **Type 3**: Indirect call through `[vtable+0x08]`

The vtable pointer allows **runtime switching** between 8bpp and 16bpp renderers — each has its own set of triangle rasterizer function pointers.

---

### R.24 Rotated Quad Rendering

#### Render_DrawRotatedQuad @ 0x0040A560

Draws a **billboard/rotated textured quad** (used for spell effects, particles, etc.):

1. Sets up clip rect to screen dimensions
2. Reads object rotation angle from `+0x6F` (index into 2048-entry sin/cos table)
3. Computes `half_size = (object[+0x7D] * object[+0x73]) << 5 / 256 / 2`
4. Generates 4 rotated corner positions:
   ```
   for each corner (±half_size, ±half_size):
       screen_x = center_x + (cos * dx - sin * dy) >> 16
       screen_y = center_y + (sin * dx + cos * dy) >> 16
   ```
5. Reads UV coordinates from `+0x75/+0x77/+0x79/+0x7B`, converted to 16.16 FP
6. Loads texture: `g_current_texture_ptr = g_texture_base_ptr + 0x400`
7. Splits quad into 2 triangles, calls rasterizer function pointer at 0x0097C000

---

### R.25 Wind/Sway Effect System

#### Model3D_ApplyVertexWind @ 0x00477640

Simulates **wind-driven vertex displacement** for trees, buildings, and other objects:

1. Object maintains a sway counter at `+0x65` (cycles 0→4, resets at 5)
2. For each vertex:
   - `height = vertex.y - base_height` (clamped 0-256)
   - `strength = height * 20 / 256` (taller = more sway)
   - `dist² = dx² + dz²` from vertex to object center (clamped to 0x100000)
   - `factor = (0x100000 - dist²) * strength >> 20` (falloff with distance)
   - `angle = atan2(-dz, dx)` → direction from center
   - `vertex.x += sin(angle) * factor >> 16`
   - `vertex.z += cos(angle) * factor >> 16`

The effect shifts by the sway phase counter, creating periodic oscillation. The strength increases with height — tree tops sway more than trunks.

---

### R.26 Selection Highlight System

The selection circle rendered around selected units/buildings uses a **3-phase approach**:

1. **Buffer setup** (`Model3D_SetupSelectionBuffer` @ 0x00476430):
   - Computes bounding box of all projected vertices
   - Expands by 8px, rounds to 4-pixel alignment
   - Clears a temporary 8bpp stencil buffer (same as `g_texture_base_ptr`)
   - Sets up render target descriptor for the stencil buffer

2. **Edge drawing** (`Model3D_DrawSelectionEdge` @ 0x004765A0):
   - Called per-face during Model3D_RenderObject
   - Draws wireframe edges and filled triangles into the stencil buffer
   - Uses line rasterizer (0x00402800) and triangle filler (0x0050F050)
   - Color = 0xFF for stencil mask

3. **Composite** (`Model3D_SubmitSelectionHighlight` @ 0x00476690):
   - Submits type 0x16 command to depth buckets
   - During render, composites the stencil mask over the scene with highlight color

---

### R.27 Complete Function Reference (Rendering)

| Address | Name | Description |
|---------|------|-------------|
| 0x005102E0 | ddraw_init_display | DirectDraw initialization |
| 0x00510110 | ddraw_cleanup | DirectDraw teardown |
| 0x00512310 | ddraw_lock_surface | Lock DDraw surface |
| 0x00510210 | ddraw_get_display_width | Returns display width |
| 0x00512F60 | get_main_window_handle | Returns g_hMainWnd |
| 0x004DF3C0 | render_frame | Core per-frame render |
| 0x004E0000 | draw_sprite_rle | RLE sprite decoder |
| 0x00486FF0 | set_topmap_onoff | Topmap WAD texture load |
| 0x0046AC90 | Terrain_RenderOrchestrator | Terrain render orchestrator |
| 0x0046DC10 | Terrain_GenerateVertices | Terrain vertex generation |
| 0x0046E0F0 | Terrain_GenerateTriangles | Triangle generation from vertices |
| 0x0046E870 | Triangle_CullAndCrossProduct | Cull + backface test |
| 0x0046F6F0 | Terrain_CreateTriangleCommand | Submit terrain tri to depth buckets |
| 0x0046EBD0 | Camera_ProjectVertex | World → screen projection |
| 0x00471730 | Model3D_RenderObject | Full 3D model render pipeline |
| 0x00472720 | Model3D_SubmitTriangle | Submit model tri to depth buckets |
| 0x004728D0 | Model3D_SubmitTriangle_Tribal | Submit tribal-colored tri |
| 0x00476330 | Model3D_SubmitShadow | Submit shadow command (type 0x15) |
| 0x00476430 | Model3D_SetupSelectionBuffer | Selection stencil buffer setup |
| 0x004765A0 | Model3D_DrawSelectionEdge | Draw edge into selection stencil |
| 0x00476690 | Model3D_SubmitSelectionHighlight | Submit selection composite (type 0x16) |
| 0x00477640 | Model3D_ApplyVertexWind | Vertex wind/sway effect |
| 0x0040A560 | Render_DrawRotatedQuad | Rotated billboard quad |
| 0x005125D0 | Render_ProcessCommandBuffer | Main command dispatch |
| 0x0046AF00 | Render_ProcessDepthBuckets_Main | Process all depth buckets |
| 0x0046D9A0 | Render_ProcessDepthBuckets_3DModels | Process model depth buckets |
| 0x00487E30 | Render_Process3DModels | 3D model render pass |
| 0x00476E40 | Render_ProcessSelectionHighlights | Selection highlight compositing |
| 0x004771A0 | Render_ProcessUnitMarkers | Unit marker rendering |
| 0x004C3B40 | Render_SetupClipRect | Set clip rect (with 512x384 handling) |
| 0x004C3BB0 | Render_SetClipRect | Set clip rect from rect struct |
| 0x0050F300 | Render_SetClipRectAndTarget | Set clip rect + compute pixel ptr |
| 0x00450320 | Matrix3x3_Identity | Identity matrix (14-bit FP) |
| 0x004BC000 | Vector3_TransformByRow | Vector × matrix |
| 0x004BC060 | Matrix3x3_Multiply | Matrix × matrix |
| 0x004BC1E0 | Matrix3x3_RotateX | X rotation |
| 0x004BC260 | Matrix3x3_RotateY | Y rotation |
| 0x004BC2E0 | Matrix3x3_RotateZ | Z rotation |
| 0x004BC360 | Matrix3x3_RotateArbitrary | Rodrigues rotation |
| 0x00564000 | Math_IntegerSqrt | Integer square root |
| 0x00564074 | Math_Atan2 | Atan2 (2048-based) |
| 0x00402800 | (line rasterizer) | Line drawing function |
| 0x0050F050 | (triangle filler) | Triangle scan fill |
| 0x0052B950 | (clip bounds) | Clip rect intersection |

### R.28 Complete Global Reference (Rendering)

| Address | Name | Description |
|---------|------|-------------|
| 0x005AEEA8 | g_pDirectDraw | IDirectDraw* |
| 0x005AEEA0 | g_hWnd | Render window HWND |
| 0x00973AE4 | g_pPrimarySurface | IDirectDrawSurface* (front) |
| 0x00973C4C | g_pBackSurface | IDirectDrawSurface* (back) |
| 0x005AEEAC | g_pDDPalette | IDirectDrawPalette* |
| 0x00973640 | g_palette_entries | PALETTEENTRY[256] |
| 0x005AF228 | g_hMainWnd | App main window |
| 0x00884C67 | g_screen_width | Screen width (WORD) |
| 0x00884C69 | g_screen_height | Screen height (WORD) |
| 0x0096CEF4 | g_render_surface_info | Surface info pointer |
| 0x0096CEF8 | g_render_buffer_ptr | Locked surface pixel ptr |
| 0x0096CEFC | g_render_buffer_pitch | Surface pitch |
| 0x0096CF10 | g_software_framebuffer | Software render target |
| 0x0096CF04 | g_color_lookup_table | 64KB sprite blend table |
| 0x006868A8 | g_camera_data_ptr | Camera struct pointer |
| 0x006868AC | g_camera_matrix | 3x3 rotation matrix (9 dwords) |
| 0x007B8FB4 | g_camera_perspective_correction | Barrel distortion factor |
| 0x007B8FBC | g_camera_near_plane | Near plane distance |
| 0x007B8FC0 | g_camera_projection_shift | Projection bit shift |
| 0x007B8FFC | g_screen_center_x | Screen center X |
| 0x007B8FFE | g_screen_center_y | Screen center Y |
| 0x007B8FE8 | g_viewport_right | Right clip boundary |
| 0x007B8FEA | g_viewport_bottom | Bottom clip boundary |
| 0x00699A64 | g_depth_buckets | 3585-slot depth bucket array |
| 0x00699A60 | g_depth_bucket_alloc_ptr | Bucket entry allocator |
| 0x00699A5C | g_depth_bucket_alloc_limit | Allocator limit |
| 0x007B9110 | g_depth_bucket_min_used | Minimum used bucket index |
| 0x007B911C | g_depth_bucket_max_used | Maximum used bucket index |
| 0x0068A050 | g_transformed_vertex_buffer | 3D model vertex buffer |
| 0x00878928 | g_object_pool | Object pointer array |
| 0x0088897C | g_terrain_cell_data | Terrain cell records (16B/cell) |
| 0x007B9170 | g_terrain_cell_array | Terrain cell array |
| 0x007B9160 | g_terrain_cell_index_map | (x,z) → cell index |
| 0x00699A50 | g_terrain_vertex_buffer_A | Terrain vertex buffer (ping) |
| 0x00699A54 | g_terrain_vertex_buffer_B | Terrain vertex buffer (pong) |
| 0x0069D5E0 | g_scanline_span_ptr | Scanline span pointer |
| 0x007B8FAC | g_terrain_columns_remaining | Columns to process |
| 0x007B8F78 | g_terrain_triangle_count | Triangle counter |
| 0x0069D5E4 | g_terrain_triangle_array | Triangle descriptors (8B each) |
| 0x00599AC0 | g_terrain_heightmap_ptr | Heightmap data pointer |
| 0x00599ADC | g_terrain_texturemap_ptr | Texture map pointer |
| 0x00599AD4 | g_terrain_shade_table | Shade lookup table |
| 0x00599AD8 | g_terrain_fog_table | Fog lookup table |
| 0x00599AC4 | g_terrain_sea_level_table | Sea level lookup |
| 0x00599AC8 | g_terrain_water_heightmap | Water heightmap data |
| 0x009735D0 | g_render_target_desc | Render target descriptor |
| 0x009735EC | g_render_target_pixels | Render target pixel ptr |
| 0x009735C0 | g_clip_rect | Current clip rectangle |
| 0x009735B8 | g_pixel_write_func | Per-bpp pixel writer |
| 0x005AC6A0 | g_sin_table_2048 | Sin lookup (2048 entries) |
| 0x005ACEA0 | g_cos_table_2048 | Cos lookup (2048 entries) |
| 0x0098A004 | g_current_texture_ptr | Active texture pointer |
| 0x0098A008 | g_texture_flags | Texture/render mode flags |
| 0x005A7D44 | g_texture_base_ptr | Texture memory base |
| 0x0097C000 | g_rasterize_triangle_func | Triangle rasterizer function ptr |
| 0x00884226 | g_face_color_index_table | Face color index LUT |
| 0x005A2F28 | g_tribal_face_flags | Per-face tribal color flags |
| 0x007B9026 | g_selected_unit_id | Selected unit ID |
| 0x0057C17C | g_anim_blend_factor | Animation blend factor |
| 0x0057C180 | g_anim_blend_duration | Animation blend duration |
| 0x0087FF19 | g_current_game_turn | Game turn/tick counter |
| 0x00686858 | g_selection_rect_x | Selection rect origin X |
| 0x0068685C | g_selection_rect_y | Selection rect origin Y |
| 0x00686860 | g_selection_rect_width | Selection rect width |
| 0x00686864 | g_selection_rect_height | Selection rect height |
| 0x008853D9 | g_fog_density | Fog density (byte) |
| 0x008853DA | g_fog_density_alt | Fog density alternate (byte) |
| 0x008853DD | g_fog_start_distance | Fog start distance |
| 0x008853E1 | g_fog_start_distance_far | Fog start distance (far) |
| 0x00686850 | g_frame_start_time | Frame start timestamp |
| 0x0096CEC8 | g_current_game_turn_render | Game tick (render copy) |
| 0x0096CB18 | g_camera_world_x | Camera X position |
| 0x0096CB1C | g_camera_world_z | Camera Z position |
| 0x005A7D48 | g_text_cmd_buffer_ptr | Text rendering command buffer |
| 0x005C4D28 | g_text_cmd_buffer_saved | Saved text cmd buffer position |
| 0x0068689C | g_render_terrain_func | Terrain render function pointer |
| 0x006868A0 | g_render_model_func | 3D model render function pointer |
| 0x006868A4 | g_render_model_alt_func | Alt model render function pointer |
| 0x00885700-05 | g_status_bar_timers[6] | Status effect display timers |
| 0x00884202 | g_display_bit_depth | Display mode (9-11 = 16bpp) |
| 0x0087E33C | g_render_mode_flags | Render mode flags (bit 0x40, 0x80) |

---

### R.29 render_frame Pipeline @ 0x004DF3C0

The top-level per-frame rendering function. Called once per game tick to composite the full scene.

**Pipeline order:**

1. **Framebuffer copy**: Copies software framebuffer (`g_software_framebuffer`) to the locked DDraw surface, line by line (0xA0 dwords = 640 bytes per line)

2. **Save text buffer position**: `Render_SaveTextCmdBufferPos` (0x0040B060) — saves current text command buffer pointer for later text rendering

3. **Scene composition**: `FUN_004E1980` (0x004E1980) — 8KB function, the main scene compositor that draws the 3D world to the framebuffer. Called with the current game tick as parameter.

4. **16bpp mode check**: `Render_Is16bppMode` (0x004533E0) — returns 1 if display mode is 9, 10, or 11 (16bpp modes). If true, sets up 16bpp palette pointer.

5. **Previous-frame text overlays**: `Render_DrawTextOverlays_Prev` (0x0040AEA0) — renders text commands that were buffered in the previous frame. Processes a command stream with per-character rendering, handling 9+ font types.

6. **RLE sprite overlay**: `draw_sprite_rle` (0x004E0000) — draws the cursor/UI sprite overlay on top of the scene

7. **Animated overlay** (conditional): If flag 0x20 is set at 0x0096CFE4, renders an animated sprite overlay (e.g., loading screen) with frame counter cycling

8. **Resolution change handling**: Detects display width changes and calls either `RenderCmd_SubmitComplex` or `Render_InitScreenBuffer` to reinitialize buffers

9. **Current-frame text overlays**: `Render_DrawTextOverlays` (0x0040AB40) — renders text commands for the current frame. Similar to the previous-frame pass but handles more font modes and render target switching.

10. **Blit to screen**: `Sprite_BlitStandard` (0x0050EDD0) — final blit of the composed scene to the display surface

---

### R.30 Camera_SetupProjection @ 0x0046EDB0

The per-frame camera initialization that sets up the rendering pipeline state.

**Rasterizer selection** (bit 0x40 of `g_render_mode_flags` at 0x0087E33C):
- Clear: `g_triangle_rasterize_func` = 0x0097C000 (`Rasterizer_Main` — the full 53KB software rasterizer)
- Set: `g_triangle_rasterize_func` = 0x00473BB0 (alternate/stub rasterizer)

**Model pipeline function pointers** (bit 0x80 of `g_render_mode_flags`):
- Clear (normal):
  - `g_render_terrain_func` (0x0068689C) = 0x00473BC0 (terrain vertex processor)
  - `g_render_model_func` (0x006868A0) = 0x00471730 (`Model3D_RenderObject`)
  - `g_render_model_alt_func` (0x006868A4) = 0x00472A80 (alternate model renderer)
- Set (disabled): All three set to 0x00473BC0 (stub — disables 3D rendering)

**Camera setup:**
- Copies camera struct (0x24 bytes) from `g_camera_data_ptr` to `g_camera_matrix`
- Computes camera shift values from camera height/angle for fog calculation
- Sets viewport bounds: left=0, top=0, right=screen_width, bottom=screen_height
- Computes screen center: `g_screen_center_x = screen_width / 2`, same for Y
- Calls `Render_SetupClipRect` with display surface parameters

**Depth bucket initialization:**
- Clears 0xE01 (3585) depth bucket slots to zero
- Resets allocator pointer from 0x00699A58
- Sets scanline span pointer to 0x0069D268
- Sets initial span size to 0xDC (220 bytes)

**Game mode flags:**
- Checks game mode at 0x00884C7F: values 0x0C and 0x0D enable terrain fog flag at 0x00686894
- Checks various game state flags to determine if shadow rendering is enabled

---

### R.31 Rasterizer_Main @ 0x0097C000

The massive software triangle rasterizer — **53KB of x86 code** (0x0097C000 to 0x00988FE7).

**Entry point:**
```
PUSHAD                          ; save all registers
params: [ESP+0x24] = vertex_a_ptr
        [ESP+0x28] = vertex_b_ptr
        [ESP+0x2C] = vertex_c_ptr
        [ESP+0x30] = render_mode
SUB ESP, 0x190                  ; 400 bytes local space
```

**Initial clipping:**
- Checks if any vertex has negative coordinate (sign bit) → skip
- Checks all vertices against max coordinate at 0x0098A00D
- Early-out if fully clipped

**Internal structure:**
Contains **16 jump tables** at addresses 0x986460–0x986B50, suggesting 16 different scanline filling modes:
- Flat-shaded triangles
- Gouraud-shaded triangles
- Textured triangles (affine)
- Textured + gouraud triangles
- Various combinations with fog, transparency, etc.

The rasterizer operates by:
1. Sorting vertices by Y coordinate (top to bottom)
2. Computing edge slopes (dx/dy) with fixed-point precision
3. Walking scanlines from top to bottom
4. For each scanline, interpolating X coordinates and filling pixels
5. For textured modes, interpolating UV coordinates per-pixel
6. For gouraud modes, interpolating shade values per-pixel

---

### R.32 Render_ProcessDepthBuckets_Main @ 0x0046AF00

The central depth bucket processor — **10.5KB** of code handling all render command types via a painter's algorithm (back-to-front).

**Structure:**
```
for each depth_bucket (0xE01 slots, back to front):
    for each command in bucket's linked list:
        type = command[0x00]      // byte
        next = command[0x02]      // dword → next command
        dispatch via jump table at 0x46D8B8[type]
```

**Complete command type dispatch** (jump table at 0x46D8B8, types 0x00–0x1F):

| Type | Handler | Description |
|------|---------|-------------|
| 0x00 | 0x0046AF71 | **Terrain triangle** — texture lookup, UV from tile tables, calls rasterizer |
| 0x01 | 0x0046B7CA | **Sprite/object** — distance scaling, sprite blit with shadow |
| 0x02 | 0x0046B6AC | **Model triangle (textured)** — terrain UV, face color, rasterizer dispatch |
| 0x06 | 0x0046C767 | **Model triangle (material)** — 10-type material jump table (0-9) at 0x46D958 |
| 0x0D | 0x0046B7CA | **Special sprite** — shares handler with type 0x01 but different scaling |
| 0x0E | 0x0046C698 | **Selection highlight** — unit selection circle overlay |
| 0x14 | 0x0046D1FC | **Ground circle** — calls `Render_DrawGroundCircle` + `Render_DrawGroundCircleAnimated` |
| 0x15 | 0x0046D344 | **Shadow blob** — calls `Render_DrawShadowBlob` (0x00476890) |
| 0x16 | 0x0046D359 | **Shadow projection** — calls `Render_DrawShadowProjection` (0x00476C40) |
| 0x17 | 0x0046D4F5 | **Effect quad** — spell/lightning bolt with directional ±8px offset, texture flag 0x17/0x1F |
| 0x18 | 0x0046D248 | **Triangle accumulator** — collects 3 triangle vertices then renders |
| 0x1A | 0x0046B7CA | **LOD sprite** — distance-based scale with LOD transition bands (0x540–0x690 depth range) |
| 0x1E | 0x0046D784 | **UI sprite (alt)** — sprite index + 0x626, with dedicated palette |
| 0x1F | 0x0046D784 | **UI sprite** — sprite index + 0x219, standard palette |
| 0x06 sub | 0x0046C6FA | **Sprite overlay** — sprite centered at screen pos |
| 0x06 sub | 0x0046C726 | **Sprite overlay (animated)** — with frame-based animation |

**Terrain texture lookup** (type 0x00 handler):
Two paths based on flag 0x80 at 0x0087E33F:
- **Indexed tiles** (flag set): tile_index from UV table at 0x59C010, texture address = `(tile >> 3) << 10 + (tile & 7) << 5 + terrain_texture_base`
- **Raw heightmap tiles** (flag clear): tile from UV table at 0x59C0D0, texture address = `(tile & 0xF) << 4 + (tile >> 4) << 8 + terrain_texture_base`

Then reads UV coordinates from the tile's 24-byte UV record and applies **water animation** using sin/cos wave displacement on UV coordinates.

**Model material types** (type 0x06, sub-dispatch at 0x46D958):
10 material types (0-9) controlling shading mode:
- Types using texture palette lookup from `g_model_texture_palette` (0x0095C6F4) with tribal-aware selection
- Types using flat color from `g_flat_color_palette` (0x957077) indexed by shade value
- Each path sets `g_texture_flags` and `g_current_texture_ptr` then calls the rasterizer

**Status bar rendering** (within sprite handler):
After rendering a sprite, checks 6 status effect flags (0x00885700-0x00885705) against the object's effect bitmask (byte `+0x7A`, bits 0x02-0x40). For each active effect, draws colored lines using `Draw_Line` (0x00402800) and `Draw_FilledRect` (0x0050EF90).

---

### R.33 Post-Processing and Camera Effects @ 0x00467890

`Render_PostProcessEffects` handles camera movement, visual effects, and per-frame state updates.

**Camera modes** (byte at 0x00686728):
- **0**: Static/no camera — skip camera processing
- **1**: Camera pan with lerping — computes dx/dz from previous frame, applies damped movement with intensity = `camera_height * 8 + 0x78` (clamped 0-255). Calls horizontal scroll (0x004ABB50) and vertical scroll (0x004ABAB0).
- **2**: Direct camera set — resets camera to new position, calls combined scroll function (0x004ABBE0)

**Game mode dispatch** (byte at 0x00884C7F, types 4-16):
| Mode | Effect |
|------|--------|
| 4 | Auto-scroll / cinematic camera with timed keyframes |
| 5 | Earthquake effect — terrain rendering with shake offset |
| 6 | Player death / timeout state |
| 9 | Fog-of-war overlay with tribal-specific rendering |
| 10 | Discovery mode — spell discovery animation |
| 11-13 | Various spell effect overlays |
| 14 | Patrolling/guard circle rendering |
| 16 | Building placement preview with cursor tracking |

**Status bar timer management:**
Six slots (0x00885700–0x00885705) are set to 6 when their corresponding effect is active on the selected unit, then count down per frame. Used by the depth bucket processor to render colored status lines on unit sprites.

---

### R.34 Render_SetupRasterizerCallback @ 0x00429A50

Initializes the per-frame rendering state for the **depth bucket / UI renderer** path (separate from Camera_SetupProjection which sets up the 3D scene path).

**Rasterizer selection** (same flag as Camera_SetupProjection):
- Bit 0x40 of 0x0087E33C: selects between full rasterizer (0x0097C000) and stub (0x00473BB0)

**Initialization:**
1. Calls 0x00492E10 (pre-render setup)
2. Clears first 10 depth bucket slots (for UI overlay layer)
3. Copies camera world position to viewport state
4. Resets viewport to full screen: left=0, top=0, right=screen_width, bottom=screen_height
5. Computes screen center
6. Calls `Render_SetupClipRect` with display parameters
7. Resets selected unit ID and hover unit to 0

---

### R.35 Water Rendering System

**Water_UpdateWavePhase** @ 0x0048E990:
Updates water level for each water body in a linked list (root at 0x007F917C).

Water body struct fields:
| Offset | Field |
|--------|-------|
| +0x04 | current_level (height) |
| +0x08 | base_level |
| +0x0C | velocity (rise/fall rate) |
| +0x20 | water_type (byte) |
| +0x21 | flags (dword) |
| +0x25 | upstream_link (ptr to connected water) |
| +0x29 | downstream_link (ptr to connected water) |

Physics:
- Velocity += 0x555 per tick (gravity constant)
- Rising water: when level reaches (upstream.level - base_level), transfers momentum to upstream body using type-dependent damping from table at 0x599BB2
- Falling water: when level reaches (downstream.level + downstream.base), transfers momentum downstream
- Flag 0x80000: water at equilibrium level, flag 0x40000: velocity zeroed out

**Water_RenderObjects** @ 0x004A75F0:
Records per-water-body rendering state. For each water body (count at 0x00957058):
- Checks if water body is enabled (flag at 0x88637F per body, stride 0xC65)
- Checks render frequency from 0x0087AE95 (skips frames if nonzero)
- Records 15-byte snapshot (position, height, wave state) into circular buffer of 100 entries

---

### R.36 Sprite Rendering Pipeline

**Sprite_RenderWithShadow** @ 0x00411B70:
Entry point for sprite commands from the depth bucket processor.

Dispatch by sub-type byte (`+0x01`):
- **Sub-type 0**: Direct render — calls `Sprite_RenderObject` (0x00411C90)
- **Sub-type 1**: Shadow render — locks DDraw surface (0x00512310), renders sprite, composites shadow, restores render target
- **Sub-type ≥2**: Skip (early return)

Object pointer lookup: `g_object_pool[command[+0x0C]]` at 0x878928

**Sprite_RenderObject** @ 0x00411C90:
The main sprite renderer — **11.5KB** (0x00411C90 to 0x00414938). Handles:
- Frame selection from animation state
- Horizontal flip based on facing direction
- Distance-based scaling via `Render_CalculateDistanceScale`
- Palette selection based on tribal affiliation
- RLE decompression and pixel blitting
- Transparency and color blending via `g_color_lookup_table`

**Render_CalculateDistanceScale** @ 0x00477420:
Computes sprite display size based on distance from camera.
Size = base_size * (near_scale - (depth - min_depth) * scale_factor)
Clamps to minimum 1 pixel.

---

### R.37 Drawing Primitives

| Address | Name | Description |
|---------|------|-------------|
| 0x00402800 | Draw_Line | Bresenham line rasterizer |
| 0x0050F050 | Draw_Line (variant) | Line drawing with color parameter |
| 0x0050EF70 | Draw_Pixel | Single pixel write |
| 0x0050EF90 | Draw_FilledRect | Filled rectangle |
| 0x0050EDD0 | Sprite_BlitStandard | Standard sprite blit (non-scaled) |
| 0x0050F6E0 | Sprite_BlitScaled | Scaled sprite blit (with clipping) |
| 0x0050FC20 | (palette loader) | Load palette for sprite rendering |
| 0x0050FCC0 | (palette loader 16bpp) | 16bpp palette variant |
| 0x004A0310 | (text renderer 16bpp) | 16bpp text character renderer |
| 0x0050FAE0 | (text renderer 8bpp) | 8bpp text outline renderer |

---

### R.38 Complete Function Reference (Rendering) — Updated

Additional functions discovered in iteration 4:

| Address | Name | Description |
|---------|------|-------------|
| 0x0046EDB0 | Camera_SetupProjection | Per-frame camera + pipeline init |
| 0x00429A50 | Render_SetupRasterizerCallback | UI/overlay rasterizer init |
| 0x004502A0 | Render_InitGlobals | Zero-init all render globals |
| 0x0097C000 | Rasterizer_Main | 53KB software triangle rasterizer |
| 0x00467890 | Render_PostProcessEffects | Camera movement + visual effects |
| 0x00427C60 | Render_FinalDisplay | DDraw page flip / blit to screen |
| 0x0040AB40 | Render_DrawTextOverlays | Current-frame text rendering |
| 0x0040AEA0 | Render_DrawTextOverlays_Prev | Previous-frame text rendering |
| 0x004533E0 | Render_Is16bppMode | Check if display is 16bpp (modes 9-11) |
| 0x0040B060 | Render_SaveTextCmdBufferPos | Save text buffer position |
| 0x00411B70 | Sprite_RenderWithShadow | Sprite dispatch (direct/shadow) |
| 0x00411C90 | Sprite_RenderObject | Main sprite renderer (11.5KB) |
| 0x00477420 | Render_CalculateDistanceScale | Distance-based sprite scaling |
| 0x0050F6E0 | Sprite_BlitScaled | Scaled sprite blitter |
| 0x0050EDD0 | Sprite_BlitStandard | Standard sprite blitter |
| 0x0050F050 | Draw_Line | Line rasterizer |
| 0x0050EF70 | Draw_Pixel | Single pixel writer |
| 0x0050EF90 | Draw_FilledRect | Filled rectangle |
| 0x00475F50 | Render_DrawGroundCircle | Ground circle effect |
| 0x004760D0 | Render_DrawGroundCircleAnimated | Animated ground circle |
| 0x00476890 | Render_DrawShadowBlob | Shadow blob renderer |
| 0x00476C40 | Render_DrawShadowProjection | Shadow projection renderer |
| 0x005094B0 | Sprite_SetRenderTarget | Set sprite render target |
| 0x005129E0 | RenderCmd_SubmitComplex | Submit complex render command |
| 0x00512B40 | Render_InitScreenBuffer | Initialize screen buffer |
| 0x0048E990 | Water_UpdateWavePhase | Water level physics |
| 0x004A75F0 | Water_RenderObjects | Water render state recording |
| 0x0048E210 | Water_AnimateMesh | Water mesh animation |
| 0x0048E730 | Water_SetupMesh | Water mesh initialization |
| 0x0048EBB0 | Terrain_SetupWaterWaveParams | Water wave parameters |
| 0x0048BF10 | Tick_UpdateWater | Water tick update |
| 0x0042B950 | Minimap_Update | Minimap update |
| 0x0042BA10 | Minimap_RenderTerrain | Minimap terrain render |
| 0x0042BBE0 | Minimap_RenderObjects | Minimap object render |
| 0x00494CF0 | Minimap_DrawSprite | Minimap sprite draw |

---

### R.39 GUI/Scene Compositor — GUI_RenderSceneElement @ 0x004E1980

**Function size**: ~8KB (0x004E1980–0x004E3996), 0x1ACC bytes stack frame

**Called from**:
- `render_frame` @ 0x004DF419 — main per-frame render
- `FUN_004DE480` @ 0x004DE4AD — alternate render path
- `FUN_004E1960` @ 0x004E1970 — wrapper function

**Overview**: This is the GUI/HUD scene element compositor. It takes a scene element
descriptor index as parameter, looks up an element definition from a table at
`0x0057B2D0` (stride 0x20, indexed by `param * 0x20`), and iterates through a
linked list of child elements. Each child element has a type field at offset
`[child+0] - 2` (range 0x00–0x0B) dispatched via jump table at `0x004E3998`.

**Element descriptor structure** (at 0x0057B2D0 + index*0x20):
- +0x00: (unknown)
- +0x04: child count
- +0x08: pointer to first child element list

**Child element structure** (8 bytes per element at +0x08):
- +0x00: type field (word) — subtract 2 for jump table index (0x00–0x0B)
- +0x04: pointer to element data

**Element data common fields**:
- +0x00: flags dword (bit 0x02 = skip, bit 0x10 = interactive, bit 0x40 = multi-line text)
- +0x04: x position (16.16 FP, scaled by screen width 0x884C67)
- +0x08: y position (16.16 FP, scaled by screen height 0x884C69)
- +0x0C: x alignment (0=left, 1=center, 2=right)
- +0x0E: y alignment (0=top, 1=center, 2=bottom)
- +0x10: h anchor alignment
- +0x12: v anchor alignment
- +0x14: font/layer ID
- +0x18: sub-type selector
- +0x1C: data pointer 1
- +0x20: data pointer 2
- +0x24: data value
- +0x28: callback function pointer (optional, called after rendering)
- +0x30: grid columns
- +0x34: grid rows
- +0x38: grid item count
- +0x3C: grid data start index

**Element types** (jump table at 0x4E3998, index = [child+0] - 2):

| Index | Type | Description |
|-------|------|-------------|
| 0x00 | Text label | Simple text string with font selection, draws via GUI_DrawString |
| 0x01 | Rich text | Text with text command buffer (GUI_DrawString + callback) |
| 0x02 | Blended text | Text with truncation from start (GUI_TruncateStringFromStart) then rich draw |
| 0x03 | Multi-line text | Multi-line text block with wrapping — two sub-paths: bit 0x40 = scrolling text area with memory alloc (0x511310), !0x40 = fixed grid text layout |
| 0x04 | Tiled panel | Grid-based tiled panel (GUI_LayoutGrid + GUI_RenderTiledElement) with alignment and border rendering |
| 0x05 | (appears similar to type 4 with different grid traversal) |
| 0x06-0x09 | (additional panel/button types with varying alignment and rendering modes) |
| 0x0A | Scaled image | Single scaled image element (GUI_RenderScaledElement) — used for backgrounds |
| 0x0B | Interactive element | Button/clickable with hover detection (checks 0x0096CECC mouse-over flag, writes center coords to 0x5C4D40/0x5C4D44) |

**Alignment system**: All elements support 3 alignment modes per axis:
- 0 = left/top aligned (default position)
- 1 = center aligned (position + half_screen/2)
- 2 = right/bottom aligned (position = screen_dimension - 1 + offset)

**Screen scaling**: Positions are 16.16 fixed-point values multiplied by the
current screen dimensions (width at 0x00884C67, height at 0x00884C69).

**Mouse hover detection**: When flag 0x20 is set in render mode (0x0096CFE4),
elements with bit 0x10 or 0x20 check if the mouse cursor is within their
bounding rect. If hit, sets 0x0096CECC = 1 and stores center X/Y at
0x005C4D40 / 0x005C4D44.

---

### R.40 GUI Helper Functions

**GUI_GetBufferForLayer** @ 0x0040AAA0:
- Jump table dispatch (0–4) selecting from 5 layer buffers
- Reads from array at 0x578108 (stride 0x18, fields at +0x00/+0x04/+0x08/+0x0C/+0x10)
- Returns a buffer pointer for the selected layer

**GUI_MeasureString** @ 0x0040B930:
- Measures pixel width and height of a null-terminated wide string
- Calls Render_Is16bppMode to select between 16bpp (0x4A02B0/0x4A0D40/0x4A0D60) and 8bpp (0x50FC20/0x50FC90/0x50FCC0) font metrics
- Returns a rect struct: {x, right_extent, y, bottom_extent, flags}

**GUI_DrawString** @ 0x0040B2B0:
- Draws a wide string to the text command buffer
- Calls GUI_MeasureString first, then checks mouse hover if flag 0x20 set
- Tests bit flags on the element flags byte for rendering mode selection
- Writes to the text command buffer at 0x005C4D28: position (2 shorts), layer byte, alpha byte, flags byte, then copies the wide string, null-terminated
- Calls Render_Is16bppMode for potential alpha override from 0x00884C8C

**GUI_DrawStringAligned** @ 0x0040B5F0:
- Extended string draw with explicit x/y position, alignment, and UV parameters
- Applies alignment offsets before calling GUI_MeasureString + GUI_DrawString

**GUI_DrawStringWithShadow** @ 0x0040B150:
- Draws string with shadow/outline — calls GUI_MeasureString, checks mouse hover,
  writes to text command buffer (same format as GUI_DrawString but with shadow byte)

**GUI_ClipStringToWidth** @ 0x0040BA80:
- Clips a wide string to fit within a given pixel width
- Iterates characters, accumulating width from font metrics
- Also checks for special characters via 0x453400/0x4534B0
- Returns pointer to the truncation point (or NULL if fits)

**GUI_TruncateStringFromEnd** @ 0x0040BB90:
- Truncates a string from the end, keeping the last N characters that fit
- Walks backwards through the string accumulating widths
- Copies the visible portion to the output buffer

**GUI_TruncateStringFromStart** @ 0x0040BC60:
- Truncates from the start, keeping the first N characters that fit
- Walks forward through the string, copies visible portion to output

**Font_ConvertEncoding** @ 0x004A1D00:
- Character encoding conversion based on display mode:
  - Mode 9: GB2312 encoding (call 0x4A0D70)
  - Mode 10: Shift-JIS → fullwidth conversion (ASCII 0x21-0x7E → 0x5C80 offset, comma→0xA1A2, period→0xA1A3, others→0xA1A1)
  - Mode 11: Other encoding (call 0x4A15B0)

**Sprite_Blit** @ 0x0050EDD0:
- vtable dispatch through 0x009735B8 → [vtable + 0x38]
- Parameters: (x_pos, y_pos, sprite_data_ptr)
- This is the low-level sprite blitter used by GUI elements

---

### R.41 GUI Layout/Rendering Sub-Functions

**GUI_LayoutGrid** @ 0x004DF6C0:
- Lays out a grid of tiles for tiled panel elements
- Parameters: (result_rect_ptr, x_start, y_start, columns, rows)
- Uses tile data from 0x0096CF94 (tile definition array, 8 bytes per tile):
  - +0x04: tile width (word)
  - +0x4C: border right offset (word)
  - +0x4E: border bottom offset (word)
- Edge tile selection logic (10 tile indices: 0=interior, 1=top-left, 2=top-edge-even, 3=top-left-corner, 4=right-edge, 5=right-interior, 6=bottom-left, 7=bottom-right, 8=bottom-edge-even, 9=bottom-right-corner, 0xA=top-right)

**GUI_LayoutGridWithRender** @ 0x004DF820:
- Same layout algorithm as GUI_LayoutGrid but calls render functions inline
- Dispatches through either Sprite_Blit (0x0050EDD0) or GUI_RenderRLESprite8bpp (0x4E0110) based on flag at 0x006701E5 bit 0x01
- Also checks flag 0x20 at 0x0096CFE4 for bounding rect tracking (via 0x40BA20)

**GUI_RenderTiledElement** @ 0x004DF9C0:
- Renders a positioned tiled element with column/row tiling
- Checks flag at 0x00884C01 bit 0x80000000 and 0x006701E5 bit 0x01 for rendering path selection
- Path A (flag set): calls 0x004936B0 or 0x0041A370 for rendering
- Path B (default): calls GUI_LayoutGridWithRender (0x4DF820)
- Both paths produce a bounding rect

**GUI_RenderScaledElement** @ 0x004DFAB0:
- Renders an element with scaling interpolation
- Checks flag 0x80 at 0x00884C04 for render path selection
- Applies scale factor (param at +0x28, clamped to 0–0x10000)
- Calculates scaled dimensions from sprite data at 0x0096CFB8
- Flag 0x20 check for bounding rect tracking
- Selects tile index (1 or 2) based on scale factor and hover state

**GUI_RenderRLESprite8bpp** @ 0x004E0110:
- 8bpp RLE sprite renderer for GUI elements
- Decodes RLE stream: negative byte = skip pixels, positive byte = N literal pixels, zero byte = newline
- Uses alpha blending lookup table at 0x0096CF08 (256×256 table: src_alpha*256 + dst_pixel → blended)
- Clips to screen bounds using 0x0096CEFC (stride) and 0x0096CEF8 (base pointer)

---

### R.42 Terrain Pipeline — Vertex Generation and Triangle Emission

**Terrain_GenerateVertices** @ 0x0046DC10:
- Generates terrain vertex data for a strip of terrain tiles
- Reads tile range from 0x0069D5E0 (terrain viewport descriptor)
- Iterates through tiles in the visible range
- For each tile:
  1. Computes tile address in the heightmap array at 0x88897C (16 bytes per tile, indexed by row*256+col style addressing)
  2. Checks tile visibility via Terrain_CheckTileVisibility (0x46E040)
  3. If visible: reads height from tile+0x04 (signed word), computes shade from tile+0x08 >> 10 + 0x20 (range 0x20–0x3F)
  4. If not visible: uses sway/wind displacement lookup — accesses 0x599AC8 (terrain sway table), applies 0x885720 (wind direction) with formula: `offset = (-wind*0x109 + tile_index*8 + 0x4C) & 0xFFFF`, height = `table[offset] + table[wind*0x109 + tile_index*8]`, shade = `(sum >> 4) + 0x10` clamped to [1, 0x3F], flags |= 0x80 (sway flag)
  5. Checks tile flags for water bit (AH & 0x02) and exclusion mask (0x100000) → sets vertex flag 0x40
  6. Calls Terrain_TransformVertex (0x46EBD0) for camera transform
- Vertex output structure (0x20 bytes per vertex, stored at 0x00699A54):
  - +0x00: heightmap data pointer
  - +0x04: height value
  - +0x08: heightmap secondary pointer
  - +0x0C: screen X after projection
  - +0x10: screen Y after projection
  - +0x14: shade/LOD value
  - +0x18: flags (0x40=water, 0x80=sway-displaced)

**Terrain_TransformVertex** @ 0x0046EBD0:
- Applies 3×3 camera rotation matrix to vertex position (14-bit FP)
- Matrix stored at 0x006868AC/B4/B8/BC/C0/C4/C8/CC (9 elements)
- After rotation: applies barrel distortion correction using 0x7B8FB4 (distortion factor)
  - `y_corrected = y - (x²*2 + z²*2) * distortion_factor >> 16`
- Perspective projection using 0x006868A8 → [+0x2A] (focal length):
  - If depth > 0: `1/depth` reciprocal, screen_x = focal * rotated_x / depth, screen_y = focal * rotated_y / depth
  - If depth ≤ 0: uses far-plane fallback from 0x7B8FFC/0x7B8FFE
- Adds screen center offset from 0x7B8FFC/0x7B8FFE

**Terrain_GenerateTriangles** @ 0x0046E0F0:
- Generates terrain triangles from the vertex strips produced by Terrain_GenerateVertices
- Two vertex strips: ESI (current row at 0x699A50) and EDI (next row at 0x699A54)
- Each pair of adjacent vertices forms a quad, split into 2 triangles
- Triangle winding depends on bit 0x01 of tile flags:
  - Flag bit clear: split as [V0, V1, V2] and [V0, V2+, V1+] (standard diagonal)
  - Flag bit set: split as [V0, V1, V2+] and [V0+, V2+, V1] (alternate diagonal)
- Visibility culling: uses Cohen-Sutherland style outcode testing
  - Outcodes: 0x02 = left of screen, 0x04 = right of screen, 0x10 = below screen
  - Tests 3 vertices of each triangle pair against viewport bounds (0x7B8FE8/0x7B8FEA)
  - AND of outcodes = all vertices outside same edge → cull triangle
- Visible triangles are emitted via Terrain_EmitTriangle (0x46F6F0) with a winding index (0-3)
- After emitting triangles, checks tile for special overlay effects:
  - Flags 0x880: coastline effects (calls 0x45A340 first, then 0x475830 with flag 0x180)
  - Flag 0x400: tribal territory overlay (checks tribal index via 0x884C88, calls 0x475830 with flag 0x400)
- Stride: 0x20 bytes per vertex, loop decrements 0x7B8FAC until zero

**Terrain_EmitTriangle** @ 0x0046F6F0:
- Emits a single terrain triangle to the depth bucket system
- Parameters: (vertex_ptrs..., winding_index)
- Not fully analyzed (large function)

**Terrain_CheckTriangleVisibility** @ 0x0046E870:
- Checks if a triangle's projection is potentially visible
- Parameters: (v0_ptr, v1_ptr, v2_ptr)
- Returns > 0 if visible (front-facing and within viewport)

**Terrain_ProcessSpecialTile** @ 0x0046FC30:
- Handles special terrain tile effects (water edges, coast rendering)

**Terrain_CheckTileVisibility** @ 0x0046E040:
- Checks if a single tile is within the render distance
- Used for LOD decisions in vertex generation

---

### R.43 Sprite Rendering Object — Sprite_RenderObject @ 0x00411C90

**Function size**: ~220K chars disassembly (11.5KB code), 0x108 bytes stack frame

**Overview**: The main sprite/object renderer that handles all 2D sprite drawing
in the 3D world. Reads object data from 0x005A7D50 (game object pointer) with
various fields:
- +0x144: field used for sprite selection
- +0x25C/+0x25E: bounding dimensions

**Rendering architecture**: Uses two parallel rendering paths (8bpp / 16bpp)
with paired function calls:
- Setup: Sprite_SetupScanline8bpp (0x402800) / Sprite_SetupScanline16bpp (0x50EF90)
- Scanline: Sprite_DrawScanline8bpp (0x402840) / Sprite_DrawScanline16bpp (0x50F050)

The rendering loop alternates between setup (which configures scanline parameters)
and draw (which rasterizes one scanline of the sprite). The 8bpp path is used
when Render_Is16bppMode returns 0, the 16bpp path otherwise.

**Other calls within**:
- 0x0047BF20: sprite frame lookup / animation
- 0x00426220: sprite clipping / bounds check
- 0x0049B790: sprite palette selection
- 0x005123C0: memory allocation for sprite buffer
- Sprite_Blit (0x50EDD0): final blit to screen

---

### R.44 Complete Function Reference (Rendering) — Iteration 6 Update

Additional functions discovered/renamed in iteration 6:

| Address | Name | Description |
|---------|------|-------------|
| 0x004E1980 | GUI_RenderSceneElement | GUI/HUD scene compositor (~8KB) |
| 0x0040AAA0 | GUI_GetBufferForLayer | Get render buffer for GUI layer (5 layers) |
| 0x0040B2B0 | GUI_DrawString | Draw wide string to text command buffer |
| 0x0040B5F0 | GUI_DrawStringAligned | Draw string with alignment + UV params |
| 0x0040B150 | GUI_DrawStringWithShadow | Draw string with shadow/outline |
| 0x0040B930 | GUI_MeasureString | Measure pixel dimensions of wide string |
| 0x0040BA80 | GUI_ClipStringToWidth | Clip string to fit pixel width |
| 0x0040BB90 | GUI_TruncateStringFromEnd | Truncate string keeping end chars |
| 0x0040BC60 | GUI_TruncateStringFromStart | Truncate string keeping start chars |
| 0x004DF6C0 | GUI_LayoutGrid | Grid layout for tiled panels |
| 0x004DF820 | GUI_LayoutGridWithRender | Grid layout + inline rendering |
| 0x004DF9C0 | GUI_RenderTiledElement | Render positioned tiled element |
| 0x004DFAB0 | GUI_RenderScaledElement | Render scaled GUI element |
| 0x004E0110 | GUI_RenderRLESprite8bpp | 8bpp RLE sprite renderer for GUI |
| 0x004A1D00 | Font_ConvertEncoding | Character encoding conversion (GB2312/SJIS) |
| 0x0050EDD0 | Sprite_Blit | vtable sprite blitter dispatch |
| 0x0046DC10 | Terrain_GenerateVertices | Generate vertex strip for terrain row |
| 0x0046EBD0 | Terrain_TransformVertex | Camera rotation + projection per vertex |
| 0x0046E0F0 | Terrain_GenerateTriangles | Generate triangles from vertex strips |
| 0x0046F6F0 | Terrain_EmitTriangle | Emit triangle to depth bucket system |
| 0x0046E870 | Terrain_CheckTriangleVisibility | Front-face + viewport visibility test |
| 0x0046E040 | Terrain_CheckTileVisibility | Single tile render distance check |
| 0x0046FC30 | Terrain_ProcessSpecialTile | Special tile effects (water edges) |
| 0x00475830 | Terrain_EmitOverlayEffects | Coastline + territory overlay emission |
| 0x00411C90 | Sprite_RenderObject | Main sprite object renderer (11.5KB) |
| 0x00402800 | Sprite_SetupScanline8bpp | 8bpp scanline setup |
| 0x0050EF90 | Sprite_SetupScanline16bpp | 16bpp scanline setup |
| 0x00402840 | Sprite_DrawScanline8bpp | 8bpp scanline rasterizer |
| 0x0050F050 | Sprite_DrawScanline16bpp | 16bpp scanline rasterizer |

**Key data structures discovered**:

| Address | Name | Description |
|---------|------|-------------|
| 0x0057B2D0 | g_scene_element_defs | GUI scene element definition array (0x20 stride) |
| 0x00578108 | g_gui_layer_buffers | GUI layer buffer array (5 layers, 0x18 stride) |
| 0x0096CF94 | g_tile_defs | Tile definition array for GUI panels (8 bytes/tile) |
| 0x0096CFB8 | g_sprite_sheet | Current sprite sheet data pointer |
| 0x0096CF08 | g_alpha_blend_table | 256×256 alpha blending lookup table |
| 0x0096CEFC | g_framebuffer_stride | Framebuffer row stride |
| 0x0096CEF8 | g_framebuffer_base | Framebuffer base pointer |
| 0x0096CECC | g_mouse_hover_hit | Mouse hover detection flag |
| 0x005C4D40 | g_hover_center_x | Hover element center X |
| 0x005C4D44 | g_hover_center_y | Hover element center Y |
| 0x0096CFA8 | g_gui_border_width | GUI element border width |
| 0x009735B8 | g_sprite_vtable | Sprite interface vtable pointer |
| 0x0088897C | g_heightmap_tiles | Terrain heightmap tile array (16 bytes/tile) |
| 0x00699A50 | g_terrain_vertex_strip_a | Terrain vertex strip A pointer |
| 0x00699A54 | g_terrain_vertex_strip_b | Terrain vertex strip B pointer |
| 0x0069D5E0 | g_terrain_viewport | Terrain viewport descriptor |
| 0x00599AC8 | g_terrain_sway_table | Terrain sway/wind displacement table |
| 0x00885720 | g_wind_direction | Wind direction byte |
| 0x007B8FAC | g_terrain_tile_count | Remaining terrain tiles to process |
| 0x007B8FD0 | g_terrain_heightmap_ptr | Current heightmap row pointer |
| 0x007B8FE2 | g_terrain_tile_index | Current tile column index |
| 0x006868AC–CC | g_camera_rotation_matrix | 3×3 camera rotation (14-bit FP, 9 dwords) |
| 0x007B8FB4 | g_barrel_distortion | Barrel distortion correction factor |
| 0x007B8FBC | g_camera_depth_offset | Camera depth offset for projection |
| 0x007B8FC0 | g_projection_shift | Projection reciprocal shift amount |
| 0x007B8FFC | g_screen_center_x | Screen center X for projection |
| 0x007B8FFE | g_screen_center_y | Screen center Y for projection |

---

## R.45 — Minimap Rendering System

### Minimap_Update @ 0x0042B950
Entry point for minimap rendering each frame.
- Calls 0x45AA50 (viewport calculation), conditionally calls 0x42BFF0
- Allocates 0x10000 byte buffer at 0x6703B8 if not already present
- Calls Minimap_RenderTerrain, then Minimap_RenderObjects
- Buffer is 256×256 pixels (0x100 stride)

### Minimap_RenderTerrain @ 0x0042BA10
Copies terrain data to the minimap buffer with torus-aware scrolling.
- Reads player tribal index from 0x884C88, looks up tribal color at 0x885784
- Scroll offset calculation:
  - x_offset = (128 - tribal_x_color) * width >> 8
  - y_offset = (128 - tribal_y_color) * height >> 8
- Source buffer at 0x6703B4 (main terrain framebuffer)
- Destination buffer at 0x6703B8 (minimap render target)
- Stride = 0x100 (256 bytes per row)
- 4 copy regions handle the torus wraparound:
  1. Top-left quadrant
  2. Top-right quadrant (wraps X)
  3. Bottom-left quadrant (wraps Y)
  4. Bottom-right quadrant (wraps both X and Y)

### Minimap_RenderObjects @ 0x0042BBE0
Renders game objects as colored dots/sprites on the minimap.
- Sets up 256×256 render target with Render_SetBitDepthVtable (0x50F520)
- Iterates linked list of game objects starting at 0x8788BC
- Object type dispatch via jump table at 0x42BFB8, 10 types via byte table at 0x42BFCC:
  - **Type 1** (units): reads tribal palette from 0x5A17A9, checks visibility via 0x4B3790
  - **Type 2** (buildings with tribe color): palette lookup from 0x5A17AA
  - **Type 3** (shaman): uses shaman color from 0x884C8D
  - **Type 4** (special): checks specific object type 0x2B
  - **Type 5** (iconographic): sprite blit via 0x3B index
- Position calculation: applies tribal scroll offset, scales to minimap coordinates
- Rendering modes:
  - Single pixel via Draw_Pixel @ 0x50EF70
  - 4-pixel cross pattern (center + 3 neighbors) for units with size=1
  - Full sprites via Sprite_Blit @ 0x50EDD0 or Minimap_DrawSprite @ 0x494CF0
- Special 0x5A7D7C flag: renders color mask overlay at 0x973C00

---

## R.46 — Terrain Render Orchestrator

### Terrain_RenderOrchestrator @ 0x0046AC90
The main terrain rendering control flow. Orchestrates the entire terrain pipeline.

**Initialization:**
- Gets/caches timing value at 0x686850 via function pointer at 0x97A710
- Checks render mode bit 0x04 at g_render_mode_flags (0x87E33C); if set, skips entire terrain render

**Terrain Init (calls Terrain_InitRenderState @ 0x46EDB0):**
- Sets render function pointers based on mode flags:
  - g_terrain_rasterizer_fn (0x686898): Rasterizer_Main (0x97C000) for 16bpp, or 0x473BB0 for 8bpp
  - g_model_renderer_fn (0x6868A0): Model3D_RenderObject (0x471730) or null (0x473BC0) when bit 0x80 set
  - g_model_face_submit_fn (0x6868A4): 0x472A80 normally, null when bit 0x80 set
- Copies 36-byte camera data block from [0x6868A8] into g_camera_rotation_matrix (0x6868AC)
- Computes terrain tile start coordinates from camera position
- Sets "god mode" / "fog of war" flags based on game mode at 0x884C7F

**Main render loop:**
- Initializes visible range at 0x7B8FB0 = 0xDC (220 tiles)
- Iterates through tile rows:
  1. Skips empty rows (where min >= max in viewport descriptor at 0x69D268)
  2. Calls Terrain_GenerateVertices (0x46DC10) for vertex strip generation
  3. Swaps vertex strip A/B between 0x699A50/0x699A54 each row
  4. Calls Terrain_GenerateTriangles (0x46E0F0) for triangle emission
  5. Advances row pointer by 0x100, tile index by 2
  6. Decrements remaining tile count at 0x7B8FB0

**Post-terrain passes (in order):**
1. Terrain_ProcessVisibleObjects @ 0x42C170 — submits visible game objects for 3D rendering
2. Terrain_RenderSpecialCells @ 0x475530 — if special tiles flagged (0x7B90B4 nonzero)
3. Object_RenderHighlight @ 0x476770 — renders selected object highlight (conditional on 0x686854 and 0x7B901A)
4. Render_ProcessSelectionHighlights @ 0x476E40 — selection circle rendering
5. Render_ProcessUnitMarkers @ 0x4771A0 — unit status markers/icons
6. Render_Process3DModels @ 0x487E30 — 3D model tile-based rendering (conditional on mode flags)
7. Render_ProcessDepthBuckets_Main @ 0x46AF00 — processes all depth-sorted render commands
8. Stores viewport bounds to 0x87E359–0x87E363 (6 words: min/max for X, Y, Z)
9. Terrain_FinalizeRender @ 0x473A70 — final terrain pass
10. Terrain_PostRenderCleanup @ 0x46EFE0 — resets render state

---

## R.47 — Terrain_InitRenderState @ 0x0046EDB0

Sets up rendering function pointers and camera state before terrain processing.

**Function pointer dispatch table:**
```
g_terrain_rasterizer_fn (0x686898):
  - 16bpp mode (bit 0x40 clear): Rasterizer_Main @ 0x97C000
  - 8bpp mode (bit 0x40 set): lightweight rasterizer @ 0x473BB0

g_model_null_renderer_fn (0x68689C):
  - Always set to 0x473BC0 (null/stub renderer)

g_model_renderer_fn (0x6868A0):
  - Normal mode (bit 0x80 clear): Model3D_RenderObject @ 0x471730
  - Simplified mode (bit 0x80 set): 0x473BC0 (null — skips 3D models)

g_model_face_submit_fn (0x6868A4):
  - Normal mode: 0x472A80 (face submitter)
  - Simplified mode: 0x473BC0 (null)
```

**Camera setup:**
- Copies 36 bytes (0x24) from camera state pointer at [0x6868A8] to g_camera_rotation_matrix at 0x6868AC
  - This includes the 3×3 rotation matrix (9 × 4 = 36 bytes) = 9 dwords of 14-bit FP values

**Tile coordinate init:**
- Camera position X: `(cam[0x24] >> 8) + 0x24` → stored as tile column start at 0x7B8FE0
- Camera position Z: `(cam[0x26] >> 8) + 0x24` → stored as tile row start at 0x7B8FE1
- Both masked with 0xFE to align to even tile boundaries

**Depth bucket clear:**
- Fills 0xE01 depth buckets starting at 0x699A64 with zeros (0xE01 × 4 = 0x3804 bytes)

**Fog of war / god mode:**
- View mode 0x0C or 0x0D → god mode enabled (0x686894 = 1)
- Otherwise checks multiple conditions (game settings, debug flags)

---

## R.48 — Camera_SetViewMode @ 0x0048B860

Handles transitions between different camera/view modes. Large function with 3 jump tables.

**Parameters:** player_struct (ESI), new_mode (BX)

**Jump table 1** (at 0x48BB4C, indexed by mode 0–12):
- Modes are camera perspectives: normal gameplay, follow unit, zoom, overhead, etc.
- Mode 0/1/2: standard game camera setup
- Mode 7: calls 0x41FA30 with camera center coords
- Mode 8: sets 0x87E37A = 5
- Mode 9: sets flag 0x20000 in 0x884BFD, marks dirty (0x87E434 = 1)
- Mode 10: saves camera position at 0x87E40E/0x87E410, centers on world half-extents
- Mode 11: complex setup with initialization calls (0x417300, 0x47DC50, 0x4247F0, 0x47C540)

**Jump table 2** (at 0x48BB7C, view mode 0x0A–0x11):
- Returns cursor type byte based on current view mode
- Mode 0x0A: returns 0x0E (default cursor)
- Mode 0x0B: reads from 0x87E430
- Mode 0x0D: reads from 0x87FF19 (bits 0x3 + 0x1E)
- Mode 0x10: returns 0x0D

**Jump table 3** (at 0x48BB98, exit mode 7–13):
- Mode 7: calls animation functions 0x4909D0, 0x491E50
- Mode 8: clears flags 0x10 and 0x40000 in 0x884BFD
- Mode 9: restores camera position from saved coords

**Sound cursor:**
- Sets cursor sound index at 0x8853CB, loads sound via SHL by 3 + base at 0x5A7D4C
- Zero mode → stops cursor sound (calls 0x512B40)

---

## R.49 — Model3D_RenderObject @ 0x00471730

The complete 3D model rendering pipeline for individual game objects. ~3KB function.

**Parameters:** game_object (EDX at ESP+0x80 adjusted)

**Phase 1: World position & interpolation**
- Reads object position from fields +0x3D/+0x43 (current X), +0x3F/+0x45 (current Y), +0x41/+0x47 (current Z)
- Computes delta: current - base position = displacement
- If flag 0x100 at [obj+0x14] is set AND not in replay mode (0x884BF9 bit 0x2):
  - Applies interpolation: `delta += base * time_delta * interpolation_factor / interpolation_divisor`
  - interpolation_factor at 0x57C17C, divisor at 0x57C180, time_delta from 0x87FF19

**Phase 2: Model bank lookup**
- Object model index from [obj+0x33]: `index * 6 * 3 = index * 18` → offset into model bank array at 0x87E459
- Scale factor from [obj+0x18] (model scale, 8-bit FP)
- Flag 0x200 at [obj+0x35]: use override model pointer from [obj+0x68] instead

**Phase 3: Rotation matrix application**
- If rotation is nonzero (angle at ESP+0x12, or obj+0x6C/0x6E):
  - Calls 0x450320 to build local rotation matrix (3×3 at ESP+0x58)
  - Applies both yaw (0x6C) and pitch (0x6E) rotations via 0x4BC1E0/0x4BC2E0
  - If flag 0x60 at [obj+0x16]: alternate rotation order
  - Applies Y-axis rotation by negated angle via 0x4BC360

**Phase 4: Vertex transformation**
- Reads vertex count from [model_bank+0x4] (short)
- Raw vertex data from [model_bank+0x18] (6 bytes per vertex: 3 × int16)
- Output to global transformed vertex buffer at 0x68A050 (0x20 bytes per vertex)
- Each vertex: scale by model scale >> 8, then optionally apply rotation matrix (14-bit FP, SAR 0xE)
- Adds world position offsets (0x50/0x54) — half-tile camera-relative position
- Height lookup: if heightmap tile at computed position has bit 0x02 set, calls 0x4E8E50 for terrain height
- Otherwise uses precomputed Y from ESP+0x24

**Phase 5: Particle/spawner emission (flags 0x200000/0x400000)**
- 0x200000: Iterates vertices, checks Y > -0xB4, spawns particles (type 0x41) at vertex positions
  - Max 2 particles per object, skips odd frames
  - Calls 0x4FCF_A0 (visibility check), 0x4AFEB0 (particle capacity check), 0x4AFFA0/0x4AFC70 (spawn)
- 0x400000: Similar but spawns type 0x33 particles at water surface level

**Phase 6: Face rendering**
- Reads face count from [model_bank+0x2] (short), face data from [model_bank+0x10] (0x3C bytes per face)
- Each face has: vertex indices at +0x28/+0x2A/+0x2C/+0x2E (4 shorts → quad vertices, transformed vertex buffer index × 0x20 + base 0x68A050)
- Face material index from [face+0x0] → lookup in material color table at 0x884226
- If face has > 3 vertices (byte at face+0x6 > 3): renders as two triangles (quad)
- Visibility check via Terrain_CheckTriangleVisibility @ 0x46E870

**Two render paths based on flag bit 0x01 at model_bank[0]:**
- **Untinted path** (bit clear): calls Model3D_SubmitTriFace @ 0x472720
- **Tinted path** (bit set, tribal coloring):
  - Reads tribe index from [obj+0x2F] (or [obj+0xAA] for building type 2 with flag)
  - Checks tribal color flag table at 0x5A2F28
  - Calls Model3D_SubmitTriFaceTinted @ 0x4728D0 for tribal faces
  - Calls Model3D_SubmitTriFace for non-tribal faces

**Phase 7: Post-render extras**
- If selected (0x686848 set, obj matches 0x7B9026): calls Model3D_SetupSelectionBuffer (0x476430)
  with optional Model3D_DrawSelectionEdge (0x4765A0) per face
- Model3D_SubmitSelectionHighlight @ 0x476690 if object is highlighted
- Model3D_SubmitShadow @ 0x476330 if flag 0x80 at [obj+0x35] set
- Model3D_ApplyVertexWind @ 0x477640 if [obj+0x65] nonzero (vegetation sway)

---

## R.50 — Render_Process3DModels @ 0x00487E30

Processes 3D model tiles for terrain-level rendering. Large function (~2.5KB stack frame, 0x8CC).

**Tile grid traversal:**
- Iterates through a linked list of visible tiles from 0x699A64
- Each tile has game objects at linked list starting from [tile+0x2]
- Spatial hash: tile coord → 128×128 grid at 0x7B9160 (short per cell → index into node list at 0x7B9170)
- Each node is 8 bytes: [x_coord, y_coord, flags, age, prev_link, next_link]

**Node management (doubly-linked list):**
- Active list head at 0x7B9158, free list head at 0x7B9154
- When a tile is encountered: marks its node as active (flag bit 0x01 at +0x2)
- Moves node from free list to active list tail
- Capacity: 0x200 nodes (512)

**Rendering paths:**
1. **Immediate render** (0x599AE4 nonzero): renders all flagged nodes immediately
   - Calls Terrain_RenderTile_Textured (0x489360) or Terrain_RenderTile_Flat (0x489EA0)
   - Selection based on g_render_mode_flags bit 0x80000000

2. **Gradual render** (0x599AE0 nonzero): age-based progressive rendering
   - Increments age counter (byte at +0x3) for active nodes each frame
   - Renders oldest nodes first (highest age), up to 150 (0x96) per frame
   - Prevents rendering spikes by amortizing across frames

3. **Normal render** (both zero): per-tile immediate render
   - Up to 150 tiles per frame
   - Gets terrain quad corner info via Terrain_GetQuadCornerInfo (0x4887A0)
     - Reads 4 corner heights, water flags, light values from heightmap
   - Computes bilinear texture coordinate interpolation (32×32 texels per tile, 11-bit FP)
   - Samples terrain texture via 0x599ADC (texture atlas), 0x599AD4/0x599AD8 (color/shade LUTs)
   - If water tile (0x87E340 bit 0x4): calls Terrain_RenderTile_Water (0x48AA00) instead
   - Post-render: applies object shadows via blend table at 0x959078 (256×256 lookup)

**Data globals:**
- 0x599AC0: terrain texture base pointer
- 0x599AD4: terrain color remap LUT
- 0x599AD8: terrain shade LUT
- 0x599ADC: terrain texture atlas base
- 0x7B9128: object spatial data array
- 0x7B913C: tile render list pointer
- 0x7B916C: terrain object grid (5 dwords per cell)

---

## R.51 — Model3D_SubmitTriFace @ 0x00472720 / Model3D_SubmitTriFaceTinted @ 0x004728D0

These are the face submission functions that emit render commands into the depth-sorted command buffer.

**Depth calculation:**
- Averages the Y (depth) values of all 3 vertices: `(v0.y + v1.y + v2.y + 0x15000)`
- Applies scaling: `depth = (sum * 21) / 256` → then divided by 16 to get bucket index
- Bucket range: 0 to 0xE00 (3584), clamped
- Below-ground adjustment: if v0.y > 0xFFFFF300 (close to camera), reduces render priority

**Command buffer entry (0x46 bytes, command type 0x06):**
- Byte 0: command type = 0x06 (3D model face)
- Byte 1: flags = 0x00
- Bytes 2–5: linked list pointer (next command in bucket)
- Bytes 6–9: vertex 0 screen X/Y
- Bytes 0xA–0xD: vertex 0 depth/outcode
- Bytes 0xE–0x11: vertex 0 U/V
- Bytes 0x12–0x15: vertex 0 something (copied from vertex buffer +0xC)
- Byte 0x16: render priority
- ... repeats for vertex 1 and vertex 2 ...
- Byte 0x42: object identifier word
- Byte 0x44: face material + 1
- Byte 0x45: face flags byte

**Tinted variant (0x4728D0):**
- Identical depth calculation and command layout
- Additional parameter: tribe index → written into face material byte with offset
- Used for tribal-colored faces on buildings/units

**Buffer management:**
- Command buffer at 0x699A64, grows from 0x699A60 (current write pointer)
- Limit at 0x699A5C; if full, command is silently dropped

---

## R.52 — Game_RenderWorld @ 0x0048C070

Very short orchestration function (0x6C bytes). Called once per frame.

- Clears flag at 0x5A7D34
- Gets main render context via 0x4C3CF0 → stores position at 0x96CB18/0x96CB1C
- Calls memory copy routine (0x5125D0) with buffer 0x599B80
- Checks 0x5A7D34: if nonzero, done
- Otherwise decrements counter at 0x5A7D38
- If counter reaches zero and player conditions met (checks indexed into 0x96CB29/0x96CC29):
  - Stores player index at 0x96CE30
  - Resets counter to 1

This is a lightweight entry point — the heavy lifting is in Terrain_RenderOrchestrator and Render_ProcessDepthBuckets_Main.

---

## R.53 — Complete Function Reference (Iteration 8 Update)

### New functions named this iteration:

| Address | Name | Description |
|---------|------|-------------|
| 0x0048C070 | Game_RenderWorld | Top-level world render entry point |
| 0x0048B860 | Camera_SetViewMode | Camera view mode transition handler |
| 0x0046AC90 | Terrain_RenderOrchestrator | Main terrain render control flow |
| 0x00471730 | Model3D_RenderObject | 3D model rendering pipeline |
| 0x00487E30 | Render_Process3DModels | Tile-based 3D model processing |
| 0x00472720 | Model3D_SubmitTriFace | Face submission to depth buckets |
| 0x004728D0 | Model3D_SubmitTriFaceTinted | Tribal-colored face submission |
| 0x0046EDB0 | Terrain_InitRenderState | Render function pointer + camera setup |
| 0x004887A0 | Terrain_GetQuadCornerInfo | Reads 4-corner terrain tile info |
| 0x00489360 | Terrain_RenderTile_Textured | Textured terrain tile renderer |
| 0x00489EA0 | Terrain_RenderTile_Flat | Flat-shaded terrain tile renderer |
| 0x0048AA00 | Terrain_RenderTile_Water | Water terrain tile renderer |
| 0x0049C020 | Model3D_ComputeFaceNormal | Computes face normal from 3 vertices |

### New data globals named this iteration:

| Address | Name | Description |
|---------|------|-------------|
| 0x00686898 | g_terrain_rasterizer_fn | Terrain rasterizer function pointer |
| 0x0068689C | g_model_null_renderer_fn | Null/stub model renderer |
| 0x006868A0 | g_model_renderer_fn | Active model renderer function pointer |
| 0x006868A4 | g_model_face_submit_fn | Active face submission function pointer |
| 0x006868AC | g_camera_rotation_matrix | 3×3 camera rotation (renamed data label) |
| 0x00599AC0 | g_terrain_texture_base | Terrain texture atlas base pointer |
| 0x00599AD4 | g_terrain_color_lut | Terrain color remap LUT |
| 0x00599AD8 | g_terrain_shade_lut | Terrain shade/lighting LUT |
| 0x00599ADC | g_terrain_atlas_base | Terrain tile texture atlas |
| 0x007B9128 | g_object_spatial_data | Object spatial data array |
| 0x007B913C | g_tile_render_list | Tile render list pointer |
| 0x007B9154 | g_tile_node_free_head | Tile node free list head |
| 0x007B9158 | g_tile_node_active_head | Tile node active list head |
| 0x007B9160 | g_tile_spatial_hash | 128×128 tile spatial hash grid |
| 0x007B916C | g_terrain_object_grid | Terrain object grid (5 dwords/cell) |
| 0x007B9170 | g_tile_node_array | Tile node array (8 bytes/node, 512 nodes) |
| 0x00599AE0 | g_tile_gradual_render | Gradual rendering mode flag |
| 0x00599AE4 | g_tile_immediate_render | Immediate rendering mode flag |
| 0x00959078 | g_shadow_blend_table | Shadow blend table (256×256 lookup) |
| 0x006703B4 | g_terrain_framebuffer | Main terrain framebuffer pointer |
| 0x006703B8 | g_minimap_buffer | Minimap 256×256 render buffer |
| 0x00885784 | g_tribal_color_table | Tribal color table (per tribe) |
| 0x008788BC | g_object_linked_list | Game object linked list head |
| 0x005A2F28 | g_tribal_face_flags | Per-material tribal coloring flag table |

### Running totals: ~110+ named functions, ~115+ named data globals

---

## R.54 — Render_BuildLayerOrder @ 0x0047CC80

Determines the render layer priority for the current player's view. ~2.5KB function.

**Purpose:** Builds a sorted array of up to 6 render layer indices at 0x87E438, used to control
draw order of terrain overlays (territory, spells, effects, etc.).

**Player state lookup:**
- Player tribal index from 0x884C88 → complex offset into player struct at 0x885760
  - Formula: `tribal * (1 + 5*2) * (1 + 9*8*4) + 0x885760` = large stride into player data
- Reads player flags at [player + 0xC23] — determines which layers are active

**Layer type IDs and their flags:**
| Layer | Flag bit at +0xC23 | Priority ID |
|-------|-------------------|-------------|
| 7 (territory?) | 0x80 | 0x18 or as determined |
| 2 | 0x04 | various |
| 3 | 0x08 | various |
| 6 | 0x40 | various |
| 4 | 0x10 | various |
| 5 | 0x20 | various |

**Layer conflict resolution:**
- Each candidate layer ID is checked against a permission bitmask table at 0x5A0BA4
  - Each entry is 11 dwords (22 bytes at stride = `id * 11 * 2`)
  - Active layers are tested: `(1 << layer_index) & permission[id]`
  - If any active layer conflicts, the candidate is dropped
- Multiple passes test different flag combinations from the player's 0x87E412 state word
- Specific layer IDs: 0x03 (basic), 0x07-0x0D (terrain effects), 0x10 (fog), 0x13 (standard overlay),
  0x14, 0x16, 0x18, 0x1B, 0x1C, 0x1D, 0x1E, 0x21, 0x22

**Fast path (flag 0x04 at +0xC23 bit 2):**
- When player is in "full control" mode, skips the multi-pass conflict resolution
- Directly assigns layer based on simple flag checks

**Output:**
- Layer count stored at 0x87E437
- Layer IDs stored as bytes at 0x87E438 (up to 6)
- Spacing value: `0x800 / layer_count` → stored at 0x87E42A (11-bit, for even distribution)

---

## R.55 — Animation_RenderFrameSequence @ 0x004E7190

Renders animated sprite sequences (spell effects, explosions, etc.). ~1.2KB function.

**Parameters:** x_offset (short), y_offset (short), animation_index (word), flags (byte)

**Flag bits:**
- Bit 0x01: forward/reverse playback direction
- Bit 0x02: skip initial frame check
- Bit 0x04: sets render flag 0x08 at 0x9735DC

**Animation data lookup:**
- Animation table at 0x5A7D84 (index → offset into frame list)
- Frame list entries at 0x5A7D88 (10 bytes/entry: frame_ptr, x_off, y_off, flags, next_index)
- Each frame has sprite data pointer at [frame_entry + 0x0] → offset into sprite bank at 0x5A7D54

**Render mode dispatch (0x8856FC):**
- **Mode 1** (3D projected): Applies Render_CalculateDistanceScale (0x477420) to both sprite position
  and frame dimensions, then calls Sprite_BlitScaled (0x50F6E0)
- **Mode 2** (resolution-scaled): Scales coordinates by `(screen_width * 256 / 640)` and
  `(screen_height * 256 / 480)` for hi-res support (reference resolution 640×480)
  - Width scale: `(0x884C67 << 8) / 0x280` then `* 30 / 32`
  - Height scale: `(0x884C69 << 8) / 0x1E0` then `* 30 / 32`
  - Applies scale to both position and frame size, then calls Sprite_BlitScaled
- **Mode 0** (direct): Adds offset directly and calls Sprite_Blit (0x50EDD0)

**Frame iteration:**
- Follows linked list via [frame_entry + 0x8] (next frame index)
- Continues until next == base (back to animation table entry)

---

## R.56 — Render_SubmitGrassPatches @ 0x00470210

Submits grass/vegetation decorative patches as render commands into depth buckets. ~1.4KB function.

**Parameters:** model_bank_ptr (at ESP+0x8C), game_object (at ESP+0x94)

**Tile grid setup:**
- Reads object position from [obj+0x3D]/[obj+0x3F] → converts to tile coordinates (>>8, &0xFE)
- Generates 9 neighboring tile positions (3×3 grid around object):
  - Offsets: (-2,+2), (0,+2), (+2,+2), (+2,0), (+2,-2), (0,-2), (-2,-2), (-2,0), center
  - Each stored as 2-byte packed tile coordinate

**Per-tile processing (9 tiles):**
- Looks up heightmap tile at 0x88897C (16 bytes/tile)
- Checks water flag: byte at [tile+0xA] >= 0x80 → skip (no grass on water)
- Checks terrain type: `(tile[0xC] & 0xF) * 7` → bit 0x02 of table at 0x5A3038 → skip if set
- Allocates 0x0C byte command in depth buffer (0x699A60)
- Gets terrain height via Terrain_GetHeightBilinear (0x4E8E50)
- Computes world-space delta from camera, with torus wrapping
- Projects via Camera_ProjectVertexWithClip (0x46EA30)
- Depth bucket: `(projected_y + 0x6ED4) / 16`, clamped to 0–0xE00
- **Command type 0x1E** (grass patch): stores screen X/Y at +0x8/+0xA, animation frame at +0x6

**Grass sway animation (5 additional patches per object):**
- Reads grass type from [obj+0x72] → indexes sway offset table at 0x599A18 (20 bytes/entry, 5 entries)
- Each sway entry adds a displacement to the base tile position
- Produces **command type 0x1F** (animated grass): same layout, animation frame = `frame_counter / 12`

---

## R.57 — Render_SubmitHealthBar @ 0x004707C0

Submits health bar overlay as a render command for game objects. ~0x170 bytes.

**Parameters:** game_object (at ESP+0x4), bar_type (at ESP+0x38)

**Position calculation:**
- Reads object position [obj+0x3D]/[obj+0x43] (current X) and [obj+0x3F]/[obj+0x45] (current Z)
- Applies position interpolation (same formula as Model3D_RenderObject: time_delta * factor / divisor)
- Computes camera-relative position with torus wrapping
- Gets terrain height via Terrain_GetHeightBilinear (0x4E8E50)
- Projects to screen via Camera_ProjectVertexWithClip (0x46EA30)

**Depth bucket submission:**
- Allocates 0x0A byte command in depth buffer
- Depth: `(projected_y + 0x6F40) / 16`, clamped to 0–0xE00
- **Command type:** `bar_type + 0x0F` (variable — health bars are types 0x0F–0x1D range)
- Stores screen X/Y at +0x6/+0x8

---

## R.58 — Game_RenderEffects @ 0x004A6BE0

This is a **stub function** — a single `RET` instruction. The visual effects rendering was either
removed, moved elsewhere, or was never implemented at this address. The actual effects rendering
is handled by Render_PostProcessEffects @ 0x467890 and the various render command types in the
depth bucket system.

---

## R.59 — Complete Rendering Architecture Summary

### Full Frame Rendering Pipeline

```
render_frame @ 0x4DF3C0
├── Framebuffer copy (previous frame → work buffer)
├── Game_RenderWorld @ 0x48C070
│   └── Memory init + player state check
├── Terrain_RenderOrchestrator @ 0x46AC90
│   ├── Terrain_InitRenderState @ 0x46EDB0
│   │   ├── Set function pointers (rasterizer, model renderer)
│   │   ├── Copy camera rotation matrix (36 bytes)
│   │   ├── Compute tile start coordinates
│   │   └── Clear depth buckets (3585 × 4 bytes)
│   ├── [Row Loop] × 220 tile rows:
│   │   ├── Terrain_GenerateVertices @ 0x46DC10
│   │   │   └── Wind/sway displacement from sway table
│   │   ├── Terrain_TransformVertex @ 0x46EBD0 (per vertex)
│   │   │   ├── 3×3 camera rotation (14-bit FP)
│   │   │   ├── Barrel distortion correction
│   │   │   └── Perspective projection
│   │   ├── Terrain_GenerateTriangles @ 0x46E0F0
│   │   │   ├── Quad split (alternate diagonal via flag bit 0x01)
│   │   │   ├── Cohen-Sutherland visibility culling
│   │   │   └── Terrain_EmitTriangle @ 0x46F6F0
│   │   └── Swap vertex strip buffers A↔B
│   ├── Terrain_ProcessVisibleObjects @ 0x42C170
│   ├── Terrain_RenderSpecialCells @ 0x475530
│   ├── Object_RenderHighlight @ 0x476770
│   ├── Render_ProcessSelectionHighlights @ 0x476E40
│   ├── Render_ProcessUnitMarkers @ 0x4771A0
│   ├── Render_Process3DModels @ 0x487E30
│   │   ├── Spatial hash grid (128×128, 512 nodes)
│   │   ├── 3 modes: immediate / gradual / per-tile
│   │   ├── Terrain_RenderTile_Textured @ 0x489360
│   │   ├── Terrain_RenderTile_Flat @ 0x489EA0
│   │   ├── Terrain_RenderTile_Water @ 0x48AA00
│   │   └── Shadow blend via 256×256 LUT @ 0x959078
│   ├── Render_ProcessDepthBuckets_Main @ 0x46AF00
│   │   └── [3585 buckets, back-to-front]
│   │       ├── Type 0x06: Model3D face → Rasterizer_Main @ 0x97C000
│   │       ├── Type 0x1E: Grass patch
│   │       ├── Type 0x1F: Animated grass
│   │       ├── Type 0x0F+: Health bars
│   │       └── 20+ other command types
│   ├── Terrain_FinalizeRender @ 0x473A70
│   └── Terrain_PostRenderCleanup @ 0x46EFE0
├── GUI_RenderSceneElement @ 0x4E1980
│   └── 12 element types via jump table
├── Render_DrawTextOverlays @ 0x40AB40
├── Render_PostProcessEffects @ 0x467890
├── Minimap_Update @ 0x42B950
│   ├── Minimap_RenderTerrain @ 0x42BA10 (torus-aware copy)
│   └── Minimap_RenderObjects @ 0x42BBE0 (10 object types)
└── Render_FinalDisplay @ 0x427C60 (DDraw page flip)
```

### 3D Model Pipeline (per object)

```
Model3D_RenderObject @ 0x471730
├── Phase 1: World position + interpolation
├── Phase 2: Model bank lookup (index × 18 + 0x87E459)
├── Phase 3: Rotation matrix (yaw/pitch via 0x4BC1E0/0x4BC2E0)
├── Phase 4: Vertex transform (scale → rotate → offset → height lookup)
├── Phase 5: Particle emission (fire 0x41, water 0x33)
├── Phase 6: Face rendering
│   ├── Untinted: Model3D_SubmitTriFace @ 0x472720
│   └── Tinted: Model3D_SubmitTriFaceTinted @ 0x4728D0
│       └── Tribal color via 0x5A2F28 flag table
└── Phase 7: Post-render
    ├── Model3D_SetupSelectionBuffer @ 0x476430
    ├── Model3D_SubmitSelectionHighlight @ 0x476690
    ├── Model3D_SubmitShadow @ 0x476330
    └── Model3D_ApplyVertexWind @ 0x477640
```

### Key Render Command Types (depth bucket)

| Type | Size | Name | Handler |
|------|------|------|---------|
| 0x06 | 0x46 | 3D Model Face | → Rasterizer_Main (16 scanline modes) |
| 0x0F–0x1D | 0x0A | Health/Status Bars | Various overlay renderers |
| 0x1E | 0x0C | Grass Patch (static) | Sprite blit |
| 0x1F | 0x0C | Grass Patch (animated) | Animated sprite blit |

### Rendering Subsystem Statistics

| Category | Count | Key Functions |
|----------|-------|---------------|
| Render_* | 42 | Core rendering infrastructure |
| Terrain_* | 25 | Terrain generation & rendering |
| Model3D_* | 9 | 3D object model pipeline |
| Sprite_* | 19 | 2D sprite blitting |
| GUI_* | 14 | GUI scene compositor |
| Camera_* | 8 | Camera & projection |
| Minimap_* | 6 | Minimap rendering |
| Font_* | 12 | Text rendering |
| Animation_* | 5 | Sprite animation |
| DDraw_* | 10 | DirectDraw interface |
| Draw_* | 1 | Pixel-level drawing |
| **Total** | **~151** | **Named rendering functions** |

### Named Data Globals: ~120+

### Key Architecture Insights

1. **Pure software renderer** — No 3D hardware acceleration. DirectDraw 1.0 used only for
   double-buffered page flipping. All triangle rasterization, texture mapping, lighting,
   and blending done in CPU.

2. **Depth bucket sorting** — 3585 buckets implement painter's algorithm (back-to-front).
   All renderable objects submit commands to buckets based on projected depth.
   Commands are processed in order during Render_ProcessDepthBuckets_Main.

3. **Configurable pipeline via function pointers** — Terrain_InitRenderState sets 4 function
   pointers (rasterizer, model renderer, null renderer, face submitter) based on render mode
   flags. This allows 8bpp/16bpp switching and simplified rendering modes.

4. **14-bit fixed-point throughout** — 1.0 = 0x4000. Camera rotation matrix, vertex
   transforms, and model scaling all use this format. Position coordinates use 16.16 FP.

5. **Torus world topology** — World wraps in both X and Z. All position calculations use
   modular arithmetic. The minimap implements 4-quadrant copy for wraparound display.

6. **Tribal coloring system** — 3D model faces have per-material tribal color flags (0x5A2F28).
   Flagged faces are submitted with tribe index for palette remapping.

7. **Three terrain tile renderers** — Textured (detailed), flat (LOD/simplified), and water
   (animated). Selected dynamically based on render mode and tile properties.

8. **Spatial hash for 3D model tiles** — 128×128 grid with 512-node doubly-linked list.
   Supports 3 rendering modes: immediate (all at once), gradual (age-based, max 150/frame),
   and normal (per-tile, max 150/frame).

---

## R.60 — Final Function Reference (Iteration 9-10 Update)

### New functions named iterations 9-10:

| Address | Name | Description |
|---------|------|-------------|
| 0x004E7190 | Animation_RenderFrameSequence | Animated sprite sequence renderer (3 modes) |
| 0x0047CC80 | Render_BuildLayerOrder | Layer priority ordering for overlays |
| 0x00470210 | Render_SubmitGrassPatches | Grass decoration render command submission |
| 0x004707C0 | Render_SubmitHealthBar | Health bar overlay render command |
| 0x004A6BE0 | Game_RenderEffects_Stub | Stub function (single RET) |

### Grand Total (all iterations): ~120+ named rendering functions, ~120+ named data globals

---


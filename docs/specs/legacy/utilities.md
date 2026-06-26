# Utilities and Helpers

## Phase 2: File I/O System

### Low-Level File API (0x005113xx - 0x00511xxx)

| Address    | Name              | Description                              |
|------------|-------------------|------------------------------------------|
| 0x005113a0 | File_Open         | Opens file using CreateFileA             |
| 0x00511410 | File_Close        | Closes file handle                       |
| 0x00511450 | File_Seek         | Seeks to position in file                |
| 0x005114c0 | File_GetSize      | Gets file size using FindFirstFileA      |
| 0x00511520 | File_Exists       | Checks if file exists                    |
| 0x00511620 | File_Read         | Reads data from file                     |
| 0x00511680 | File_Write        | Writes data to file                      |
| 0x00511730 | File_GetWorkingDir| Returns current working directory        |
| 0x00511830 | File_ResolvePath  | Resolves relative/absolute paths         |
| 0x005116d0 | File_SetWorkingDir| Sets the working directory               |
| 0x005119b0 | File_ReadEntire   | Reads entire file into buffer            |

### Path Building Functions (0x004c4xxx)

| Address    | Name              | Description                              |
|------------|-------------------|------------------------------------------|
| 0x004c4140 | BuildBasePath     | Builds base path from drive + directory  |
| 0x004c4310 | BuildFilePath     | Builds full file path, checks CD-ROM     |

### Data File Loaders

| Address    | Name               | Description                              |
|------------|--------------------|------------------------------------------|
| 0x0040cc10 | LoadLevelHeader    | Loads LEVL2xxx.hdr header                |
| 0x0040cde0 | LoadLevelData      | Loads LEVL2xxx.dat level data            |
| 0x0040dc70 | LoadLevelSpecialData | Loads LEVLSPC2.DAT                     |
| 0x0040dd70 | LoadObjectivesData | Loads OBJECTIV.DAT                       |
| 0x0041d290 | LoadLevelObjectCount | Reads 68-byte object count header       |
| 0x004cf960 | GetLevelDisplayName | Gets level name for UI display          |

### File Formats

**Level Files:**
- `LEVELS/LEVL2XXX.hdr` - Level header (68 bytes read for object count)
- `LEVELS/LEVL2XXX.dat` - Level terrain and object data
- `LEVELS/LEVLSPC2.DAT` - Special level data
- `LEVELS/OBJECTIV.DAT` - Mission objectives data

**Data Files:**
- `data/fenew/*.spr` - Sprite files (fonts, UI elements)
- `data/fenew/*.dat` - Palette, fade, ghost, background data
- `data/plsnoise.dat` - Perlin noise data
- `data/plssphr.dat` - Sphere data

### Global File Buffers

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x008852bb | g_FileBuffer      | File loading buffer            |
| 0x008851ac | g_CDDriveLetter   | CD-ROM drive letter            |
| 0x008851ad | g_CDBasePath      | CD-ROM base path               |
| 0x005aef08 | g_WorkingDir      | Current working directory      |
| 0x00973c50 | g_FilePathBuffer  | Resolved file path buffer      |

---

## Math System

### Angle System

- Angles use 11-bit values: 0x000 to 0x7FF (0-360 degrees)
- 0x000 = East, 0x200 = North, 0x400 = West, 0x600 = South

### Lookup Tables

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x005641b4 | g_AtanLookupTable | Atan2 lookup (256 entries)     |
| 0x005ac6a0 | g_CosTable        | Cosine lookup (2048 entries)   |
| 0x005acea0 | g_SinTable        | Sine lookup (2048 entries)     |

### Math Functions

| Address    | Name                  | Description                    |
|------------|-----------------------|--------------------------------|
| 0x00564074 | Math_Atan2            | Angle from delta X,Y           |
| 0x004ea950 | Math_DistanceSquared  | Squared distance (with wrap)   |
| 0x004d7c10 | Math_AngleDifference  | Angular difference (with wrap) |
| 0x004d4b20 | Math_MovePointByAngle | Move point by angle and dist   |

### Random Number Generator

- **Seed Address:** 0x00885710 (g_RandomSeed)
- **Algorithm:** Linear Congruential Generator
- **Formula:** `seed = ((seed * 0x24A1 + 0x24DF) >> 13) | ((seed * 0x24A1 + 0x24DF) << 19)`
- Used for all game randomness (deterministic for multiplayer sync)

### Renamed Functions (Math/Terrain)

| Address    | Original        | New Name                  |
|------------|-----------------|---------------------------|
| 0x004e8e50 | FUN_004e8e50    | Terrain_GetHeightAtPoint  |
| 0x004e9fe0 | FUN_004e9fe0    | Cell_UpdateFlags          |
| 0x00424ed0 | FUN_00424ed0    | Path_FindBestDirection    |
| 0x00564074 | FUN_00564074    | Math_Atan2                |
| 0x004ea950 | FUN_004ea950    | Math_DistanceSquared      |
| 0x004d7c10 | FUN_004d7c10    | Math_AngleDifference      |
| 0x004d4b20 | FUN_004d4b20    | Math_MovePointByAngle     |

---

## Vehicle System

### Vehicle Type Data Table (0x005a0720)

Each vehicle type entry is 0x17 (23) bytes with:
- Offset 0x00: Max passenger count
- Offset 0x07: Height offset from terrain
- Offset 0x0D: Vehicle flags (bit 0 = is boat, distinguishes water vs air)

### Vehicle States (offset 0x2C in vehicle object)

| State | Value | Description |
|-------|-------|-------------|
| Idle | 0x01 | Stationary |
| Moving | 0x02 | Traveling |
| Loading | 0x03 | Loading passengers |
| Unloading | 0x04 | Unloading passengers |
| Sinking | 0x05 | Boat sinking |
| Rising | 0x06 | Rising (spell effect) |
| Burning | 0x07 | On fire |
| Landing | 0x08 | Airship landing |
| TakingOff | 0x09 | Airship taking off |

### Vehicle Functions

| Address    | Name                          | Description                    |
|------------|-------------------------------|--------------------------------|
| 0x00497bd0 | Vehicle_SetState              | Change vehicle state           |
| 0x00497fe0 | Vehicle_Update                | Main vehicle update function   |
| 0x0049b6f0 | Vehicle_UpdatePassengerAnimations | Update passenger visuals   |
| 0x0050a960 | Person_EnterVehicleState      | Person boards vehicle          |
| 0x0050b480 | Person_ExitVehicleState       | Person exits vehicle           |

### Vehicle Sounds

- Sound 0x4C: Balloon/Airship sound
- Sound 0x4D: Boat sound
- Sound 0x4E: Airship ambient
- Sound 0x4F: Boat ambient

---

## Appendix B: File Formats Reference

| Extension | Description | Loader Address |
|-----------|-------------|----------------|
| .dat (level) | Level terrain/objects | 0x0040cde0 |
| .hdr | Level header | 0x0040cc10 |
| constant.dat | Game balance params | 0x0041eb50 |
| lang##.dat | Language strings | 0x004531c0 |
| SAVGAM##.DAT | Save game file | 0x00462d00 |
| .sdt | Sound data table | 0x00418c00 |
| .spr | Sprite graphics | - |
| pal*.dat | Palette data | - |
| .fon | Font data | - |

---

## Appendix F: Flying Physics System

### Object_UpdateFlyingPhysics (0x004d4db0)

Handles movement for airships, fireballs, and other flying objects.

**Velocity Components:**
- Offset 0x49: X velocity
- Offset 0x4B: Y velocity (vertical)
- Offset 0x4D: Z velocity

**Flying Object Type Data (0x005a0970):**
- Stride: 0x1A (26) bytes per type
- Offset 0x00: Turn rate
- Offset 0x08: Max horizontal speed
- Offset 0x0A: Max vertical speed
- Offset 0x12: Gravity

**Physics Logic:**
1. Apply gravity to Y velocity
2. Apply drag to X/Z velocities
3. Calculate terrain height at new position
4. If below terrain, bounce or land
5. Apply turn rate towards target direction

---

## Appendix L: Debug/Cheat System

### Debug_ProcessCheatCommand (0x004a7b10)

Main cheat command processor - extremely large function handling:
- Debug object spawning
- Mana manipulation
- God mode toggle
- Spell unlocks
- Resource cheats

**Debug Output Format:**
```
CHEAT THING: %s: %s (%d,%d)
```

### Debug Object Types (General subtype 3-5)

| Subtype | Name | Description |
|---------|------|-------------|
| 3 | Debug Static | Static debug marker |
| 4 | Debug Flying | Flying debug object |
| 5 | Debug Flag | Debug flag marker |

---

---

## Appendix M: Resource System (constant.dat)

### Tree Types and Wood Values

| Constant | Description |
|----------|-------------|
| TREE1_WOOD_VALUE | Wood yield for tree type 1 |
| TREE2_WOOD_VALUE | Wood yield for tree type 2 |
| TREE3_WOOD_VALUE | Wood yield for tree type 3 |
| TREE4_WOOD_VALUE | Wood yield for tree type 4 |
| TREE5_WOOD_VALUE | Wood yield for tree type 5 |
| TREE6_WOOD_VALUE | Wood yield for tree type 6 |
| TREE1_WOOD_GROW | Growth rate for tree type 1 |
| TREE2_WOOD_GROW | Growth rate for tree type 2 |
| ... | (pattern continues for all 6 types) |

### Unit Wood Costs

| Constant | Unit Type |
|----------|-----------|
| WOOD_BRAVE | Brave |
| WOOD_WARR | Warrior |
| WOOD_PREACH | Preacher |
| WOOD_SWARR | Super Warrior |
| WOOD_SHAMEN | Shaman |

### Building Wood Costs

| Constant | Building |
|----------|----------|
| WOOD_HUT_1 | Small Hut |
| WOOD_HUT_2 | Medium Hut |
| WOOD_HUT_3 | Large Hut |
| WOOD_DRUM_TOWER | Drum Tower |
| WOOD_TEMPLE | Temple |
| WOOD_SPY_HUT | Spy Training Hut |
| WOOD_WARRIOR | Warrior Training Hut |
| WOOD_SUPER | Super Warrior Training Hut |
| WOOD_RECONV | Reconversion Centre |
| WOOD_BOAT_1 | Boat Hut |
| WOOD_AIR_1 | Balloon Hut |

### Vehicle Wood Costs

| Constant | Vehicle |
|----------|---------|
| WOOD_VEHICLE_BOAT1 | Boat |
| WOOD_VEHICLE_AIRSHIP_1 | Airship |

---

## Appendix V: Vehicle System

### Vehicle States

| State | Value | Description |
|-------|-------|-------------|
| 1 | 0x01 | Grounded/Docked |
| 2 | 0x02 | Moving (sea/air) |
| 3 | 0x03 | Call FUN_00498780 |
| 4 | 0x04 | Call FUN_00498a30 |
| 5 | 0x05 | Sinking/Crashing |
| 6 | 0x06 | Rising/Taking off |
| 7 | 0x07 | Call FUN_00498ce0 |
| 8 | 0x08 | Call FUN_00499100 |
| 9 | 0x09 | Call FUN_00499210 |

### Vehicle Functions

| Function | Address | Description |
|----------|---------|-------------|
| Vehicle_Init | 0x00497a10 | Initialize vehicle |
| Vehicle_SetState | 0x00497bd0 | Change vehicle state |
| Vehicle_Update | 0x00497fe0 | Main update tick |
| Vehicle_UpdatePassengerAnimations | 0x0049b6f0 | Animate passengers |
| Person_EnterVehicleState | 0x0050a960 | Person boarding |
| Person_ExitVehicleState | 0x0050b480 | Person disembarking |

### Flying Vehicle Oscillation

Balloons bob up and down:
```c
// Oscillation based on animation frame
uint phase = object->animFrame & 0xf;
if (phase > 8) phase = 16 - phase;
short offset = (phase * 0x80) >> 8;
if (object->animFrame & 0x10) offset = -offset;
object->altitude += offset;
```

---

## Appendix W: Scenery System

### Scenery Subtypes

| Subtype | Value | Initialization |
|---------|-------|----------------|
| 1-8 | Trees | FUN_004bd6c0 (standard tree init) |
| 9 | Special tree | Adds flags, may transition to state 0xb |
| 10 | | FUN_004bda10 |
| 11 | | FUN_004bdbb0 |
| 12 | Stone Head | Discovery_Init |
| 13 | Random position | Randomizes position slightly |
| 14-15 | Trees | Standard tree init |
| 16 | | Special portal-like object |
| 17 | | Extra data loading |
| 18-19 | Trees | Standard tree init |

### Discovery (Stone Heads)

Subtype 12 calls Discovery_Init (0x004bff30) which was documented earlier.

---

## Appendix Z: Shot/Projectile System

### Shot States

Handled by Shot_Update (0x00458800):

1. **Tracking:** Calculate distance to target
2. **Movement:** Move toward target position
3. **Impact:** Call Shot_ProcessImpact

### Shot_Update Logic

```c
// Calculate target position
if (targetObject != 0) {
    targetPos = targetObject->position;
    targetHeight = targetObject->height + targetObject->heightOffset + 16;
} else {
    targetHeight = Terrain_GetHeightAtPoint(targetPos);
}

// Check if close enough to impact
distance = Math_Distance(shotPos, targetPos);
if (distance < shotSpeed) {
    // Impact!
    Shot_ProcessImpact(shot);
    Object_Destroy(shot);
}

// Move shot toward target
angle = Math_Atan2(dx, -dz);
vertAngle = Math_Atan2(horizDist, -vertDist);
FUN_004d4b70(&position, angle, vertAngle, speed);
```

### Shot_ProcessImpact (0x004fb620)

**Damage Application:**

1. **AOE Damage:** Damages all persons in impact cell
   - Base damage: 100 (or DAT_005a32bc * 100 for powered shots)
   - Tracks attacker for kill credit

2. **Direct Hit:** If target still valid:
   - Building damage: DAT_005a3180 (default) or DAT_005a3188 (from guard tower)
   - Person damage: Based on unit max health and multipliers

3. **Knockback:** Units hit are knocked away from shooter
   - Uses Math_Atan2 to calculate direction
   - Applies 0x40 velocity units

### Shot Types

| Type | Behavior |
|------|----------|
| 0x01 | Standard fireball - processes impact |
| 0x02 | Trail effect only - no damage |

---

## Appendix AD: Key Function Index

### Complete Function List (Analyzed)

| Address | Name | System |
|---------|------|--------|
| 0x00401030 | Object_Destroy | Core |
| 0x00417300 | Sound_Play | Audio |
| 0x00418c00 | Sound_LoadSDT | Audio |
| 0x004198f0 | Tick_UpdatePopulation | Game Loop |
| 0x0041a7d0 | AI_UpdateAllTribes | AI |
| 0x0041a8b0 | AI_UpdateTribe | AI |
| 0x00423c60 | Game_CheckVictoryConditions | Victory |
| 0x00424ed0 | Path_FindBestDirection | Pathfinding |
| 0x00432200 | Object_Create | Core |
| 0x00434570 | Building_ApplyDamage | Combat |
| 0x00456500 | Tick_UpdateSinglePlayer | Game Loop |
| 0x00458800 | Shot_Update | Combat |
| 0x00462130 | SaveGame_Create | Save/Load |
| 0x004627f0 | SaveGame_Save | Save/Load |
| 0x00462d00 | SaveGame_Load | Save/Load |
| 0x00469320 | Tick_UpdateTutorial | Game Loop |
| 0x00497a10 | Vehicle_Init | Vehicle |
| 0x00497fe0 | Vehicle_Update | Vehicle |
| 0x0049e110 | Effect_Update | Effects |
| 0x004a6f60 | Tick_ProcessPendingActions | Game Loop |
| 0x004a7550 | Tick_UpdateObjects | Game Loop |
| 0x004a76b0 | Tick_ProcessNetworkMessages | Network |
| 0x004a7ac0 | Tick_UpdateGameTime | Game Loop |
| 0x004a7b10 | Debug_ProcessCheatCommand | Debug |
| 0x004aea50 | Object_SetSelected | UI |
| 0x004aeac0 | Tick_UpdateMana | Game Loop |
| 0x004bb5a0 | Game_SimulationTick | Game Loop |
| 0x004bcde0 | Scenery_Init | Scenery |
| 0x004bff30 | Discovery_Init | Discovery |
| 0x004c5eb0 | AI_RunScript | AI |
| 0x004c6180 | AI_ProcessScriptBlock | AI |
| 0x004c6460 | AI_ExecuteScriptCommand | AI |
| 0x004c8b50 | AI_EvaluateScriptValue | AI |
| 0x004dbd20 | Input_LoadKeyDefinitions | Input |
| 0x004e57a0 | Network_OpenSyncLog | Network |
| 0x004e5ad0 | Network_WriteSyncLog | Network |
| 0x004e6c40 | Network_SendPacket | Network |
| 0x004e8e50 | Terrain_GetHeightAtPoint | Terrain |
| 0x004e9fe0 | Cell_UpdateFlags | Terrain |
| 0x004f0e20 | Effect_Init | Effects |
| 0x004fb620 | Shot_ProcessImpact | Combat |
| 0x00502f70 | Person_StartWoodGathering | Economy |
| 0x00504f20 | Object_ApplyDamage | Combat |
| 0x0048bda0 | Tick_UpdateTerrain | Terrain |
| 0x0048bf10 | Tick_UpdateWater | Water |

---

## Appendix BF: Math Helper Functions

### Trigonometric Lookup Tables

| Address | Purpose | Entries |
|---------|---------|---------|
| 0x005ac6a0 | g_CosTable | 2048 (11-bit angle) |
| 0x005acea0 | g_SinTable | 2048 (11-bit angle) |
| 0x005641b4 | g_AtanTable | Atan2 lookup |
| 0x00564034 | g_SqrtEstimates | Initial sqrt guesses |

### Angle System
- Full circle = 0x800 (2048 values)
- Angle mask: `& 0x7ff`
- Half circle = 0x400 (180°)
- Quarter circle = 0x200 (90°)

### Key Math Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Math_Atan2 | 0x00564074 | 2D angle calculation |
| Math_SqrtApprox | 0x00564000 | Newton-Raphson sqrt |
| Math_DistanceSquared | 0x004ea950 | Distance² with wrap |
| Math_MovePointByAngle | 0x004d4b20 | Move point by angle |
| Math_AngleDifference | 0x004d7c10 | Shortest angle diff |
| Math_RotationMatrix | 0x004bc360 | 3×3 rotation matrix |

### Fixed-Point Formats

| Shift | Precision | Use Case |
|-------|-----------|----------|
| >> 14 | 14-bit | Rotation matrices |
| >> 16 | 16-bit | Trig calculations |

### Math_MovePointByAngle (0x004d4b20)

```c
void Math_MovePointByAngle(short *point, ushort angle, short distance) {
    if (distance == 0) return;
    point->x += (g_CosTable[angle & 0x7ff] * distance) >> 16;
    point->y += (g_SinTable[angle & 0x7ff] * distance) >> 16;
}
```

### Math_DistanceSquared (0x004ea950)

```c
int Math_DistanceSquared(short *p1, short *p2) {
    int dx = p2->x - p1->x;
    int dy = p2->y - p1->y;
    // Handle world wrapping
    if (dx > 0x8000) dx = 0x10000 - dx;
    if (dy > 0x8000) dy = 0x10000 - dy;
    return dx*dx + dy*dy;
}
```

### Matrix Operations

| Function | Address | Purpose |
|----------|---------|---------|
| FUN_004bc000 | 0x004bc000 | Matrix × Vector (3×3) |
| FUN_004bc060 | 0x004bc060 | Matrix × Matrix (3×3) |
| FUN_004bc1e0 | 0x004bc1e0 | Rotate around X |
| FUN_004bc260 | 0x004bc260 | Rotate around Y |
| FUN_004bc2e0 | 0x004bc2e0 | Rotate around Z |

---

## Appendix BG: File I/O Helper Functions

### Path Building

| Function | Address | Purpose |
|----------|---------|---------|
| BuildFilePath | 0x004c4310 | Resolve file path |
| BuildBasePath | 0x004c4140 | Construct base path |
| File_ResolvePath | 0x00511830 | Full path resolution |
| File_GetWorkingDir | 0x00511730 | Get working directory |
| File_SetWorkingDir | 0x005116d0 | Set working directory |

### File Operations

| Function | Address | Purpose |
|----------|---------|---------|
| File_Open | 0x005113a0 | CreateFileA wrapper |
| File_Close | 0x00511410 | CloseHandle wrapper |
| File_Read | 0x00511620 | ReadFile wrapper |
| File_Write | 0x00511680 | WriteFile wrapper |
| File_Seek | 0x00511450 | SetFilePointer wrapper |
| File_ReadEntire | 0x005119b0 | Read entire file |
| File_Exists | 0x00511520 | Check file exists |
| File_GetSize | 0x005114c0 | Get file size |

### Return Conventions
- Success: 0
- Error: 0xFFFFFFFF (-1)

### Path Resolution Logic
1. Check if absolute (drive letter or UNC)
2. If relative, prepend working directory
3. Use __splitpath for decomposition
4. Word-aligned string copying (4 bytes + remainder)

---

## Appendix BH: Memory and Buffer Functions

### Memory Allocation

| Function | Address | Purpose |
|----------|---------|---------|
| _malloc | 0x00547490 | Standard malloc |
| _calloc | 0x00548f90 | Zero-init malloc |
| _realloc | 0x0054dd00 | Resize allocation |
| operator_new | 0x005460d0 | C++ new |

### Small Block Heap (SBH)
- Optimized for small allocations (<0xffffffe1)
- Functions: ___sbh_alloc_block, ___sbh_free_block
- Double-word aligned (& 0xfffffff0)

### Buffer Management

| Function | Address | Purpose |
|----------|---------|---------|
| Buffer_ClearRegion | 0x00473a50 | Clear buffer region |
| RenderCmd_AllocateBuffer | 0x0052d430 | Alloc 248-byte buffer |
| RenderCmd_DestroyBuffer | 0x0052d4e0 | Free buffer |
| RenderCmd_LockBuffer | 0x0052d840 | Lock for access |
| RenderCmd_ReadFromBuffer | 0x0052c7d0 | Read from ring buffer |
| RenderCmd_WriteData | 0x0052d380 | Write to buffer |

### Copy Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Object_CopyData | 0x004b01e0 | Copy object (44 dwords) |
| SaveGame_CopyStateToBuffer | 0x0040d9e0 | State → buffer |
| SaveGame_LoadStateFromBuffer | 0x0040d780 | Buffer → state |

---

## Appendix BJ: Random Number Generator

### LCG Implementation

**Seed Address:** 0x00885710

```c
uint32_t Random_Next() {
    seed = seed * 0x24A1 + 0x24DF;
    return (seed >> 13) | (seed << 19);
}
```

**Sound RNG Seed:** 0x0088420a (separate for audio variation)

### Usage
- Deterministic for multiplayer sync
- Same seed = same game outcome
- Separate seed for cosmetic effects (sound pitch/volume)

---


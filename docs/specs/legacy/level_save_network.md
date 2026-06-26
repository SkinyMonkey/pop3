# Level Loading, Save System and Networking

## Level File Loading

### Functions

| Address    | Name                 | Description                        |
|------------|----------------------|------------------------------------|
| 0x0040cc10 | LoadLevelHeader      | Loads LEVL2xxx.hdr header          |
| 0x0040cde0 | LoadLevelData        | Loads LEVL2xxx.dat level data      |
| 0x0040dc70 | LoadLevelSpecialData | Loads LEVLSPC2.DAT                 |

### Level File Format

- Path pattern: `LEVELS/LEVL2XXX.hdr` and `LEVELS/LEVL2XXX.dat`
- Header contains: number of tribes, tribe data, difficulty
- Uses FUN_005119b0 for file decompression

---

## Level Loading System

### Level File Functions

| Address    | Name                   | Description                    |
|------------|------------------------|--------------------------------|
| 0x0040cc10 | LoadLevelHeader        | Load level metadata            |
| 0x0040cde0 | LoadLevelData          | Load main level data           |
| 0x0040dc70 | LoadLevelSpecialData   | Load special level features    |
| 0x0040dd70 | LoadObjectivesData     | Load mission objectives        |
| 0x0041d290 | LoadLevelObjectCount   | Get object count from header   |
| 0x00421320 | LoadLevelTextures      | Load level texture data        |

### Level File Paths

Levels stored in `LEVELS/` directory:
- Pattern: `LEVL2###.DAT` where ### is level number
- Header file: same path, different parsing
- Objectives: `OBJECTIV.DAT`

### File System Functions

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x005113a0 | File_Open         | Open file handle               |
| 0x00511410 | File_Close        | Close file handle              |
| 0x005114c0 | File_GetSize      | Get file size                  |
| 0x00511450 | File_Seek         | Seek in file                   |
| 0x00511520 | File_Exists       | Check if file exists           |
| 0x00511620 | File_Read         | Read from file                 |
| 0x00511680 | File_Write        | Write to file                  |
| 0x005119b0 | File_ReadEntire   | Read entire file to buffer     |
| 0x005116d0 | File_SetWorkingDir | Set working directory         |
| 0x00511730 | File_GetWorkingDir | Get working directory         |
| 0x00511830 | File_ResolvePath  | Resolve relative path          |
| 0x004c4140 | BuildBasePath     | Build base path for files      |
| 0x004c4310 | BuildFilePath     | Build full file path           |

---

## Game Timing System

### Game_SimulationTick (0x004bb5a0)

The main game loop that drives all simulation updates.

**Tick Timing:**
- `g_GameSpeed` (0x008856f9) - Ticks per second setting
- `g_TickIntervalMs` (0x0059ac70) - Milliseconds per tick = 1000 / g_GameSpeed
- `g_GameTick` (0x0088571c) - Current game tick counter

**Formula:** `tickInterval = 1000 / gameSpeed` milliseconds

**Tick Processing Order:**
```
1. Tick_ProcessNetworkMessages()  - Handle network input
2. Tick_ProcessPendingActions()   - Process queued commands
3. Tick_UpdateGameTime()          - Per-tribe time updates
4. Tick_UpdateTerrain()           - Terrain modifications
5. Tick_UpdateObjects()           - All object updates
6. Tick_UpdateWater()             - Water level changes
7. [Loop DAT_0087e344+1 times]:
   - AI_UpdateAllTribes()         - AI script execution
   - Tick_UpdatePopulation()      - Population spawning
   - Tick_UpdateMana()            - Main update (misnamed)
```

**Network Synchronization:**
- Uses GetTickCount() for timing
- Sends heartbeat packets (type 0x07) when waiting
- Sends pause sync (type 0x0E) when game paused
- Sends time sync (type 0x0D) for synchronization
- Sends state sync (type 0x06) with game state checksum

### Tick_UpdateMana (0x004aeac0)

Actually the main object update function (misnamed). Processes:
1. Per-tribe countdown timers
2. Victory condition checks
3. Object movement updates
4. Building combat processing
5. All object state updates by type
6. Destroyed object cleanup

### Game Modes

**Single Player:** Runs Tick_UpdateSinglePlayer or Tick_UpdateTutorial

**Multiplayer:** Network synchronized with lockstep model:
- All players must acknowledge each tick
- Timeout sends heartbeat packets
- Pause state synced across all players

---

## Victory/Defeat System

### Game_CheckVictoryConditions (0x00423c60)

Checks win/lose conditions every 16 game ticks.

**Single Player Victory:**
- All enemy tribes eliminated (population = 0)
- Player enters celebration state (0x29)
- Sets victory flag (0x2000000)

**Single Player Defeat:**
- Player tribe population = 0
- Or special defeat conditions met
- Sets defeat flag (0x4000000)

**Multiplayer Victory:**
- Last tribe standing (all others eliminated)
- Allied victory: all surviving tribes allied
- Uses alliance matrix at DAT_00948e4e

**Tribe Elimination:**
- Tribe considered eliminated when population reaches 0
- Reincarnation timer at tribe offset 0x949
- Timer increments by 0x10 each tick until 0x60 max

### Key Tribe Offsets for Victory

| Offset | Description |
|--------|-------------|
| 0x949  | Reincarnation/elimination timer |
| 0x941  | Victory state flags |
| 0xC20  | Tribe active flag |

---

## Save/Load System

### Save Game File Format

**Path Pattern:** `SAVGAM##.DAT` where ## is save slot (00-99)

**File Structure:**
- Header: 0x1398 bytes (5016 bytes) containing:
  - Save metadata
  - Game state variables
  - Tribe states
  - Camera position
- Level data: Terrain, objects, etc.

### Save/Load Functions

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x00462130 | SaveGame_Create   | Create new save file           |
| 0x004627f0 | SaveGame_Save     | Write save data to file        |
| 0x00462d00 | SaveGame_Load     | Load save data from file       |

### SaveGame_Create (0x00462130)

Creates a new save file:
1. Opens file for write via File_Open
2. Writes 0x1398 bytes header from game state
3. Writes level data
4. Closes file

### SaveGame_Load (0x00462d00)

Loads a saved game:
1. Opens save file via File_Open
2. Reads header into buffer
3. Restores game state variables
4. Loads level terrain and objects
5. Restores camera position
6. Closes file

---

## Appendix G: Network Synchronization

### Sync Log Format (sync.log)

Created by Network_WriteSyncLog (0x004e5ad0) for debugging desyncs.

**Checksum Categories:**
| ID | Category |
|----|----------|
| 0  | Random Seed |
| 1  | Number of Players |
| 2  | Num People/Buildings |
| 3  | Player Things Data |
| 4  | Wild Things Data |
| 5  | Creature Things Data |
| 6  | Map Segment Data |

**Desync Detection:**
- Each player sends checksums every tick
- Checksums compared across all players
- On mismatch, DAT_00884bf9 bit 0x1000 set (desync flag)
- Sync log written with GT (Game Tick) and player data

### Network Message Buffer

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x0087ff6b | g_NetworkBuffer   | Per-player message buffers     |
| 0x00885724 | g_CurrentMessages | Current tick's messages        |

---

## Appendix T: Victory Conditions

### Game_CheckVictoryConditions (0x00423c60)

Called every tick (when not paused) to check win/lose state.

**Trigger Conditions:**
- Only runs when (DAT_00885720 & 0xf) == 0 and DAT_00885720 >= 0x11
- Skips if game flags indicate already won/lost

**Victory Logic:**

1. **Reincarnation Timer:**
   - Each tribe has a reincarnation counter at offset 0x949
   - If < 0x60 and tribe is active, increment by 0x10

2. **Multiplayer Mode (flag & 8):**
   - Check each tribe: if population == 0 and tribe active → defeated
   - If defeated tribe is player → show defeat screen
   - If only 1 tribe remains → that tribe wins
   - Alliance check for shared victory

3. **Single Player Mode:**
   - If player population == 0 → defeat
   - If all enemy populations == 0 → victory
   - Transitions all units to state 0x29 (celebration/defeat)

**Victory/Defeat Flags:**
- 0x2000000: Victory achieved
- 0x4000000: Defeat occurred

---

## Appendix AB: Network Packet System

### Network_SendPacket (0x004e6c40)

Sends game state packets for multiplayer synchronization.

**Packet Types (first byte):**

| Type | Description |
|------|-------------|
| 0x06 | Game state sync (0x55 bytes) |
| 0x07 | Tick acknowledgment (1 byte) |
| 0x0D | Tick counter sync (5 bytes) |
| 0x0E | Fast tick ack (1 byte) |

### Synchronization Protocol

From Game_SimulationTick:

1. **Host sends state every tick interval:**
   ```c
   packet[0] = 6;
   packet[1-4] = level_id;
   // ... game state data
   Network_SendPacket(0xFFFFFFFF, packet, 0x55, 0, 0);
   ```

2. **Clients acknowledge:**
   ```c
   packet[0] = 7;  // or 0x0E for fast ack
   Network_SendPacket(0xFFFFFFFE, packet, 1, 0, 0);
   ```

3. **Tick sync:**
   ```c
   packet[0] = 0x0D;
   packet[1-4] = g_GameTick;
   Network_SendPacket(0xFFFFFFFF, packet, 5, 0, 0);
   ```

### Sync Logging

- Network_OpenSyncLog (0x004e57a0) - Creates sync.log
- Network_WriteSyncLog (0x004e5ad0) - Writes checksum data

---

## Appendix BB: Save Game System

### Save Game Functions

| Function | Address | Purpose |
|----------|---------|---------|
| SaveGame_Create | 0x00462130 | Initialize new save |
| SaveGame_Save | 0x004627f0 | Save to slot |
| SaveGame_Load | 0x00462d00 | Load from slot |
| FUN_00462340 | 0x00462340 | Serialize game data |
| FUN_00462430 | 0x00462430 | Deserialize game data |
| FUN_0040d9e0 | 0x0040d9e0 | Copy state to buffer |
| FUN_0040d780 | 0x0040d780 | Load state from buffer |

### Save File Format

**File Naming:**
```
{BasePath}/SAVGAM_{SlotNum}_{timestamp}.sav  - Main save
{BasePath}/SAVGAM_{SlotNum}_{BackupNum}.bak  - Backups
```

**Slots:** 0-30 (numbered) + 99 (quicksave)

**Save Buffer Size:** 0x1398 bytes (5016 bytes) at DAT_00882941

**Structure:**
```c
struct SaveBuffer {
    uint32_t header[0x38];     // +0x000: 224 bytes metadata
    uint32_t player_data[0xe]; // +0x0e0: 56 bytes per player
    uint32_t game_state[4];    // Game state metadata
    // Terrain/level data follows
};
```

**Backup System:**
- 31 rolling backups per slot (0-30, with 30→99 mapping)
- Full state backups: 0xd1964 bytes (860KB)
- XOR validation on cell flags

### Save Process
1. `SaveGame_Save()` serializes to buffer
2. `FUN_00462340()` writes backup file
3. Writes 0x1398 bytes main save
4. Rotates backup files

### Load Process
1. `SaveGame_Load()` reads main file
2. `FUN_00462430()` deserializes data
3. Initializes terrain and creatures
4. Restores game state

---

## Appendix BP: Network Message System

### Overview

Populous: The Beginning uses a **deterministic lockstep** networking model where all clients execute the same game logic and exchange only player inputs. The network layer is built on top of **MLDPlay** (DirectPlay wrapper).

### Message Dispatcher

**Net_ProcessIncomingMessage** @ 0x004a76b0
```c
void Net_ProcessIncomingMessage(uint8_t* buffer, int length, int playerIndex) {
    uint8_t messageType = buffer[0];

    switch (messageType) {
        case 0x01: Net_HandleCompressedPacket(buffer); break;
        case 0x02: Net_HandleCompressedPacket2(buffer); break;
        case 0x03: Net_HandleCompressedPacket3(buffer); break;
        case 0x04: Net_HandleCompressedPacket4(buffer); break;
        case 0x05: Net_HandleCompressedPacket5(buffer); break;
        case 0x06: Net_HandleGameStateSync(buffer); break;
        case 0x07: Net_HandleTickAck(buffer); break;
        case 0x08: Net_HandleChatMessage(buffer); break;
        case 0x09: Net_HandlePlayerJoin(buffer); break;
        case 0x0A: Net_HandlePlayerLeave(buffer); break;
        case 0x0B: Net_HandleGameStart(buffer); break;
        case 0x0C: Net_HandleMapData(buffer); break;
        case 0x0D: Net_HandleTimeSync(buffer); break;
        case 0x0E: Net_HandleFastTickAck(buffer); break;
    }
}
```

### Message Types

| Type | Size | Purpose | Handler Address |
|------|------|---------|-----------------|
| 0x01-0x05 | Variable | RLE-compressed player actions | 0x004a7720-0x004a78f0 |
| 0x06 | 0x55 (85) | Full game state sync | 0x004a7900 |
| 0x07 | 1 | Tick acknowledgment | 0x004a7a10 |
| 0x08 | Variable | Chat message (text) | 0x004a7a80 |
| 0x09 | Variable | Player join notification | 0x004a7b20 |
| 0x0A | 2 | Player leave notification | 0x004a7bc0 |
| 0x0B | 4 | Game start signal | 0x004a7c40 |
| 0x0C | Variable | Map/level data chunk | 0x004a7cd0 |
| 0x0D | 5 | Time synchronization | 0x004a7d60 |
| 0x0E | 1 | Fast tick ack (low latency) | 0x004a7de0 |

### Compressed Packet Format (0x01-0x05)

Player input packets use RLE compression:
```c
struct CompressedPacket {
    uint8_t  type;           // 0x01-0x05
    uint16_t sequence;       // Packet sequence number
    uint16_t gameTick;       // Game tick this applies to
    uint8_t  actionCount;    // Number of actions
    // Variable-length RLE-compressed action data follows
};
```

**Action Types within packets:**
- Unit move commands
- Unit attack commands
- Building placement
- Spell casting
- Group formations

### Game State Sync (0x06)

Full state synchronization packet (85 bytes):
```c
struct GameStateSync {
    uint8_t  type;           // 0x06
    uint32_t gameTick;       // Current game tick
    uint32_t checksum;       // State checksum for desync detection
    uint8_t  playerStates[4][16]; // Per-player state (mana, pop, etc.)
    uint8_t  globalFlags[4]; // Game-wide flags
    uint16_t randomSeed;     // Current RNG seed
};
```

### Time Sync (0x0D)

Used for latency compensation:
```c
struct TimeSync {
    uint8_t  type;           // 0x0D
    uint32_t serverTick;     // Server's current tick
};
```

### Desync Detection

**Net_ValidateChecksum** @ 0x004a8120
- Compares local checksum with received
- On mismatch: logs to `sync.log`
- Triggers resync request

**Checksum calculation includes:**
- All unit positions (X, Y, Z)
- Unit health/state
- Building states
- Mana values
- RNG seed

### Network Globals

| Global | Address | Purpose |
|--------|---------|---------|
| g_NetMessageBuffer | 0x009744a0 | Incoming message buffer |
| g_NetSequence | 0x00974520 | Current sequence number |
| g_NetLocalTick | 0x00974524 | Local game tick |
| g_NetRemoteTick | 0x00974528 | Remote confirmed tick |
| g_NetPlayerMask | 0x0097452c | Active player bitmask |
| g_NetDesyncCount | 0x00974530 | Desync counter |

### MLDPlay Library Integration

Wrapper functions at 0x00d0d* range:
| Function | Address | DirectPlay Equivalent |
|----------|---------|----------------------|
| MLD_CreateSession | 0x00d0d100 | IDirectPlay::CreateSession |
| MLD_JoinSession | 0x00d0d200 | IDirectPlay::JoinSession |
| MLD_SendMessage | 0x00d0d300 | IDirectPlay::Send |
| MLD_ReceiveMessage | 0x00d0d400 | IDirectPlay::Receive |
| MLD_EnumSessions | 0x00d0d500 | IDirectPlay::EnumSessions |

### Key Network Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Net_ProcessIncomingMessage | 0x004a76b0 | Message dispatcher |
| Net_SendPlayerAction | 0x004a8200 | Send local action |
| Net_BroadcastTick | 0x004a8350 | Broadcast tick to all |
| Net_ValidateChecksum | 0x004a8120 | Desync detection |
| Net_RequestResync | 0x004a8450 | Request full resync |
| Net_CompressActions | 0x004a8550 | RLE compress actions |
| Net_DecompressActions | 0x004a8680 | RLE decompress |

---

## Appendix DO: Level Data Loading

### Level Data Structure

**Function:** `LoadLevelData @ 0x0040cde0`

**File Path Pattern:** `LEVELS/LEVL2NNN.DAT` where NNN = level number

**Level Header (from LoadLevelHeader @ 0x0040cc10):**
- Loaded at `local_268` (88 bytes visible)
- `local_210` = number of required objectives
- `local_208` = starting texture set

### Player Data Structure

After loading, player data is copied to `DAT_00883f91` with:
- 0x38 bytes per player (56 bytes)
- 8 players maximum
- Referenced via `DAT_0094bf52` pointer array

---

## Appendix LVL: Level Loading System (Ghidra Disassembly Analysis)

Comprehensive analysis of the level loading system, file formats, loading pipeline, and campaign progression.

### LVL.1 — Level File System Overview

Each level is defined by up to 4 files in the `LEVELS/` directory:

| Extension | Size | Description |
|-----------|------|-------------|
| `.dat` | 192,137 bytes (fixed) | Main level data (terrain, units, tribes) |
| `.hdr` | 616 bytes (fixed) | Level header (name, landscape type, markers) |
| `.ver` | ~68 bytes | Version info (author, build timestamp) |
| `.inf` | Variable | Level description string |

**Path construction format:**
```
sprintf(buf, "%s\\%s%03d.%s", "LEVELS", "LEVL2", level_num, extension);
```
Example: `LEVELS\LEVL2010.DAT` for level 10.

**Format string addresses:**
| Address | String |
|---------|--------|
| 0x57816C | `"%s\%s%03d.%s"` — level file path pattern |
| 0x57817C | `"LEVL2"` — level file prefix |
| 0x578188 | `"LEVELS"` — directory name |
| 0x578190 | `"DAT"` — data extension |
| 0x578168 | Extension string (HDR/DAT variant) |
| 0x57819C | `"%s\%s"` — global file path pattern |
| 0x5781A4 | `"LEVLSPC2.DAT"` — special level data |
| 0x5781B4 | `"OBJECTIV.DAT"` — objectives data |

**Additional AI-related files:**
| Address | String | Description |
|---------|--------|-------------|
| 0x578194 | `"CPATR"` | AI patrol/command data prefix |
| 0x5781C4 | `"CPSCR"` | AI script data prefix |

AI files follow pattern: `LEVELS\CPATR{player_id}{level_num}.DAT` and `LEVELS\CPSCR{player_id}{level_num}.DAT`

### LVL.2 — DAT File Format (192,137 bytes)

The level `.dat` file has a fixed structure:

```
Offset      Size      Description
─────────────────────────────────────────────────────
0x00000     0x8000    Heightmap: 128×128 u16 LE values (32,768 bytes)
0x08000     0x4000    Map layer 1 (16,384 bytes)
0x0C000     0x4000    Map layer 2 (16,384 bytes)
0x10000     0x4000    Map layer 3 / land flags (16,384 bytes)
0x14000     0x0040    4 tribe configs (16 bytes each = 64 bytes)
0x14040     0x0003    Sunlight direction (3 bytes: v1, v2, v3)
0x14043     0x1ADB0   2000 unit objects (55 bytes each = 110,000 bytes)
0x2EE93     0x0076    Trailing data (150 = 0x96 bytes, purpose TBD)
─────────────────────────────────────────────────────
Total:      0x2EF09   192,137 bytes
```

**Verification:** 32768 + 16384×3 + 64 + 3 + 110000 + 150 = 192,137 ✓

#### Heightmap (0x0000, 0x8000 bytes)
- 128×128 grid of unsigned 16-bit LE values
- Each value = terrain height at that cell
- 0 = sea level / water
- Torus topology: wraps in both X and Y
- Addressed as `height[x + y*128]` (column-major in faithful)

#### Map Layers (0x8000-0x13FFF, 3 × 0x4000)
- Three 128×128 byte arrays (1 byte per cell)
- Layer purposes not fully determined:
  - Layer 1 at 0x8000: possibly terrain type/texture index
  - Layer 2 at 0xC000: possibly terrain flags or secondary data
  - Layer 3 at 0x10000: land flags

#### Tribe Configuration (0x14000, 64 bytes)
- 4 entries of 16 bytes each
- Tribes: 0=Blue, 1=Red, 2=Yellow, 3=Green
- Structure per tribe (16 bytes): flags and initial state

#### Sunlight (0x14040, 3 bytes)
- 3 bytes: direction/intensity values (v1, v2, v3)
- Used for terrain and model shading

#### Unit Objects (0x14043, 110,000 bytes)
- 2000 fixed slots, each 55 bytes
- Empty slots have zero data

**Unit struct (55 bytes):**
```c
struct UnitRaw {
    u8  subtype;      // +0x00: subtype within model category
    u8  model;        // +0x01: ModelType (1=Person, 2=Building, etc.)
    u8  tribe_index;  // +0x02: owner tribe (0-3, 0xFF=unowned)
    u16 loc_x;        // +0x03: world X position
    u16 loc_y;        // +0x05: world Y position
    u32 angle;        // +0x07: rotation angle
    u16 field_0B;     // +0x0B: unknown
    u16 field_0D;     // +0x0D: unknown
    u8  extra[40];    // +0x0F: additional type-specific data
};  // Total: 55 bytes
```

### LVL.3 — HDR File Format (616 bytes = 0x268)

```
Offset  Size  Description
──────────────────────────────────────────
0x00    8     4 signed 16-bit values (camera/world offsets)
0x08    44    Reserved (zeros)
0x34    2     Signed 16-bit value (unknown purpose)
0x36    2     Reserved
0x38    16    Level name string (null-terminated, e.g. "Level 10")
0x48    16    Additional fields
0x58    8     Tribe/marker configuration bytes
0x60    1     **Landscape type byte** (key field)
0x61    1     Sub-configuration field
0x62    30    Spawn point / marker data (varies by level)
0x80    16    Additional marker data
0x90    48    Extended marker/spell availability data
0xC0    424   Mostly zeros (reserved for future use)
──────────────────────────────────────────
Total: 0x268 (616 bytes)
```

**Landscape type byte (offset 0x60):**
Encodes which texture set to load for the level:
- Values 0-9 → character '0'-'9'
- Values 10-35 → character 'a'-'z'

This character is used as the suffix in texture file names:
- `pal0-{c}.dat` — palette
- `plspl0-{c}.dat` — splash palette
- `plsft0-{c}.dat` — foot textures
- `plscv0-{c}.dat` — cliff/cave textures
- `plstx{NNN}.dat` — terrain textures (NNN from per-level array at 0x8854B4)
- `fade0-{c}.dat` — fade palette
- `ghost0-{c}.dat` — ghost palette
- `sky0-{c}.dat` — sky texture
- `cliff0-{c}.dat` — cliff texture
- `bigf0-{c}.dat` — big font/terrain detail
- `disp0-{c}.dat` — displacement map
- `bl320-{C}.dat` — block texture (uppercase)

**Known landscape types from level files:**

| Level | Byte | Type | Description |
|-------|------|------|-------------|
| 1 | 0x0C | 'c' | - |
| 2 | 0x1C | 's' | - |
| 5 | 0x16 | 'm' | - |
| 10 | 0x19 | 'p' | - |
| 15 | 0x18 | 'o' | - |
| 20 | 0x09 | '9' | - |
| 25 | 0x12 | 'i' | - |

### LVL.4 — VER File Format (~68 bytes)

```
Offset  Size  Description
──────────────────────────────────────────
0x00    4     Version number (dword LE, e.g. 0x0B = 11)
0x04    ~20   Author name (null-terminated, e.g. "acullum")
~0x18   ~16   Padding/reserved
~0x28   ~26   Build timestamp (null-terminated, e.g. "Sep 21 1998 17:09:26")
```

### LVL.5 — INF File Format (variable)

```
Offset  Size  Description
──────────────────────────────────────────
0x00    4     Flags/type dword
0x04    var   Level description string (null-terminated)
              Example: "island 1 Access Level"
```

### LVL.6 — Global Level Files

Two files shared across all levels:

**LEVLSPC2.DAT (924 bytes):**
- Loaded by `LoadLevelSpecialData` @ 0x0040DC70
- Read into buffer at 0x8825A5
- Contains per-level special configuration data

**OBJECTIV.DAT (768 bytes):**
- Loaded by `LoadObjectivesData` @ 0x0040DD70
- Read into buffer at 0x881713
- Contains objective definitions for all levels
- 768 / 48 levels ≈ 16 bytes per level (4 dwords: type + params)

### LVL.7 — AI Script Files (CPATR / CPSCR)

Per-player AI data files:

**CPATR files (AI patrol/command data):**
- Pattern: `LEVELS\CPATR{player_id}{level_num}.DAT`
- ~144 bytes each
- Contains AI patrol routes and command sequences
- "Doc 010" name visible at offset 0x30 in sample

**CPSCR files (AI scripts):**
- Pattern: `LEVELS\CPSCR{player_id}{level_num}.DAT`
- Loaded by `LoadAIScripts` @ 0x0040DE70
- Read into buffer starting at 0x948E52
- Each AI player gets 0x3108 bytes of script space
- Buffer pointer at `[base + 0x2000]` stored at `[base + 0x3108 - 8]`

**Player ID source:** Byte array at 0x883D32, indexed by loop counter. This maps abstract AI player indices to concrete player IDs.

### LVL.8 — Loading Pipeline

#### LoadLevelHeader @ 0x0040CC10

**Purpose:** Load and validate a level's header information.

**Flow:**
1. Build path `LEVELS\LEVL2{num}.DAT`, read into local buffer
2. If fails, try alternate path (0x4C41A0 fallback)
3. If both fail, return 0 (failure)
4. Build path with HDR extension (0x578168), call `LoadLevelObjectCount`
5. If object count < 10 (0x0A), return 0 (invalid level)
6. Call `GetLevelDisplayName` @ 0x4CF960 to extract display name
7. Extract from header buffer:
   - Tribe count: byte at buffer+0x50
   - AI availability: check bytes at buffer+0x51 (values >= 0x50)
   - Player count: byte at buffer+0x58
8. Return 1 (success)

#### LoadLevelObjectCount @ 0x0041D290

**Purpose:** Validate a level file by reading its first 0x44 bytes.

**Flow:**
1. Build file path from argument
2. Read 0x44 bytes from file
3. If read size ≠ 0x44: return 0 (invalid)
4. Return byte[0] of the read data (object count)
5. Caller checks if return value >= 0x0A (10)

#### LoadLevelData @ 0x0040CF80 (full version, renamed)

**Purpose:** Load terrain, player, and AI data from a level file.

**Flow:**
1. If level_num == 0x36 (54): skip (special case), return
2. Clear 0x268 bytes (0x9A dwords) at 0x883CD9
3. Build path `LEVELS\LEVL2{num}.DAT`, read into 0x883CD9
4. If fails, try alternate path; if both fail, call error handler (0x4C4AF0 with code 0x1A)
5. Clear 8 dwords at 0x883CED
6. For each AI player (count-1 iterations, from byte at 0x883D31):
   a. Clear 0x24 dwords (0x90 bytes) at player slot (0x883F41 + i*0x90)
   b. Build path `LEVELS\CPATR{player_id}{level_num}.DAT`
   c. Read into player AI buffer (0x883F41 + i*0x90)
   d. If fails, try alternate path; if both fail, show error
   e. Clear 0xC dwords (0x30 bytes) after the copied data
   f. Advance to next player slot (+0x90)
7. Call `LoadAIScripts` @ 0x0040DE70

#### LoadAIScripts @ 0x0040DE70

**Purpose:** Load CPSCR (AI script) files for each AI player.

**Flow:**
1. For each AI player (count-1, from 0x883D31):
   a. Build path `LEVELS\CPSCR{player_id}{level_num}.DAT`
   b. Read into 0x948E52 + i*0x3108
   c. If fails, try alternate path; if both fail, call init function 0x4CC3E0
   d. Set pointer: `[base + 0x3108 - 8] = base + 0x2000`

#### LoadLevelSpecialData @ 0x0040DC70

**Purpose:** Load the shared `LEVLSPC2.DAT` file.

**Flow:**
1. Build path `LEVELS\LEVLSPC2.DAT`
2. Read into 0x8825A5
3. If fails, try alternate path; if both fail, show error (code 0x1A)

#### LoadObjectivesData @ 0x0040DD70

**Purpose:** Load the shared `OBJECTIV.DAT` file.

**Flow:**
1. Build path `LEVELS\OBJECTIV.DAT`
2. Read into 0x881713
3. If fails, try alternate path; if both fail, show error (code 0x1A)

### LVL.9 — Loading State Machine (GameState_Loading @ 0x0041FAB0)

The loading process is managed by a state machine at byte 0x87759A.

**State 0 (Default — Main Loading):**
1. Call `GameState_Loading_MainTick` @ 0x0041FD60
2. Process zlib decompression (inflate buffer at 0x973C00)
3. Call post-load initialization (0x4B9950)

**State 1 (Palette/Texture Init):**
1. Load palette (0x492DF0)
2. Load textures via per-level index array at 0x8854B4 (3 dwords per entry)
3. Call initialization (0x41FC50) and sound init (0x418700)
4. Transition to state 3

**State 4/5 (Transition States):**
1. Call completion handler (0x4ABCD0)
2. Similar texture loading path
3. Final setup

### LVL.10 — GameState_Loading_MainTick @ 0x0041FD60

**Purpose:** Per-tick update during the loading state. Handles save game loading, palette setup, and timer.

**Flow:**
1. Check flag 0x6701E4 bit 3 (0x08): if set, load save game
   - Calculate save slot index from 0x664BA0/0x664BAC
   - Call `Level_StartLevel` @ 0x0040D210
   - Set game state 2 (0x4ABC80 with arg 2)
   - Clear bit 3
2. Check flag 0x6701E4 bit 1 (0x02): if set, load palette
   - Call palette functions (0x4C57F0, 0x4C5810, 0x450FC0)
   - Find transparent color index in palette (scan for RGB=0,0,0)
   - Store at 0x6701E0
   - Clear bit 1
3. Check flag 0x6701E5 bit 0: determines loading mode
   - If set: call 0x4E81B0 (special loading)
   - If not: call standard command buffer processing (0x5125D0 + 0x512920)
4. Read timer via function pointer at 0x97A710
5. Calculate delta time (capped at 1000ms = 0x3E8)
6. Convert to float, multiply by scale at 0x56D138
7. Store as float at 0x6701EC
8. Call additional init functions (0x419D30, 0x420D90, 0x40E5F0, 0x416A20, 0x417BB0)

### LVL.11 — LoadLevelTextures @ 0x00421320

**Purpose:** Load the terrain texture files for a level.

**Flow:**
1. Call `LoadLevelHeader` to get landscape type from HDR byte 0x60
2. Convert type byte to character:
   - 0-9 → '0'-'9' (add 0x30)
   - 10+ → 'a'-'z' (add 0x57)
3. Build and load 4 texture files:
   a. `data/plspl0-{c}.dat` → palette buffer at [0x867590] (0x400 = 1024 bytes)
   b. `data/plsft0-{c}.dat` → terrain texture buffer at 0x957078 (0x4000 = 16,384 bytes)
   c. `data/plscv0-{c}.dat` → cliff texture buffer at 0x802398 (0x6000 = 24,576 bytes)
   d. `data/plstx{NNN}.dat` → sky texture buffer at 0x7B9178 (0x40000 = 262,144 bytes)
      (NNN = level number, not type char)
4. Set flag: OR 0x6701E4 with 0x02 (triggers palette loading in MainTick)

**Texture buffer sizes:**
| Buffer | Address | Size | Format String |
|--------|---------|------|---------------|
| Palette (plspl0) | [0x867590] | 0x400 (1KB) | `data/plspl0-%c.dat` |
| Terrain (plsft0) | 0x957078 | 0x4000 (16KB) | `data/plsft0-%c.dat` |
| Cliff (plscv0) | 0x802398 | 0x6000 (24KB) | `data/plscv0-%c.dat` |
| Sky (plstx) | 0x7B9178 | 0x40000 (256KB) | `data/plstx%03d.dat` |

### LVL.12 — Level_StartLevel @ 0x0040D210

**Purpose:** Initialize a level for gameplay, loading all data and setting up game state.

**Flow:**
1. Clear flag bit 0x08000000 at 0x884BFD
2. Check flag at 0x885714 bit 0: if set, set game state 7 and return
3. Call initialization functions (0x41F4A0, 0x4066B0)
4. Get level number from argument; compare with current level (0x8828AD)
   - If same: use index 0x63 (99 = "current level" marker)
   - Otherwise: search player array at 0x8825F1 (stride 0x18, up to 0x8828C1)
5. Store level at 0x884C75 (word) and 0x883CD7 (byte)
6. Check campaign mode flag (0x884BFE bit 1):
   - If campaign: call `Campaign_AdvanceToNextLevel` @ 0x421500
   - Otherwise: call `SaveGame_LoadStateFromBuffer` @ 0x0040D780
7. Handle special level transitions (switch on level - 0xF, 6 cases)
   - Cases set flags at 0x883CD9 (bits 0x2000 or 0x10000)
8. Call terrain setup (0x4517A0), camera setup (0x4528E0, 0x452F40)
9. Set camera mode via 0x48B210

### LVL.13 — Campaign_AdvanceToNextLevel @ 0x00421500

**Purpose:** Advance through the campaign level sequence.

This is a large function (~0x4B0 bytes) with complex level progression logic.

**Key features:**
- Uses level availability check function at 0x41A7C0
- Searches campaign level table at 0x664BAC (entries of 0x34 bytes, level ID at +0x28)
- Campaign count at 0x664BA8
- Current campaign index at 0x664B9C
- Level ranges: 0x01-0x19 (1-25) for standard levels
- Special transitions: level 0x18 (credits) and 0x19 (endgame)
- Handles sequential and branching level selection
- Sets "completed" flag (bit 0x04) on level entries at 0x8828FD + level*0xA4

### LVL.14 — SaveGame_LoadStateFromBuffer @ 0x0040D780

**Purpose:** Populate the save game buffer from current level data.

**Flow:**
1. Clear save game header (12 bytes at 0x948C96)
2. Clear player data A (0x38 dwords = 0xE0 bytes at 0x948CA2)
3. Clear player data B (0x30 dwords = 0xC0 bytes at 0x948D82)
4. Clear additional fields at 0x948E42 (16 bytes)
5. If level index == 0x63 (current level): skip data copy, return
6. Copy 12 bytes from level entry (0x8825E5 + index*0x18) to 0x948C96
7. Copy 0xE dwords (0x38 bytes) from 0x882941 to 0x948CA2
8. Clear additional fields at 0x948CAA/0x948CAE and 8 dwords at 0x948CB6
9. For each player (count-1 from 0x883D31):
   a. Copy 0xE dwords (0x38 bytes) from 0x883F91 + i*0x90 to 0x948CDA + i*0x38
10. For each player (count-1):
   a. Copy 0xC dwords (0x30 bytes) from 0x883F41 + i*0x90 to 0x948DB2 + i*0x30
11. Copy dword from 0x883D35 to 0x948E4E

### LVL.15 — Key Global Addresses

| Address | Name | Description |
|---------|------|-------------|
| 0x883CD9 | g_level_data | Main level data buffer (reads from .dat file) |
| 0x883CED | g_level_sub_buffer | Sub-buffer within level data |
| 0x883D31 | g_player_count | Number of players (byte) |
| 0x883D32 | g_player_id_array | Player ID mapping array |
| 0x883F41 | g_player_ai_patrol | AI patrol data per player (stride 0x90) |
| 0x883F91 | g_player_data | Player configuration data (stride 0x90) |
| 0x884141 | g_player_data_end | End of player data block |
| 0x882941 | g_shared_player_template | Shared player template (0x38 bytes) |
| 0x8825A5 | g_levlspc2_buffer | LEVLSPC2.DAT destination |
| 0x8825E5 | g_level_entry_table | Level entry table (0x18 bytes per entry) |
| 0x8825F1 | g_level_player_ids | Player IDs within level entries |
| 0x8828AD | g_current_level_id | Current level ID (byte) |
| 0x8828FD | g_level_completion_flags | Per-level completion flags |
| 0x881713 | g_objectives_buffer | OBJECTIV.DAT destination |
| 0x8852BB | g_file_path_buffer | File path construction buffer |
| 0x8854B4 | g_per_level_texture_array | Texture index per level (3 dwords/entry) |
| 0x8856F6 | g_loading_level_num | Level number being loaded (byte) |
| 0x87759A | g_loading_state | Loading state machine byte |
| 0x867590 | g_palette_buffer_ptr | Pointer to palette buffer (for plspl0) |
| 0x948C96 | g_savegame_header | Save game header (12 bytes) |
| 0x948CA2 | g_savegame_player_data | Save game player data |
| 0x948E52 | g_ai_script_buffer | AI script (CPSCR) buffer base |
| 0x957078 | g_terrain_texture_buf | Terrain foot textures (plsft0) |
| 0x802398 | g_cliff_texture_buf | Cliff textures (plscv0) |
| 0x7B9178 | g_sky_texture_buf | Sky textures (plstx) |
| 0x6701E4 | g_loading_flags | Loading state flags (dword) |
| 0x664BA0 | g_campaign_current_slot | Current campaign slot index |
| 0x664BA8 | g_campaign_level_count | Number of campaign levels |
| 0x664BAC | g_campaign_level_table_ptr | Pointer to campaign level table |
| 0x664B9C | g_campaign_selected_index | Selected campaign level index |

### LVL.16 — Function Reference

| Address | Name | Size | Description |
|---------|------|------|-------------|
| 0x0040CC10 | LoadLevelHeader | ~0x1C8 | Load + validate level header |
| 0x0040CF80 | LoadLevelData | ~0x286 | Load terrain, AI patrols, AI scripts |
| 0x0040D210 | Level_StartLevel | ~0x1F3 | Full level initialization |
| 0x0040D780 | SaveGame_LoadStateFromBuffer | ~0x11F | Populate save state from level |
| 0x0040DC70 | LoadLevelSpecialData | ~0xFB | Load LEVLSPC2.DAT |
| 0x0040DD70 | LoadObjectivesData | ~0xFB | Load OBJECTIV.DAT |
| 0x0040DE70 | LoadAIScripts | ~0x14D | Load CPSCR files per player |
| 0x0041D290 | LoadLevelObjectCount | ~0x4D | Validate level file (read 0x44 bytes) |
| 0x0041FAB0 | GameState_Loading | ~var | Loading state machine |
| 0x0041FD60 | GameState_Loading_MainTick | ~0x13B | Per-tick loading update |
| 0x00421320 | LoadLevelTextures | ~0x19E | Load 4 texture files per level |
| 0x00421500 | Campaign_AdvanceToNextLevel | ~0x4B0 | Campaign level progression |
| 0x004CF960 | GetLevelDisplayName | - | Extract level name string |
| 0x004CC3E0 | AI_InitPlayerScript | - | Initialize default AI script |
| 0x004C41A0 | File_GetAlternatePath | - | Fallback file path (CD-ROM?) |
| 0x005119B0 | File_ReadFromDisk | - | Read file into buffer |
| 0x004C4310 | File_BuildFullPath | - | Construct full file path |
| 0x004C55D0 | File_SetReadMode | - | Set file access mode |
| 0x004BA1B0 | File_Prepare | - | Prepare for file operation |
| 0x004C4AF0 | Error_ShowDialog | - | Display error dialog |

### LVL.17 — Loading Flag Bits (0x6701E4)

| Bit | Mask | Description |
|-----|------|-------------|
| 1 | 0x02 | Palette needs loading (set by LoadLevelTextures) |
| 3 | 0x08 | Save game needs loading |

### LVL.18 — Faithful Project Correlation

The faithful open-source project (`/Users/adriencandiotti/Pop/faithful/`) independently confirms:

1. **DAT format**: Heightmap at offset 0, 128×128 u16 values, followed by 3×0x4000 layers, tribes, sunlight, units
2. **HDR byte 96 (0x60)**: Landscape type → texture set character mapping
3. **Unit struct**: 55 bytes with subtype, model, tribe, position, angle fields
4. **Tribe config**: 16 bytes per tribe, 4 tribes
5. **Model types**: Enum 1-11 matching GetObjectTypeName exactly
6. Source: `faithful/src/pop/level.rs`, `faithful/src/pop/units.rs`

---


### Multiplayer System (MLDPlay)

MLDPlay is Bullfrog's DirectPlay wrapper library.

| Address    | Name                  | Description                              |
|------------|----------------------|------------------------------------------|
| 0x004e5ad0 | Network_WriteSyncLog  | Writes to sync.log for desync debugging  |
| 0x004e57a0 | Network_OpenSyncLog   | Opens sync.log file                      |

**MLDPlay API (imported from MLDPLAY.DLL):**
- `MLDPlay::GetCurrentMs()` - Gets current network time
- Session management for up to 4 players
- Deterministic lockstep synchronization
- Checksum validation for desync detection

**Network Files:**
- `sync.log` - Desync debugging log
- Contains player GT (Game Time) and sync info


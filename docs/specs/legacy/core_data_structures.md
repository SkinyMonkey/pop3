# Core Data Structures and Enums

## Phase 1: Core Data Structures and Enums

### ModelType Enum (param_1 in GetObjectTypeName @ 0x00454050)

| Value | Type Name   | Description          |
|-------|-------------|----------------------|
| 1     | PERSON      | Human units          |
| 2     | BUILDING    | Structures           |
| 3     | CREATURE    | Animals              |
| 4     | VEHICLE     | Boats/Airships       |
| 5     | SCENERY     | Trees, rocks, etc.   |
| 6     | GENERAL     | Lights, triggers     |
| 7     | EFFECT      | Visual effects       |
| 8     | SHOT        | Projectiles          |
| 9     | SHAPE       | Shapes               |
| 10    | INTERNAL    | Formations, beacons  |
| 11    | SPELL       | Spell objects        |

### Person Subtypes (ModelType = 1)

| Value | Name           |
|-------|----------------|
| 1     | Wild           |
| 2     | Brave          |
| 3     | Warrior        |
| 4     | Religious      |
| 5     | Spy            |
| 6     | Super Warrior  |
| 7     | Medicine Man (Shaman) |
| 8     | Angel of Death |

### Building Subtypes (ModelType = 2)

| Value | Name                  |
|-------|-----------------------|
| 1     | Tepee                 |
| 2     | Tepee Stage 2         |
| 3     | Tepee Stage 3         |
| 4     | Drum Tower            |
| 5     | Temple                |
| 6     | Spy Train             |
| 7     | Warrior Train         |
| 8     | Super W Train         |
| 9     | Reconversion Centre   |
| 10    | Wall                  |
| 11    | Gate                  |
| 12    | Ignore                |
| 13    | BoatHut 1             |
| 14    | BoatHut 2             |
| 15    | AirHut 1              |
| 16    | AirHut 2              |
| 17    | Guard Post            |
| 18    | Library               |
| 19    | Prison                |

### Creature Subtypes (ModelType = 3)

| Value | Name    |
|-------|---------|
| 1     | Bear    |
| 2     | Buffalo |
| 3     | Wolf    |
| 4     | Eagle   |
| 5     | Rabbit  |
| 6     | Beaver  |
| 7     | Fish    |

### Vehicle Subtypes (ModelType = 4)

| Value | Name      |
|-------|-----------|
| 1     | Boat 1    |
| 2     | Boat 2    |
| 3     | Airship 1 |
| 4     | Airship 2 |

### Spell Subtypes (ModelType = 11)

| Value | Name           |
|-------|----------------|
| 1     | Burn           |
| 2     | Blast          |
| 3     | Lightning Bolt |
| 4     | Whirlwind      |
| 5     | Insect Plague  |
| 6     | Invisibility   |
| 7     | Hypnotism      |
| 8     | Firestorm      |
| 9     | Ghost Army     |
| 10    | Erosion        |
| 11    | Swamp          |
| 12    | Land Bridge    |
| 13    | Angel of Death |
| 14    | Earthquake     |
| 15    | Flatten Land   |
| 16    | Volcano        |
| 17    | Convert Wild   |
| 18    | Armageddon     |
| 19    | Shield         |
| 20    | Blood Lust     |
| 21    | Teleport       |

### Scenery Subtypes (ModelType = 5)

| Value | Name                  |
|-------|-----------------------|
| 1     | Mass Tree             |
| 2     | Special Tree 1        |
| 3     | Special Tree 2        |
| 4     | Mass Fruit Tree       |
| 5     | Special Fruit Tree 1  |
| 6     | Special Fruit Tree 2  |
| 7     | Plant 1               |
| 8     | Plant 2               |
| 9     | Head (Stone Head)     |
| 10    | Fire                  |
| 11    | Wood Pile             |
| 12    | Reincarnation Pillar  |
| 13    | Rock                  |
| 14    | Portal                |
| 15    | Island                |
| 16    | Bridge                |
| 18    | Top Level Scenery     |
| 19    | Sub Level Scenery     |

### General Subtypes (ModelType = 6)

| Value | Name              |
|-------|-------------------|
| 1     | Light             |
| 2     | Discovery         |
| 3     | Debug Static      |
| 4     | Debug Flying      |
| 5     | Debug Flag        |
| 6     | Trigger           |
| 10    | Discovery Marker  |

### Effect Subtypes (ModelType = 7)

| Value | Name                       |
|-------|----------------------------|
| 1     | Simple Blast               |
| 2     | Sprite Circles             |
| 3     | Smoke                      |
| 4     | Lightning Element          |
| 5     | Burn Cell Obstacles        |
| 6     | Flatten Land               |
| 7     | Move RS Pillar             |
| 8     | Prepare RS Land            |
| 9     | Sphere Explode 1           |
| 10    | Fireball                   |
| 11    | Firecloud                  |
| 12    | Ghost Army                 |
| 13    | Invisibility               |
| 14    | Explode Building Partial   |
| 15    | Volcano                    |
| 16    | Hypnotism                  |
| 17    | Lightning Bolt             |
| 18    | Swamp                      |
| 19    | Angel of Death             |
| 20    | Whirlwind                  |
| 21    | Insect Plague              |
| 22    | Firestorm                  |
| 23    | Erosion                    |
| 24    | Land Bridge                |
| 25    | Wrath of God               |
| 26    | Earthquake                 |
| 27    | Fly Thingummy              |
| 28    | Sphere Explode and Fire    |
| 29    | Big Fire                   |
| 30    | Lightning                  |
| 31    | Flatten                    |
| 32    | General                    |
| 33    | Shape Sparkle              |
| 34    | Lava Flow                  |
| 35    | Volcano Explosions         |
| 36    | Purify Land                |
| 37    | Unpurify Land              |
| 38    | Explosion 1                |
| 39    | Explosion 2                |
| 40    | Lava Square                |
| 41    | Whirlwind Element          |
| 42    | Lightning Strand           |
| 43    | Whirlwind Dust             |
| 44    | Raise Land                 |
| 45    | Lower Land                 |
| 46    | Hill                       |
| 47    | Valley                     |
| 48    | Place Tree                 |
| 49    | Rise                       |
| 51    | Rein Rock Debris           |
| 52    | Clear Mapwho               |
| 53    | Place Shaman               |
| 54    | Place Wild Man             |
| 55    | Building Smoke             |
| 56    | Much Simpler Blast         |
| 57    | Tumbling Branch            |
| 58    | Conversion Flash           |
| 59    | Hypnosis Flash             |
| 60    | Sparkle                    |
| 61    | Small Sparkle              |
| 62    | Explosion 3                |
| 63    | Rock Explosion             |
| 64    | Lava Gloop                 |
| 65    | Splash                     |
| 66    | Smoke Cloud                |
| 67    | Smoke Cloud Constant       |
| 68    | Fireball 2                 |
| 69    | Ground Shockwave           |
| 70    | Orbiter                    |
| 71    | Big Sparkle                |
| 72    | Meteor                     |
| 73    | Convert Wild               |
| 74    | Building Smoke Full        |
| 75    | Building Smoke Part        |
| 76    | Building Damaged Smoke     |
| 77    | Delete RS Pillars          |
| 78    | Spell Blast                |
| 79    | Firestorm Smoke            |
| 80    | Player Dead                |
| 81    | Reveal Fog Area            |
| 82    | Shield                     |
| 83    | Boat Hut Repair            |
| 84    | Swamp Reeds                |
| 85    | Swamp Mist                 |
| 87    | Bloodlust                  |
| 88    | Teleport                   |
| 89    | Atlantis Set               |
| 90    | Atlantis Invoke            |
| 91    | Statue To AOD              |
| 92    | Fill Spell One Shots       |
| 93    | Fire Roll Element          |

### Shot Subtypes (ModelType = 8)

| Value | Name                          |
|-------|-------------------------------|
| 1     | Standard Shot                 |
| 2     | Standard Shot 2               |
| 4     | Fireball Shot                 |
| 7     | Volcano Small Fireball Shot   |
| 8     | Volcano Big Fireball Shot     |

### Internal Subtypes (ModelType = 10)

| Value | Name              |
|-------|-------------------|
| 1     | Formation         |
| 2     | Beacon            |
| 4     | Soul Convert      |
| 5     | MM Attract        |
| 6     | Formation         |
| 7     | Obj Face          |
| 8     | Fight             |
| 9     | Pre Fight         |
| 10    | Guard Control     |
| 12    | Soul Convert 2    |
| 13    | DT Beacon         |
| 16    | Guard Post Display|
| 18    | Wood Distrib      |
| 19    | Sinking Building  |

---

## Object Instance Structure

- **Size**: 0xB3 (179) bytes per object
- **Object Array Base**: 0x008c89c0
- **Pointer Array**: 0x00878928 (g_ObjectPtrArray)

### Known Field Offsets

| Offset | Size | Field Name      | Notes                              |
|--------|------|-----------------|-------------------------------------|
| 0x0C   | 1    | flags           | Bit 0 = inactive/deleted           |
| 0x14   | 4    | state_flags     | Bit 23 (0x800000) = special state  |
| 0x24   | 2    | object_index    | Index/handle                       |
| 0x26   | 2    | unknown         |                                    |
| 0x2A   | 1    | is_alive        | Non-zero if alive                  |
| 0x2B   | 1    | subtype         | Object subtype                     |
| 0x2C   | 1    | state           | Current state                      |
| 0x2E   | 1    | flags2          | Additional flags                   |
| 0x3D   | 2    | pos_x           | X position                         |
| 0x3F   | 2    | pos_y           | Y position                         |
| 0x41   | 2    | pos_z           | Z/height position                  |
| 0x6E   | 2    | next_ptr_idx    | Index for linked list traversal    |
| 0x7E   | 4    | action_flags    | Bit flags for current action       |
| 0x82   | 2    | counter1        |                                    |
| 0x84   | 2    | timer           | Action timer                       |
| 0x86   | 2    | target_idx      | Target object index                |
| 0x8A   | 2    | resource1       | HP or mana related                 |
| 0x8C   | 2    | resource2       | HP or mana related                 |
| 0x93   | 2    | saved_z         | Saved Z position                   |

---

## Key Global Data

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x00878928 | g_ObjectPtrArray  | Array of object pointers       |
| 0x008c89c0 | g_ObjectArray     | Actual object instance storage |
| 0x008852bb | g_FileBuffer      | File loading buffer            |
| 0x00883cd9 | g_LevelData       | Level configuration data       |
| 0x005a063f | g_UnitTypeData    | Unit type properties (30-byte stride) |

---

## Constants System

### constant.dat Parser (0x0041eb50)

The game's balance parameters are stored in `LEVELS/constant.dat` and parsed at startup.

**File Format:**
- Text file with key=value pairs
- Keys prefixed with `P3CONST_` (stripped during parsing)
- Optional XOR encryption (magic bytes 0x40 0x7E)
- Comments start with `#`

**Constants Table:** 0x005a3300
- Each entry: 31 bytes (0x1F)
  - 0x00-0x18: Parameter name (25 bytes)
  - 0x19: Data size (1, 2, or 4 bytes)
  - 0x1A: Flags (bit 2 = loaded, bit 1 = percent, bit 0 = special)
  - 0x1B-0x1E: Pointer to destination variable

### Resource Constants (Trees/Wood)

**Tree Types (6):**
| Constant | Description |
|----------|-------------|
| TREE1_WOOD_VALUE | Wood yield for tree type 1 |
| TREE1_WOOD_GROW | Growth rate for tree type 1 |
| TREE1_DORMANT_TIME | Dormancy period for tree type 1 |
| (TREE2-6 same pattern) | |

**Wood Costs:**
- WOOD_BRAVE, WOOD_WARR, WOOD_PREACH, etc. - Unit wood costs
- WOOD_HUT_1, WOOD_HUT_2, WOOD_HUT_3 - Housing costs
- WOOD_DRUM_TOWER, WOOD_TEMPLE, etc. - Building costs
- WOOD_VEHICLE_BOAT1, WOOD_VEHICLE_AIRSHIP_1 - Vehicle costs

---

## Key Global Variables

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x008856f9 | g_GameSpeed       | Ticks per second               |
| 0x0088571c | g_GameTick        | Current tick counter           |
| 0x0059ac70 | g_TickIntervalMs  | Milliseconds per tick          |
| 0x0059ac6c | g_LastTickTime    | Last tick GetTickCount value   |
| 0x00884bf9 | g_GameFlags       | Game state flags               |
| 0x00884c88 | g_PlayerTribe     | Local player's tribe index     |
| 0x00884be9 | g_ActiveObjectCount | Current active objects       |
| 0x00884bf1 | g_LowPriorityCount | Low priority object count    |
| 0x00885720 | g_TickCounter     | Total ticks this session       |
| 0x0087e344 | g_AIUpdateMult    | AI update multiplier           |

### Game Flags (g_GameFlags at 0x00884bf9)

| Bit | Mask     | Description |
|-----|----------|-------------|
| 1   | 0x02     | Paused |
| 3   | 0x08     | Multiplayer mode |
| 5   | 0x20     | Network waiting |
| 6   | 0x40     | Network paused |
| 23  | 0x800000 | Victory/Defeat |
| 25  | 0x2000000 | Player won |
| 26  | 0x4000000 | Player lost |

---

## Appendix A: Key Memory Addresses

### Global Data Structures

| Address    | Name                | Size    | Description                    |
|------------|---------------------|---------|--------------------------------|
| 0x00878928 | g_ObjectPtrArray    | 0x1134  | Object pointer array           |
| 0x008c89c0 | g_ObjectArray       | ~200KB  | Object instance storage        |
| 0x00885760 | g_TribeArray        | 0x3194  | 4 tribe structures (0xC65 each)|
| 0x00888980 | g_Heightmap         | 0x8000  | 128x128 terrain heights        |
| 0x0088897c | g_CellFlags         | 0x10000 | 128x128 cell flags             |
| 0x005a3300 | g_ConstantsTable    | ~8KB    | Parsed constant.dat values     |
| 0x0059fe44 | g_UnitTypeData      | 0x258   | Unit type properties           |
| 0x005a0720 | g_VehicleTypeData   | 0x5C    | Vehicle type properties        |
| 0x005a07a0 | g_SceneryTypeData   | ~0x300  | Scenery type properties        |
| 0x0059fb30 | g_PersonAnimationTable| 0x48  | Animation lookup table         |

### Game State

| Address    | Name                | Description                    |
|------------|---------------------|--------------------------------|
| 0x00877598 | g_GameState         | Current game state (0x02=menu) |
| 0x00884bf9 | g_GameFlags         | Game state flags               |
| 0x00884c88 | g_PlayerTribe       | Local player tribe index       |
| 0x008856f9 | g_GameSpeed         | Ticks per second               |
| 0x0088571c | g_GameTick          | Current tick counter           |
| 0x00885710 | g_RandomSeed        | LCG random seed                |
| 0x00884c90 | g_CurrentLanguage   | Active language (0-10)         |

### Object Lists

| Address    | Name                | Description                    |
|------------|---------------------|--------------------------------|
| 0x008788f0 | g_BuildingListHead  | Building linked list           |
| 0x008788f4 | g_FightingListHead  | Fighting objects list          |
| 0x008788f8 | g_MiscListHead      | Effects/shots/internal list    |
| 0x008788b4 | g_FreeListHighPri   | Free objects (high priority)   |
| 0x008788b8 | g_FreeListLowPri    | Free objects (low priority)    |
| 0x008788c0 | g_DestroyedList     | Objects pending cleanup        |
| 0x008788c8 | g_SpellListHead     | Active spell objects           |

### Rendering

| Address    | Name                | Description                    |
|------------|---------------------|--------------------------------|
| 0x00973ae4 | g_PrimarySurface    | DDraw primary surface          |
| 0x00973c4c | g_BackBuffer        | DDraw back buffer              |
| 0x006868a8 | g_CameraTarget      | Tracked object pointer         |
| 0x00952408 | g_CameraX           | Camera X position              |
| 0x0095240c | g_CameraY           | Camera Y position              |

### Math Tables

| Address    | Name                | Description                    |
|------------|---------------------|--------------------------------|
| 0x005641b4 | g_AtanLookupTable   | Atan2 lookup (256 entries)     |
| 0x005ac6a0 | g_CosTable          | Cosine lookup (2048 entries)   |
| 0x005acea0 | g_SinTable          | Sine lookup (2048 entries)     |

---

## Appendix C: Important Constants

### Object Limits
- Max active objects: 1101 (0x44D)
- Low priority pool: 640 (0x280)
- Max tribes: 4
- Max players: 4

### Terrain
- Map size: 128x128 cells
- Height values: 16-bit per cell
- Water level: Variable per level

### Angles
- Full rotation: 0x800 (2048 steps)
- Per quadrant: 0x200 (512 steps)
- East: 0x000, North: 0x200, West: 0x400, South: 0x600

### Timing
- Default game speed: Variable ticks/second
- Tick interval: 1000 / g_GameSpeed milliseconds

---


### Key Global Data (Phase 6-10)

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x00877598 | g_GameState       | Current game state             |
| 0x00885760 | g_TribeArray      | Array of tribe data structures |
| 0x005a7f80 | g_MLDPlaySession  | MLDPlay session handle         |
| 0x0087dc42 | g_IsMultiplayer   | Multiplayer game flag          |
| 0x0087d291 | g_NetworkState    | Network connection state       |


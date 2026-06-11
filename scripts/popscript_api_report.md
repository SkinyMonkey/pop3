# PopScript Lua API Analysis Report

Generated: 2026-04-09

## Summary

| Category | Count |
|----------|-------|
| Our implemented functions | 25 |
| Our stub functions | 146 |
| **Our total functions** | **171** |
| Documented items (all) | 4,604 |
| Documented Lua API (filtered) | 4,191 |
| **Functions in both** | **68** |

## Key Findings

### 1. Implemented Functions (Documented)

These 13 functions are **implemented and documented**:
- `ATTACK`
- `ATTACK_MARKER`
- `CONVERT_AT_MARKER`
- `DELETE_SMOKE_STUFF`
- `PRAY_AT_HEAD`
- `SEND_PEOPLE_TO_MARKER`
- `SET_ATTACK_VARIABLE`
- `SET_BASE_RADIUS`
- `SET_BUCKET_USAGE`
- `SET_MARKER_ENTRY`
- `SET_REINCARNATION`
- `SET_SPELL_ENTRY`
- `TRAIN_PEOPLE_NOW`

### 2. Stub Functions (Documented)

These stub functions are **documented and should be implemented**:

#### Core Game Functions (Priority: High)
- `GAME_TURN` - Get current game turn
- `G_RANDOM` - Random number generator
- `EVERY_2POW_TURNS` - Check if turn is power of 2
- `HAS_TIMER_REACHED_ZERO` - Timer check
- `REMOVE_TIMER` - Remove a timer
- `OPEN_DIALOG` - Open dialog box
- `FLASH_BUTTON` - Flash UI button

#### Tribe Status Functions (Priority: High)
- `MY_MANA` - Get player mana (implemented for some tribes)
- `MY_NUM_PEOPLE` - Get player population
- `MY_NUM_BRAVES` - Get brave count
- `MY_NUM_WARRIORS` - Get warrior count
- `MY_NUM_SPIES` - Get spy count
- `MY_NUM_PREACHERS` - Get preacher count
- `MY_NUM_SUPER_WARRIORS` - Get super warrior count
- `MY_SHAMAN_LIVES` - Get shaman lives
- `MY_REINCARNATION_TIMER` - Get reincarnation timer

#### Building/Unit Counts
- `MY_NUM_SMALL_HUT` - Small hut count
- `MY_NUM_MEDIUM_HUT` - Medium hut count
- `MY_NUM_LARGE_HUT` - Large hut count
- `MY_NUM_DRUM_TOWER` - Drum tower count
- `MY_NUM_TEMPLE` - Temple count
- `MY_NUM_BOATS` - Boat count
- `MY_NUM_AIRSHIPS` - Airship count
- `MY_NUM_VEHICLES` - Vehicle count
- `MY_WOOD_COUNT` - Wood resource count

#### Combat Functions
- `MY_ATTACK_ARMY_COUNT` - Attack army size
- `MY_DEFEND_ARMY_COUNT` - Defense army size
- `MY_NUM_KILLED_BY_BLUE` - Kills by blue tribe
- `MY_NUM_KILLED_BY_RED` - Kills by red tribe
- `MY_NUM_KILLED_BY_GREEN` - Kills by green tribe
- `MY_NUM_KILLED_BY_YELLOW` - Kills by yellow tribe

#### Targeting Functions
- `TARGET_SHAMAN` - Target enemy shaman
- `TARGET_MEDICINE_MAN` - Target medicine man
- `TARGET_WARRIORS` - Target warriors
- `TARGET_S_WARRIOR` - Target super warriors
- `TARGET_FIREWARRIORS` - Target fire warriors
- `TARGET_*_DRUM_TOWERS` - Target specific tribe drum towers
- `TARGET_*_SHAMAN` - Target specific tribe shaman
- `TARGET_*_SUPER_WARRIORS` - Target specific tribe super warriors

#### Send/Move Functions
- `SEND_ALL_PEOPLE_TO_MARKER` - Send all to marker
- `SEND_PEOPLE_TO_MARKER` - Send people to marker
- `SEND_*_PEOPLE_TO_MARKER` - Send specific tribe people
- `SEND_SHAMAN_DEFENDERS_HOME` - Return shaman defenders
- `ONLY_STAND_AT_MARKERS` - Stand at markers only

#### Spell Functions
- `SPELL_AT_MARKER` - Cast spell at marker
- `SPELL_AT_THING` - Cast spell at thing
- `MY_SPELL_BURN_COST` - Burn spell cost
- `MY_SPELL_LIGHTNING_COST` - Lightning spell cost
- `MY_SPELL_BLAST_COST` - Spell blast cost

#### Building/Construction
- `BUILD_AT` - Build at location
- `SET_BUILDING_DIRECTION` - Set building direction
- `PARTIAL_BUILDING_COUNT` - Partial building count
- `MAX_BUILDING_TYPE` - Max building type

#### Camera/View
- `CAMERA_ROTATION` - Camera rotation
- `GET_HEIGHT_AT_POS` - Get terrain height
- `FLYBY_*` - Cinematic flyby functions (16 functions)

#### Message System
- `CREATE_MSG_*` - Create message functions (4 variants)
- `SET_MSG_*` - Set message properties (12+ functions)

#### Special Actions
- `DO_TRIGGER` - Trigger an event
- `TRIGGER_THING` - Trigger a thing
- `TRIGGER_LEVEL_WON` - Level won trigger
- `TRIGGER_LEVEL_LOST` - Level lost trigger
- `I_KILL_CONVERTABLE` - Kill convertable check
- `IS_SHAMAN_ALIVE` - Check shaman alive
- `IS_SHAMAN_AVAILABLE` - Check shaman available
- `IS_PRISONER_LEFT` - Check prisoner remaining
- `IS_BUILDING_NEAR` - Check building proximity

#### Tribe-Specific Constants (Stubs)
- `BLUE_*`, `RED_*`, `GREEN_*`, `YELLOW_*` - Tribe-specific counts
- `SET_NO_BLUE`, `SET_NO_RED`, etc. - Disable tribe options

### 3. Extra Functions (Not in Official Docs)

103 functions in our implementation that aren't in the official documentation:
- Most tribe-specific constants (`BLUE_BRAVES`, `RED_MANA`, etc.)
- These were discovered through reverse engineering

### 4. Constants Already Added

All 59 missing constants from the previous work are now in `constants.rs`:
- Attack types: `ATTACK_NORMAL`, `ATTACK_BY_BOAT`, `ATTACK_BY_BALLOON`
- Attribute flags: `ATTR_AWAY_MEDICINE_MAN`, `ATTR_PREF_*_DRIVERS`, etc.
- Object flags: `ABF_END_LIST`, `AOF_END_LIST`, `AMBIENT_FLAG_*`
- Effect system: `AFFECT_*`, `AOD2_FLAG_*`
- Add-on types: `ADD_ON_TYPE_*`
- Terrain: `AAM_FLATTEN`, `AAM_RAISE_LOWER`
- And more...

## Recommended Implementation Priority

### Priority 1: Core Game State (5 functions)
1. `GAME_TURN` - Essential for turn-based logic
2. `G_RANDOM` - Random number generation
3. `MY_MANA` - Player resource
4. `MY_NUM_PEOPLE` - Population count
5. `HAS_TIMER_REACHED_ZERO` - Timer system

### Priority 2: Combat & Targeting (10 functions)
1. `ATTACK` variants (already have stubs)
2. `TARGET_*` functions (12 variants)
3. `MY_ATTACK_ARMY_COUNT` / `MY_DEFEND_ARMY_COUNT`
4. `MY_NUM_KILLED_BY_*` (4 variants)

### Priority 3: Building & Resources (8 functions)
1. `BUILD_AT`
2. `MY_WOOD_COUNT`
3. `MY_NUM_*` building counts (8 variants)
4. `PARTIAL_BUILDING_COUNT`

### Priority 4: Movement & Commands (8 functions)
1. `SEND_*_TO_MARKER` (6 variants)
2. `ONLY_STAND_AT_MARKERS`
3. `SET_BUILDING_DIRECTION`

### Priority 5: Spells & Effects (6 functions)
1. `SPELL_AT_MARKER` / `SPELL_AT_THING`
2. `MY_SPELL_*_COST` (3 variants)
3. `DELETE_SMOKE_STUFF`

### Priority 6: UI & Messages (lower priority)
1. `OPEN_DIALOG`
2. `CREATE_MSG_*` variants
3. `FLASH_BUTTON`

### Priority 7: Cinematic/Flyby (lowest priority)
- 16 `FLYBY_*` functions for cinematics

## Files to Update

- `src/engine/ai/popscript.rs` - Implement stub functions
- `docs/specs/ai_scripting.md` - Document implemented functions
- `docs/specs/re_meta.md` - Track renamed functions

## Conclusion

Our implementation has **171 functions** (25 implemented + 146 stubs). The official documentation contains ~4,191 Lua API items, but many are:
- Constants (already added 59)
- Internal C++ helpers (not Lua-exposed)
- Redundant tribe-specific variants

**Actual missing Lua API functions worth implementing: ~50-60**

The remaining stubs are mostly:
- Tribe-specific variants that can be consolidated
- Constants already added
- Internal helpers not meant for Lua

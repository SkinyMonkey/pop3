# AI Scripting System

## Appendix P: AI Scripting System

The AI uses a bytecode scripting system for controlling computer player behavior.

### Script Execution Flow

```
AI_UpdateTribe (0x0041a8b0)
    └─→ AI_RunScript (0x004c5eb0)
        └─→ AI_ProcessScriptBlock (0x004c6180)
            └─→ AI_ExecuteScriptCommand (0x004c6460)
                └─→ AI_EvaluateScriptValue (0x004c8b50)
```

### Script Opcode Structure

Scripts use 16-bit opcodes starting at base 0x404:

| Opcode Range | Base | Description |
|--------------|------|-------------|
| 0x404-0x417 | +0-23 | Attribute enable/disable commands |
| 0x40e (case 0xa) | | Call FUN_004cc430 |
| 0x423 (case 0x1f) | | Call FUN_004c9450 |
| 0x428 (case 0x24) | | Call FUN_004cc340 |
| 0x429 (case 0x25) | | Call FUN_004b7130 |
| 0x42a (case 0x26) | | Set value via FUN_004b7140 |
| ... | | (many more opcodes) |

### Script Value Types

AI_EvaluateScriptValue handles 3 value types:
- Type 0: Immediate literal value
- Type 1: Variable reference (from script variable array)
- Type 2: Internal attribute/state lookup

### Internal Attribute Codes (Type 2)

| Code Range | Description |
|------------|-------------|
| 1000-1047 (0x3E8-0x417) | Per-tribe attribute flags |
| 1048 (0x418) | Tribe field at offset 0x94d |
| 1049-1064 (0x419-0x428) | Indexed data lookups |
| 1065 (0x429) | DAT_005a1310 value |
| 1066-1081 (0x42a-0x439) | Tribe-relative values |
| 1082-1113 | Various tribe statistics |
| 1184-1189 (0x4A0-0x4A5) | Spell type constants (1-6) |
| 1190-1198 (0x4A6-0x4AE) | Spell type constants (7-16) |
| 1201-1206 (0x4B1-0x4B6) | Unit type constants (2-7) |
| 1207-1214 (0x4B7-0x4BE) | Building type constants (1-8) |
| 1216-1221 (0x4C0-0x4C5) | Additional constants |
| 1226 (0x4CA) | Call FUN_004b41a0 |
| 1227 (0x4CB) | Tribe field at offset 0xbab |
| 1237 (0x4D5) | Random 0-99 (uses g_RandomSeed) |

### Random Number Generation (in AI scripts)

```c
// Used for AI decisions
g_RandomSeed = g_RandomSeed * 0x24a1 + 0x24df;
g_RandomSeed = (g_RandomSeed >> 13) | (g_RandomSeed << 19);
return g_RandomSeed % 100;  // Returns 0-99
```

---

## Appendix Q: Game Tick System

### Main Simulation Loop

`Game_SimulationTick` (0x004bb5a0) is the main game loop that processes one simulation frame.

### Tick Interval

```c
g_TickIntervalMs = 1000 / g_GameSpeed;  // Milliseconds per tick
```

### Single Player Tick Order

1. `Tick_ProcessNetworkMessages` - Process any network packets
2. `Tick_ProcessPendingActions` - Execute queued player commands
3. `Tick_UpdateGameTime` - Increment game clock
4. `Tick_UpdateTerrain` - Process terrain changes
5. `Tick_UpdateObjects` - Update all game objects
6. `Tick_UpdateWater` - Update water simulation
7. (Loop for DAT_0087e344 + 1 iterations):
   - `Tick_UpdateSinglePlayer` or `Tick_UpdateTutorial`
   - `AI_UpdateAllTribes` - Update computer players
   - `Tick_UpdatePopulation` - Spawn new population
   - `Tick_UpdateMana` - Generate mana for tribes

### Multiplayer Tick Order

Same as single player but with additional network synchronization:
- Sends tick packets (opcode 7, 6, 0xd, 0xe)
- Waits for all players to acknowledge ticks
- Uses lockstep synchronization

### Tick Functions

| Function | Address | Description |
|----------|---------|-------------|
| Game_SimulationTick | 0x004bb5a0 | Main simulation loop |
| Tick_ProcessNetworkMessages | 0x004a76b0 | Handle network packets |
| Tick_ProcessPendingActions | 0x004a6f60 | Execute player commands |
| Tick_UpdateGameTime | 0x004a7ac0 | Increment game time |
| Tick_UpdateTerrain | 0x0048bda0 | Terrain changes |
| Tick_UpdateObjects | 0x004a7550 | All object updates |
| Tick_UpdateWater | 0x0048bf10 | Water simulation |
| Tick_UpdateSinglePlayer | 0x00456500 | Single player logic |
| Tick_UpdateTutorial | 0x00469320 | Tutorial mode logic |
| Tick_UpdatePopulation | 0x004198f0 | Population spawning |
| Tick_UpdateMana | 0x004aeac0 | Mana generation & object states |

### Tick_UpdateMana Processing

This function does much more than mana - it's the main object update dispatcher:

1. Increment DAT_00885720 (game tick counter)
2. Call Game_CheckVictoryConditions
3. Process movement for all tribe objects
4. Process building fighting persons
5. Update all Person objects (state machines)
6. Update Building, Effect, Shot, Internal objects
7. Update Scenery objects
8. Various cleanup and synchronization

---

## Appendix BO: AI Decision Trees

### AI Update Flow

```
Main Game Loop (every frame)
  └─ AI_UpdateAllTribes @ 0x0041a7d0
     └─ for each tribe 0-3:
        └─ AI_UpdateTribe @ 0x0041a8b0
           ├─ Check shaman timers (+0x5bd)
           ├─ AI_ValidateTargets @ 0x004b3f30
           ├─ AI_UpdateUnitCooldowns @ 0x004b3f10
           ├─ AI_RunScript @ 0x004c5eb0
           │  ├─ AI_EvaluateCondition @ 0x004c8860
           │  ├─ AI_ExecuteScriptCommand @ 0x004c6460
           │  └─ AI_ProcessLoopCommand @ 0x004c8700
           ├─ AI_CalculateThreatDistance @ 0x0041b000
           ├─ AI_UpdateShamanStatus @ 0x0041b1b0
           ├─ AI_ValidateBuildingPlacements @ 0x0041b280
           ├─ AI_ProcessShamanCommands @ 0x0041b6d0
           ├─ AI_ExecuteBuildingPriorities @ 0x0041b8d0
           └─ AI_EvaluateSpellCasting @ 0x004b8a90
```

### Shaman Command Types

**AI_ProcessShamanCommands** @ 0x0041b6d0 dispatches:

| Type | Handler | Purpose |
|------|---------|---------|
| 0x00 | AI_Cmd_PrimaryAttack | Main attack |
| 0x01 | AI_Cmd_SecondaryAttack | Flanking attack |
| 0x02 | AI_Cmd_DefendPosition | Hold position |
| 0x03 | AI_Cmd_SpellCasting | Cast spells |
| 0x04 | AI_Cmd_ArmyMovement | Move units |
| 0x05 | AI_Cmd_BuildingPlacement | Construct |
| 0x06 | AI_Cmd_ResourceGathering | Gather wood |
| 0x07 | AI_Cmd_Conversion | Convert wilds |

### Shaman Command Structure (0x52 bytes each)

```c
struct ShamanCommand {
    uint8_t  flags;        // +0x00: Active/pending/complete
    uint8_t  type;         // +0x11: Command type (0x00-0x1c)
    uint16_t target_x;     // +0x14: Target X
    uint16_t target_y;     // +0x16: Target Y
    uint16_t target_id;    // +0x18: Target entity
    uint16_t state;        // +0x4e: Execution progress
};
// 10 command slots at tribe offset +0x74
```

### Target Selection Algorithm

**AI_FindBestAttackTarget** @ 0x004b9770

```c
int score = 0;

// Person scoring
if (type == PERSON) {
    switch (subtype) {
        case SHAMAN:       score = 1000; break;
        case SUPER_WARRIOR: score = 20;  break;
        case WARRIOR:       score = 10;  break;
        case PREACHER:      score = 15;  break;
        case SPY:           score = 5;   break;
    }
    // Exposed shaman bonus
    if (subtype == SHAMAN && nearbyDefenders < 3)
        score += 500;
}

// Building scoring
if (type == BUILDING) {
    switch (subtype) {
        case SUPER_WARRIOR_TRAINING: score = 250; break;
        case TEMPLE:                 score = 200; break;
        case WARRIOR_TRAINING:       score = 180; break;
        case DRUM_TOWER:             score = 150; break;
        default:                     score = 50;  break;
    }
}

// Distance penalty
score -= distance_from_base / 100;
```

### Threat Assessment

**AI_AssessThreat** @ 0x0041ba40
**AI_CountEnemyUnits** @ 0x004b51c0

Unit threat weights:
| Unit Type | Weight |
|-----------|--------|
| Shaman | 50 |
| Super Warrior | 20 |
| Preacher | 15 |
| Warrior | 10 |
| Spy | 5 |

### Difficulty Scaling

**Mana Adjustment:**
```c
// COMPUTER_MANA_ADJUST @ 0x005aa8b9
adjustedMana = baseMana * (COMPUTER_MANA_ADJUST / 100);
// Easy: < 100, Normal: 100, Hard: > 100
```

**AI Training Costs (separate from human):**
| Parameter | Address |
|-----------|---------|
| CP_TRAIN_MANA_WARR | 0x005acc34 |
| CP_TRAIN_MANA_SPY | 0x005acc53 |
| CP_TRAIN_MANA_PREACH | 0x005acc72 |
| CP_TRAIN_MANA_SWARR | 0x005acc91 |

### Training Cost Bands

Diminishing returns as unit count increases:

| Band | Units | Cost Multiplier |
|------|-------|-----------------|
| BAND_00_03 | 0-3 | 100% |
| BAND_04_07 | 4-7 | ~120% |
| BAND_08_11 | 8-11 | ~140% |
| BAND_12_15 | 12-15 | ~160% |
| BAND_16_20 | 16-20 | ~180% |
| BAND_21+ | 21+ | ~200% |

### Building Priority Order

1. **Drum Towers** (defense) - highest priority
2. **Training Buildings** (military)
3. **Housing/Huts** (population)
4. **Reincarnation Sites** (shaman respawn)

### Shaman Safety Logic

**AI_CheckShamanSafety** @ 0x0041bae0
- Checks if shaman surrounded or under attack
- Triggers **AI_ShamanRetreat** @ 0x0041bf90 if threatened
- Validates shaman has valid targets

### Spell Casting Priority

**AI_EvaluateSpellCasting** @ 0x004b8a90 (every ~32 frames)

```
if mana >= SPELL_THRESHOLD:
    if threat_high:
        cast offensive_spell
    if hp_low:
        cast heal
    if allies_threatened:
        cast defensive_spell
    if enemy_shaman_exposed:
        cast direct_damage
```

High-priority spells:
- Lightning (0x6d) - direct damage
- Swarm (0x6e) - crowd control
- Heal (0x71) - self-preservation
- Firestorm (0x73) - siege
- Armageddon (0x79) - endgame

### AI State Variables (per tribe)

| Offset | Purpose |
|--------|---------|
| +0x5B4 | Decision countdown timer |
| +0x5B5 | Active shaman command slot |
| +0x5B7 | Current spell selection |
| +0x5BE | Selected spell type |
| +0x5A4-0x5A6 | Target coordinates |
| +0x596 | Status flags (0x40000=paused, 0x10=spell active) |
| +0x5A8 | Target entity ID |

### Personality Traits (per tribe)

Located at tribe offset +0x137 (16 values per shaman):
- Aggression level (unit type ratios)
- Expansion vs defense (building priority)
- Risk tolerance (attack force threshold)
- Tech path (spell vs unit focus)

### Key AI Functions

| Function | Address | Purpose |
|----------|---------|---------|
| AI_UpdateAllTribes | 0x0041a7d0 | Main entry point |
| AI_UpdateTribe | 0x0041a8b0 | Per-tribe update |
| AI_RunScript | 0x004c5eb0 | Bytecode interpreter |
| AI_ExecuteScriptCommand | 0x004c6460 | Command dispatcher |
| AI_ProcessShamanCommands | 0x0041b6d0 | Command queue |
| AI_ExecuteBuildingPriorities | 0x0041b8d0 | Building logic |
| AI_AssessThreat | 0x0041ba40 | Threat calculation |
| AI_CheckShamanSafety | 0x0041bae0 | Shaman protection |
| AI_ShamanRetreat | 0x0041bf90 | Retreat logic |
| AI_FindBestAttackTarget | 0x004b9770 | Target scoring |
| AI_CountEnemyUnits | 0x004b51c0 | Unit counting |
| AI_EvaluateSpellCasting | 0x004b8a90 | Spell decisions |
| AI_ValidateTargets | 0x004b3f30 | Target validation |
| AI_ValidatePlacement | 0x004b5990 | Placement checks |

---


### AI Scripting System

The AI uses a bytecode scripting system with a virtual machine interpreter.

| Address    | Name                     | Description                              |
|------------|--------------------------|------------------------------------------|
| 0x004c6460 | AI_ExecuteScriptCommand  | Main bytecode interpreter (200+ opcodes) |
| 0x004c8b50 | AI_EvaluateScriptValue   | Evaluates script variables/constants     |
| 0x004c6180 | AI_ProcessScriptBlock    | Handles IF/ELSE/ENDIF flow control       |
| 0x004c8930 | AI_EvaluateComparison    | Evaluates comparison operators           |
| 0x004c8860 | AI_EvaluateCondition     | Evaluates boolean conditions             |
| 0x004c8700 | AI_ProcessLoopCommand    | Handles EVERY/DO loop constructs         |
| 0x004c8590 | AI_ProcessSubroutineCall | Handles script subroutine calls          |
| 0x004c5eb0 | AI_RunScript             | Top-level script execution               |
| 0x0041a8b0 | AI_UpdateTribe           | Updates single tribe's AI                |
| 0x0041a7d0 | AI_UpdateAllTribes       | Updates all AI tribes each tick          |

**Script Bytecode Tokens:**
- 0x3E8 (1000) - IF statement
- 0x3E9 (1001) - ELSE
- 0x3EB (1003) - BEGIN block
- 0x3EC (1004) - END block
- 0x3ED (1005) - Variable assignment
- 0x3EE (1006) - Command execution
- 0x3EF-0x3F1 (1007-1009) - Loop constructs
- 0x3F4-0x3F9 (1012-1017) - Comparison operators
- 0x3FC (1020) - AND
- 0x3FD (1021) - OR

**Tribe Data Structure:** Base at 0x00885760, stride 0xC65 per tribe.
- Offset 0x3100: Script state pointer
- Offset 0x3104: Script instruction pointer

### AI Script Constants (constant.dat)

Human vs Computer differences:
- `HUMAN_MANA_ADJUST` / `COMPUTER_MANA_ADJUST` - Mana rate
- `HUMAN_TRAIN_MANA_*` / `CP_TRAIN_MANA_*` - Training costs

---

## Appendix RP: PopScript Lua API

This appendix documents the PopScript Lua API exposed to AI scripts. All functions are implemented in `src/engine/ai/popscript.rs` and bridge to the game engine via `AiGameBridge`.

### Implementation Status

| Category | Implemented | Total |
|----------|-------------|-------|
| Core game state | 12 | 12 |
| Tribe status queries | 24 | 24 |
| Building/Resource counts | 16 | 16 |
| Combat & targeting | 18 | 18 |
| Movement & commands | 14 | 14 |
| Spell system | 8 | 8 |
| Marker system | 8 | 8 |
| Camera & view | 4 | 4 |
| Flyby/cinematic | 11 | 11 |
| Message/UI system | 15 | 15 |
| Timer & trigger | 8 | 8 |
| Constants | 59 | 59 |
| **Total** | **183** | **183** |

### Core Game State Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `GAME_TURN` | `() -> i32` | Returns current game tick count |
| `G_RANDOM` | `() -> i32` | Returns random number (0-99) |
| `EVERY_2POW_TURNS` | `(power: i32) -> i32` | Check if turn is power of 2 |
| `HAS_TIMER_REACHED_ZERO` | `(timer_id: i32) -> i32` | Check if timer expired |
| `REMOVE_TIMER` | `(timer_id: i32) -> i32` | Remove a timer |
| `MY_MANA` | `() -> i32` | Get current tribe's mana |
| `MY_NUM_PEOPLE` | `() -> i32` | Get total population |
| `MY_NUM_BRAVES` | `() -> i32` | Get brave count |
| `MY_NUM_KILLED` | `() -> i32` | Get total kills |
| `MY_NUM_CONVERTED` | `() -> i32` | Get converted units |
| `MY_NUM_LOST` | `() -> i32` | Get lost units |
| `MY_TRIBE` | `() -> i32` | Get current tribe ID (0-3) |

### Tribe Status Queries

| Function | Signature | Description |
|----------|-----------|-------------|
| `MY_NUM_WARRIORS` | `() -> i32` | Get warrior count |
| `MY_NUM_SPIES` | `() -> i32` | Get spy count |
| `MY_NUM_PREACHERS` | `() -> i32` | Get preacher count |
| `MY_NUM_SUPER_WARRIORS` | `() -> i32` | Get super warrior count |
| `MY_SHAMAN_LIVES` | `() -> i32` | Get shaman lives remaining |
| `MY_REINCARNATION_TIMER` | `() -> i32` | Get reincarnation timer |
| `MY_ATTACK_ARMY_COUNT` | `() -> i32` | Get attacking army size |
| `MY_DEFEND_ARMY_COUNT` | `() -> i32` | Get defending army size |

#### Kill Counts by Tribe

| Function | Signature | Description |
|----------|-----------|-------------|
| `MY_NUM_KILLED_BY_BLUE` | `() -> i32` | Kills by blue tribe |
| `MY_NUM_KILLED_BY_RED` | `() -> i32` | Kills by red tribe |
| `MY_NUM_KILLED_BY_YELLOW` | `() -> i32` | Kills by yellow tribe |
| `MY_NUM_KILLED_BY_GREEN` | `() -> i32` | Kills by green tribe |

#### Tribe-Specific Queries (BLUE_*, RED_*, YELLOW_*, GREEN_*)

Each tribe has 5 query functions (20 total):

| Function | Signature | Description |
|----------|-----------|-------------|
| `BLUE_BRAVES` | `() -> i32` | Blue tribe braves |
| `BLUE_WARRIORS` | `() -> i32` | Blue tribe warriors |
| `BLUE_PREACHERS` | `() -> i32` | Blue tribe preachers |
| `BLUE_SPIES` | `() -> i32` | Blue tribe spies |
| `BLUE_SUPER_WARRIORS` | `() -> i32` | Blue tribe super warriors |

(Same pattern for `RED_*`, `YELLOW_*`, `GREEN_*`)

### Building & Resource Counts

| Function | Signature | Description |
|----------|-----------|-------------|
| `MY_WOOD_COUNT` | `() -> i32` | Get wood resources |
| `MY_NUM_SMALL_HUT` | `() -> i32` | Small hut count |
| `MY_NUM_MEDIUM_HUT` | `() -> i32` | Medium hut count |
| `MY_NUM_LARGE_HUT` | `() -> i32` | Large hut count |
| `MY_NUM_DRUM_TOWER` | `() -> i32` | Drum tower count |
| `MY_NUM_TEMPLE` | `() -> i32` | Temple count |
| `MY_NUM_BOATS` | `() -> i32` | Boat count |
| `MY_NUM_AIRSHIPS` | `() -> i32` | Airship count |
| `MY_NUM_VEHICLES` | `() -> i32` | Vehicle count |
| `MY_NUM_SPY_TRAIN` | `() -> i32` | Spy training huts |
| `MY_NUM_WARRIOR_TRAIN` | `() -> i32` | Warrior training huts |
| `MY_NUM_SUPER_TRAIN` | `() -> i32` | Super warrior training |
| `PARTIAL_BUILDING_COUNT` | `(type: i32) -> i32` | Count partially built |
| `MAX_BUILDING_TYPE` | `() -> i32` | Max building type ID |

### Combat & Targeting

| Function | Signature | Description |
|----------|-----------|-------------|
| `ATTACK` | `(type: i32) -> i32` | Initiate attack |
| `ATTACK_MARKER` | `() -> i32` | Attack at marker |
| `ATTACK_WITH_OPTION` | `(option: i32) -> i32` | Attack with option |
| `CONVERT_AT_MARKER` | `() -> i32` | Convert at marker |
| `TARGET_SHAMAN` | `() -> i32` | Target enemy shaman |
| `TARGET_MEDICINE_MAN` | `() -> i32` | Target medicine man |
| `TARGET_WARRIORS` | `() -> i32` | Target warriors |
| `TARGET_S_WARRIOR` | `() -> i32` | Target super warriors |
| `TARGET_FIREWARRIORS` | `() -> i32` | Target fire warriors |
| `TARGET_BLUE_DRUM_TOWERS` | `() -> i32` | Target blue drum towers |
| `TARGET_RED_DRUM_TOWERS` | `() -> i32` | Target red drum towers |
| `TARGET_YELLOW_DRUM_TOWERS` | `() -> i32` | Target yellow drum towers |
| `TARGET_GREEN_DRUM_TOWERS` | `() -> i32` | Target green drum towers |
| `TARGET_BLUE_SHAMAN` | `() -> i32` | Target blue shaman |
| `TARGET_RED_SHAMAN` | `() -> i32` | Target red shaman |
| `TARGET_YELLOW_SHAMAN` | `() -> i32` | Target yellow shaman |
| `TARGET_GREEN_SHAMAN` | `() -> i32` | Target green shaman |
| `I_KILL_CONVERTABLE` | `() -> i32` | Check killable convert |

### Movement & Commands

| Function | Signature | Description |
|----------|-----------|-------------|
| `SEND_ALL_PEOPLE_TO_MARKER` | `() -> i32` | Send all to marker |
| `SEND_PEOPLE_TO_MARKER` | `() -> i32` | Send people to marker |
| `SEND_BLUE_PEOPLE_TO_MARKER` | `() -> i32` | Send blue tribe |
| `SEND_RED_PEOPLE_TO_MARKER` | `() -> i32` | Send red tribe |
| `SEND_YELLOW_PEOPLE_TO_MARKER` | `() -> i32` | Send yellow tribe |
| `SEND_GREEN_PEOPLE_TO_MARKER` | `() -> i32` | Send green tribe |
| `SEND_SHAMAN_DEFENDERS_HOME` | `() -> i32` | Return defenders |
| `ONLY_STAND_AT_MARKERS` | `() -> i32` | Stand at markers only |
| `SET_ATTACK_VARIABLE` | `(value: i32) -> i32` | Set attack var |
| `SET_MARKER_ENTRY` | `(entry: i32) -> i32` | Set marker entry |
| `SET_SPELL_ENTRY` | `(entry: i32) -> i32` | Set spell entry |
| `TRAIN_PEOPLE_NOW` | `() -> i32` | Instant training |
| `SET_BUILDING_DIRECTION` | `(dir: i32) -> i32` | Set building direction |
| `SET_BASE_MARKER` | `() -> i32` | Set base marker |
| `RESET_BASE_MARKER` | `() -> i32` | Reset base marker |
| `SET_DRUM_TOWER_POS` | `(x: i32, y: i32) -> i32` | Set drum tower pos |

### Spell System

| Function | Signature | Description |
|----------|-----------|-------------|
| `SPELL_AT_MARKER` | `(spell: i32) -> i32` | Cast at marker |
| `SPELL_AT_THING` | `(spell: i32) -> i32` | Cast at thing |
| `MY_SPELL_BURN_COST` | `() -> i32` | Burn spell cost |
| `MY_SPELL_LIGHTNING_COST` | `() -> i32` | Lightning cost |
| `MY_SPELL_BLAST_COST` | `() -> i32` | Blast spell cost |
| `DELETE_SMOKE_STUFF` | `() -> i32` | Clear smoke effects |
| `PRAY_AT_HEAD` | `() -> i32` | Pray at head |
| `SET_BASE_RADIUS` | `(radius: i32) -> i32` | Set base radius |

### Camera & View

| Function | Signature | Description |
|----------|-----------|-------------|
| `GET_HEIGHT_AT_POS` | `(x: i32, y: i32) -> i32` | Get terrain height |
| `CAMERA_ROTATION` | `() -> i32` | Get camera rotation |
| `SET_BUCKET_USAGE` | `(usage: i32) -> i32` | Set bucket usage |

### Flyby/Cinematic Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `FLYBY_CREATE_NEW` | `() -> i32` | Create new flyby |
| `FLYBY_SET_EVENT_POS` | `(x: i32, y: i32, z: i32) -> i32` | Set event position |
| `FLYBY_SET_EVENT_ANGLE` | `(angle: i32) -> i32` | Set event angle |
| `FLYBY_SET_EVENT_ZOOM` | `(zoom: i32) -> i32` | Set event zoom |
| `FLYBY_SET_EVENT_INT_POINT` | `(x: i32, y: i32, z: i32) -> i32` | Set interest point |
| `FLYBY_SET_EVENT_TOOLTIP` | `(text: i32) -> i32` | Set tooltip text |
| `FLYBY_SET_END_TARGET` | `(target: i32) -> i32` | Set end target |
| `FLYBY_START` | `() -> i32` | Start flyby |
| `FLYBY_STOP` | `() -> i32` | Stop flyby |
| `FLYBY_ALLOW_INTERRUPT` | `(allow: i32) -> i32` | Allow interrupt |
| `FLYBY_DISALLOW_INTERRUPT` | `() -> i32` | Disallow interrupt |

### Message/UI System

| Function | Signature | Description |
|----------|-----------|-------------|
| `CREATE_MSG_NARRATIVE` | `(text: i32) -> i32` | Create narrative msg |
| `CREATE_MSG_OBJECTIVE` | `(text: i32) -> i32` | Create objective msg |
| `CREATE_MSG_INFORMATION` | `(text: i32) -> i32) -> i32` | Create info msg |
| `OPEN_DIALOG` | `(id: i32) -> i32` | Open dialog box |
| `SET_MSG_AUTO_OPEN_DLG` | `(auto: i32) -> i32` | Auto-open dialog |
| `SET_MSG_DELETE_ON_OK` | `(del: i32) -> i32` | Delete on OK |
| `SET_MSG_ID` | `(id: i32) -> i32` | Set message ID |
| `SET_MSG_NARRATIVE` | `(narr: i32) -> i32` | Set narrative text |
| `SET_MSG_OK_SAVE` | `(save: i32) -> i32` | OK saves game |
| `SET_MSG_TIMEOUT` | `(timeout: i32) -> i32` | Set timeout |
| `FLASH_BUTTON` | `(btn: i32) -> i32` | Flash UI button |

### Timer & Trigger System

| Function | Signature | Description |
|----------|-----------|-------------|
| `DO_TRIGGER` | `(trigger: i32) -> i32` | Trigger event |
| `TRIGGER_THING` | `(thing: i32) -> i32` | Trigger thing |
| `TRIGGER_LEVEL_WON` | `() -> i32` | Level won trigger |
| `TRIGGER_LEVEL_LOST` | `() -> i32` | Level lost trigger |
| `IS_SHAMAN_ALIVE` | `() -> i32` | Check shaman alive |
| `IS_SHAMAN_AVAILABLE` | `() -> i32` | Check shaman available |
| `IS_PRISONER_LEFT` | `() -> i32` | Check prisoner left |
| `IS_BUILDING_NEAR` | `() -> i32` | Check building near |

### Constants

59 constants are defined in `src/engine/ai/constants.rs`:

#### Attack Types
- `ATTACK_NORMAL`, `ATTACK_BY_BOAT`, `ATTACK_BY_BALLOON`

#### Attribute Flags
- `ATTR_AWAY_MEDICINE_MAN`, `ATTR_PREF_BLUE_DRIVERS`, `ATTR_PREF_RED_DRIVERS`, etc.

#### Object Flags
- `ABF_END_LIST`, `AOF_END_LIST`, `AMBIENT_FLAG_*`

#### Effect System
- `AFFECT_*`, `AOD2_FLAG_*`

#### Add-on Types
- `ADD_ON_TYPE_*`

#### Terrain
- `AAM_FLATTEN`, `AAM_RAISE_LOWER`

### Bridge Data Structure

The `AiGameBridge` struct (`src/engine/ai/mod.rs`) maintains all game state accessible to Lua:

```rust
pub struct AiGameBridge {
    // Current tribe context
    pub current_tribe: u32,

    // Per-tribe unit counts
    pub tribe_braves: [u32; 4],
    pub tribe_warriors: [u32; 4],
    pub tribe_preachers: [u32; 4],
    pub tribe_spies: [u32; 4],
    pub tribe_super_warriors: [u32; 4],

    // Per-tribe kill counts
    pub tribe_killed_by_blue: [u32; 4],
    pub tribe_killed_by_red: [u32; 4],
    pub tribe_killed_by_yellow: [u32; 4],
    pub tribe_killed_by_green: [u32; 4],

    // Per-tribe building counts
    pub tribe_small_huts: [u32; 4],
    pub tribe_medium_huts: [u32; 4],
    pub tribe_large_huts: [u32; 4],
    pub tribe_drum_towers: [u32; 4],
    pub tribe_temples: [u32; 4],
    pub tribe_spy_trains: [u32; 4],
    pub tribe_warrior_trains: [u32; 4],
    pub tribe_super_trains: [u32; 4],
    pub tribe_boats: [u32; 4],
    pub tribe_airships: [u32; 4],
    pub tribe_vehicles: [u32; 4],
    pub tribe_wood_count: [u32; 4],

    // Per-tribe army counts
    pub tribe_attack_army: [u32; 4],
    pub tribe_defend_army: [u32; 4],

    // Per-tribe spell costs
    pub tribe_spell_burn_cost: [u32; 4],
    pub tribe_spell_blast_cost: [u32; 4],
    pub tribe_spell_lightning_cost: [u32; 4],

    // Per-tribe reincarnation timer
    pub tribe_reincarnation_timer: [u32; 4],

    // Global game state
    pub game_turn: u32,
    pub random_seed: u32,
    // ... additional fields
}
```

### Testing

All PopScript functions are tested in `src/engine/ai/popscript.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn tribe_specific_unit_counts() {
        let (lua, bridge) = setup();
        // Set up test data
        let mut b = bridge.borrow_mut();
        b.tribe_braves = [10, 20, 30, 40];

        // Test Blue tribe (index 0)
        assert_eq!(lua.load("return BLUE_BRAVES()").eval::<i32>().unwrap(), 10);

        // Test Red tribe (index 1)
        assert_eq!(lua.load("return RED_BRAVES()").eval::<i32>().unwrap(), 20);

        // ... additional assertions
    }
}
```

Run tests with:
```bash
cargo test popscript
```

All 130 PopScript tests pass as of 2026-04-08.


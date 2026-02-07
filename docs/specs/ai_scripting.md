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


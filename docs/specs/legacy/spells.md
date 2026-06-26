# Spell System

## Phase 5: Spell System

### Spell Constants (constant.dat)

All 21 spells have configurable parameters loaded from constant.dat:
- `SPELL_<name>` - Mana cost
- `SP_W_RANGE_<name>` - Working range
- `SP_1_OFF_MAX_<name>` - Maximum effect value
- `SPELL_<name>_OPT_S` - Optional spell settings

### Altitude Bands

Spells have power modifiers based on altitude (8 bands):
- `ALT_BAND_0_SPELL_INCR` through `ALT_BAND_7_SPELL_INCR`
- Higher altitude = more spell power (up to 33% bonus)

### Spell Effect Functions

| Address    | Name                     | Description                              |
|------------|--------------------------|------------------------------------------|
| 0x004f3a50 | Spell_ProcessBlast       | Blast spell - area damage with knockback |
| 0x004f2950 | Spell_ProcessShockwave   | Whirlwind - expanding damage ring        |
| 0x004f7330 | Spell_ProcessLightningSwarm | Lightning - targets buildings, hits units |
| 0x004f2550 | Spell_ProcessBurn        | Burn - single cell fire damage           |
| 0x004958b0 | Spell_SetState           | Spell object state handler (stub)        |

### Spell Targeting

| Address    | Name                     | Description                              |
|------------|--------------------------|------------------------------------------|
| 0x004a5a30 | Spell_FindTargetScenery  | Finds valid scenery targets for spells   |
| 0x004a5b60 | Spell_CheckTargetValid   | Validates spell target location          |

### Shield Spell System

| Address    | Name                     | Description                              |
|------------|--------------------------|------------------------------------------|
| 0x0049a230 | Shield_EjectPerson       | Ejects person from shield when hit       |
| 0x0049a9f0 | Shield_FindExitPosition  | Finds valid position for shield ejection |
| 0x0049b4e0 | Person_CheckShieldOnDeath| Checks if shield can save dying unit     |

Shield parameters:
- `SHIELD_COUNT_X8` - Shield duration multiplier
- `SHIELD_NUM_PEOPLE` - Max people protected by one shield

### Wild Person Conversion

| Address    | Name                     | Description                              |
|------------|--------------------------|------------------------------------------|
| 0x00502e60 | Wild_ConvertToBrave      | Converts wild person to tribe brave      |

### Building Damage

| Address    | Name                     | Description                              |
|------------|--------------------------|------------------------------------------|
| 0x00434570 | Building_ApplyDamage     | Applies damage to building (offset 0x9E) |

### Physics

| Address    | Name                     | Description                              |
|------------|--------------------------|------------------------------------------|
| 0x004d4db0 | Object_UpdateFlyingPhysics | Updates flying object movement/physics |

### Spell Subtypes Reference

| Value | Spell Name      | Category   |
|-------|-----------------|------------|
| 1     | Burn            | Offensive  |
| 2     | Blast           | Offensive  |
| 3     | Lightning Bolt  | Offensive  |
| 4     | Whirlwind       | Offensive  |
| 5     | Insect Plague   | Offensive  |
| 6     | Invisibility    | Buff       |
| 7     | Hypnotism       | Buff       |
| 8     | Firestorm       | Offensive  |
| 9     | Ghost Army      | Utility    |
| 10    | Erosion         | Terrain    |
| 11    | Swamp           | Terrain    |
| 12    | Land Bridge     | Terrain    |
| 13    | Angel of Death  | Offensive  |
| 14    | Earthquake      | Terrain    |
| 15    | Flatten Land    | Terrain    |
| 16    | Volcano         | Terrain    |
| 17    | Convert Wild    | Utility    |
| 18    | Armageddon      | Utility    |
| 19    | Shield          | Buff       |
| 20    | Blood Lust      | Buff       |
| 21    | Teleport        | Utility    |

### Mana System Constants

From constant.dat:
- `MAX_MANA` - Maximum mana pool
- `START_MANA` - Initial mana at game start
- `CONVERT_MANA` - Mana for converting wild people
- `HUMAN_MANA_ADJUST` / `COMPUTER_MANA_ADJUST` - Mana rate modifiers
- `SHAMEN_DEAD_MANA_%_LOST` / `SHAMEN_DEAD_MANA_%_GAIN` - Mana on shaman death
- `MANA_F_<unit>` - Mana generation per unit type
- `MANA_IDLE_BRAVES` / `MANA_BUSY_BRAVES` - Mana for idle vs working braves
- `TRAIN_MANA_BAND_XX_YY` - Training cost bands by population

### Renamed Functions (Phase 5)

| Address    | Original        | New Name                      |
|------------|-----------------|-------------------------------|
| 0x004f3a50 | FUN_004f3a50    | Spell_ProcessBlast            |
| 0x004f2950 | FUN_004f2950    | Spell_ProcessShockwave        |
| 0x004f7330 | FUN_004f7330    | Spell_ProcessLightningSwarm   |
| 0x004f2550 | FUN_004f2550    | Spell_ProcessBurn             |
| 0x004a5a30 | FUN_004a5a30    | Spell_FindTargetScenery       |
| 0x004a5b60 | FUN_004a5b60    | Spell_CheckTargetValid        |
| 0x0049a230 | FUN_0049a230    | Shield_EjectPerson            |
| 0x0049a9f0 | FUN_0049a9f0    | Shield_FindExitPosition       |
| 0x00502e60 | FUN_00502e60    | Wild_ConvertToBrave           |
| 0x00434570 | FUN_00434570    | Building_ApplyDamage          |
| 0x004d4db0 | FUN_004d4db0    | Object_UpdateFlyingPhysics    |

---

## Additional Spell Effects

| Address    | Name                     | Description                    |
|------------|--------------------------|--------------------------------|
| 0x004f6480 | Spell_CreateSwarmEffect  | Creates insect swarm (16 objects) |
| 0x004f3ee0 | Spell_CreateFirestorm    | Creates fireballs for firestorm |

### Renamed Functions (Person States)

| Address    | Original        | New Name                          |
|------------|-----------------|-----------------------------------|
| 0x00500b00 | FUN_00500b00    | Person_EnterMovingState           |
| 0x00437b40 | FUN_00437b40    | Person_EnterFightingState         |
| 0x00501750 | FUN_00501750    | Person_EnterBuildingState         |
| 0x005021c0 | FUN_005021c0    | Person_EnterGatheringState        |
| 0x00501c00 | FUN_00501c00    | Person_EnterTrainingState         |
| 0x00501e20 | FUN_00501e20    | Person_EnterHousingState          |
| 0x00503190 | FUN_00503190    | Person_EnterDrowningState         |
| 0x00502f70 | FUN_00502f70    | Person_StartWoodGathering         |
| 0x00503e50 | FUN_00503e50    | Person_EnterPreachingState        |
| 0x00504410 | FUN_00504410    | Person_EnterBeingConvertedState   |
| 0x0050a960 | FUN_0050a960    | Person_EnterVehicleState          |
| 0x0050b480 | FUN_0050b480    | Person_ExitVehicleState           |
| 0x0050b990 | FUN_0050b990    | Person_EnterCelebrationState      |
| 0x0050d620 | FUN_0050d620    | Person_EnterTeleportState         |
| 0x00509e90 | FUN_00509e90    | Preacher_StartConverting          |
| 0x004f6480 | FUN_004f6480    | Spell_CreateSwarmEffect           |
| 0x004f3ee0 | FUN_004f3ee0    | Spell_CreateFirestorm             |

---

## Spell System

### Spell Processing Functions

| Address    | Name                       | Description                    |
|------------|----------------------------|--------------------------------|
| 0x004f2550 | Spell_ProcessBurn          | Burn spell damage + knockback  |
| 0x004f2950 | Spell_ProcessShockwave     | Shockwave spell effect         |
| 0x004f3a50 | Spell_ProcessBlast         | Blast spell (conversion + dmg) |
| 0x004f3ee0 | Spell_CreateFirestorm      | Create firestorm spell         |
| 0x004f6480 | Spell_CreateSwarmEffect    | Create swarm visual effect     |
| 0x004f7330 | Spell_ProcessLightningSwarm | Lightning swarm processing    |
| 0x004958b0 | Spell_SetState             | Change spell object state      |
| 0x004a5b60 | Spell_CheckTargetValid     | Validate spell target          |
| 0x004a5a30 | Spell_FindTargetScenery    | Find scenery target for spell  |

### Spell Damage Constants

| Address    | Name                | Value | Description                    |
|------------|---------------------|-------|--------------------------------|
| 0x005a3220 | SPELL_BURN_DAMAGE   | -     | Damage per tick from Burn      |
| 0x005a32bc | BLOODLUST_MULT      | -     | Bloodlust damage multiplier    |
| 0x005a32c0 | SHIELD_SHIFT        | -     | Shield damage reduction (shift)|
| 0x005a32c8 | BURN_MAX_TARGETS    | -     | Max targets for Burn spell     |

### Blast Spell Mechanics

The Blast spell (0x004f3a50) has two modes based on flags at offset 0x76:
- Bit 0: Convert wild followers to braves
- Bit 1: Damage enemy units (excluding wild)

Creates 32 visual ring particles rotating at 0x40 angle increments.
Expands radius by 0xA0 per tick, max radius 0xA00.

### Burn Spell Mechanics

The Burn spell (0x004f2550):
- Damages all persons in cell
- Applies knockback away from center
- Knockback force: `(param_74 * (max_radius - distance)) / max_radius`
- Sets person on fire (state 0x1A) for 0x18 ticks
- Skips friendly shaman, shield-protected units

---

## Appendix I: Spell Configuration (constant.dat)

### Spell Cost Constants

| Constant | Description |
|----------|-------------|
| SPELL_BURN | Mana cost for Burn |
| SPELL_BLAST | Mana cost for Blast |
| SPELL_BOLT | Mana cost for Lightning Bolt |
| SPELL_WWIND | Mana cost for Whirlwind |
| SPELL_PLAGUE | Mana cost for Insect Plague |
| SPELL_INVIS | Mana cost for Invisibility |
| SPELL_FIREST | Mana cost for Firestorm |
| SPELL_HYPNO | Mana cost for Hypnotism |
| SPELL_GARMY | Mana cost for Ghost Army |
| SPELL_EROSION | Mana cost for Erosion |
| SPELL_SWAMP | Mana cost for Swamp |
| SPELL_LBRIDGE | Mana cost for Land Bridge |
| SPELL_AOD | Mana cost for Angel of Death |
| SPELL_QUAKE | Mana cost for Earthquake |
| SPELL_FLATTEN | Mana cost for Flatten |
| SPELL_VOLCANO | Mana cost for Volcano |
| SPELL_ARMAGEDDON | Mana cost for Armageddon |
| SPELL_CONVERT_WILD | Mana cost for Convert Wild |
| SPELL_SHIELD | Mana cost for Shield |
| SPELL_BLOODLUST | Mana cost for Bloodlust |
| SPELL_TELEPORT | Mana cost for Teleport |

### Spell Optional Settings (SPELL_*_OPT_S)

Additional spell parameters for each spell type.

### Altitude Spell Bonus

Spells cast from higher ground receive damage bonuses:

| Constant | Altitude Band |
|----------|---------------|
| ALT_BAND_0_SPELL_INCR | Lowest altitude |
| ALT_BAND_1_SPELL_INCR | |
| ALT_BAND_2_SPELL_INCR | |
| ALT_BAND_3_SPELL_INCR | |
| ALT_BAND_4_SPELL_INCR | |
| ALT_BAND_5_SPELL_INCR | |
| ALT_BAND_6_SPELL_INCR | |
| ALT_BAND_7_SPELL_INCR | Highest altitude (max bonus) |

---

## Appendix K: Shield Spell System

### Shield Functions

| Address    | Name                    | Description                    |
|------------|-------------------------|--------------------------------|
| 0x0049a230 | Shield_EjectPerson      | Eject person when shield hit   |
| 0x0049a9f0 | Shield_FindExitPosition | Find safe position for ejection|
| 0x0049b4e0 | Person_CheckShieldOnDeath| Check if shield saves unit    |

### Shield_EjectPerson (0x0049a230)

When a shielded unit is hit:
1. Finds the person in shield's passenger list
2. Calculates ejection velocity away from damage source
3. Applies knockback physics
4. Plays ejection animation
5. Decrements shield passenger count
6. If shield empty, begins shield decay

### Shield Structure (in Effect object)

| Offset | Description |
|--------|-------------|
| 0x7A-0x89 | Passenger object indices (up to 8) |
| 0x9E | Current passenger count |
| 0xA0 | Shield state flags |

---

## Appendix N: Mana Generation System

### Mana Rate Adjustments

| Constant | Description |
|----------|-------------|
| HUMAN_MANA_ADJUST | Human player mana rate modifier |
| COMPUTER_MANA_ADJUST | AI mana rate modifier |
| SHAMEN_DEAD_MANA_%_LOST | % mana lost on shaman death |
| SHAMEN_DEAD_MANA_%_GAIN | % mana gained when killing enemy shaman |

### Mana Per Unit Type (MANA_F_*)

| Constant | Unit | Description |
|----------|------|-------------|
| MANA_F_BRAVE | Brave | Mana generation per brave |
| MANA_F_WARR | Warrior | Mana generation per warrior |
| MANA_F_SPY | Spy | Mana generation per spy |
| MANA_F_PREACH | Preacher | Mana generation per preacher |
| MANA_F_SWARR | Super Warrior | Mana generation per super warrior |
| MANA_F_SHAMEN | Shaman | Mana generation from shaman |

### Mana Per Activity

| Constant | Description |
|----------|-------------|
| MANA_F_TRAINING | Mana from units in training |
| MANA_F_HOUSED | Mana from housed units |
| MANA_F_WORKING | Mana from working units |
| MANA_IDLE_BRAVES | Mana from idle braves |
| MANA_IDLE_SPECIALS | Mana from idle special units |
| MANA_BUSY_BRAVES | Mana from busy braves |
| MANA_BUSY_SPECIALS | Mana from busy special units |

### Mana Per Housing Level

| Constant | Housing Level |
|----------|---------------|
| MANA_F_HUT_LEVEL_1 | Small Hut |
| MANA_F_HUT_LEVEL_2 | Medium Hut |
| MANA_F_HUT_LEVEL_3 | Large Hut |

---

## Appendix AK: Spell System

### Spell_Init (0x00495440)

Initializes a newly cast spell object.

**Spell Types (subtypes 1-21):**
- Types 1-21 (0x01-0x15): Call FUN_00495b00 (standard spell init)
- Types 23-30 (0x17-0x1E): Direct state transition

**Mana Cost:**
```c
if (!(tribeFlags & 8)) {
    FUN_00425320(tribe, -manaCost, 0);  // Deduct mana
}
```

**Spell Cooldown:**
```c
cooldownArray[tribe + spellType * 4]++;
if (cooldownTimer[tribe + spellType * 4] == 0) {
    cooldownTimer[tribe + spellType * 4] = GetSpellCooldown(tribe, spellType);
}
```

### Spell Effect Functions

| Function | Address | Spell |
|----------|---------|-------|
| Spell_ProcessBurn | 0x004f2550 | Burn |
| Spell_ProcessShockwave | 0x004f2950 | Blast shockwave |
| Spell_ProcessBlast | 0x004f3a50 | Blast |
| Spell_CreateFirestorm | 0x004f3ee0 | Firestorm |
| Spell_CreateSwarmEffect | 0x004f6480 | Swarm/Insect Plague |
| Spell_ProcessLightningSwarm | 0x004f7330 | Lightning |
| Spell_CheckTargetValid | 0x004a5b60 | Target validation |
| Spell_FindTargetScenery | 0x004a5a30 | Find scenery target |

---

## Appendix BN: Spell Implementations

### Spell System Overview

- 22 spell types (0x01-0x16)
- Base range: 0xa00 (2560 units)
- Ring expansion: 0xa0 (160) units/frame
- 32 projectiles per blast spell

### Spell_InitDispatcher @ 0x00495440

Reads spell data from `DAT_00885760 + player_id * 0xc65`

### Volcano (Type 0x08)

**Spell_CreateVolcano** @ 0x004f3ee0
- Creates 32 fire projectiles
- Random angular velocity: -0x5a to +0x5a units/frame
- Projectile lifespan: 1-7 frames (random)
- Animation bank: 0x2b standard, 0x4ba special

### Firestorm (Type 0x01)

**Spell_ProcessBurn** @ 0x004f2550
- Area-of-effect burning
- Grid-based spatial query
- Max targets: 3-6 (DAT_005a32c8)
- Damage: SPELL_BURN_DAMAGE (15-25 HP)
- Knockback: `(spell_range - distance) / spell_range`
- Sets target state to 0x1a (burning)

### Earthquake/Shockwave

**Spell_ProcessShockwave** @ 0x004f2950
- Expanding circular wave
- Range: starts 0xa0, adds 0xa0/frame, max 0xa00
- Damage scales with distance
- Building destruction via Building_ApplyDamage
- 8-direction radial spread

### Lightning Swarm (Angel of Death)

**Spell_ProcessLightningSwarm** @ 0x004f7330

**4-Stage State Machine:**
| Stage | Action |
|-------|--------|
| 0 | Find target building |
| 1 | Travel to building |
| 2 | Attack (eject up to 6 people) |
| 3 | Spawn lightning bolts |

- Targeting: 8-cell radius
- Damage per person: DAT_005a3224
- Animation bank: 0x22

### Blast/Conversion

**Spell_ProcessBlast** @ 0x004f3a50
- 32 projectiles in circular formation
- Expanding ring (0xa0 units/frame)
- Max radius: 0xa00 (2560 units)

**Conversion flags (+0x76):**
- Bit 0: Convert wild creatures
- Bit 1: Damage enemy buildings

### Mana System

**Tick_UpdateMana** @ 0x004aeac0

```c
// Global mana counter
DAT_00885720 += 1;

// Per-player cooldown at DAT_00886384 + player * 0xc65
if (cooldown > 0) cooldown--;

// Mana regeneration pools
_DAT_006847bc += DAT_006847a8;  // Base
_DAT_006847c4 += DAT_006847b8;  // Secondary
_DAT_006847c0 += DAT_006847ac;  // Tertiary
```

### Spell_CheckTargetValid @ 0x004a5b60

- Validates target location against grid
- Checks neighboring cells (±2 in both axes)
- Validates mana availability
- Returns castability status

---


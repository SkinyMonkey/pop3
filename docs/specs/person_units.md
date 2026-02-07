# Person Units and Training

## Phase 4: Unit System

### Object Lifecycle Functions

| Address    | Name                    | Description                              |
|------------|-------------------------|------------------------------------------|
| 0x004afc70 | Object_Create           | Main object spawn function               |
| 0x004b00c0 | Object_Destroy          | Removes object from linked lists         |
| 0x004b01e0 | Object_CopyData         | Copies object data preserving key fields |
| 0x004af950 | Object_InitByType       | Dispatches to type-specific init         |
| 0x004b0950 | Object_SetPosition      | Updates object position and map cell     |
| 0x004aea50 | Object_SetSelected      | Handles unit selection                   |

### Type-Specific Initialization

| Address    | Name              | Description                              |
|------------|-------------------|------------------------------------------|
| 0x004fd260 | Person_Init       | Initializes person objects (8 subtypes)  |
| 0x0042e230 | Building_Init     | Initializes building objects             |
| 0x00483270 | Creature_Init     | Initializes creature objects             |
| 0x005007b0 | Person_InitCommon | Common person initialization             |

### State Machine System

The game uses a state machine for all object types. State is stored at offset 0x2C.

| Address    | Name                  | Description                              |
|------------|----------------------|------------------------------------------|
| 0x004afac0 | Object_OnStateExit   | Called when exiting a state (stub)       |
| 0x004afa10 | Object_OnStateEnter  | Dispatches to type-specific state handler|
| 0x004fd5d0 | Person_SetState      | Massive switch for person behaviors (40+ states) |
| 0x0042e430 | Building_SetState    | Building state handler                   |
| 0x00483580 | Creature_SetState    | Creature state handler                   |
| 0x00497bd0 | Vehicle_SetState     | Vehicle state handler                    |
| 0x004bd100 | Scenery_SetState     | Scenery state handler                    |
| 0x004600c0 | General_SetState     | General object state handler             |
| 0x004f1950 | Effect_SetState      | Effect state handler                     |
| 0x004576f0 | Shot_SetState        | Projectile state handler                 |
| 0x0048f9b0 | Shape_SetState       | Shape state handler                      |
| 0x004ed340 | Internal_SetState    | Internal object state handler            |
| 0x004958b0 | Spell_SetState       | Spell object state handler               |
| 0x004b14d0 | Person_SetIdleState  | Sets person to idle state                |

### Combat System

| Address    | Name                    | Description                              |
|------------|-------------------------|------------------------------------------|
| 0x00504f20 | Object_ApplyDamage      | Applies damage to object (offset 0x6E -= damage) |
| 0x004fb620 | Shot_ProcessImpact      | Handles projectile impact and damage     |
| 0x004f3a50 | Spell_ProcessBlast      | Processes blast spell area damage        |
| 0x004cd3a0 | Tribe_KillAllUnits      | Kills all units of a tribe               |
| 0x004b5000 | Tribe_TrackKill         | Tracks kill statistics                   |
| 0x0049b4e0 | Person_CheckShieldOnDeath | Checks shield buff before death        |
| 0x004fed30 | Person_SelectAnimation  | Selects animation based on state         |

### Object Structure (Updated)

| Offset | Size | Field Name      | Notes                                    |
|--------|------|-----------------|------------------------------------------|
| 0x0C   | 4    | flags           | Bit 0 = inactive, Bit 4 = hit, etc.      |
| 0x10   | 4    | flags2          | Various state flags                      |
| 0x14   | 4    | flags3          | More flags (0x100 = creature flag)       |
| 0x20   | 2    | cell_list_next  | Next object in cell linked list          |
| 0x24   | 2    | object_index    | Index/handle in pointer array            |
| 0x26   | 2    | angle           | Facing direction (0x000-0x7FF)           |
| 0x2A   | 1    | model_type      | Object type (1-11)                       |
| 0x2B   | 1    | subtype         | Object subtype                           |
| 0x2C   | 1    | state           | Current state                            |
| 0x2D   | 1    | state_counter   | Counter for state transitions            |
| 0x2E   | 1    | flags4          | Additional flags                         |
| 0x2F   | 1    | owner           | Tribe owner (0-3, 0xFF = neutral)        |
| 0x30   | 1    | anim_type       | Animation type                           |
| 0x3D   | 2    | pos_x           | X position (world coords)                |
| 0x3F   | 2    | pos_y           | Y position (world coords)                |
| 0x41   | 2    | pos_z           | Z/height position                        |
| 0x57   | 2    | direction       | Movement direction                       |
| 0x5D   | 2    | target_dir      | Target direction                         |
| 0x5F   | 2    | move_timer      | Movement timer                           |
| 0x68   | 4    | home_pos        | Home position (X,Y)                      |
| 0x6C   | 2    | max_health      | Maximum health                           |
| 0x6E   | 2    | health          | Current health (damage subtracts here)   |
| 0x70   | 2    | timer2          | Secondary timer                          |
| 0x7B   | 1    | attack_power    | Attack power                             |
| 0x7C   | 1    | defense         | Defense value                            |
| 0x7D   | 1    | prev_state      | Previous state (for state restoration)   |
| 0x88   | 2    | attacker_idx    | Index of attacking object                |
| 0x8A   | 2    | target_idx      | Index of target object                   |
| 0x9F   | 2    | shield_idx      | Index of shield spell object             |
| 0xA0   | 1    | shield_broken   | Shield has been broken flag              |
| 0xAF   | 1    | killer_unit_type| Unit type of killer (for stats)          |
| 0xB0   | 1    | killer_tribe    | Tribe of killer (for stats)              |

### Unit Type Data Table (0x0059FE44)

Located at 0x0059FE44, stride 0x32 (50 bytes) per unit type.

| Offset | Field              | Description                    |
|--------|--------------------|--------------------------------|
| 0x00   | default_state      | Initial state on spawn         |
| 0x02   | anim_type          | Animation type                 |
| 0x0C   | max_health         | Base health value              |
| 0x24   | speed              | Movement speed                 |
| 0x2C   | flags              | Unit flags (bit 3 = varies HP) |

### Fight Damage Constants

Located in constant.dat, stored at 0x0059FE50:
- FIGHT_DAMAGE_BRAVE
- FIGHT_DAMAGE_WARR
- FIGHT_DAMAGE_SPY
- FIGHT_DAMAGE_PREACH
- FIGHT_DAMAGE_SWARR
- FIGHT_DAMAGE_SHAMEN

### Renamed Functions (Phase 4)

| Address    | Original        | New Name                    |
|------------|-----------------|-----------------------------|
| 0x004afc70 | FUN_004afc70    | Object_Create               |
| 0x004b00c0 | FUN_004b00c0    | Object_Destroy              |
| 0x004b01e0 | FUN_004b01e0    | Object_CopyData             |
| 0x004af950 | FUN_004af950    | Object_InitByType           |
| 0x004b0950 | FUN_004b0950    | Object_SetPosition          |
| 0x004aea50 | FUN_004aea50    | Object_SetSelected          |
| 0x004fd260 | FUN_004fd260    | Person_Init                 |
| 0x0042e230 | FUN_0042e230    | Building_Init               |
| 0x00483270 | FUN_00483270    | Creature_Init               |
| 0x005007b0 | FUN_005007b0    | Person_InitCommon           |
| 0x004afac0 | FUN_004afac0    | Object_OnStateExit          |
| 0x004afa10 | FUN_004afa10    | Object_OnStateEnter         |
| 0x004fd5d0 | FUN_004fd5d0    | Person_SetState             |
| 0x0042e430 | FUN_0042e430    | Building_SetState           |
| 0x00483580 | FUN_00483580    | Creature_SetState           |
| 0x00497bd0 | FUN_00497bd0    | Vehicle_SetState            |
| 0x004bd100 | FUN_004bd100    | Scenery_SetState            |
| 0x004600c0 | FUN_004600c0    | General_SetState            |
| 0x004f1950 | FUN_004f1950    | Effect_SetState             |
| 0x004576f0 | FUN_004576f0    | Shot_SetState               |
| 0x0048f9b0 | FUN_0048f9b0    | Shape_SetState              |
| 0x004ed340 | FUN_004ed340    | Internal_SetState           |
| 0x004958b0 | FUN_004958b0    | Spell_SetState              |
| 0x004b14d0 | FUN_004b14d0    | Person_SetIdleState         |
| 0x00504f20 | FUN_00504f20    | Object_ApplyDamage          |
| 0x004fb620 | FUN_004fb620    | Shot_ProcessImpact          |
| 0x004f3a50 | FUN_004f3a50    | Spell_ProcessBlast          |
| 0x004cd3a0 | FUN_004cd3a0    | Tribe_KillAllUnits          |
| 0x004b5000 | FUN_004b5000    | Tribe_TrackKill             |
| 0x0049b4e0 | FUN_0049b4e0    | Person_CheckShieldOnDeath   |
| 0x004fed30 | FUN_004fed30    | Person_SelectAnimation      |

---

## Person State Machine

### Person States (offset 0x2C)

The Person_SetState function handles 44+ different states:

| State | Value | Description |
|-------|-------|-------------|
| Idle | 0x01 | Standing idle |
| Dying | 0x02 | Death animation |
| Moving | 0x03 | Walking/running |
| Wander | 0x04 | Random movement |
| GoToPoint | 0x05 | Moving to specific location |
| FollowPath | 0x06 | Following waypoint path |
| GoToMarker | 0x07 | Moving to marker |
| WaitForPath | 0x08 | Waiting for pathfinding |
| WaitAtMarker | 0x09 | Waiting at destination |
| EnterBuilding | 0x0A | Entering a building |
| InsideBuilding | 0x0B | Inside building (housing) |
| InsideTraining | 0x0C | Inside training building |
| Building | 0x0D | Constructing building |
| InTraining | 0x0E | Being trained |
| WaitOutside | 0x0F | Waiting outside building |
| Training | 0x10 | Training complete |
| Housing | 0x11 | In housing |
| Gathering | 0x13 | Gathering wood |
| GatheringWood | 0x15 | Chopping tree |
| CarryingWood | 0x16 | Carrying wood to building |
| Drowning | 0x17 | Drowning in water |
| Dead | 0x18 | Dead |
| Fighting | 0x19 | In combat |
| Fleeing | 0x1A | Running away |
| Spawning | 0x1B | Spawning from building |
| BeingSacrificed | 0x1C | Being sacrificed |
| InShield | 0x1D | Inside shield spell |
| InShieldIdle | 0x1E | Idle inside shield |
| Preaching | 0x1F | Preacher converting |
| SitDown | 0x20 | Sitting animation |
| BeingConverted | 0x21 | Being converted by preacher |
| WaitingAfterConvert | 0x22 | Post-conversion wait |
| WaitingForBoat | 0x23 | Waiting to board boat |
| Placeholder | 0x24 | Placeholder state |
| GetOffBoat | 0x25 | Exiting boat |
| WaitingInWater | 0x26 | Waiting in shallow water |
| EnteringVehicle | 0x27 | Boarding vehicle |
| ExitingVehicle | 0x28 | Leaving vehicle |
| Celebrating | 0x29 | Victory celebration |
| Teleporting | 0x2A | Being teleported |
| InternalState | 0x2B | Internal state |
| WaitingAtReincPillar | 0x2C | At reincarnation pillar |

### State Entry Functions

| Address    | Name                          | Description                    |
|------------|-------------------------------|--------------------------------|
| 0x00500b00 | Person_EnterMovingState       | Start movement to target       |
| 0x00437b40 | Person_EnterFightingState     | Enter combat state             |
| 0x00501750 | Person_EnterBuildingState     | Start building construction    |
| 0x005021c0 | Person_EnterGatheringState    | Start resource gathering       |
| 0x00501c00 | Person_EnterTrainingState     | Enter training building        |
| 0x00501e20 | Person_EnterHousingState      | Enter housing                  |
| 0x00503190 | Person_EnterDrowningState     | Start drowning                 |
| 0x00502f70 | Person_StartWoodGathering     | Begin chopping tree            |
| 0x00503e50 | Person_EnterPreachingState    | Preacher starts converting     |
| 0x00504410 | Person_EnterBeingConvertedState | Being converted               |
| 0x0050a960 | Person_EnterVehicleState      | Board boat/airship             |
| 0x0050b480 | Person_ExitVehicleState       | Exit boat/airship              |
| 0x0050b990 | Person_EnterCelebrationState  | Victory celebration            |
| 0x0050d620 | Person_EnterTeleportState     | Being teleported               |
| 0x00509e90 | Preacher_StartConverting      | Preacher begins conversion     |

---

## Conversion System

### Preacher Conversion Constants

| Constant | Description |
|----------|-------------|
| PREACHEE_CONV_FREQ | Conversion tick frequency |
| PREACHEE_CONV_CHANCE | Base conversion chance |
| CONV_BRAVE, CONV_WARR, etc. | Conversion time per unit type |
| CONV_TIME_TEMPLE | Temple conversion speed |
| CONV_TIME_SPY, CONV_TIME_WARRIOR | Training building conversion |

### Spy System Constants

| Constant | Description |
|----------|-------------|
| SPY_DISGUISE_DELAY | Time before spy can disguise |
| SPY_DT_RADIUS | Spy detection radius |
| SPY_SPEED | Spy movement speed |

---

## Tribe System

### Tribe Data Structure

- **Base Address:** 0x00885760
- **Stride:** 0xC65 (3173 bytes per tribe)
- **Max Tribes:** 4 (indexed 0-3)

### Key Tribe Offsets

| Offset | Description |
|--------|-------------|
| 0x93D  | Tribe flags (bit 0 = spell active) |
| 0x37F  | AI control flag (1 = AI controlled) |

### Tribe Functions

| Address    | Name                  | Description                    |
|------------|-----------------------|--------------------------------|
| 0x004b5000 | Tribe_TrackKill       | Track kill for statistics      |
| 0x004cd3a0 | Tribe_KillAllUnits    | Kill all units of a tribe      |
| 0x0041a7d0 | AI_UpdateAllTribes    | Run AI for all tribes          |
| 0x0041a8b0 | AI_UpdateTribe        | Run AI for single tribe        |

### Kill Tracking

Tribe_TrackKill (0x004b5000) adds kill points based on victim unit type:
- Points stored at tribe offset 0x744 + (killer_subtype * 0x52)
- Point values from DAT_0059fe5e table per unit subtype

---

## Wild Conversion System

### Wild_ConvertToBrave (0x00502e60)

Converts a wild follower to a brave for a tribe:
1. Creates conversion visual effect (type 7, subtype 0x3A)
2. Creates new Brave (type 1, subtype 2) for specified tribe
3. Plays conversion sound (sound ID 5)
4. Triggers minimap ping if player's tribe
5. Destroys original wild follower

---

## Selection System

### Object_SetSelected (0x004aea50)

- Selection flag at offset 0x7A (bit 7)
- Additional flag at offset 0x14 (bit 28) for multi-select
- Tracks currently selected object at DAT_00952422

---

## Appendix J: Training System

### Training Cost Bands

Training costs vary based on current population:

| Constant | Population Range |
|----------|------------------|
| TRAIN_MANA_BAND_00_03 | 0-3 population |
| TRAIN_MANA_BAND_04_07 | 4-7 population |
| TRAIN_MANA_BAND_08_11 | 8-11 population |
| TRAIN_MANA_BAND_12_15 | 12-15 population |
| TRAIN_MANA_BAND_16_20 | 16-20 population |
| TRAIN_MANA_BAND_21+ | 21+ population |

### Human vs Computer Training Costs

| Human Constant | Computer Constant | Unit |
|----------------|-------------------|------|
| HUMAN_TRAIN_MANA_WARR | CP_TRAIN_MANA_WARR | Warrior |
| HUMAN_TRAIN_MANA_SPY | CP_TRAIN_MANA_SPY | Spy |
| HUMAN_TRAIN_MANA_PREACH | CP_TRAIN_MANA_PREACH | Preacher |
| HUMAN_TRAIN_MANA_SWARR | CP_TRAIN_MANA_SWARR | Super Warrior |

---

## Appendix O: Population System

### Housing Capacity

| Constant | Description |
|----------|-------------|
| MAX_POP_VALUE__HUT_1 | Max population in Small Hut |
| MAX_POP_VALUE__HUT_2 | Max population in Medium Hut |
| MAX_POP_VALUE__HUT_3 | Max population in Large Hut |

### Spawn Rate Bands (17 bands)

Population spawn rate varies based on housing capacity utilization:

| Constant | Capacity Range |
|----------|----------------|
| SPROG%_POP_BAND_00_04% | 0-4% capacity |
| SPROG%_POP_BAND_05_09% | 5-9% capacity |
| SPROG%_POP_BAND_10_14% | 10-14% capacity |
| SPROG%_POP_BAND_15_19% | 15-19% capacity |
| SPROG%_POP_BAND_20_24% | 20-24% capacity |
| ... | (continues in 5% increments) |
| SPROG%_POP_BAND_95_99% | 95-99% capacity |

Higher capacity utilization = slower spawn rate.

---

## Appendix X: Selection System

### Object_SetSelected (0x004aea50)

```c
void Object_SetSelected(int object, bool selected, byte flags) {
    if (!selected) {
        object->flags &= ~0x80;
        object->selectionFlags &= 0x7f;
    } else {
        object->selectionFlags |= 0x80;

        if (flags & 4) {
            object->flags |= 0x10000000;  // Multi-select flag
        } else {
            object->flags &= ~0x10000000;
        }

        // Update UI selection tracking
        if (object->owner == g_LocalPlayer &&
            DAT_00952422 != object->objectId) {
            DAT_0095242f &= 0xfd;
            DAT_00952422 = object->objectId;
            DAT_0095242c = 0;
        }
    }
}
```

---

## Appendix AG: Training System

### Person_EnterTrainingState (0x00501c00)

Initializes a person entering a training building.

```c
void Person_EnterTrainingState(int person) {
    person->targetAngle = 0;
    person->subState = 0x19;  // Training sub-state

    // Random training duration: 50-100 ticks
    person->stateTimer = (Random() % 50) + 50;

    // Get building's facing direction
    cellCoord = GetCellCoord(person->position);
    buildingAngle = buildingData[cellFlags[cellCoord] & 0xF].angle * 256;

    if (person->flags & 0x80) {
        person->targetFacing = buildingAngle;
    }
    person->facing = buildingAngle;

    // Face opposite if flag set
    if (person->flags & 0x8000) {
        person->angle = (buildingAngle + 0x400) & 0x7FF;
    } else {
        person->angle = buildingAngle;
    }
}
```

---

## Appendix AI: Conversion System

### Preacher_StartConverting (0x00509e90)

Initiates the conversion process for a preacher.

**Process:**
1. Check if conversion is possible (FUN_0047c150)
2. Get target position from task data
3. If in guard tower, use different conversion type (0x1F vs 0x11)
4. Set up conversion targeting
5. Clean up pathfinding and start conversion animation

### Wild_ConvertToBrave (0x00502e60)

Converts a wild person to a tribe's brave.

```c
int Wild_ConvertToBrave(int wildPerson, byte tribe, short* position) {
    // Store extra data for new brave
    DAT_0087a9db[0] = position->x;
    DAT_0087a9db[1] = position->z;
    DAT_0087a9db[2] = wildPerson->angle;

    // Create new brave at wild person's location
    brave = Object_Create(PERSON, BRAVE, tribe, wildPerson->position);

    if (brave != 0) {
        // Copy formation flag
        if (!(wildPerson->flags & 2)) {
            brave->flags |= 0x40000;
        }

        // Create conversion effect
        effect = Object_Create(EFFECT, 0x3A, NEUTRAL, wildPerson->position);
        if (effect != 0) {
            Sound_Play(wildPerson, 5, 0);
            effect->linkedObject = brave->objectId;

            // Notify player if their tribe
            if (brave->owner == g_LocalPlayer) {
                // ... minimap ping logic
            }
        }

        // Kill wild person
        FUN_004ff9e0(wildPerson);
    }
    return brave;
}
```

---


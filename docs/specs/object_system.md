# Object System

## Object System

### Object Management

The game uses a linked list system for managing all game objects.

**Key Global Pointers:**
- `g_PersonListHead` - Head of active object linked list
- `g_ObjectPtrArray` - Array of object pointers indexed by object ID
- `DAT_008788b4` - Free list head (high priority objects)
- `DAT_008788b8` - Free list head (low priority objects)
- `DAT_008788c0` - Destroyed objects list

### Object Structure Offsets

| Offset | Size | Description |
|--------|------|-------------|
| 0x00   | 4    | Previous object pointer |
| 0x04   | 4    | Next object pointer |
| 0x0C   | 4    | Object flags 1 |
| 0x0E   | 1    | Object status flags |
| 0x10   | 4    | Object flags 2 |
| 0x14   | 4    | Object flags 3 |
| 0x20   | 2    | Next object in cell list |
| 0x22   | 2    | Previous object in cell list |
| 0x24   | 2    | Object index in array |
| 0x26   | 2    | Angle (0x000-0x7FF) |
| 0x2A   | 1    | Model type (1-11) |
| 0x2B   | 1    | Model subtype |
| 0x2C   | 1    | Current state |
| 0x2D   | 1    | State phase |
| 0x2E   | 1    | Spawn index |
| 0x2F   | 1    | Owner tribe (-1 = neutral) |
| 0x3D   | 2    | Position X |
| 0x3F   | 2    | Position Y |
| 0x41   | 2    | Position Z (height) |
| 0x43   | 2    | Velocity X |
| 0x45   | 2    | Velocity Y |
| 0x47   | 2    | Velocity Z |
| 0x57   | 2    | Target angle |
| 0x5D   | 2    | Movement angle |
| 0x6C   | 2    | Max health |
| 0x6E   | 2    | Current health |
| 0x7A   | 1    | Selection flags (bit 7 = selected) |
| 0x9F   | 2    | Shield object index |
| 0xB0   | 1    | Last attacker tribe |

### Object_Create (0x004afc70)

Creates a new game object:
```c
int* Object_Create(byte modelType, byte subType, byte ownerTribe, position* pos)
```

1. Checks object pool availability
2. Removes from free list, adds to active list
3. Initializes base object fields
4. Calls type-specific init function
5. Returns object pointer (or NULL if pool full)

**Object Pool Limits:**
- Max active objects: 0x44D (1101)
- Low priority pool: 0x280 (640) for effects/particles
- High priority pool: remaining for units/buildings

### Object_Destroy (0x004b00c0)

Destroys a game object:
1. Removes from cell linked list
2. Removes from active object list
3. Adds to destroyed objects list
4. Clears model type

### Object_InitByType (0x004af950)

Dispatches to type-specific initialization:

| Type | Address    | Function Name |
|------|------------|---------------|
| 1    | 0x004fd260 | Person_Init   |
| 2    | 0x0042e230 | Building_Init |
| 3    | 0x00483270 | Creature_Init |
| 4    | 0x00497a10 | Vehicle_Init  |
| 5    | 0x004bcde0 | Scenery_Init  |
| 6    | 0x0045fe00 | General_Init  |
| 7    | 0x004f0e20 | Effect_Init   |
| 8    | 0x004573e0 | Shot_Init     |
| 9    | 0x0048f8d0 | Shape_Init    |
| 10   | 0x004ecf50 | Internal_Init |
| 11   | 0x00495440 | Spell_Init    |

### Person_Init (0x004fd260)

Initializes person objects with subtype-specific logic:

| Subtype | Special Initialization |
|---------|----------------------|
| 1 (Wild) | Sets tribe to -1, randomizes appearance |
| 2 (Brave) | Standard init, double health in tutorial |
| 3-4 (Warrior/Religious) | Standard init |
| 5 (Spy) | Sets disguise flag, stores original tribe |
| 6 (Super Warrior) | Standard init |
| 7 (Shaman) | Stores shaman pointer in tribe data |
| 8 (Angel of Death) | Plays spawn sound, random direction |

---

## Internal Object System (Type 10)

Handles markers, beacons, formations, and other invisible game logic objects.

### Internal Subtypes

| Subtype | Description |
|---------|-------------|
| 1       | Tribe marker (start position) |
| 2       | Destroyed immediately |
| 3       | Flag marker (guard/patrol point) |
| 5       | Animation flag |
| 6       | Building damage trigger |
| 7       | Special trigger |
| 8       | Combat marker |
| 10      | Reincarnation site |
| 12      | Discovery trigger |
| 16      | Armageddon marker |
| 18      | Shaman reincarnation pillar |
| 19      | Ghost/clone marker |

### Marker Functions

| Address    | Name           | Description                    |
|------------|----------------|--------------------------------|
| 0x004ecf50 | Internal_Init  | Initialize internal object     |
| 0x004eedc0 | Internal_Update | Update internal objects       |

---

## Scenery System (Type 5)

Handles trees, rocks, and other environmental objects.

### Scenery Subtypes (Trees)

| Subtype | Description |
|---------|-------------|
| 1-8     | Tree types (different growth/wood yield) |
| 9       | Stone head (worship site) |
| 10      | Obelisk |
| 11      | Totem pole |
| 12      | Discovery item |
| 13      | Swamp grass |
| 14-15   | Additional trees |
| 16      | Trigger scenery |
| 17      | Spell discovery item |
| 18-19   | Additional vegetation |

### Scenery Data Table (0x005a07a0)

Per-scenery type data (0x18 bytes each):
- Offset 0x01: Default state
- Offset 0x02: Growth state
- Offset 0x04: Flags
- Offset 0x10: Min height
- Offset 0x1A: Max height

### Scenery Functions

| Address    | Name          | Description                    |
|------------|---------------|--------------------------------|
| 0x004bcde0 | Scenery_Init  | Initialize scenery object      |
| 0x004bd6c0 | Tree_Init     | Initialize tree specifically   |

---

## Appendix AJ: Object State Machine

### Object_OnStateEnter (0x004afa10)

Dispatches state initialization based on model type:

```c
void Object_OnStateEnter(int object) {
    switch(object->modelType) {
        case 1:  Person_SetState(object);   break;
        case 2:  Building_SetState(object); break;
        case 3:  Creature_SetState(object); break;
        case 4:  Vehicle_SetState(object);  break;
        case 5:  Scenery_SetState(object);  break;
        case 6:  General_SetState(object);  break;
        case 7:  Effect_SetState(object);   break;
        case 8:  Shot_SetState(object);     break;
        case 9:  Shape_SetState(object);    break;
        case 10: Internal_SetState(object); break;
        case 11: Spell_SetState(object);    break;
    }
}
```

### Person States (Complete List)

| State | ID | Handler | Description |
|-------|----|---------| ------------|
| STATE_NONE | 0x00 | - | Invalid |
| STATE_NEW | 0x01 | Random timer | Just spawned |
| STATE_SPAWN | 0x02 | Person_SetIdleState | Spawn animation |
| STATE_MOVE | 0x03 | Person_EnterMovingState | Walking |
| STATE_WANDER | 0x04 | Random direction | Idle wandering |
| STATE_WAIT | 0x05 | FUN_004d7be0 | Waiting |
| STATE_WAIT2 | 0x06 | FUN_004d7be0 | Waiting (alt) |
| STATE_GOTO | 0x07 | FUN_004d7e20 | Moving to target |
| STATE_8 | 0x08 | - | Unknown |
| STATE_9 | 0x09 | - | Unknown |
| STATE_ENTER_BLDG | 0x0A | FUN_00477810 | Entering building |
| STATE_B | 0x0B | Set flags | Unknown |
| STATE_C | 0x0C | Set flags | Unknown |
| STATE_BUILD | 0x0D | Person_EnterBuildingState | Constructing |
| STATE_HOUSED | 0x0E | Person count++ | Inside housing |
| STATE_EXIT_BLDG | 0x0F | FUN_004be8f0 | Exiting building |
| STATE_TRAINING | 0x10 | Person_EnterTrainingState | In training |
| STATE_HOUSING | 0x11 | Person_EnterHousingState | Entering hut |
| STATE_GATHER | 0x13 | Person_EnterGatheringState | Wood gathering |
| STATE_WOODCUT | 0x15 | Person_StartWoodGathering | Cutting trees |
| STATE_ATTACK | 0x16 | Attack animation | Combat |
| STATE_DROWN | 0x17 | Person_EnterDrowningState | Drowning |
| STATE_DYING | 0x18 | Person_CheckShieldOnDeath | Death |
| STATE_FIGHT | 0x19 | Person_EnterFightingState | Melee combat |
| STATE_FLEE | 0x1A | Flee animation | Running away |
| STATE_DIE_COMPLETE | 0x1B | Death complete | Dead |
| STATE_GUARD | 0x1C | Set timer | Guarding |
| STATE_1D | 0x1D | FUN_00437ae0 | Unknown |
| STATE_1E | 0x1E | Path cleanup | Unknown |
| STATE_PREACH | 0x1F | Person_EnterPreachingState | Converting |
| STATE_DANCE | 0x20 | Random dance | Celebrating |
| STATE_CONVERTED | 0x21 | Person_EnterBeingConvertedState | Being converted |
| STATE_22 | 0x22 | Animation | Unknown |
| STATE_23 | 0x23 | FUN_004b1220 | Unknown |
| STATE_25 | 0x25 | Set flags | Unknown |
| STATE_26 | 0x26 | Building eject | Unknown |
| STATE_VEHICLE | 0x27 | Person_EnterVehicleState | In vehicle |
| STATE_EXIT_VEH | 0x28 | Person_ExitVehicleState | Leaving vehicle |
| STATE_CELEBRATE | 0x29 | Person_EnterCelebrationState | Victory dance |
| STATE_TELEPORT | 0x2A | Person_EnterTeleportState | Teleporting |
| STATE_2B | 0x2B | - | Unknown |
| STATE_2C | 0x2C | Set flags | Unknown |

---

## Appendix AE: Object Creation

### Object_Create (0x00432200)

Creates a new game object of the specified type.

**Parameters:**
- `param_1`: Model type (1-11)
- `param_2`: Subtype
- `param_3`: Owner tribe (0-3, or 0xFF for neutral)
- `param_4`: Position (x, z, y)

**Object Pool Management:**

The game uses a fixed-size object pool with two free lists:
- `DAT_008788b4`: Primary free list
- `DAT_008788b8`: Secondary free list (for high-priority objects)

**Creation Limits:**
```c
// Check object count limits
if (DAT_00884be9 - DAT_00884bf1 >= 0x44D) {
    // At capacity - check if can use secondary pool
}
```

**Object Initialization:**
```c
object->modelType = param_1;
object->subtype = param_2;
object->owner = param_3;
object->position = param_4->position;
object->height = param_4->height;
object->velocity = (0, 0, 0);
object->creationTime = g_GameTick;

Object_InitByType(object);  // Type-specific initialization
```

**Object ID System:**
- Objects with ID < 0x280 tracked in DAT_00884bf1
- Used for object limits and priority

### Tribe_TrackKill (0x004b5000)

Records kills for scoring:
```c
void Tribe_TrackKill(int killer, int victim) {
    int tribeOffset = killer->owner * 0xC65;

    if (tribeData[tribeOffset + 0x1F] == 1 &&  // AI tracking enabled
        killer->modelType == PERSON) {

        byte taskId = killer->currentTask;
        if (taskId > 0 && taskId < 10) {
            // Add victim's kill value to task statistics
            tribeStats[tribeOffset + taskId * 0x52] +=
                unitKillValues[victim->subtype];
        }
    }
}
```

---

## Appendix AF: Internal Objects

"Internal" objects (model type 10) are invisible helper objects for game logic.

### Internal_Update (0x004eedc0)

```c
void Internal_Update(int object) {
    if ((object->animFrame & 1) == 0) {
        if (object->subtype == 6) {
            // Type 6: Special handler
            result = FUN_0043eb20(object, object->field72,
                                  object->field76, object->field77, 0, 0);
        } else {
            // Default handler
            result = FUN_0043e160(object, object->field72,
                                  object->field76, object->field77, 0, 1);
        }
        object->field78 = result;
    }
}
```

Internal objects are used for:
- Waypoint markers
- Trigger zones
- AI pathfinding nodes
- Spell targeting helpers

---

## Appendix BI: Linked List System

### Object List Heads

| Address | List Type |
|---------|-----------|
| 0x008788f0 | Building list |
| 0x008788f4 | Fighting list |
| 0x008788f8 | Misc/effects list |
| 0x008788c8 | Spell list |
| 0x008788b4 | Free list (high priority) |
| 0x008788b8 | Free list (low priority) |
| 0x008788c0 | Destroyed list |

### Object Node Structure

```c
struct ObjectNode {
    void *prev;           // +0x00
    void *next;           // +0x04
    // ... object data ...
    uint16_t cell_next;   // +0x20 (cell list next)
    uint16_t cell_prev;   // +0x22 (cell list prev)
    uint16_t handle;      // +0x24 (array index)
};
```

### Cell-Based Spatial Lists

- Base: 0x00888982
- 128×128 grid
- Index: `(y & 0xFE) * 2 | (x & 0xFE00)`

### Object Pool

| Parameter | Value |
|-----------|-------|
| Max objects | 1101 (0x44D) |
| Object size | 179 bytes (0xB3) |
| Pool address | 0x008c89c0 |
| Pointer array | 0x00878928 |

### List Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Object_Create | 0x004afc70 | Allocate from pool |
| Object_Destroy | 0x004b00c0 | Return to pool |
| InitObjectPointerArray | 0x004afbf0 | Init free lists |
| Cell_UpdateFlags | 0x004e9fe0 | Rebuild cell lists |

---

## Appendix BK: Object State Machine

### State Dispatcher

**Object_OnStateEnter** @ 0x004afa10 dispatches by object type (+0x2A):
```
1 = Person_SetState    @ 0x004fd5d0
2 = Building_SetState  @ 0x0042e430
3 = Creature_SetState  @ 0x00483580
4 = Vehicle_SetState   @ 0x00497bd0
5 = Scenery_SetState
6 = General_SetState   @ 0x004600c0
7 = Effect_SetState    @ 0x004f1950
8 = Shot_SetState      @ 0x004576f0
9 = Shape_SetState     @ 0x0048f9b0
10 = Internal_SetState
11 = Spell_SetState
```

### Person States (45 states)

| State | Value | Name | Entry Function |
|-------|-------|------|----------------|
| 0x01 | RandomIdle | Random timer setup | - |
| 0x02 | Idle | Person_SetIdleState | 0x004b14d0 |
| 0x03 | Moving | Person_EnterMovingState | 0x00500b00 |
| 0x04 | Carrying | Random anim + movement | - |
| 0x05 | Attacking | Combat setup | 0x004d7be0 |
| 0x06 | AttackingSpecial | Combat setup | 0x004d7be0 |
| 0x07 | Fleeing | Combat setup | 0x004d7be0 |
| 0x08 | Fear | - | - |
| 0x09 | Wandering | - | - |
| 0x0A | Dead1 | Death setup | - |
| 0x0B | Fighting | Person_EnterFightingState | 0x00437b40 |
| 0x0C | JoinedFighting | Person_EnterFightingState | 0x00437b40 |
| 0x0D | EnteringBuilding | Person_EnterBuildingState | 0x00501750 |
| 0x0E | InBuilding | Occupancy++ | - |
| 0x0F | LeavingBuilding | Random idle | - |
| 0x10 | Training | Person_EnterTrainingState | 0x00501c00 |
| 0x11 | Housing | Person_EnterHousingState | 0x00501e20 |
| 0x13 | Gathering | Person_EnterGatheringState | 0x005021c0 |
| 0x15 | WoodGathering | Person_StartWoodGathering | 0x00502f70 |
| 0x16 | Converted | - | - |
| 0x17 | Drowning | Person_EnterDrowningState | 0x00503190 |
| 0x18 | Dead | Shield check + death | - |
| 0x19 | Fighting2 | - | - |
| 0x1A | LeavingAfterAttack | - | - |
| 0x1B | IdleAfterAction | - | - |
| 0x1C | SpellRecovery | - | - |
| 0x1E | LumberGathering | - | - |
| 0x1F | Preaching | Person_EnterPreachingState | 0x00503e50 |
| 0x20 | Working | - | - |
| 0x21 | BeingConverted | Person_EnterBeingConvertedState | 0x00504410 |
| 0x22 | ConversionComplete | - | - |
| 0x23 | Praying | - | - |
| 0x25 | DeadFalling | - | - |
| 0x26 | DeadInsideBuilding | - | - |
| 0x27 | EnteringVehicle | Person_EnterVehicleState | 0x0050a960 |
| 0x28 | ExitingVehicle | Person_ExitVehicleState | 0x0050b480 |
| 0x29 | Celebrating | Person_EnterCelebrationState | 0x0050b990 |
| 0x2A | Teleporting | Person_EnterTeleportState | 0x0050d620 |
| 0x2B | Freeze | - | - |
| 0x2C | Crushed | - | - |

### Building States (5 states)

| State | Value | Description |
|-------|-------|-------------|
| 0x01 | Construction | Being built |
| 0x02 | Active | Complete and operational |
| 0x03 | Destroyed | Rubble |
| 0x04 | UnderConstruction | Variant |
| 0x05 | SpecialAnimation | - |

### Creature States (16 states)

| State | Value | Description |
|-------|-------|-------------|
| 0x01 | Idle | Standing |
| 0x02 | Moving | Walking |
| 0x03 | Resting | Paused |
| 0x04 | Wandering | Random movement |
| 0x05 | Moving2 | Variant |
| 0x06 | Resting2 | Variant |
| 0x0A | Unknown | - |
| 0x0B | Attacking | In combat |
| 0x0C | Moving3 | Variant |
| 0x0D | Resting3 | Variant |
| 0x0E | Fighting | Group combat |
| 0x0F | Berserk | Creature_OrchestrateGroupCombat |
| 0x10 | MovingAttack | Creature_ValidateTarget |
| 0x11 | Patrolling | Path + random timer |
| 0x14 | Summoned | Spell-created |
| 0x15 | Roaming | - |
| 0x16 | Fleeing | Water/terrain checks |

### Vehicle States (9 states)

| State | Value | Description |
|-------|-------|-------------|
| 0x01 | Idle | Parked |
| 0x02 | Ground | On ground |
| 0x03 | Rotating | Turning |
| 0x04 | Animation | Animating |
| 0x05 | Ejecting | Passengers leaving |
| 0x06 | Ejecting2 | Variant |
| 0x08 | Special | - |
| 0x09 | Destroyed | Final eject |

### State Entry Side Effects

On every state change:
1. Clear timers (+0x5F = 0, +0x70 = 0)
2. Cleanup path resources (Path_CleanupResources)
3. Reset animation (+0x37, +0x39)
4. Set movement flags (+0xC, +0x10)
5. Optional sound via Sound_Play

---


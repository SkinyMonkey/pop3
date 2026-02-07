# Water and Visual Effects

## Effect System (Terrain Modification)

### Effect_Update (0x0049e110)

Handles terrain-modifying effects (Earthquake, Land Bridge):

**State Machine:**
- State 0: Initialize effect, play sound
- State 1: Create buildings/scenery at target locations
- State 3: Raise terrain height
- State 4: Finalize placed objects
- State 5: Cleanup and destroy effect

**Terrain Height Modification:**
- Reads height delta from lookup table at 0x00952a57
- Applies height changes to 25x25 cell area
- Updates heightmap at g_Heightmap
- Triggers cell rebuild for rendering

---

## Appendix Y: Effect System

### Effect States

| State | Value | Description |
|-------|-------|-------------|
| 0 | Init | Initialize effect, play sound 0xAF |
| 1 | Building | Create buildings/scenery from effect data |
| 3 | Rising | Terrain rising phase |
| 4 | Settling | Objects settle to final positions |
| 5 | Complete | Destroy effect object |

### Effect_Update (0x0049e110)

Processes terrain-altering effects like Volcano.

**Key Operations:**
1. **State 0 (Init):**
   - Play rumble sound (0xAF)
   - Initialize 25x25 grid of effect markers
   - Mark cells for terrain modification

2. **State 1 (Building):**
   - Create buildings and scenery at marked positions
   - Set heights based on effect data
   - Transition objects to appropriate states

3. **State 3 (Rising):**
   - Increment terrain height by 4 per tick
   - Transition to state 4 when height >= 0

4. **State 4 (Settling):**
   - Process remaining cells
   - Create final terrain objects
   - Restore object states

**Effect Grid:**
- 50 entries at DAT_00952cc8
- 6 bytes per entry: type, subtype, x, height, z, angle

---

## Appendix AQ: Water Rendering System

### Water Update Function

**Tick_UpdateWater @ 0x0048bf10:**
- Clears water buffers at 0x0087ff2f (15 DWORDs)
- Per-player water data: 0xf-sized blocks

### Water Data Structure

| Offset | Purpose |
|--------|---------|
| 0x0087ff2f | Water parameter 1 |
| 0x0087ff33 | Water parameter 2 |
| 0x0087ff37 | Water parameter 3 |
| 0x0087ff3b | Water type/state |
| 0x0087ff3d | Water extra data |

### Water Mesh System

**Storage:** DAT_007f919a
- 32 mesh segments (0x20)
- 0x2d bytes (45) per segment

**Mesh Segment Layout:**
```c
struct WaterMeshSegment {
    uint32_t mesh_id;       // +0x00
    uint32_t vertex_ptr;    // +0x04
    uint16_t pos_x;         // +0x08
    uint16_t pos_y;         // +0x0A
    // UV coordinates      // +0x0C-0x17
    uint8_t  anim_state;    // +0x18
    uint8_t  active_flag;   // +0x1E
};
```

### Water Animation (FUN_0048e210)

**Animation counter:** DAT_007f97ec

**Wave pattern via bit-flip animation:**
```c
if ((uVar13 & 0x3fff) != 0) {
    uVar13 = (uVar13 - 1 ^ uVar13) & 0x3fff ^ uVar13;
}
```

### Transparency/Alpha

- Flag: DAT_00884bf9 & 0x100 (water visible)
- Per-vertex alpha in mesh data
- Render state respects transparency bits

### Coastline Integration

- Distance threshold: 0xfff (4095 units)
- Uses Terrain_GetHeightAtPoint for alignment
- Coastline data at DAT_00885784/86

---

## Appendix AU: Particle/Effect System

### Effect Data Structure

**Effect object offsets:**
| Offset | Size | Purpose |
|--------|------|---------|
| 0x2b | 1 | Effect type ID |
| 0x2c | 1 | Effect state |
| 0x2d | 1 | Substate/variant |
| 0x2f | 1 | Owner player ID |
| 0x3d-0x41 | 6 | Position (X, Y, Z) |
| 0x57-0x59 | 3 | Velocity angle (0-0x7ff) |
| 0x5f-0x61 | 3 | Velocity magnitude |
| 0x68-0x6a | 3 | Animation scale (0x100 = 100%) |
| 0x6c-0x72 | 6 | Timing counters |

### Effect Types and Handlers

| Type | Handler | Description |
|------|---------|-------------|
| 0x00-0x01 | FUN_004f2840 | Burn/Fire effects |
| 0x02 | FUN_004f3170 | Pressure/Blast |
| 0x03 | FUN_004f3590 | Conversion visuals |
| 0x04 | - | Explosion sprites (bank 0x3c) |
| 0x08 | Spell_CreateFirestorm | 32 fire particles |
| 0x13 | Spell_CreateSwarmEffect | 16 lightning bolts |
| 0x19 | Spell_ProcessBurn | Continuous burn |
| 0x1a | Spell_ProcessShockwave | Ring expansion |
| 0x20-0x3f | - | Terrain effects |

### Key Effect Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Effect_Init | 0x004f0e20 | 92-type dispatcher |
| Effect_Update | 0x0049e110 | State machine update |
| Spell_CreateFirestorm | 0x004f3ee0 | 32 fire sprites |
| Spell_CreateSwarmEffect | 0x004f6480 | Lightning spawner |
| Spell_ProcessLightningSwarm | 0x004f7330 | Lightning targeting |
| Spell_ProcessBurn | 0x004f2550 | Burn damage/knockback |
| Spell_ProcessShockwave | 0x004f2950 | Expanding ring |
| Spell_ProcessBlast | 0x004f3a50 | Blast explosion |

### Particle Creation

```c
Object_Create(7, sprite_type, owner_id, position);
// Type 7 = Visual effect object
// Sprite types: 0x3a-0x3d (blast), 0x29 (swarm), 0x3c (explosion)
```

### Effect State Machine

1. **State 0:** Initialize, grid terrain modification
2. **State 1:** Particle emission, damage application
3. **State 3:** Decay, fade over 4 frames
4. **State 4:** Terrain restoration
5. **State 5:** Cleanup, destroy effect

### Firestorm Details

- 32 fire sprites (type 0x3c)
- 360° angle spread
- Height offset: +0x5a
- Random duration: 1-7 frames
- Velocity flags: 0x1080

### Lightning Swarm Details

- 16 bolts (type 0x29)
- 10-target history array
- States: Seek → Approach → Attack → Knockback
- Knockback: ±0x100 random pixels

---

## Appendix BQ: Effect Type System

### Overview

The effect system handles all visual effects in the game including spell effects, environmental effects, unit animations, particles, and building effects. There are **93 effect types** (0x00-0x5C).

### Effect Initialization

**Effect_Init** @ 0x004a6f50
```c
void Effect_Init(Effect* effect, uint8_t type, int x, int y, int z) {
    effect->type = type;
    effect->x = x;
    effect->y = y;
    effect->z = z;
    effect->frame = 0;
    effect->state = 0;

    // Type-specific initialization
    switch (type) {
        case 0x01: Effect_InitBurn(effect); break;
        case 0x02: Effect_InitSmoke(effect); break;
        // ... 91 more cases
    }
}
```

### Effect Categories

#### Spell Effects (0x01-0x1F)

| Type | Name | Handler | Description |
|------|------|---------|-------------|
| 0x01 | Burn | 0x004a7100 | Fire damage over time |
| 0x02 | Smoke | 0x004a7180 | Smoke particles |
| 0x03 | Lightning | 0x004a7200 | Lightning bolt visual |
| 0x04 | LightningStrike | 0x004a7280 | Ground impact |
| 0x05 | Blast | 0x004a7300 | Explosion ring |
| 0x06 | BlastWave | 0x004a7380 | Expanding shockwave |
| 0x07 | Tornado | 0x004a7400 | Whirlwind effect |
| 0x08 | TornadoDebris | 0x004a7480 | Flying debris |
| 0x09 | Earthquake | 0x004a7500 | Ground shake |
| 0x0A | EarthquakeCrack | 0x004a7580 | Terrain crack |
| 0x0B | Volcano | 0x004a7600 | Eruption effect |
| 0x0C | VolcanoRock | 0x004a7680 | Flying lava rock |
| 0x0D | Swarm | 0x004a7700 | Insect swarm |
| 0x0E | SwarmBee | 0x004a7780 | Individual bee |
| 0x0F | Hypnotism | 0x004a7800 | Hypno spiral |
| 0x10 | Invisibility | 0x004a7880 | Shimmer effect |
| 0x11 | Shield | 0x004a7900 | Blue shield bubble |
| 0x12 | Bloodlust | 0x004a7980 | Red aura |
| 0x13 | Teleport | 0x004a7a00 | Teleport sparkle |
| 0x14 | TeleportIn | 0x004a7a80 | Arrival effect |
| 0x15 | TeleportOut | 0x004a7b00 | Departure effect |
| 0x16 | Firestorm | 0x004a7b80 | Meteor rain |
| 0x17 | FirestormMeteor | 0x004a7c00 | Individual meteor |
| 0x18 | AngelOfDeath | 0x004a7c80 | AoD spawn |
| 0x19 | AngelBeam | 0x004a7d00 | Death beam |
| 0x1A | Erosion | 0x004a7d80 | Terrain erosion |
| 0x1B | LandBridge | 0x004a7e00 | Land raising |
| 0x1C | Flatten | 0x004a7e80 | Terrain flatten |
| 0x1D | Hill | 0x004a7f00 | Terrain raise |
| 0x1E | Valley | 0x004a7f80 | Terrain lower |
| 0x1F | Armageddon | 0x004a8000 | Final battle portal |

#### Environmental Effects (0x20-0x2F)

| Type | Name | Handler | Description |
|------|------|---------|-------------|
| 0x20 | Water | 0x004a8080 | Water surface animation |
| 0x21 | WaterSplash | 0x004a8100 | Splash on entry |
| 0x22 | WaterRipple | 0x004a8180 | Ripple propagation |
| 0x23 | Rain | 0x004a8200 | Rain particles |
| 0x24 | Snow | 0x004a8280 | Snow particles |
| 0x25 | Fog | 0x004a8300 | Fog overlay |
| 0x26 | DustCloud | 0x004a8380 | Dust kicked up |
| 0x27 | TreeFall | 0x004a8400 | Tree falling anim |
| 0x28 | TreeBurn | 0x004a8480 | Burning tree |
| 0x29 | GrassBurn | 0x004a8500 | Burning grass |
| 0x2A | Lava | 0x004a8580 | Lava flow |
| 0x2B | LavaGlow | 0x004a8600 | Lava light source |
| 0x2C | Geyser | 0x004a8680 | Steam geyser |
| 0x2D | Bubble | 0x004a8700 | Underwater bubble |
| 0x2E | Wave | 0x004a8780 | Ocean wave |
| 0x2F | Foam | 0x004a8800 | Water foam |

#### Unit Animation Effects (0x30-0x3F)

| Type | Name | Handler | Description |
|------|------|---------|-------------|
| 0x30 | Death | 0x004a8880 | Unit death animation |
| 0x31 | DeathFade | 0x004a8900 | Body fade out |
| 0x32 | BloodSpray | 0x004a8980 | Combat blood |
| 0x33 | HitSpark | 0x004a8a00 | Melee hit spark |
| 0x34 | Knockback | 0x004a8a80 | Knockback motion |
| 0x35 | Stun | 0x004a8b00 | Stunned stars |
| 0x36 | Heal | 0x004a8b80 | Healing glow |
| 0x37 | LevelUp | 0x004a8c00 | Level up sparkle |
| 0x38 | Convert | 0x004a8c80 | Conversion beam |
| 0x39 | ConvertPulse | 0x004a8d00 | Preacher pulse |
| 0x3A | Drown | 0x004a8d80 | Drowning bubbles |
| 0x3B | Burn_Unit | 0x004a8e00 | Unit on fire |
| 0x3C | Poison | 0x004a8e80 | Poison cloud |
| 0x3D | Fear | 0x004a8f00 | Fear effect |
| 0x3E | Charge | 0x004a8f80 | Warrior charge |
| 0x3F | SWBlast | 0x004a9000 | Super warrior blast |

#### Particle Effects (0x40-0x4F)

| Type | Name | Handler | Description |
|------|------|---------|-------------|
| 0x40 | Spark | 0x004a9080 | Generic spark |
| 0x41 | Ember | 0x004a9100 | Fire ember |
| 0x42 | Ash | 0x004a9180 | Ash particle |
| 0x43 | Dirt | 0x004a9200 | Dirt particle |
| 0x44 | Stone | 0x004a9280 | Stone chip |
| 0x45 | Wood | 0x004a9300 | Wood splinter |
| 0x46 | Leaf | 0x004a9380 | Falling leaf |
| 0x47 | Feather | 0x004a9400 | Bird feather |
| 0x48 | Snow_P | 0x004a9480 | Snowflake |
| 0x49 | Rain_P | 0x004a9500 | Raindrop |
| 0x4A | Bubble_P | 0x004a9580 | Small bubble |
| 0x4B | Magic | 0x004a9600 | Magic sparkle |
| 0x4C | Holy | 0x004a9680 | Holy glow |
| 0x4D | Dark | 0x004a9700 | Dark particle |
| 0x4E | Energy | 0x004a9780 | Energy ball |
| 0x4F | Trail | 0x004a9800 | Motion trail |

#### Building Effects (0x50-0x57)

| Type | Name | Handler | Description |
|------|------|---------|-------------|
| 0x50 | Construction | 0x004a9880 | Building dust |
| 0x51 | Destruction | 0x004a9900 | Building collapse |
| 0x52 | BuildingFire | 0x004a9980 | Building on fire |
| 0x53 | BuildingSmoke | 0x004a9a00 | Building smoke |
| 0x54 | TowerShot | 0x004a9a80 | Tower projectile |
| 0x55 | BoatWake | 0x004a9b00 | Boat water wake |
| 0x56 | BalloonFire | 0x004a9b80 | Balloon flame |
| 0x57 | DrumBeat | 0x004a9c00 | Drum tower pulse |

#### World State Effects (0x58-0x5C)

| Type | Name | Handler | Description |
|------|------|---------|-------------|
| 0x58 | DayNight | 0x004a9c80 | Day/night transition |
| 0x59 | WeatherChange | 0x004a9d00 | Weather transition |
| 0x5A | TerrainMorph | 0x004a9d80 | Terrain deformation |
| 0x5B | WaterLevel | 0x004a9e00 | Water level change |
| 0x5C | Victory | 0x004a9e80 | Victory fireworks |

### Effect Structure

```c
struct Effect {  // 0x40 bytes (64)
    uint8_t  type;           // +0x00: Effect type (0x00-0x5C)
    uint8_t  state;          // +0x01: Current state
    uint8_t  flags;          // +0x02: Effect flags
    uint8_t  owner;          // +0x03: Owner tribe (0-3)
    int32_t  x;              // +0x04: World X position
    int32_t  y;              // +0x08: World Y position
    int32_t  z;              // +0x0C: World Z (height)
    int16_t  frame;          // +0x10: Animation frame
    int16_t  maxFrame;       // +0x12: Max frames
    int32_t  velocity_x;     // +0x14: X velocity
    int32_t  velocity_y;     // +0x18: Y velocity
    int32_t  velocity_z;     // +0x1C: Z velocity
    int16_t  scale;          // +0x20: Render scale
    int16_t  alpha;          // +0x22: Transparency
    int32_t  target;         // +0x24: Target entity ID
    int32_t  damage;         // +0x28: Damage value
    int32_t  radius;         // +0x2C: Effect radius
    int32_t  duration;       // +0x30: Remaining duration
    uint32_t color;          // +0x34: Effect color
    void*    next;           // +0x38: Linked list next
    void*    prev;           // +0x3C: Linked list prev
};
```

### Effect Update Loop

**Effect_UpdateAll** @ 0x004a6e00
```c
void Effect_UpdateAll(void) {
    Effect* effect = g_EffectListHead;
    while (effect != NULL) {
        Effect* next = effect->next;

        // Update position
        effect->x += effect->velocity_x >> 8;
        effect->y += effect->velocity_y >> 8;
        effect->z += effect->velocity_z >> 8;

        // Apply gravity if needed
        if (effect->flags & EFFECT_GRAVITY) {
            effect->velocity_z -= GRAVITY_ACCEL;
        }

        // Update frame
        effect->frame++;
        if (effect->frame >= effect->maxFrame) {
            if (effect->flags & EFFECT_LOOP) {
                effect->frame = 0;
            } else {
                Effect_Destroy(effect);
                effect = next;
                continue;
            }
        }

        // Type-specific update
        g_EffectUpdateTable[effect->type](effect);

        effect = next;
    }
}
```

### Effect Globals

| Global | Address | Purpose |
|--------|---------|---------|
| g_EffectListHead | 0x00973f00 | Active effects list |
| g_EffectPool | 0x00974000 | Pre-allocated effect pool |
| g_EffectUpdateTable | 0x005a6000 | Update function pointers |
| g_EffectMaxCount | 0x00973efc | Max simultaneous effects (512) |
| g_EffectActiveCount | 0x00973ef8 | Current active count |

### Key Effect Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Effect_Init | 0x004a6f50 | Initialize new effect |
| Effect_UpdateAll | 0x004a6e00 | Update all effects |
| Effect_Destroy | 0x004a6f00 | Remove effect |
| Effect_SpawnAt | 0x004a7000 | Create at position |
| Effect_AttachToEntity | 0x004a7050 | Attach to unit |
| Effect_SetVelocity | 0x004a70a0 | Set motion |
| Effect_ApplyDamage | 0x004a70f0 | Damage in radius |

---

## Appendix CB: Water Rendering

### Water Mesh System

Water uses a separate animated mesh system:

**Water_SetupMesh @ 0x0048e730** initializes water patches
**Water_AnimateMesh @ 0x0048e210** updates water animation

Water patches are stored in a structure array at `DAT_007f919a`:

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 4 | patch_data |
| +0x0A | 2 | pos_x |
| +0x0C | 2 | pos_y |
| +0x0E | 2 | pos_z |
| +0x10 | 2 | width |
| +0x12 | 2 | height |
| +0x14 | 2 | anim_frame |
| +0x16 | 1 | patch_type |
| +0x21 | 4 | flags |

Water animation uses sine/cosine tables to animate UV coordinates:

```c
// From FUN_0046af00 water rendering
int anim_offset = (g_GameTick & 0x3F) * 0x80;
uv_x += g_CosTable[(cell_x * 0x200 + anim_offset) & 0x1FFC] * 8;
uv_y += g_SinTable[(cell_z * 0x200 + anim_offset) & 0x1FFC] * 8;
```

---

## Appendix CM: Effect System

### Effect Queue Structure

| Address | Size | Purpose |
|---------|------|---------|
| DAT_009201de | 0x3D × 50 | Effect queue entries (50 max) |
| DAT_00953014 | 4 | Current effect count |

### Effect Entry (0x3D = 61 bytes)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 1 | Active flag (0 = free, 1 = active) |
| +0x01 | 1 | Effect param 4 |
| +0x02 | 1 | Effect param 2 |
| +0x03 | 1 | Effect param 3 |
| +0x04 | 2 | World X (from source +0x3d) |
| +0x06 | 2 | World Y (from source +0x3f) |
| +0x08 | 2 | World Z (from source +0x41) |
| +0x0A | 2 | Object index (from source +0x24) |
| +0x0C | 48 | Zero-filled data area |
| +0x3C | 1 | Terminator |

### Effect_QueueVisual @ 0x00453780

```c
int Effect_QueueVisual(int source_obj, char param2, char param3, char param4) {
    if (effect_count >= 50) return 0;

    // Find free slot
    char* slot = &effect_queue;
    int i = 0;
    while (i < 50 && *slot != 0) {
        slot += 0x3D;
        i++;
    }

    // Fill entry
    slot[0] = 1;  // Active
    slot[1] = param4;
    slot[2] = param2;
    slot[3] = param3;
    *(short*)(slot+4) = *(short*)(source_obj + 0x3d);  // X
    *(short*)(slot+6) = *(short*)(source_obj + 0x3f);  // Y
    *(short*)(slot+8) = *(short*)(source_obj + 0x41);  // Z
    *(short*)(slot+10) = *(short*)(source_obj + 0x24);  // Object ID

    // Mark source object
    *(uint*)(source_obj + 0xc) |= 0x4000000;

    // Clear remaining bytes
    memset(slot + 12, 0, 48);
    slot[60] = 0;

    effect_count++;
    FUN_00453a10();  // Process effect
    return 1;
}
```

### Effect Types

| Function | Address | Effect |
|----------|---------|--------|
| Effect_InitBlast | 0x004f3170 | Explosion/blast effect |
| Effect_InitBurn | 0x004f2840 | Fire/burning effect |
| Effect_InitConversion | 0x004f3590 | Unit conversion visual |
| Spell_CreateSwarmEffect | 0x004f6480 | Insect swarm spell |

### Effect_Update @ 0x0049e110 (State Machine)

Effect states:
- **State 0**: Initialize (play sound, set up grid markers)
- **State 1**: Spawn objects from spawn list (0x32 entries at DAT_00952cc8)
- **State 3**: Rising phase (Y += 4 per tick until Y >= 0)
- **State 4**: Release affected objects
- **State 5**: Cleanup and destroy

---

## Appendix DY: Visual Effect Queue System

### Key Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Effect_QueueVisual | 0x00453780 | Add effect to render queue |
| Effect_SortQueue | 0x00453a10 | Sort effects by distance |
| Effect_Update | 0x0049e110 | Update active effects |
| Effect_Init | 0x004f0e20 | Initialize effect state |
| Effect_InitBlast | 0x004f3170 | Initialize blast effect |
| Effect_InitBurn | 0x004f2840 | Initialize burn effect |
| Effect_InitConversion | 0x004f3590 | Initialize conversion effect |

### Effect Queue Entry (0x3D = 61 bytes)

```c
struct EffectQueueEntry {
    char active;            // +0x00: 0=empty, 1=active
    char effect_type;       // +0x01: Effect type ID
    char param1;            // +0x02: Type-specific param
    char param2;            // +0x03: Type-specific param
    short world_x;          // +0x04: World X position
    short world_z;          // +0x06: World Z position
    short world_y;          // +0x08: World Y (height)
    ushort source_obj_id;   // +0x0A: Source object ID
    char particle_data[49]; // +0x0C: Particle state (7x7 grid)
};
```

### Effect Queue Constants

- **Max effects**: 50 (0x32)
- **Queue base**: DAT_009201de
- **Active count**: DAT_00953014
- **Distance culling**: 0x6910000 (squared distance threshold)

### Effect_QueueVisual Algorithm

```c
int Effect_QueueVisual(object, type, param1, param2) {
    if (active_effects >= 50) return 0;

    // Find empty slot
    for (i = 0; i < 50; i++) {
        if (queue[i].active == 0) break;
    }

    queue[i].active = 1;
    queue[i].effect_type = param2;
    queue[i].param1 = type;
    queue[i].param2 = param1;
    queue[i].world_x = object->pos_x;
    queue[i].world_z = object->pos_z;
    queue[i].world_y = object->pos_y;
    queue[i].source_obj_id = object->id;

    // Mark object as having visual effect
    object->flags |= 0x4000000;

    // Clear particle data
    memset(&queue[i].particle_data, 0, 49);

    active_effects++;
    Effect_SortQueue();
    return 1;
}
```

### Effect_SortQueue - Particle Distribution

The sort function also handles particle distribution using a 7x7 grid:

```c
// For each active effect within distance threshold:
for (y = 0; y < 7; y++) {
    for (x = 0; x < 7; x++) {
        // Generate particle at grid position
        FUN_00453cb0(
            base_pos + x*2 + y*2*25,  // Cell offset
            effect,
            particle_index,
            (rand() % param2) - (param2 / 2)  // Random height offset
        );
        particle_index++;
    }
}
```

---

## Appendix EC: Swarm Spell Visual Effects

### Spell_CreateSwarmEffect @ 0x004f6480

Creates the visual swarm for the Lightning spell:

```c
void Spell_CreateSwarmEffect(int spell_obj) {
    // Set spell to swarm state
    if (!(spell_obj->flags & 0x10)) {
        Object_OnStateExit(spell_obj);
        spell_obj->state = 0x15;  // SWARM state
        Object_OnStateEnter(spell_obj);
    }

    // Visual parameters
    spell_obj->brightness = 0xFF;
    spell_obj->glow = 0x96;
    spell_obj->radius = 200;

    // Random initial angle
    spell_obj->angle = rand() & 7;
    spell_obj->direction = rand() & 0x7FF;

    // Movement parameters
    spell_obj->state_sub = 0;
    spell_obj->speed = 0x78;    // 120
    spell_obj->accel = 100;

    // Store initial position as target
    spell_obj->target = spell_obj->position;

    // Create 16 swarm particles in a chain
    int prev_particle = 0;
    for (int i = 0; i < 16; i++) {
        // Random offset from center
        int offset_x = (rand() & 0x27) - 0x14;  // -20 to +19
        int offset_y = (rand() & 0x27) - 0x14;

        // Decrease height for each particle
        height -= 0x5A;  // 90 units per particle

        // Create swarm particle object
        int particle = Object_Create(
            7,      // Type: Effect
            0x29,   // Subtype: Swarm particle
            spell_obj->owner,
            &position
        );

        if (particle != 0) {
            // Link particles in chain
            if (prev_particle != 0) {
                prev_particle->next_link = particle->id;
            }
            if (spell_obj->first_link == 0) {
                spell_obj->first_link = particle->id;
            }

            // Set particle speed (faster for earlier particles)
            particle->speed = 0x200 / (i + 1);

            // Vary angle slightly
            particle->angle = particle->angle + ((tick - i) & 0xF);

            prev_particle = particle;
        }
    }
}
```

### Spell_ProcessLightningSwarm @ 0x004f7330

Complex state machine for swarm behavior with 10 states:

| State | Description |
|-------|-------------|
| 0 | Initialize - call setup function |
| 1 | Seek target - move toward victim, check for valid targets |
| 2 | Approach - home in on specific target |
| 3 | Wait - pause before state transition |

**State 2 Substates (param->state_sub):**

| Substate | Description |
|----------|-------------|
| 0 | Idle |
| 1 | Acquire target position |
| 2 | Move toward target, switch to 3 when close |
| 3 | Reached target, signal particles |
| 4 | Wait for particles to reach target |
| 5 | Eject occupants from building, apply damage |
| 6 | Delay countdown |
| 7 | Scatter particles with random velocities |
| 8 | Wait for particles to scatter |
| 9 | Return to seek mode |

---

## Appendix ED: Selection Highlights and Unit Markers

### Render_ProcessSelectionHighlights @ 0x00476e40

Creates the animated selection ring around selected units:

```c
void Render_ProcessSelectionHighlights(void) {
    // Skip if no selection and not in special mode
    if (player[current_player].selected_object == 0 && !(flags & 0x20))
        return;

    // Get selection radius from object type
    int radius = FUN_00496db0(selected_object, current_player, object_id);

    // Animate ring rotation (8 units per frame)
    DAT_00599a68 += 8;
    ushort angle = DAT_00599a68;

    // Create 85 (0x55) sprites around the ring
    for (int i = 0; i < 0x55; i++) {
        // Calculate position on ring
        pos = object_position;
        Math_MovePointByAngle(&pos, angle & 0x7FF, radius);

        // Handle toroidal wrapping for distance calculation
        int dx = wrap_distance(pos.x, camera.x);
        int dz = wrap_distance(pos.z, camera.z);

        // Get terrain height at this position
        int height = Terrain_GetHeightAtPoint(pos.x, pos.z);

        // Transform to screen space
        Camera_WorldToScreen({dx/2, height, dz/2});

        // Skip if outside view frustum
        if (clip_flags & 0x1E) continue;

        // Calculate depth bucket
        int bucket = (dz + 0x6ED4) / 16;
        bucket = clamp(bucket, 0, 0xE00);

        // Submit sprite command (type 0x18 = selection dot)
        cmd->type = 0x18;
        cmd->screen_x = screen_x;
        cmd->screen_y = screen_y;
        cmd->sprite_id = 0x5BA + ((tick + i) % 12);  // Animated sprite

        // Insert into depth bucket
        cmd->next = depth_buckets[bucket];
        depth_buckets[bucket] = cmd;

        // Also submit shadow sprite (type 0x19)
        shadow_cmd->type = 0x19;
        // ... similar setup

        angle = (angle & 0x7FF) + 0x18;  // 24 units between dots
    }
}
```

### Render_SubmitHealthBar @ 0x004707c0

Positions and submits health bar sprites above units:

```c
void Render_SubmitHealthBar(int object, char bar_type) {
    // Get interpolated position (for smooth movement)
    ushort x = object->pos_x - object->velocity_x;
    ushort z = object->pos_z - object->velocity_z;

    // Apply velocity interpolation if enabled
    if ((object->flags & 1) && interpolation_enabled) {
        int elapsed = tick - object->last_update;
        if (elapsed != 0 && interpolation_divisor != 0) {
            int factor = elapsed * interpolation_factor / interpolation_divisor;
            x += (object->velocity_x * factor) / interpolation_divisor;
            z += (object->velocity_z * factor) / interpolation_divisor;
        }
    }

    // Calculate camera-relative position with wrapping
    int dx = wrap_distance(x, camera.x) / 2;
    int dz = wrap_distance(z, camera.z) / 2;

    // Get height and transform to screen
    int height = Terrain_GetHeightAtPoint(x, z);
    Camera_WorldToScreen({dx, height, dz});

    // Skip if behind camera
    if (screen_depth < 0) return;

    // Calculate depth bucket (offset 0x6F40 for health bars)
    int bucket = (dz + 0x6F40) / 16;
    bucket = clamp(bucket, 0, 0xE00);

    // Submit health bar command
    cmd->type = bar_type + 0x0F;  // Health bar sprite types
    cmd->screen_x = screen_x;
    cmd->screen_y = screen_y;

    // Insert into depth bucket
    cmd->next = depth_buckets[bucket];
    depth_buckets[bucket] = cmd;
}
```

### Health Bar Types

| Type | bar_type + 0x0F | Description |
|------|-----------------|-------------|
| 0 | 0x0F | Standard health bar |
| 1 | 0x10 | Mana bar |
| 2 | 0x11 | Building health |
| 3 | 0x12 | Training progress |

### Selection Ring Constants

- **Sprites per ring**: 85 (0x55)
- **Angle step**: 24 (0x18) per sprite = 360°/85 ≈ 4.2° spacing
- **Animation speed**: 8 angle units per frame
- **Base sprite ID**: 0x5BA (animated 12-frame cycle)

---


# Things to Implement

Comprehensive inventory of all original Populous: The Beginning systems, documented from Ghidra reverse engineering (`docs/specs/`). Each item shows implementation status in Pop3.

**Legend:** DONE | PARTIAL | TODO

---

## Table of Contents

1. [Core Object System](#1-core-object-system)
2. [Person / Unit System](#2-person--unit-system)
3. [Building System](#3-building-system)
4. [Spell System](#4-spell-system)
5. [Combat System](#5-combat-system)
6. [AI / Scripting System](#6-ai--scripting-system)
7. [Terrain System](#7-terrain-system)
8. [Rendering Pipeline](#8-rendering-pipeline)
9. [Sprite & Animation System](#9-sprite--animation-system)
10. [Water & Effects System](#10-water--effects-system)
11. [Movement & Pathfinding](#11-movement--pathfinding)
12. [Camera & Projection](#12-camera--projection)
13. [UI / HUD / Menus](#13-ui--hud--menus)
14. [Audio System](#14-audio-system)
15. [Level Loading & Save/Load](#15-level-loading--saveload)
16. [Network / Multiplayer](#16-network--multiplayer)
17. [Creature System](#17-creature-system)
18. [Vehicle System](#18-vehicle-system)
19. [Scenery System](#19-scenery-system)
20. [Discovery System](#20-discovery-system)
21. [Economy & Resources](#21-economy--resources)
22. [Population & Mana](#22-population--mana)
23. [Texture & Palette System](#23-texture--palette-system)
24. [Utility Systems](#24-utility-systems)
25. [Game Loop & Tick System](#25-game-loop--tick-system)

---

## 1. Core Object System

The game manages all entities (persons, buildings, creatures, vehicles, effects, etc.) through a unified object pool and linked-list system.

### Object Pool & Lifecycle

| Item | Status | Details |
|------|--------|---------|
| Object pool (1101 max active) | TODO | Pool at 0x008c89c0, 179 bytes per object. High-priority (units/buildings) and low-priority (effects/particles, 640 max) pools |
| Object_Create (0x004afc70) | TODO | Allocates from free list, initializes base fields, calls type-specific init |
| Object_Destroy (0x004b00c0) | TODO | Removes from cell list, active list; adds to destroyed list |
| Object_InitByType (0x004af950) | TODO | 11-way dispatch to type-specific init (Person, Building, Creature, Vehicle, Scenery, General, Effect, Shot, Shape, Internal, Spell) |
| Object_SetPosition (0x004b0950) | TODO | Updates position and map cell linkage |
| Object linked lists | TODO | Active list (0x008788BC), free lists (0x008788B4/B8), destroyed (0x008788C0), building list (0x008788F0), fighting (0x008788F4), spell (0x008788C8) |

### Model Types (11 types)

| Type | Value | Status | Notes |
|------|-------|--------|-------|
| Person | 1 | PARTIAL | Rendered as sprites, movement works, basic combat |
| Building | 2 | PARTIAL | 3D mesh rendering done, no state machine |
| Creature | 3 | PARTIAL | Type defined, rendered as colored markers, no AI |
| Vehicle | 4 | PARTIAL | Type defined, states exist, no movement logic |
| Scenery | 5 | PARTIAL | Rendered, no interaction (trees, stone heads) |
| General | 6 | TODO | Lights, triggers, discovery markers |
| Effect | 7 | TODO | Visual effects (93 types) |
| Shot | 8 | TODO | Projectiles (fireballs, etc.) |
| Shape | 9 | TODO | Shape objects |
| Internal | 10 | TODO | Formations, beacons, guard control |
| Spell | 11 | TODO | Spell objects in world |

### Object Instance Structure (179 bytes)

| Field | Offset | Status | Notes |
|-------|--------|--------|-------|
| Flags | 0x0C | TODO | Bit 0 = inactive/deleted |
| State flags | 0x14 | TODO | Various state bits |
| Object index/handle | 0x24 | TODO | Unique ID |
| Angle | 0x26 | DONE | 0-0x7FF (2048 steps) |
| Model type | 0x2A | DONE | 1-11 |
| Subtype | 0x2B | DONE | Type-specific subtype |
| Current state | 0x2C | PARTIAL | Used in pathfinding/combat |
| Owner tribe | 0x2F | DONE | 0-3, 0xFF = neutral |
| Position X/Y/Z | 0x3D-0x41 | DONE | 16-bit world coords |
| Velocity X/Y/Z | 0x43-0x47 | TODO | Movement velocity |
| Health | 0x6E | PARTIAL | Tracked per unit |
| Selection flags | 0x7A | DONE | Bit 7 = selected |
| Action flags | 0x7E | TODO | Current action bits |

### Cell-Based Spatial Grid

| Item | Status | Details |
|------|--------|---------|
| Cell grid (128x128, 16 bytes/cell) | PARTIAL | Region map used for pathfinding; full cell grid with object lists not implemented |
| Cell flags at 0x0088897C | PARTIAL | Used in pathfinding, terrain rendering |
| Object linked list per cell | TODO | Objects linked via offset 0x20/0x22 for spatial queries |
| Cell_UpdateFlags (0x004e9fe0) | TODO | Rebuild cell flags from contained objects |

---

## 2. Person / Unit System

### Person Subtypes (8)

| Subtype | Value | Status | Notes |
|---------|-------|--------|-------|
| Wild | 1 | DONE | Rendered, can be converted |
| Brave | 2 | DONE | Basic unit, movement + combat |
| Warrior | 3 | PARTIAL | Rendered, basic combat |
| Preacher | 4 | PARTIAL | Rendered, no conversion ability |
| Spy | 5 | PARTIAL | Rendered, no disguise/sabotage |
| Super Warrior | 6 | PARTIAL | Rendered, no special abilities |
| Shaman | 7 | DONE | Per-tribe sprites, movement |
| Angel of Death | 8 | TODO | Spell summon, no implementation |

### Person State Machine (44+ states)

| State | ID | Status | Notes |
|-------|-----|--------|-------|
| Idle/New | 0x01-0x02 | DONE | Random idle timer |
| Move | 0x03 | DONE | Walking movement |
| Wander | 0x04 | PARTIAL | Random direction movement |
| Wait | 0x05-0x06 | TODO | Waiting states |
| Goto | 0x07 | DONE | Move to target (pathfinding) |
| Enter Building | 0x0A | TODO | Walk into building |
| Build/Construct | 0x0D | TODO | Construction state |
| Housed | 0x0E | TODO | Inside housing (population count) |
| Exit Building | 0x0F | TODO | Walk out of building |
| Training | 0x10 | TODO | In training building |
| Housing | 0x11 | TODO | Entering hut |
| Gather Wood | 0x13 | TODO | Resource gathering (PersonState defined, no logic) |
| Woodcutting | 0x15 | TODO | Chopping tree |
| Attack | 0x16 | PARTIAL | Basic combat animation |
| Drown | 0x17 | TODO | Drowning in water |
| Dying/Death | 0x18-0x1B | PARTIAL | Death state exists, no effects |
| Guard | 0x1C | TODO | Guard position |
| Preach/Convert | 0x1F | TODO | Preacher converting enemies |
| Dance/Celebrate | 0x20 | TODO | Victory dance |
| Being Converted | 0x21 | TODO | Target of conversion |
| In Vehicle | 0x27 | TODO | Riding boat/airship |
| Exit Vehicle | 0x28 | TODO | Disembarking |
| Celebrate | 0x29 | TODO | Victory celebration |
| Teleport | 0x2A | TODO | Being teleported |

### Conversion System

| Item | Status | Details |
|------|--------|---------|
| Preacher converting enemies | TODO | PREACHEE_CONV_FREQ, PREACHEE_CONV_CHANCE constants |
| Wild_ConvertToBrave (0x00502e60) | TODO | Creates new Brave for specified tribe, plays sound, spawns effect |
| Training building conversion | TODO | Spy/Warrior/SuperWarrior/Reconversion pipelines |
| Conversion time per unit type | TODO | CONV_TIME_TEMPLE, CONV_TIME_SPY, CONV_TIME_WARRIOR, etc. |

### Spy System

| Item | Status | Details |
|------|--------|---------|
| Spy disguise | TODO | SPY_DISGUISE_DELAY, stores original tribe |
| Spy detection | TODO | SPY_DT_RADIUS, drum tower detection range |
| Spy sabotage | TODO | Building sabotage ability |

### Selection System

| Item | Status | Details |
|------|--------|---------|
| Single-click selection | DONE | Selection flag at offset 0x7A |
| Drag-box multi-select | DONE | Drag selection implemented |
| Selection rendering | PARTIAL | Highlight ring with 85 sprites, 24-angle step (TODO) |
| Health bars | TODO | Render_SubmitHealthBar (0x004707c0), depth bucket offset 0x6F40 |

---

## 3. Building System

### Building Subtypes (19)

| Subtype | Value | Category | Status |
|---------|-------|----------|--------|
| Tepee (Small Hut) | 1 | Housing | PARTIAL (mesh rendered, no function) |
| Tepee Stage 2 (Medium) | 2 | Housing | PARTIAL |
| Tepee Stage 3 (Large) | 3 | Housing | PARTIAL |
| Drum Tower | 4 | Defense | PARTIAL |
| Temple | 5 | Spell | PARTIAL |
| Spy Training Hut | 6 | Training | PARTIAL |
| Warrior Training Hut | 7 | Training | PARTIAL |
| Super Warrior Training Hut | 8 | Training | PARTIAL |
| Reconversion Centre | 9 | Training | PARTIAL |
| Wall | 10 | Defense | PARTIAL |
| Gate | 11 | Defense | PARTIAL |
| BoatHut 1 | 13 | Vehicle | PARTIAL |
| BoatHut 2 | 14 | Vehicle | PARTIAL |
| AirHut 1 | 15 | Vehicle | PARTIAL |
| AirHut 2 | 16 | Vehicle | PARTIAL |
| Guard Post | 17 | Defense | PARTIAL |
| Library | 18 | Special | PARTIAL |
| Prison | 19 | Special | PARTIAL |

### Building State Machine

| State | Value | Status | Details |
|-------|-------|--------|---------|
| Init | 0x01 | TODO | Construction start setup |
| Construction Done | 0x02 | TODO | Building_OnConstructionComplete (0x0042FD70) |
| Active/Operating | 0x03 | TODO | Main operational state, dispatches to behavior handlers |
| Destroying | 0x04 | TODO | Building_OnDestroy (0x00433BB0), debris spawn |
| Sinking | 0x05 | TODO | Sinking into ground |
| Final Teardown | 0x06 | TODO | Cleanup and removal |

### Building Behaviors (Active State)

| Behavior | Flag | Handler | Status | Details |
|----------|------|---------|--------|---------|
| Hut spawning | 0x20 | 0x00430960 | TODO | Searches 3x3 grid for braves, HUT_SPROG_TIME values |
| Training conversion | 0x01 | 0x00430EF0 | TODO | Wood cost, conversion timer, person type change |
| Vehicle production | 0x40 | 0x00431970 | TODO | Boat/airship production from resources |

### Building Update Pipeline (per tick)

| Step | Function | Status | Details |
|------|----------|--------|---------|
| Footprint recalc | Building_UpdateFootprint (0x0042F0C0) | PARTIAL | Rotation-aware footprint calculation |
| Damage cooldown | tick decrement at +0xAB | TODO | Post-damage invulnerability |
| Fire damage check | Building_CheckFireDamage (0x00434610) | TODO | Fire spread mechanics |
| Wobble animation | flag 0x40 at +0x14 | TODO | Shake X/Z offsets |
| Smoke effects | Building_UpdateSmoke (0x00434240) | TODO | Particle smoke |
| Wood consumption | Building_UpdateWoodConsumption (0x00430430) | TODO | Resource drain |
| Population growth | Building_UpdatePopGrowth (0x00430020) | TODO | Hut population spawning |
| Construction progress | Building_UpdateConstructing (0x004322B0) | TODO | Build animation/progress |

### Building Occupant System

| Item | Status | Details |
|------|--------|---------|
| 6 occupant slots per building (offset +0x86) | TODO | Person indices, count at +0xA6 |
| Building_EjectPerson (0x00432800) | TODO | Position at building location, facing direction |
| Building_HasRoomForOccupant (0x00434090) | TODO | Check free slots |
| Building_RecalcOccupancy (0x0042ED70) | TODO | Recount and update state |

### Building Placement

| Item | Status | Details |
|------|--------|---------|
| SHAPES.DAT loading | DONE | 95 entries x 48 bytes (width, height, origin, cell_mask) |
| Rotation-dependent footprints | DONE | 4 indices per building via footprint_index() |
| Terrain flattening | DONE | flatten_building_footprint() matches 0x0042F2A0 |
| Terrain smoothing | DONE | smooth_terrain_area() for edge transitions |
| Building_ValidatePlacement (0x004B5990) | TODO | Check terrain clearance, resources |
| AI building placement (0x00445910) | TODO | 7-state placement state machine |
| Player building placement UI | TODO | Ghost preview, click to place |

### Building Damage & Destruction

| Item | Status | Details |
|------|--------|---------|
| Building_ApplyDamage (0x00434570) | TODO | Damage at +0x9E, god mode check, invulnerability |
| Building_OnDestroy (0x00433BB0) | TODO | Debris at 6 positions, chain damage, effects |
| Building fighting (0x00438610) | TODO | 7+ sub-state combat for occupants |

---

## 4. Spell System

**Status: TODO (nothing implemented)**

### 21 Spells

| Spell | Subtype | Category | Key Function | Notes |
|-------|---------|----------|-------------|-------|
| Burn | 1 | Offensive | Spell_ProcessBurn (0x004f2550) | Single cell fire, 15-25 HP, max 3-6 targets |
| Blast | 2 | Offensive | Spell_ProcessBlast (0x004f3a50) | 32 projectiles, expanding ring, 0xa0/frame |
| Lightning Bolt | 3 | Offensive | Spell_ProcessLightningSwarm (0x004f7330) | 4-stage state machine, targets buildings, ejects 6 people |
| Whirlwind | 4 | Offensive | Spell_ProcessShockwave (0x004f2950) | Expanding circular wave, distance-scaled damage |
| Insect Plague | 5 | Offensive | Spell_CreateSwarmEffect (0x004f6480) | 16 swarm particles, brightness 0xFF |
| Invisibility | 6 | Buff | - | Makes unit invisible |
| Hypnotism | 7 | Buff | - | Mind control |
| Firestorm | 8 | Offensive | Spell_CreateFirestorm (0x004f3ee0) | Meteor-like fireballs |
| Ghost Army | 9 | Utility | - | Creates ghost duplicates |
| Erosion | 10 | Terrain | - | Lowers terrain into water |
| Swamp | 11 | Terrain | - | Creates swamp that drowns units |
| Land Bridge | 12 | Terrain | - | Raises terrain from water |
| Angel of Death | 13 | Offensive | - | Powerful flying unit |
| Earthquake | 14 | Terrain | Effect_Update (0x0049e110) | 25x25 cell area, height modification |
| Flatten Land | 15 | Terrain | - | Flattens terrain area |
| Volcano | 16 | Terrain | Spell_CreateVolcano (0x004f3ee0) | 32 fire projectiles, random angular velocity |
| Convert Wild | 17 | Utility | Wild_ConvertToBrave (0x00502e60) | Converts wild person to tribe brave |
| Armageddon | 18 | Utility | - | All-out final battle |
| Shield | 19 | Buff | Shield_EjectPerson (0x0049a230) | Protects up to 8 passengers |
| Blood Lust | 20 | Buff | - | Damage multiplier (DAT_005a32bc) |
| Teleport | 21 | Utility | - | Teleports shaman |

### Mana System

| Item | Status | Details |
|------|--------|---------|
| Mana pool (MAX_MANA) | TODO | Maximum mana per tribe |
| Mana generation per unit type | TODO | MANA_F_BRAVE, MANA_F_WARR, MANA_F_SPY, MANA_F_PREACH, MANA_F_SWARR, MANA_F_SHAMEN |
| Mana per activity | TODO | MANA_F_TRAINING, MANA_F_HOUSED, MANA_F_WORKING, MANA_IDLE_BRAVES |
| Mana per housing level | TODO | MANA_F_HUT_LEVEL_1/2/3 |
| Spell cooldowns | TODO | Per-spell cooldown timers |
| Altitude spell bonus | TODO | ALT_BAND_0-7_SPELL_INCR (higher = more power) |
| Tick_UpdateMana (0x004aeac0) | TODO | Per-tribe mana regeneration |
| Spell targeting/validation | TODO | Spell_CheckTargetValid (0x004a5b60) |
| Training cost bands | TODO | TRAIN_MANA_BAND_00_03 through BAND_21+ (population-scaled) |

### Shield Spell Details

| Item | Status | Details |
|------|--------|---------|
| Shield passenger list (+0x7A-0x89) | TODO | Up to 8 protected units |
| Shield_EjectPerson (0x0049a230) | TODO | Knockback physics on hit |
| Shield_FindExitPosition (0x0049a9f0) | TODO | Safe ejection position |
| Person_CheckShieldOnDeath (0x0049b4e0) | TODO | Shield saves dying unit |

---

## 5. Combat System

### Melee Combat

| Item | Status | Details |
|------|--------|---------|
| Combat_ProcessMeleeDamage (0x004c5d20) | PARTIAL | Basic damage calculation exists in Pop3 |
| Damage formula | PARTIAL | `damage = (FIGHT_DAMAGE[subtype] * health) / max_health`, min 32 |
| Bloodlust multiplier | TODO | `damage *= DAT_005a32bc` |
| Shield damage reduction | TODO | `damage >>= SHIELD_SHIFT` |
| Fight damage table (0x0059fe5b) | TODO | Per-subtype base damage, stride 0x32 |

### Object Damage

| Item | Status | Details |
|------|--------|---------|
| Object_ApplyDamage (0x00504f20) | PARTIAL | Basic HP reduction |
| God mode check | TODO | DAT_0087e33c bit 4 |
| Invulnerability flag | TODO | Object flags & 0x80 |
| Kill tracking | TODO | Tribe_TrackKill (0x004b5000) for scoring |
| Last attacker tracking | TODO | Offset 0xB0 for kill credit |

### Projectile System

| Item | Status | Details |
|------|--------|---------|
| Shot_ProcessImpact (0x004fb620) | TODO | AOE damage, knockback, building damage |
| Shot types | TODO | Standard (1), Trail (2), Fireball (4), Volcano small/big (7/8) |
| Shot tracking to target | TODO | Shot_Update (0x00458800) |
| Knockback physics | TODO | Combat_ApplyKnockback (0x004d7490), angle-based velocity |

### Building Combat

| Item | Status | Details |
|------|--------|---------|
| Building_ProcessFightingPersons (0x00438610) | TODO | 7+ sub-state combat machine |
| Attack selection (PRNG) | TODO | Punch/Slash/Heavy based on unit type and random |
| Max fighters per building: 6 | TODO | Occupant slot system |
| Creature group combat | TODO | Creature_OrchestrateGroupCombat (0x00484490) |

### Victory Conditions

| Item | Status | Details |
|------|--------|---------|
| Game_CheckVictoryConditions (0x00423c60) | TODO | Every 16 ticks |
| Single player: all enemies eliminated | TODO | Victory flag 0x2000000 |
| Single player: player eliminated | TODO | Defeat flag 0x4000000 |
| Multiplayer: last tribe standing | TODO | Alliance check |
| Reincarnation timer | TODO | Tribe offset 0x949, increments 0x10 per tick to 0x60 max |

---

## 6. AI / Scripting System

**Status: TODO (traits defined, NoOp implementation only)**

### AI Update Pipeline

| Function | Address | Status | Details |
|----------|---------|--------|---------|
| AI_UpdateAllTribes | 0x0041a7d0 | TODO | Per-tribe AI loop |
| AI_UpdateTribe | 0x0041a8b0 | TODO | Individual tribe AI update |
| AI_RunScript | 0x004c5eb0 | TODO | Execute AI bytecode |
| AI_ProcessScriptBlock | 0x004c6180 | TODO | IF/ELSE/ENDIF flow control |
| AI_ExecuteScriptCommand | 0x004c6460 | TODO | 200+ opcodes |
| AI_EvaluateScriptValue | 0x004c8b50 | TODO | Variable/constant lookup |

### AI Script System (Bytecode)

| Item | Status | Details |
|------|--------|---------|
| Script loading from CPSCR files | TODO | 0x3108 bytes per AI player at 0x948E52 |
| 16-bit opcodes (base 0x404) | TODO | 0x404-0x417: attribute enable/disable |
| Script value types | TODO | Type 0: literal, Type 1: variable, Type 2: internal attribute |
| Internal attributes (1000-1237) | TODO | Per-tribe state lookups |
| Comparison operators | TODO | Tokens 0x3F4-0x3F9 |
| Loop constructs | TODO | EVERY/DO loops |
| Subroutine calls | TODO | Nested script execution |

### Shaman Command System

| Command Type | Value | Status | Details |
|-------------|-------|--------|---------|
| Primary Attack | 0x00 | TODO | Main army attack |
| Secondary Attack | 0x01 | TODO | Flanking maneuver |
| Defend Position | 0x02 | TODO | Hold position |
| Spell Casting | 0x03 | TODO | Cast spells at target |
| Army Movement | 0x04 | TODO | Move units to location |
| Building Placement | 0x05 | TODO | Construct buildings |
| Resource Gathering | 0x06 | TODO | Send braves to gather wood |
| Conversion | 0x07 | TODO | Convert wild people |

Shaman command structure: 0x52 bytes, 10 slots per tribe at offset +0x74.

### AI Decision Making

| Item | Status | Details |
|------|--------|---------|
| Target selection scoring | TODO | Shaman=1000, SuperWarrior=20, Warrior=10, Preacher=15, Temple=200 |
| Threat assessment | TODO | AI_AssessThreat (0x0041ba40), distance-weighted |
| Building priorities | TODO | AI_ExecuteBuildingPriorities (0x0041b8d0), bubble-sort ordering |
| Spell casting evaluation | TODO | AI_EvaluateSpellCasting (0x004b8a90), every ~32 frames |
| Shaman safety check | TODO | AI_CheckShamanSafety (0x0041bae0), retreat if threatened |
| Personality traits | TODO | Per-tribe values at offset +0x137 (aggression, expansion, risk, tech path) |

### Difficulty Scaling

| Item | Status | Details |
|------|--------|---------|
| Mana adjustment | TODO | COMPUTER_MANA_ADJUST vs HUMAN_MANA_ADJUST |
| Separate training costs | TODO | CP_TRAIN_MANA_* vs HUMAN_TRAIN_MANA_* |
| Training cost bands | TODO | Scales with population (100% at 0-3, ~200% at 21+) |

---

## 7. Terrain System

### Heightmap

| Item | Status | Details |
|------|--------|---------|
| 128x128 heightmap (u16 per cell) | DONE | LandscapeMesh heights array |
| Height interpolation | DONE | interpolate_height_at() matches 0x004E8E50 |
| Triangle split direction (cell flag bit 0) | TODO | Original uses bit 0 to flip diagonal; Pop3 always uses "/" split |
| Height modification | TODO | Terrain_ModifyHeight (0x004ea2e0), gradual height change |
| Water level | PARTIAL | Water surface exists, no dynamic water level |

### Cell Flags (0x0088897C)

| Flag | Bit | Status | Details |
|------|-----|--------|---------|
| Triangle split direction | 0x0001 | TODO | 0=NW-SE, 1=NE-SW diagonal |
| Has object | 0x0002 | TODO | Updated by Cell_UpdateFlags |
| Needs redraw | 0x0008 | TODO | Dirty flag |
| Water surface | 0x0200 | TODO | Cell is water |
| Has building | 0x80000 | TODO | Building footprint |
| Building foundation | 0x100000 | TODO | Under construction |

### Terrain Rendering

| Item | Status | Details |
|------|--------|---------|
| Landscape mesh generation | DONE | 128x128 grid, 6 vertices per cell |
| GPU heights buffer | DONE | u32 per cell, updated after flattening |
| Per-vertex curvature | DONE | Matches vertex shader formula |
| Viewport fade/discard | DONE | Circular falloff at viewport_radius |
| Terrain shading (brightness LUT) | TODO | Brightness 0-63 from sun direction + height tables |
| Distance attenuation | TODO | Quadratic falloff beyond threshold |
| Coastline/boundary rendering | TODO | Edge pattern lookup with rotation (0x00475830) |

---

## 8. Rendering Pipeline

### Original Architecture (for reference)

The original uses a hybrid renderer: 3D terrain mesh + 2D pre-rendered sprites for units/buildings + depth bucket sorting (3585 buckets). Pop3 uses modern wgpu with hardware z-buffer instead.

### Current Pop3 Rendering

| Item | Status | Details |
|------|--------|---------|
| Landscape mesh (WGSL shaders) | DONE | 4 shader variants (landscape.wgsl, _cpu, _full, _grad) |
| Building 3D meshes | DONE | Per-vertex terrain height + curvature |
| Building viewport culling | DONE | Distance check matching viewport_radius |
| Unit sprites (2D) | DONE | Per-tribe sprite atlases, animation frames |
| Object markers (colored squares) | DONE | Color-coded by type (green=scenery, etc.) |
| Sky rendering | DONE | Sky viewer exists, sky texture loading |
| Scenery 3D meshes | PARTIAL | Rendered as markers, not full meshes |

### Missing Rendering Features

| Item | Status | Details |
|------|--------|---------|
| Shadow rendering | TODO | Sprite-based shadows with offset calculation (0x00416410) |
| Depth-scaled sprites | TODO | Objects appear smaller at distance |
| Selection ring animation | TODO | 85 sprites, 24-angle step, 12-frame cycle at sprite 0x5BA |
| Health bars above units | TODO | 4 bar types: standard, mana, building health, training |
| Building ghost/preview | TODO | Transparent placement preview |
| Lightning visual effects | TODO | Special render command type 0x13 |
| Spell visual effects | TODO | Render command type 0x18 |
| Wireframe debug render | TODO | Render command type 0x1B |
| Path arrow indicators | TODO | Render command type 0x1C |

### Layer-Based Rendering Order (Original)

| Priority | Layer | Content | Status |
|----------|-------|---------|--------|
| 0 | 0x03 | Terrain/ground | DONE |
| 1 | 0x0B | Ground objects | PARTIAL |
| 2 | 0x13 | Buildings | DONE |
| 3 | 0x16 | Special structures | TODO |
| 4 | 0x10 | Magic effects | TODO |
| 5 | 0x1B | Creatures | PARTIAL |
| 6 | 0x21 | Flying effects | TODO |
| 7-9 | 0x22/0x14/0x1E | Effects, spells, highest | TODO |

---

## 9. Sprite & Animation System

### Sprite Loading

| Item | Status | Details |
|------|--------|---------|
| HSPR sprite banks | DONE | Sprite data loading from .dat files |
| VSTART animation starts | DONE | Animation start frame data |
| VFRA frame data | DONE | Per-frame animation data |
| VELE element data | DONE | Animation element chains |
| Sprite atlas building | DONE | Per-tribe and direct atlases |
| RLE sprite decompression | DONE | Run-length encoded pixel data |

### Animation System

| Item | Status | Details |
|------|--------|---------|
| Per-direction animations (5+3 mirrored) | DONE | 8 direction support |
| Person_SetAnimation (0x004feed0) | PARTIAL | State-based animation selection |
| Animation state tracking per object | PARTIAL | Frame counter, current bank |
| Mounted/vehicle animations | TODO | DAT_0059fce0 mounted animation table |
| Shaman-specific animations | DONE | Per-tribe shaman sprites |

### 3D Object Rendering

| Item | Status | Details |
|------|--------|---------|
| ObjectRaw/PointRaw/FaceRaw parsing | DONE | objects.rs binary deserialization |
| 3D mesh construction (Object3D) | DONE | Face iteration with UV mapping |
| Texture atlas (8x32 grid, 256x256 tiles) | DONE | TexModel with tex_id per vertex |
| Per-vertex terrain height sampling | DONE | Building vertices follow terrain |
| Sphere mesh (plssphr.dat) | TODO | XOR encrypted, text-based format |

---

## 10. Water & Effects System

### Water Rendering

| Item | Status | Details |
|------|--------|---------|
| Water texture generation | DONE | BIGF0 + DISP0 displacement lookup |
| Tick_UpdateWater (0x0048bf10) | PARTIAL | Basic water rendering |
| Water mesh animation | TODO | UV animation with sin/cos tables, 0x3F frame cycle |
| Water transparency | TODO | DAT_00884bf9 & 0x100 flag |
| Coastline distance threshold | TODO | 0xFFF (4095) |

### Effect System (93 effect types)

**Status: TODO (none implemented)**

| Category | Types | Count | Details |
|----------|-------|-------|---------|
| Spell effects | 0x01-0x1F | 31 | Burn, smoke, lightning, blast, tornado, earthquake, volcano, swarm, etc. |
| Environmental | 0x20-0x2F | 16 | Water splash, ripple, rain, snow, fog, dust, tree fall/burn, lava, geyser |
| Unit animation | 0x30-0x3F | 16 | Death, blood spray, hit spark, knockback, stun, heal, level up, convert, drown |
| Particle | 0x40-0x4F | 16 | Spark, ember, ash, dirt, stone, wood, leaf, feather, magic, energy, trail |
| Building | 0x50-0x57 | 8 | Construction, destruction, fire, smoke, tower shot, boat wake, balloon fire |
| World state | 0x58-0x5C | 5 | Day/night, weather, terrain morph, water level change, victory |

### Effect Infrastructure

| Item | Status | Details |
|------|--------|---------|
| Effect pool (512 max) | TODO | g_EffectPool at 0x00974000, 64 bytes per effect |
| Effect linked list | TODO | Head at 0x00973f00 |
| Effect_QueueVisual (0x00453780) | TODO | Queue for rendering, max 50 active |
| Effect_SortQueue (0x00453a10) | TODO | Sort by distance, 7x7 grid distribution |
| Effect_SpawnAt (0x004a7000) | TODO | Create effect at position |
| Effect_AttachToEntity (0x004a7050) | TODO | Attach to moving object |

---

## 11. Movement & Pathfinding

### Movement System

| Item | Status | Details |
|------|--------|---------|
| MovePointByAngle (0x004d4b20) | DONE | Per-tick position update, ~320 calls/sec |
| Sin/Cos lookup tables (2048 entries) | DONE | 16.16 fixed-point |
| Formation movement (up to 12 followers) | DONE | Object_UpdateMovement (0x4ed510) |
| Formation grid (3x4, 16-unit spacing) | DONE | Anchor + signed offset |
| Arrival threshold (72 units per axis) | DONE | |
| Follower catch-up speed (1.5x) | DONE | |

### Pathfinding

| Item | Status | Details |
|------|--------|---------|
| STATE_GOTO dispatcher (0x4d7e20) | DONE | Thin dispatcher to route lookup |
| RouteTableLookup (0x4d7f20) | DONE | 4-tier cache system |
| Tier 1: Region Map (128x128) | DONE | O(1) same-region check |
| Tier 2: Segment Pool (400 slots) | DONE | Cached path segments, 109 bytes each |
| Tier 3: Failure Cache (8 entries) | DONE | Skip known-impossible routes |
| Tier 4: Wall-following pathfinder (0x45D090) | DONE | Dual-arm Bug2 algorithm |
| AdjustTargetForWalkability (0x4da480) | DONE | Snap to nearest walkable |
| Line-of-sight optimizer (0x45e3c0) | DONE | Bresenham-based waypoint reduction |
| ProcessRouteMovement (0x4d8e60) | DONE | ~3KB, waypoint stepping + building entrance |

### Path_FindBestDirection (0x00424ed0)

| Item | Status | Details |
|------|--------|---------|
| Local greedy direction finding | TODO | Confirmed dormant during normal gameplay (Frida verified) |
| 22-step terrain sampling | TODO | ±67.5° from facing, step distance 0xe0 |
| Spiral search (±0x5d increments) | TODO | Fallback when forward blocked |

---

## 12. Camera & Projection

### Camera System

| Item | Status | Details |
|------|--------|---------|
| Camera orbit (angle_z rotation, angle_x pitch) | DONE | view.rs Camera struct |
| MVP matrix computation | DONE | Perspective projection |
| Toroidal world wrapping | DONE | Wrap at world boundaries |
| Camera scroll/pan | DONE | Keyboard/mouse control |

### Original Projection (for reference)

| Item | Status | Details |
|------|--------|---------|
| 3x3 rotation matrix (16.14 fixed-point) | N/A | Pop3 uses float MVP instead |
| Curvature: y -= (x²+z²) * 46000 | DONE | Per-vertex in landscape shader and building mesh |
| Perspective division | DONE | Hardware perspective |
| Near plane: 6500 (0x1964) | N/A | Different projection model |
| FOV exponent: 11 (0x0B) | N/A | Different projection model |
| Resolution-specific quad offsets | TODO | 800x600, 1024x768, 1280x1024 viewport quads |

---

## 13. UI / HUD / Menus

**Status: TODO (no UI implemented)**

### In-Game HUD

| Item | Status | Details |
|------|--------|---------|
| Minimap (128x128, bottom-left) | TODO | Minimap_RenderTerrain (0x0042ba10), tribe-colored unit dots |
| Spell bar (bottom-center) | TODO | UI_ProcessSpellButtons (0x00494430) |
| Unit info panel (bottom-right) | TODO | UI_RenderBuildingInfo (0x004937f0) |
| Mana bar (top-left) | TODO | |
| Population display (top-right) | TODO | |
| Resource display | TODO | UI_RenderResourceDisplay (0x00493350) |
| Status text | TODO | UI_RenderStatusText (0x00493560) |
| Frame rate display | TODO | DrawFrameRate (0x004a6bf0) |
| Fog of war on minimap | TODO | Cell visibility check |

### Menu System

| Menu | Address | Status |
|------|---------|--------|
| Main Menu | 0x0041f400 | TODO |
| Single Player | 0x0041f600 | TODO |
| Campaign | 0x0041f800 | TODO |
| Skirmish | 0x0041fa00 | TODO |
| Multiplayer Lobby | 0x0041fe00 | TODO |
| Options | 0x00420000 | TODO |
| Graphics Settings | 0x00420200 | TODO |
| Sound Settings | 0x00420400 | TODO |
| Controls / Key Remap | 0x00420600 | TODO |
| Load Game | 0x00420800 | TODO |
| Save Game | 0x00420a00 | TODO |
| Credits | 0x00420c00 | TODO |
| Level Select | 0x00421000 | TODO |
| Tutorial | 0x00421200 | TODO |

### Menu Button System

| Item | Status | Details |
|------|--------|---------|
| MenuButton struct (0x30 bytes) | TODO | x, y, width, height, state, sprites, onClick, hotkey |
| Button_ProcessInput (0x00421400) | TODO | Hit test, state transitions, sound |
| Menu transitions (fade in/out) | TODO | Menu_TransitionTo (0x0041f100) |

### Font / Text Rendering

| Item | Status | Details |
|------|--------|---------|
| Font loading (12/16/24pt) | TODO | font24j.fon, font16j.fon, font12j.fon |
| Character rendering | TODO | Render_DrawCharacter (0x004a0570) |
| Multi-language support (12 languages) | TODO | European (single-byte), CJK (multi-byte) |
| String table loading | TODO | LANGUAGE/lang##.dat (0x526 strings) |
| Language_SetCurrent (0x004531c0) | TODO | 0-10: EN/FR/DE/IT/ES/SV/NO/DA/FI/NL/PT |

### Input System

| Item | Status | Details |
|------|--------|---------|
| Keyboard input | DONE | winit event handling |
| Mouse input (click, drag) | DONE | Unit selection, camera control |
| Key definition file (key_def.dat) | TODO | Binary format, 15 bytes per binding, 207 actions |
| Input_LoadKeyDefinitions (0x004dbd20) | TODO | Load custom key bindings |
| Game_ProcessInput (0x004c4c20) | TODO | Full input state with 256-entry buffer |

---

## 14. Audio System

**Status: TODO (nothing implemented)**

### Sound Playback

| Item | Status | Details |
|------|--------|---------|
| Sound_Play (0x00417300) | TODO | 3D positional audio with distance attenuation |
| Max audible distance: 0x9000000 (squared) | TODO | Volume falloff formula |
| Sound data table at 0x005a5c70 | TODO | 0x0C bytes per entry: pitch, quality, variations, priority, volume |
| Sound variations (PRNG at 0x0088420a) | TODO | Pitch/volume randomization |
| Sound flags | TODO | 2D/3D, looping, priority, ignore pause |
| Pan calculation | TODO | Atan2 angle to camera, 0-127 range |

### Sound Files

| File | Purpose | Status |
|------|---------|--------|
| soundd2.sdt / soundd2low.sdt | SFX (high/low quality) | TODO |
| popdrones22.sdt | Ambient drone sounds | TODO |
| popfight.sf2 | SoundFont for music | TODO |

### Audio Library (QSWaveMix.dll)

| Item | Status | Details |
|------|--------|---------|
| Channel-based mixing (24 functions) | TODO | Init, channel control, 3D positioning, volume |
| MIDI music via winmm.dll | TODO | midiOutOpen, mciSendCommand |
| SoundFont rendering | TODO | SF2-based music playback |

### Sound Categories

| Category | IDs | Status |
|----------|-----|--------|
| Ambient loops | 0x1C, 0x50, 0xC2, 0xC6 | TODO |
| Unit acknowledgements | 0x73-0x86 | TODO |
| Combat sounds | Various | TODO |
| UI/menu sounds | 0x1E-0x21, 0xC8-0xCA, 0xD5 | TODO |
| Spell sounds | Various | TODO |
| Building sounds | Various | TODO |

---

## 15. Level Loading & Save/Load

### Level Loading

| Item | Status | Details |
|------|--------|---------|
| DAT file parsing (192,137 bytes) | DONE | Heightmap, map layers, tribe config, sunlight, 2000 units |
| HDR file parsing (616 bytes) | DONE | Level name, landscape type, markers |
| Unit struct (55 bytes, 2000 slots) | DONE | Subtype, model, tribe, position, angle |
| Landscape type to texture mapping | DONE | HDR byte 0x60 → texture file character |
| LoadLevelTextures (0x00421320) | DONE | 4 texture files per level |
| AI patrol files (CPATR) | TODO | ~144 bytes per AI player |
| AI script files (CPSCR) | TODO | 0x3108 bytes per AI player |
| LEVLSPC2.DAT (924 bytes) | TODO | Per-level special configuration |
| OBJECTIV.DAT (768 bytes) | TODO | 48-level objectives |

### Save/Load System

| Item | Status | Details |
|------|--------|---------|
| SaveGame_Create (0x00462130) | TODO | Write 0x1398 byte header + level data |
| SaveGame_Save (0x004627f0) | TODO | Serialize full game state |
| SaveGame_Load (0x00462d00) | TODO | Restore game state, terrain, objects, camera |
| Save file format | TODO | SAVGAM##.DAT, 31 rolling backups, 860KB state |
| Quicksave (slot 99) | TODO | |

### Campaign System

| Item | Status | Details |
|------|--------|---------|
| Campaign_AdvanceToNextLevel (0x00421500) | TODO | Level availability check, progression |
| Campaign level table | TODO | At 0x664BAC, stride 0x34, level ID at +0x28 |
| Completion flags | TODO | At 0x8828FD + level*0xA4 |
| Level ranges 1-25 | TODO | Special transitions at levels 24 (credits) and 25 (endgame) |

---

## 16. Network / Multiplayer

**Status: TODO (traits defined only)**

### Network Protocol

| Item | Status | Details |
|------|--------|---------|
| Network_SendPacket (0x004e6c40) | TODO | MLDPlay library integration |
| Lockstep synchronization | TODO | Deterministic tick model |
| Heartbeat packets (type 0x07) | TODO | Per-tick acknowledgment |
| Game state sync (type 0x06) | TODO | 0x55 bytes with checksum |
| Time sync (type 0x0D) | TODO | Server tick counter |
| Pause sync (type 0x0E) | TODO | Coordinated pause |
| RLE-compressed actions (types 0x01-0x05) | TODO | Player action compression |
| Chat messages (type 0x08) | TODO | In-game chat |

### Desync Detection

| Item | Status | Details |
|------|--------|---------|
| Checksum comparison | TODO | Per-tick checksums across players |
| Sync categories | TODO | Random seed, player count, people/buildings, creatures, map segments |
| Sync logging | TODO | Network_WriteSyncLog (0x004e5ad0) |
| Deterministic RNG | DONE | Same seed = same outcome (implemented in pathfinding) |

---

## 17. Creature System

**Status: PARTIAL (type defined, no behavior)**

### Creature Subtypes (7)

| Creature | Value | Status | Notes |
|----------|-------|--------|-------|
| Bear | 1 | TODO | |
| Buffalo | 2 | TODO | |
| Wolf | 3 | TODO | |
| Eagle | 4 | TODO | |
| Rabbit | 5 | TODO | |
| Beaver | 6 | TODO | |
| Fish | 7 | TODO | |

### Creature State Machine (16 states)

| Item | Status | Details |
|------|--------|---------|
| Creature_Init (0x00483270) | TODO | Initialize creature object |
| Creature_SetState (0x00483580) | TODO | 16-state handler |
| Creature group combat | TODO | Creature_OrchestrateGroupCombat (0x00484490) |
| Idle → Berserk state propagation | TODO | Triggers nearby creatures |

---

## 18. Vehicle System

**Status: PARTIAL (type defined, states exist, no logic)**

### Vehicle Subtypes (4)

| Vehicle | Value | Status | Notes |
|---------|-------|--------|-------|
| Boat 1 | 1 | TODO | |
| Boat 2 | 2 | TODO | |
| Airship 1 | 3 | TODO | |
| Airship 2 | 4 | TODO | |

### Vehicle States (9)

| State | Value | Status | Details |
|-------|-------|--------|---------|
| Idle/Docked | 0x01 | TODO | |
| Moving | 0x02 | TODO | Sea/air movement |
| Loading | 0x03 | TODO | Passengers boarding |
| Unloading | 0x04 | TODO | Passengers disembarking |
| Sinking/Crashing | 0x05 | TODO | Destruction |
| Rising/TakingOff | 0x06 | TODO | |
| Burning | 0x07 | TODO | |
| Landing | 0x08 | TODO | |
| TakingOff | 0x09 | TODO | |

### Vehicle Features

| Item | Status | Details |
|------|--------|---------|
| Vehicle type data (0x005a0720) | TODO | 23 bytes per type: max passengers, height, flags |
| Person boarding/exit | TODO | Person_EnterVehicleState (0x0050a960), ExitVehicle (0x0050b480) |
| Passenger animation | TODO | Vehicle_UpdatePassengerAnimations (0x0049b6f0) |
| Flying vehicle oscillation | TODO | Phase-based altitude oscillation |
| Vehicle production from buildings | TODO | Building_UpdateActive_Vehicle (0x00431970) |

---

## 19. Scenery System

**Status: PARTIAL (rendered as markers)**

### Scenery Subtypes (19)

| Subtype | Value | Status | Notes |
|---------|-------|--------|-------|
| Mass Tree | 1 | PARTIAL | Rendered, no interaction |
| Special Tree 1/2 | 2-3 | PARTIAL | |
| Mass Fruit Tree | 4 | PARTIAL | |
| Special Fruit Tree 1/2 | 5-6 | PARTIAL | |
| Plant 1/2 | 7-8 | PARTIAL | |
| Stone Head | 9 | PARTIAL | Discovery site (no worship) |
| Fire | 10 | TODO | |
| Wood Pile | 11 | TODO | Resource |
| Reincarnation Pillar | 12 | PARTIAL | Rendered, no function |
| Rock | 13 | PARTIAL | |
| Portal | 14 | TODO | Level exit |
| Island | 15 | TODO | |
| Bridge | 16 | TODO | |
| Top Level Scenery | 18 | TODO | |
| Sub Level Scenery | 19 | TODO | |

### Tree System

| Item | Status | Details |
|------|--------|---------|
| Tree types (6) with growth | TODO | TREE1-6_WOOD_VALUE, WOOD_GROW, DORMANT_TIME |
| Tree wood harvesting | TODO | Person_StartWoodGathering (0x00502f70) |
| Tree burning | TODO | Effect type 0x28 (TreeBurn) |
| Tree fall animation | TODO | Effect type 0x27 (TreeFall) |
| Scenery data table (0x005a07a0) | TODO | 0x18 bytes per type: state, growth, flags, height range |

---

## 20. Discovery System

**Status: TODO**

| Item | Status | Details |
|------|--------|---------|
| Stone head worship | TODO | Discovery_Init (0x004bec80) |
| Discovery_Check (0x004bed50) | TODO | Check worship progress |
| Discovery_Grant (0x004bee20) | TODO | Unlock spell/building/ability |
| Discovery types | TODO | 1=Spell unlock, 2=Building unlock, 3=Special ability |
| Worship progress tracking | TODO | g_WorshipProgress at 0x00885810 |
| Discovered spells/buildings | TODO | g_DiscoveredSpells (0x00885800), g_DiscoveredBuildings (0x00885808) |

---

## 21. Economy & Resources

### Wood System

| Item | Status | Details |
|------|--------|---------|
| Tree wood values | TODO | TREE1-6_WOOD_VALUE |
| Tree growth/regrowth | TODO | TREE1-6_WOOD_GROW, DORMANT_TIME |
| Wood gathering by braves | TODO | PersonState::GatheringWood defined, no logic |
| Wood carrying to buildings | TODO | PersonState::CarryingWood defined, no logic |
| Building wood storage (+0x63) | TODO | Per-building wood counter |
| Building wood consumption | TODO | Building_UpdateWoodConsumption (0x00430430) |

### Wood Costs

| Item | Cost Constant | Status |
|------|--------------|--------|
| Brave | WOOD_BRAVE | TODO |
| Warrior | WOOD_WARR | TODO |
| Preacher | WOOD_PREACH | TODO |
| Super Warrior | WOOD_SWARR | TODO |
| Small Hut | WOOD_HUT_1 | TODO |
| Medium Hut | WOOD_HUT_2 | TODO |
| Large Hut | WOOD_HUT_3 | TODO |
| Drum Tower | WOOD_DRUM_TOWER | TODO |
| Temple | WOOD_TEMPLE | TODO |
| Boat Hut | WOOD_BOAT_1 | TODO |
| Air Hut | WOOD_AIR_1 | TODO |

---

## 22. Population & Mana

### Population System

| Item | Status | Details |
|------|--------|---------|
| Tick_UpdatePopulation (0x004198f0) | TODO | Per-tick population updates |
| Housing capacity per hut level | TODO | MAX_POP_VALUE_HUT_1/2/3 |
| Spawn rate bands (17 bands) | TODO | SPROG%_POP_BAND_00_04% through 95_99% |
| Hut brave recruitment | TODO | 3x3 tile search for recruitable braves |
| Population cap (MAX_POP_VALUE) | TODO | Maximum tribe population |

### Mana Generation

| Item | Status | Details |
|------|--------|---------|
| Tick_UpdateMana (0x004aeac0) | TODO | Per-tribe mana regeneration |
| Mana per unit type | TODO | MANA_F_BRAVE through MANA_F_SHAMEN |
| Mana per activity | TODO | Idle, busy, housed, training |
| Mana per housing level | TODO | MANA_F_HUT_LEVEL_1/2/3 |
| Mana on shaman death | TODO | SHAMEN_DEAD_MANA_%_LOST/GAIN |
| Human vs Computer mana adjust | TODO | Difficulty scaling |

---

## 23. Texture & Palette System

### Textures

| Item | Status | Details |
|------|--------|---------|
| Terrain textures (plstx###.dat) | DONE | 0x40000 bytes per level |
| Splash/palette data (plspl0-X.dat) | DONE | 0x400 bytes |
| Font textures (plsft0-X.dat) | DONE | 0x4000 bytes |
| Cliff/cave textures (plscv0-X.dat) | DONE | 0x6000 bytes |
| UV rotation tables (4 rotations) | TODO | Terrain_InitializeUVRotationTables (0x00451110) |

### Palette System

| Item | Status | Details |
|------|--------|---------|
| Primary palette (256 x BGRA) | DONE | 0x00973640 |
| Palette file loading | DONE | pal0-0.dat and per-level variants |
| Fade palette (fade0-0.dat) | TODO | Screen fade effects |
| Ghost palette (ghost0-0.dat) | TODO | Transparency/ghost effect |
| Blue palette (bl320-0.dat) | TODO | Water colors |
| Animated palette (anibl0-0.dat) | TODO | Animated water/lava |
| XOR palette decryption | DONE | Magic bytes 0x40, 0x7E |
| Palette_FindClosestColor (0x0050f7f0) | TODO | Color matching with luminance preference |
| Special color indices | TODO | 12 pre-assigned colors for tribes (blue/red/yellow/green at 3 brightness levels) |

---

## 24. Utility Systems

### Math System

| Item | Status | Details |
|------|--------|---------|
| 11-bit angle system (0-2047) | DONE | Used throughout movement/rendering |
| Sin/Cos tables (2048 entries, 16.16) | DONE | At 0x005ac6a0/0x005acea0 |
| Math_Atan2 (0x00564074) | DONE | Angle calculation |
| Math_DistanceSquared (0x004ea950) | DONE | Toroidal wrap-aware |
| Math_MovePointByAngle (0x004d4b20) | DONE | Per-tick position update |
| Math_AngleDifference (0x004d7c10) | DONE | Wrap-aware angle diff |

### Random Number Generator

| Item | Status | Details |
|------|--------|---------|
| LCG RNG (seed at 0x885710) | DONE | `seed = seed * 0x24A1 + 0x24DF; rotate_right(seed, 13)` |
| Deterministic for multiplayer | DONE | Same seed = same outcome |
| Separate audio RNG | TODO | Seed at 0x0088420a |

### File I/O

| Item | Status | Details |
|------|--------|---------|
| File_Open/Close/Read/Write | DONE | Rust std::fs |
| Path building | DONE | Level file path construction |
| CD-ROM fallback | TODO | BuildFilePath checks CD drive |
| File_ReadEntire (0x005119b0) | DONE | Full file reads |

### Constant.dat Parser

| Item | Status | Details |
|------|--------|---------|
| Parser (0x0041eb50) | PARTIAL | Constants defined in Rust code, not parsed from file |
| XOR decryption (0x40, 0x7E header) | TODO | Encrypted constant files |
| Key=value text format | TODO | P3CONST_ prefix stripped |
| 31-byte constant entries | TODO | Name (25), size, flags, pointer |

### Debug / Cheat System

| Item | Status | Details |
|------|--------|---------|
| Debug_ProcessCheatCommand (0x004a7b10) | TODO | Object spawning, mana, god mode, spell unlocks |
| Debug object types (3-5) | TODO | Static, flying, flag markers |

### Flying Physics

| Item | Status | Details |
|------|--------|---------|
| Object_UpdateFlyingPhysics (0x004d4db0) | TODO | Gravity, drag, terrain bounce |
| Flying type data (0x005a0970) | TODO | Turn rate, max speed, gravity per type |

---

## 25. Game Loop & Tick System

### Main Game Loop

| Item | Status | Details |
|------|--------|---------|
| GameLoop (0x004ba520) | PARTIAL | winit event loop exists |
| Game_SimulationTick (0x004bb5a0) | PARTIAL | Tick traits defined, mostly NoOp |
| Game_RenderWorld (0x0048c070) | PARTIAL | Landscape + buildings + sprites rendered |
| Frame rate limiting | PARTIAL | |

### Tick Processing Order

| Step | Function | Status | Details |
|------|----------|--------|---------|
| 1. Process network messages | Tick_ProcessNetworkMessages (0x004a76b0) | TODO | |
| 2. Process pending actions | Tick_ProcessPendingActions (0x004a6f60) | TODO | |
| 3. Update game time | Tick_UpdateGameTime (0x004a7ac0) | TODO | |
| 4. Update terrain | Tick_UpdateTerrain (0x0048bda0) | TODO | Dynamic terrain changes |
| 5. Update objects | Tick_UpdateObjects (0x004a7550) | PARTIAL | Movement updates work |
| 6. Update water | Tick_UpdateWater (0x0048bf10) | PARTIAL | Basic water rendering |
| 7. AI for all tribes | AI_UpdateAllTribes (0x0041a7d0) | TODO | |
| 8. Update population | Tick_UpdatePopulation (0x004198f0) | TODO | |
| 9. Update mana | Tick_UpdateMana (0x004aeac0) | TODO | |
| 10. Single player logic | Tick_UpdateSinglePlayer (0x00456500) | TODO | |
| 11. Tutorial logic | Tick_UpdateTutorial (0x00469320) | TODO | |

### Game States

| State | Value | Status | Details |
|-------|-------|--------|---------|
| Init | 0x00 | DONE | Startup |
| Frontend/Menu | 0x02 | TODO | GameState_Frontend (0x004baa40) |
| In-Game | 0x07 | PARTIAL | Rendering + movement works |
| Victory | 0x08 | TODO | |
| Transition | 0x09 | TODO | |
| Loading | 0x0A | PARTIAL | Level loading works |
| Outro | 0x0B | TODO | GameState_Outro (0x004bae70) |
| Multiplayer Lobby | 0x0C | TODO | GameState_Multiplayer (0x004c03d0) |

### Timing

| Item | Status | Details |
|------|--------|---------|
| g_GameSpeed (0x008856f9) | TODO | Configurable ticks per second |
| g_TickIntervalMs (0x0059ac70) | TODO | 1000 / g_GameSpeed |
| g_GameTick (0x0088571c) | TODO | Current tick counter |
| Delta time capping (1000ms max) | TODO | Prevents spiral of death |

---

## Summary Statistics

| Category | DONE | PARTIAL | TODO | Total |
|----------|------|---------|------|-------|
| Core Object System | 4 | 3 | 8 | 15 |
| Person/Unit System | 5 | 6 | 25 | 36 |
| Building System | 5 | 1 | 28 | 34 |
| Spell System | 0 | 0 | 30 | 30 |
| Combat System | 0 | 2 | 13 | 15 |
| AI/Scripting | 0 | 0 | 18 | 18 |
| Terrain System | 5 | 1 | 7 | 13 |
| Rendering Pipeline | 7 | 1 | 10 | 18 |
| Sprite/Animation | 6 | 2 | 3 | 11 |
| Water/Effects | 1 | 1 | 8 | 10 |
| Movement/Pathfinding | 11 | 0 | 3 | 14 |
| Camera/Projection | 4 | 0 | 1 | 5 |
| UI/HUD/Menus | 2 | 0 | 24 | 26 |
| Audio | 0 | 0 | 8 | 8 |
| Level/Save/Load | 5 | 0 | 9 | 14 |
| Network | 1 | 0 | 9 | 10 |
| Creatures | 0 | 0 | 3 | 3 |
| Vehicles | 0 | 0 | 5 | 5 |
| Scenery | 0 | 1 | 5 | 6 |
| Discovery | 0 | 0 | 6 | 6 |
| Economy/Resources | 0 | 0 | 6 | 6 |
| Population/Mana | 0 | 0 | 6 | 6 |
| Texture/Palette | 4 | 1 | 7 | 12 |
| Utilities | 7 | 1 | 5 | 13 |
| Game Loop/Tick | 1 | 3 | 9 | 13 |
| **TOTAL** | **68** | **23** | **275** | **366** |

**Progress: 18.6% complete (68/366), 24.9% with partial (91/366)**

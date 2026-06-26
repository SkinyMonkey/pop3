# Combat and Pathfinding

## Damage System

### Object_ApplyDamage (0x00504f20)

```c
void Object_ApplyDamage(obj, attacker_tribe, damage, ignore_immunity) {
    if (cheat_godmode) return;
    if (!ignore_immunity && obj_has_immunity) return;

    if (obj_has_shield) {
        damage = damage >> SHIELD_SHIFT;  // Shield reduces damage
    }

    obj->health -= damage;  // Health at offset 0x6E

    if (attacker_tribe != -1) {
        obj->last_attacker = attacker_tribe;  // Track attacker at 0xB0
    }
}
```

### Building_ApplyDamage (0x00434570)

Similar to object damage but for buildings with fire spread mechanics.

### Shot_ProcessImpact (0x004fb620)

Projectile impact handler:
- Damages all persons in cell
- Tracks kills for tribe statistics
- Applies knockback effect
- Handles building damage separately
- Can ignite scenery (trees)

---

## Appendix D: Combat System Details

### Melee Damage Formula (Combat_ProcessMeleeDamage @ 0x004c5d20)

```c
// Calculate attacker damage
damage = (FIGHT_DAMAGE[attacker_subtype] * attacker_health) / attacker_max_health;
if (damage < 0x21) damage = 0x20;  // Minimum damage = 32

// Apply bloodlust multiplier
if (attacker_has_bloodlust) {
    damage *= BLOODLUST_MULT;  // DAT_005a32bc
}

// Apply to defender
Object_ApplyDamage(defender, attacker_tribe, damage, 1);

// Track kills
if (defender_health <= 0) {
    Tribe_TrackKill(attacker, defender);
}
```

### Fight Damage Table (DAT_0059fe5b)

Per unit subtype, stride 0x32 (50 bytes):
- Offset 0x00: Base fight damage value
- Used in: `damage = (table[subtype*0x32] * current_health) / max_health`

### Shield Damage Reduction

```c
if (obj_has_shield) {
    damage = damage >> SHIELD_SHIFT;  // Right shift reduces damage
}
```

---

## Appendix R: Pathfinding System

### Path_FindBestDirection (0x00424ed0)

Finds optimal movement direction avoiding terrain obstacles.

**Algorithm:**
1. Calculate initial direction from current facing (±0x300 angle units)
2. Sample terrain heights along projected path (22 iterations)
3. If obstacles found, try direct path to target
4. If still blocked, spiral search through angles (±0x5d increments)
5. Return best direction with lowest obstacle score

**Key Constants:**
- 0x7ff = Angle mask (2048 angles = 360°)
- 0x300 = ±67.5° from current facing for initial search
- 0x400 = 90° offset for perpendicular sampling
- 0xe0 = Step distance per sample
- 0x4e0 = Reference height (1248 units)
- 0x5d = Spiral search angle increment

**Terrain Sampling:**
```c
sVar1 = Terrain_GetHeightAtPoint(x, z);
if (currentHeight < sVar1) {
    obstacleScore += (sVar1 - currentHeight);
}
```

---

## Appendix U: Damage System

### Object_ApplyDamage (0x00504f20)

```c
void Object_ApplyDamage(int object, char attacker, int damage, char ignoreFlags) {
    // Check god mode
    if ((DAT_0087e33c >> 24) & 4) return;

    // Check invulnerability flag
    if (!ignoreFlags && (object->flags & 0x80)) return;

    // Shield reduces damage
    if (object->flags2 & 8) {
        damage = damage >> DAT_005a32c0;  // Shield damage reduction
    }

    object->health -= damage;

    // Record attacker for kill credit
    if (object->owner != -1 && attacker != -1) {
        object->lastAttacker = attacker;
    }
}
```

### Building_ApplyDamage (0x00434570)

Separate function for building damage with different mechanics.

---

## Appendix AC: Kill Tracking

### Tribe_TrackKill

Called when a unit dies to credit the killer:

```c
if (killer != 0) {
    Tribe_TrackKill(killer, victim);
}
```

Used for:
- Score tracking
- AI response to losses
- Victory condition evaluation

---

## Appendix BL: Pathfinding System

### Algorithm: Iterative Direction Refinement

**NOT A* or flow-field** - uses local greedy pathfinding.

### Path_FindBestDirection @ 0x00424ed0

```c
int Path_FindBestDirection(int target_angle, Object* unit) {
    // Evaluate 2 candidate angles (±0x300 offset)
    int angle1 = target_angle + 0x300;
    int angle2 = target_angle - 0x300;

    // Project 22 steps ahead (0x16 steps)
    // Each step = 0xe0 (224) map units
    // Total scan depth = 22 × 224 = 4928 units

    int cost1 = 0, cost2 = 0;
    for (int i = 0; i < 22; i++) {
        int h1 = Terrain_GetHeightAtPoint(pos + angle1 * i * 224);
        int h2 = Terrain_GetHeightAtPoint(pos + angle2 * i * 224);
        // Accumulate uphill costs only
        if (h1 > current_height) cost1 += (h1 - current_height);
        if (h2 > current_height) cost2 += (h2 - current_height);
    }

    // Return lower-cost direction
    return (cost1 < cost2) ? angle1 : angle2;
}
```

### Path_UpdateDirection @ 0x004248c0

- Called every 4 ticks (counter at +0x5c2)
- Recalculates best direction
- Applies smooth rotation (max 0x44 = 68 angle units/tick)
- Uses Math_GetRotationDirection for turn direction

### Terrain_GetHeightAtPoint @ 0x004e8e50

```c
int Terrain_GetHeightAtPoint(int x, int y) {
    int cell_x = (x & 0x1fe) >> 1;
    int cell_y = (y & 0x1fe) >> 1;
    // Bilinear interpolation between 4 vertices
    // Cell flag bit 0 determines diagonal direction
}
```

### Formation Movement

**Object_UpdateMovement** @ 0x004ed510

- Formation grid: 3×4 = 12 units max
- Unit spacing: 0x10 (16) map units per cell
- Minimum inter-unit distance: 0x48 (72) units
- Target approach distance: 0x237 (567) units

```c
// Formation offset calculation
offset_x = formation_slot[0xc] * 0x10;  // Column
offset_y = formation_slot[0x19] * 0x10; // Row
```

**Formation_ReorderUnits** @ 0x004ee500
- Fills gaps when units leave formation
- Maintains compact formation shape

### Movement Speed

Base speed from unit type data (0x0059FE44 + type × 0x32):
- Modified by: `(speed * DAT_00884c41) >> 8`
- Applied via Math_MovePointByAngle with sin/cos tables

### Collision Avoidance

- Inter-unit spacing: min 72 units
- Uses squared distance for efficiency
- Smooth evasion (reduce speed, vector away)
- No explicit avoidance pathfinding

---

## Appendix BM: Combat System

### Combat_ProcessMeleeDamage @ 0x004c5d20

**Damage Formula:**
```c
base_damage = UnitStats[attacker_type].damage;
damage = (base_damage * attack_multiplier) / defense_divisor;
if (damage < 0x20) damage = 0x20;  // Minimum 32

// Shield reduction
if (has_shield) {
    damage = damage / DAT_005a32bc;
}

defender.health -= damage;
```

### Building_ProcessFightingPersons @ 0x00438610

**7-State Combat State Machine:**

| State | Value | Description |
|-------|-------|-------------|
| 0x00 | Idle | Setup |
| 0x01 | Approaching | Move to building |
| 0x02 | BasicStrike | Standard attack |
| 0x03 | AimedStrike | Targeted attack |
| 0x04 | OverheadSlam | Monks only |
| 0x05 | Knockback | Pushed back |
| 0x06 | Recovery1 | Cooldown |
| 0x07 | Recovery2 | Cooldown |

**Combat Parameters:**
- Minimum engagement: 0x0B (16 pixels)
- Max fighters per building: 6
- Random combat type: 0xF values for variety
- Directional combat via Math_Atan2

### Combat_ApplyKnockback @ 0x004d7490

```c
void Combat_ApplyKnockback(short* velocity, int* position, int force, uint angle) {
    // Angle decomposition
    vx += (cos(angle) * force) / 256;
    vy += (sin(angle) * force) / 256;

    // Slope adjustment (1/16 to 16/16)
    int height_diff = get_height_after() - get_height_before();
    force *= slope_ratio;

    // Minimum impulse: 3 units
}
```

### Creature Group Combat

**Creature_OrchestrateGroupCombat** @ 0x00484490
- Propagates fight state to nearby creatures
- Searches creature list by type (+0x88) and variant (+0x24)
- Triggers nearby creatures to enter Berserk (0x0F) state

---


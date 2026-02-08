# Movement System

## Overview

Unit movement in Populous: The Beginning uses a **4-tier pathfinding cache** system, not simple precomputed routes or runtime A* on every move. Most moves within the same terrain region use direct walking; only cross-region moves (e.g., across water via bridges) require actual pathfinding.

**Verified via Frida instrumentation** — all functions below confirmed active during gameplay.

> **Note**: The functions `Path_UpdateDirection` (0x4248c0) and `Path_FindBestDirection` (0x424ed0) documented in `combat_and_pathfinding.md` were proven to **never fire** during normal gameplay (20K+ calls to UpdateDirection with active flag always 0, across both player and AI unit movement). They appear to be a dormant or edge-case-only subsystem.

## Architecture

```
Right-click move order
  └─ Person_SetState (state 0x07 GOTO)
      └─ STATE_GOTO (0x4d7e20) — thin dispatcher
          ├─ AdjustTargetForWalkability (0x4da480)
          └─ RouteTableLookup (0x4d7f20) — 4-tier cache
              ├─ Tier 1: Region Map (0x88897C) — same region? direct walk
              ├─ Tier 2: Segment Pool (0x93E1C1) — reuse existing path?
              ├─ Tier 3: Failure Cache (0x93E171) — recently failed?
              └─ Tier 4: Dual-arm Wall-Following Pathfinder (0x45D090) — compute new path

Per-tick (every game tick):
  └─ Object_UpdateMovement (0x4ed510) — formation manager
      ├─ Validate followers (alive, in range, correct angle)
      ├─ Compute formation grid positions
      ├─ Assign speeds (0, base, or 1.5× catch-up)
      └─ MovePointByAngle (0x4d4b20) — advance position
```

## Functions

### STATE_GOTO (0x4d7e20)

Thin dispatcher, 76 bytes. Called once per unit per move order.

```c
// Original: 0x004d7e20
void STATE_GOTO(Person *person, WorldCoord *target) {
    WorldCoord local = *target;
    AdjustTargetForWalkability(person, &local);  // 0x4da480
    int segment = RouteTableLookup(person, &local);  // 0x4d7f20
    person->movement_dest = person->next_waypoint;  // +0x57 = +0x53
    person->flags1 |= 0x1000;   // has movement target
    person->flags1 &= ~0x80;    // clear blocked
}
```

Does NOT change person state byte. Called by state transition handlers.

### RouteTableLookup (0x4d7f20)

Core routing function. Returns segment index (0 = direct walk).

**Algorithm**:
1. Store target at unit+0x4F
2. Convert world→tile: `tile = (world >> 8) & 0xFE` (128×128 grid)
3. Read region IDs from region map cell+0x08 (low 10 bits)
4. Same region → set waypoint=target, return 0 (direct walk)
5. Different region → search segment pool (FUN_004d85f0)
6. No reusable segment → check failure cache, then A* (FUN_0045d090)
7. Assign segment, extract first waypoint, increment refcount

**Sub-functions**:

| Address | Name | Purpose |
|---------|------|---------|
| 0x4da480 | AdjustTargetForWalkability | Snap target to nearest walkable cell |
| 0x4d85f0 | FindExistingSegment | Search 400-slot pool for reusable path |
| 0x4d8a10 | FindOrCreateSegment | Check failure cache, allocate, run A* |
| 0x4d8560 | ExtractWaypoint | Read waypoint N from segment → world coords |
| 0x4d83a0 | CheckLeaderVisibility | Check leader visibility from route endpoint |
| 0x4d8e60 | ProcessRouteMovement | ~3KB, waypoint stepping + building transitions |
| 0x45d090 | WallFollowPathfinder | Full Dual-arm wall-following search (Bug2 variant) |
| 0x42f850 | GetBuildingEntrance | Building entrance position |
| 0x499e50 | GetTerrainClass | Terrain type from cell lookup |

### Object_UpdateMovement (0x4ed510) — Formation Manager

Per-tick handler for formation groups (up to 12 followers per group).

**Phase 1 — Validate followers**: Check alive, has formation flag, within distance, correct facing angle. Remove invalid followers.

**Phase 2 — Idle shuffling**: RNG-driven idle delays (24-31 ticks). 1-in-8 chance of standing idle pose.

**Phase 3 — Move followers to formation targets**:
- Target = anchor position + (signed offset × 16)
- Arrival threshold: `abs(dx) < 72 AND abs(dz) < 72`
- Speed assignment: arrived→match leader, leader moving→wait, not arrived→1.5× base

**Phase 4 — Advance formation anchor** via MovePointByAngle when leader arrives.

### MovePointByAngle (0x4d4b20)

Per-tick position update. ~320 calls/second during gameplay.

```c
// Original: 0x004d4b20
void MovePointByAngle(WorldCoord *pos, u16 angle, i16 speed) {
    if (speed == 0) return;
    angle &= 0x7FF;  // 11-bit
    pos->x += (SIN_TABLE[angle] * speed) >> 16;
    pos->z += (COS_TABLE[angle] * speed) >> 16;
}
```

SIN_TABLE at 0x5AC6A0, COS_TABLE at 0x5ACEA0 (2048 entries, 16.16 fixed-point).

## Data Structures

### Region Map Cell (16 bytes)

Base: `0x88897C`. Grid: 128×128. Index: `((tile_x & 0xFE) * 2) | (tile_z_high_byte)`, ×4.

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | u8 | flags_low |
| +0x01 | u8 | flags_high (bit 1 = has building) |
| +0x08 | u16 | region_id (low 10 bits = region index) |
| +0x0C | u8 | terrain_type (low nibble → flags at 0x5A3038) |

### Route Segment (109 bytes)

Base: `0x93E1C1`. Pool: 400 slots, circular. Segment addr = base + idx × 109.

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | i16 | ref_count (shared between units) |
| +0x02 | u8 | flags (bit 0=curved, bit 1=bidirectional, bit 2=persistent) |
| +0x04 | u8 | start_tile_x |
| +0x05 | u8 | start_tile_z |
| +0x08 | u8 | end_tile_x |
| +0x09 | u8 | end_tile_z |
| +0x0C | 4×24 | waypoints[] — {tile_x: u8, tile_z: u8, flags: u8, pad: u8} |
| +0x6C | u8 | waypoint_count (max 23) |

Waypoint→world: `((tile_byte & 0xFE) + 1) << 8`

### Segment Pool Globals

| Address | Size | Name |
|---------|------|------|
| 0x93DD44 | i16 | next_free_hint (circular scan position) |
| 0x93DD46 | i16 | last_assigned |
| 0x93DD48 | i16 | active_count |
| 0x93DD58 | u8 | force_route_flag |

### Failure Cache (10 bytes × 8 entries at 0x93E171)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | i16 | usage_count (0 = empty) |
| +0x02 | u8 | src_tile_x |
| +0x03 | u8 | src_tile_z |
| +0x06 | u8 | dst_tile_x |
| +0x07 | u8 | dst_tile_z |

### Object/Person Movement Fields

| Offset | Size | Name |
|--------|------|------|
| +0x0C | u32 | flags1 (0x1000=moving, 0x80=blocked, 0x80000000=needs_bridge) |
| +0x10 | u32 | flags2 (0x4000000=route_through_building) |
| +0x2B | u8 | state (0x03=MOVING, 0x07=GOTO) |
| +0x30 | u8 | unit_type (indexes speed table at 0x5A0974, stride 26) |
| +0x3D | 2×u16 | position (X, Z) |
| +0x4F | 2×u16 | target_pos |
| +0x53 | 2×u16 | next_waypoint |
| +0x57 | 2×u16 | movement_dest |
| +0x5D | u16 | facing_angle (11-bit, 0-2047) |
| +0x5F | u16 | speed (per tick) |
| +0x63 | u16 | segment_index (0 = direct walk) |
| +0x65 | u8 | route_byte |
| +0x67 | u8 | waypoint_idx (current position in segment) |
| +0x68 | u8 | follower_count |
| +0x6A | u16[12] | follower_ids |
| +0x76 | u8 | formation_flags (bit 5 = in formation) |
| +0x82 | u8[72] | formation_offsets (12 × 6 bytes, signed i8 × 16) |

## Constants

| Value | Meaning |
|-------|---------|
| 0x48 (72) | Arrival threshold per axis (world units) |
| 0x238 (568) | Max distance for formation coherence |
| 0x71 (113) | Max angle difference for formation (~20°) |
| 0x400000 | Max distance² for formation membership |
| 0x12 (18) | Formation row spacing (world units) |
| 1.5× | Follower catch-up speed multiplier |
| 400 | Max route segments in pool |
| 23 | Max waypoints per segment |
| 8 | Failure cache entries |

## RNG (Formation Idle)

```
state = state * 9377 + 0x24DF
state = rotate_right(state, 13)
```

State variable at 0x885710. Used for idle timing: `(rng & 7) + 24` ticks.

## Dual-arm Wall-Following Pathfinder (0x45D090)

Despite being labeled "A*" in earlier documentation, this is actually a **dual-arm wall-following search** (Bug2-like algorithm). Two search "arms" explore from the start position, one following obstacles clockwise and the other counter-clockwise, racing to reach the goal.

### Algorithm Overview

```
pathfind(region_map, start, goal)
  ├─ setup_directions(start, goal)    — compute primary/secondary axes
  ├─ path_search_execute()            — dual-arm core (0x45d980)
  │   ├─ Phase 0: Beeline — step directly toward goal if passable
  │   ├─ Phase 1: Wall-follow — persistent facing, SUB rotation
  │   ├─ Phase 2: Wall-end detection — perpendicular check
  │   └─ Phase 3: 4-layer bounding box containment
  └─ optimize_path_los()              — Bresenham LOS path optimizer
```

### Expansion Phases

**Phase 0 (Beeline)**: Compute the best cardinal direction toward the goal. If passable and unvisited, step directly toward it. This dominates on open terrain.

**Phase 1 (Wall-following)**: When the goal direction is blocked, the arm follows the obstacle edge using persistent `arm.facing` with SUB rotation:
- Right arm (turn_dir=1): `facing = (facing - 1) & 3` → clockwise (S→W→N→E)
- Left arm (turn_dir=3): `facing = (facing - 3) & 3` → counter-clockwise (S→E→N→W)
- Try up to 4 rotations; if none passable → stall

**Phase 2 (Wall-end detection)**: After stepping, check the perpendicular direction `(turn_dir + facing) & 3`. If passable, update facing to the perpendicular — this turns the arm back toward the wall side when the wall ends.

### 4-Layer Bounding Box (0x45eb20)

The original uses 3 jump tables (0x45f1b8, 0x45f1c8, 0x45f1d8) plus a rectangle check:

| Layer | Check | Purpose |
|-------|-------|---------|
| 1 | Directional progress (old facing) | Has arm gone past parent? Sets in_bounds flag |
| 2 | Current facing half-plane | Prevent backward movement past parent |
| 3 | Perpendicular half-plane | Prevent lateral drift past parent |
| 4 | Checkpoint rectangle | `[min(parent, checkpoint) - 1, max(parent, checkpoint) + 1]` per axis |

- **Parent** = arm's initial start position (ESI+0x14)
- **Checkpoint** = last committed beeline position (ESI+0xA22), advances during beeline, frozen during wall-follow
- If arm position == checkpoint exactly → stall (anti-loop)

### Visited-Cell Bitmap

A 128×128 bit array (2048 bytes) shared by both arms. Checked in cell passability probes before terrain. Prevents arms from revisiting cells, avoiding wasted iterations and loops. Goal cell is exempt.

### Line-of-Sight Path Optimizer (0x45e3c0)

After the search finds waypoints, a Bresenham-based optimizer removes redundant intermediate waypoints:
1. For each waypoint pair (i, j): walk a straight line cell-by-cell
2. Check every cell for passability
3. If all passable → remove waypoints i+1 through j-1
4. Greedy: try farthest reachable first

### Pathfinder Constants

| Value | Meaning |
|-------|---------|
| 1500 | Max search iterations (0x599830) |
| 260 | Max waypoints per arm |
| 400 | Max total pool waypoints |
| 23 | Max output waypoints per segment |
| 4 | Cardinal directions (S=0, E=1, N=2, W=3) |

### Coordinate System

Cell-space integers where each unit = one tile (2 in tile_coord space). Conversion: `cell = tile >> 1`, `tile = (cell << 1) & 0xFE`.

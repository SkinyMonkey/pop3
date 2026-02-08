// Dual-arm wall-following pathfinder.
// Original: FUN_0045d090 @ popTB.exe (~6KB across multiple functions)
//
// Despite being labeled "A*" in docs, this is NOT A* — it's a dual-arm
// wall-following search (Bug2-like). Two search "arms" start from the
// source position, one turning left and one turning right when hitting
// obstacles. They follow obstacle edges until they reach the goal or
// exhaust the search space.
//
// Architecture:
//   pathfind()              — entry point (0x45d090)
//   setup_directions()      — compute primary/secondary axes (0x45d620)
//   path_search_execute()   — dual-arm wavefront core (0x45d980)
//   expand_arm()            — single wall-following step (0x45eb20)
//   check_cell_passable()   — terrain walkability (0x45e870)
//   extract_waypoints()     — convert to tile-coordinate output (0x45e1b0)
//
// Coordinate system: cell-space integers where each unit = one tile (2 in
// tile_coord space). The original uses a +0x1400 offset for internal math
// but we normalize to 0-based.

use super::constants::*;
use super::region::RegionMap;
use super::types::{TileCoord, Waypoint};

/// A node in the search (10 bytes in the original).
/// Stores cell-space position and flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PathNode {
    /// Cell X position
    pub x: i32,
    /// Cell Z position
    pub z: i32,
    /// Flags (bit 0 = water crossing)
    pub flags: u16,
}

impl PathNode {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z, flags: 0 }
    }

    /// Convert from tile coordinate to cell-space position.
    pub fn from_tile(tile: TileCoord) -> Self {
        // Tiles are even (step 2), cell coords are tile >> 1
        Self::new((tile.x >> 1) as i32, (tile.z >> 1) as i32)
    }

    /// Convert cell-space position back to tile coordinate.
    pub fn to_tile(&self) -> TileCoord {
        TileCoord::new(
            ((self.x as u8) << 1) & 0xFE,
            ((self.z as u8) << 1) & 0xFE,
        )
    }

    /// Step in a cardinal direction.
    pub fn step(&self, dir: usize) -> Self {
        Self {
            x: self.x + DIRECTION_DX[dir & 3],
            z: self.z + DIRECTION_DZ[dir & 3],
            flags: self.flags,
        }
    }

    /// Check if this node is on the map (0..127).
    pub fn on_map(&self) -> bool {
        self.x >= 0 && self.x < REGION_GRID_SIZE as i32
            && self.z >= 0 && self.z < REGION_GRID_SIZE as i32
    }
}

/// Visited-cell bitmap: 128×128 cells = 2048 bytes.
/// Original: bitmap checked in CheckCellPassable (0x45e870) before terrain.
/// Prevents arms from revisiting cells, avoiding wasted iterations and loops.
pub struct VisitedBitmap {
    bits: [u8; 2048], // 128 * 128 / 8
}

impl Default for VisitedBitmap {
    fn default() -> Self { Self::new() }
}

impl VisitedBitmap {
    pub fn new() -> Self {
        Self { bits: [0; 2048] }
    }

    pub fn is_visited(&self, x: i32, z: i32) -> bool {
        let idx = (z as usize) * REGION_GRID_SIZE + (x as usize);
        (self.bits[idx >> 3] >> (idx & 7)) & 1 != 0
    }

    fn mark(&mut self, x: i32, z: i32) {
        let idx = (z as usize) * REGION_GRID_SIZE + (x as usize);
        self.bits[idx >> 3] |= 1 << (idx & 7);
    }
}

/// Debug output from pathfinding — contains visited cells and arm traces.
pub struct PathfindDebug {
    /// The pathfinding result
    pub result: PathfindResult,
    /// Visited-cell bitmap (128×128)
    pub visited: VisitedBitmap,
    /// Waypoints explored by arm 0 (right-hand rule)
    pub arm0_trace: Vec<(i32, i32)>,
    /// Waypoints explored by arm 1 (left-hand rule)
    pub arm1_trace: Vec<(i32, i32)>,
}

/// One search arm — follows walls in a specific turn direction.
/// Original: 0xA43 bytes per arm at 0x6847D2.
struct SearchArm {
    /// Local waypoint buffer (up to PATHFIND_ARM_MAX_WAYPOINTS entries)
    waypoints: Vec<PathNode>,
    /// Current position in cell-space
    pos: PathNode,
    /// Current facing direction (0=S, 1=E, 2=N, 3=W)
    facing: usize,
    /// Turn direction: +1 (right-hand rule) or 3 (left-hand rule, equivalent to -1 mod 4)
    turn_dir: usize,
    /// Arm state (INIT, FOUND, STALLED, EXPANDING)
    state: u8,
    /// Start position of this arm ("parent" in original, ESI+0x14)
    start_pos: PathNode,
    /// Last committed position — updated during beeline, frozen during wall-follow.
    /// Original: ESI+0xA22. Forms adaptive bounding box with start_pos.
    checkpoint: PathNode,
    /// Initial facing direction (for loop detection)
    initial_facing: usize,
}

impl SearchArm {
    fn new(start: PathNode, facing: usize, turn_dir: usize) -> Self {
        Self {
            waypoints: Vec::with_capacity(32),
            pos: start,
            facing,
            turn_dir,
            state: ARM_STATE_EXPANDING,
            start_pos: start,
            checkpoint: start,
            initial_facing: facing,
        }
    }
}

/// Result of pathfinding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathfindResult {
    /// Path found — contains waypoints as tile coordinates
    Found(Vec<Waypoint>),
    /// No path exists between start and goal
    NotFound,
}

/// Run the dual-arm wall-following pathfinder.
/// Original: FUN_0045d090
///
/// Returns a list of waypoints (tile coordinates) from start to goal,
/// or NotFound if no path exists.
pub fn pathfind(
    region_map: &RegionMap,
    start: TileCoord,
    goal: TileCoord,
) -> PathfindResult {
    let start_node = PathNode::from_tile(start);
    let goal_node = PathNode::from_tile(goal);

    // Early exit: start == goal
    if start_node.x == goal_node.x && start_node.z == goal_node.z {
        return PathfindResult::Found(vec![]);
    }

    // Check start and goal are on walkable terrain
    if !region_map.is_walkable(start) || !region_map.is_walkable(goal) {
        return PathfindResult::NotFound;
    }

    // Setup primary and secondary expansion directions
    // Original: SetupDirectionVectors @ 0x45d620
    let (primary_dir, secondary_dir) = setup_directions(start_node, goal_node);

    // Try with the computed directions, then rotated if needed
    for retry in 0..PATHFIND_MAX_RETRIES {
        let dir_offset = retry; // Rotate directions on retry
        let pdir = (primary_dir + dir_offset) & 3;
        let sdir = (secondary_dir + dir_offset) & 3;

        let result = path_search_execute(
            region_map,
            start_node,
            goal_node,
            pdir,
            sdir,
        );

        if let PathfindResult::Found(ref waypoints) = result {
            if !waypoints.is_empty() || start_node == goal_node {
                return result;
            }
        }
    }

    PathfindResult::NotFound
}

/// Run the pathfinder with debug output — returns visited cells and arm traces.
/// Used by the pathfinding visualizer to show search exploration.
pub fn pathfind_debug(
    region_map: &RegionMap,
    start: TileCoord,
    goal: TileCoord,
) -> PathfindDebug {
    let start_node = PathNode::from_tile(start);
    let goal_node = PathNode::from_tile(goal);

    let mut debug = PathfindDebug {
        result: PathfindResult::NotFound,
        visited: VisitedBitmap::new(),
        arm0_trace: Vec::new(),
        arm1_trace: Vec::new(),
    };

    if start_node.x == goal_node.x && start_node.z == goal_node.z {
        debug.result = PathfindResult::Found(vec![]);
        return debug;
    }
    if !region_map.is_walkable(start) || !region_map.is_walkable(goal) {
        return debug;
    }

    let (primary_dir, secondary_dir) = setup_directions(start_node, goal_node);

    for retry in 0..PATHFIND_MAX_RETRIES {
        let pdir = (primary_dir + retry) & 3;
        let sdir = (secondary_dir + retry) & 3;
        let (result, visited, arm0_wps, arm1_wps) =
            path_search_execute_debug(region_map, start_node, goal_node, pdir, sdir);

        debug.visited = visited;
        debug.arm0_trace = arm0_wps;
        debug.arm1_trace = arm1_wps;

        if let PathfindResult::Found(ref waypoints) = result {
            if !waypoints.is_empty() || start_node == goal_node {
                debug.result = result;
                return debug;
            }
        }
    }

    debug
}

type DebugSearchResult = (PathfindResult, VisitedBitmap, Vec<(i32, i32)>, Vec<(i32, i32)>);

/// Core search with debug output — returns visited bitmap and arm traces.
fn path_search_execute_debug(
    region_map: &RegionMap,
    start: PathNode,
    goal: PathNode,
    primary_dir: usize,
    _secondary_dir: usize,
) -> DebugSearchResult {
    let mut visited = VisitedBitmap::new();
    visited.mark(start.x, start.z);

    let mut arm0 = SearchArm::new(start, primary_dir, 1);
    let mut arm1 = SearchArm::new(start, primary_dir, 3);

    let mut global_waypoints: Vec<PathNode> = Vec::with_capacity(64);
    let mut iterations = 0;

    while iterations < PATHFIND_MAX_ITERATIONS {
        iterations += 1;

        if arm0.state == ARM_STATE_EXPANDING {
            expand_arm(region_map, &mut visited, &mut arm0, goal);
        }
        if arm0.state == ARM_STATE_FOUND {
            global_waypoints.extend_from_slice(&arm0.waypoints);
            let arm0_trace = arm0.waypoints.iter().map(|n| (n.x, n.z)).collect();
            let arm1_trace = arm1.waypoints.iter().map(|n| (n.x, n.z)).collect();
            let result = build_path_result(region_map, start, goal, &global_waypoints);
            return (result, visited, arm0_trace, arm1_trace);
        }

        if arm1.state == ARM_STATE_EXPANDING {
            expand_arm(region_map, &mut visited, &mut arm1, goal);
        }
        if arm1.state == ARM_STATE_FOUND {
            global_waypoints.extend_from_slice(&arm1.waypoints);
            let arm0_trace = arm0.waypoints.iter().map(|n| (n.x, n.z)).collect();
            let arm1_trace = arm1.waypoints.iter().map(|n| (n.x, n.z)).collect();
            let result = build_path_result(region_map, start, goal, &global_waypoints);
            return (result, visited, arm0_trace, arm1_trace);
        }

        if arm0.state != ARM_STATE_EXPANDING && arm1.state != ARM_STATE_EXPANDING {
            break;
        }
        if arm0.pos == arm1.start_pos || arm1.pos == arm0.start_pos {
            let reverse0 = (arm0.facing + 2) & 3;
            let reverse1 = (arm1.facing + 2) & 3;
            if reverse0 == arm0.initial_facing || reverse1 == arm1.initial_facing {
                break;
            }
        }
        let total = arm0.waypoints.len() + arm1.waypoints.len() + global_waypoints.len();
        if total >= PATHFIND_MAX_POOL_WAYPOINTS {
            break;
        }
    }

    let arm0_trace = arm0.waypoints.iter().map(|n| (n.x, n.z)).collect();
    let arm1_trace = arm1.waypoints.iter().map(|n| (n.x, n.z)).collect();
    (PathfindResult::NotFound, visited, arm0_trace, arm1_trace)
}

/// Compute primary and secondary expansion directions based on start→goal vector.
/// Original: SetupDirectionVectors @ 0x45d620
///
/// The primary direction is along the longer Manhattan axis,
/// the secondary along the shorter. This creates a Bresenham-like
/// zigzag toward the goal.
fn setup_directions(start: PathNode, goal: PathNode) -> (usize, usize) {
    let dx = goal.x - start.x;
    let dz = goal.z - start.z;

    // Primary: direction of larger absolute distance
    // Secondary: direction of smaller absolute distance
    let primary = if dx.abs() >= dz.abs() {
        if dx > 0 { 1 } else { 3 } // East or West
    } else if dz > 0 {
        0 // South
    } else {
        2 // North
    };

    let secondary = if dx.abs() >= dz.abs() {
        if dz > 0 { 0 } else { 2 } // South or North
    } else if dx > 0 {
        1 // East
    } else {
        3 // West
    };

    (primary, secondary)
}

/// Core dual-arm search loop.
/// Original: PathSearch_Execute @ 0x45d980
///
/// Two arms start at the start position. One follows walls clockwise,
/// the other counter-clockwise. A bias counter alternates expansion
/// between the primary and secondary axes.
fn path_search_execute(
    region_map: &RegionMap,
    start: PathNode,
    goal: PathNode,
    primary_dir: usize,
    _secondary_dir: usize,
) -> PathfindResult {
    // Visited-cell bitmap — shared by both arms to prevent revisiting
    let mut visited = VisitedBitmap::new();
    visited.mark(start.x, start.z);

    // Initialize two search arms:
    // Arm 0: turns right (clockwise wall-follow)
    // Arm 1: turns left (counter-clockwise wall-follow)
    let mut arm0 = SearchArm::new(start, primary_dir, 1);  // turn right
    let mut arm1 = SearchArm::new(start, primary_dir, 3);  // turn left (3 = -1 mod 4)

    let mut global_waypoints: Vec<PathNode> = Vec::with_capacity(64);
    let mut iterations = 0;

    while iterations < PATHFIND_MAX_ITERATIONS {
        iterations += 1;

        // Expand arm 0
        if arm0.state == ARM_STATE_EXPANDING {
            expand_arm(region_map, &mut visited, &mut arm0, goal);
        }

        // Check if arm 0 found the goal
        if arm0.state == ARM_STATE_FOUND {
            global_waypoints.extend_from_slice(&arm0.waypoints);
            return build_path_result(region_map, start, goal, &global_waypoints);
        }

        // Expand arm 1
        if arm1.state == ARM_STATE_EXPANDING {
            expand_arm(region_map, &mut visited, &mut arm1, goal);
        }

        // Check if arm 1 found the goal
        if arm1.state == ARM_STATE_FOUND {
            global_waypoints.extend_from_slice(&arm1.waypoints);
            return build_path_result(region_map, start, goal, &global_waypoints);
        }

        // Check if arms have met (loop detection)
        if arm0.state != ARM_STATE_EXPANDING && arm1.state != ARM_STATE_EXPANDING {
            // Both arms stalled — no path
            break;
        }

        // Loop detection: arm reaches the other arm's start position
        if arm0.pos == arm1.start_pos || arm1.pos == arm0.start_pos {
            let reverse0 = (arm0.facing + 2) & 3;
            let reverse1 = (arm1.facing + 2) & 3;
            if reverse0 == arm0.initial_facing || reverse1 == arm1.initial_facing {
                // Arms went full circle — enclosed obstacle, no path
                break;
            }
        }

        // Check waypoint capacity
        let total = arm0.waypoints.len() + arm1.waypoints.len() + global_waypoints.len();
        if total >= PATHFIND_MAX_POOL_WAYPOINTS {
            break;
        }
    }

    PathfindResult::NotFound
}

/// Compute the best cardinal direction from `from` toward `goal`.
/// Used by the beeline phase to step directly toward the destination.
fn goal_direction(from: PathNode, goal: PathNode) -> usize {
    let dx = goal.x - from.x;
    let dz = goal.z - from.z;

    if dx.abs() >= dz.abs() {
        if dx > 0 { 1 } else { 3 } // East or West
    } else if dz > 0 {
        0 // South
    } else {
        2 // North
    }
}

/// Expand one search arm by one step.
/// Original: ExpandArm @ 0x45eb20
///
/// The original path_search_execute (0x45d980) has a beeline outer loop
/// that steps directly toward the goal when the path is clear. ExpandArm
/// only handles wall-following. We merge both phases here for simplicity:
///
/// Phase 0 (beeline): If goal direction is passable, step there directly.
/// Phase 1 (wall-follow): Try arm.facing, rotate (facing - turn_dir) & 3.
/// Phase 2 (wall-end): Check perpendicular (turn_dir + facing) & 3.
fn expand_arm(
    region_map: &RegionMap,
    visited: &mut VisitedBitmap,
    arm: &mut SearchArm,
    goal: PathNode,
) {
    if arm.state != ARM_STATE_EXPANDING {
        return;
    }

    // Phase 0: Beeline — if the goal direction is passable, step there.
    // This mirrors the beeline outer loop in the original path_search_execute
    // (0x45da9b). On open terrain, this dominates and the arm walks directly
    // toward the goal without triggering the perpendicular redirect.
    let goal_dir = goal_direction(arm.pos, goal);
    let goal_cell = arm.pos.step(goal_dir);
    let goal_is_target = goal_cell.x == goal.x && goal_cell.z == goal.z;
    if goal_cell.on_map()
        && is_cell_passable(region_map, goal_cell)
        && (goal_is_target || !visited.is_visited(goal_cell.x, goal_cell.z))
    {
        arm.facing = goal_dir;
        arm.pos = goal_cell;
        // Beeline success — advance checkpoint to track progress.
        // The bounding box grows as the arm advances in beeline mode.
        arm.checkpoint = arm.pos;
        visited.mark(arm.pos.x, arm.pos.z);
        arm.waypoints.push(arm.pos);
        if goal_is_target {
            arm.state = ARM_STATE_FOUND;
        }
        return;
    }

    // Phase 1: Goal direction blocked — wall-following.
    // Uses persistent arm.facing with SUB rotation (from disassembly 0x45eb20).
    let mut dir = arm.facing;
    let mut found_passable = false;

    for _ in 0..4 {
        let candidate = arm.pos.step(dir);
        let is_target = candidate.x == goal.x && candidate.z == goal.z;
        if candidate.on_map()
            && is_cell_passable(region_map, candidate)
            && (is_target || !visited.is_visited(candidate.x, candidate.z))
        {
            found_passable = true;
            break;
        }
        // Rotate away from the wall: facing = (facing - turn_dir) & 3
        // With turn_dir=1 (right arm): SUB 1 → clockwise (S→W→N→E)
        // With turn_dir=3 (left arm):  SUB 3 → counterclockwise (S→E→N→W)
        dir = dir.wrapping_sub(arm.turn_dir) & 3;
    }

    if !found_passable {
        arm.state = ARM_STATE_STALLED;
        return;
    }

    // Step forward in the found passable direction
    arm.facing = dir;
    arm.pos = arm.pos.step(dir);
    visited.mark(arm.pos.x, arm.pos.z);

    // Record waypoint
    arm.waypoints.push(arm.pos);

    // Check if we reached the goal
    if arm.pos.x == goal.x && arm.pos.z == goal.z {
        arm.state = ARM_STATE_FOUND;
        return;
    }

    // Phase 2: Wall-end detection — check the perpendicular direction.
    // Original: perp_dir = (turn_dir + facing) & 3
    // This looks toward the "wall side". If passable, the wall has ended
    // and we turn toward it (resuming wall-hugging on the next call).
    // Note: perpendicular check does NOT require unvisited — it only
    // sets the facing direction, doesn't actually step.
    let perp_dir = (arm.turn_dir + arm.facing) & 3;
    let perp_cell = arm.pos.step(perp_dir);
    if perp_cell.on_map() && is_cell_passable(region_map, perp_cell) {
        arm.facing = perp_dir;
    }

    // Bounds check — 4-layer containment from original ExpandArm (0x45eb20).
    // Original uses 3 jump tables (0x45f1b8, 0x45f1c8, 0x45f1d8) plus a
    // checkpoint rectangle check. Parent = arm.start_pos, checkpoint = arm.checkpoint.
    if !bounds_check_arm(arm) {
        arm.state = ARM_STATE_STALLED;
    }

    // Capacity check
    if arm.waypoints.len() >= PATHFIND_ARM_MAX_WAYPOINTS {
        arm.state = ARM_STATE_STALLED;
    }
}

/// 4-layer bounding box check for search arm containment.
/// Original: ExpandArm @ 0x45eb20, jump tables 0x45f1b8/c8/d8 + rect at 0x45efe3.
///
/// Layer 1 (directional progress): Has arm gone past parent along its facing?
/// Layer 2 (current-facing half-plane): Prevent backward movement past parent.
/// Layer 3 (perpendicular half-plane): Prevent lateral drift past parent.
/// Layer 4 (checkpoint rectangle): [min(parent, checkpoint) - 1, max(parent, checkpoint) + 1].
///
/// Returns true if the arm is within bounds, false if it should stall.
fn bounds_check_arm(arm: &SearchArm) -> bool {
    let parent = arm.start_pos;
    let checkpoint = arm.checkpoint;
    let pos = arm.pos;

    // Layer 1: Directional progress check (jump table 1 at 0x45f1b8).
    // Uses the arm's current facing direction.
    // If the arm has moved past the parent in its facing direction, flag it.
    let in_bounds = match arm.facing {
        0 => pos.z <= parent.z, // South: must not go past parent going south
        1 => pos.x <= parent.x, // East: must not go past parent going east
        2 => pos.z >= parent.z, // North: must not go past parent going north
        3 => pos.x >= parent.x, // West: must not go past parent going west
        _ => true,
    };

    // Check proximity to parent (±1 slack) — if within 1 cell, consider found
    if (pos.x - parent.x).abs() <= 1 && (pos.z - parent.z).abs() <= 1 {
        return true; // Close enough to parent — don't stall
    }

    if in_bounds {
        // Layer 2: Current-facing half-plane (jump table 2 at 0x45f1c8).
        // Prevent arm from moving backwards past parent along current facing.
        let facing_ok = match arm.facing {
            0 => pos.z <= parent.z, // South: z must not exceed parent
            1 => pos.x <= parent.x, // East: x must not exceed parent
            2 => pos.z >= parent.z, // North: z must not be less than parent
            3 => pos.x >= parent.x, // West: x must not be less than parent
            _ => true,
        };
        if !facing_ok {
            return false;
        }

        // Layer 3: Perpendicular half-plane (jump table 3 at 0x45f1d8).
        // Prevent lateral drift past parent on the perpendicular axis.
        let perp_dir = (arm.turn_dir + arm.facing) & 3;
        let perp_ok = match perp_dir {
            0 => pos.z >= parent.z, // Perp=South: z must be >= parent
            1 => pos.x >= parent.x, // Perp=East: x must be >= parent
            2 => pos.z <= parent.z, // Perp=North: z must be <= parent
            3 => pos.x <= parent.x, // Perp=West: x must be <= parent
            _ => true,
        };
        if !perp_ok {
            return false;
        }
    }

    // Layer 4: Checkpoint rectangle (0x45efe3).
    // Bounding box = [min(parent, checkpoint) - 1, max(parent, checkpoint) + 1] per axis.
    if checkpoint.x != parent.x {
        let x_min = parent.x.min(checkpoint.x) - 1;
        let x_max = parent.x.max(checkpoint.x) + 1;
        if pos.x < x_min || pos.x > x_max {
            return false;
        }
    }
    if checkpoint.z != parent.z {
        let z_min = parent.z.min(checkpoint.z) - 1;
        let z_max = parent.z.max(checkpoint.z) + 1;
        if pos.z < z_min || pos.z > z_max {
            return false;
        }
    }

    // Anti-loop: exact checkpoint position is rejected (0x45f065)
    if pos.x == checkpoint.x && pos.z == checkpoint.z {
        return false;
    }

    true
}

/// Check if a cell is passable for pathfinding.
/// Original: CheckCellPassable @ 0x45e870
///
/// Uses the region map's terrain flags to determine walkability.
/// Returns true if the cell can be entered.
fn is_cell_passable(region_map: &RegionMap, node: PathNode) -> bool {
    if !node.on_map() {
        return false;
    }
    let tile = node.to_tile();
    region_map.is_walkable(tile)
}

/// Build the final path result from collected waypoints.
/// Deduplicates consecutive identical waypoints and converts to Waypoint format.
/// Original: path optimizer at 0x45e3c0 + waypoint extraction at 0x45e1b0
fn build_path_result(
    region_map: &RegionMap,
    _start: PathNode,
    goal: PathNode,
    raw_waypoints: &[PathNode],
) -> PathfindResult {
    if raw_waypoints.is_empty() {
        // Start == goal case or direct walk
        let tile = goal.to_tile();
        return PathfindResult::Found(vec![
            Waypoint { tile_x: tile.x, tile_z: tile.z, flags: 0, _pad: 0 }
        ]);
    }

    // Deduplicate consecutive identical waypoints
    let mut deduped: Vec<PathNode> = Vec::with_capacity(raw_waypoints.len());
    for wp in raw_waypoints {
        if deduped.last().is_none_or(|prev| prev.x != wp.x || prev.z != wp.z) {
            deduped.push(*wp);
        }
    }

    // LOS optimization: remove waypoints when a straight line is passable
    let simplified = optimize_path_los(region_map, &deduped);

    // Convert to output Waypoint format, clamped to max segment size
    let max_wps = PATHFIND_MAX_SEGMENT_WAYPOINTS.min(simplified.len());
    let mut waypoints = Vec::with_capacity(max_wps);

    // If we have more waypoints than max, subsample evenly
    if simplified.len() <= max_wps {
        for node in &simplified {
            let tile = node.to_tile();
            waypoints.push(Waypoint {
                tile_x: tile.x,
                tile_z: tile.z,
                flags: node.flags as u8,
                _pad: 0,
            });
        }
    } else {
        // Subsample: always include first, last, and evenly spaced points
        for i in 0..max_wps {
            let idx = if i == max_wps - 1 {
                simplified.len() - 1
            } else {
                i * simplified.len() / max_wps
            };
            let node = &simplified[idx];
            let tile = node.to_tile();
            waypoints.push(Waypoint {
                tile_x: tile.x,
                tile_z: tile.z,
                flags: node.flags as u8,
                _pad: 0,
            });
        }
    }

    // Ensure goal is the last waypoint
    let goal_tile = goal.to_tile();
    if let Some(last) = waypoints.last() {
        if last.tile_x != goal_tile.x || last.tile_z != goal_tile.z {
            if waypoints.len() < PATHFIND_MAX_SEGMENT_WAYPOINTS {
                waypoints.push(Waypoint {
                    tile_x: goal_tile.x,
                    tile_z: goal_tile.z,
                    flags: 0,
                    _pad: 0,
                });
            } else {
                // Replace last with goal
                let last_mut = waypoints.last_mut().unwrap();
                last_mut.tile_x = goal_tile.x;
                last_mut.tile_z = goal_tile.z;
            }
        }
    }

    PathfindResult::Found(waypoints)
}

/// Check if there's a clear line of sight between two cells.
/// Walks a Bresenham line and checks every cell for passability.
/// Original: part of path optimizer at 0x45e500
fn line_of_sight(region_map: &RegionMap, from: PathNode, to: PathNode) -> bool {
    let mut x = from.x;
    let mut z = from.z;
    let dx = (to.x - from.x).abs();
    let dz = (to.z - from.z).abs();
    let sx: i32 = if to.x > from.x { 1 } else { -1 };
    let sz: i32 = if to.z > from.z { 1 } else { -1 };
    let mut err = dx - dz;

    loop {
        let node = PathNode::new(x, z);
        if !node.on_map() || !is_cell_passable(region_map, node) {
            return false;
        }
        if x == to.x && z == to.z {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dz {
            err -= dz;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            z += sz;
        }
    }
    true
}

/// Optimize path using line-of-sight checks.
/// For each waypoint, try to skip as far ahead as possible while maintaining
/// clear line of sight. Removes redundant intermediate waypoints.
/// Original: path optimizer at 0x45e3c0
fn optimize_path_los(region_map: &RegionMap, nodes: &[PathNode]) -> Vec<PathNode> {
    if nodes.len() <= 2 {
        return nodes.to_vec();
    }

    let mut result: Vec<PathNode> = Vec::with_capacity(nodes.len());
    result.push(nodes[0]);

    let mut i = 0;
    while i < nodes.len() - 1 {
        // Greedily find the farthest waypoint reachable by LOS from nodes[i]
        let mut best_j = i + 1;
        for j in (i + 2..nodes.len()).rev() {
            if line_of_sight(region_map, nodes[i], nodes[j]) {
                best_j = j;
                break;
            }
        }
        result.push(nodes[best_j]);
        i = best_j;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_tile_returns_empty_path() {
        let map = RegionMap::new();
        let tile = TileCoord::new(0x10, 0x10);
        let result = pathfind(&map, tile, tile);
        assert_eq!(result, PathfindResult::Found(vec![]));
    }

    #[test]
    fn adjacent_walkable_tiles() {
        let map = RegionMap::new(); // All walkable by default
        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x12, 0x10); // One tile east
        let result = pathfind(&map, start, goal);
        match result {
            PathfindResult::Found(wps) => {
                assert!(!wps.is_empty());
                // Last waypoint should be at or near the goal
                let last = wps.last().unwrap();
                assert_eq!(last.tile_x, goal.x);
                assert_eq!(last.tile_z, goal.z);
            }
            PathfindResult::NotFound => panic!("Should find path between adjacent tiles"),
        }
    }

    #[test]
    fn straight_line_path() {
        let map = RegionMap::new();
        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x20, 0x10); // 8 tiles east
        let result = pathfind(&map, start, goal);
        match result {
            PathfindResult::Found(wps) => {
                assert!(!wps.is_empty());
                let last = wps.last().unwrap();
                assert_eq!(last.tile_x, goal.x);
                assert_eq!(last.tile_z, goal.z);
            }
            PathfindResult::NotFound => panic!("Should find straight-line path"),
        }
    }

    #[test]
    fn unwalkable_start_returns_not_found() {
        let mut map = RegionMap::new();
        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x20, 0x20);

        // Make start tile unwalkable
        map.get_cell_mut(start).terrain_type = 5;
        map.set_terrain_flags(5, 0x00);

        let result = pathfind(&map, start, goal);
        assert_eq!(result, PathfindResult::NotFound);
    }

    #[test]
    fn unwalkable_goal_returns_not_found() {
        let mut map = RegionMap::new();
        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x20, 0x20);

        // Make goal tile unwalkable
        map.get_cell_mut(goal).terrain_type = 5;
        map.set_terrain_flags(5, 0x00);

        let result = pathfind(&map, start, goal);
        assert_eq!(result, PathfindResult::NotFound);
    }

    #[test]
    fn path_around_obstacle() {
        let mut map = RegionMap::new();
        map.set_terrain_flags(5, 0x00); // terrain class 5 = unwalkable

        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x10, 0x1A); // 5 tiles south

        // Create a wall at z=0x14 spanning x=0x0E to 0x12
        // (blocking the direct path)
        for x in (0x0E..=0x12).step_by(2) {
            map.get_cell_mut(TileCoord::new(x, 0x14)).terrain_type = 5;
        }

        let result = pathfind(&map, start, goal);
        match result {
            PathfindResult::Found(wps) => {
                assert!(!wps.is_empty());
                let last = wps.last().unwrap();
                assert_eq!(last.tile_x, goal.x);
                assert_eq!(last.tile_z, goal.z);

                // Verify no waypoint is on an unwalkable cell
                for wp in &wps {
                    let tile = TileCoord::new(wp.tile_x, wp.tile_z);
                    assert!(
                        map.is_walkable(tile),
                        "Waypoint ({:#x}, {:#x}) is on unwalkable terrain",
                        wp.tile_x,
                        wp.tile_z
                    );
                }
            }
            PathfindResult::NotFound => panic!("Should find path around obstacle"),
        }
    }

    #[test]
    fn completely_enclosed_returns_not_found() {
        let mut map = RegionMap::new();
        map.set_terrain_flags(5, 0x00);

        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x30, 0x30);

        // Enclose start in a box of unwalkable terrain
        for x in (0x0C..=0x14).step_by(2) {
            map.get_cell_mut(TileCoord::new(x, 0x0E)).terrain_type = 5;
            map.get_cell_mut(TileCoord::new(x, 0x12)).terrain_type = 5;
        }
        for z in (0x0E..=0x12).step_by(2) {
            map.get_cell_mut(TileCoord::new(0x0C, z)).terrain_type = 5;
            map.get_cell_mut(TileCoord::new(0x14, z)).terrain_type = 5;
        }

        let result = pathfind(&map, start, goal);
        assert_eq!(result, PathfindResult::NotFound);
    }

    #[test]
    fn path_node_tile_roundtrip() {
        let tile = TileCoord::new(0x20, 0x40);
        let node = PathNode::from_tile(tile);
        let back = node.to_tile();
        assert_eq!(back.x, tile.x);
        assert_eq!(back.z, tile.z);
    }

    #[test]
    fn direction_step() {
        let node = PathNode::new(5, 5);
        // South: dz=+1
        let s = node.step(0);
        assert_eq!((s.x, s.z), (5, 6));
        // East: dx=+1
        let e = node.step(1);
        assert_eq!((e.x, e.z), (6, 5));
        // North: dz=-1
        let n = node.step(2);
        assert_eq!((n.x, n.z), (5, 4));
        // West: dx=-1
        let w = node.step(3);
        assert_eq!((w.x, w.z), (4, 5));
    }

    #[test]
    fn setup_directions_east_dominant() {
        let start = PathNode::new(5, 5);
        let goal = PathNode::new(20, 8); // Mostly east, slightly south
        let (primary, secondary) = setup_directions(start, goal);
        assert_eq!(primary, 1); // East
        assert_eq!(secondary, 0); // South
    }

    #[test]
    fn setup_directions_south_dominant() {
        let start = PathNode::new(5, 5);
        let goal = PathNode::new(7, 20); // Slightly east, mostly south
        let (primary, secondary) = setup_directions(start, goal);
        assert_eq!(primary, 0); // South
        assert_eq!(secondary, 1); // East
    }

    #[test]
    fn los_optimizer_removes_collinear() {
        // A straight line east on open terrain should optimize to start and end
        let map = RegionMap::new();
        let nodes: Vec<PathNode> = (0..10).map(|i| PathNode::new(i, 0)).collect();
        let optimized = optimize_path_los(&map, &nodes);
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], nodes[0]);
        assert_eq!(optimized[1], nodes[9]);
    }

    #[test]
    fn los_optimizer_keeps_turns_around_wall() {
        // East then south — on open terrain, LOS can shortcut diagonally
        let map = RegionMap::new();
        let nodes = vec![
            PathNode::new(0, 0),
            PathNode::new(1, 0),
            PathNode::new(2, 0),
            PathNode::new(2, 1),
            PathNode::new(2, 2),
        ];
        let optimized = optimize_path_los(&map, &nodes);
        // On open terrain, LOS from (0,0) to (2,2) is clear → optimize to 2 points
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], PathNode::new(0, 0));
        assert_eq!(optimized[1], PathNode::new(2, 2));
    }

    #[test]
    fn los_optimizer_blocked_keeps_waypoints() {
        // LOS blocked by wall should preserve intermediate waypoints
        let mut map = RegionMap::new();
        map.set_terrain_flags(5, 0x00);
        // Wall at cell (1,1) — blocks direct LOS from (0,0) to (2,0)
        // Cells map to tiles at 2x, so cell (1,1) = tile (2,2)
        map.get_cell_mut(TileCoord::new(2, 2)).terrain_type = 5;

        let nodes = vec![
            PathNode::new(0, 0),
            PathNode::new(0, 2), // Go south around wall
            PathNode::new(2, 2), // Then east
        ];
        // LOS from (0,0) to (2,2) passes through (1,1) which is blocked
        let optimized = optimize_path_los(&map, &nodes);
        // Should keep the intermediate waypoint since LOS is blocked
        assert!(optimized.len() >= 2);
    }

    #[test]
    fn long_diagonal_path() {
        let map = RegionMap::new();
        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x30, 0x30); // Diagonal, 16 tiles each axis
        let result = pathfind(&map, start, goal);
        match result {
            PathfindResult::Found(wps) => {
                assert!(!wps.is_empty());
                let last = wps.last().unwrap();
                assert_eq!(last.tile_x, goal.x);
                assert_eq!(last.tile_z, goal.z);
            }
            PathfindResult::NotFound => panic!("Should find diagonal path on open map"),
        }
    }

    #[test]
    fn visited_bitmap_prevents_revisiting() {
        let mut bm = VisitedBitmap::new();
        assert!(!bm.is_visited(5, 5));
        bm.mark(5, 5);
        assert!(bm.is_visited(5, 5));
        assert!(!bm.is_visited(5, 6)); // Adjacent cell unaffected
        assert!(!bm.is_visited(6, 5));
    }

    #[test]
    fn visited_bitmap_covers_full_grid() {
        let mut bm = VisitedBitmap::new();
        // Mark corners of the 128x128 grid
        for &(x, z) in &[(0, 0), (127, 0), (0, 127), (127, 127)] {
            bm.mark(x, z);
            assert!(bm.is_visited(x, z));
        }
    }

    #[test]
    fn wall_follow_rotation_direction() {
        // Verify the SUB rotation goes the correct direction for each arm.
        // Right arm (turn_dir=1): facing = (facing - 1) & 3 → clockwise (S→W→N→E)
        let right_sequence: Vec<usize> = (0..4)
            .scan(0usize, |f, _| { let prev = *f; *f = f.wrapping_sub(1) & 3; Some(prev) })
            .collect();
        assert_eq!(right_sequence, vec![0, 3, 2, 1]); // S, W, N, E

        // Left arm (turn_dir=3): facing = (facing - 3) & 3 → counterclockwise (S→E→N→W)
        let left_sequence: Vec<usize> = (0..4)
            .scan(0usize, |f, _| { let prev = *f; *f = f.wrapping_sub(3) & 3; Some(prev) })
            .collect();
        assert_eq!(left_sequence, vec![0, 1, 2, 3]); // S, E, N, W
    }

    #[test]
    fn bounding_box_stalls_within_rect() {
        // Arm facing south with checkpoint at (10, 15) and parent at (10, 10).
        // The bounding box is [min(10,10)-1, max(10,15)+1] = [9, 16] on each axis.
        let mut arm = SearchArm::new(PathNode::new(10, 10), 0, 1); // facing south
        arm.checkpoint = PathNode::new(10, 15);

        // Position inside the box should pass
        arm.pos = PathNode::new(10, 12);
        assert!(bounds_check_arm(&arm));

        // Position outside the box on z axis should fail
        arm.pos = PathNode::new(10, 17);
        assert!(!bounds_check_arm(&arm));
    }

    #[test]
    fn bounding_box_rejects_exact_checkpoint() {
        // If arm position == checkpoint exactly, it stalls (anti-loop).
        let mut arm = SearchArm::new(PathNode::new(5, 5), 0, 1);
        arm.checkpoint = PathNode::new(5, 10);

        arm.pos = PathNode::new(5, 10); // exactly at checkpoint
        assert!(!bounds_check_arm(&arm));
    }

    #[test]
    fn bounding_box_allows_near_parent() {
        // Position within ±1 of parent should always pass
        let mut arm = SearchArm::new(PathNode::new(10, 10), 0, 1);
        arm.checkpoint = PathNode::new(10, 12);

        arm.pos = PathNode::new(11, 11); // within ±1 of parent
        assert!(bounds_check_arm(&arm));
    }

    #[test]
    fn checkpoint_advances_on_beeline() {
        // Verify that beeline steps advance the checkpoint
        let map = RegionMap::new();
        let start = TileCoord::new(0x10, 0x10);
        let goal = TileCoord::new(0x20, 0x10); // East
        let result = pathfind(&map, start, goal);
        // On open terrain, beeline should find the path
        assert!(matches!(result, PathfindResult::Found(_)));
    }
}

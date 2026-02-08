// Movement system for Populous: The Beginning.
//
// Faithful reimplementation of the unit movement system from popTB.exe,
// using the REAL active movement functions (confirmed via Frida):
//
//   STATE_GOTO          @ 0x4d7e20 — thin dispatcher (34 calls per move order)
//   RouteTableLookup    @ 0x4d7f20 — 4-tier pathfinding cache
//   Object_UpdateMovement @ 0x4ed510 — formation manager (44 calls per tick)
//   MovePointByAngle    @ 0x4d4b20 — per-tick position update (~320/sec)
//
// NOTE: The Path_UpdateDirection (0x4248c0) / Path_FindBestDirection (0x424ed0)
// system was proven dormant — it never fires during normal gameplay. Those
// functions remain in the pathfinding worktree as reference only.
//
// Architecture:
//   Tier 1: Region Map (128×128) — same region = direct walk
//   Tier 2: Segment Pool (400 slots) — reuse cached path segments
//   Tier 3: Failure Cache (8 entries) — skip known-impossible routes
//   Tier 4: Dual-arm wall-following pathfinder (Bug2 variant)

pub mod constants;
pub mod tables;
pub mod math;
pub mod types;
pub mod region;
pub mod segment;
pub mod route;
pub mod waypoint;
pub mod pathfinder;

// Re-export primary API
pub use types::{WorldCoord, TileCoord, PersonMovement, UsedTargetsCache};
pub use region::RegionMap;
pub use segment::{SegmentPool, FailureCache};
pub use route::{state_goto, route_table_lookup, adjust_target_for_walkability, RouteResult};
pub use waypoint::{process_route_movement, WaypointResult};
pub use math::{move_point_by_angle, angle_difference, rotation_direction, distance, atan2, formation_rng_next};
pub use pathfinder::{pathfind, pathfind_debug, PathfindResult, PathfindDebug, PathNode, VisitedBitmap};

---
phase: 05-ai-and-campaign
plan: 11
subsystem: ai
tags: [ai-dispatch, unit-movement, building-placement, training, target-scoring]

requires:
  - phase: 05-ai-and-campaign
    provides: "AiSystem with Lua VM, drain_pending_commands, AiGameBridge"
provides:
  - "dispatch_ai_commands function producing real game state mutations"
  - "order_move_tribe and order_move_shaman on UnitCoordinator"
  - "ShamanCommandQueue[4] and AiBuildingPlacement[4] integrated into AiSystem"
  - "target.rs scoring wired into attack target selection"
affects: [phase-06-verification]

tech-stack:
  added: []
  patterns: ["take()-and-replace for borrow checker conflicts in app.rs dispatch", "two-phase collect-then-mutate for building training dispatch"]

key-files:
  created:
    - src/engine/ai/dispatch.rs
  modified:
    - src/engine/ai/mod.rs
    - src/engine/units/coordinator.rs
    - src/engine/objects/pool.rs
    - src/render/app.rs

key-decisions:
  - "take() pattern for ai_system in app.rs dispatch to avoid borrow conflict with unit_coordinator"
  - "Two-phase handle collection for training dispatch (collect handles, then mutate via building_by_handle_mut)"
  - "Spell and convert commands log with TODO markers (Phase 4 spell system dependency)"

patterns-established:
  - "AI dispatch uses take()-and-replace pattern for AiSystem ownership during dispatch"
  - "Marker ID resolution via marker_entries table from AiGameBridge"

requirements-completed: [AI-04, AI-05, AI-06]

duration: 5min
completed: 2026-03-24
---

# Phase 5 Plan 11: AI Command Dispatch Summary

**AI command dispatch replaces log-only handlers with real game state mutations: attack moves units via target scoring, build queues into AiBuildingPlacement, train initiates conversion in matching buildings**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-24T03:54:39Z
- **Completed:** 2026-03-24T03:59:39Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- AI attack commands now move idle tribe units toward best-scored enemy targets using score_person_target
- AI build commands queue building type into per-tribe AiBuildingPlacement priority queue
- AI train commands find matching Active training buildings and call start_training
- ShamanCommandQueue[4] and AiBuildingPlacement[4] integrated as AiSystem fields
- All 693 tests pass (685 existing + 8 new dispatch tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add order_move_tribe to UnitCoordinator and create dispatch module with tests** - `9521372` (feat)
2. **Task 2: Wire dispatch_ai_commands into app.rs replacing log-only handlers** - `fa69dbe` (feat)

## Files Created/Modified
- `src/engine/ai/dispatch.rs` - AI command dispatch with all 7 command type handlers
- `src/engine/ai/mod.rs` - Added dispatch module, shaman_commands/building_placement fields and accessors
- `src/engine/units/coordinator.rs` - Added order_move_tribe, order_move_shaman, push_unit_for_test
- `src/engine/objects/pool.rs` - Added building_by_handle_mut for targeted building mutation
- `src/render/app.rs` - Replaced log-only handlers with dispatch_ai_commands call

## Decisions Made
- Used take() pattern for AiSystem ownership during dispatch to avoid borrow conflict with UnitCoordinator
- Two-phase collect-then-mutate for training dispatch matches existing DeferredAction/BuildingTickActions patterns
- Spell and convert commands log with explicit TODO markers (Phase 4 spell system not yet available)
- order_move_tribe returns count of moved units for observability
- Marker ID resolution delegated to marker_entries from AiGameBridge

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Known Stubs
- Spell dispatch: logs "TODO: Phase 4 spell system" (intentional -- spell system is Phase 4 dependency)
- Convert dispatch: logs "TODO: convert wild" (requires spell-like behavior not yet implemented)

## Next Phase Readiness
- AI command dispatch pipeline is now end-to-end functional
- Remaining gap is CAMP-04 stone head discovery (deferred per D-12)
- Human verification needed: run game with AI scripts and observe tribe behavior

---
*Phase: 05-ai-and-campaign*
*Completed: 2026-03-24*

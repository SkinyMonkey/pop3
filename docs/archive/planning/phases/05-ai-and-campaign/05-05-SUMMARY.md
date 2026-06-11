---
phase: 05-ai-and-campaign
plan: 05
subsystem: ai
tags: [target-scoring, shaman-commands, building-ai, difficulty-scaling, popscript]

requires:
  - phase: 05-01
    provides: AI module skeleton with Lua VM and PopScript constants
provides:
  - Target scoring algorithm matching original binary weights
  - Shaman command queue with 10 slots and 8 command types
  - AI building placement state machine with priority ordering
  - Difficulty scaling with mana adjust and training cost bands
affects: [05-06, 05-07, 05-08]

tech-stack:
  added: []
  patterns: [binary-faithful-constants, tdd-red-green]

key-files:
  created:
    - src/engine/ai/target.rs
    - src/engine/ai/shaman_cmd.rs
    - src/engine/ai/building_ai.rs
    - src/engine/ai/difficulty.rs
  modified:
    - src/engine/ai/mod.rs

key-decisions:
  - "Fixed plan test case: score_person_target(SHAMAN, 0, 100) gets exposed bonus (0 < 3 defenders)"

patterns-established:
  - "Binary-faithful constants with address annotations in doc comments"

requirements-completed: [AI-04, AI-05, AI-06, AI-07]

duration: 4min
completed: 2026-03-24
---

# Phase 05 Plan 05: AI Behavior Modules Summary

**Target scoring, shaman commands, building AI, and difficulty scaling with binary-faithful constants from AI_FindBestAttackTarget and AI_AssessThreat**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-24T02:14:49Z
- **Completed:** 2026-03-24T02:19:05Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Target scoring with person weights (Shaman=1000, exposed bonus=+500) and building weights matching original binary
- Threat assessment weights from AI_AssessThreat @ 0x0041ba40
- ShamanCommandQueue with 10 slots, push/clear/count, all 8 command types with from_raw dispatch
- AiBuildingPlacement 7-state machine with priority queue (DrumTower > Training > Housing > Reincarnation)
- Difficulty scaling: mana adjust multiplier and 6 training cost bands (100%-200%)

## Task Commits

Each task was committed atomically:

1. **Task 1: Target scoring and threat assessment** - `33be91b` (feat)
2. **Task 2: Shaman commands, building AI, and difficulty scaling** - `b1dd7b2` (feat)

## Files Created/Modified
- `src/engine/ai/target.rs` - Target scoring (person, building) and threat assessment with binary-faithful weights
- `src/engine/ai/shaman_cmd.rs` - ShamanCommandQueue with 10 slots, 8 command types, push/clear/count
- `src/engine/ai/building_ai.rs` - AiBuildingPlacement 7-state machine with priority queue
- `src/engine/ai/difficulty.rs` - Mana adjust, training cost bands, AI training costs, DifficultyScaling
- `src/engine/ai/mod.rs` - Added module declarations for target, shaman_cmd, building_ai, difficulty

## Decisions Made
- Fixed plan test expectation: score_person_target(SHAMAN, 0, 100) returns 1499 not 999 because 0 defenders < 3 triggers exposed bonus

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed incorrect test expectation for exposed shaman scoring**
- **Found during:** Task 1 (Target scoring GREEN phase)
- **Issue:** Plan specified score_person_target(SHAMAN, 0, 100) = 999, but 0 defenders < 3 triggers exposed bonus, giving 1499
- **Fix:** Corrected test expectation to 1499 and added separate test for shaman with enough defenders (5, distance 100) = 999
- **Files modified:** src/engine/ai/target.rs
- **Verification:** All 8 target tests pass
- **Committed in:** 33be91b

---

**Total deviations:** 1 auto-fixed (1 bug in test expectation)
**Impact on plan:** Corrected test to match actual binary-faithful behavior. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- AI behavior modules ready for script integration in plan 06-08
- Target scoring and threat assessment available for AI attack decisions
- Shaman command queue ready for PopScript ATTACK/SPELL dispatching
- Difficulty scaling ready for computer tribe mana/training adjustments

---
*Phase: 05-ai-and-campaign*
*Completed: 2026-03-24*

---
phase: 05-ai-and-campaign
plan: 04
subsystem: ai-scripting
tags: [lua, popscript, ai-bridge, game-state]

requires:
  - phase: 05-01
    provides: AiSystem with Lua VM, constants, tribe script state
provides:
  - PopScript function registry with EVERY macro and tick-counter tracking
  - AiGameBridge struct for Rust-Lua game state bridging
  - Action command pipeline (attack, build, train, spell, move, pray, cleanup)
  - Configuration setters (defence radius, base radius, attack variable, spell entry, etc.)
  - Game state read functions (GAME_TURN, population, mana)
affects: [05-05, 05-06, 05-07]

tech-stack:
  added: []
  patterns: [Rc<RefCell<AiGameBridge>> shared between Lua closures and Rust, EVERY tick-counter via Lua-side table with sequential IDs]

key-files:
  created: [src/engine/ai/popscript.rs]
  modified: [src/engine/ai/mod.rs]

key-decisions:
  - "EVERY uses sequential ID per (interval, call-order) instead of debug.getinfo since mlua sandbox disables debug library"
  - "AiGameBridge is a flat struct with per-tribe arrays, populated before tick and drained after"
  - "Pending commands use Vec-based queues (attacks, builds, trains, spells, moves, etc.)"
  - "Unimplemented functions panic per D-04 -- not log::warn or default return values"

patterns-established:
  - "Rc<RefCell<AiGameBridge>> pattern: shared mutable bridge between Lua closures and Rust game loop"
  - "_every_reset_ids() must be called before each script tick to reset sequential counter"

requirements-completed: [AI-02, AI-03]

duration: 6min
completed: 2026-03-24
---

# Phase 05 Plan 04: PopScript Function Registry Summary

**PopScript function registry with EVERY macro, AiGameBridge, 168 registered functions (game state reads, action commands, config setters, panic stubs)**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-24T02:14:53Z
- **Completed:** 2026-03-24T02:20:34Z
- **Tasks:** 2/2
- **Files modified:** 2

## Accomplishments

### Task 1: EVERY macro, AiGameBridge, and game state read functions
- Created `AiGameBridge` struct with per-tribe state arrays and pending command queues
- Created 9 command structs (AiAttackCommand, AiBuildCommand, AiSpellCommand, AiTrainCommand, AiMoveCommand, AiPrayCommand, AiCleanupCommand, AiConvertCommand, AiShamanMoveCommand) plus MarkerEntry
- Implemented EVERY(interval, body) macro with Lua-side tick counter tracking using sequential IDs
- Implemented game state read functions: GAME_TURN, MY_NUM_PEOPLE, BLUE/RED/YELLOW/GREEN_PEOPLE, MY_MANA
- Registered remaining ~159 functions as panic stubs per D-04
- **Commit:** d99f188

### Task 2: Action commands, configuration setters, and remaining stubs
- Replaced panic stubs with real implementations for 11 action commands: ATTACK, ATTACK_MARKER, BUILD_AT, TRAIN_PEOPLE_NOW, SPELL_AT_MARKER, CONVERT_AT_MARKER, SEND_PEOPLE_TO_MARKER, SEND_SHAMAN_DEFENDERS_HOME, PRAY_AT_HEAD, SET_MARKER_ENTRY, DELETE_SMOKE_STUFF
- Replaced panic stubs with real implementations for 8 configuration setters: SET_DEFENSE_RADIUS, SET_BASE_RADIUS, SET_ATTACK_VARIABLE, SET_SPELL_ENTRY, SET_REINCARNATION, SET_BUCKET_USAGE, MAX_BUILDING_TYPE
- Remaining unimplemented functions keep panic stubs per D-04
- **Commit:** a6aeeec

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] EVERY macro debug.getinfo unavailable in mlua sandbox**
- **Found during:** Task 1
- **Issue:** mlua's `Lua::new()` does not include the `debug` library, causing EVERY to fail with "attempt to index a nil value (global 'debug')"
- **Fix:** Replaced `debug.getinfo(2, "Sl")` source-line keying with sequential auto-incrementing ID per call site. Added `_every_reset_ids()` function to reset counter each tick.
- **Files modified:** src/engine/ai/popscript.rs
- **Commit:** d99f188

## Verification

- All 23 popscript tests pass
- All 68 AI module tests pass (0 failures)
- EVERY fires body exactly at interval boundaries (tick 4, 8 for interval 4)
- Different EVERY intervals coexist correctly in same script
- Game state reads return correct bridge values
- Action commands accumulate in bridge pending lists
- Configuration setters store values in bridge
- Unimplemented functions panic (not silent no-op)

## Known Stubs

None that block this plan's goals. The ~140 remaining panic-stub functions are intentional per D-04 and will be wired when underlying game systems are ready in plans 05-05 and beyond.

## Self-Check: PASSED

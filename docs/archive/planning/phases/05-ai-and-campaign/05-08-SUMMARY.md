---
phase: 05-ai-and-campaign
plan: 08
subsystem: integration
tags: [menu, campaign, ai, save, wiring, game-loop]

requires:
  - phase: 05-01
    provides: "Game state machine (Frontend/Loading/InGame/Outro)"
  - phase: 05-02
    provides: "Save/load with bincode serialization"
  - phase: 05-03
    provides: "MenuSystem with render data contract"
  - phase: 05-06
    provides: "CampaignState with victory/defeat progression"
  - phase: 05-07
    provides: "AiSystem with Lua VM and tick_update_ai"
provides:
  - "Full game loop: menu -> level select -> gameplay -> victory/defeat -> stats"
  - "AiSystem wired into tick loop with update_bridge pre-sync"
  - "QuickSave/QuickLoad accessible via F9/F10"
  - "Campaign progress checking after simulation ticks"
  - "Menu input isolation (no game command leakage in Frontend)"
affects: [phase-06, ui-input, audio]

tech-stack:
  added: [dirs]
  patterns: [menu-input-routing, ai-bridge-pre-sync, campaign-progress-check]

key-files:
  created: []
  modified:
    - src/render/app.rs
    - src/engine/command.rs
    - src/engine/ai/mod.rs
    - Cargo.toml

key-decisions:
  - "ToggleSimulation uses pause flag instead of Frontend state transition (avoids menu activation)"
  - "Escape in InGame transitions to Frontend menu instead of quitting"
  - "AiSystem update_bridge clears pending commands each tick for clean slate"
  - "dirs crate for platform-appropriate save directory"

patterns-established:
  - "Menu input routing: Frontend/Outro state intercepts arrow/enter/escape before translate_key"
  - "AI bridge sync: update_bridge called before simulation_tick, not inside it"

requirements-completed: [MENU-01, MENU-02, MENU-03, MENU-04, MENU-05, CAMP-01, CAMP-02, CAMP-03, SAVE-01, SAVE-02, SAVE-03]

duration: 7min
completed: 2026-03-24
---

# Phase 05 Plan 08: Full Game Loop Integration Summary

**All Phase 5 subsystems wired into App: menu navigation, campaign progression, AI tick integration, and quicksave/quickload via F9/F10**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-24T02:28:43Z
- **Completed:** 2026-03-24T02:36:02Z
- **Tasks:** 2 (auto) + 1 (human-verify, auto-approved)
- **Files modified:** 4

## Accomplishments

- MenuSystem and CampaignState wired into GameEngine with full menu navigation in Frontend/Outro states
- AiSystem plugged into TickSubsystems ai slot with update_bridge pre-sync each tick
- QuickSave/QuickLoad commands handled with bincode serialization to platform save directory
- Campaign progress check after each simulation tick, triggering StatsScreen on victory/defeat
- Input isolation: Frontend state intercepts arrow keys, Enter, Escape for menu; no game commands leak

## Task Commits

1. **Task 1: Wire MenuSystem and CampaignState into App** - `1208de2` (feat)
2. **Task 2: Wire AiSystem and save/load into App** - `8ded31b` (feat)
3. **Task 3: Verify full game flow end-to-end** - auto-approved (human-verify checkpoint)

## Files Created/Modified

- `src/render/app.rs` - Added menu_system, campaign_state, ai_system to GameEngine; menu command routing; campaign progress check; AI tick wiring
- `src/engine/command.rs` - Added F9/F10 key bindings for QuickSave/QuickLoad
- `src/engine/ai/mod.rs` - Added update_bridge() method and tests
- `Cargo.toml` - Added dirs dependency for platform save directory

## Decisions Made

- ToggleSimulation now toggles pause flag instead of switching game state to Frontend (avoids accidental menu activation)
- Escape key in InGame transitions to Frontend menu instead of quitting the application
- update_bridge clears all pending AI commands each tick to ensure clean command collection
- Added dirs crate (v6) for cross-platform save directory resolution

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed ToggleSimulation switching to Frontend state**
- **Found during:** Task 1 (menu wiring)
- **Issue:** ToggleSimulation toggled between Frontend and InGame, but Frontend now means "show menu" which was incorrect for pause
- **Fix:** Changed to toggle pause flag on GameFlags instead
- **Files modified:** src/render/app.rs
- **Verification:** Build succeeds, no state confusion between menu and pause

**2. [Rule 3 - Blocking] Added dirs crate dependency**
- **Found during:** Task 1 (save directory)
- **Issue:** Plan referenced dirs::data_dir() but dirs crate was not in Cargo.toml
- **Fix:** Added dirs = "6" to dependencies
- **Files modified:** Cargo.toml
- **Verification:** Build succeeds

**3. [Rule 2 - Missing] Added update_bridge method to AiSystem**
- **Found during:** Task 2 (AI wiring)
- **Issue:** Plan referenced update_bridge but method did not exist on AiSystem
- **Fix:** Implemented update_bridge with game state sync and pending command clearing
- **Files modified:** src/engine/ai/mod.rs
- **Verification:** Tests pass (update_bridge_populates_state, update_bridge_clears_pending_commands)

---

**Total deviations:** 3 auto-fixed (1 bug, 1 blocking, 1 missing critical)
**Impact on plan:** All fixes necessary for correct integration. No scope creep.

## Issues Encountered

None beyond the deviations listed above.

## Known Stubs

- `buildings` array in update_bridge is `[0u32; 4]` (TODO: wire building counts when available from building system)
- `ai_script_variables` and `ai_every_counters` in SaveFile are empty Vecs (TODO: extract from AiSystem state)

## Next Phase Readiness

- Phase 5 complete: all modules connected to main game loop
- Game flow operational: menu -> level select -> gameplay -> victory/defeat -> stats -> next level
- AI scripts will log warnings for missing .lua files but do not crash
- Ready for Phase 6 or additional subsystem implementations

---
*Phase: 05-ai-and-campaign*
*Completed: 2026-03-24*

---
phase: 05-ai-and-campaign
plan: 03
subsystem: menu-system
tags: [menu, state-machine, hud, campaign, frontend]
dependency_graph:
  requires: [hud-rendering]
  provides: [menu-state-machine, menu-rendering, campaign-select]
  affects: [app-loop, game-command]
tech_stack:
  patterns: [state-machine, render-data-contract]
key_files:
  created:
    - src/engine/menu/mod.rs
    - src/render/hud/menu.rs
  modified:
    - src/engine/mod.rs
    - src/engine/command.rs
    - src/render/hud/mod.rs
    - src/render/app.rs
decisions:
  - "MenuSystem uses MenuRenderData contract to decouple engine from HUD renderer"
  - "Options screen uses cursor positions 0/1/2 for speed-/speed+/back"
  - "Campaign levels 1-indexed to match original game level numbering"
metrics:
  duration: "3min"
  completed: "2026-03-24"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 6
---

# Phase 05 Plan 03: Menu System Summary

Text-based menu state machine with 5 screens, GameCommand extensions for menu/campaign/save, and HUD overlay rendering with cursor highlights.

## What Was Built

### Task 1: Menu state machine and GameCommand extensions
- `MenuSystem` struct with 5 screen types: MainMenu, CampaignSelect, LoadGame, Options, StatsScreen
- State transitions via `navigate_to()`, `back()`, `move_cursor()`, `select_item()`
- `MenuAction` return type for select operations (NavigateTo, StartLevel, LoadSave, Back, Quit)
- `MenuRenderData` + `MenuItem` as the HUD rendering contract
- `render_data()` builds display data for each screen (titles, item labels, cursor position)
- `MenuTarget` enum and 11 new `GameCommand` variants (MenuNavigate, MenuSelect, MenuBack, MenuUp, MenuDown, StartLevel, QuickSave, QuickLoad, SaveGame, LoadGame)
- 13 unit tests covering all state transitions, cursor wrapping, and game speed clamping

### Task 2: HUD menu rendering
- `build_menu_vertices()` generates full-screen dark overlay and highlight bar quads
- `push_quad()` helper for 6-vertex colored rectangles
- `menu_render_data: Option<MenuRenderData>` added to `HudState` for data flow
- `pub mod menu` added to HUD module

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 5c93fea | Menu state machine and GameCommand extensions |
| 2 | f578144 | HUD menu rendering with overlay and cursor highlight |

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None - all menu screens produce real data, no placeholder text flows to rendering.

## Verification

- `cargo test menu` -- 13 tests pass
- `cargo build` -- succeeds with no errors

## Self-Check: PASSED

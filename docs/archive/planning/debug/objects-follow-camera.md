---
status: awaiting_human_verify
trigger: "objects-follow-camera: When the camera moves, all objects follow the camera movement instead of staying fixed in world space"
created: 2026-03-24T04:45:00Z
updated: 2026-03-24T04:55:00Z
---

## Current Focus

hypothesis: Two bugs combine: (1) victory fires immediately because empty iterator .all() returns true, putting game in Outro state; (2) in-menu keyboard path applies camera commands but skips GPU-side rebuild_spawn_model(), so objects keep stale shift-baked positions
test: Fix both issues and verify objects stay fixed during camera movement
expecting: Objects remain at fixed world positions when camera orbits/pans/zooms
next_action: Apply fixes to victory check and keyboard handler

## Symptoms

expected: Camera orbits/pans around the terrain; objects (units, buildings, meshes) stay at fixed world positions
actual: All objects (units, buildings, 3D meshes) follow camera movement -- they appear stuck to the viewport
errors: No error messages -- visual rendering bug
reproduction: Run `cargo run --release -- --base data/original_game --level 1`, then move camera with WASD or Q/E keys
started: After Phase 5 commits. Last known good: commit dba330c

## Eliminated

- hypothesis: MVP matrix is wrong or double-applies shift
  evidence: MVP uses fixed focus at world center, shift is only baked into vertex positions via rebuild_spawn_model
  timestamp: 2026-03-24T04:48:00Z

- hypothesis: model_transform_buffer shared by objects is corrupted
  evidence: model_transform_buffer correctly applies LANDSCAPE_OFFSET + LANDSCAPE_SCALE to all object types consistently
  timestamp: 2026-03-24T04:49:00Z

- hypothesis: in_menu check intercepts camera keys at startup (game starts in Frontend)
  evidence: App init sets game_world.state = GameState::InGame at line 1085, overriding the default Frontend
  timestamp: 2026-03-24T04:50:00Z

## Evidence

- timestamp: 2026-03-24T04:46:00Z
  checked: Game startup logs
  found: "Victory! Showing stats screen" fires within first second of gameplay
  implication: Game immediately transitions to Outro state

- timestamp: 2026-03-24T04:48:00Z
  checked: TribeData::new() defaults
  found: All tribes start with active=false, population=0
  implication: check_singleplayer_victory sees empty set of active enemies, .all() returns true

- timestamp: 2026-03-24T04:50:00Z
  checked: In-menu keyboard handler (lines 3747-3780)
  found: Camera commands (PanScreen, RotateCamera) pass through via `_ => translate_key(key)` then `self.engine.apply_command(&cmd)` but NO rebuild_spawn_model() is called
  implication: Terrain shifts but objects keep stale positions = objects appear fixed to viewport

- timestamp: 2026-03-24T04:52:00Z
  checked: Non-menu keyboard handler (lines 3781-3823)
  found: Has full GPU-side effect handling: prev_shift check, rebuild_spawn_model for pan/rotate/tilt/reset
  implication: Confirms the in-menu path is missing essential GPU rebuilds

## Resolution

root_cause: Two interacting bugs: (1) check_singleplayer_victory uses .all() on an empty iterator of active enemy tribes (all start inactive), returning true and triggering immediate victory -> Outro state. (2) The in_menu keyboard handler path applies camera commands via apply_command() but skips rebuild_spawn_model() and other GPU-side effects, so objects keep stale shift-baked vertex positions while terrain shifts.
fix: (1) Add guard in check_singleplayer_victory: require at least one active enemy tribe before declaring victory. (2) Add GPU-side effect handling to the in_menu keyboard path for camera commands.
verification: All 694 tests pass. New test test_sp_no_victory_when_no_active_enemies passes. Runtime check confirms no "Victory! Showing stats screen" at startup. Game stays in InGame state.
files_changed: [src/engine/state/victory.rs, src/render/app.rs]

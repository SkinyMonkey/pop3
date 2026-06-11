# Phase 5: AI and Campaign - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

AI tribes driven by Lua scripts execute each game tick, 25-level campaign is playable from main menu to victory screen with stats, and game state can be saved/loaded. This phase wires AI behavior, campaign progression, menu system, and save/load into the existing tick loop and rendering infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Lua scripting engine
- **D-01:** Use rlua or mlua crate (Rust bindings to standard Lua 5.4 C library) as the runtime
- **D-02:** Implement the full Script3/Script4 API — all ~460 PopScript commands, ~300 INT_ internal variables, ~56 ATTR_ attributes, all flow control (IF/ELSE/ENDIF, EVERY loops, variables)
- **D-03:** Focus on the Script4_Popscript module (168 functions, 299 constants) plus Script4_Defines (3170 constants). Other Script4 modules (LevelEdit, Imgui, Network, Draw, etc.) are out of scope — they are remastered-edition features
- **D-04:** Unimplemented Lua function calls must panic/error, not silently no-op. This forces complete implementation of everything campaign scripts need before they run

### Script file pipeline
- **D-05:** Use Lua scripts only — not the original bytecode format. No bytecode VM needed
- **D-06:** Generate .lua files using existing community toolchain: KrampusPopEditor (Java bytecode decompiler: cpscr*.dat → .scr text) + popscript-upgrader (Python: .scr → .lua Script4 format)
- **D-07:** Ship generated .lua files in data/scripts/ directory (e.g., data/scripts/level_XX_tribe_Y.lua). Loaded from disk at level start. User-editable/moddable
- **D-08:** The 59 cpscr*.dat files in the original game data are the source — one per tribe per level

### AI behavior and tick integration
- **D-09:** AI personality traits and difficulty scaling use faithful constants from the original binary (target scoring weights, threat assessment, building priorities, COMPUTER_MANA_ADJUST, CP_TRAIN_MANA_* costs)
- **D-10:** Faithful tick model: AI_UpdateAllTribes runs once per tick, each tribe's Lua script runs to completion. EVERY blocks track their own tick counters. Matches original deterministic behavior
- **D-11:** Implement via the existing `AiTick` trait (`tick_update_ai(&mut self)`) which already has a NoOp stub in the tick loop

### Campaign progression
- **D-12:** Simplified v1: linear 25-level sequence with victory/defeat detection. All spells available from start — skip stone head discovery and spell/building unlock system (defer to v2)
- **D-13:** Victory/defeat transitions show a stats screen (kills, population, etc.) with 'Continue'/'Retry'/'Menu' buttons. Not instant transition

### Menu system
- **D-14:** Minimal functional menus: text-based campaign level select (level list), load game screen, basic options (game speed). No sprite-based graphics or animated buttons — focus effort on gameplay

### Save/Load system
- **D-15:** Own serialization format using serde (bincode or messagepack). NOT compatible with original 860KB binary save format. Simpler and extensible
- **D-16:** Quicksave support (single slot)

### Claude's Discretion
- Specific serde format choice (bincode vs messagepack vs other)
- Lua state management details (per-tribe Lua VM instances vs shared VM with per-tribe state)
- Menu rendering approach within existing HUD infrastructure
- Stats screen layout and specific statistics shown
- Error handling strategy for corrupt/missing script files

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### AI Scripting
- `docs/specs/ai_scripting.md` — Complete RE of AI bytecode VM, opcodes, decision trees, shaman commands, target scoring, threat assessment, difficulty scaling, personality traits
- `src/engine/state/traits.rs` — AiTick trait definition and NoOp stub (integration point)
- `src/engine/state/tick.rs` — Game tick loop where AI_UpdateAllTribes is called

### Script4 API (external)
- populous3.info Script3 documentation: `https://www.populous3.info/script3_doc/` — Full Script4 Lua API reference (460+ commands)
- GitHub: `TylerTheFox/popscript-upgrader` — Script2→Script4 converter with script4_system_spec.json (769 functions, 4067 constants across 31 modules)
- GitHub: `NightTerror1721/KrampusPopEditor` — Java bytecode decompiler (Decompiler.java) that reads cpscr*.dat → .scr text

### Victory/Campaign
- `src/engine/state/victory.rs` — Existing victory/defeat condition checking
- `src/engine/state/tribe.rs` — TribeData struct with per-tribe state (mana, population, reincarnation timer)
- `docs/specs/level_save_network.md` — Level file format, objectives loading, save system RE

### Game State
- `src/engine/state/mod.rs` — GameState structure
- `src/engine/state/flags.rs` — Game flags (has_won, has_lost, etc.)
- `src/engine/state/constants.rs` — Game constants

### Existing Infrastructure
- `src/engine/command.rs` — GameCommand enum (input boundary)
- `src/engine/frame.rs` — FrameState (output boundary)
- `src/render/app.rs` — App struct, GameEngine, rendering loop
- `src/render/hud/mod.rs` — HUD overlay rendering (menu will extend this)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AiTick` trait in `traits.rs`: ready-made integration point for AI into tick loop
- `TribeData` struct: already has mana, population, reincarnation_timer, active flag
- Victory condition checking in `victory.rs`: already works, campaign builds on top
- `GameCommand` enum: menu navigation commands can be added here
- HUD rendering infrastructure: menu screens can reuse the existing text rendering and sprite system
- Font/text rendering from Phase 3: 12/16/24pt text, string table with 0x526 strings

### Established Patterns
- Tick loop trait pattern: each subsystem implements a trait, NoOp for unimplemented
- GameCommand → GameEngine → FrameState pipeline: all input goes through commands, all output through frame state
- Binary parsers in `src/data/`: level loading already exists, save format would follow same patterns
- Test fixtures in `tests/`: captured from original binary via Frida

### Integration Points
- `tick_update_ai()` — where Lua scripts execute each tick
- `check_victory_conditions()` — already called from tick loop, campaign extends this
- Level loading in `src/data/` — campaign level sequencing hooks in here
- `App::handle_event()` in `app.rs` — menu input handling
- `FrameState` — menu rendering data flows through here

</code_context>

<specifics>
## Specific Ideas

- Community toolchain for script conversion: KrampusPopEditor decompiler (Java) → popscript-upgrader (Python) → .lua files
- Script4 system spec JSON at `TylerTheFox/popscript-upgrader/Script4_Language/System/script4_system_spec.json` contains the authoritative API definition (769 functions, 4067 constants)
- The PopScript module alone (168 functions + 299 constants + 3170 Defines constants) covers all AI scripting needs
- 59 cpscr*.dat bytecode files in game data need conversion (multiple tribes per level)

</specifics>

<deferred>
## Deferred Ideas

- Stone head discovery system (spell/building unlocks between levels) — v2
- Sprite-based animated menus matching original aesthetic — v2
- Original 860KB binary save format compatibility — v2
- Script4 modules beyond Popscript (LevelEdit, Imgui, Draw, Network, etc.) — remastered-edition features, not needed
- Bytecode VM as alternative to Lua — decided against, using Lua only

</deferred>

---

*Phase: 05-ai-and-campaign*
*Context gathered: 2026-03-23*

# Phase 5: AI and Campaign - Research

**Researched:** 2026-03-23
**Domain:** Lua scripting engine, AI behavior, campaign progression, menu system, save/load
**Confidence:** MEDIUM

## Summary

Phase 5 wires four major subsystems into the existing game: (1) a Lua-based AI scripting engine that executes per-tribe scripts every tick, (2) a 25-level campaign with victory/defeat detection and progression, (3) a minimal menu system for navigation, and (4) a serde-based save/load system. The largest technical challenge is implementing the ~168 PopScript API functions that the converted campaign scripts call -- each function bridges Lua into the Rust game state. The campaign scripts exist as 59 cpscr*.dat bytecode files in `data/original_game/levels/` and need external conversion to .lua via the KrampusPopEditor + popscript-upgrader toolchain before this phase begins.

The existing codebase has strong integration points: `AiTick` trait with NoOp stub in the tick loop, `GameState` enum with `Frontend`/`Loading`/`InGame`/`Outro` states, victory/defeat checking in `victory.rs`, and HUD rendering infrastructure. The `GameWorld` struct already owns tribe data, flags, and tick counters that the AI and campaign systems need to read and modify.

**Primary recommendation:** Use `mlua` 0.11 with the `lua54` feature for Lua 5.4 runtime. Use `bincode` 2.x for save format (fast, compact, no schema overhead). Implement PopScript functions incrementally -- start with the subset needed for level 1 scripts, then expand as campaign levels demand more functions.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Use rlua or mlua crate (Rust bindings to standard Lua 5.4 C library) as the runtime
- D-02: Implement the full Script3/Script4 API -- all ~460 PopScript commands, ~300 INT_ internal variables, ~56 ATTR_ attributes, all flow control (IF/ELSE/ENDIF, EVERY loops, variables)
- D-03: Focus on the Script4_Popscript module (168 functions, 299 constants) plus Script4_Defines (3170 constants). Other Script4 modules (LevelEdit, Imgui, Network, Draw, etc.) are out of scope
- D-04: Unimplemented Lua function calls must panic/error, not silently no-op
- D-05: Use Lua scripts only -- not the original bytecode format. No bytecode VM needed
- D-06: Generate .lua files using existing community toolchain: KrampusPopEditor (Java bytecode decompiler: cpscr*.dat -> .scr text) + popscript-upgrader (Python: .scr -> .lua Script4 format)
- D-07: Ship generated .lua files in data/scripts/ directory. Loaded from disk at level start. User-editable/moddable
- D-08: The 59 cpscr*.dat files in the original game data are the source -- one per tribe per level
- D-09: AI personality traits and difficulty scaling use faithful constants from the original binary
- D-10: Faithful tick model: AI_UpdateAllTribes runs once per tick, each tribe's Lua script runs to completion. EVERY blocks track their own tick counters
- D-11: Implement via the existing AiTick trait (tick_update_ai)
- D-12: Simplified v1: linear 25-level sequence with victory/defeat detection. All spells available from start -- skip stone head discovery
- D-13: Victory/defeat transitions show a stats screen with Continue/Retry/Menu buttons
- D-14: Minimal functional menus: text-based campaign level select, load game screen, basic options (game speed)
- D-15: Own serialization format using serde (bincode or messagepack). NOT compatible with original 860KB binary save format
- D-16: Quicksave support (single slot)

### Claude's Discretion
- Specific serde format choice (bincode vs messagepack vs other)
- Lua state management details (per-tribe Lua VM instances vs shared VM with per-tribe state)
- Menu rendering approach within existing HUD infrastructure
- Stats screen layout and specific statistics shown
- Error handling strategy for corrupt/missing script files

### Deferred Ideas (OUT OF SCOPE)
- Stone head discovery system (spell/building unlocks between levels) -- v2
- Sprite-based animated menus matching original aesthetic -- v2
- Original 860KB binary save format compatibility -- v2
- Script4 modules beyond Popscript (LevelEdit, Imgui, Draw, Network, etc.) -- remastered-edition features
- Bytecode VM as alternative to Lua -- decided against
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AI-01 | AI scripting engine -- Lua-based interpreter | mlua 0.11 crate with lua54 feature; PopScript API function list documented |
| AI-02 | Script flow control (IF/ELSE/ENDIF, EVERY/DO loops, subroutine calls) | Native Lua control flow handles IF/ELSE; EVERY macro needs Rust-side tick counter tracking |
| AI-03 | Script value types (literal, variable, internal attribute 1000-1237) | INT_* variables map to Rust closures reading GameState; ATTR_* map to per-tribe config |
| AI-04 | AI decision making (target selection scoring, threat assessment) | Documented scoring algorithm in ai_scripting.md; constants verified from binary |
| AI-05 | AI building placement (7-state placement machine) | AI_ValidateBuildingPlacements + AI_ExecuteBuildingPriorities in RE docs |
| AI-06 | Shaman command system (8 command types, 10 slots per tribe) | ShamanCommand struct documented at 0x52 bytes; dispatch table in ai_scripting.md |
| AI-07 | Difficulty scaling (separate mana/training costs for AI vs human) | COMPUTER_MANA_ADJUST + CP_TRAIN_MANA_* constants from binary; training cost bands documented |
| MENU-01 | Main menu with campaign/load/options navigation | GameState::Frontend already exists; text rendering from Phase 3 available |
| MENU-02 | Campaign level select screen | 25 levels from levl2001.dat-levl2025.dat; text list UI |
| MENU-03 | Load game screen | List save files from save directory |
| MENU-04 | Options/settings screen | Game speed adjustment already in GameCommand enum |
| MENU-05 | Menu button system with transitions | GameState state machine handles Frontend->Loading->InGame->Outro transitions |
| CAMP-01 | Victory conditions (all enemies eliminated) | Already implemented in victory.rs check_singleplayer_victory |
| CAMP-02 | Defeat conditions (player eliminated, reincarnation timer) | Already implemented in victory.rs with reincarnation timer |
| CAMP-03 | Campaign progression (25-level sequence, completion flags) | Need CampaignState struct tracking current_level and completed flags |
| CAMP-04 | Discovery system (stone head worship) | DEFERRED to v2 per D-12 -- all spells available from start |
| CAMP-05 | Level objectives loading (OBJECTIV.DAT) | File exists at levels/objectiv.dat; 16-byte records per level (verified via hexdump) |
| SAVE-01 | Save full game state to file | serde + bincode on GameState/GameWorld; must serialize all subsystem state |
| SAVE-02 | Load game state and restore all systems | Deserialize + rebuild derived state (meshes, spatial grid, etc.) |
| SAVE-03 | Quicksave support (slot 99) | Single quicksave slot via GameCommand::QuickSave/QuickLoad |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| mlua | 0.11.6 | Lua 5.4 runtime bindings | Actively maintained successor to rlua; rlua is archived and re-exports mlua since 0.20. Supports lua54 feature, has serde integration, 2x faster than rlua |
| serde | 1.x | Serialization framework | De facto standard for Rust serialization; derive macros for GameState structs |
| bincode | 2.0.0 | Binary serialization format | Fast, compact, no schema overhead. Ideal for game save files where human readability is irrelevant |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde_derive | 1.x | Derive macros for Serialize/Deserialize | All structs that go into save files |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| bincode | rmp-serde (MessagePack) 1.3 | MessagePack is self-describing (easier debugging), but larger files and slower. bincode is better for game saves where speed matters |
| mlua | rlua 0.20 | rlua 0.20 is literally a re-export of mlua; use mlua directly to avoid indirection |

**Recommendation (Claude's Discretion - serde format):** Use `bincode` 2.0. It produces the smallest files and fastest serialization. Game saves don't need human readability or cross-language interop.

**Recommendation (Claude's Discretion - Lua VM strategy):** Use one shared `mlua::Lua` instance with per-tribe state stored in Lua tables (one table per tribe). Creating 3 separate VM instances wastes memory and makes inter-tribe queries harder. The original game uses a single script engine with per-tribe script pointers.

**Installation:**
```bash
# Add to Cargo.toml [dependencies]:
mlua = { version = "0.11", features = ["lua54", "vendored"] }
serde = { version = "1", features = ["derive"] }
bincode = "2"
```

Note: `vendored` feature on mlua compiles Lua 5.4 from source, avoiding system Lua dependency.

## Architecture Patterns

### Recommended Project Structure
```
src/
  engine/
    ai/              # NEW: AI scripting subsystem
      mod.rs         # AiSystem struct implementing AiTick
      popscript.rs   # PopScript function registry (168 functions)
      constants.rs   # INT_*, ATTR_* constant tables (3469 constants)
      shaman_cmd.rs  # ShamanCommand struct and dispatch
      building_ai.rs # AI building placement state machine
      target.rs      # Target scoring and threat assessment
      difficulty.rs  # Difficulty scaling constants
    campaign/        # NEW: Campaign progression
      mod.rs         # CampaignState, level sequencing
      objectives.rs  # OBJECTIV.DAT parser
    save/            # NEW: Save/load system
      mod.rs         # SaveGame struct, save/load functions
    menu/            # NEW: Menu state machine
      mod.rs         # MenuState enum, menu logic
  render/
    hud/
      menu.rs        # NEW: Menu rendering (text-based)
      stats.rs       # NEW: Stats screen rendering
  data/
    scripts.rs       # NEW: Lua script file loading
```

### Pattern 1: AiSystem implementing AiTick

**What:** A struct that owns the Lua VM and per-tribe script state, implementing the `AiTick` trait to plug into the tick loop.

**When to use:** This is the core integration pattern -- the tick loop calls `tick_update_ai()` which iterates over all non-player tribes and runs their scripts.

```rust
pub struct AiSystem {
    lua: mlua::Lua,
    // Per-tribe script state (variables, EVERY counters, instruction pointer analog)
    tribe_states: [TribeScriptState; 4],
}

impl AiTick for AiSystem {
    fn tick_update_ai(&mut self) {
        // Mirrors AI_UpdateAllTribes @ 0x0041a7d0
        for tribe_idx in 0..4 {
            if !self.is_player(tribe_idx) && self.is_active(tribe_idx) {
                self.update_tribe(tribe_idx);
            }
        }
    }
}
```

### Pattern 2: PopScript Functions as Lua Closures

**What:** Each PopScript command (ATTACK, BUILD_AT, TRAIN_PEOPLE_NOW, etc.) is registered as a Lua function that captures a reference to the game state.

**When to use:** For all ~168 PopScript functions that scripts call.

```rust
// Register PopScript functions into Lua globals
fn register_popscript_functions(lua: &mlua::Lua, game_state: &Rc<RefCell<GameState>>) {
    let globals = lua.globals();

    // DO ATTACK(target_tribe, num_people, target_type, ...)
    let gs = game_state.clone();
    globals.set("ATTACK", lua.create_function(move |_, args: (i32, i32, i32)| {
        let mut state = gs.borrow_mut();
        state.ai_attack(args.0, args.1, args.2);
        Ok(())
    }).unwrap()).unwrap();

    // INT_MY_NUM_PEOPLE -> reads tribe population
    let gs = game_state.clone();
    globals.set("INT_MY_NUM_PEOPLE", lua.create_function(move |_, ()| {
        let state = gs.borrow();
        Ok(state.current_tribe_population())
    }).unwrap()).unwrap();
}
```

### Pattern 3: EVERY Macro as Lua Function with Tick Tracking

**What:** The PopScript EVERY construct (repeat block every N ticks) is implemented as a Lua function that checks a tick counter table.

**When to use:** EVERY is the primary loop construct in PopScript; nearly every script uses it.

```rust
// EVERY(interval, offset, function)
// Original: AI_ProcessLoopCommand @ 0x004c8700
// Executes the function body when (game_tick - offset) % interval == 0
```

In converted .lua scripts, EVERY becomes:
```lua
EVERY(64, function()
    -- This body runs every 64 ticks (~8 seconds at 8 ticks/sec)
    if MY_NUM_PEOPLE() > 10 then
        ATTACK(BLUE, 5, ATTACK_NORMAL)
    end
end)
```

### Pattern 4: GameState Extension for Menu/Campaign

**What:** Extend the existing `GameState` enum to handle menu navigation and campaign flow.

**When to use:** Menu screens, level transitions, victory/defeat screens.

```rust
// GameState already has: Frontend, Loading, InGame, Outro, Multiplayer
// Menu state lives inside App or a new MenuSystem that activates during Frontend
pub enum MenuScreen {
    MainMenu,
    CampaignSelect,
    LoadGame,
    Options,
    StatsScreen { victory: bool, stats: GameStats },
}
```

### Pattern 5: Save State via serde Derive

**What:** Add `#[derive(Serialize, Deserialize)]` to all game state structs that need saving.

**When to use:** GameWorld, TribeArray, TribeData, and all subsystem state that must survive save/load.

```rust
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    pub level_num: u32,
    pub game_world: GameWorldSave,  // Serializable subset of GameWorld
    pub campaign_state: CampaignState,
    pub ai_state: AiStateSave,      // Per-tribe script variables and counters
}
```

### Anti-Patterns to Avoid
- **Separate Lua VM per tribe:** Wastes memory; the original uses one engine with per-tribe state pointers. Use one VM with per-tribe Lua tables.
- **Lazy registration of PopScript functions:** Register ALL functions at VM creation. If a script calls an unregistered function, Lua itself will error, but D-04 says unimplemented functions must explicitly panic with a descriptive message.
- **Saving Lua VM state directly:** Lua VMs are not serializable. Save the per-tribe script variables and EVERY counters as Rust data, then restore them into the VM on load.
- **Polling for victory in campaign code:** Victory checking already runs in the tick loop. Campaign progression should react to flags, not re-check conditions.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Lua runtime | Custom bytecode interpreter | mlua + Lua 5.4 | Lua is battle-tested, community scripts already exist in Lua format |
| Binary serialization | Custom binary writer | serde + bincode | Save format correctness is critical; bincode handles endianness, versioning headers, etc. |
| Script file conversion | Custom cpscr*.dat parser | KrampusPopEditor + popscript-upgrader | External Java/Python tools already solve this; not worth reimplementing |
| PopScript flow control | Custom IF/ELSE/EVERY parser | Native Lua control flow | Lua already has if/else/end; EVERY is a simple tick-counter wrapper |

**Key insight:** The conversion from original bytecode to Lua was already solved by the community. Embrace that toolchain rather than reimplementing a bytecode VM.

## Common Pitfalls

### Pitfall 1: Lua Borrow Conflicts with Game State
**What goes wrong:** Lua closures need mutable access to game state, but multiple closures can't hold &mut simultaneously.
**Why it happens:** Rust borrow checker prevents multiple mutable borrows; Lua callbacks execute in an unpredictable order.
**How to avoid:** Use `Rc<RefCell<GameState>>` for the game state reference passed to Lua closures. This moves borrow checking to runtime. Alternatively, use mlua's `UserData` trait with `scope` for temporary borrows.
**Warning signs:** Compile errors about lifetimes when registering Lua functions.

### Pitfall 2: Script Execution Time Budget
**What goes wrong:** A buggy or complex script takes too long, causing frame drops.
**Why it happens:** Per D-10, scripts run to completion each tick. A tight loop in Lua blocks the game.
**How to avoid:** Set a Lua instruction count hook via `mlua::Lua::set_hook()` to abort scripts that exceed a reasonable instruction budget (e.g., 100,000 instructions per tick).
**Warning signs:** Sudden FPS drops when a specific level loads.

### Pitfall 3: Save/Load Missing Derived State
**What goes wrong:** Game loads but terrain mesh, spatial grid, or pathfinding cache is stale/empty.
**Why it happens:** Only "source of truth" data is saved; derived data (GPU meshes, cell grid, cached paths) must be rebuilt.
**How to avoid:** Explicitly list what is saved vs. what is derived. After load, call the same rebuild functions used during level loading.
**Warning signs:** Visual glitches, units walking through walls, empty spatial queries after loading.

### Pitfall 4: EVERY Counter Desync After Load
**What goes wrong:** After loading a save, EVERY blocks fire at wrong times.
**Why it happens:** EVERY counters are per-tribe state that must be saved. If only game_tick is saved but not the per-EVERY offset, timing is wrong.
**How to avoid:** Save the per-tribe EVERY counter table as part of AI state. Each EVERY instance has a last-fired-tick that must be preserved.
**Warning signs:** AI suddenly attacks immediately after load, or never attacks.

### Pitfall 5: Menu State vs Game State Confusion
**What goes wrong:** Menu input leaks into game commands, or game keeps ticking while in menu.
**Why it happens:** The existing event handling in app.rs doesn't distinguish Frontend from InGame input.
**How to avoid:** Check `GameWorld.state` before processing game commands. In Frontend state, only menu commands should be processed. The tick loop already guards on `GameState::InGame`.
**Warning signs:** Camera moves while navigating menus; simulation runs during level select.

### Pitfall 6: Conversion Toolchain Generates Incompatible Lua
**What goes wrong:** The popscript-upgrader output uses Script4 API functions that don't match the PopScript API names used in the original game scripts.
**Why it happens:** The remastered edition renamed/extended functions. The generated .lua files may reference Module_PopScript functions by different names than the original bytecode commands.
**How to avoid:** Inspect a few generated .lua files before writing the Rust function registry. Map function names from the generated output, not from assumptions about the original bytecode opcodes.
**Warning signs:** "attempt to call a nil value" errors when running scripts.

## Code Examples

### Creating an mlua Lua VM with PopScript globals
```rust
use mlua::prelude::*;

fn create_lua_vm() -> LuaResult<Lua> {
    let lua = Lua::new();

    // Register constants (INT_LIGHTNING = spell ID, etc.)
    let globals = lua.globals();
    globals.set("INT_LIGHTNING", 1)?;
    globals.set("INT_TORNADO", 2)?;
    globals.set("ATTACK_NORMAL", 0)?;
    globals.set("ATTACK_BUILDING", 1)?;
    // ... 3469 constants total

    Ok(lua)
}
```

### Loading and executing a tribe script
```rust
fn load_tribe_script(lua: &Lua, path: &str) -> LuaResult<()> {
    let script = std::fs::read_to_string(path)
        .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
    lua.load(&script).exec()?;
    Ok(())
}
```

### Save file with bincode
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    level_num: u32,
    game_tick: u32,
    // ... all game state fields
}

fn save_game(state: &SaveFile, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = bincode::serde::encode_to_vec(state, bincode::config::standard())?;
    std::fs::write(path, encoded)?;
    Ok(())
}

fn load_game(path: &str) -> Result<SaveFile, Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    let (decoded, _): (SaveFile, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())?;
    Ok(decoded)
}
```

### EVERY implementation
```rust
// Rust-side EVERY tracking
struct EveryState {
    interval: u32,
    last_fired: u32,
}

// In Lua, EVERY is a function that checks tick counter
fn register_every(lua: &Lua) -> LuaResult<()> {
    lua.load(r#"
        local _every_counters = {}
        function EVERY(interval, body)
            local key = debug.getinfo(2, "Sl").short_src .. ":" .. debug.getinfo(2, "l").currentline
            if not _every_counters[key] then
                _every_counters[key] = 0
            end
            local tick = GAME_TURN()
            if (tick - _every_counters[key]) >= interval then
                _every_counters[key] = tick
                body()
            end
        end
    "#).exec()?;
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| rlua for Lua bindings | mlua (rlua archived Sep 2025) | 2025 | Must use mlua directly; rlua 0.20 is just a re-export |
| bincode 1.x | bincode 2.0 | 2024 | New API: `encode_to_vec`/`decode_from_slice` instead of `serialize`/`deserialize` |
| Custom PopScript bytecode VM | Lua scripts via community conversion | 2019-2021 | Populous Reincarnated project made .lua scripts the standard modding format |

**Deprecated/outdated:**
- rlua: Archived September 2025. Use mlua instead.
- bincode 1.x: API changed significantly in 2.0. Use new `config::standard()` pattern.

## Open Questions

1. **Script conversion toolchain output format**
   - What we know: KrampusPopEditor decompiles cpscr*.dat to .scr text, popscript-upgrader converts .scr to .lua Script4 format
   - What's unclear: Exact function names in the generated .lua files. Do they match the populous3.info API docs, or use different naming? Need to run the toolchain on a sample file to verify
   - Recommendation: Run the conversion toolchain on cpscr010.dat before implementation begins. Inspect the output to determine exact function names and calling conventions

2. **Scope of "168 functions" actually used by campaign scripts**
   - What we know: The PopScript module has 168 functions total
   - What's unclear: How many of those 168 are actually called by the 59 campaign scripts? Some may be multiplayer-only or remastered-only
   - Recommendation: After converting scripts, grep for all function calls to build a "used functions" list. Implement those first; stub the rest with panic per D-04

3. **constant.dat file format**
   - What we know: File exists at `data/original_game/levels/constant.dat`, appears encrypted/encoded (not plaintext)
   - What's unclear: Whether the AI difficulty constants (COMPUTER_MANA_ADJUST, CP_TRAIN_MANA_*) come from this file or are hardcoded in the binary
   - Recommendation: Check if Ghidra RE docs have the constant.dat parsing function. If not, use hardcoded values from the binary analysis

4. **OBJECTIV.DAT record format**
   - What we know: File exists, appears to be 16-byte records (4 x u32 per record per level)
   - What's unclear: Exact field meanings. The LoadObjectivesData function at 0x0040dd70 hasn't been fully documented
   - Recommendation: For v1 simplified campaign, use the already-implemented victory conditions (all enemies eliminated). OBJECTIV.DAT parsing can be deferred if needed

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | All | Yes | (checked via cargo) | -- |
| Lua 5.4 C library | mlua | N/A (vendored) | Compiled from source | mlua `vendored` feature bundles it |
| Java runtime | KrampusPopEditor (script conversion) | Needs check | -- | Pre-convert scripts on a machine with Java |
| Python 3 | popscript-upgrader (script conversion) | Needs check | -- | Pre-convert scripts separately |
| Game data | Campaign levels, scripts | Yes | GOG install via Whisky | -- |

**Missing dependencies with no fallback:**
- None blocking. Script conversion is a one-time pre-processing step that can be done on any machine.

**Missing dependencies with fallback:**
- Java/Python for script conversion: If not available on dev machine, run conversion elsewhere and commit .lua files to `data/scripts/`.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in #[cfg(test)] |
| Config file | None (uses cargo test) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AI-01 | Lua VM creates, loads script, executes | unit | `cargo test ai::tests -x` | Wave 0 |
| AI-02 | IF/ELSE evaluates, EVERY fires at correct ticks | unit | `cargo test ai::tests::test_every -x` | Wave 0 |
| AI-03 | INT_* variables return correct game state values | unit | `cargo test ai::tests::test_int_variables -x` | Wave 0 |
| AI-04 | Target scoring matches original binary weights | unit | `cargo test ai::target::tests -x` | Wave 0 |
| AI-05 | AI building placement validates terrain | unit | `cargo test ai::building_ai::tests -x` | Wave 0 |
| AI-06 | Shaman command queue dispatches correctly | unit | `cargo test ai::shaman_cmd::tests -x` | Wave 0 |
| AI-07 | Difficulty scaling applies mana adjust | unit | `cargo test ai::difficulty::tests -x` | Wave 0 |
| MENU-01 | Main menu renders, transitions work | manual | -- | Wave 0 |
| MENU-02 | Level select shows 25 levels, selection works | manual | -- | Wave 0 |
| MENU-03 | Load screen lists saves, loads selected | manual | -- | Wave 0 |
| MENU-04 | Options screen changes game speed | manual | -- | Wave 0 |
| MENU-05 | Menu transitions don't leak input to game | unit | `cargo test menu::tests -x` | Wave 0 |
| CAMP-01 | Victory detected when all enemies eliminated | unit | `cargo test victory::tests -x` | Exists |
| CAMP-02 | Defeat detected with reincarnation timeout | unit | `cargo test victory::tests -x` | Exists |
| CAMP-03 | Campaign advances to next level on victory | unit | `cargo test campaign::tests -x` | Wave 0 |
| CAMP-04 | Discovery system | N/A | -- | DEFERRED |
| CAMP-05 | OBJECTIV.DAT loads correctly | unit | `cargo test campaign::objectives::tests -x` | Wave 0 |
| SAVE-01 | Save file written, contains all state | unit | `cargo test save::tests::test_save_roundtrip -x` | Wave 0 |
| SAVE-02 | Load restores identical game state | unit | `cargo test save::tests::test_load_restore -x` | Wave 0 |
| SAVE-03 | Quicksave uses slot 99, overwrites | unit | `cargo test save::tests::test_quicksave -x` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before /gsd:verify-work

### Wave 0 Gaps
- [ ] `src/engine/ai/mod.rs` -- AiSystem tests (AI-01 through AI-07)
- [ ] `src/engine/campaign/mod.rs` -- CampaignState tests (CAMP-03, CAMP-05)
- [ ] `src/engine/save/mod.rs` -- Save/load roundtrip tests (SAVE-01 through SAVE-03)
- [ ] `src/engine/menu/mod.rs` -- Menu state transition tests (MENU-05)
- [ ] mlua + serde + bincode in Cargo.toml -- dependency installation

## Project Constraints (from CLAUDE.md)

- **Test-first development:** Write or modify tests BEFORE implementing code. Red-green-refactor.
- **Read before editing:** Understand existing code before modifying it.
- **Minimal changes:** Only change what is necessary. No drive-by refactors.
- **Match the original binary:** Implementations must be faithful to the original game behavior. Reference `docs/specs/` for documented subsystems.
- **Rust 2021 edition, stable toolchain.**
- **No `unsafe` unless matching binary-level behavior.**
- **Tests live as `#[cfg(test)] mod tests` inside their module file.**
- **Binary format parsers go in `src/data/`; keep parsing separate from rendering.**
- **Use `Result` with descriptive error types; avoid `.unwrap()` in library code.**
- **Always use timeout when running `cargo run`** -- the renderer doesn't exit on its own.
- **Maintain I/O separation:** Input -> GameCommand -> GameEngine -> FrameState -> Renderer. GameEngine has NO GPU types.

## Sources

### Primary (HIGH confidence)
- `docs/specs/ai_scripting.md` -- Complete RE of AI system: execution flow, opcodes, target scoring, threat assessment, difficulty scaling, shaman commands, personality traits
- `src/engine/state/traits.rs` -- AiTick trait definition (integration point)
- `src/engine/state/tick.rs` -- Tick loop with AI_UpdateAllTribes call
- `src/engine/state/victory.rs` -- Victory/defeat detection (already implemented)
- `src/engine/state/tribe.rs` -- TribeData struct with per-tribe state
- `docs/specs/level_save_network.md` -- Save system RE: SaveGame_Create/Save/Load functions, file format (0x1398 byte header)
- crates.io: mlua 0.11.6, bincode 2.0.0, serde 1.x -- verified current versions via cargo search

### Secondary (MEDIUM confidence)
- [PopScript Wiki HTML Help File](https://ts.popre.net/archive/Downloads/Docs/PopScript_Wiki_HTML_Help_File.htm) -- Complete DO command list, INT_* variables, ATTR_* attributes
- [populous3.info Script3 docs](http://www.populous3.info/script3_doc/globals_a.html) -- Script4 API reference (incomplete Doxygen index)
- [mlua GitHub](https://github.com/mlua-rs/mlua) -- Confirmed lua54 feature, vendored build, serde support
- [rlua archived](https://github.com/mlua-rs/rlua/issues/294) -- Confirmed rlua merged into mlua, archived Sep 2025

### Tertiary (LOW confidence)
- Script conversion toolchain (KrampusPopEditor + popscript-upgrader) -- referenced in CONTEXT.md but not independently verified. Need to test the conversion on actual cpscr*.dat files before relying on it.
- constant.dat file format -- appears encrypted/encoded; parsing approach unclear

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- mlua and bincode are well-established, versions verified on crates.io
- Architecture: MEDIUM -- AI integration pattern is clear (AiTick trait exists), but PopScript function mapping depends on inspecting converted scripts which hasn't been done yet
- Pitfalls: MEDIUM -- borrow checker issues with Lua closures are well-known; save/load state completeness needs careful enumeration during implementation
- Campaign: HIGH -- victory/defeat already works; campaign progression is straightforward linear sequencing
- Menu: HIGH -- text-based menus are simple; HUD infrastructure exists

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable domain, crate versions unlikely to change significantly)

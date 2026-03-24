# Phase 5: AI and Campaign - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 05-ai-and-campaign
**Areas discussed:** Lua engine scope, Lua API surface, Script loading, AI tick integration, Campaign progression, Save/Load + Menu

---

## Lua Engine Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full Script3 API | All ~460 commands, ~300 internal vars, ~56 attributes, all flow control. Can run community Lua scripts unmodified. | ✓ |
| Campaign-complete | Only commands actually used by the 25 original campaign scripts. | |
| Core subset first | Start with ~50-80 most critical commands. | |

**User's choice:** Full Script3 API
**Notes:** User wants maximum completeness, following community Script3 documentation at populous3.info

---

## Lua Runtime

| Option | Description | Selected |
|--------|-------------|----------|
| rlua/mlua | Mature Rust bindings to standard Lua 5.4 (C library). Battle-tested. | ✓ |
| Pure Rust Lua | Something like piccolo — no C dependency, fully safe Rust. | |
| You decide | Claude picks. | |

**User's choice:** rlua/mlua

---

## Script Format

| Option | Description | Selected |
|--------|-------------|----------|
| Lua scripts only | Ship with pre-converted .lua files. Ignore original bytecode. | ✓ |
| Bytecode loader + Lua | Parse original .dat bytecode and translate to Lua at load time. | |
| Both formats | Support loading either .lua or .dat files. | |

**User's choice:** Lua scripts only
**Notes:** Existing community toolchain (KrampusPopEditor decompiler + popscript-upgrader) will generate .lua files from the original cpscr*.dat bytecode files.

---

## Script Conversion Approach

| Option | Description | Selected |
|--------|-------------|----------|
| A. Use existing tools | Run Krampus decompiler + popscript-upgrader. Low effort. | ✓ |
| B. Write Rust transpiler | Read bytecode directly, emit Lua. Medium effort. | |
| C. Write Rust bytecode VM | Execute original bytecode directly. No Lua conversion. | |

**User's choice:** Use existing tools (Option A)
**Notes:** Research confirmed: no pre-made Lua campaign scripts exist online, but KrampusPopEditor (Java) has Decompiler.java and popscript-upgrader (Python) handles .scr→.lua conversion.

---

## AI Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Faithful constants | Match original binary's target scoring, threat assessment, building priorities exactly. | ✓ |
| Script-driven | All behavior driven by Lua scripts — personality traits as script variables. | |
| You decide | Claude picks. | |

**User's choice:** Faithful constants

---

## Lua API Surface

| Option | Description | Selected |
|--------|-------------|----------|
| PopScript module only | Script4_Popscript (168 funcs) + Defines (3170 constants). What campaign scripts use. | ✓ |
| PopScript + game modules | ~500 funcs across Popscript + Players + Person + Map + Objects + Spells + Commands. | |
| All 769 functions | Stubs for every module. Maximum Script4 compatibility. | |

**User's choice:** PopScript module only

---

## Unimplemented Function Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Log warning + no-op | Print warning, return default. Scripts keep running. | |
| Panic/error | Halt script execution with error. Forces complete implementation. | ✓ |
| You decide | Claude picks. | |

**User's choice:** Panic/error

---

## Script File Location

| Option | Description | Selected |
|--------|-------------|----------|
| data/ directory | Ship as data/scripts/level_XX_tribe_Y.lua. Loaded from disk. Moddable. | ✓ |
| Embedded in binary | include_str!() or build script. No external files. | |
| You decide | Claude picks. | |

**User's choice:** data/ directory

---

## AI Tick Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Faithful tick model | AI_UpdateAllTribes once per tick, each script runs to completion. EVERY blocks track counters. | ✓ |
| Coroutine model | Lua coroutines yield after N instructions per tick. | |
| You decide | Claude picks. | |

**User's choice:** Faithful tick model

---

## Campaign Progression

| Option | Description | Selected |
|--------|-------------|----------|
| Faithful to original | Stone head discovery, spell/building unlocks, objectives from OBJECTIV.DAT. | |
| Simplified v1 | Linear 25-level sequence, all spells available. Skip discovery system. | ✓ |
| You decide | Claude picks. | |

**User's choice:** Simplified v1

---

## Victory/Defeat Transitions

| Option | Description | Selected |
|--------|-------------|----------|
| Instant transition | Victory → brief message → next level. | |
| Stats screen | Victory → kill/population stats → Continue button. Defeat → Retry/Menu. | ✓ |
| You decide | Claude picks. | |

**User's choice:** Stats screen

---

## Menu Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal functional | Text-based: level select, load game, basic options. No graphics. | ✓ |
| Styled menus | Sprite-based matching original aesthetic. | |
| You decide | Claude picks. | |

**User's choice:** Minimal functional

---

## Save Format

| Option | Description | Selected |
|--------|-------------|----------|
| Own format | Serde (bincode/messagepack). Simpler, extensible. Incompatible with original saves. | ✓ |
| Original format | Match 860KB binary format. Can load original saves. | |
| You decide | Claude picks. | |

**User's choice:** Own format

---

## Claude's Discretion

- Specific serde format choice
- Lua state management (per-tribe VM instances vs shared VM)
- Menu rendering approach within HUD infrastructure
- Stats screen layout
- Error handling for corrupt/missing scripts

## Deferred Ideas

- Stone head discovery system — v2
- Sprite-based animated menus — v2
- Original binary save format compatibility — v2
- Script4 modules beyond Popscript — remastered-edition features

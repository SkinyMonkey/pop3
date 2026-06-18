# AI Brain — autonomous computer player (v2)

> **Subsystem:** #9 (AI / scripting) — *the autonomous decision layer that drives the
> script VM and the shaman command queue.*
> **Companion to:** [`ai_script_vm.md`](./ai_script_vm.md) (the bytecode interpreter)
> and [`ai_script_commands.md`](./ai_script_commands.md) (the 168 command opcodes).
> This is the third and final piece of legacy `ai_scripting.md` (its **Appendix BO**).
> **Trust level:** decompilation + disassembly (level 2–3), addresses cited. No capture
> fixture yet — behaviour is decompilation-confirmed, numbers are not gameplay-validated.
> **⚠️ This spec primarily *refutes* legacy Appendix BO**, most of whose tables are
> fabricated (see [§7](#7-corrections-legacy-appendix-bo-is-largely-invented)).

## 1. Scope & the three AI layers

```
AI_UpdateAllTribes  0x0041a7d0   driver: per-tribe, every tick
  └─ AI_UpdateTribe 0x0041a8b0   THE BRAIN (this spec): cooldowns, runs the script,
       │                          periodic re-planning, dispatches the command queue
       ├─ AI_RunScript           0x004c5eb0  → the script VM  (ai_script_vm.md)
       │    └─ AI_ExecuteScriptCommand 0x004c6460 → commands  (ai_script_commands.md)
       └─ AI_ProcessShamanCommands 0x0041b6d0  → the command-queue scheduler (§4)
```

The **script** sets policy (markers, attack variables, spell entries) by writing tribe
fields; the **brain** turns that policy into concrete shaman commands and executes the
command queue. This spec covers the brain: `AI_UpdateTribe` and the functions it calls
that are *not* the VM.

## 2. Driver — `AI_UpdateAllTribes` (`0x0041a7d0`)

Each tick, for tribes `0 .. DAT_00957057-1`:

1. Decrement a per-tribe shaman timer at `tribe+0xc5e` (if tribe active, `tribe+0xc20`).
2. Skip the tribe if globally paused (`g_ui_state_flags & 0x20`, `DAT_0087e33c`).
3. If `AI_IsTribeReady(tribe) == 0` and `tribe+0x941 & 0x40 == 0`:
   * if `DAT_0088637f[tribeId * 0xc65] == 1` → `AI_UpdateTribe()` (full brain),
   * else → `AI_EvaluateSpellCasting(tribe)` only (a lighter path).

So `DAT_0088637f` per tribe selects **full brain vs spell-only**. Tribe struct base is
`0x885760`, stride `0xc65`; tribe index byte is `tribe+0xc22`.

## 3. The brain — `AI_UpdateTribe` (`0x0041a8b0`)

Decompilation unavailable (asm only); structure from `dump-asm`:

1. **Shaman build-timer** (`tribe+0x5bd`): decrement if nonzero.
2. **`AI_ValidateTargets(tribe)`** (`0x004b3f30`) gate — if it returns 0, skip the
   cooldown sweep. (NB: the real `AI_ValidateTargets` just returns
   `tribe+0x596 & 0x40000`, a single status bit — see §7.)
3. **Cooldown sweep:** loop `0x16` (22) times over `tribe+0x540` (stride 4), decrement
   word timers; when one reaches 0 with its companion byte nonzero, call
   `AI_UpdateUnitCooldowns` (`0x004b3f10`) to refresh it. (22 = the count of
   cooldown/Attribute slots, matching the script attribute span.)
4. **`AI_ValidateBuildingPlacements(tribe)`** (`0x0041b280`).
5. **Run the script:** compute the script context and call `AI_RunScript`:
   ```
   ctx = 0x945d4a + tribeId * 0x3108      // global per-tribe script-context array
   AI_RunScript(tribe, ctx)               // 0x004c5eb0
   ```
   > This **resolves the `[UNVERIFIED]` provenance** flagged in `ai_script_vm.md` §2.1:
   > the script context is **not** heap-allocated per tribe — it is a fixed global array
   > at **`0x945d4a`, stride `0x3108`** (just above the `+0x3104` IP field), indexed by
   > tribe id. Tribes 0–3 ⇒ `0x945d4a / 0x948e52 / 0x94bf5a / 0x94f062`.
6. **Periodic re-planning gate:** `((tribeId + DAT_00885720 + 0xd) & 0x3f) == 0` — i.e.
   **every 64 ticks, phase-offset by tribe id** (so the four tribes re-plan on different
   ticks). Inside the gate: threat/target evaluation using `AI_CalculateThreatDistance`
   (`0x0041b000`), `Math_IntegerSqrt`, nearest-target search, then
   `AI_FindBestAttackTarget` / shaman-safety / retreat decisions.
7. **Command execution (every tick):** `AI_ExecuteBuildingPriorities` (`0x0041b8d0`),
   `AI_ProcessShamanCommands` (`0x0041b6d0`), plus building-eject and queued-spell
   processing.

### 3.1 Decision cadences (verified — replace legacy guesses)

| Activity | Condition | Period |
|---|---|---|
| Re-planning / target eval | `(tribeId + tick + 0xd) & 0x3f == 0` | **every 64 ticks**, tribe-phased |
| Spell evaluation (`AI_EvaluateSpellCasting`) | `(tribeId*8 + tick + 0x11) & 0x7f == 0` | **every 128 ticks**, tribe-phased ×8 |
| Command-queue dispatch | every call | every tick |

Legacy "every ~32 frames" for spells is wrong; the real period is 128 ticks (§7).

## 4. Command queue & scheduler

The shaman/AI command queue is **10 slots of `0x52` bytes** at `tribe+0x74`. (Legacy got
the geometry right; the field layout below is the corrected version.)

### 4.1 Slot layout (`0x52` bytes, confirmed from `AI_SetShamanCommand`/`AI_ShamanRetreat`)

| Offset | Field |
|---|---|
| `+0x00` | flags — **bit 0 = active**, bit 1 = (cleared on set) |
| `+0x11` | (used as a sub-type/tag by some queries, e.g. `AI_FindBestAttackTarget` tests `== 0x11`) |
| `+0x68`/`+0x6c`/`+0x70` | command params (`param_4`/`param_5`/`param_6`) |
| `+0x78` | aux (`param_7`) |
| `+0x85` | **command type** (the scheduler's `switch` selector) |

`AI_SetShamanCommand(tribe, slot, type, p4, p5, p6, p7)` (`0x0041ba80`) writes
`slot = tribe + slotIndex*0x52`, sets `flags |= 1` (and clears bit 1), stores type at
`+0x85` and the params, then calls `AI_ShamanRetreat` to reset stale per-type state.
**This is the bridge from the script-command layer:** catalog handlers like `0x43e`,
`0x44e`, `0x43a`, `0x447`, `0x448` (see `ai_script_commands.md` §6.5/§6.10) call
`AI_SetShamanCommand` to enqueue work here.

### 4.2 Scheduler — `AI_ProcessShamanCommands` (`0x0041b6d0`)

Round-robin over the 10 slots using a per-tribe cursor at `tribe+0x5b5`:

```c
i = 0;
do {                                   // advance cursor to next ACTIVE slot
  if (slot[cursor].flags & 1) break;
  cursor = (cursor + 1) % 10;          // wraps at 10
  i++;
} while (i < 10);
if (i != 10)                           // found an active slot
  dispatch( slot[cursor].type @ +0x85 );
```

The `switch` on the command type has **24 distinct cases** (not the legacy's 8):
`0–9, 0xb, 0xd, 0xe, 0xf, 0x10, 0x11, 0x12, 0x13, 0x14, 0x18, 0x19, 0x1a, 0x1b, 0x1c`.
Confirmed mapping for the lower range:

| Type | Handler | | Type | Handler |
|---|---|---|---|---|
| `0` | `AI_Cmd_PrimaryAttack` | | `0xb` | `AI_ProcessBuildCommand` |
| `1` | `AI_Cmd_SecondaryAttack` | | `0xd` | `AI_ProcessSpyCommand` |
| `2` | `AI_Cmd_DefendPosition` | | `0xe` | `AI_ProcessPatrolCommand` |
| `3` | `AI_Cmd_SpellCasting` | | `0xf` | `AI_ProcessConvertCommand` |
| `4` | `AI_Cmd_ArmyMovement` | | `0x10` | `AI_ProcessGarrisonCommand` |
| `5` | `AI_Cmd_BuildingPlacement` | | `0x11` | `AI_ProcessShamanMoveCommand` |
| `6` | `AI_Cmd_ResourceGathering` | | `0x12`–`0x1c` | further commands (raid/garrison/evacuate/train families; per-handler derivation pending) |
| `7` | `AI_Cmd_Conversion` | | | |
| `8` | `AI_ProcessGatherAttack` | | | |
| `9` | `AI_ProcessAttackSpell` | | | |

Command types `0x19`/`0x1a`/`0x1c` are the ones enqueued by the script-command catalog's
shaman-issuer handlers (`ai_script_commands.md` §6.5/§6.10) — closing the script→brain loop.

> Legacy Appendix BO's "Shaman Command Types 0x00–0x07" table (PrimaryAttack…Conversion)
> matches types 0–7 **by luck of ordering**, but it stops at 8 and omits the other 16
> cases, and its claim that `AI_ProcessShamanCommands` "dispatches" by a `+0x11`-typed
> structure is wrong — the type byte is at `+0x85`, the cursor at `+0x5b5`.

### 4.3 Per-command state machines

The individual `AI_Cmd_*` / `AI_Process*Command` handlers are **per-slot state
machines**, not one-shot actions. Example — `AI_Cmd_SpellCasting` (`0x00445a40`) switches on a
state word at `slot+0x42` (`puVar1[0x21]`): states `0→4→5→6→7…` walk
"select target → assign person → cast → complete", flipping the state and assigning
persons (`AI_AssignPersonTarget`, `AI_SetPersonTarget`) each tick. `slot+0x36` holds a
sub-object pointer block the handler reads (`[0x19]` = target object id, `[0x1f]`/`[0x21]`
= flags/state). These deserve their own per-command derivation when the brain is built;
this spec documents the dispatch and queue model, not every state graph.

## 5. Threat / target / spell helpers (the real versions)

Short functions the brain calls — all decompiled, all **contradicting** the legacy tables:

| Function | Addr | What it ACTUALLY does |
|---|---|---|
| `AI_FindBestAttackTarget` | `0x004b9770` | **Not** a weighted scorer. Scans the 10 command slots for one that is active (`&1`), tagged `+0x11 == 0x11`, and matches a target id (`slot[-0xc] == target+0x24`); returns 1/0. A **slot-membership test**, not target selection. |
| `AI_AssessThreat` | `0x0041ba40` | **Not** weighted by unit type. Counts how many of the 10 command slots are active (`&1`). Returns 0–10. |
| `AI_CountEnemyUnits` | `0x004b51c0` | **Not** weighted. Walks the tribe person list (`tribe+0x881`); counts persons of model `0x0a`/`0x21` whose effect-record type (`0x920dc8 + id*10`) is `0x1e`. |
| `AI_CalculateThreatDistance` | `0x0041b000` | **Toroidal 1-D distance**: `d = |a-b|; if d > 0x80: d = 0x100 - d`. Wraps at 256, not "distance/100". Used per-axis, then `Math_IntegerSqrt` for range. |
| `AI_ValidateTargets` | `0x004b3f30` | Just returns `tribe+0x596 & 0x40000` — a single status bit, not "target validation". |
| `AI_SelectAttackSpell` | `0x0044f490` | Real spell choice: iterate candidate spells, compare **spell cost** `DAT_005a0eb4 + spell*0x3e` against **tribe mana** `tribe+0x94d`, pick the affordable/available one (`Building_CheckAvailability`). This is the concrete mechanism the legacy "if mana ≥ THRESHOLD" pseudocode invented. |

## 6. Building priorities — `AI_ExecuteBuildingPriorities` (`0x0041b8d0`)

**Not** a hardcoded "Drum Towers > Training > Housing" order (legacy). It is a
**function-pointer priority list** at `tribe+0x36e`: 12 entries × `0x14` bytes, each
`{ _, fnptr @ +0x4, counter @ +0x8, … }`. Each tick (when a free command slot exists and
`AI_CheckShamanBeforeBuild` passes) it:

1. Finds a free command slot (first of 10 with `flags & 1 == 0`).
2. Calls each entry's `fnptr(tribe, slot)` in order, incrementing that entry's counter,
   until one returns nonzero (that builder claimed the slot).
3. Runs a second pass over a sorted sub-priority array (`tribe+0x376`, stride `0x14`,
   count at `+0x37e`) to order pending build tasks.

The **priority order is therefore data**, populated elsewhere (likely from the script /
level config), not a constant in this function. `[UNVERIFIED]` where the 12 function
pointers and their order are initialised — trace writes to `tribe+0x36e`.

## 7. Corrections: legacy Appendix BO is largely invented

`ai_scripting.md` Appendix BO ("AI Decision Trees") is the least trustworthy part of the
legacy doc. Verdicts from decompilation:

| Legacy claim | Reality |
|---|---|
| `AI_FindBestAttackTarget` scoring table (shaman=1000, super_warrior=20, warrior=10, preacher=15, spy=5; "+500 exposed shaman"; "−distance/100") | **Fabricated.** The function is a 10-slot membership test returning 1/0 (§5). None of these constants exist in it. |
| `AI_AssessThreat` unit threat weights (shaman=50, swarrior=20, …) | **Fabricated.** It counts active command slots (0–10) with no type weighting (§5). |
| "Distance penalty `score -= distance/100`" | **Fabricated.** Distance is toroidal `|Δ|` wrapped at 256 (§5). |
| Spell casting "every ~32 frames", "if mana ≥ SPELL_THRESHOLD" pseudocode | Period is **128 ticks** (`& 0x7f`, §3.1); selection compares per-spell cost vs `tribe+0x94d` mana (§5). |
| Shaman command types = exactly 8 (0x00–0x07), struct typed at `+0x11`, state at `+0x4e` | **Wrong/incomplete.** 24 dispatch cases; type byte at `+0x85`, scheduler cursor at `+0x5b5`, slots `0x52` bytes at `+0x74` (§4). |
| Building priority order "Drum Towers > Training > Housing > Reincarnation" | **Fabricated as a constant.** It is a 12-entry function-pointer table at `tribe+0x36e`; order is data, not hardcoded (§6). |
| Difficulty/training-cost bands (`BAND_00_03 … 100%/120%/…`), personality traits at `+0x137` | **Unverified / not found in these functions.** No such band table appears in the brain path; treat as invented until a fixture or a cited global proves otherwise. `[UNVERIFIED]` |
| State-variable offsets `+0x5B4/+0x5B5/+0x5B7/+0x5BE/+0x596` | **Partly right:** `+0x5b5` is the command cursor (§4.2), `+0x596` a status-flag word (`ai_script_commands.md` §6.1), `+0x5b7` spell-select (catalog §6.1b). The *meanings* the legacy assigned are mostly guesses; the offsets exist. |

What survives from Appendix BO: the **command-queue geometry** (10 slots × `0x52` at
`+0x74`) and the rough **type 0–7 ordering**. Everything quantitative (scores, weights,
thresholds, bands) is invented and must not enter the engine.

## 8. Faithful-rewrite notes

1. Script context is a **global array** `0x945d4a` stride `0x3108` — not per-tribe heap
   (fixes `ai_script_vm.md` §2.1 `[UNVERIFIED]`).
2. Re-planning is **64-tick, tribe-phased**; spell eval **128-tick, tribe-phased ×8**.
3. The command queue is the brain↔script bridge: scripts enqueue via the
   `AI_SetShamanCommand`-calling catalog opcodes; the scheduler round-robins 10 slots.
4. Do **not** port any Appendix BO scoring/weight/threshold/band constant — re-derive
   each decision from the cited function instead.
5. `AI_AssessThreat`/`AI_FindBestAttackTarget` are mis-*named* by Ghidra (they are
   slot-count / slot-membership predicates); keep the addresses, distrust the names.

## 9. Evidence index

| Function | Address | Role |
|---|---|---|
| `AI_UpdateAllTribes` | `0x0041a7d0` | per-tribe driver; full-brain vs spell-only select |
| `AI_UpdateTribe` | `0x0041a8b0` | the brain (asm only) |
| `AI_ProcessShamanCommands` | `0x0041b6d0` | command-queue scheduler (24 types) |
| `AI_SetShamanCommand` | `0x0041ba80` | enqueue a command (script↔brain bridge) |
| `AI_ShamanRetreat` | `0x0041bf90` | per-type state reset on (re)assign |
| `AI_ExecuteBuildingPriorities` | `0x0041b8d0` | function-pointer priority table at `tribe+0x36e` |
| `AI_EvaluateSpellCasting` | `0x004b8a90` | 128-tick spell eval |
| `AI_SelectAttackSpell` | `0x0044f490` | cost (`0x5a0eb4 + s*0x3e`) vs mana (`+0x94d`) |
| `AI_Cmd_SpellCasting` | `0x00445a40` | per-slot spell state machine |
| `AI_FindBestAttackTarget` | `0x004b9770` | slot-membership predicate (mis-named) |
| `AI_AssessThreat` | `0x0041ba40` | active-slot count (mis-named) |
| `AI_CountEnemyUnits` | `0x004b51c0` | effect-type `0x1e` person count |
| `AI_CalculateThreatDistance` | `0x0041b000` | toroidal 1-D distance (wrap 0x100) |

Key data: tribe base `0x885760` stride `0xc65`; tribe index byte `+0xc22`; command
queue `+0x74` (10 × `0x52`); cursor `+0x5b5`; script-context array `0x945d4a` stride
`0x3108`; tick counter `DAT_00885720`; spell-cost table `0x5a0eb4` stride `0x3e`; mana
`tribe+0x94d`.

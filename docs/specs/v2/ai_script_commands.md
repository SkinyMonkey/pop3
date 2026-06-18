# AI Script Command Catalog (v2)

> **Subsystem:** #9 (AI / scripting) — *the command sub-opcode space*.
> **Companion to:** [`ai_script_vm.md`](./ai_script_vm.md). The VM spec defines the
> interpreter and statement grammar; this defines the **command vocabulary** the
> `0x3ee` COMMAND statement dispatches to.
> **Trust level — read this first:** the opcode → handler-address **routing** (§3)
> and per-opcode operand counts (§2.2) are ground truth — Ghidra reconstructs them
> from the binary's jump table (level 2–3). Handler **behaviour** is derived from
> decompilation (level 2–3) for the 59 formerly-unnamed `FUN_` handlers in §6, but
> **has no capture-fixture confirmation yet** — treat exact bit/field meanings there as
> strong hypotheses. The 44 non-flyby Ghidra-named handlers are checked against
> decompilation in §7 (all names confirmed accurate); their effects and operand counts
> are listed, but **arg meanings and fixtures are still outstanding.** Of the 49
> functions that carry an `AI_Script*` label, 42 are real and 7 are the mislabeled
> flyby block (§6.9). Net: every one of the 168 opcodes now has a verified route +
> operand count; behaviour is decompilation-level for the whole command set; no command
> is capture-confirmed yet.

## 1. Scope

`AI_ExecuteScriptCommand` (`0x004c6460`) is the dispatcher reached from the
interpreter's `0x3ee` COMMAND statement (see `ai_script_vm.md` §7). It `switch`es
over **168 command sub-opcodes** within `0x404`–`0x4c7` (the range is sparse — 28
values in it are unused gaps, e.g. `0x419`, `0x41c`–`0x422`, `0x424`–`0x427`) and
routes each to a handler.

**Operands are resolved by the dispatcher, not the handler** (corrected from the
first draft): for each operand the dispatcher reads the next bytecode token, calls
`AI_EvaluateScriptValue(tribe, ctx, ctx[0x3100] + index*8)`, advances IP, and passes
the **already-resolved integer** to the handler as a normal argument (see §2.2). So a
command's operand count equals the number of `AI_EvaluateScriptValue` calls in its
`case` block, and every handler signature is visible in the dispatcher. This is why
the handlers themselves never call the evaluator.

This catalog enumerates the routing (§3), the per-opcode operand counts (§2.2), and the
**derived behaviour** of every handler: §6 covers the 60 formerly-unnamed `FUN_`
handlers (59 derived, 1 undetermined export), §7 the 44 non-flyby Ghidra-named ones
(names confirmed) with operand counts and effects, each re-read from its
decompilation/disassembly. The 10 flyby handlers are corrected in §3.1/§6.9.

## 2. Dispatcher shape (verified)

`AI_ExecuteScriptCommand` reads one 16-bit command opcode at the current IP, advances
past it, and dispatches. Confirmed structure (decompilation at `0x004c6460`):

1. **`0x404`–`0x41b` except `0x40e` — inline flag set/clear (22 opcodes).** No
   handler call. The opcode selects a bit: `bit = opcode - 4` (so `0x404`→bit 0 …
   `0x41b`→bit 23). The **next** token selects the operation:
   `0x3fe` ⇒ set bit, `0x3ff` ⇒ clear bit, in the 32-bit flag word at
   `*(uint *)(tribe + 0x59a)`. IP then advances one more token. *(Note `0x419` is
   not a case — it is a gap in the switch.)*
   ```c
   command_offset = (char)opcode - 4;                              // 0x4c6460
   if (next == 0x3fe) tribe[0x59a] |=  (1 << (command_offset & 0x1f));
   if (next == 0x3ff) tribe[0x59a] &= ~(1 << (command_offset & 0x1f));
   ```
2. **`0x40e` and `0x423`–`0x4c7` — handler dispatch (146 opcodes).** Each routes to
   one of **114 distinct handler functions** (§3.1). Several opcodes share a handler
   (the handler switches internally on a resolved argument, not the opcode — e.g.
   `0x42a`/`0x42b` pass their evaluated operand to `FUN_004b7140`/`FUN_004b7160`).
3. Opcodes outside the listed cases fall through (no-op / return).

> `[UNVERIFIED]` the meaning of the `tribe+0x59a` flag bits (which behaviour each of
> the 24 bits gates). Mechanism is exact; semantics need a fixture. These are almost
> certainly the script "attribute enable/disable" flags the legacy doc gestured at.

### 2.2 Operand counts (verified)

Across all 168 command opcodes, the dispatcher resolves: **0 operands for 138
opcodes, 1 operand for 23, 2 operands for 7.** Counts per handler are in §6 for the
derived set; for the rest, count the `AI_EvaluateScriptValue` calls in
the opcode's `case` block at `0x004c6460`. (The inline flag-ops `0x404`–`0x41b` take
no descriptor operand but consume one extra literal token — the `0x3fe`/`0x3ff`
set/clear selector — which is not an evaluated operand.)

## 3. Opcode → handler routing map

**Legend** — handler-name trust:
* *(plain `AI_Script*` name)* — Ghidra label, **level 4 (hint, unverified)**.
* `FUN_xxxxxxxx` — **unnamed**, purpose unknown; address is exact.
* *(flyby)* — **corrected** here: the `decomp_export`/Ghidra DB name is wrong (it
  reads `AI_ScriptCmd_CastSpell*` / `SetPatrolTarget` / `SetRenderDirty`); the real
  function is cinematic-camera, per `docs/specs/legacy/re_meta.md` → Flyby and
  confirmed by this routing (`0x4b4`–`0x4be` → `0x004da7a0`/`0x004c9d90`+ block).

Routing (opcode → entry address) is **verified**. Names are per the legend.

| Opcode(s) | Handler address | Handler (trust per legend) |
|---|---|---|
| `0x404`–`0x40d`, `0x40f`–`0x418`, `0x41a`, `0x41b` | — (inline) | flag set/clear on `tribe+0x59a` (§2 item 1) |
| `0x40e` | `0x004cc430` | `AI_ScriptSetSpellTarget` |
| `0x423` | `0x004c9450` | `AI_ScriptCmd_AttackWithArmy` |
| `0x428` | `0x004cc340` | `AI_ScriptSetGuardPoint` |
| `0x429` | `0x004b7130` | `FUN_004b7130` |
| `0x42a` | `0x004b7140` | `FUN_004b7140` |
| `0x42b` | `0x004b7160` | `FUN_004b7160` |
| `0x42c` | `0x004cbe80` | `AI_ScriptCountPeopleInArea` |
| `0x42d`, `0x439`, `0x43a`, `0x447`, `0x448`, `0x493`, `0x495`, `0x4bf` | `0x004c8b50` | `AI_EvaluateScriptValue` (the value evaluator itself; see VM spec §5) |
| `0x431` | `0x004b6320` | `FUN_004b6320` |
| `0x432` | `0x004b6390` | `AI_ScriptCmd_EnsureShamanSafe` |
| `0x433` | `0x004b6340` | `AI_ScriptCmd_CheckShamanSafe` |
| `0x434` | `0x004cbf90` | `AI_ScriptGetTribeAttribute` |
| `0x435` | `0x004cc060` | `AI_ScriptCountTribeBuildings` |
| `0x43b`, `0x43c` | `0x004cc1c0` | `AI_ExecuteMultiParamCommand` |
| `0x43d` | `0x004cc120` | `AI_ScriptGetTerrainHeight` |
| `0x43e` | `0x004ec9c0` | `FUN_004ec9c0` |
| `0x441`, `0x442`, `0x443` | `0x004c9760` | `AI_ScriptCmd_SetAttackParams` |
| `0x444` | `0x004c9c40` | `AI_ScriptCmd_CastSpellAtTarget` |
| `0x445` | `0x004cb890` | `AI_ScriptCmd_SetPatrolPoints` |
| `0x446` | `0x004b5c20` | `FUN_004b5c20` |
| `0x449` | `0x004cbde0` | `AI_ScriptSetMarkerPosition` |
| `0x44a` | `0x004cbd00` | `AI_ScriptCheckBuildingState` |
| `0x44d` | `0x004cbbb0` | `AI_ScriptDefineArea` |
| `0x44e` | `0x004ece70` | `FUN_004ece70` |
| `0x44f` | `0x004b50f0` | `FUN_004b50f0` |
| `0x452` | `0x004cba40` | `AI_ScriptCmd_CountObjectsInArea` |
| `0x453` | `0x004cb960` | `AI_ScriptCmd_SetPersonState` |
| `0x454` | `0x004c9ae0` | `AI_ScriptCmd_ConfigureDefense` |
| `0x455` | `0x004b4ca0` | `FUN_004b4ca0` |
| `0x456` | `0x004b4c90` | `FUN_004b4c90` |
| `0x457` | `0x004c99e0` | `AI_ScriptCmd_SetViewTarget` |
| `0x458`, `0x459`, `0x45a`, `0x45b` | `0x004c9950` | `AI_ScriptCmd_SetTribeProperty` |
| `0x45c` | `0x004b4380` | `FUN_004b4380` |
| `0x45d`, `0x462` | `0x004cb5b0` | `AI_ScriptCmd_FindTargetObject` |
| `0x463` | `0x004b48d0` | `FUN_004b48d0` |
| `0x464` | `0x004b48e0` | `FUN_004b48e0` |
| `0x465` | `0x004b48f0` | `FUN_004b48f0` |
| `0x466` | `0x004b4900` | `FUN_004b4900` |
| `0x467` | `0x004b4910` | `FUN_004b4910` |
| `0x468` | `0x004b4920` | `FUN_004b4920` |
| `0x469` | `0x004b48b0` | `FUN_004b48b0` |
| `0x46a` | `0x004cb4d0` | `AI_ScriptCmd_CountPersonsByType` |
| `0x46b` | `0x004cb430` | `AI_ScriptCmd_CountPersonsInRange` |
| `0x46c` | `0x004b46f0` | `FUN_004b46f0` |
| `0x46d` | `0x004cb360` | `AI_ScriptCmd_DefendWithSpell` |
| `0x46e` | `0x004cb290` | `AI_ScriptCmd_AttackWithSpell` |
| `0x46f` | `0x004b46a0` | `FUN_004b46a0` |
| `0x470`, `0x471` | `0x004b4620` | `FUN_004b4620` |
| `0x472` | `0x004cb1e0` | `AI_ScriptCmd_TrainPersons` |
| `0x473`, `0x474`, `0x475` | `0x004b4600` | `FUN_004b4600` |
| `0x476` | `0x004cb010` | `AI_ScriptCmd_SetActiveSpell` |
| `0x477` | `0x004cc520` | `FUN_004cc520` |
| `0x478` | `0x004caef0` | `AI_ScriptCmd_EnableProperty` |
| `0x479`–`0x47f` | `0x00461820` | `FUN_00461820` |
| `0x480` | `0x004c9cd0` | `AI_ScriptCmd_LookAtShaman` |
| `0x481`, `0x482`, `0x483` | `0x004b4580` | `FUN_004b4580` |
| `0x484` | `0x004b44d0` | `FUN_004b44d0` |
| `0x485`–`0x488` | `0x004b4450` | `FUN_004b4450` |
| `0x489`, `0x48a` | `0x004cae90` | `AI_ScriptCmd_SetBucketUsage` |
| `0x48b` | `0x004cad70` | `AI_ScriptCmd_DisableProperty` |
| `0x48c`, `0x48d` | `0x004b40d0` | `FUN_004b40d0` |
| `0x48e` | `0x004cac90` | `AI_ScriptCmd_SetAttackTarget` |
| `0x48f` | `0x004ca7a0` | `AI_ScriptCmd_GetAttributeCount` |
| `0x490` | `0x004cabf0` | `AI_ScriptCmd_GetIdlePersonCount` |
| `0x491` | `0x0041cb90` | `FUN_0041cb90` |
| `0x492` | `0x0041cb60` | `FUN_0041cb60` |
| `0x494` | `0x004b3f40` | `FUN_004b3f40` |
| `0x496` | `0x004c8280` | `FUN_004c8280` |
| `0x497` | `0x004c82b0` | `FUN_004c82b0` |
| `0x498` | `0x004c82e0` | `FUN_004c82e0` |
| `0x499` | `0x004ca840` | `AI_ScriptCmd_SendPersonToPos` |
| `0x49a` | `0x004caa40` | `AI_ScriptCmd_SetPersonTarget` |
| `0x49b` | `0x0048dc80` | `FUN_0048dc80` |
| `0x49c` | `0x004c8310` | `FUN_004c8310` |
| `0x49d` | `0x004c8350` | `FUN_004c8350` |
| `0x49e` | `0x004c8390` | `FUN_004c8390` |
| `0x49f` | `0x004c83d0` | `FUN_004c83d0` |
| `0x4a0` | `0x004c8410` | `FUN_004c8410` |
| `0x4a1` | `0x004c8450` | `FUN_004c8450` |
| `0x4a2` | `0x004c8490` | `FUN_004c8490` |
| `0x4a3` | `0x004c84d0` | `FUN_004c84d0` |
| `0x4a4` | `0x0041cad0` | `FUN_0041cad0` |
| `0x4a5` | `0x004c8510` | `FUN_004c8510` |
| `0x4a6` | `0x004cab50` | `AI_ScriptCmd_KillPersonsInArea` |
| `0x4a7` | `0x004ca6d0` | `FUN_004ca6d0` |
| `0x4a8` | `0x004ca640` | `AI_ScriptCmd_GetAttribute` |
| `0x4a9`, `0x4aa` | `0x004ca540` | `AI_ScriptCmd_GetTribePersonCount` |
| `0x4ab` | `0x004ca440` | `AI_ScriptCmd_CastSpell` |
| `0x4ac`, `0x4ad`, `0x4ae` | `0x00437060` | `FUN_00437060` |
| `0x4af` | `0x004ca3a0` | `AI_ScriptCmd_SetGuardRegion` |
| `0x4b0`, `0x4b1` | `0x004198d0` | `FUN_004198d0` |
| `0x4b2` | `0x00419a70` | `FUN_00419a70` |
| `0x4b3` | `0x004b39f0` | `FUN_004b39f0` |
| `0x4b4`, `0x4b5` | `0x004da7a0` | `Flyby_CreateNew` *(flyby; DB name wrong)* |
| `0x4b6` | `0x004da7c0` | `Flyby_Start` *(flyby; was `AI_ScriptCmd_SetRenderDirty`)* |
| `0x4b7` | `0x004da8e0` | `Flyby_Stop` *(flyby)* |
| `0x4b8` | `0x004da940` | `Flyby_AllowInterrupt` *(flyby)* |
| `0x4b9` | `0x004c9d90` | `Flyby_SetEventPos` *(flyby; was `AI_ScriptCmd_CastSpellDirect`)* |
| `0x4ba` | `0x004c9e80` | `Flyby_SetEventAngle` *(flyby; was `AI_ScriptCmd_CastSpellArea`)* |
| `0x4bb` | `0x004c9f30` | `Flyby_SetEventZoom` *(flyby; was `AI_ScriptCmd_CastSpellDirectional`)* |
| `0x4bc` | `0x004c9ff0` | `Flyby_SetEventIntPoint` *(flyby; was `AI_ScriptCmd_CastSpellTargeted`)* |
| `0x4bd` | `0x004ca0e0` | `Flyby_SetEventTooltip` *(flyby; was `AI_ScriptCmd_CastSpellComplex`)* |
| `0x4be` | `0x004ca2b0` | `Flyby_SetEndTarget` *(flyby; was `AI_ScriptCmd_SetPatrolTarget`)* |
| `0x4c0` | `0x004ca210` | `AI_ScriptCmd_SetGuardArea` |
| `0x4c1` | `0x0048dd90` | `FUN_0048dd90` |
| `0x4c2`, `0x4c3`, `0x4c4` | `0x004c8550` | `FUN_004c8550` |
| `0x4c5` | `0x00427840` | `AI_KillTribeUnits` |
| `0x4c6`, `0x4c7` | `0x0041c300` | `FUN_0041c300` |

### 3.1 Coverage summary

| Bucket | Count | Trust |
|---|---|---|
| Total command opcodes (`0x404`–`0x4c7`) | 168 | routing verified |
| Inline flag set/clear (`0x404`–`0x41b` ∖ `0x40e`) | 22 | mechanism verified, bit semantics `[UNVERIFIED]` |
| Routed to a handler function | 146 | routing verified |
| **Distinct handler functions** | **114** | — |
| ↳ **flyby** (cinematic camera; DB names wrong) | 10 | corrected via `re_meta.md`; 7 carry `AI_ScriptCmd_CastSpell*`/`SetPatrolTarget`/`SetRenderDirty` names, 3 carry `FUN_` |
| ↳ **non-flyby, Ghidra-named** (`AI_*`) | 44 | behaviour + operand counts derived in §7 (42 `AI_Script*` + `AI_ExecuteMultiParamCommand` + `AI_KillTribeUnits`) |
| ↳ **non-flyby, formerly unnamed** (`FUN_`) | 60 | behaviour derived in §6 (59 derived across §6.1–§6.10; 1 undetermined export, §6.11) — includes the 7 two-operand late-comers in §6.10 |

(10 + 44 + 60 = 114. The 7 flyby handlers that carry `AI_ScriptCmd_*` *names* are
counted under "flyby", not under "Ghidra-named" — so the 49 `AI_Script*`-labelled
functions = 42 real + 7 flyby.)

## 4. Notable findings

* **The flyby block is not spell-casting.** Opcodes `0x4b4`–`0x4be` route into the
  `Flyby_*` cinematic-camera functions (`0x004da7a0`, `0x004c9d90`…), **not** the
  `AI_ScriptCmd_CastSpell*`/`SetPatrolTarget` names the Ghidra DB and
  `~/decomp_export` still carry. This routing independently confirms the
  `re_meta.md` flyby correction. A faithful implementation of these opcodes is the
  flyby camera (see `re_meta.md` → Flyby for event-queue/channel mechanics), not the
  spell system.
* **`0x42d…0x4bf` (8 opcodes) route to `AI_EvaluateScriptValue` itself.** These are
  not commands that *act* — they evaluate operand(s) and store the result into a
  variable slot; the dispatcher's `case` body (not the evaluator) does the store.
* **The formerly-unnamed handlers split into clear families** (§6): per-tribe
  AI-flag bit-ops on `tribe+0x596`, shaman-command issuers, person-list query
  counters, area-effect appliers, and global/UI toggles.
* **The whole command set is now characterized at decompilation level.** All 168
  opcodes have a verified route + operand count; §6 derives the 60 formerly-unnamed
  `FUN_` handlers and §7 verifies the 44 non-flyby Ghidra-named ones (names accurate;
  a further 7 `AI_Script*`-named functions are the mislabeled flyby block). The only
  remaining static gap is one empty export
  (`FUN_004b4380`, §6.10). What remains is **capture-fixture confirmation** and
  pinning the exact *meaning* of each command's arguments (e.g. which property/state a
  "SetProperty"/"SetPersonState" opcode selects).
* **Two reusable idioms** (§7.1): the `0x45e..0x461` immediate-tribe-selector operand
  (= BLUE/RED/YELLOW/GREEN constants), and the query-result store into variable slot
  `ctx[0x3000 + dst*4]` (the script's function-return mechanism — the VM has no value
  stack).

## 5. How to pin a command's exact argument meanings

Routing, operand counts, and effects are done (§6, §7). The remaining per-command work
is interpreting *what each argument means* and confirming against the running game. The
recipe:

1. `ghidra-bridge dump-asm <handler-address>` (from §3) — or read its `.c` in
   `~/decomp_export/functions/`.
2. For dispatcher-resolved operands, read the opcode's `case` body at `0x004c6460`; for
   self-resolving named handlers (§7), read the handler body — operands are evaluated
   in order via `AI_EvaluateScriptValue`.
3. Identify the game-state effect (which tribe/person/object field it writes) — §6/§7
   give the field/helper; map each operand to its role.
4. Confirm against a capture fixture before relying on it (outstanding for **all**
   commands — nothing here is capture-verified yet).

§6 applies steps 1–3 to the 60 formerly-unnamed `FUN_` handlers; §7 to the 44 non-flyby
named ones. Step 4 is the open frontier for the entire command set.

## 6. Derived handler behaviour (formerly-unnamed set)

Each entry below was re-read from the handler's decompilation/disassembly. **Trust:
decompilation-level (2–3) for the mechanics; no capture fixture yet, so treat exact
bit/field meanings as strong hypotheses.** `tribe` = `param_1` (the tribe struct base,
see VM spec §2.1); `ctx` = script context. Field offsets are on the tribe struct
unless noted.

### 6.1 Per-tribe AI-flag bit-ops on `tribe+0x596` (0 operands)

`tribe+0x596` is a 32-bit AI status-flag word (the legacy doc's "+0x596 status
flags"). These set or clear a single bit and return. Mechanism exact; the *meaning*
of each bit is `[UNVERIFIED]` (needs a fixture observing AI behaviour with the bit
toggled).

| Opcode | Handler | Operation |
|---|---|---|
| `0x455` | `FUN_004b4ca0` | `0x596 |= 0x400` |
| `0x456` | `FUN_004b4c90` | `0x596 &= ~0x400` |
| `0x463` | `FUN_004b48d0` | `0x596 |= 0x2000` |
| `0x464` | `FUN_004b48e0` | `0x596 &= ~0x2000` |
| `0x465` | `FUN_004b48f0` | `0x596 |= 0x4000` |
| `0x466` | `FUN_004b4900` | `0x596 &= ~0x4000` |
| `0x467` | `FUN_004b4910` | `0x596 |= 0x8000` |
| `0x468` | `FUN_004b4920` | `0x596 &= ~0x8000` |
| `0x48c` | `FUN_004b40d0` | `0x596 |= 0x20000` |
| `0x48d` | `FUN_004b40e0` | `0x596 &= ~0x20000` |
| `0x494` | `FUN_004b3f40` / `FUN_004b3f50` | set/clear `0x596` bit `0x40000` — selector token `0x3fe`/`0x3ff` (same pattern as the inline `0x404`–`0x41b` ops; **takes a following literal token, not an evaluated operand**) |
| `0x429` | `FUN_004b7130` | `0x596 &= ~0x10` (plain clear of bit 4; the matching set is folded into `0x42a` below) |

Note the `|`/`&~` pairs (`0x455`/`0x456`, `0x463`/`0x464`, …) are enable/disable
twins — the script language exposes both directions of each flag.

### 6.1b Spell-selection state (`0x42a`/`0x42b`, 1 operand)

| Opcode | Handler | Behaviour |
|---|---|---|
| `0x42a` | `FUN_004b7140` | `0x596 |= 0x10` (set bit 4 — the twin of `0x429`) **and** store arg into `tribe+0x5b7` (the AI "current spell selection" byte). |
| `0x42b` | `FUN_004b7160` | Store arg into `tribe+0x5b8`. |

These corroborate the legacy `+0x5B7` "current spell selection" offset.

### 6.2 Network-player tribe-flag setters on `DAT_0088609d + player*0xc65` (0 operands)

A parallel flag word indexed by `g_network_player_id` (stride `0xc65` = the tribe
stride, so this is the local player's tribe-flags at base `0x88609d`).

| Opcode | Handler | Operation |
|---|---|---|
| `0x491` | `FUN_0041cb90` | `flags |= 0x20000` |
| `0x492` | `FUN_0041cb60` | `flags |= 0x40000` |
| `0x4a4` | `FUN_0041cad0`/`FUN_0041cb00` | `flags |= 0x200000` |
| `0x469` | `FUN_004b48b0` | returns `DAT_0088609d & 0x200` (a **query**, not a setter) |

### 6.3 Selection / UI flag-ops on `DAT_007f91bb + sel*0x2d` (1 operand: bool)

A record array (stride `0x2d`) indexed by the current selection `DAT_007f97ef` (skip
if `-1`). The 1 operand is a boolean: non-zero sets the bit, zero clears it. These
are almost certainly per-selected-object render/UI flags.

| Opcode | Handler | Bit | (set when arg≠0) |
|---|---|---|---|
| `0x49c` | `FUN_004c8310` | `0x200` | |
| `0x49d` | `FUN_004c8350` | `0x400` | |
| `0x49e` | `FUN_004c8390` | `0x800` | |
| `0x49f` | `FUN_004c83d0` | `0x1000` | |
| `0x4a0` | `FUN_004c8410` | `0x100` | |
| `0x4a1` | `FUN_004c8450` | `0x4000` | |
| `0x4a2` | `FUN_004c8490` | `0x2000` | |
| `0x4a3` | `FUN_004c84d0` | `0x20000` | |
| `0x4a5` | `FUN_004c8510` | `0x100000` | |

(`0x49c`/`0x49d`/`0x4a5` shown to set-or-clear; the others set the bit when arg≠0.)

### 6.4 Sound channel + selection (1 operand)

| Opcode | Handler | Behaviour |
|---|---|---|
| `0x496`–`0x498` | `FUN_004c8280`/`82b0`/`82e0` | `Sound_AllocateChannel(1)` then, if a selection is active (`DAT_007f97ef != -1`), call `FUN_0048db90(sel, arg)` — three near-identical sound-cue commands. |
| `0x49b` | `FUN_0048dc80` | If selection record's capability bit `0x80` is set, store `arg` (u16) into `DAT_007f91aa + sel*0x2d`. |
| `0x4c4` | `FUN_004c8550` | Walk a linked list (`DAT_007f917c`, next at `+0x29`); for records whose `+0x1c == arg`, call `FUN_0048dd30(recordIndex)`. |

### 6.5 Shaman-command issuers (1 operand)

These find a free AI command slot and enqueue a shaman command (cf. named
`AI_SetShamanCommand`, command-slot model in the deferred `ai_brain.md`).

| Opcode | Handler | Behaviour |
|---|---|---|
| `0x43e` | `FUN_004ec9c0` | `slot = AI_FindFreePersonSlot(tribe); if slot != -1: AI_SetShamanCommand(tribe, slot, 0x19, arg, 0,0,0)` — command type `0x19`. |
| `0x44e` | `FUN_004ece70` | Same shape, command type `0x1c`; guarded by a slot-count check (`0x41c030`) and a readiness check (`0x4b4400`); then copies 3 personality bytes from `0x948d8d`/`8e`/`8f` into the command record (`+0x7e`…). |

### 6.6 Person-list query counters (0 operands, return a count)

Walk a person/object list and count matches; the dispatcher stores the return into a
variable slot. **Note these are called with the literal tribe base `0x885760` = tribe
slot 0 (the player tribe), not the executing AI tribe** — i.e. they query the player's
forces. (`0x46f` is the exception: it uses `ai_tribe`.)

| Opcode | Handler | Counts (on tribe 0 unless noted) |
|---|---|---|
| `0x483` | `FUN_004b4580` | Objects of class `0x09` belonging to tribe 0 (`obj+0x2f == 0`) with `+0x92 == 0`. |
| `0x484` | `FUN_004b44d0` | Tribe-0 persons with flag `+0xc & 0x800000`, standing on a cell whose object is class `0x02`, matching attribute-table bit (`0x5a0050 + subtype*0x4c & 0x20`), subtype ≠ 4. |
| `0x488` | `FUN_004b4450` | Tribe-0 persons of model `0x0a`/`0x21` whose effect record (`0x920dc8 + …`) has type `0x06`. |
| `0x46f` | `FUN_004b46a0` | Query on **`ai_tribe`**: returns whether the shaman command queue (`tribe+0x74`, 10 slots × `0x52`) has an active slot of type `0x14` — "is the shaman doing X?" |

> `[UNVERIFIED]` why these count tribe 0 specifically. Likely these opcodes are only
> emitted in the human player's own script context, or are objective/trigger
> predicates about the player. Confirm against which level scripts emit them.

### 6.7 Area / position effect appliers (1 operand: a packed marker)

The 1 operand indexes a marker/position table `DAT_00883d3d` (u16, x/y packed in the
two bytes, snapped to cell centres `(b & 0xfe)+1` — the same packing the flyby pos
channel uses). They apply a timed effect to persons near that position.

| Opcode | Handler | Behaviour |
|---|---|---|
| `0x431` | `FUN_004b6320` | On `ai_tribe`: sets `+0x596 |= 0x40` and stores marker into `+0x5a6` (arms a pending target). |
| `0x46c` | `FUN_004b46f0` | On **tribe 0**: if it has a shaman (`+0x89d`), `Person_ApplyTimedEffect(shaman, &pos)`. |
| `0x471` | `FUN_004b4620` | On **tribe 0**: for every object on its person list except class `0x07`, `Person_ApplyTimedEffect(obj, &pos)`. |
| `0x44f` | `FUN_004b50f0` | 0-operand, on `ai_tribe`: picks position from `+0x5a2`/`0x36a` (selected by flag `+0x5b4`) and applies the effect to list persons of model `0x0a`/`0x21` whose effect type is `0x1e`. |

### 6.8 Global / tribe-wide commands (0 operands)

| Opcode | Handler | Behaviour |
|---|---|---|
| `0x475` | `FUN_004b4600` | Clears bit `0x80` of `+0x7a` on every node of global list `DAT_00885fe1` (mass flag-clear). |
| `0x4ae` | `FUN_00437060` | For each object on player `arg`'s list (`DAT_00885fe5 + arg*0xc65`), call `FUN_004370a0(obj)`. The player index is the **opcode-resolved 1st arg** (a `char`). |
| `0x4b3` | `FUN_004b39f0` | On **tribe 0**: clears `+0x93d & ~0x10000`, then `FUN_004246b0(tribe0)`. |
| `0x47f` | `FUN_00461820` | From a marker (1 operand), walk a cell's object chain to find a class-`0x06`/subtype-`0x06` object and set its `+0x6d |= 2`. |
| `0x4c7` | `FUN_0041c300` | Query: returns `DAT_0087a880 != 0`. |

### 6.9 Trigger / state machine commands

| Opcode | Handler | Behaviour |
|---|---|---|
| `0x477` | `FUN_004cc520` | Per-`arg` (0/1/2): rewrites entries in a `0x71`-stride record table (`DAT_0067c604`, walked via `+0x6d`) — sets fields keyed on record-type `0x26`/`0x27`/`0x28` — then calls a finalizer (`FUN_0045ae30`/`10`). Looks like a **dialogue/objective branch selector**. |
| `0x4b0` | `FUN_004198d0` | **Start timer.** 1 operand; arms a global countdown `DAT_00952df4 = g_GameSpeed * arg; DAT_00952df8 = {3, …}` (operand scaled into ticks by game speed). |
| `0x4b1` | `FUN_004198d0(0,0)` | **Stop timer**, then `FUN_00419a60()` + `FUN_00419a90()` (reset/finalize). 0 operands. |
| `0x4b2` | `FUN_00419a70` | **Query timer:** 1 operand = destination var slot; stores `FUN_00419a70()` (true once `df8 & 1` and `df4 == 0`, i.e. timer elapsed) into `ctx[0x3000 + slot*4]`. |
| `0x4c1` | `FUN_0048dd90` | **Fisher–Yates shuffle** of `DAT_007f97eb` items using the AI LCG `seed = seed*0x24a1 + 0x24df; rotr 13` (`DAT_0088420a`) — randomized ordering/assignment. Confirms the AI RNG constant from the legacy doc. |
| `0x4c6` | `FUN_0041c300` | (query, see §6.8) |
| `0x4a7` | `FUN_004ca6d0` | Reads its own operands from IP (does **not** use the dispatcher's resolution): stores `DAT_0088609d >> 0x1f` (a sign bit) into a variable slot, and conditionally two more values (`DAT_00854198`/`99`) — a **state-capture into script variables**. |

### 6.10 Two-operand variants and late additions

Seven more `FUN_` handlers (found when the routing was re-extracted precisely — the
first parse had mis-attributed these opcodes to a neighbouring call). They are
two-operand variants of the families above, plus one flyby late-comer:

| Opcode | Handler | Behaviour |
|---|---|---|
| `0x42d` | `FUN_004b70f0` | On `ai_tribe`: `+0x596 |= 0x20` and store packed (a,b) target into `+0x5a4` — arms a coordinate target (sibling of `0x431`). |
| `0x43a` | `FUN_004ebab0` | Shaman-command issuer: `AI_SetShamanCommand(tribe, slot, type 0, 4, packed_xy, 1, 0)` — command type 0 with a target. |
| `0x446` | `FUN_004b5c20` | `tribe+0x5b2 = arg` (1 operand — a config byte). |
| `0x447` | `FUN_004ecbd0` | Shaman-command issuer (type 6): maps spell-arg `3/4/5/6 → 7/5/6/8`, resolves a spell handle (`FUN_004b5540`), issues if available. Gated on free slot + person count. |
| `0x448` | `FUN_004ecc70` | Shaman-command issuer (type `0x1a`), 2 operands; copies personality bytes (`tribe*0x30`-stride) into the command like `0x44e`. |
| `0x493` | `FUN_004b3fc0` | From a marker, walk a cell's object chain to a class-`0x06`/subtype-`0x06` object and `Object_DestroyByType` it (the *destroy* twin of `0x47f`'s flag-set). |
| `0x4bf` | `FUN_004dab20` | **Flyby** `Flyby_QueueEvent` (sorted insert into the event queue; was `AI_QueueSpellCommand` — per `re_meta.md`). Belongs to the cinematic-camera family, routed just past the `0x4b4`–`0x4be` block. |

### 6.11 Undetermined

| Opcode | Handler | Status |
|---|---|---|
| `0x45c` | `FUN_004b4380` | Export is empty in `~/decomp_export` (thunk or export gap). `[UNVERIFIED]` — dump-asm at `0x004b4380` directly to resolve. |

## 7. Named handlers — verification & operand counts

**49 handlers carry an `AI_Script*` Ghidra label.** Each was checked against its
decompilation. **Verdict: 42 names are accurate** (behaviour matches the label — unlike
the flyby block, these were correctly identified); **7 are the flyby functions** whose
labels are wrong (corrected in §3 / §6.9, excluded from the table below). Two further
handlers carry non-`AI_Script*` `AI_*` names — `AI_ExecuteMultiParamCommand` (`0x43b`/
`0x43c`) and `AI_KillTribeUnits` (`0x4c5`) — also confirmed accurate; with the 42 that
makes **44 non-flyby named handlers** (the §3.1 figure). Names here are promoted from
level-4 hint to **decompilation-confirmed** (still no capture fixture; arg *meanings* —
e.g. which property a "property" opcode sets — remain to be pinned per command).

### 7.1 Shared idioms (verified)

Two patterns recur across these handlers and are worth stating once:

* **Immediate-tribe-selector operand.** Many handlers read an operand token and, *if it
  lies in `0x45e..0x461`*, treat it as an **immediate tribe index** `token - 0x45e`
  (= 0..3); otherwise they evaluate it normally via `AI_EvaluateScriptValue`. So
  `0x45e`/`0x45f`/`0x460`/`0x461` are effectively the constants BLUE/RED/YELLOW/GREEN
  in operand position. (Seen in `GetAttribute`, `GetTribePersonCount`,
  `CountTribeBuildings`, `TrainPersons`, `SetBucketUsage`, `SetTribeProperty`, …)
* **Query result store.** "Getter/counter" handlers finish by writing their result into
  a **variable slot**: `ctx[0x3000 + dstDesc.value*4]`, where the final operand token is
  the destination descriptor index. This is the script's function-return mechanism (the
  VM has no value stack). Marked "→ var" below.

### 7.2 Named-handler table

`ops` = number of `AI_EvaluateScriptValue` resolutions the handler performs itself
(these handlers receive 0 dispatcher-resolved operands and parse their own). Field
offsets / helper addresses are the confirmed effect.

| Opcode | Handler | ops | Effect (decompilation-confirmed) |
|---|---|---|---|
| `0x40e` | `AI_ScriptSetSpellTarget` | 2 | Sets/clears spell-active flags on `tribe+0x59a`/`+0x596`; stores packed target (x,y) into `tribe+0x46e`. |
| `0x423` | `AI_ScriptCmd_AttackWithArmy` | 11 | Large attack-config command (11 operands → attack parameters). |
| `0x428` | `AI_ScriptSetGuardPoint` | 3 | Set a guard point (3 operands). |
| `0x42c` | `AI_ScriptCountPeopleInArea` | 3 | → var: count people in an area. |
| `0x434` | `AI_ScriptGetTribeAttribute` | 2 | → var: read `*(byte *)(tribe*0xc65 + 0x886386 + attr)` — per-tribe attribute byte. |
| `0x435` | `AI_ScriptCountTribeBuildings` | 2 | → var: `Building_GetCount(tribe, type)`. |
| `0x43d` | `AI_ScriptGetTerrainHeight` | 1 | → var: terrain height at a marker (via `g_terrain_cell_height_lookup`). |
| `0x444` | `AI_ScriptCmd_CastSpellAtTarget` | 4 | Cast spell at a target (`FUN_004ecad0`); consumes flag `+0x596 & 0x800`. |
| `0x445` | `AI_ScriptCmd_SetPatrolPoints` | 1 | Set patrol points (`AI_GetTargetPerson`). |
| `0x449` | `AI_ScriptSetMarkerPosition` | 3 | Set a marker position (`FUN_004ecd40`). |
| `0x44a` | `AI_ScriptCheckBuildingState` | 1 | → var: `Building_CheckAvailability`. |
| `0x44d` | `AI_ScriptDefineArea` | 5 | Define a named area; sub-token `0x450`/`0x451` select a mode. |
| `0x452` | `AI_ScriptCmd_CountObjectsInArea` | 5 | → var: count objects in an area. |
| `0x453` | `AI_ScriptCmd_SetPersonState` | 4 | Set person state (`FUN_004eba60`). |
| `0x454` | `AI_ScriptCmd_ConfigureDefense` | 6 | Configure defensive posture (6 operands). |
| `0x457` | `AI_ScriptCmd_SetViewTarget` | 3 | Set view/camera target. |
| `0x45b` | `AI_ScriptCmd_SetTribeProperty` | 2 | `FUN_00496c50(value, tribe, 1)` — set a tribe property. |
| `0x462` | `AI_ScriptCmd_FindTargetObject` | 3 | → var: find a target object (`Building_GetAnchorPosition`, `Tribe_GetShaman`). |
| `0x46a` | `AI_ScriptCmd_CountPersonsByType` | 3 | → var: count persons by type. |
| `0x46b` | `AI_ScriptCmd_CountPersonsInRange` | 2 | → var: count persons in range. |
| `0x46d` | `AI_ScriptCmd_DefendWithSpell` | 1 | Shaman defends with a spell (`Tribe_GetShaman`, `Input_ProcessKeyCommand`). |
| `0x46e` | `AI_ScriptCmd_AttackWithSpell` | 1 | Shaman attacks with a spell (drives input cmd `0x15`; `FUN_00424890`). |
| `0x472` | `AI_ScriptCmd_TrainPersons` | 2 | `FUN_00425320(tribe, count)` — train units. |
| `0x476` | `AI_ScriptCmd_SetActiveSpell` | 1 | Set the active spell (`FUN_0045b060`). |
| `0x478` | `AI_ScriptCmd_EnableProperty` | 2 | Enable a property (paired with `0x48b`). |
| `0x480` | `AI_ScriptCmd_LookAtShaman` | 1 | Point view at the shaman (`Input_ProcessKeyCommand`). |
| `0x48a` | `AI_ScriptCmd_SetBucketUsage` | 1 | `tribe+0x5ba = value` — bucket-usage byte. |
| `0x48b` | `AI_ScriptCmd_DisableProperty` | 2 | Disable a property. |
| `0x48e` | `AI_ScriptCmd_SetAttackTarget` | 4 | Set attack target (4 operands). |
| `0x48f` | `AI_ScriptCmd_GetAttributeCount` | 1 | → var: `FUN_004b4040(tribe)` attribute count. |
| `0x490` | `AI_ScriptCmd_GetIdlePersonCount` | 1 | → var: `FUN_004b4050(tribe)` idle-person count. |
| `0x4a6` | `AI_ScriptCmd_KillPersonsInArea` | 3 | Kill persons in an area (3 operands). |
| `0x4a8` | `AI_ScriptCmd_GetAttribute` | 1 | → var: `*(short *)(0x886197 + tribe*0xc65)`. |
| `0x4aa` | `AI_ScriptCmd_GetTribePersonCount` | 3 | → var: `FUN_004b7170(tribe, a, b)`. |
| `0x4ab` | `AI_ScriptCmd_CastSpell` | 4 | Cast spell: packs (x,y), remaps spell index (0→1, 1→2), `FUN_0045cac0(spell, pos, arg)`. |
| `0x443` | `AI_ScriptCmd_SetAttackParams` | 7 | Set attack parameters (7 operands). |
| `0x499` | `AI_ScriptCmd_SendPersonToPos` | 4 | Send a person to a position (`Sound_AllocateChannel`). |
| `0x49a` | `AI_ScriptCmd_SetPersonTarget` | 3 | Set a person's target. |
| `0x4af` | `AI_ScriptCmd_SetGuardRegion` | 3 | Set a guard region. |
| `0x4c0` | `AI_ScriptCmd_SetGuardArea` | 3 | Set a guard area. |
| `0x432` | `AI_ScriptCmd_EnsureShamanSafe` | 1 | Issue shaman-safety command (`AI_SetShamanCommand`, `AI_CheckShamanSafety`). |
| `0x433` | `AI_ScriptCmd_CheckShamanSafe` | 0 | Check/issue shaman-safety command. |

> The `Cast*`/spell handlers were **explicitly confirmed** to cast spells (not flyby):
> `CastSpell` (`0x4ab`) and `CastSpellAtTarget` (`0x444`) call the cast helpers
> `0x45cac0`/`0x4ecad0`; `AttackWithSpell`/`DefendWithSpell` drive the shaman through
> `Input_ProcessKeyCommand`. This is the boundary that the **mislabeled** flyby block
> (`0x4b9`–`0x4be`, the `CastSpellDirect/Area/Directional/Targeted/Complex` *names*)
> sits just past — do not confuse the two (§6.9, §3 legend).

## 8. Evidence index

| Item | Address | Note |
|---|---|---|
| `AI_ExecuteScriptCommand` (dispatcher) | `0x004c6460` | the `switch` enumerated in §3 |
| Flag word (inline `0x404`–`0x41b`) | `tribe + 0x59a` | 32-bit, bit = opcode−4 |
| Flyby correction source | — | `docs/specs/legacy/re_meta.md` → "Flyby cinematic camera" |
| Interpreter / operand model | — | `docs/specs/v2/ai_script_vm.md` §5, §7 |

All handler addresses in §3 are taken from `~/decomp_export/functions/*.json`
(`address` field) and the dispatcher's reconstructed jump table.

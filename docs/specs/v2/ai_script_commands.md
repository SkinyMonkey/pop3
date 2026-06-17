# AI Script Command Catalog (v2)

> **Subsystem:** #9 (AI / scripting) — *the command sub-opcode space*.
> **Companion to:** [`ai_script_vm.md`](./ai_script_vm.md). The VM spec defines the
> interpreter and statement grammar; this defines the **command vocabulary** the
> `0x3ee` COMMAND statement dispatches to.
> **Trust level — read this first:** the opcode → handler-address **routing** in
> §3 is ground truth (Ghidra reconstructs it from the binary's jump table; level
> 2–3). The handler **names and per-command semantics are NOT** — they are Ghidra
> labels (level 4, hints) or unknown (`FUN_`). This document is a **routing map**,
> not a behavioural spec. Each command's actual behaviour is its own future
> re-derivation. Do not implement a command from its name here.

## 1. Scope

`AI_ExecuteScriptCommand` (`0x004c6460`) is the dispatcher reached from the
interpreter's `0x3ee` COMMAND statement (see `ai_script_vm.md` §7). It `switch`es
over **168 command sub-opcodes** within `0x404`–`0x4c7` (the range is sparse — 28
values in it are unused gaps, e.g. `0x419`, `0x41c`–`0x422`, `0x424`–`0x427`) and
routes each to a handler that parses its own operands from the bytecode stream.

This catalog enumerates that routing. It does **not** specify handler behaviour,
operand layouts, or argument counts (a few exceptions are noted where already known
from `re_meta.md`). Those belong to per-command or per-handler re-derivation, done
just-in-time before the AI brain is rewritten.

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
   one of **107 distinct handler functions** (§3). Several opcodes share a handler
   (the handler presumably switches internally on the opcode).
3. Opcodes outside the listed cases fall through (no-op / return).

> `[UNVERIFIED]` the meaning of the `tribe+0x59a` flag bits (which behaviour each of
> the 24 bits gates). Mechanism is exact; semantics need a fixture. These are almost
> certainly the script "attribute enable/disable" flags the legacy doc gestured at.

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
| `0x404`–`0x40d`, `0x40f`–`0x418`, `0x41a`, `0x41b` | — (inline) | flag set/clear on `tribe+0x59a` (§2.1) |
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
| Distinct handler functions | 107 | — |
| ↳ with `AI_Script*` Ghidra label (level-4 hint) | ~45 | name unverified |
| ↳ unnamed (`FUN_`) — purpose unknown | 53 | name absent |
| ↳ flyby (corrected from wrong DB name) | 10 | corrected via `re_meta.md` |

## 4. Notable findings

* **The flyby block is not spell-casting.** Opcodes `0x4b4`–`0x4be` route into the
  `Flyby_*` cinematic-camera functions (`0x004da7a0`, `0x004c9d90`…), **not** the
  `AI_ScriptCmd_CastSpell*`/`SetPatrolTarget` names the Ghidra DB and
  `~/decomp_export` still carry. This routing independently confirms the
  `re_meta.md` flyby correction. A faithful implementation of these opcodes is the
  flyby camera (see `re_meta.md` → Flyby for event-queue/channel mechanics), not the
  spell system.
* **`0x42d…0x4bf` (8 opcodes) route to `AI_EvaluateScriptValue` itself.** These are
  not commands that *act* — they evaluate an operand and (presumably) push/store the
  result. Their exact effect needs the dispatcher's surrounding code per opcode.
* **53 of 107 handlers are unnamed.** The command layer is far less reverse-engineered
  than the VM core. This catalog's value is as a *worklist*: it pins every command's
  entry address so each can be triaged with `ghidra-bridge decompile <addr>` when its
  behaviour is needed.

## 5. How to extend this into a behavioural spec

For any command opcode, the re-derivation recipe is:

1. `ghidra-bridge dump-asm <handler-address>` (from §3).
2. Read how it advances IP — that gives the **operand count and layout** (most
   handlers call `AI_EvaluateScriptValue(tribe, ctx, ctx[0x3100] + index*8)` per
   operand, mirroring the VM-spec operand model).
3. Identify the game-state effect (which tribe/person/object field it writes).
4. Confirm against a capture fixture before relying on it.

This catalog deliberately stops at step 1 (the routing). Steps 2–4 are per-command
work, scheduled with the AI brain rewrite.

## 6. Evidence index

| Item | Address | Note |
|---|---|---|
| `AI_ExecuteScriptCommand` (dispatcher) | `0x004c6460` | the `switch` enumerated in §3 |
| Flag word (inline `0x404`–`0x41b`) | `tribe + 0x59a` | 32-bit, bit = opcode−4 |
| Flyby correction source | — | `docs/specs/legacy/re_meta.md` → "Flyby cinematic camera" |
| Interpreter / operand model | — | `docs/specs/v2/ai_script_vm.md` §5, §7 |

All handler addresses in §3 are taken from `~/decomp_export/functions/*.json`
(`address` field) and the dispatcher's reconstructed jump table.

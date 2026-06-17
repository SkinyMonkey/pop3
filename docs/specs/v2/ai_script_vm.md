# AI Script VM (v2)

> **Subsystem:** #9 (AI / scripting) — *interpreter core only*.
> **Trust level:** 2–3 (disassembly + decompilation, `ghidra-bridge dump-asm` /
> `decompile`, cross-checked against `~/decomp_export`). Every claim cites a binary
> address. Items that could not be confirmed from static analysis are marked
> `[UNVERIFIED]` with the address to check against a capture fixture.
> **Supersedes for this scope:** `docs/specs/legacy/ai_scripting.md` Appendix P and
> the second "AI Scripting System" section. The legacy doc is left in place as an
> address map; see [§9 Corrections to the legacy spec](#9-corrections-to-the-legacy-spec).

## 1. Scope

This spec covers the **bytecode interpreter** that runs each computer tribe's AI
script — the eight mutually-recursive functions at `0x4c5eb0`–`0x4c8b50` plus the
value evaluator at `0x4c8b50`. It defines the script-context memory model, the
statement opcode space, operand resolution, and the operator/assignment semantics.

**Deliberately out of scope** (each gets its own v2 spec, just-in-time before its
rewrite):

| Deferred | What it is | Where it lives |
|---|---|---|
| Command catalog | The 168 command sub-opcodes `0x404`–`0x4c7` dispatched by `AI_ExecuteScriptCommand` (`0x4c6460`) | future `ai_script_commands.md` |
| Autonomous brain | Shaman command queue, threat/target scoring, building priorities (legacy App. BO) | future `ai_brain.md` |
| Tick loop | `Game_SimulationTick` and tick ordering (legacy App. Q — **not AI**) | subsystem #2/#15 spec |
| PopScript Lua API | The *legacy Rust reimplementation* (`src/engine/ai/popscript.rs`, legacy App. RP) — trust level 6, not binary-derived | n/a (rewrite target, not a spec source) |

The interpreter reaches the deferred command catalog through exactly one statement
opcode (`0x3ee` COMMAND); that boundary is defined in [§4](#4-statement-opcode-space)
and [§7](#7-the-command-boundary).

## 2. Execution model

```
AI_UpdateAllTribes  0x0041a7d0   for each active tribe 0..N
  └─ AI_UpdateTribe 0x0041a8b0   per-tribe update; calls the VM once
       └─ AI_RunScript 0x004c5eb0   runs the whole script to completion this tick
            ├─ AI_ProcessScriptBlock        0x004c6180   (recursive: nested blocks)
            ├─ AI_ExecuteScriptCommand      0x004c6460   COMMAND dispatch (deferred catalog)
            ├─ AI_EvaluateScriptValue       0x004c8b50   operand → integer
            ├─ AI_EvaluateComparison        0x004c8930   relational ops
            ├─ AI_EvaluateCondition         0x004c8860   boolean tree (IF-condition form)
            ├─ AI_EvaluateConditionExpression 0x004c8a30  boolean tree (alt. form)
            ├─ AI_ProcessLoopCommand        0x004c8700   = / += / -=  (MISNAMED, see §9)
            └─ AI_ProcessSubroutineCall     0x004c8590   *= / /=      (MISNAMED, see §9)
```

`AI_RunScript` runs the **entire script to completion every time it is called** —
there is no per-tick instruction budget or yield. Control flow is ordinary recursion
over nested blocks; the script terminates when the outer loop reads the
`SCRIPT-END` token (`0x3fb`).

### 2.1 Parameters

Both VM entry points take `(param_1, param_2)`. Ghidra names `param_1` **`person`**,
but the disassembly shows it is the **tribe/player struct base pointer**
(`0x885760 + tribeIndex * 0xc65`): every use is `*(char *)(person + 0xc22)`, the
**tribe-index byte** (`AI_RunScript` case `0x3ed`; `AI_EvaluateScriptValue`
`0x004c8bc6: MOVSX ECX,[ESI+0xc22]`). Treat `param_1` as `tribe_ptr` in the rewrite;
the "person" name is misleading.

`param_2` is the **script context** — a separate, larger allocation (offsets reach
`+0x3104`, far beyond the `0xc65` tribe stride, so it is **not** inside the tribe
struct, contrary to the legacy claim; see §9). Called from `AI_UpdateTribe`
`0x0041a93d`: `PUSH EDX` (context) / `PUSH EDI` (tribe_ptr) / `CALL 0x4c5eb0`.

> `[UNVERIFIED]` The allocation/source of the context pointer (`EDX` at
> `0x0041a93d`) was not traced to its origin in this pass — confirm whether it is a
> per-tribe field or a shared scratch buffer. Its *layout and usage* (§3) are fully
> confirmed.

## 3. Script-context memory map

All offsets are relative to `param_2` (the context pointer) and are taken directly
from `AI_RunScript` (`0x4c5eb0`) and `AI_EvaluateScriptValue` (`0x4c8b50`).

| Offset | Type | Meaning |
|---|---|---|
| `+0x0002` | `int16[]` | **Bytecode**: a stream of 16-bit tokens (opcodes + operand indices). IP is initialised here. |
| `+0x2000` | `Operand[]` | **Operand-descriptor table**: 8-byte entries `{ i32 type; i32 value; }`. Bytecode operands are *indices* into this table (see §5). |
| `+0x3000` | `i32[]` | **Variable slots**: script-local integer variables, 4 bytes each, addressed `ctx[0x3000 + slot*4]`. |
| `+0x3100` | `ptr` | **Operand-descriptor base**, set once at entry to `ctx + 0x2000`. Used as `desc_base + index*8`. |
| `+0x3104` | `ptr` | **Instruction pointer (IP)**, set at entry to `ctx + 0x2`, advanced token-by-token. |

`AI_RunScript` prologue (`0x4c5eb0`), decompiled:

```c
*(int *)(ctx + 0x3104) = ctx;                 // transient, immediately overwritten
*(int *)(ctx + 0x3100) = ctx + 0x2000;         // descriptor base
*(short **)(ctx + 0x3104) = (short *)(ctx + 2); // IP := bytecode start
```

## 4. Statement opcode space

Statements use the **`0x3E8` (1000) range**. This corrects the legacy doc's
fabricated "base `0x404`" table (§9). Verified from the `switch` structure of
`AI_RunScript` (`0x4c5eb0`) and `AI_ProcessScriptBlock` (`0x4c6180`):

| Token | Dec | Mnemonic | Operands (16-bit tokens that follow) | Semantics |
|---|---|---|---|---|
| `0x3e8` | 1000 | **IF** | next token = a comparison/condition opcode | Evaluate condition; run the following block if true, else skip to matching `0x3ec`; optional `ELSE` block (`0x3e9`). |
| `0x3e9` | 1001 | **ELSE** | — | Marks the else-block; consumed during IF handling. |
| `0x3eb` | 1003 | **BLOCK-BEGIN** | — | Opens a block; used for `{}`-style nesting and depth counting when skipping. |
| `0x3ec` | 1004 | **BLOCK-END** | — | Closes a block; inner dispatch loop terminates here. |
| `0x3ed` | 1005 | **GUARDED-BLOCK** | `descA_index`, `[descB_index]` | Run the following block iff `descA.value & (tribeId + B + tickCounter) == 0` (see §6.3). |
| `0x3ee` | 1006 | **COMMAND** | (consumed by dispatcher) | Calls `AI_ExecuteScriptCommand` — entry to the deferred command catalog (§7). |
| `0x3ef` | 1007 | **SET** (`=`) | `dstDesc_index`, `srcDesc_index` | `dst = value` (`AI_ProcessLoopCommand`; §6.4). |
| `0x3f0` | 1008 | **ADD** (`+=`) | `dstDesc_index`, `srcDesc_index` | `dst += value`. |
| `0x3f1` | 1009 | **SUB** (`-=`) | `dstDesc_index`, `srcDesc_index` | `dst -= value`. |
| `0x3f4` | 1012 | **CMP-GT** (`>`) | `aDesc_index`, `bDesc_index` | `a > b` (§6.1). |
| `0x3f5` | 1013 | **CMP-LT** (`<`) | `aDesc_index`, `bDesc_index` | `a < b`. |
| `0x3f6` | 1014 | **CMP-EQ** (`==`) | `aDesc_index`, `bDesc_index` | `a == b`. |
| `0x3f7` | 1015 | **CMP-NE** (`!=`) | `aDesc_index`, `bDesc_index` | `a != b`. |
| `0x3f8` | 1016 | **CMP-GE** (`>=`) | `aDesc_index`, `bDesc_index` | `a >= b`. |
| `0x3f9` | 1017 | **CMP-LE** (`<=`) | `aDesc_index`, `bDesc_index` | `a <= b`. |
| `0x3fb` | 1019 | **SCRIPT-END** | — | Outer-loop terminator; ends `AI_RunScript`. |
| `0x3fc` | 1020 | **AND** | two nested conditions | Logical AND of two conditions (§6.2). |
| `0x3fd` | 1021 | **OR** | two nested conditions | Logical OR of two conditions. |
| `0x401` | 1025 | **MUL** (`*=`) | `dstDesc_index`, `srcDesc_index` | `dst *= value` (`AI_ProcessSubroutineCall`; §6.4). |
| `0x402` | 1026 | **DIV** (`/=`) | `dstDesc_index`, `srcDesc_index` | `dst /= value`, **div-by-zero ⇒ 0**. |

### 4.1 The `0` → `0x3ec` self-patching sentinel

In both `AI_RunScript` and `AI_ProcessScriptBlock`, a token value of `0` is
**rewritten in place to `0x3ec`** (BLOCK-END):

```c
else if (uVar2 == 0) { *puVar4 = 0x3ec; }   // 0x4c5eb0 / 0x4c6180
```

This lazily caps an unterminated block the first time it is hit. The bytecode buffer
is therefore **mutated during execution** — a faithful rewrite must treat the script
context as mutable, not as a read-only program image.

> `[UNVERIFIED]` intent: this looks like defensive termination of malformed/zeroed
> script tails. Confirm against a fixture that exercises a script whose block is not
> explicitly closed (check the buffer byte at the patched offset before/after).

## 5. Operand resolution

Bytecode operands are **not immediates**. Each operand token is a 16-bit **index**
into the operand-descriptor table at `ctx+0x2000`. Callers compute the descriptor
address as `desc_base + index*8` (`desc_base = ctx[0x3100]`) and pass it to
`AI_EvaluateScriptValue`. Confirmed identically in `AI_EvaluateComparison`
(`0x4c8930`), `AI_ProcessLoopCommand` (`0x4c8700`), and `AI_ProcessSubroutineCall`
(`0x4c8590`), e.g.:

```c
iVar6 = AI_EvaluateScriptValue(tribe, ctx, ctx[0x3100] + (uint)opIndex * 8);
```

### 5.1 `AI_EvaluateScriptValue` (`0x4c8b50`) — operand → integer

The descriptor's first dword selects one of three value types
(`0x4c8b5b: MOV ECX,[EDX]`):

| `type` | Source | Decompiled / asm |
|---|---|---|
| **0** | **Immediate** — return `descriptor.value` (`[desc+4]`) | `0x4c8b71: MOV EAX,[EDX+4]` |
| **1** | **Variable slot** — `ctx[0x3000 + value*4]` | `0x4c8b83: MOV EAX,[EAX + ECX*4 + 0x3000]` |
| **2** | **Internal code** — game-state lookup keyed by `value` (a code ≥ 1) | `0x4c8b8e …` (see §5.2) |

### 5.2 Type-2 internal codes (mechanism)

Type-2 resolves a numeric **code** to a piece of live game state. Two sub-ranges
(`0x4c8b91: CMP EDX,0x3e8`):

* **Codes `< 1000`** (`0x4c9172`): index a **global descriptor table** at
  `0x5a6c30`/`0x5a6c34` (8-byte stride: `ptr @ +0x5a6c30`, `type @ +0x5a6c34`).
  The type (0–5) selects the read width (byte/word/dword, signed/unsigned). Codes
  `1` and `6` are biased by the tribe index (`+0xc22`) before lookup
  (`0x4c917e`/`0x4c918c`), i.e. they address *this tribe's* row.
* **Codes `1000`–`1246`** (`0x3e8`–`0x4de`): dispatched through a **247-entry jump
  table** at `0x4c9210`, indexed by a byte table at `0x4c9338`
  (`0x4c8bb3: MOV CL,[ESI+0x4c9338]` / `0x4c8bb9: JMP [ECX*4 + 0x4c9210]`; range
  guard `CMP ESI,0xf6 / JA`).

  Confirmed representative handlers:

  | Code(s) | Resolves to | Evidence |
  |---|---|---|
  | `1000`–`1047` (`0x3e8`–`0x417`) | **Per-tribe attribute byte**: `*(int8 *)(0x94899a + tribeId*0x30 + code)` — 48 attribute bytes per tribe | `0x4c8bc6`–`0x4c8bd3`; cross-checked in `AI_ProcessLoopCommand` and `AI_ProcessSubroutineCall` (`tribeId*0x30 + 0x94899a + code`, guard `999 < code < 0x418`) |
  | `1048` (`0x418`) | `*(int *)(tribe + 0x94d)` | `0x4c8be5` |
  | (one type-2 code) | `*(int *)(0x5910e4 + code*0x3e)` — stride 62 (`code<<6 - code - code`) | `0x4c8bf0`–`0x4c8bfe` |
  | `1065` (`0x429`) | `*(int *)0x5a1310` | `0x4c8c05` |
  | (several) | tribe field `+0x596` flag `0x10000` toggles between two source arrays (e.g. `+0x35d` vs `+0x32b`; `0x885a9d` vs `0x885a6b`; `0x8866e2` vs `0x8866b0`) | `0x4c8c10`–`0x4c8cbd` |

> The exhaustive 1000–1246 code → meaning enumeration belongs to the deferred
> **command/constants catalog**, not this VM spec. What is fixed here is the
> *mechanism*: type-2 is a jump-table dispatch over game-state codes, with tribe
> attributes occupying `1000`–`1047`.

## 6. Operators and assignment

### 6.1 Comparisons — `AI_EvaluateComparison` (`0x4c8930`)

Reads `opcode, aIndex, bIndex` (three tokens), resolves both operands via
`AI_EvaluateScriptValue`, and compares as **signed integers**. Decoded one-by-one
from the decompilation (operand `a` = first index, `b` = second):

| Opcode | Test | Verbatim logic |
|---|---|---|
| `0x3f4` | `a > b` | `if (a <= b) return false; else true` |
| `0x3f5` | `a < b` | `if (b <= a) return false; else true` |
| `0x3f6` | `a == b` | `return b == a` |
| `0x3f7` | `a != b` | `return 1 - (b == a)` |
| `0x3f8` | `a >= b` | `if (a < b) return false; else true` |
| `0x3f9` | `a <= b` | `true; if (b < a) false` |
| default | `false` | unknown opcode ⇒ false |

### 6.2 Boolean trees — `AI_EvaluateCondition` (`0x4c8860`) & `AI_EvaluateConditionExpression` (`0x4c8a30`)

Both evaluate `AND`(`0x3fc`)/`OR`(`0x3fd`) over nested conditions, recursing into
themselves for nested boolean nodes and into `AI_EvaluateComparison` for leaf
relations. They are **near-duplicate** implementations reached from different
contexts:

* `AI_EvaluateCondition` is the form invoked from `AI_ProcessScriptBlock`'s IF
  handling (it advances IP past the `AND`/`OR` token, then evaluates both sub-conditions).
* `AI_EvaluateConditionExpression` is invoked from `AI_RunScript`'s IF handling
  (`case 0x3fc/0x3fd: iVar7 = AI_EvaluateConditionExpression(...)`).

`AND` ⇒ true iff both children are non-zero; `OR` ⇒ true iff either is non-zero;
any non-boolean/non-comparison token ⇒ false.

> `[UNVERIFIED]` why two implementations exist (compiler-duplicated inline vs two
> hand-written sites). Behaviour is equivalent as decompiled; a single Rust
> `eval_condition` should serve both. Confirm with a fixture that nests
> `AND`/`OR`/comparisons under both an `AI_RunScript`-level IF and an
> `AI_ProcessScriptBlock`-level IF.

### 6.3 Guarded block — opcode `0x3ed`

```c
descA  = ctx[0x3100] + aIndex*8;            // descriptor A (bitmask lives in .value, i.e. [descA+4])
// B operand is present iff the NEXT token is not BLOCK-BEGIN (0x3eb):
B = (nextToken != 0x3eb) ? eval(ctx[0x3100] + bIndex*8) : 0;
if ( (descA.value & (tribeId + B + DAT_00885720)) == 0 )   // tickCounter = DAT_00885720
      run_block();
else  skip_to_matching_0x3ec();
```

`DAT_00885720` is the **game tick counter** (legacy App. Q hint; used as the running
tick). The guard ANDs a per-operand bitmask with `(tribeId + B + tick)`.

> `[UNVERIFIED]` semantic intent: this is almost certainly a **tick-phased gate**
> ("run this block on a periodic schedule / every 2ⁿ turns", matching the legacy
> `EVERY_2POW_TURNS` notion). Mechanism is exact as above; the *interpretation* of
> the mask and the `+tribeId+B` bias needs a capture fixture that logs which ticks
> the block fires on.

### 6.4 Assignment — `AI_ProcessLoopCommand` (`0x4c8700`) & `AI_ProcessSubroutineCall` (`0x4c8590`)

Both read `opcode, dstIndex, srcIndex`, resolve the **destination descriptor**
(`dst = ctx[0x3100] + dstIndex*8`) and the **source value** (`eval(src)`), then
mutate the destination. The destination's `type` selects where the write lands:

* **`dst.type == 1` (variable):** target is `ctx[0x3000 + dst.value*4]` (an `i32`).
* **`dst.type == 2` (attribute), `1000 ≤ dst.value < 1048`:** target is the per-tribe
  attribute **byte** `*(int8 *)(tribeId*0x30 + 0x94899a + dst.value)`. Writes are
  truncated to 8 bits (`(char)value`).

| Opcode | Op | Function |
|---|---|---|
| `0x3ef` | `dst  = v` | `AI_ProcessLoopCommand` |
| `0x3f0` | `dst += v` | `AI_ProcessLoopCommand` |
| `0x3f1` | `dst -= v` | `AI_ProcessLoopCommand` |
| `0x401` | `dst  = a * b` | `AI_ProcessSubroutineCall` |
| `0x402` | `dst  = a / b`, **`b == 0 ⇒ dst = 0`** | `AI_ProcessSubroutineCall` |

`AI_ProcessSubroutineCall` reads an **extra** operand: it resolves both `a` and `b`
(two evaluated values) and writes their product/quotient — i.e. `MUL`/`DIV` are
binary, three-operand statements (`dst, a, b`), whereas `SET/ADD/SUB` are
two-operand (`dst, v`).

## 7. The command boundary

Statement `0x3ee` (COMMAND) is the **only** path from the interpreter into the
deferred command catalog. `AI_RunScript`/`AI_ProcessScriptBlock` call
`AI_ExecuteScriptCommand` (`0x4c6460`), which `switch`es over **168 command
sub-opcodes `0x404`–`0x4c7`** and reads each command's own operands from the
bytecode. The command operands are *not* descriptor indices in general — each
handler parses its own argument layout (arg counts differ per command). That entire
catalog — including the misclassified **flyby** opcodes `0x4b5`–`0x4be` (still
carried as `AI_ScriptCmd_CastSpell*`/`SetPatrolTarget` names in `~/decomp_export`;
corrected in `docs/specs/legacy/re_meta.md` → Flyby) — is specified separately.

For the VM rewrite, the command dispatcher is a single opaque call: *advance IP to
the command opcode, dispatch, let the handler consume its own operands, resume.*

## 8. Faithful-rewrite checklist

Behaviours a tick-accurate Rust VM must preserve (all binary-confirmed above):

1. The script runs **to completion** every `AI_RunScript` call — no instruction budget.
2. Operands are **descriptor indices**, resolved through the `{type,value}` table;
   immediates, variable slots, and internal codes are the three value types.
3. Comparisons are **signed**; the six opcode→relation mappings in §6.1 are exact.
4. `DIV` (`0x402`) and `MUL` are three-operand; **division by zero yields 0**.
5. Attribute writes/reads truncate to **8-bit** bytes at `tribeId*0x30 + 0x94899a + code`.
6. A `0` token **mutates** to `0x3ec` in the live buffer (§4.1) — context is mutable.
7. The guarded-block test (§6.3) uses the **global tick counter** `DAT_00885720`.
8. `param_1` is the **tribe pointer**, not a person; tribe index is its `+0xc22` byte.

## 9. Corrections to the legacy spec

`docs/specs/legacy/ai_scripting.md` is retained as an address map. The following
claims in it are **wrong or misleading** and are corrected here:

| Legacy claim | Reality (this spec) |
|---|---|
| "Scripts use 16-bit opcodes starting at **base 0x404**" (App. P) | Fabricated. **Statement** opcodes are the `0x3E8` range (§4). `0x404`–`0x4c7` is the *command* sub-opcode space inside `AI_ExecuteScriptCommand`, reached only via `0x3ee` (§7). The doc's own second section uses the correct `0x3E8` base — it contradicts itself. |
| `0x3ef`–`0x3f1` = "Loop constructs"; `AI_ProcessLoopCommand` | **Misnamed.** These are `= / += / -=` **assignment** (§6.4). No loop construct exists in the VM. |
| `0x401`/`0x402` = "subroutine calls"; `AI_ProcessSubroutineCall` | **Misnamed.** These are `*= / /=` arithmetic assignment (§6.4). |
| "Tribe Data Structure … Offset 0x3100: Script state pointer, 0x3104: Script IP" | **Wrong base.** `0x3100`/`0x3104` are offsets into the **script context** (`param_2`), not the tribe struct (whose stride is only `0xc65`). The tribe struct is `param_1`; its relevant field is the index byte at `+0xc22`. |
| App. Q "Game Tick System" filed under AI | Not AI — belongs to the tick-loop spec (#2/#15). |
| App. RP "PopScript Lua API" (183 functions) | Documents the **legacy Rust reimplementation**, not the binary. Trust level 6; excluded from v2. |
| App. BO scoring tables (shaman=1000, etc.), training cost bands, personality offsets | Not part of the interpreter; unverified. Belong to the deferred `ai_brain.md`, to be validated against captures before use. |

## 10. Evidence index

| Function | Address | Role |
|---|---|---|
| `AI_RunScript` | `0x004c5eb0` | Top-level interpreter loop |
| `AI_ProcessScriptBlock` | `0x004c6180` | Recursive block executor |
| `AI_ExecuteScriptCommand` | `0x004c6460` | COMMAND dispatch (deferred catalog) |
| `AI_ProcessSubroutineCall` | `0x004c8590` | `*=` / `/=` (misnamed) |
| `AI_ProcessLoopCommand` | `0x004c8700` | `=` / `+=` / `-=` (misnamed) |
| `AI_EvaluateCondition` | `0x004c8860` | Boolean tree (block form) |
| `AI_EvaluateComparison` | `0x004c8930` | Relational ops |
| `AI_EvaluateConditionExpression` | `0x004c8a30` | Boolean tree (RunScript form) |
| `AI_EvaluateScriptValue` | `0x004c8b50` | Operand → integer (3 value types) |
| `AI_UpdateTribe` | `0x0041a8b0` | Caller; supplies `(tribe_ptr, ctx)` |
| `AI_UpdateAllTribes` | `0x0041a7d0` | Per-tribe driver |

Key data addresses: tribe struct base `0x885760` (stride `0xc65`); per-tribe
attribute table `0x94899a` (stride `0x30`); type-2 global descriptor table
`0x5a6c30`/`0x5a6c34`; type-2 jump table `0x4c9210` (index byte table `0x4c9338`);
game tick counter `DAT_00885720`.

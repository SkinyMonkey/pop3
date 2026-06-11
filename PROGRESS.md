# Pop3 Progress — single source of truth

Reboot plan: `tmp/RESTART.md`. This file supersedes `.planning/` (archived) and the
status columns of `things-to-implement.md`.

## The contract

**Goal:** a reverse-engineered Populous: The Beginning — same level + same inputs +
same RNG seed ⇒ same game state every tick as `popTB.exe`, verified against runtime
captures. Rendering: visually faithful, not pixel-exact. Out of scope until v2:
audio, multiplayer, vehicles, creatures, remaining spells.

**Trust ladder** (higher beats lower; nothing from 4–6 enters the new engine without
re-derivation from 1–3):

1. Runtime Frida captures from `popTB.exe` under Wine
2. Disassembly — `ghidra-bridge dump-asm`
3. Decompilation/indexes — `ghidra-bridge decompile/search/xrefs-to/...` over `.ghidra-exports/`
4. Ghidra labels / `re_meta.md` names (LLM-assigned hints)
5. `docs/specs/legacy/` prose (weaker-model output, unverified)
6. Legacy engine code (built on 5 + ~30 documented shortcuts)

**Definition of done per subsystem:** v2 spec with evidence citations → capture
fixtures → replay test green → demo.

## Status

Status values: `unverified` (legacy code exists, faithfulness unknown — the default
after the spec-trust reset), `respec` (v2 spec in progress), `fixtures`, `implementing`,
`verified` (replay green), `out-of-scope-v1`.

**Output-validated exceptions** (kept, not rewritten): `src/data/` parsers and the
renderer — wrong parsers don't render correct levels. Their bugs are tracked as
ordinary bugs, not faithfulness work.

| # | Subsystem | Status | Notes |
|---|-----------|--------|-------|
| 0 | Workspace conversion (M0) | in progress | this branch |
| 1 | Verification harness (`pop3-verify`) | todo | Phase 2 — before any engine work |
| 2 | RNG + tick loop | unverified | first rewrite target; pipeline shakedown |
| 3 | Core object system / pool | unverified | two-tier free list this time |
| 4 | Movement + pathfinding | unverified | known broken (tmp/TODO); fixtures partial |
| 5 | Person / unit state machine | unverified | |
| 6 | Building system | unverified | legacy constants partly invented |
| 7 | Combat system | unverified | damage tables need binary verification |
| 8 | Spell system | unverified | mana mapping was guessed |
| 9 | AI / scripting | unverified | legacy used Lua; original bytecode VM required |
| 10 | Level loading | unverified | |
| 11 | Save/load (original format) | unverified | legacy used bincode |
| 12 | Economy / population / mana | unverified | |
| 13 | Scenery system | unverified | |
| 14 | Discovery system | unverified | |
| 15 | Game loop & tick ordering | unverified | merge into #2 respec |
| 16 | Terrain (data + sim flags) | unverified | rendering side output-validated |
| 17 | Camera & projection | output-validated | visual; faithfulness bugs in tmp/TODO |
| 18 | Rendering pipeline | output-validated | Tier B: audit + bugfix only |
| 19 | Sprites & animation | output-validated | |
| 20 | Water & effects | unverified | sim side; visuals output-validated |
| 21 | UI / HUD / menus | unverified | wiring redone in Phase 3 step 10 |
| 22 | Texture & palette | output-validated | |
| 23 | Utilities (RNG, math, file I/O) | unverified | RNG is #2 |
| 24 | Audio | out-of-scope-v1 | |
| 25 | Network / multiplayer | out-of-scope-v1 | |
| 26 | Creatures | out-of-scope-v1 | |
| 27 | Vehicles | out-of-scope-v1 | |

## Milestones

- **M0** — workspace boots, renders level 1 identically to old `main`
- **M1** — 1000-tick zero-input replay of level 1 matches capture (RNG, objects)
- **M2** — level 1 playable & winnable; movement/combat/buildings replay-verified
- **M3** — campaign levels 1–5 with original AI bytecode VM
- **M4** — full 25-level campaign + original-format save/load

## Accepted divergences

(Each entry: binary address, what diverges, why accepted. Empty so far.)

| Address | Divergence | Reason |
|---------|------------|--------|

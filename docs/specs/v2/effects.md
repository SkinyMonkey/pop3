# Spell Effects — visual representation (v2, staged)

> **Subsystem:** spell effects rendering (the visible result of casting a spell).
> **Target binary:** PopRe `pop3.exe` via `ghidra-bridge` over `.ghidra-exports`
> (NOT popTB; see memory `ai-specs-are-poptb-not-popre`). Addresses below are
> PopRe VAs.
> **Trust level:** 2–4. RTTI class/union descriptors + sol3 binding tables +
> asset strings are confirmed (level 2–3); the per-effect draw path and
> bank/frame indices are **not yet traced** and are marked `[INFERRED]` /
> `[OPEN]`.
> **Status:** STAGED. This documents the *structure* and *taxonomy* of the effect
> subsystem and classifies each effect's representation. The exact draw dispatcher
> and per-effect sprite-bank/frame (or 3D model) indices are open items in §6 —
> resolve those before claiming pixel-accurate rendering.

## 1. Scope & goal

Goal: pin **how each spell effect is drawn** — 2D billboard sprite, a drawn
geometric primitive (line/circle), a 3D mesh, or a procedural terrain change — so
`src/bin/spell_demo.rs` can render spell effects the way the original does, not as
placeholders.

Out of scope: spell *mechanics* (damage, terrain mutation, mana), casting input,
and the AI that queues spells (see `ai_script_commands.md`).

## 2. Structural model — one Thing, a type union

Effects are **Things**. They share a single `EffectGeneral` struct whose
per-effect-type data lives in an anonymous **union `ut`** (confirmed from RTTI
member descriptors `ut@EffectGeneral@@U<EffectType>@@`).

Union `ut` members confirmed in `EffectGeneral`:

```
EffectGeneral.ut = union {
    EffectAlphaInfo   EffectFireCloud   EffectFireStorm
    EffectFlatten     EffectFlyThing    EffectLandAlter
    EffectLavaFlow    EffectLightning   EffectMeteor
    EffectOrbiter     EffectRSPrepare   EffectSwamp
    EffectAtlantis
}
```

Additional effect usertypes that are **not** in the `ut` union (separate Things or
sub-structs — [OPEN] which): `EffectArmageddon`, `EffectAttached`,
`EffectEarthquake`, `EffectFireRoll`, `EffectInsectPlague`, `EffectLandBridge`,
`EffectWhirlwind`.

Evidence: `ghidra-bridge strings "ut@EffectGeneral"` and `strings "DPQEffect"`.

### Spell-delivery descriptor (CONFIRMED offsets)

`SpellGeneral` (binding `FUN_00b935d0` @ `0x00b935d0`) registers a spell's
delivery/generator sub-struct whose member-pointer offsets are recoverable from
the sol registration (`FUN_00bec4a0` call, field→offset locals). **This is the
struct that selects the visual.** Offsets (bytes from struct base):

| Field | Offset | Type | Meaning |
|---|---|---|---|
| `Count` | +0x00 | u16 | items to spawn |
| `ItemsPerTurn` | +0x02 | u16 | spawn rate |
| `SpeedPerItem` | +0x04 | — | per-item speed |
| `Flags` | +0x06 | — | |
| `StartCoord` | +0x08 | Coord | origin |
| `TargetCoord` | +0x0e | Coord | destination |
| **`EffectType`** | **+0x14** | u8 | which effect (selects `ut` union arm) |
| **`EffectModel`** | **+0x15** | u8 | **visual model index** (sprite/mesh selector) |
| `EffectNumParams` | +0x16 | u8 | param count |
| `ItemDuration` | +0x17 | u8 | lifetime |
| `EffectParams` | +0x18 | array | per-effect params |
| `ItemThingIdxs` | +0x20 | array | spawned thing handles |
| `TargetThingIdx` | +0x60 | | |

`EffectModel` (a single byte) is the key: it indexes the same unified **model**
table used by buildings (`BldgModel`) and other things (`CurrModel`) — the
renderer dispatches sprite-vs-mesh off the model. The model table is the same
asset system `pop3-data` already parses (HSPR sprite banks + OBJS 3D meshes).

### Common render-relevant fields (Lua-exposed, offsets [OPEN])

From the sol3 usertype registrations (the master binding functions
`FUN_008df250` @ `0x008df250` and `FUN_00a76f20` @ `0x00a76f20` — both register
the whole game's Lua surface, so they enumerate field *names*, not offsets):

| Field | Meaning (inferred) |
|---|---|
| `DrawInfo`, `DrawnAtInfo`, `DrawFrame`, `LevelDrawNum` | what/where to draw |
| `CurrentFrame`, `Count`, `Duration` | animation / lifetime |
| `FaceIdx`, `AngleXZ`, `AngleIncrs` | orientation (billboard facing / mesh) |
| `Alpha` (+ `EffectAlphaInfo` union member) | transparency |
| `Height`, `AltOffset`, `CellRadius`, `CurrRadius` | size / placement |
| `Colour` | tint (used by `SpriteCircles`?) |

Offsets must be read from the individual getter/setter thunks (each `"Name"` in
the binding is preceded by its handler `&LAB_xxxx`). [OPEN]

## 3. Render primitives

Confirmed sol usertypes for drawing primitives:

- **`TbSprite`** — binding `FUN_005fcfc0` @ `0x005fcfc0`; the engine's 2D
  billboard sprite. Field `PAC` (palette?) seen in RTTI. This is the billboard
  blit path for sprite-based effects.
- **`SpriteCircles`** — usertype @ string `0x01383598`; draws circle/ring
  geometry (ground rings — e.g. spell area markers, shockwaves).

## 4. Asset sources (confirmed strings)

- **HSPR sprite banks**: `data/hspr0-0.dat`, `data/hspr1-0.dat`, generic
  `data/hspr%?i-%?i.dat`. Loader emits `"Cannot open sprite bank"` /
  `"Invalid bank number!"`; helper named `change_sprite_bank`. Billboard
  (`TbSprite`) effects index frames out of these banks. **pop3-data already
  parses HSPR** (used by `src/bin/sprite_viewer.rs`) — reuse it.
- **BIGF**: `data/bigf0-%c.dat`, `data/bigf0-0.dat` — large sprites/fonts.
- 3D meshes: OBJS/PNTS/FACS banks (used by `src/bin/pop_obj_view.rs`) for
  mesh effects [INFERRED — bank id per effect OPEN].

## 5. Per-effect representation (classified)

Confidence: **C** = confirmed by structure/RTTI; **I** = inferred from union
data type / name, draw path not yet traced.

| Effect | Representation | Asset / primitive | Conf |
|---|---|---|---|
| `EffectLightning` | **Drawn polyline** — `ut::EffectLightning` holds `array<Coord3D>` (the bolt segments). Not a sprite. | line geometry between Coord3D points | C (structure), I (draw call) |
| `EffectFireCloud` | Billboard sprite | HSPR bank, animated frames | I |
| `EffectFireStorm` | Billboard sprite(s), multiple | HSPR | I |
| `EffectFireRoll` | Billboard sprite | HSPR | I |
| `EffectMeteor` | Moving 3D thing / sprite (falls) | mesh or HSPR | I |
| `EffectOrbiter` | Moving 3D thing (orbits) | mesh | I |
| `EffectFlyThing` | Moving 3D thing | mesh | I |
| `EffectWhirlwind` | Billboard sprite column / sprite stack | HSPR | I |
| `EffectInsectPlague` | Sprite swarm | HSPR | I |
| `EffectSwamp` | **Procedural** terrain (swamp tiles) + bubble sprites | terrain + HSPR | I |
| `EffectFlatten` | **Procedural** terrain flatten | terrain | I |
| `EffectLandAlter` / `EffectLandBridge` | **Procedural** terrain raise/bridge | terrain | I |
| `EffectEarthquake` | **Procedural** terrain shake | terrain | I |
| `EffectLavaFlow` | **Procedural** terrain (lava) + glow sprites | terrain + HSPR | I |
| `EffectAlphaInfo` | Not an effect — alpha/blend config used by others | — | C |
| `EffectArmageddon`, `EffectAtlantis`, `EffectRSPrepare`, `EffectAttached` | Composite / control effects | [OPEN] | I |

**Takeaway for the demo:** three rendering families —
1. **billboard sprites** (HSPR) — the largest group; reuse `sprite_viewer` path.
2. **drawn geometry** — Lightning (polyline), area rings (`SpriteCircles`).
3. **procedural terrain** — the land-alter family; out of demo render scope,
   represent with a marker.

## 6. Open items

Static-analysis limit reached: the draw dispatcher and the `EffectModel`→asset
table are unlabeled `FUN_` code with **no string/RTTI anchors** in this PopRe
export, so a precise static trace is high-cost/low-yield. The remaining unknowns
are best resolved per the trust ladder by **runtime capture** (captures rank
above disassembly):

1. **Per-spell `EffectModel`/`EffectType` constants** — capture the
   spell-delivery descriptor (struct in §2) at cast time for each castable spell.
   This is the authoritative spell→visual map and beats grinding the static
   model table.
2. **`EffectModel` → asset resolution** — confirm whether a given model index
   resolves to an HSPR sprite (2D billboard) or an OBJS mesh (3D). `pop3-data`
   already loads both; the demo can render any model index by reusing
   `sprite_viewer` (HSPR) / `pop_obj_view` (OBJS) and the mapping verified
   visually against the game.
3. **Lightning bolt geometry** — segment count, jitter, color, width
   (`ut::EffectLightning` `array<Coord3D>`) — capture or trace later.
4. **`EffectGeneral` field offsets** (DrawInfo/CurrentFrame/FaceIdx/Alpha/Height)
   — read getter thunks if/when needed for animation fidelity.

### Recommendation for the demo

Build `spell_demo.rs` as an **EffectModel/EffectType browser + caster**: select an
`EffectType`/`EffectModel` (and Lightning's polyline / SpriteCircles ring as
special cases), spawn at a clicked world position, render the real asset for that
model index via the existing `pop3-data` loaders. Wire exact per-spell constants
from a capture (item 1) once available; until then the browser lets us confirm
each spell's visual against the running original.

## 7. Sources

- RTTI: `ghidra-bridge strings "ut@EffectGeneral" | "DPQEffect" | "AVEffect"`.
- Bindings: `decompile FUN_008df250`, `FUN_00a76f20`, `FUN_00b935d0`
  (SpellGeneral), `FUN_005fcfc0` (TbSprite).
- Assets: `strings "hspr" | "bigf" | "sprite bank"`.

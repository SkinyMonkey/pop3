# File-format cross-reference: Pop-World-Editor ↔ pop3

This page maps the on-disk file formats used by Populous: The Beginning, as
documented in the third-party **Pop-World-Editor** (PopEdt) source (`pop.h`,
`engine.cpp`, `script.h`, `3ds.cpp`, `network.h`), against the Rust parsers in
`src/data/`.

PopEdt reads and writes the **same** `.dat` / `.hdr` / `.ver` files as the
original game binary that pop3 reimplements, so `pop.h` is effectively a
recovered, authoritative type dictionary. Use it to verify byte layouts.

| Status | Meaning |
|--------|---------|
| ✅ | Parsed and exposed correctly |
| ⚠️ | Partially parsed (some fields skipped or relabeled) |
| 🟡 | Documented but not yet parsed |
| ⛔ | Out of scope for pop3 (editor-only) |

---

## 1. Level data file (`.dat`)

`LEVELDATv2` / `LEVELDATv3` — `pop.h:1133-1153`. pop3 reads the v2 layout
(seeking past LandBlocks / LandOrients / NoAccessSquares without using them
because the height grid is enough for rendering).

| Field | Type | PopEdt ref | pop3 status | pop3 ref |
|-------|------|-----------|-------------|----------|
| `MAGIC[5]` ("LEVL3") | char[5] | `pop.h:1135` | 🟡 not validated | — |
| `GroundHeight[128*128]` | u16 LE | `pop.h:1137` | ✅ via `Landscape::from_reader` | `level.rs:422` |
| `LandBlocks[128*128]` (v2) | u8 | `pop.h:1146` | 🟡 skipped (seek past 0x4000) | `level.rs:138` |
| `LandOrients[128*128]` (v2) | u8 | `pop.h:1147` | 🟡 skipped | `level.rs:140` |
| `NoAccessSquares[128*128]` | u8 | `pop.h:1148` | 🟡 skipped | `level.rs:142` |
| `psi[4]: PLAYERSAVEINFO` | 16 B × 4 | `pop.h:1062` | ✅ as `LevelRes.player_starts` | `level.rs`, `units.rs` |
| `ssi: SUNLIGHTSAVEINFO` | 3 B | `pop.h:1054` | ✅ as `LevelRes.sunlight` | `level.rs:90-108` |
| `Things[2000]: THINGSAVE` | 55 B × 2000 | `pop.h:1116` | ✅ as `LevelRes.units` (with union variants) | `units.rs` |
| `asi[50]: ACCESSSAVEINFO` (v2) | 3 B × 50 | `pop.h:1046` | 🟡 not parsed | — |

### THINGSAVE union variants

The 48-byte union region (bytes 7..55 of a THINGSAVE) is interpreted by the
`Type` byte. pop3 exposes this via [`UnitRaw::payload`](../../src/data/units.rs)
returning a `ThingPayload` enum.

| Type | Variant | PopEdt ref | pop3 |
|------|---------|-----------|------|
| 2 Building | `BUILDINGSAVE { SLONG Angle }` | `pop.h:1072` | ✅ `BuildingSave` |
| 5 Scenery  | `SCENERYSAVE { portal_*; angle; user_id; island_alt/num; bridge_num }` | `pop.h:1078` | ✅ `ScenerySave` |
| 6 General  | `GENERALSAVE { discovery_*; mana_amt }` | `pop.h:1091` | ✅ `GeneralSave` |
| 6 (alt) Trigger | `TRIGGERSAVE { trigger_*; cell_radius; thing_idxs[10]; pray_time; … }` | `pop.h:1101` | ✅ `TriggerSave` (via `as_trigger`) |
| other (Person, Creature, Vehicle, Shape, Effect) | — | — | exposed as raw `[u8; 48]` |

---

## 2. Level header file (`.hdr`)

`LEVELHEADERv2` / `LEVELHEADERv3` — `pop.h:1017-1043`. Previously only bytes
96 and 97 were extracted; the full header is now parsed.

| Offset | Field | Type | PopEdt ref | pop3 status |
|--------|-------|------|-----------|-------------|
| 0–55 | `DefaultThings: PLAYERTHINGS` | 56 B | `pop.h:1001` | ✅ `LevelHeader.default_things` |
| 56–87 | `Name[32]` | char | `pop.h:1020` | ✅ `LevelHeader.name` |
| 88 | `NumPlayers` | u8 | `pop.h:1021` | ✅ |
| 89–91 | `ComputerPlayerIndex[3]` | u8 | `pop.h:1022` | ✅ |
| 92–95 | `DefaultAllies[4]` | u8 | `pop.h:1023` | ✅ |
| 96 | `LevelType` | u8 | `pop.h:1024` | ✅ (was already parsed) |
| 97 | `ObjectsBankNum` | u8 | `pop.h:1025` | ✅ (was already parsed) |
| 98 | `LevelFlags` | u8 | `pop.h:1026` | ✅ → `LevelConfig` |
| 99 | `Pad[1]` | u8 | `pop.h:1027` | n/a |
| 100–611 | `Markers[256]` | u16×256 | `pop.h:1028` | ✅ decoded to `[Option<(u8, u8)>; 256]` |
| 612 | `StartPos` | u16 | `pop.h:1029` | ✅ |
| 614 | `StartAngle` | u16 | `pop.h:1030` | ✅ (0x100 = 45°) |
| 616 | `Version` (v3 trailer) | u8 | `pop.h:1038` | ✅ |
| 617 | `MaxAltPoints` | u32 | `pop.h:1039` | ✅ |
| 621 | `MaxNumObjects` | u32 | `pop.h:1040` | ✅ |
| 625 | `MaxNumPlayers` | u32 | `pop.h:1041` | ✅ |
| 629… | `Script2[10][32]` | char | `pop.h:1042` | ✅ trimmed strings |

### Level flags (`pop.h:945-950`)

| Bit | Flag | Engine consumer |
|----:|------|-----------------|
| 0 | `LEVEL_FLAGS_USE_FOG` | 🟡 no consumer yet (fog-of-war rendering pending) |
| 1 | `LEVEL_FLAGS_SHAMAN_OMNI` | 🟡 no consumer yet (spell subsystem pending) |
| 4 | `LEVEL_FLAGS_NO_GUEST` | 🟡 no consumer yet (spell subsystem pending) |
| 5 | `LEVEL_NO_REINCARNATE_TIME` | ✅ honored in `victory::check_victory_conditions` |

### PLAYERTHINGS (starting abilities — `pop.h:1001`)

The 56-byte default-abilities block is parsed but not yet consumed: actual
spell/building gating depends on the spell subsystem
(`docs/specs/spells.md`). Bitmask layouts:

- `SpellsAvailable: u32` — bit N = spell type N available (1=Burn … 21=Teleport).
- `BuildingsAvailable: u32` — bit N = building type N available.
- `SpellsAvailableLevel: u32` — granted in this level only.
- `SpellsAvailableOnce[32]: u8` — one-shot counts per spell.
- `VehiclesAvailable: u16`, `TrainingManaOff: u8`, `Flags: u8`.

---

## 3. Version file (`.ver`) — ✅ parsed

`LEVELVERSION` — `pop.h:1156`. 68 bytes total: `VersionNum: i32`,
`CreatedBy[32]: char`, `CreatedOn[28]: char`, `CheckSum: i32`. Exposed via
[`LevelVersion::from_file`](../../src/data/level.rs) and
`LevelPaths::ver_path`. The game does not depend on it for gameplay; it
carries creation metadata for the level editor.

---

## 4. AI scripts

Pop-World-Editor handles compiled **PopScript bytecode** (`script.h` defines
~197 opcodes starting at 1000, plus an extensive internal-variable table at
`pop.h:382-947`). pop3 instead uses **Lua scripts** (`src/data/scripts.rs` —
file pattern `level_XX_tribe_Y.lua`).

| Aspect | PopEdt | pop3 |
|--------|--------|------|
| Script format | binary bytecode (`SCRIPTINFO` w/ `Codes[4096]` + `Fields[512]`) | Lua text |
| Opcodes | enumerated in `script.h:14-1107` | only the game-state *queries* are bridged (`engine/ai/popscript.rs`) |
| Compile/decompile | `script_compile.cpp`, `script_decompile.cpp` | n/a |

⛔ **Not a gap unless the original `.SCR` files need to be importable.** The
~197-opcode table in `script.h` is useful future reference if such an importer
is ever wanted, but pop3 deliberately uses Lua.

---

## 5. Network packets — 🟡 deferred until multiplayer

`network.h` defines:

- `struct Packet { WORD wType; WORD wData[13]; }` (28 bytes, 30 packet types).
- `struct LandPacket { WORD wType; WORD wData[4096]; }` (large terrain-sync buffer).

Packet types (selection): `PACKETTYPE_CREATE_OBJECT (0)`, `MOVE_OBJECT (2)`,
`LAND_MODIFY (10)`, `UPDATE_VIEW (11)`, `ALLIES (14)`, `FOG_OF_WAR (16)`,
`OMNIPRESENCE (17)`, `OBJECT_BANK (21)`, `MAP_TYPE (22)`, plus marker /
trigger / sync variants.

Defer until the save-network subsystem starts; cross-link from
`docs/specs/level_save_network.md`.

---

## 6. 3DS models — ⛔ not a gap

PopEdt ships `.3ds` meshes in its `data/` directory (`hut1_blue.3ds`,
`tree01.3ds`, `eagle.3ds`, …) and a chunk parser in `3ds.cpp` (M3DMAGIC,
MDATA, NAMED_OBJECT, POINT_ARRAY, FACE_ARRAY, TEX_VERTS, MAT_ENTRY).

This is **editor-only**: the original game (and pop3) load models from the
proprietary `OBJS0-*.DAT` / `PNTS0-*.DAT` / `FACS0-*.DAT` / `SHAPES.DAT`
format (already parsed in `src/data/objects.rs`). Adding a `.3ds` parser would
not improve faithfulness to the original.

---

## 7. Notable constants (sanity-check table)

| Constant | PopEdt | pop3 |
|----------|--------|------|
| `MAX_THINGS` | 66535 (`pop.h:11`) | — (uses fixed 2000 v2 slots; matches `MAX_V2_THINGS`) |
| `MAX_V2_THINGS` | 2000 (`pop.h:12`) | `LEVEL_UNIT_SLOTS = 2000` (`level.rs:10`) |
| `MAX_MANA_VALUE` | 1,000,000 (`pop.h:14`) | `MAX_MANA` (economy::mana) |
| `OWNER_NEUTRAL` | `-1` as SBYTE (`pop.h:251`) | `Option<u8>::None` via `UnitRaw::owner()` |
| `ANGLE_45` | `0x0100` (`pop.h:959`) | matches: full circle = `0x800` (`raw.angle() & 0x7FF`) |
| Tribe ids | `OWNER_BLUE=0` … `OWNER_GREEN=3` (`pop.h:251-272`) | same |
| ModelType ids | `T_PERSON=1` … `T_SPELL=11` (`pop.h:274+`) | `ModelType` enum (`units.rs:9-21`) |

---

## 8. Open work

1. **`PLAYERTHINGS` consumer** — ✅ bitmask accessors `is_spell_available`,
   `is_building_available`, `is_vehicle_available`, `one_shot_count`, plus
   `available_spells` / `available_buildings` iterators on `PlayerThings`.
   The spell/building subsystem will read these via `LevelRes.header.default_things`
   when it wires real availability gating.
2. **Markers in AI** — ✅ `AiSystem::set_level_markers` populates the AI
   bridge's `marker_entries` from `LevelHeader.markers` at level load. AI
   commands like `ATTACK_MARKER` / `SET_BASE_MARKER` / `GUARD_AT_MARKER`
   resolve marker ids via the existing `resolve_marker` helper.
3. **Start position** — ✅ engine stores `player_starts: [PlayerSaveInfo; 4]`
   and the level-load camera centring falls back to
   `player_starts[player_tribe]` when no shaman has been placed yet.
   The previously hardcoded Blue-tribe ghost preview now uses
   `game_world.player_tribe`.
4. **Fog of war / shaman omnipresence / no_guest** — ✅ flag accessors on
   `LevelConfig`: `is_fog_enabled`, `is_shaman_omni`, `guest_spells_allowed`.
   Consumers in render / spell / UI layers will plug in when those subsystems
   wire up.
5. **`.ver` parsing** — ✅ `LevelVersion::from_file` reads the 68-byte
   metadata struct.
6. **`LevelLoadError`** — ✅ `LevelRes::try_new` returns
   `Result<LevelRes, LevelLoadError>` with variants for I/O, truncated DAT /
   HDR, and unknown landscape type. `new()` remains as a panicking wrapper
   for legacy call sites.
7. **Full owner-type migration** — ✅ `Unit.tribe_index` and
   `ObjectHeader.tribe` now expose `owner() -> Option<u8>` accessors.
   Storage stays as `u8` with 255-sentinel for compact representation; the
   accessor is the recommended API for any new code that would otherwise
   risk a `.min(3)`-style sentinel-clamping bug. The known
   `MINIMAP_TRIBE_COLORS[(dot.tribe_index).min(3)]` bug was fixed by routing
   neutral entities to a dedicated `MINIMAP_NEUTRAL_COLOR`, and the
   out-of-bounds sprite-row computation for wild units was guarded.

---

## 9. Bug fixes applied during the cross-reference pass

- **`TribeConfigRaw` → `PlayerSaveInfo`**: the 16-byte struct in the level
  `.dat` between landscape and sunlight was `PLAYERSAVEINFO psi[4]` per
  `pop.h:1062`, not a tribe configuration. Renamed and field-typed correctly.
- **Neutral-owner mis-coloring**: `UnitRaw.tribe_index = 0xFF` (i8 = -1 =
  neutral) used to map to tribe 3 (Green) inside `building_obj_index` via
  `.min(3)`. Replaced with `Option<u8>` and a `subtype == 18` (Vault of
  Knowledge) special-case; neutral tribed buildings now return `None`.
- **THINGSAVE union mis-decoded for non-Building variants**: bytes 7..10 are
  the building angle, but for Scenery / General / Trigger the same region
  encodes different fields. Added `ThingPayload` enum.
- **`LEVEL_NO_REINCARNATE_TIME` ignored**: tribes with zero population were
  always eliminated by timer even on levels that explicitly disabled it.
  Wired through `LevelConfig` → `victory::check_victory_conditions`.

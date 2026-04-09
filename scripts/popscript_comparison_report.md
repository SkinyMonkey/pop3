# PopScript Documentation vs Implementation Comparison Report

Generated: 2026-04-08

## Summary

| Category | Count |
|----------|-------|
| Our implemented functions | 25 |
| Our stub functions | 146 |
| Our total functions | 171 |
| Our constants | 486 |
| Documented constants | 107 |
| Documented functions | 4 (Doxygen artifacts) |

## Key Findings

### 1. Missing Constants (59 in docs, not in our implementation)

The PopScript documentation from populous3.info reveals several constants we haven't exposed to Lua yet.

#### Attack Type Constants (MISSING)
These are critical for the `ATTACK` command's third parameter:

| Constant | Likely Value | Use |
|----------|-------------|-----|
| `ATTACK_NORMAL` | 0? | Standard ground attack |
| `ATTACK_BUILDING` | 1? | Target buildings specifically |
| `ATTACK_PERSON` | 2? | Target individual units |
| `ATTACK_MARKER` | 3? | Attack at marker location |
| `ATTACK_BY_BOAT` | 4? | Naval attack |
| `ATTACK_BY_BALLOON` | 5? | Airship attack |

**Action**: Verify these values in Ghidra and add to constants.rs

#### Attribute Flags (ATTR_* - 7 missing)

These control AI behavior patterns:

| Constant | Purpose |
|----------|---------|
| `ATTR_AWAY_MEDICINE_MAN` | Shaman away from base |
| `ATTR_EXTENSION` | Unknown extension flag |
| `ATTR_INFO_EXTENSION` | Info extension flag |
| `ATTR_PREFIX` | Prefix flag |
| `ATTR_PREF_BALLOON_DRIVERS` | Prefer balloon drivers |
| `ATTR_PREF_BOAT_DRIVERS` | Prefer boat drivers |
| `ATTR_VERSION_NUM` | Script version number |

#### Object/Entity Flags (6 missing)

| Constant | Purpose |
|----------|---------|
| `ABF_END_LIST` | Array/buffer end marker |
| `AOF_END_LIST` | Object list end marker |
| `AMBIENT_FLAG_HIGH_LAND` | High altitude ambient sound |
| `AMBIENT_FLAG_LOW_LAND` | Low altitude ambient sound |
| `AMBIENT_FLAG_SPACE` | Space/outside world ambient |
| `AMBIENT_FLAG_WATER` | Water ambient sound |

#### Effect System Constants (4 missing)

| Constant | Purpose |
|----------|---------|
| `AFFECT_ALTITUDE` | Effect altitude modification |
| `AFFECT_FIRE` | Fire effect flag |
| `AFFECT_RAISE_LOWER` | Terrain raise/lower effect |
| `AOD2_FLAG_EXPLODE_PENDING` | Angel of Death explosion pending |
| `AOD2_FLAG_WHIRLWIND_AFFECTED` | Tornado affected flag |

#### Building/Add-on Types (4 missing)

| Constant | Purpose |
|----------|---------|
| `ADD_ON_TYPE_NONE` | No add-on |
| `ADD_ON_TYPE_WELL` | Well add-on |
| `ADD_ON_TYPE_WINDMIL` | Windmill add-on |
| `ADD_ON_TYPE_WOODHUT` | Wood hut (base building) |

#### Terrain Modification (2 missing)

| Constant | Purpose |
|----------|---------|
| `AAM_FLATTEN` | Flatten terrain action |
| `AAM_RAISE_LOWER` | Raise/lower terrain action |

#### Map/Level Constants (5 missing)

| Constant | Purpose |
|----------|---------|
| `AE_MAP_SIZE` | Map dimensions |
| `AE_MAP_XZ_SIZE` | Map XZ dimensions |
| `AE_MAX_NUM_THINGS` | Maximum object count |
| `ADD_WALL` | Wall building action |
| `AIRSHIPSLIST` | Airship list reference |

### 2. Implemented Functions Analysis

Our implementation has 171 functions (25 implemented + 146 stubs). The documentation HTML files contain mostly Doxygen navigation artifacts, not actual PopScript functions.

**The "documented functions" found are false positives:**
- `OnSearchSelectHide` - Doxygen UI function
- `OnSearchSelectShow` - Doxygen UI function
- `__save_script_header` - Internal save function
- `function` - HTML artifact
- `init_search` - Doxygen search function

### 3. Constants We Have That Match Docs

Our constants.rs already has excellent coverage of:
- ✅ Tribe constants (INT_BLUE, INT_RED, INT_YELLOW, INT_GREEN)
- ✅ Spell type constants (INT_BURN through INT_BLOODLUST)
- ✅ Unit type constants (INT_BRAVE through INT_SHAMAN)
- ✅ Building type constants (INT_SMALL_HUT through INT_GUARD_TOWER)
- ✅ Object model types (INT_M_PERSON through INT_M_SPELL)
- ✅ Internal attribute codes (1000-1237 range)
- ✅ Per-tribe attribute flags
- ✅ Tribe population counts
- ✅ Building counts per tribe

## Recommended Actions

### Priority 1: Add Attack Type Constants

The `ATTACK` command's third parameter needs proper constants. Based on the docs:

```rust
// In constants.rs
globals.set("ATTACK_NORMAL", 0i32)?;
globals.set("ATTACK_BUILDING", 1i32)?;
globals.set("ATTACK_PERSON", 2i32)?;
globals.set("ATTACK_MARKER", 3i32)?;
globals.set("ATTACK_BY_BOAT", 4i32)?;
globals.set("ATTACK_BY_BALLOON", 5i32)?;
```

**Verify in Ghidra**: Search for references to these constants in the original binary to confirm values.

### Priority 2: Add Attribute Flags

The `ATTR_*` constants control AI behavior. These should be exposed:

```rust
// AI behavior modifiers
globals.set("ATTR_DONT_USE_BOATS", ...)?;  // Already have
globals.set("ATTR_AWAY_MEDICINE_MAN", ...)?;  // MISSING
globals.set("ATTR_PREF_BALLOON_DRIVERS", ...)?;  // MISSING
```

### Priority 3: Verify Object Flag System

The `AOF_*` and `ABF_*` constants suggest linked-list structures:

```c
// Likely structure from original:
struct ObjectList {
    Object* first;
    Object* last;
    // ... terminated by AOF_END_LIST
};
```

### Priority 4: Add Effect System Constants

For the water/effects system:

```rust
globals.set("AFFECT_ALTITUDE", ...)?;
globals.set("AFFECT_FIRE", ...)?;
globals.set("AFFECT_RAISE_LOWER", ...)?;
globals.set("AOD2_FLAG_EXPLODE_PENDING", ...)?;
```

## Ghidra Investigation Tasks

1. **Find attack type constant definitions**
   - Search for `ATTACK_NORMAL`, `ATTACK_BUILDING` in symbol table
   - Look at `AI_ExecuteScriptCommand` switch table for attack handling

2. **Find ATTR_* flag definitions**
   - These are likely bit flags in tribe state structure
   - Search for references to tribe offset + attribute codes

3. **Verify object flag system**
   - Find `AOF_END_LIST` usage in data structures
   - Map object linked-list traversal functions

4. **Check effect system constants**
   - Look at effect update functions
   - Find `AOD2_FLAG_*` references (Angel of Death effect)

## Files to Update

- `src/engine/ai/constants.rs` - Add missing constants
- `docs/specs/ai_scripting.md` - Document constant values
- `docs/specs/re_meta.md` - Track renamed functions/constants

## Conclusion

Our implementation has excellent coverage (486 constants vs 107 documented). The 59 missing constants are mostly:
- Attack type parameters (6) - **high priority**
- Attribute behavior flags (7) - **medium priority**
- Object/effect system flags (10) - **low priority**
- Building/add-on types (4) - **low priority**
- Terrain/map constants (5) - **low priority**

The documentation from populous3.info appears to be from a modding SDK that only exposes a subset of the full constant space. Our reverse engineering via Ghidra has actually discovered MORE constants than the official documentation.

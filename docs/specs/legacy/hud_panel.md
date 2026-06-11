# In-game HUD panel system

Verified against popTB.exe via `~/decomp_export` (2026-06). Supersedes the
invented "In-Game HUD" table that used to live in `ui_and_input.md`.

**Identification note:** `data\plspanel.spr` and the widget table at
`0x57b978` are the *level-select screen*, not the in-game HUD — see
`level_select.md`. The in-game HUD is a data-driven **left sidebar** plus
tab pages and a bottom bar.

## Architecture

- Static **panel definition table** at `0x577886` (stride 0x3e, id==0
  terminates) describes each panel's rect in 640×480 virtual coordinates
  and points to its element list.
- Static **element definition lists** (stride 0x42, type==9 terminates)
  describe each widget: rect (panel-relative), command id, render callback,
  tooltip string id, icon index.
- `Panel_Open` (0x45b930, was mislabeled `Effect_TriggerCinematic`) copies a
  panel def into the runtime panel array at 0x67c2f8/0x67c336, converting
  pixel coords to 16.16 screen fractions: `frac = (v << 16) / 640` (x/w) and
  `/ 480` (y/h). `Element_Instantiate` (0x45bd60) does the same for elements
  into the runtime pool at 0x67c604 (stride 0x71).
- Scaling to the actual screen: `px = (frac * screen_dim) >> 16`
  (**truncating** integer math, not rounding).
- The per-frame **panel manager** `FUN_00459ee0` walks open panels and
  invokes each element's render callback (def +0x29). It is called from
  `UI_RenderGamePanel` (0x492390).
- 3D viewport inset (`FUN_00422090`): with the sidebar open,
  `vp.x = sidebar_w + border*8`, `vp.y = border*8`,
  `vp.w = screen_w − sidebar_w − border*16`,
  `vp.h = screen_h − border*16`. `sidebar_w` comes from
  `Panel_GetSidebarWidth` (0x45ae60, was mislabeled
  `Frontend_UpdateLevelSelectPos`): the open panel's 16.16 width fraction
  scaled to the screen (100/640 of width; 0x80 px fallback when none open).

## Record layouts

Panel def (stride 0x3e): `+0` id (u32), `+4` mode (u16), `+6` elements ptr
(8bpp), `+0xa` elements ptr (16bpp), `+0xe/+0x12/+0x16/+0x1a` x/y/w/h
(i32, 640×480 virtual), `+0x1e..+0x2d` four callbacks.

Element def (stride 0x42): `+0` command id (i32), `+4` type (u8: 0 static,
1 button, 2 canvas/minimap, 3 toggle, 5 panel-opener tab, 9 list end),
`+0xd/+0x11/+0x15` input handler fn ptrs, `+0x19/+0x1b` interactive x,y,
`+0x1d/+0x1f` draw x,y, `+0x21/+0x23` w,h, `+0x25` param (sound/cmd),
`+0x29` render callback, `+0x2d` tooltip/help string id, `+0x37` icon
index, `+0x3f` group id, `+0x41` flags (bit0 8bpp-only, bit1 16bpp-only,
bit2 no-input, bits 4-6 align mode).

## Panel definitions (@ 0x577886)

| id | rect (x,y,w,h) 640×480 | elements (8bpp / 16bpp) | role |
|----|------------------------|--------------------------|------|
| 1 | (0, 0, 100, 480)   | 0x575668 | main sidebar |
| 2 | (0, 204, 100, 277) | 0x575ff8 / 0x5764a0 | buildings tab page |
| 3 | (0, 204, 100, 277) | 0x576c20 | spells tab page |
| 4 | (0, 204, 100, 277) | 0x576eb8 | units tab page |
| 6 | (99, 444, 106, 32) | 0x575d20 | bottom bar |
| 7 | (0, 204, 100, 277) | 0x576988 | alt buildings page (MP/one-shot) |

## Sidebar elements (panel 1 @ 0x575668)

Coordinates are panel-relative (panel origin (0,0)).

| # | rect | type | cmd | render cb | role |
|---|------|------|-----|-----------|------|
| e24 | (0,0,100,96) | 2 canvas | — | 0x401280 | **minimap** (content via Minimap_Update 0x42b950 / Minimap_RenderTerrain 0x42ba10; tooltip 753) |
| e04 | (0,86→draw 0,82, 34×27) | 5 tab | 39 | 0x405b10 | **spells tab** (opens panel 3) |
| e03 | (32,86→32,82, 34×27) | 5 tab | 38 | 0x405b10 | **buildings tab** (opens panel 2) |
| e05 | (64,86→64,82, 34×27) | 5 tab | 40 | 0x405b10 | **units tab** (opens panel 4) |
| e23 | (0,90,100,32) | 1 | — | 0x405ec0 | **mana display** (string id 700 in icon field) |
| e22 | (0,110,100,64) | 1 | — | 0x405e40 | info block A (population/followers) |
| e21 | (0,149,100,64) | 1 | — | 0x405e40 | info block B |
| e01 | (33,114,30×35) | 1 | 41 | 0x404130 | big center button (shaman) |
| e12 | (6,122,24×18) | 3 toggle | — | 0x405c80 | toggle |
| e19 | (64,114,13×12) | 1 | — | 0x4013e0 | small button (param 105) |
| e02 | (64,126,10×22) | 1 | — | 0x404310 | small button |
| e06-08 | (78,114/126/138, 20×11) | 1 | 34/35/36 | 0x4015f0 | arrow stack (16bpp; e09-11 same at 0x4014b0 for 8bpp) |
| e13-18 | (0/16/32/48/64/80, 153, 15×36) | 1 | — | 0x404ae0 / 0x4047e0 | quick-spell row (icons 0,2,3,6,4,5) |
| e20 | (4,190,92×13) | 1 | — | 0x402b70 | status strip |
| e00 | (97,194,6×287) | 1 | 37 | none | sidebar right-edge trim/divider |

## Tab pages (anchored at panel origin (0,204))

**Spells (panel 3)**: 9 buttons 46×52, 2 columns x=3/49, rows y=3+54k;
commands 2..10; icons 1,4,7,5,6,8,13,15,17; render cb 0x4018a0;
tooltips 768-776; sounds (param) 1028-1036.

**Buildings (panel 2, 8bpp)**: 17 buttons 31×43, columns x=2/34/66, rows
y=8 (partial top row at x=18/50), 52, 96, 140, 184, 228; commands 13..31;
icons 0..16; generic icon-button render cb 0x401d10 (was mislabeled
`UI_RenderBuildingStatsPanel`). 16bpp variant (0x5764a0): 18 buttons,
full top row at x=2/34/66, icons 0..17.

**Units (panel 4)**: 36 cells 15×34, 6 columns x=16k. Rows y=6/47/88/129:
column 0 = cb 0x4044c0 (params 639/641/643/645), columns 1-5 = cb
0x405160 (icons 10-13, 15-18, 30-33, 20-23, 25-28). Rows y=190/231 = cb
0x405870 (icons 0,2,3,6,4,5 / 8,10,11,14,12,13; params 653/647 on col 0).

**Bottom bar (panel 6, origin (99,444))**: strip (0,0,462×32) cb 0x402e90,
strip (462,0,74×32) cb 0x403930, four 20×10 widgets at x=465/501 y=4/18
(cb 0x4039e0, icons 1-4), four 10×10 at x=487/523 (cb 0x403b50).

## Per-frame draw order (UI_RenderGamePanel 0x492390)

1. viewport clip + terrain render
2. cursor
3. `UI_RenderPanelAnimations(1)`
4. tooltip pass (`FUN_0048de60`)
5. **panel manager `FUN_00459ee0`** — sidebar + open tab page elements
6. `UI_RenderPanelAnimations(0)`
7. `UI_RenderResourceDisplay` (0x493350) — two text lines centered in the
   3D viewport at `y = vp_y + 7*(vp_h − char_h)/8`
8. cursor-attached spell/building icon overlay (0x494430, was mislabeled
   `UI_ProcessSpellButtons`; blits the selected icon at cursor y−16)
9. busy-cursor sprite
10. `UI_RenderBuildingInfo` (0x4937f0), objective/status text,
    `UI_RenderInfoPanel` (0x494d90), chat, game timer (top-right)

Element-type render dispatch: 0x4d1df0 (was mislabeled
`Compass_RenderPanel`) is the generic per-type element renderer (7 types).

## Icon sprites

In-game HUD icons come from the **interface sprite bank `data/hfx0-0.dat`**
(`Sprite_Blit(x, y, bank + idx*8)`), NOT from `plspanel.spr` and NOT from
`hspr0-0.dat` (ids there land in people animation frames). Verified by
contact sheet (2026-06): spell glyphs at 0-35 (state*18 + icon), building
line-art at 354+ (normal) / 390+ (highlight) via the building table at
0x5a0ec0, quick-row people figures 666-674, shaman 664, tab icons 676
(hut/buildings), 678 (burst/spells), 680 (people/units) (+1 active),
minimap rock-arch frame 690-697 (50x49 quadrants), gold nine-patch tile
sets 740-766 (tabs), 794-829 (spell/building buttons). Screen tab order is
buildings (x=0), spells (x=32), units (x=64).

## Remaining unknowns (next discovery pass)

- icon-index → HSPR sprite-id arrays used by cbs 0x401d10 / 0x4018a0 /
  0x405160 (static arrays around 0x575490-0x575508)
- minimap border tile lists at 0x5752f8 / 0x575310 (≤512px screens)
- internals of 0x405ec0 (mana) and 0x405e40 (info blocks) — .c exports
  truncated, use .asm
- Ghidra DB renames pending (server down): Panel_Open, Panel_GetSidebarWidth,
  Element_Instantiate, Panel_TickManager (0x459ee0), UI_RenderIconButton
  (0x401d10), UI_ElementRenderDispatch (0x4d1df0), UI_RenderCursorIcon
  (0x494430), UI_LoadLevelSelectPanel (0x419bf0)

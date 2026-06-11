# Level-select screen ("plspanel")

`data\plspanel.spr` and the widget table at `0x57b978` belong to the
**level-select screen**, not the in-game HUD (a previous RE pass confused
the two).

- Loader: `UI_LoadLevelSelectPanel` (0x419bf0, exported as FUN_00419bf0):
  loads `data\plspanel.spr`, anchors the panel bottom-center
  (`x = screen_w/2 − sprite_w/2`, `y = screen_h − sprite_h`), then offsets
  every widget in the 0x57b978 table by the panel origin (−1; type 2 gets
  an extra −1).
- Renderer: `UI_RenderLevelSelectScreen` (0x419e50) — blits plspanel
  sprite 0 as background and draws the widgets as the 25 level buttons.
- Widget table @ 0x57b978: 7 dwords per record `{type, x, y, bbox_x,
  bbox_y, bbox_flags, state}` (fields 3..6 written at runtime), terminated
  by `0x80000000`. 25 records (= 25 campaign levels). `type` selects the
  base sprite: 0→5, 1→3, 2→1 (+1 when the level is completed); sprites
  7/8/9 mark the current level, with anchor offsets from a table at
  0x57bc50.

Widget table values (type, x, y):

```
 0:(0,34,27)  1:(0,54,27)  2:(0,74,27)  3:(0,94,27)  4:(1,114,25)
 5:(2,137,23) 6:(1,168,25) 7:(2,191,23) 8:(1,224,25) 9:(1,249,25)
10:(2,270,23) 11:(2,298,23) 12:(1,329,25) 13:(1,354,25) 14:(1,379,25)
15:(1,408,8) 16:(1,412,19) 17:(1,412,30) 18:(1,408,41) 19:(1,442,12)
20:(1,442,38) 21:(2,466,23) 22:(2,495,23) 23:(2,523,23) 24:(2,551,23)
```

Not yet implemented in the Rust frontend; kept here so the in-game HUD
work (`hud_panel.md`) doesn't re-confuse the two systems.

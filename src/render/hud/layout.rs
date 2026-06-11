//! Faithful in-game HUD layout from popTB.exe's data-driven panel system.
//!
//! See docs/specs/hud_panel.md. Panels and elements are defined in 640×480
//! virtual coordinates (static tables at 0x577886 / 0x575668+) and scaled
//! through 16.16 fixed point exactly like the original:
//!   frac  = (v << 16) / 640        (or / 480 for y; truncating)
//!   abs   = frac(panel) + frac(elem)
//!   x_px  = (abs_x * screen_w) >> 16
//!   y_px  = (abs_y * screen_h + screen_h/2) >> 16
//! Right/bottom edges are converted from the *summed* fractions, so
//! adjacent elements stay seamless (Panel_TickManager, 0x459ee0).

use super::HudTab;

pub const VIRTUAL_W: i32 = 640;
pub const VIRTUAL_H: i32 = 480;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Element type byte from the binary's element defs (+4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Static, // 0
    Button, // 1
    Canvas, // 2 (minimap)
    Toggle, // 3
    Tab,    // 5 (panel opener)
}

/// One element definition (subset of the binary's 0x42-byte record that
/// matters for layout; see hud_panel.md for the full record).
#[derive(Debug, Clone, Copy)]
pub struct ElementDef {
    pub cmd: i32,
    pub kind: ElementKind,
    /// Interactive (hit-test) position, panel-relative 640×480 units.
    pub ix: i16,
    pub iy: i16,
    /// Draw position (usually == interactive; tabs draw 4px higher).
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    pub icon: i32,
    pub flags: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct PanelDef {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub const PANEL_SIDEBAR: PanelDef = PanelDef { id: 1, x: 0, y: 0, w: 100, h: 480 };
/// Panels 2/3/4 (buildings/spells/units pages) share this rect.
pub const PANEL_TAB_PAGE: PanelDef = PanelDef { id: 3, x: 0, y: 204, w: 100, h: 277 };
pub const PANEL_BOTTOM: PanelDef = PanelDef { id: 6, x: 99, y: 444, w: 106, h: 32 };

const fn el(
    cmd: i32,
    kind: ElementKind,
    x: i16,
    y: i16,
    w: i16,
    h: i16,
    icon: i32,
) -> ElementDef {
    ElementDef { cmd, kind, ix: x, iy: y, x, y, w, h, icon, flags: 0 }
}

const fn btn(cmd: i32, x: i16, y: i16, w: i16, h: i16, icon: i32) -> ElementDef {
    el(cmd, ElementKind::Button, x, y, w, h, icon)
}

use ElementKind::{Button, Canvas, Static, Tab, Toggle};

/// Sidebar element list (panel 1 @ 0x575668), binary order.
pub const SIDEBAR_ELEMENTS: [ElementDef; 25] = [
    el(37, Button, 97, 194, 6, 287, 0), // right-edge trim
    el(41, Button, 33, 114, 30, 35, 7), // big center (shaman) button
    el(0, Button, 64, 126, 10, 22, 7),
    // Tabs draw 4px above their interactive rect (draw y=82, hit y=86).
    ElementDef { cmd: 38, kind: Tab, ix: 32, iy: 86, x: 32, y: 82, w: 34, h: 27, icon: 0, flags: 0 },
    ElementDef { cmd: 39, kind: Tab, ix: 0, iy: 86, x: 0, y: 82, w: 34, h: 27, icon: 0, flags: 0 },
    ElementDef { cmd: 40, kind: Tab, ix: 64, iy: 86, x: 64, y: 82, w: 34, h: 27, icon: 0, flags: 0 },
    ElementDef { cmd: 34, kind: Button, ix: 78, iy: 114, x: 78, y: 114, w: 20, h: 11, icon: 1, flags: 0x22 },
    ElementDef { cmd: 35, kind: Button, ix: 78, iy: 126, x: 78, y: 126, w: 20, h: 11, icon: 2, flags: 0x22 },
    ElementDef { cmd: 36, kind: Button, ix: 78, iy: 138, x: 78, y: 138, w: 20, h: 11, icon: 3, flags: 0x22 },
    ElementDef { cmd: 34, kind: Button, ix: 78, iy: 114, x: 78, y: 114, w: 20, h: 11, icon: 1, flags: 0x21 },
    ElementDef { cmd: 35, kind: Button, ix: 78, iy: 126, x: 78, y: 126, w: 20, h: 11, icon: 2, flags: 0x21 },
    ElementDef { cmd: 36, kind: Button, ix: 78, iy: 138, x: 78, y: 138, w: 20, h: 11, icon: 3, flags: 0x21 },
    el(0, Toggle, 6, 122, 24, 18, 0),
    // Quick-spell row, 6 cells at y=153.
    ElementDef { cmd: 0, kind: Button, ix: 0, iy: 153, x: 0, y: 153, w: 15, h: 36, icon: 0, flags: 0x40 },
    el(0, Button, 16, 153, 15, 36, 2),
    el(0, Button, 32, 153, 15, 36, 3),
    el(0, Button, 48, 153, 15, 36, 6),
    el(0, Button, 64, 153, 15, 36, 4),
    el(0, Button, 80, 153, 15, 36, 5),
    el(0, Button, 64, 114, 13, 12, 0),
    el(0, Button, 4, 190, 92, 13, 0),   // status strip
    el(0, Button, 0, 149, 100, 64, 0),  // info block B
    el(0, Button, 0, 110, 100, 64, 0),  // info block A
    el(0, Button, 0, 90, 100, 32, 700), // mana display (icon = string id)
    el(0, Canvas, 0, 0, 100, 96, 0),    // minimap
];

/// The three panel-opener tabs in binary order (buildings, spells, units):
/// the 2px overlaps resolve first-match like the original element walk.
pub const SIDEBAR_TABS: [(HudTab, &ElementDef); 3] = [
    (HudTab::Buildings, &SIDEBAR_ELEMENTS[3]),
    (HudTab::Spells, &SIDEBAR_ELEMENTS[4]),
    (HudTab::Units, &SIDEBAR_ELEMENTS[5]),
];

pub fn minimap_element() -> ElementDef {
    SIDEBAR_ELEMENTS[24]
}

/// Spells page (panel 3 @ 0x576c20): 9 buttons, 2 columns.
pub const SPELLS_PAGE: [ElementDef; 9] = [
    btn(2, 3, 3, 46, 52, 1),
    btn(3, 49, 3, 46, 52, 4),
    btn(6, 3, 57, 46, 52, 7),
    btn(4, 49, 57, 46, 52, 5),
    btn(5, 3, 111, 46, 52, 6),
    btn(7, 49, 111, 46, 52, 8),
    btn(8, 3, 165, 46, 52, 13),
    btn(9, 49, 165, 46, 52, 15),
    btn(10, 3, 219, 46, 52, 17),
];

/// Buildings page (panel 2, 16bpp variant @ 0x5764a0): 18 buttons, 3 columns.
pub const BUILDINGS_PAGE: [ElementDef; 18] = [
    btn(31, 66, 8, 31, 43, 17),
    btn(28, 34, 8, 31, 43, 16),
    btn(29, 2, 8, 31, 43, 15),
    btn(27, 66, 52, 31, 43, 14),
    btn(26, 34, 52, 31, 43, 13),
    btn(25, 2, 52, 31, 43, 12),
    btn(24, 66, 96, 31, 43, 11),
    btn(23, 34, 96, 31, 43, 10),
    btn(22, 2, 96, 31, 43, 9),
    btn(21, 66, 140, 31, 43, 8),
    btn(20, 34, 140, 31, 43, 7),
    btn(19, 2, 140, 31, 43, 6),
    btn(18, 66, 184, 31, 43, 5),
    btn(17, 34, 184, 31, 43, 4),
    btn(16, 2, 184, 31, 43, 3),
    btn(15, 66, 228, 31, 43, 2),
    btn(14, 34, 228, 31, 43, 1),
    btn(13, 2, 228, 31, 43, 0),
];

/// Units page (panel 4 @ 0x576eb8): 36 cells, 6 columns of 15×34.
pub const UNITS_PAGE: [ElementDef; 36] = [
    btn(0, 0, 6, 15, 34, 0),
    btn(0, 0, 47, 15, 34, 1),
    btn(0, 0, 88, 15, 34, 2),
    btn(0, 0, 129, 15, 34, 3),
    btn(0, 16, 6, 15, 34, 10),
    btn(0, 16, 47, 15, 34, 11),
    btn(0, 16, 88, 15, 34, 12),
    btn(0, 16, 129, 15, 34, 13),
    btn(0, 32, 6, 15, 34, 15),
    btn(0, 32, 47, 15, 34, 16),
    btn(0, 32, 88, 15, 34, 17),
    btn(0, 32, 129, 15, 34, 18),
    btn(0, 48, 6, 15, 34, 30),
    btn(0, 48, 47, 15, 34, 31),
    btn(0, 48, 88, 15, 34, 32),
    btn(0, 48, 129, 15, 34, 33),
    btn(0, 64, 6, 15, 34, 20),
    btn(0, 64, 47, 15, 34, 21),
    btn(0, 64, 88, 15, 34, 22),
    btn(0, 64, 129, 15, 34, 23),
    btn(0, 80, 6, 15, 34, 25),
    btn(0, 80, 47, 15, 34, 26),
    btn(0, 80, 88, 15, 34, 27),
    btn(0, 80, 129, 15, 34, 28),
    btn(0, 0, 190, 15, 34, 0),
    btn(0, 16, 190, 15, 34, 2),
    btn(0, 32, 190, 15, 34, 3),
    btn(0, 48, 190, 15, 34, 6),
    btn(0, 64, 190, 15, 34, 4),
    btn(0, 80, 190, 15, 34, 5),
    btn(0, 0, 231, 15, 34, 8),
    btn(0, 16, 231, 15, 34, 10),
    btn(0, 32, 231, 15, 34, 11),
    btn(0, 48, 231, 15, 34, 14),
    btn(0, 64, 231, 15, 34, 12),
    btn(0, 80, 231, 15, 34, 13),
];

/// Bottom bar (panel 6 @ 0x575d20, origin (99,444)).
pub const BOTTOM_BAR: [ElementDef; 10] = [
    btn(0, 465, 4, 20, 10, 1),
    btn(0, 465, 18, 20, 10, 2),
    btn(0, 501, 4, 20, 10, 3),
    btn(0, 501, 18, 20, 10, 4),
    btn(0, 487, 4, 10, 10, 1),
    btn(0, 487, 18, 10, 10, 2),
    btn(0, 523, 4, 10, 10, 3),
    btn(0, 523, 18, 10, 10, 4),
    el(0, Static, 0, 0, 462, 32, 0),
    el(0, Static, 462, 0, 74, 32, 0),
];

/// `(v << 16) / 640`, truncating — Panel_Open / Element_Instantiate.
pub fn frac_x(v: i32) -> i32 {
    (v << 16) / VIRTUAL_W
}

/// `(v << 16) / 480`, truncating.
pub fn frac_y(v: i32) -> i32 {
    (v << 16) / VIRTUAL_H
}

/// `(frac * screen_dim) >> 16`, truncating — Panel_TickManager.
pub fn scale_frac(frac: i32, screen_dim: i32) -> i32 {
    ((frac as i64 * screen_dim as i64) >> 16) as i32
}

/// Sidebar pixel width — Panel_GetSidebarWidth (0x45ae60).
pub fn sidebar_width(screen_w: i32) -> i32 {
    scale_frac(frac_x(PANEL_SIDEBAR.w), screen_w)
}

/// Absolute pixel rect of an element, replicating the original fraction
/// pipeline: per-coordinate fracs are summed, right/bottom edges converted
/// from summed fractions (Panel_TickManager 0x459ee0; y adds screen_h/2
/// before the shift like the binary).
pub fn element_rect(panel: &PanelDef, e: &ElementDef, screen_w: i32, screen_h: i32) -> Rect {
    rect_from_virtual(panel, e.x as i32, e.y as i32, e.w as i32, e.h as i32, screen_w, screen_h)
}

/// Like `element_rect` but for the interactive (hit-test) position.
pub fn element_hit_rect(panel: &PanelDef, e: &ElementDef, screen_w: i32, screen_h: i32) -> Rect {
    rect_from_virtual(panel, e.ix as i32, e.iy as i32, e.w as i32, e.h as i32, screen_w, screen_h)
}

fn rect_from_virtual(
    panel: &PanelDef,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    screen_w: i32,
    screen_h: i32,
) -> Rect {
    let fx = frac_x(panel.x) + frac_x(x);
    let fy = frac_y(panel.y) + frac_y(y);
    let fw = frac_x(w);
    let fh = frac_y(h);
    let x0 = scale_frac(fx, screen_w);
    let x1 = scale_frac(fx + fw, screen_w);
    let y0 = scale_y_frac(fy, screen_h);
    let y1 = scale_y_frac(fy + fh, screen_h);
    Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
}

/// Y conversion adds screen_h/2 before the shift (Panel_TickManager).
fn scale_y_frac(frac: i32, screen_h: i32) -> i32 {
    ((frac as i64 * screen_h as i64 + (screen_h / 2) as i64) >> 16) as i32
}

/// 3D viewport rect — Render_SetViewportInset (0x422090):
/// x = sidebar_w + border*8, y = border*8,
/// w = screen_w − sidebar_w − border*16, h = screen_h − border*16.
pub fn viewport_rect(sidebar_open: bool, border: i32, screen_w: i32, screen_h: i32) -> Rect {
    let sw = if sidebar_open { sidebar_width(screen_w) } else { 0 };
    Rect {
        x: sw + border * 8,
        y: border * 8,
        w: screen_w - sw - border * 16,
        h: screen_h - border * 16,
    }
}

/// Hit-test the sidebar tabs in binary table order (first match wins,
/// resolving the 2px overlaps like the original element walk).
pub fn tab_hit(mouse_x: i32, mouse_y: i32, screen_w: i32, screen_h: i32) -> Option<HudTab> {
    for (tab, e) in SIDEBAR_TABS.iter() {
        let r = element_hit_rect(&PANEL_SIDEBAR, e, screen_w, screen_h);
        if mouse_x >= r.x && mouse_x < r.x + r.w && mouse_y >= r.y && mouse_y < r.y + r.h {
            return Some(*tab);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fracs_match_original_truncation() {
        // (v << 16) / 640, truncating: 100 is exact, 49 is not.
        assert_eq!(frac_x(100), 10240);
        assert_eq!(frac_x(49), 5017); // 5017.6 truncated
        assert_eq!(frac_y(204), 27852); // 27852.8 truncated
    }

    #[test]
    fn sidebar_width_scales_like_panel_getsidebarwidth() {
        assert_eq!(sidebar_width(640), 100);
        assert_eq!(sidebar_width(800), 125);
        assert_eq!(sidebar_width(1024), 160);
        assert_eq!(sidebar_width(1280), 200);
    }

    #[test]
    fn viewport_rect_matches_render_setviewportinset() {
        // FUN_00422090: x = sidebar_w + border*8, w = sw - sidebar_w - border*16.
        assert_eq!(
            viewport_rect(true, 0, 640, 480),
            Rect { x: 100, y: 0, w: 540, h: 480 }
        );
        assert_eq!(
            viewport_rect(true, 1, 640, 480),
            Rect { x: 108, y: 8, w: 524, h: 464 }
        );
    }

    #[test]
    fn minimap_rect_is_sidebar_canvas() {
        // Sidebar element e24: (0,0,100,96).
        let mm = element_rect(&PANEL_SIDEBAR, &minimap_element(), 640, 480);
        assert_eq!(mm, Rect { x: 0, y: 0, w: 100, h: 96 });
    }

    #[test]
    fn spells_button_rect_replicates_double_truncation() {
        // Spells page e01: draw (49,3) 46x52 in panel (0,204).
        // frac(49)=5017 → x0 = 5017*640>>16 = 48; fy = 27852+409 = 28261 →
        // y0 = (28261*480+240)>>16 = 206 (the original's own truncating
        // math loses a pixel at native res for values not divisible by 5).
        let e = SPELLS_PAGE[1];
        assert_eq!(e.cmd, 3);
        let r = element_rect(&PANEL_TAB_PAGE, &e, 640, 480);
        assert_eq!(r, Rect { x: 48, y: 206, w: 46, h: 52 });
    }

    #[test]
    fn truncating_scale_at_800x600() {
        // x=34 at 800 wide: frac(34)=3481 → 3481*800>>16 = 42 (not 43).
        let e = SIDEBAR_TABS[0]; // buildings tab at x=32? no: table order
        let _ = e;
        assert_eq!(scale_frac(frac_x(34), 800), 42);
    }

    #[test]
    fn element_tables_match_binary_counts() {
        assert_eq!(SIDEBAR_ELEMENTS.len(), 25);
        assert_eq!(SPELLS_PAGE.len(), 9);
        assert_eq!(BUILDINGS_PAGE.len(), 18); // 16bpp variant
        assert_eq!(UNITS_PAGE.len(), 36);
        assert_eq!(BOTTOM_BAR.len(), 10);
        assert_eq!(SIDEBAR_TABS.len(), 3);
    }

    #[test]
    fn tab_hit_test_uses_interactive_rects() {
        // Tabs (interactive): spells (0,86,34x27), buildings (32,86,34x27),
        // units (64,86,34x27); binary table order buildings→spells→units,
        // so the 2px overlap at x=32..34 resolves to buildings.
        assert_eq!(tab_hit(16, 90, 640, 480), Some(HudTab::Spells));
        assert_eq!(tab_hit(48, 90, 640, 480), Some(HudTab::Buildings));
        assert_eq!(tab_hit(80, 90, 640, 480), Some(HudTab::Units));
        assert_eq!(tab_hit(33, 90, 640, 480), Some(HudTab::Buildings));
        assert_eq!(tab_hit(16, 70, 640, 480), None); // above tab row
        assert_eq!(tab_hit(120, 90, 640, 480), None); // in 3D viewport
        // Scaled: same panel positions at 1280x960.
        assert_eq!(tab_hit(32, 180, 1280, 960), Some(HudTab::Spells));
    }

    #[test]
    fn spells_page_grid_positions() {
        // 2 columns x=3/49, rows y=3+54k — all 9 cells.
        for (i, e) in SPELLS_PAGE.iter().enumerate() {
            let col = [3, 49][i % 2];
            let row = 3 + 54 * (i as i16 / 2);
            assert_eq!((e.x, e.y), (col, row), "spell cell {i}");
            assert_eq!((e.w, e.h), (46, 52));
        }
    }

    #[test]
    fn units_page_grid_positions() {
        // 6 columns x=16k; rows 6/47/88/129 then 190/231.
        for e in UNITS_PAGE.iter() {
            assert_eq!(e.x % 16, 0);
            assert!([6, 47, 88, 129, 190, 231].contains(&e.y), "y={}", e.y);
            assert_eq!((e.w, e.h), (15, 34));
        }
    }
}

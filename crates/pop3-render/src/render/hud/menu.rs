/// Menu overlay rendering — generates HUD vertices for menu screens.

use crate::engine::menu::MenuRenderData;
use super::{HudVertex, HudLayout};

/// Generate HUD vertices for the current menu screen.
/// Returns vertices to be appended to the HUD vertex buffer.
pub fn build_menu_vertices(
    menu: &MenuRenderData,
    layout: &HudLayout,
    verts: &mut Vec<HudVertex>,
) {
    // Full-screen dark overlay
    let overlay_color = [0.0, 0.0, 0.05, 0.85];
    push_quad(verts, 0.0, 0.0, layout.screen_w, layout.screen_h, overlay_color);

    let center_x = layout.screen_w / 2.0;
    let _start_y = layout.screen_h * 0.15;

    // Menu items rendered as text lines with cursor highlight
    let line_height = layout.screen_h * 0.05;
    let item_start_y = layout.screen_h * 0.15 + line_height * 2.0;

    for (i, item) in menu.items.iter().enumerate() {
        let y = item_start_y + (i as f32) * line_height;
        if i == menu.cursor && item.enabled {
            // Highlight bar behind selected item
            let bar_w = layout.screen_w * 0.4;
            let bar_x = center_x - bar_w / 2.0;
            push_quad(verts, bar_x, y - 2.0, bar_w, line_height, [0.2, 0.3, 0.6, 0.7]);
        }
    }
}

fn push_quad(verts: &mut Vec<HudVertex>, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    let uv = [0.0, 0.0]; // solid color, no texture
    verts.push(HudVertex { position: [x, y], uv, color });
    verts.push(HudVertex { position: [x + w, y], uv, color });
    verts.push(HudVertex { position: [x + w, y + h], uv, color });
    verts.push(HudVertex { position: [x, y], uv, color });
    verts.push(HudVertex { position: [x + w, y + h], uv, color });
    verts.push(HudVertex { position: [x, y + h], uv, color });
}

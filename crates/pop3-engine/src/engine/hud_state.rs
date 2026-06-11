// HUD data contract: game logic -> HUD renderer.
// Pure data, no GPU types. Moved out of render/hud for crate layering.
use crate::engine::menu::MenuRenderData;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HudTab {
    Buildings,
    Spells,
    Units,
}


// ---------------------------------------------------------------------------
// Data contract: game logic → HUD
// ---------------------------------------------------------------------------

/// Data the game logic provides to the HUD each frame.
/// The HUD renders whatever is in here — no game logic knowledge.
pub struct HudState {
    pub active_tab: HudTab,
    pub minimap: MinimapData,
    pub panel_entries: Vec<PanelEntry>,
    pub tribe_populations: Vec<TribePopulation>,
    pub level_num: u32,
    pub frame_count: u64,
    // Phase 3: player resource and spell cooldown data
    pub player_mana: u32,
    pub player_max_mana: u32,
    pub player_population: u32,
    pub player_max_population: u16,
    pub spell_cooldowns: Vec<SpellCooldown>,
    pub spell_charges: [u8; 16],
    pub camera_viewport: MinimapViewport,
    pub selected_info: Option<SelectedEntityInfo>,
    pub health_bars: Vec<HealthBarEntry>,
    pub menu_render_data: Option<MenuRenderData>,
}

pub struct MinimapData {
    pub heights: [[u16; 128]; 128],
    pub dots: Vec<MinimapDot>,
}

pub struct MinimapDot {
    pub cell_x: u8,
    pub cell_y: u8,
    pub tribe_index: u8,
}

pub struct PanelEntry {
    pub label: String,
    pub color: [f32; 4],
}

pub struct TribePopulation {
    pub tribe_index: u8,
    pub count: u32,
    pub color: [f32; 4],
}

/// Minimap viewport rectangle data for camera position overlay.
pub struct MinimapViewport {
    pub cam_cell_x: f32,       // camera center in cell coords (0-127)
    pub cam_cell_y: f32,
    pub view_width_cells: f32,  // visible area width in cells
    pub view_height_cells: f32,
}

/// Selected entity info for sidebar detail panel.
pub struct SelectedEntityInfo {
    pub name: String,
    pub health: u16,
    pub max_health: u16,
    pub subtype: u8,
    pub tribe_index: u8,
    pub extra_lines: Vec<String>,
}

/// Health bar entry for world-projected health bars in the HUD overlay.
pub struct HealthBarEntry {
    pub screen_x: f32,         // screen-space center X
    pub screen_y: f32,         // screen-space top Y (above entity)
    pub health_fraction: f32,  // 0.0-1.0
    pub bar_type: HealthBarType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HealthBarType {
    Unit,
    Building,
}

/// Spell cooldown state for HUD rendering.
/// Phase 4 will populate from SpellSystem cooldown timers.
pub struct SpellCooldown {
    pub spell_index: u8,        // 0-15 matching spell panel order
    pub cooldown_remaining: u32, // ticks remaining (0 = ready)
    pub cooldown_total: u32,     // total cooldown duration
}


/// Tribe colors for HUD text overlay (RGBA, 0.0-1.0).
pub const HUD_TRIBE_COLORS: [[f32; 4]; 4] = [
    [0.3, 0.5, 1.0, 0.9],  // Blue
    [1.0, 0.3, 0.3, 0.9],  // Red
    [1.0, 1.0, 0.3, 0.9],  // Yellow
    [0.3, 1.0, 0.3, 0.9],  // Green
];

/// Map unit subtype id to display name.
pub fn unit_subtype_name(subtype: u8) -> &'static str {
    match subtype {
        1 => "Wild",
        2 => "Brave",
        3 => "Warrior",
        4 => "Preacher",
        5 => "Spy",
        6 => "Super Warrior",
        7 => "Shaman",
        _ => "Unknown",
    }
}


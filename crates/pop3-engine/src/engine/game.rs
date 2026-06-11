//! GameEngine: all game-logic state, no GPU types. Produces FrameState
//! for the renderer. Extracted from render/app.rs during the workspace
//! reboot; fields are temporarily `pub` because the render-side App still
//! accesses them directly (boundary tightens during the Phase 3 rewrite).
use std::path::PathBuf;

use cgmath::{Matrix4, Point2, Point3, Vector3, Vector4};

use crate::data::objects::{Object3D, Shape, ShapeFootprints};
use crate::engine::state::constants::*;
use crate::engine::camera::{Camera, Screen, MVP, screen_to_scene_zoom};
use crate::engine::terrain::landscape_mesh::{LandscapeMesh, LandscapeUniformData, LANDSCAPE_SCALE, LANDSCAPE_OFFSET};
use crate::engine::hud_state::*;
use crate::engine::level_objects::LevelObject;
use crate::engine::units::coords::{
    triangle_to_cell, project_to_screen, ScreenRect,
};
use crate::engine::picking::intersect_iter;
use crate::engine::units::{DragState, Unit, UnitCoordinator};
use crate::engine::ai::flyby::FlybyState;
use crate::engine::campaign::CampaignState;
use crate::engine::effects::EffectPool;
use crate::engine::menu::{MenuAction, MenuScreen, MenuSystem};
use crate::engine::state::state_machine::GameState;
use crate::engine::state::tick::{GameWorld, StdTimeSource};
use crate::engine::{FrameState, GameCommand};

pub type LandscapeMeshS = LandscapeMesh<128>;

pub struct AppConfig {
    pub base: Option<PathBuf>,
    pub level: Option<u8>,
    pub landtype: Option<String>,
    pub cpu: bool,
    pub cpu_full: bool,
    pub debug: bool,
    pub light: Option<(i16, i16)>,
    pub script: Option<PathBuf>,
}


/// All game-logic state — no GPU types. Produces FrameState for the renderer.
pub struct GameEngine {
    pub landscape_mesh: LandscapeMeshS,
    pub camera: Camera,
    pub screen: Screen,
    pub curvature_scale: f32,
    pub curvature_enabled: bool,
    pub zoom: f32,
    pub level_num: u8,
    pub sunlight: Vector4<f32>,
    pub show_objects: bool,
    pub show_shadows: bool,
    pub show_lighting: bool,
    pub show_markers: bool,
    pub sprite_z_offset: f32,
    pub sprite_scale: f32,
    pub hud_tab: HudTab,
    pub hud_visible: bool,
    pub compass_visible: bool,
    pub walkability_visible: bool,
    pub hud_panel_sprite_count: usize,

    // Game simulation
    pub unit_coordinator: UnitCoordinator,
    pub game_world: GameWorld,
    pub game_time: StdTimeSource,
    pub effect_pool: EffectPool,

    // Menu and campaign
    pub menu_system: MenuSystem,
    pub campaign_state: CampaignState,

    // AI system (None if mlua initialization fails)
    pub ai_system: Option<crate::engine::ai::AiSystem>,

    // Flyby camera (active during intro cinematics)
    pub flyby: Option<FlybyState>,
    /// Sub-cell camera focus offset in grid cells, set by the flyby for
    /// smooth panning (the landscape shift only has whole-cell resolution).
    pub focus_frac: (f32, f32),

    // Level data
    pub level_objects: Vec<LevelObject>,
    pub building_objects: Vec<Option<Object3D>>, // from OBJS bank 0 (building models)
    pub scenery_objects: Vec<Option<Object3D>>,  // from level-specific OBJS bank (scenery models)
    pub shapes: Vec<Shape>,
    pub shape_footprints: ShapeFootprints,
    /// PLAYERSAVEINFO start positions per tribe (from `.dat`). Used as
    /// fallback camera centring when no shaman has spawned yet.
    pub player_starts: [crate::data::units::PlayerSaveInfo; 4],
    /// Decoded markers from `.hdr` (`pop.h:1028`), 256 slots indexed by
    /// PopScript marker id. AI bridge consumer.
    pub level_markers: [crate::data::level::MarkerCell; 256],

    // Water animation
    pub wat_offset: i32,
    pub wat_interval: u32,
    pub frame_count: u32,

    // Config (read-only after init)
    pub config: AppConfig,
}

impl GameEngine {
    pub fn reset_camera(&mut self) {
        self.camera.angle_x = -55;
        self.camera.angle_y = 0;
        self.camera.angle_z = 0;
        self.camera.angle_z_frac = 0.0;
        self.zoom = 1.0;
        self.focus_frac = (0.0, 0.0);
    }

    /// Camera focus point in world space: terrain center plus the sub-cell
    /// flyby offset.
    pub fn world_focus(&self) -> Vector3<f32> {
        let center = self.world_center();
        let s = LANDSCAPE_SCALE * self.landscape_mesh.step();
        Vector3::new(
            center + self.focus_frac.0 * s,
            center + self.focus_frac.1 * s,
            0.0,
        )
    }

    pub fn build_landscape_params(&self) -> LandscapeUniformData {
        let shift = self.landscape_mesh.get_shift_vector();
        LandscapeUniformData {
            level_shift: [shift.x, shift.y, shift.z, shift.w],
            height_scale: self.landscape_mesh.height_scale(),
            step: self.landscape_mesh.step(),
            width: self.landscape_mesh.width() as i32,
            _pad_width: 0,
            sunlight: [
                self.sunlight.x,
                self.sunlight.y,
                self.sunlight.z,
                self.sunlight.w,
            ],
            wat_offset: self.wat_offset,
            curvature_scale: if self.curvature_enabled {
                self.curvature_scale
            } else {
                0.0
            },
            camera_focus: {
                let center =
                    (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
                let step = self.landscape_mesh.step();
                [
                    center + self.focus_frac.0 * step,
                    center + self.focus_frac.1 * step,
                ]
            },
            viewport_radius: {
                let center =
                    (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
                center * 0.9
            },
            _pad2: [0.0; 3],
        }
    }

    /// World-space center of the terrain (accounting for model transform).
    pub fn world_center(&self) -> f32 {
        let center_model =
            (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        LANDSCAPE_SCALE * center_model + LANDSCAPE_OFFSET
    }

    pub fn camera_focus_vertex(&self) -> f32 {
        let center_model =
            (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        center_model / self.landscape_mesh.step()
    }

    pub fn camera_min_z(&self) -> f32 {
        let center = self.world_center();
        let az = (self.camera.angle_z as f32).to_radians();
        let ax = (self.camera.angle_x as f32).to_radians();
        let radius = 1.5 / self.zoom;
        let eye_x = center + radius * ax.cos() * az.sin();
        let eye_y = center + radius * ax.cos() * az.cos();
        // Convert world-space eye position back to grid coords for height lookup
        let model_x = (eye_x - LANDSCAPE_OFFSET) / LANDSCAPE_SCALE;
        let model_y = (eye_y - LANDSCAPE_OFFSET) / LANDSCAPE_SCALE;
        let step = self.landscape_mesh.step();
        let n = self.landscape_mesh.width();
        let gx = (model_x / step).clamp(0.0, (n - 1) as f32) as usize;
        let gy = (model_y / step).clamp(0.0, (n - 1) as f32) as usize;
        let shift = self.landscape_mesh.get_shift_vector();
        let sx = (gx + shift.x as usize) % n;
        let sy = (gy + shift.y as usize) % n;
        self.landscape_mesh.height_at(sx, sy) as f32 * self.landscape_mesh.height_scale() + 0.05
    }

    pub fn screen_to_cell(&self, mouse_pos: &Point2<f32>) -> Option<(f32, f32)> {
        let focus = self.world_focus();
        let min_z = self.camera_min_z();
        let (v1, v2) = screen_to_scene_zoom(
            &self.screen,
            &self.camera,
            mouse_pos,
            self.zoom,
            focus,
            min_z,
        );
        let mvp_transform =
            Matrix4::from_translation(Vector3::new(LANDSCAPE_OFFSET, LANDSCAPE_OFFSET, 0.0))
                * Matrix4::from_scale(LANDSCAPE_SCALE);
        let iter = self.landscape_mesh.iter();
        match intersect_iter(iter, &mvp_transform, v1, v2) {
            Some((triangle_id, _)) => {
                let shift = self.landscape_mesh.get_shift_vector();
                Some(triangle_to_cell(
                    triangle_id,
                    self.landscape_mesh.width(),
                    shift.x as usize,
                    shift.y as usize,
                ))
            }
            None => None,
        }
    }

    pub fn unit_pvm(&self) -> Matrix4<f32> {
        let focus = self.world_focus();
        let min_z = self.camera_min_z();
        let mvp = MVP::with_zoom(&self.screen, &self.camera, self.zoom, focus, min_z);
        let model_transform =
            Matrix4::from_translation(Vector3::new(LANDSCAPE_OFFSET, LANDSCAPE_OFFSET, 0.0))
                * Matrix4::from_scale(LANDSCAPE_SCALE);
        mvp.projection * mvp.view * model_transform
    }

    pub fn unit_screen_pos(&self, unit: &Unit, pvm: &Matrix4<f32>) -> Option<(f32, f32)> {
        let step = self.landscape_mesh.step();
        let w = self.landscape_mesh.width() as f32;
        let shift = self.landscape_mesh.get_shift_vector();
        let height_scale = self.landscape_mesh.height_scale();
        let center = (w - 1.0) * step / 2.0;
        let cs = if self.curvature_enabled {
            self.curvature_scale
        } else {
            0.0
        };
        let vis_x = ((unit.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((unit.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;
        let ix = (unit.cell_x as usize).min(127);
        let iy = (unit.cell_y as usize).min(127);
        let gz = self.landscape_mesh.height_at(ix, iy) as f32 * height_scale;
        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * cs;
        let z_base = gz - curvature_offset;
        project_to_screen(
            [gx, gy, z_base],
            pvm,
            self.screen.width as f32,
            self.screen.height as f32,
        )
    }

    /// Compute the billboard's screen-space AABB for a unit.
    /// Uses the same billboard geometry as `build_unit_markers`.
    pub fn unit_screen_rect(
        &self,
        unit: &Unit,
        pvm: &Matrix4<f32>,
        right: &Vector3<f32>,
        up: &Vector3<f32>,
    ) -> Option<ScreenRect> {
        let step = self.landscape_mesh.step();
        let w = self.landscape_mesh.width() as f32;
        let shift = self.landscape_mesh.get_shift_vector();
        let height_scale = self.landscape_mesh.height_scale();
        let center = (w - 1.0) * step / 2.0;
        let cs = if self.curvature_enabled {
            self.curvature_scale
        } else {
            0.0
        };
        let vis_x = ((unit.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((unit.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;
        let ix = (unit.cell_x as usize).min(127);
        let iy = (unit.cell_y as usize).min(127);
        let gz = self.landscape_mesh.height_at(ix, iy) as f32 * height_scale;
        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * cs;
        let z_base = gz - curvature_offset;

        let half_w = step * 0.15;
        let sprite_h = step * 0.4;
        let base = Vector3::new(gx, gy, z_base);
        let bl = base - right * half_w;
        let br = base + right * half_w;
        let tl = bl + up * sprite_h;
        let tr = br + up * sprite_h;

        let sw = self.screen.width as f32;
        let sh = self.screen.height as f32;
        let s_bl = project_to_screen([bl.x, bl.y, bl.z], pvm, sw, sh)?;
        let s_br = project_to_screen([br.x, br.y, br.z], pvm, sw, sh)?;
        let s_tl = project_to_screen([tl.x, tl.y, tl.z], pvm, sw, sh)?;
        let s_tr = project_to_screen([tr.x, tr.y, tr.z], pvm, sw, sh)?;

        let min_x = s_bl.0.min(s_br.0).min(s_tl.0).min(s_tr.0);
        let max_x = s_bl.0.max(s_br.0).max(s_tl.0).max(s_tr.0);
        let min_y = s_bl.1.min(s_br.1).min(s_tl.1).min(s_tr.1);
        let max_y = s_bl.1.max(s_br.1).max(s_tl.1).max(s_tr.1);

        Some(ScreenRect {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    /// Compute the view matrix right/up vectors for billboard orientation.
    pub fn billboard_axes(&self) -> (Vector3<f32>, Vector3<f32>) {
        let center = self.world_center();
        let az = (self.camera.angle_z as f32).to_radians();
        let ax = (self.camera.angle_x as f32).to_radians();
        let eye = Point3::new(
            center + ax.cos() * az.sin(),
            center + ax.cos() * az.cos(),
            -ax.sin(),
        );
        let target = Point3::new(center, center, 0.0);
        let view = Matrix4::look_at_rh(eye, target, Vector3::new(0.0, 0.0, 1.0));
        let right = Vector3::new(view.x.x, view.y.x, view.z.x);
        let up = Vector3::new(view.x.y, view.y.y, view.z.y);
        (right, up)
    }

    pub fn find_unit_at_screen_pos(&self, mouse: &Point2<f32>) -> Option<usize> {
        let pvm = self.unit_pvm();
        let (right, up) = self.billboard_axes();
        let mut best: Option<(usize, f32)> = None;
        for unit in self.unit_coordinator.units() {
            if let Some(rect) = self.unit_screen_rect(unit, &pvm, &right, &up) {
                if rect.contains(mouse.x, mouse.y) {
                    let (cx, cy) = rect.center();
                    let dist_sq = (cx - mouse.x).powi(2) + (cy - mouse.y).powi(2);
                    if best.is_none() || dist_sq < best.unwrap().1 {
                        best = Some((unit.id, dist_sq));
                    }
                }
            }
        }
        best.map(|(id, _)| id)
    }

    pub fn units_in_screen_rect(&self, corner_a: Point2<f32>, corner_b: Point2<f32>) -> Vec<usize> {
        let drag_rect = ScreenRect {
            min_x: corner_a.x.min(corner_b.x),
            max_x: corner_a.x.max(corner_b.x),
            min_y: corner_a.y.min(corner_b.y),
            max_y: corner_a.y.max(corner_b.y),
        };
        let pvm = self.unit_pvm();
        let (right, up) = self.billboard_axes();
        let mut ids = Vec::new();
        for unit in self.unit_coordinator.units() {
            if let Some(rect) = self.unit_screen_rect(unit, &pvm, &right, &up) {
                if rect.overlaps(&drag_rect) {
                    ids.push(unit.id);
                }
            }
        }
        ids
    }

    pub fn handle_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::None => {}
            MenuAction::NavigateTo(screen) => {
                self.menu_system.navigate_to(screen);
            }
            MenuAction::StartLevel(level) => {
                // StatsScreen returns 0 for Continue/Retry; resolve here
                let actual_level = if level == 0 {
                    // From StatsScreen: cursor 0 = Continue (next), cursor 1 = Retry (same)
                    if self.menu_system.cursor == 0 {
                        // Continue: advance and start next level
                        self.campaign_state.advance_level();
                        self.campaign_state.current_level
                    } else {
                        // Retry: restart current level
                        self.campaign_state.retry_level();
                        self.campaign_state.current_level
                    }
                } else {
                    self.campaign_state.set_level(level);
                    level
                };
                self.level_num = actual_level as u8;
                self.game_world.state = GameState::InGame;
                self.game_world.game_tick = 0;
                self.game_world.flags = crate::engine::state::flags::GameFlags::new();
            }
            MenuAction::LoadSave(filename) => {
                let save_dir = dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("pop3")
                    .join("saves");
                let path = save_dir.join(format!("{}.pop3save", filename));
                match crate::engine::save::load_game(&path) {
                    Ok(save) => {
                        self.game_world.game_tick = save.game_tick;
                        self.game_world.tick_counter = save.tick_counter;
                        self.game_world.game_speed = save.game_speed;
                        self.game_world.player_tribe = save.player_tribe;
                        self.game_world.ai_update_mult = save.ai_update_mult;
                        self.game_world.flags = save.flags;
                        self.game_world.rng = save.rng;
                        self.game_world.tribes = save.tribes;
                        self.campaign_state.set_level(save.level_num);
                        self.level_num = save.level_num as u8;
                        self.game_world.state = GameState::InGame;
                        log::info!("Loaded save: {}", filename);
                    }
                    Err(e) => log::error!("Failed to load save {}: {}", filename, e),
                }
            }
            MenuAction::Back => {
                self.menu_system.back();
            }
            MenuAction::Quit => {
                // Will be handled by the event loop checking game_world.state
                // or a quit flag
            }
        }
    }

    pub fn build_hud_state(&self) -> HudState {
        let dots: Vec<MinimapDot> = self
            .unit_coordinator
            .units()
            .iter()
            .filter(|u| u.alive)
            .map(|u| MinimapDot {
                cell_x: (u.cell_x as u8).min(127),
                cell_y: (u.cell_y as u8).min(127),
                tribe_index: u.tribe_index,
            })
            .collect();
        let minimap = MinimapData {
            heights: *self.landscape_mesh.heights(),
            dots,
        };
        let panel_entries = match self.hud_tab {
            HudTab::Spells => [
                "Burn",
                "Blast",
                "Lightning",
                "Whirlwind",
                "Plague",
                "Invisibility",
                "Firestorm",
                "Hypnotize",
                "Ghost Army",
                "Erosion",
                "Swamp",
                "Land Bridge",
                "Angel/Death",
                "Earthquake",
                "Flatten",
                "Volcano",
            ]
            .iter()
            .map(|name| PanelEntry {
                label: name.to_string(),
                color: [0.8, 0.9, 1.0, 0.9],
            })
            .collect(),
            HudTab::Buildings => [
                "Hut",
                "Guard Tower",
                "Temple",
                "Spy Hut",
                "Warrior Hut",
                "Firewarrior Hut",
                "Prison",
                "Boat Hut",
            ]
            .iter()
            .map(|name| PanelEntry {
                label: name.to_string(),
                color: [0.9, 0.85, 0.7, 0.9],
            })
            .collect(),
            HudTab::Units => {
                let unit_types: [(u8, &str); 6] = [
                    (PERSON_SUBTYPE_BRAVE, "Brave"),
                    (PERSON_SUBTYPE_WARRIOR, "Warrior"),
                    (PERSON_SUBTYPE_SPY, "Spy"),
                    (PERSON_SUBTYPE_PREACHER, "Preacher"),
                    (PERSON_SUBTYPE_FIREWARRIOR, "Firewarr"),
                    (PERSON_SUBTYPE_SHAMAN, "Shaman"),
                ];
                unit_types
                    .iter()
                    .map(|(subtype, name)| {
                        let count = self
                            .unit_coordinator
                            .units()
                            .iter()
                            .filter(|u| u.alive && u.subtype == *subtype && u.tribe_index == 0)
                            .count();
                        PanelEntry {
                            label: format!("{}: {}", name, count),
                            color: [0.7, 1.0, 0.7, 0.9],
                        }
                    })
                    .collect()
            }
        };
        let mut tribe_counts = [0u32; 4];
        for u in self.unit_coordinator.units() {
            if u.alive && (u.tribe_index as usize) < 4 {
                tribe_counts[u.tribe_index as usize] += 1;
            }
        }
        let tribe_populations: Vec<TribePopulation> = (0..4u8)
            .filter(|&t| tribe_counts[t as usize] > 0)
            .map(|t| TribePopulation {
                tribe_index: t,
                count: tribe_counts[t as usize],
                color: HUD_TRIBE_COLORS[t as usize],
            })
            .collect();
        // Camera viewport: shift values are in cell coords (0-127)
        let shift_vec = self.landscape_mesh.get_shift_vector();
        let cam_cx = (shift_vec.x as f32).rem_euclid(128.0);
        let cam_cy = (shift_vec.y as f32).rem_euclid(128.0);
        let view_w = 20.0 / self.zoom.max(0.1);
        let view_h = view_w * (self.screen.height as f32 / self.screen.width.max(1) as f32);
        let camera_viewport = MinimapViewport {
            cam_cell_x: cam_cx,
            cam_cell_y: cam_cy,
            view_width_cells: view_w,
            view_height_cells: view_h,
        };

        // Selection info: show first selected unit details
        let selected_info =
            if let Some(&first_id) = self.unit_coordinator.selection.selected.first() {
                self.unit_coordinator
                    .units()
                    .get(first_id)
                    .and_then(|unit| {
                        if !unit.alive {
                            return None;
                        }
                        let name = unit_subtype_name(unit.subtype).to_string();
                        let mut extra_lines = Vec::new();
                        extra_lines.push(format!("State: {:?}", unit.state));
                        if self.unit_coordinator.selection.selected.len() > 1 {
                            extra_lines.push(format!(
                                "Selected: {}",
                                self.unit_coordinator.selection.selected.len()
                            ));
                        }
                        Some(SelectedEntityInfo {
                            name,
                            health: unit.health,
                            max_health: unit.max_health,
                            subtype: unit.subtype,
                            tribe_index: unit.tribe_index,
                            extra_lines,
                        })
                    })
            } else {
                None
            };

        // Health bars: project damaged units from world space to screen space
        let health_bars = {
            let pvm = self.unit_pvm();
            let sw = self.screen.width as f32;
            let sh = self.screen.height as f32;
            let mut bars = Vec::new();
            for unit in self
                .unit_coordinator
                .units()
                .iter()
                .filter(|u| u.alive && u.health < u.max_health)
            {
                if let Some((sx, sy)) = self.unit_screen_pos(unit, &pvm) {
                    // Offset upward so bar appears above the unit sprite
                    let bar_y = sy - 8.0;
                    if sx >= 0.0 && sx <= sw && bar_y >= 0.0 && bar_y <= sh {
                        bars.push(HealthBarEntry {
                            screen_x: sx,
                            screen_y: bar_y,
                            health_fraction: unit.health as f32 / unit.max_health.max(1) as f32,
                            bar_type: HealthBarType::Unit,
                        });
                    }
                }
            }
            bars
        };

        let player_tribe = &self.game_world.tribes.tribes[0]; // tribe 0 = player
        HudState {
            active_tab: self.hud_tab,
            minimap,
            panel_entries,
            tribe_populations,
            level_num: self.level_num as u32,
            frame_count: self.frame_count as u64,
            player_mana: player_tribe.mana,
            player_max_mana: 1_000_000,
            player_population: player_tribe.population,
            player_max_population: player_tribe.max_population,
            spell_cooldowns: Vec::new(), // Phase 4 will populate from SpellSystem
            spell_charges: crate::engine::economy::mana::compute_spell_charges(player_tribe.mana),
            camera_viewport,
            selected_info,
            health_bars,
            menu_render_data: match self.game_world.state {
                GameState::Frontend | GameState::Outro => Some(self.menu_system.render_data()),
                _ => None,
            },
        }
    }

    /// Process a game command. Returns true if the renderer needs to redraw.
    /// Sets dirty flags for specific rebuilds.
    pub fn apply_command(&mut self, cmd: &GameCommand) -> bool {
        // FLYBY_ALLOW_INTERRUPT: user camera input skips the flyby.
        if command_interrupts_flyby(cmd) {
            if let Some(flyby) = &mut self.flyby {
                if flyby.active && flyby.allow_interrupt {
                    flyby.interrupt();
                    self.flyby = None;
                    self.camera.angle_z_frac = 0.0;
                    self.focus_frac = (0.0, 0.0);
                    log::info!("Flyby interrupted by user input");
                }
            }
        }
        match cmd {
            GameCommand::RotateCamera { delta_z } => {
                self.camera.angle_z += delta_z;
                true
            }
            GameCommand::TiltCamera { delta_x } => {
                self.camera.angle_x = (self.camera.angle_x + delta_x).clamp(-90, -30);
                true
            }
            GameCommand::PanScreen { forward, right } => {
                let az = (self.camera.angle_z as f32).to_radians();
                let gx = -right * az.cos() - forward * az.sin();
                let gy = right * az.sin() - forward * az.cos();
                self.landscape_mesh.shift_x(gx.round() as i32);
                self.landscape_mesh.shift_y(gy.round() as i32);
                true
            }
            GameCommand::PanTerrain { dx, dy } => {
                self.landscape_mesh.shift_x(*dx);
                self.landscape_mesh.shift_y(*dy);
                true
            }
            GameCommand::ResetCamera => {
                self.reset_camera();
                true
            }
            GameCommand::TopDownView => {
                self.camera.angle_x = -90;
                true
            }
            GameCommand::CenterOnShaman => {
                // Needs unit_renders (App-level data). The actual centering
                // is done by App after apply_command returns.
                true
            }
            GameCommand::SetZoom(z) => {
                self.zoom = z.clamp(0.3, 5.0);
                true
            }
            GameCommand::ToggleCurvature => {
                self.curvature_enabled = !self.curvature_enabled;
                log::info!(
                    "curvature {}",
                    if self.curvature_enabled { "on" } else { "off" }
                );
                true
            }
            GameCommand::AdjustCurvature { factor } => {
                self.curvature_scale *= factor;
                log::info!("curvature_scale = {:.6}", self.curvature_scale);
                true
            }
            GameCommand::AdjustSpriteOffset { delta } => {
                self.sprite_z_offset += delta;
                eprintln!(
                    "[SPRITE] z_offset={:.4} scale={:.2}",
                    self.sprite_z_offset, self.sprite_scale
                );
                true
            }
            GameCommand::AdjustSpriteScale { delta } => {
                self.sprite_scale = (self.sprite_scale + delta).max(0.05);
                eprintln!(
                    "[SPRITE] z_offset={:.4} scale={:.2}",
                    self.sprite_z_offset, self.sprite_scale
                );
                true
            }
            GameCommand::NextLevel => {
                self.level_num = (self.level_num + 1) % 26;
                if self.level_num == 0 {
                    self.level_num = 1;
                }
                true
            }
            GameCommand::PrevLevel => {
                self.level_num = if self.level_num == 1 {
                    25
                } else {
                    self.level_num - 1
                };
                true
            }
            GameCommand::NextShader | GameCommand::PrevShader => {
                // Shader cycling stays renderer-side (program_container is GPU state)
                true
            }
            GameCommand::ToggleObjects => {
                self.show_objects = !self.show_objects;
                log::info!("objects {}", if self.show_objects { "on" } else { "off" });
                true
            }
            GameCommand::ToggleShadows => {
                self.show_shadows = !self.show_shadows;
                self.show_lighting = !self.show_lighting;
                log::info!(
                    "shadows+lighting {}",
                    if self.show_shadows { "on" } else { "off" }
                );
                true
            }
            GameCommand::ToggleMarkers => {
                self.show_markers = !self.show_markers;
                log::info!("markers {}", if self.show_markers { "on" } else { "off" });
                true
            }
            GameCommand::AdjustSunlight { dx, dy } => {
                self.sunlight.x += dx;
                self.sunlight.y += dy;
                log::debug!("sunlight = {:?}", self.sunlight);
                true
            }
            GameCommand::SelectUnit(id) => {
                self.unit_coordinator.selection.select_single(*id);
                true
            }
            GameCommand::SelectMultiple(ids) => {
                self.unit_coordinator.selection.select_multiple(ids.clone());
                true
            }
            GameCommand::ClearSelection => {
                self.unit_coordinator.selection.clear();
                true
            }
            GameCommand::OrderMove { x, z } => {
                let target = crate::engine::movement::WorldCoord::new(*x as i16, *z as i16);
                self.unit_coordinator.order_move(target);
                true
            }
            GameCommand::ToggleSimulation => {
                if self.game_world.state == GameState::InGame {
                    let paused = self.game_world.flags.is_paused();
                    self.game_world.flags.set_paused(!paused);
                    log::info!("game simulation {}", if paused { "ON" } else { "OFF" });
                }
                true
            }
            GameCommand::IncreaseGameSpeed => {
                let new_speed = (self.game_world.game_speed + 2).min(30);
                self.game_world.set_game_speed(new_speed);
                println!("game speed: {} ticks/sec", self.game_world.game_speed);
                false
            }
            GameCommand::DecreaseGameSpeed => {
                let new_speed = self.game_world.game_speed.saturating_sub(2).max(4);
                self.game_world.set_game_speed(new_speed);
                println!("game speed: {} ticks/sec", self.game_world.game_speed);
                false
            }
            GameCommand::SetHudTab(tab) => {
                self.hud_tab = *tab;
                true
            }
            GameCommand::ToggleHud => {
                self.hud_visible = !self.hud_visible;
                true
            }
            GameCommand::ToggleCompass => {
                self.compass_visible = !self.compass_visible;
                true
            }
            GameCommand::ToggleWalkability => {
                self.walkability_visible = !self.walkability_visible;
                true
            }
            GameCommand::Quit => true,
            GameCommand::StartFlyby(ref state) => {
                let mut flyby = state.clone();
                let n = self.landscape_mesh.width() as i32;
                flyby.set_pos_modulus(2 * n);
                // Channels animate from the camera's current state, like the
                // original (FUN_004daf50/FUN_004db200 read the live camera).
                let focus_vertex = self.camera_focus_vertex();
                let shift = self.landscape_mesh.get_shift_vector();
                let (wx, wy) = shift_to_flyby_pos(shift.x as i32, shift.y as i32, n, focus_vertex);
                flyby.set_initial(&crate::engine::ai::flyby::FlybyCameraOutput {
                    angle_z: deg_to_flyby_angle(self.camera.angle_z),
                    angle_x: 0,
                    zoom: factor_to_flyby_zoom(self.zoom),
                    world_x: wx,
                    world_y: wy,
                });
                self.flyby = Some(flyby);
                log::info!("Flyby started at tick {}", state.start_tick);
                true
            }
            GameCommand::StopFlyby => {
                self.flyby = None;
                self.camera.angle_z_frac = 0.0;
                self.focus_frac = (0.0, 0.0);
                log::info!("Flyby stopped");
                true
            }
            // Menu navigation
            GameCommand::MenuUp => {
                self.menu_system.move_cursor(-1);
                true
            }
            GameCommand::MenuDown => {
                self.menu_system.move_cursor(1);
                true
            }
            GameCommand::MenuSelect => {
                let action = self.menu_system.select_item();
                self.handle_menu_action(action);
                true
            }
            GameCommand::MenuBack => {
                self.menu_system.back();
                true
            }
            GameCommand::MenuNavigate(target) => {
                let screen = match target {
                    crate::engine::command::MenuTarget::CampaignSelect => {
                        MenuScreen::CampaignSelect
                    }
                    crate::engine::command::MenuTarget::LoadGame => MenuScreen::LoadGame,
                    crate::engine::command::MenuTarget::Options => MenuScreen::Options,
                };
                self.menu_system.navigate_to(screen);
                true
            }
            GameCommand::StartLevel { level_num } => {
                self.campaign_state.set_level(*level_num);
                self.level_num = *level_num as u8;
                self.game_world.state = GameState::InGame;
                self.game_world.game_tick = 0;
                true
            }
            // Save/Load
            GameCommand::QuickSave => {
                let (ai_vars, ai_every) = match &self.ai_system {
                    Some(ai) => ai.extract_save_state(),
                    None => (Vec::new(), Vec::new()),
                };
                let save = crate::engine::save::SaveFile {
                    version: crate::engine::save::SAVE_VERSION,
                    level_num: self.campaign_state.current_level,
                    game_tick: self.game_world.game_tick,
                    tick_counter: self.game_world.tick_counter,
                    game_speed: self.game_world.game_speed,
                    player_tribe: self.game_world.player_tribe,
                    ai_update_mult: self.game_world.ai_update_mult,
                    flags: self.game_world.flags.clone(),
                    rng: self.game_world.rng.clone(),
                    tribes: self.game_world.tribes.clone(),
                    ai_script_variables: ai_vars,
                    ai_every_counters: ai_every,
                };
                let save_dir = dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("pop3")
                    .join("saves");
                match crate::engine::save::quicksave(&save, &save_dir) {
                    Ok(()) => log::info!("Quicksave complete"),
                    Err(e) => log::error!("Quicksave failed: {}", e),
                }
                false
            }
            GameCommand::QuickLoad => {
                let save_dir = dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("pop3")
                    .join("saves");
                match crate::engine::save::quickload(&save_dir) {
                    Ok(save) => {
                        self.game_world.game_tick = save.game_tick;
                        self.game_world.tick_counter = save.tick_counter;
                        self.game_world.game_speed = save.game_speed;
                        self.game_world.player_tribe = save.player_tribe;
                        self.game_world.ai_update_mult = save.ai_update_mult;
                        self.game_world.flags = save.flags;
                        self.game_world.rng = save.rng;
                        self.game_world.tribes = save.tribes;
                        self.campaign_state.set_level(save.level_num);
                        if let Some(ref mut ai) = self.ai_system {
                            ai.restore_save_state(
                                &save.ai_script_variables,
                                &save.ai_every_counters,
                            );
                        }
                        log::info!("Quickload complete");
                    }
                    Err(e) => log::error!("Quickload failed: {}", e),
                }
                true
            }
            GameCommand::SaveGame { .. } | GameCommand::LoadGame { .. } => false,
            // Building commands: handled by game logic, not renderer
            GameCommand::PlaceBuilding { .. }
            | GameCommand::CancelPlacement
            | GameCommand::EnterBuildMode { .. }
            | GameCommand::EnterBuilding { .. }
            | GameCommand::TrainUnit { .. } => false,
        }
    }

    /// Produce the output boundary for the renderer — a snapshot of all
    /// game-logic state needed to draw one frame.
    pub fn frame_state<'a>(&'a self, drag_state: &'a DragState) -> FrameState<'a> {
        FrameState {
            camera: &self.camera,
            screen: &self.screen,
            zoom: self.zoom,
            landscape: &self.landscape_mesh,
            curvature_scale: if self.curvature_enabled {
                self.curvature_scale
            } else {
                0.0
            },
            sunlight: self.sunlight,
            wat_offset: self.wat_offset,
            show_objects: self.show_objects,
            show_shadows: self.show_shadows,
            show_lighting: self.show_lighting,
            show_markers: self.show_markers,
            unit_coordinator: &self.unit_coordinator,
            level_objects: &self.level_objects,
            building_objects: &self.building_objects,
            scenery_objects: &self.scenery_objects,
            shapes: &self.shapes,
            hud_state: self.build_hud_state(),
            drag_state,
            ghost_preview: None,
            needs_building_rebuild: false,
            needs_spawn_rebuild: false,
            needs_unit_rebuild: false,
            needs_level_reload: false,
        }
    }
}


pub fn toroidal_delta(from: usize, to: usize, n: usize) -> i32 {
    let d = (to as i32 - from as i32).rem_euclid(n as i32);
    if d <= (n as i32) / 2 {
        d
    } else {
        d - n as i32
    }
}

/******************************************************************************/
// Flyby coordinate mapping.
//
// Flyby script positions are half-cells in the *original* game's world axes.
// The renderer's map is that world rotated 90°: renderer_x = orig_y,
// renderer_y = (n-1) - orig_x (the same mapping used by
// sprites::extract_all_unit_cells). Original angles are 2048 units per full
// turn (sin/cos LUTs indexed `& 0x7ff`); the map rotation adds a constant
// +90° to the renderer yaw.

/// Renderer yaw offset caused by the 90° map rotation.
const FLYBY_YAW_OFFSET_DEG: f32 = 90.0;

/// Original flyby zoom accumulator clamps to ±0x2000 before `>>3`,
/// i.e. ±1024 zoom units around the default.
const FLYBY_ZOOM_UNITS: f32 = 1024.0;

/// Convert a flyby half-cell position to a landscape shift plus the
/// sub-cell remainder in grid cells. `focus_vertex` is the renderer's
/// screen-center vertex (63.5 for a 128 map, see camera_focus_vertex);
/// shift + frac + focus_vertex ≡ target renderer cell (mod n). The original
/// camera works in world units (512 per cell), so whole-cell quantization
/// would be 512× coarser than the original.
pub fn flyby_pos_to_shift_frac(
    world_x: f32,
    world_y: f32,
    n: i32,
    focus_vertex: f32,
) -> ((usize, usize), (f32, f32)) {
    let nf = n as f32;
    let rend_x = world_y / 2.0;
    let rend_y = (nf - 1.0) - world_x / 2.0;
    let sx = (rend_x - focus_vertex).rem_euclid(nf);
    let sy = (rend_y - focus_vertex).rem_euclid(nf);
    (
        (sx.floor() as usize % n as usize, sy.floor() as usize % n as usize),
        (sx.fract(), sy.fract()),
    )
}

/// Inverse of `flyby_pos_to_shift_frac` for whole shifts: current shift →
/// flyby half-cell position of the cell at screen center.
pub fn shift_to_flyby_pos(sx: i32, sy: i32, n: i32, focus_vertex: f32) -> (i16, i16) {
    let nf = n as f32;
    let rend_x = (sx as f32 + focus_vertex).rem_euclid(nf);
    let rend_y = (sy as f32 + focus_vertex).rem_euclid(nf);
    let orig_x = (nf - 1.0) - rend_y;
    let orig_y = rend_x;
    ((orig_x * 2.0).round() as i16, (orig_y * 2.0).round() as i16)
}

/// User camera input skips an interruptible flyby (FLYBY_ALLOW_INTERRUPT).
pub fn command_interrupts_flyby(cmd: &GameCommand) -> bool {
    matches!(
        cmd,
        GameCommand::RotateCamera { .. }
            | GameCommand::TiltCamera { .. }
            | GameCommand::PanScreen { .. }
            | GameCommand::PanTerrain { .. }
            | GameCommand::ResetCamera
            | GameCommand::TopDownView
            | GameCommand::CenterOnShaman
            | GameCommand::SetZoom(_)
    )
}

/// Convert a flyby angle (2048 units/turn) to renderer yaw degrees.
pub fn flyby_angle_to_deg_f(units: f32) -> f32 {
    (units * 360.0 / crate::engine::ai::flyby::FLYBY_ANGLE_MODULUS as f32 + FLYBY_YAW_OFFSET_DEG)
        .rem_euclid(360.0)
}

/// Whole-degree variant of `flyby_angle_to_deg_f`.
pub fn flyby_angle_to_deg(units: i16) -> i16 {
    flyby_angle_to_deg_f(units as f32).round() as i16
}

/// Inverse of `flyby_angle_to_deg`: renderer yaw degrees → flyby angle units.
pub fn deg_to_flyby_angle(deg: i16) -> i16 {
    let m = crate::engine::ai::flyby::FLYBY_ANGLE_MODULUS as f32;
    ((deg as f32 - FLYBY_YAW_OFFSET_DEG) * m / 360.0).rem_euclid(m).round() as i16
}

/// Convert flyby zoom units (±1024 around 0) to the renderer zoom factor.
pub fn flyby_zoom_to_factor_f(units: f32) -> f32 {
    (1.0 + units / FLYBY_ZOOM_UNITS).clamp(0.3, 5.0)
}

/// Whole-unit variant of `flyby_zoom_to_factor_f`.
pub fn flyby_zoom_to_factor(units: i16) -> f32 {
    flyby_zoom_to_factor_f(units as f32)
}

/// Inverse of `flyby_zoom_to_factor`.
pub fn factor_to_flyby_zoom(factor: f32) -> i16 {
    ((factor - 1.0) * FLYBY_ZOOM_UNITS).round() as i16
}


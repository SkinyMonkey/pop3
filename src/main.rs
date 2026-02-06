use std::path::{Path, PathBuf};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, ArgAction, Command};

use cgmath::{Point2, Vector2, Vector3, Vector4, Matrix4, SquareMatrix};

use faithful::model::{VertexModel, MeshModel};
use faithful::default_model::DefaultModel;
use faithful::tex_model::{TexModel, TexVertex};
use faithful::view::*;

use faithful::pop::psfb::ContainerPSFB;
use faithful::pop::types::BinDeserializer;

use faithful::intersect::intersect_iter;

use faithful::landscape::{LandscapeMesh, LandscapeModel};
use faithful::pop::level::LevelRes;
use faithful::pop::landscape::{make_texture_land, draw_texture_u8};

use faithful::gpu::context::GpuContext;
use faithful::gpu::pipeline::create_pipeline;
use faithful::gpu::buffer::GpuBuffer;
use faithful::gpu::texture::GpuTexture;
use faithful::envelop::*;

/******************************************************************************/

fn obj_colors() -> Vec<Vector3<u8>> {
    vec![ Vector3{x: 255, y: 0, z: 0}
        , Vector3{x: 128, y: 0, z: 128}
        , Vector3{x: 0, y: 255, z: 0}
        , Vector3{x: 64, y: 64, z: 128}
        , Vector3{x: 128, y: 0, z: 128}
        , Vector3{x: 0, y: 255, z: 255}
        , Vector3{x: 0, y: 0, z: 255}
        , Vector3{x: 0, y: 64, z: 0}
        , Vector3{x: 128, y: 64, z: 0}
    ]
}

type LandscapeMeshS = LandscapeMesh<128>;

#[derive(Debug, PartialEq, Clone, Copy)]
enum ActionMode {
    /// Torus view: fixed camera, world rotates (matches original game).
    /// Left/Right = rotate world, Up/Down = tilt view, W/S = zoom.
    TorusView,
    /// Free camera: move anywhere in 3D, prints camera state on every keypress.
    FreeCamera,
    GlobalMoveXY,
    GlobalMoveXZ,
    GlobalRotateXZ,
    GlobalRotateXY,
    GlobalMoveRot,
}

impl ActionMode {
    fn process_key(&mut self, key: KeyCode, camera: &mut Camera, landscape_mesh: &mut LandscapeMeshS) -> bool {
        let prev_self = *self;
        match self {
            Self::TorusView =>
                match key {
                    // World rotation (orbit camera around scene center)
                    KeyCode::KeyQ => { camera.angle_z -= 5; },
                    KeyCode::KeyE => { camera.angle_z += 5; },
                    // Tilt (pitch) the view
                    KeyCode::ArrowUp => { camera.angle_x = (camera.angle_x + 5).min(-30); },
                    KeyCode::ArrowDown => { camera.angle_x = (camera.angle_x - 5).max(-90); },
                    // Scroll the world (toroidal panning, screen-relative)
                    KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD => {
                        let (dx, dy) = match key {
                            KeyCode::KeyW => (0.0f32, -1.0f32),
                            KeyCode::KeyS => (0.0, 1.0),
                            KeyCode::KeyA => (1.0, 0.0),
                            KeyCode::KeyD => (-1.0, 0.0),
                            _ => unreachable!(),
                        };
                        // Rotate screen direction by angle_z to get grid direction
                        let az = (camera.angle_z as f32).to_radians();
                        let gx = dx * az.cos() - dy * az.sin();
                        let gy = dx * az.sin() + dy * az.cos();
                        landscape_mesh.shift_x(gx.round() as i32);
                        landscape_mesh.shift_y(gy.round() as i32);
                    },
                    KeyCode::KeyP => { *self = Self::FreeCamera; },
                    _ => (),
                },
            Self::FreeCamera =>
                match key {
                    // Rotation
                    KeyCode::ArrowUp => { camera.angle_x += 5; },
                    KeyCode::ArrowDown => { camera.angle_x -= 5; },
                    KeyCode::ArrowLeft => { camera.angle_z -= 5; },
                    KeyCode::ArrowRight => { camera.angle_z += 5; },
                    // Position
                    KeyCode::KeyW => { camera.pos.y += 0.1; },
                    KeyCode::KeyS => { camera.pos.y -= 0.1; },
                    KeyCode::KeyA => { camera.pos.x -= 0.1; },
                    KeyCode::KeyD => { camera.pos.x += 0.1; },
                    KeyCode::KeyE => { camera.pos.z += 0.1; },
                    KeyCode::KeyF => { camera.pos.z -= 0.1; },
                    // Y rotation
                    KeyCode::KeyI => { camera.angle_y += 5; },
                    KeyCode::KeyO => { camera.angle_y -= 5; },
                    KeyCode::KeyP => { *self = Self::GlobalRotateXZ; },
                    _ => (),
                },
            Self::GlobalRotateXZ =>
                match key {
                    KeyCode::ArrowUp => { camera.angle_x += 5; },
                    KeyCode::ArrowDown => { camera.angle_x -= 5; },
                    KeyCode::ArrowLeft => { camera.angle_z += 5; },
                    KeyCode::ArrowRight => { camera.angle_z -= 5; },
                    KeyCode::KeyP => { *self = Self::GlobalRotateXY; },
                    _ => (),
                },
            Self::GlobalRotateXY =>
                match key {
                    KeyCode::ArrowUp => { camera.angle_x += 5; },
                    KeyCode::ArrowDown => { camera.angle_x -= 5; },
                    KeyCode::ArrowLeft => { camera.angle_y += 5; },
                    KeyCode::ArrowRight => { camera.angle_y -= 5; },
                    KeyCode::KeyP => { *self = Self::GlobalMoveXY; },
                    _ => (),
                },
            Self::GlobalMoveXY =>
                match key {
                    KeyCode::ArrowUp => { camera.pos.x += 0.1; },
                    KeyCode::ArrowDown => { camera.pos.x -= 0.1; },
                    KeyCode::ArrowLeft => { camera.pos.y += 0.1; },
                    KeyCode::ArrowRight => { camera.pos.y -= 0.1; },
                    KeyCode::KeyP => { *self = Self::GlobalMoveXZ; },
                    _ => (),
                },
            Self::GlobalMoveXZ =>
                match key {
                    KeyCode::ArrowUp => { camera.pos.z += 0.1; },
                    KeyCode::ArrowDown => { camera.pos.z -= 0.1; },
                    KeyCode::ArrowLeft => { camera.pos.z += 0.1; },
                    KeyCode::ArrowRight => { camera.pos.z -= 0.1; },
                    KeyCode::KeyP => { *self = Self::GlobalMoveRot; },
                    _ => (),
                },
            Self::GlobalMoveRot =>
                match key {
                    KeyCode::ArrowUp => { camera.pos.z += 0.1; },
                    KeyCode::ArrowDown => { camera.pos.z -= 0.1; },
                    KeyCode::ArrowLeft => { camera.angle_z -= 5; },
                    KeyCode::ArrowRight => { camera.angle_z += 5; },
                    KeyCode::KeyP => { *self = Self::TorusView; },
                    _ => (),
                },
        }
        if *self != prev_self {
            println!("{:?}", self);
        }
        true
    }
}

/******************************************************************************/

/// Packed landscape uniform data matching the WGSL LandscapeParams struct.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LandscapeUniformData {
    level_shift: [i32; 4],
    height_scale: f32,
    step: f32,
    width: i32,
    selected_frag: i32,
    selected_color: [f32; 4],
    sunlight: [f32; 4],
    wat_offset: i32,
    curvature_scale: f32,
    camera_focus: [f32; 2],
    viewport_radius: f32,
    _pad2: [f32; 3],
}

/// A landscape program variant with its own pipeline and group-1 bind group.
struct LandscapeVariant {
    pipeline: wgpu::RenderPipeline,
    bind_group_1: wgpu::BindGroup,
}

struct LandscapeProgramContainer {
    variants: Vec<LandscapeVariant>,
    index: usize,
}

impl LandscapeProgramContainer {
    fn new() -> Self {
        Self { variants: Vec::new(), index: 0 }
    }

    fn add(&mut self, variant: LandscapeVariant) {
        self.variants.push(variant);
    }

    fn next(&mut self) {
        if !self.variants.is_empty() {
            self.index = (self.index + 1) % self.variants.len();
        }
    }

    fn prev(&mut self) {
        if self.variants.is_empty() { return; }
        self.index = if self.index == 0 {
            self.variants.len() - 1
        } else {
            self.index - 1
        };
    }

    fn current(&self) -> Option<&LandscapeVariant> {
        self.variants.get(self.index)
    }
}

/******************************************************************************/

fn make_landscape_model(device: &wgpu::Device, landscape_mesh: &LandscapeMeshS) -> ModelEnvelop<LandscapeModel> {
    let mut model: LandscapeModel = MeshModel::new();
    landscape_mesh.to_model(&mut model);
    log::debug!("Landscape mesh - vertices={:?}, indices={:?}", model.vertices.len(), model.indices.len());
    let m = vec![(RenderType::Triangles, model)];
    let mut model_main = ModelEnvelop::<LandscapeModel>::new(device, m);
    if let Some(m) = model_main.get(0) {
        m.location.x = -2.0;
        m.location.y = -2.0;
        m.scale = 2.5;
    }
    model_main
}

/// Extract spawn cell coordinates from level data.
/// Returns (cell_x, cell_y, tribe_index) for each reincarnation site (unit_class == 2).
fn extract_spawn_cells(level_res: &LevelRes) -> Vec<(f32, f32, u8)> {
    let mut found = [false; 4];
    let mut cells = Vec::new();
    let n = level_res.landscape.land_size() as f32;
    for unit in &level_res.units {
        let tribe = unit.tribe_index() as usize;
        if unit.unit_class == 2 && tribe < 4 && !found[tribe] {
            found[tribe] = true;
            // Bevy convention: cell_x from loc_x, cell_z from loc_y
            let bevy_x = ((unit.loc_x() >> 8) / 2) as f32 + 0.5;
            let bevy_z = ((unit.loc_y() >> 8) / 2) as f32 + 0.5;
            // Faithful grid: x = bevy_z, y = (N-1) - bevy_x
            // Because faithful loads height[file_col][file_row] then flips first index,
            // so grid_x maps to file_row (=bevy_z) and grid_y maps to 127-file_col (=127-bevy_x).
            let cell_x = bevy_z;
            let cell_y = (n - 1.0) - bevy_x;
            cells.push((cell_x, cell_y, unit.tribe_index()));
        }
    }
    cells.sort_by_key(|(_, _, t)| *t);
    cells
}

const SHAMAN_SPRITE_START: usize = 7578;
const SHAMAN_FRAMES_PER_DIR: usize = 8;
const STORED_DIRECTIONS: usize = 5;
struct ShamanAtlasInfo {
    frame_width: u32,
    frame_height: u32,
}

/// Build a 5-row × 8-column sprite atlas for the shaman idle animation.
/// Layout: rows = 5 stored directions, cols = 8 animation frames.
/// Returns (atlas_w, atlas_h, rgba_data, atlas_info) or None if unavailable.
fn build_shaman_atlas(base: &Path, palette: &[u8]) -> Option<(u32, u32, Vec<u8>, ShamanAtlasInfo)> {
    let hspr_path = base.join("data").join("HSPR0-0.DAT");
    let container = ContainerPSFB::from_file(&hspr_path)?;

    // First pass: find max frame dimensions
    let mut max_w: u16 = 0;
    let mut max_h: u16 = 0;
    for dir in 0..STORED_DIRECTIONS {
        for f in 0..SHAMAN_FRAMES_PER_DIR {
            let idx = SHAMAN_SPRITE_START + dir * SHAMAN_FRAMES_PER_DIR + f;
            if let Some(info) = container.get_info(idx) {
                max_w = max_w.max(info.width);
                max_h = max_h.max(info.height);
            }
        }
    }
    if max_w == 0 || max_h == 0 { return None; }

    let fw = max_w as u32;
    let fh = max_h as u32;
    let atlas_w = fw * SHAMAN_FRAMES_PER_DIR as u32;
    let atlas_h = fh * STORED_DIRECTIONS as u32;
    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    // Second pass: decode sprites into atlas cells, centered
    for dir in 0..STORED_DIRECTIONS {
        for f in 0..SHAMAN_FRAMES_PER_DIR {
            let idx = SHAMAN_SPRITE_START + dir * SHAMAN_FRAMES_PER_DIR + f;
            if let Some(image) = container.get_image(idx) {
                let info = container.get_info(idx).unwrap();
                let sw = info.width as u32;
                let sh = info.height as u32;
                let ox = (fw - sw) / 2;
                let oy = (fh - sh) / 2;
                let cell_x = f as u32 * fw;
                let cell_y = dir as u32 * fh;

                for y in 0..sh {
                    for x in 0..sw {
                        let pal_index = image.data[(y * sw + x) as usize] as usize;
                        let dst_x = cell_x + ox + x;
                        let dst_y = cell_y + oy + y;
                        let off = ((dst_y * atlas_w + dst_x) * 4) as usize;
                        if pal_index != 0 {
                            rgba[off] = palette[pal_index * 4];
                            rgba[off + 1] = palette[pal_index * 4 + 1];
                            rgba[off + 2] = palette[pal_index * 4 + 2];
                            rgba[off + 3] = 255;
                        }
                    }
                }
            }
        }
    }
    Some((atlas_w, atlas_h, rgba, ShamanAtlasInfo { frame_width: fw, frame_height: fh }))
}

/// Returns (source_direction_row, is_mirrored) for display direction 0-7.
fn get_source_direction(dir: usize) -> (usize, bool) {
    match dir {
        0 => (0, false),
        1 => (1, false),
        2 => (2, false),
        3 => (3, false),
        4 => (4, false),
        5 => (3, true),
        6 => (2, true),
        7 => (1, true),
        _ => (0, false),
    }
}

/// Fixed facing direction per tribe, in game angle units (0x000-0x7FF).
/// From RE_NOTES: 0x000=East, 0x100=NE, 0x200=North, 0x300=NW, 0x400=West, etc.
fn tribe_facing_direction(tribe_index: u8) -> u16 {
    match tribe_index {
        0 => 0x200, // North
        1 => 0x600, // South
        2 => 0x000, // East
        3 => 0x400, // West
        _ => 0x000,
    }
}

/// Compute display sprite direction (0-7) from camera angle and unit facing.
/// Implements the exact game formula from RE_NOTES (FUN_0046af00):
///   direction = ((g_CameraTarget->rotation - object->facing) - 0x380) & 0x700) >> 8
fn sprite_direction_from_angle(camera_angle_z: i16, unit_facing_game: u16) -> usize {
    // Convert angle_z (degrees) to game angle units (0-2047, where 2048 = 360°)
    let camera_rot = ((camera_angle_z as i32) * 2048 / 360 % 2048 + 2048) % 2048;
    let raw = (camera_rot - unit_facing_game as i32 - 0x380) & 0x700;
    (raw >> 8) as usize
}

/// Build camera-facing billboard quads for spawn markers.
/// Each spawn gets a single quad (6 vertices) oriented to face the camera.
/// `angle_z` controls billboard orientation (face the camera).
/// Sprite direction is computed from `angle_z` (camera rotation) and each unit's
/// facing using the game formula from RE_NOTES.
fn build_spawn_model(device: &wgpu::Device, cells: &[(f32, f32, u8)],
                     landscape: &LandscapeMesh<128>, curvature_scale: f32,
                     angle_x: i16, angle_z: i16,
                     atlas_info: Option<&ShamanAtlasInfo>) -> ModelEnvelop<TexModel> {
    let mut model: TexModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();

    // Sprite sizing: use atlas aspect ratio if available
    let sprite_h = step * 1.5;
    let half_w = if let Some(info) = atlas_info {
        let aspect = info.frame_width as f32 / info.frame_height as f32;
        sprite_h * aspect / 2.0
    } else {
        step * 0.6
    };

    let center = (w - 1.0) * step / 2.0;

    // Billboard orientation: screen-aligned right and up vectors from camera angles.
    // transform = Rx(ax) * Rz(az), so inverse = Rz(-az) * Rx(-ax).
    // Screen right = column 0 of inverse, screen up = column 1 of inverse.
    let az = (angle_z as f32).to_radians();
    let ax = (angle_x as f32).to_radians();
    // Right vector (screen X axis in world space)
    let right = Vector3::new(az.cos(), -az.sin(), 0.0);
    // Up vector (screen Y axis in world space) — tilted by angle_x
    let up = Vector3::new(az.sin() * ax.cos(), az.cos() * ax.cos(), -ax.sin());

    let fpd = SHAMAN_FRAMES_PER_DIR as f32;
    let uv_scale_x = 1.0 / fpd;
    let uv_scale_y = 1.0 / STORED_DIRECTIONS as f32;
    let uv_off_x = 0.0; // frame 0 (idle)

    for &(cell_x, cell_y, tribe_index) in cells {
        let vis_x = ((cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;

        let ix = (cell_x as usize).min(127);
        let iy = (cell_y as usize).min(127);
        let gz = landscape.height_at(ix, iy) as f32 * height_scale;

        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * curvature_scale;
        let z_base = gz - curvature_offset + 0.01;

        let tid = tribe_index as i16;

        // Per-unit sprite direction using game formula from RE_NOTES
        let unit_facing = tribe_facing_direction(tribe_index);
        let display_dir = sprite_direction_from_angle(angle_z, unit_facing);
        let (src_dir, mirrored) = get_source_direction(display_dir);

        let uv_off_y = src_dir as f32 / STORED_DIRECTIONS as f32;
        let (u_left, u_right) = if mirrored {
            (uv_off_x + uv_scale_x, uv_off_x)
        } else {
            (uv_off_x, uv_off_x + uv_scale_x)
        };
        let v_bottom = uv_off_y + uv_scale_y;
        let v_top = uv_off_y;

        // Screen-facing billboard quad using right and up vectors
        let base = Vector3::new(gx, gy, z_base);
        let bl = base - right * half_w;
        let br = base + right * half_w;
        let tl = bl + up * sprite_h;
        let tr = br + up * sprite_h;

        let v = |p: Vector3<f32>, u: f32, v: f32| -> TexVertex {
            TexVertex { coord: p, uv: Vector2::new(u, v), tex_id: tid }
        };

        // Single camera-facing quad (2 triangles, 6 vertices)
        model.push_vertex(v(bl, u_left,  v_bottom));
        model.push_vertex(v(br, u_right, v_bottom));
        model.push_vertex(v(tr, u_right, v_top));
        model.push_vertex(v(bl, u_left,  v_bottom));
        model.push_vertex(v(tr, u_right, v_top));
        model.push_vertex(v(tl, u_left,  v_top));
    }
    let m = vec![(RenderType::Triangles, model)];
    ModelEnvelop::<TexModel>::new(device, m)
}

/******************************************************************************/
// Overlay text rendering — minimal bitmap font

/// 8×8 bitmap font for ASCII 32..127 (96 glyphs).
/// Each glyph is 8 bytes (one byte per row, MSB = leftmost pixel).
/// This is a compact CP437-style font embedded as a constant.
const FONT_8X8: [[u8; 8]; 96] = {
    let mut f = [[0u8; 8]; 96];
    // Space (32)
    f[0] = [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00];
    // ! (33)
    f[1] = [0x18,0x18,0x18,0x18,0x18,0x00,0x18,0x00];
    // " (34)
    f[2] = [0x6C,0x6C,0x6C,0x00,0x00,0x00,0x00,0x00];
    // # (35)
    f[3] = [0x6C,0x6C,0xFE,0x6C,0xFE,0x6C,0x6C,0x00];
    // $ (36)
    f[4] = [0x18,0x7E,0xC0,0x7C,0x06,0xFC,0x18,0x00];
    // % (37)
    f[5] = [0x00,0xC6,0xCC,0x18,0x30,0x66,0xC6,0x00];
    // & (38)
    f[6] = [0x38,0x6C,0x38,0x76,0xDC,0xCC,0x76,0x00];
    // ' (39)
    f[7] = [0x18,0x18,0x30,0x00,0x00,0x00,0x00,0x00];
    // ( (40)
    f[8] = [0x0C,0x18,0x30,0x30,0x30,0x18,0x0C,0x00];
    // ) (41)
    f[9] = [0x30,0x18,0x0C,0x0C,0x0C,0x18,0x30,0x00];
    // * (42)
    f[10] = [0x00,0x66,0x3C,0xFF,0x3C,0x66,0x00,0x00];
    // + (43)
    f[11] = [0x00,0x18,0x18,0x7E,0x18,0x18,0x00,0x00];
    // , (44)
    f[12] = [0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x30];
    // - (45)
    f[13] = [0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00];
    // . (46)
    f[14] = [0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x00];
    // / (47)
    f[15] = [0x06,0x0C,0x18,0x30,0x60,0xC0,0x80,0x00];
    // 0 (48)
    f[16] = [0x7C,0xC6,0xCE,0xD6,0xE6,0xC6,0x7C,0x00];
    // 1 (49)
    f[17] = [0x18,0x38,0x18,0x18,0x18,0x18,0x7E,0x00];
    // 2 (50)
    f[18] = [0x7C,0xC6,0x06,0x1C,0x30,0x66,0xFE,0x00];
    // 3 (51)
    f[19] = [0x7C,0xC6,0x06,0x3C,0x06,0xC6,0x7C,0x00];
    // 4 (52)
    f[20] = [0x1C,0x3C,0x6C,0xCC,0xFE,0x0C,0x1E,0x00];
    // 5 (53)
    f[21] = [0xFE,0xC0,0xFC,0x06,0x06,0xC6,0x7C,0x00];
    // 6 (54)
    f[22] = [0x38,0x60,0xC0,0xFC,0xC6,0xC6,0x7C,0x00];
    // 7 (55)
    f[23] = [0xFE,0xC6,0x0C,0x18,0x30,0x30,0x30,0x00];
    // 8 (56)
    f[24] = [0x7C,0xC6,0xC6,0x7C,0xC6,0xC6,0x7C,0x00];
    // 9 (57)
    f[25] = [0x7C,0xC6,0xC6,0x7E,0x06,0x0C,0x78,0x00];
    // : (58)
    f[26] = [0x00,0x18,0x18,0x00,0x00,0x18,0x18,0x00];
    // ; (59)
    f[27] = [0x00,0x18,0x18,0x00,0x00,0x18,0x18,0x30];
    // < (60)
    f[28] = [0x0C,0x18,0x30,0x60,0x30,0x18,0x0C,0x00];
    // = (61)
    f[29] = [0x00,0x00,0x7E,0x00,0x00,0x7E,0x00,0x00];
    // > (62)
    f[30] = [0x60,0x30,0x18,0x0C,0x18,0x30,0x60,0x00];
    // ? (63)
    f[31] = [0x7C,0xC6,0x0C,0x18,0x18,0x00,0x18,0x00];
    // @ (64)
    f[32] = [0x7C,0xC6,0xDE,0xDE,0xDE,0xC0,0x78,0x00];
    // A (65)
    f[33] = [0x38,0x6C,0xC6,0xC6,0xFE,0xC6,0xC6,0x00];
    // B (66)
    f[34] = [0xFC,0x66,0x66,0x7C,0x66,0x66,0xFC,0x00];
    // C (67)
    f[35] = [0x3C,0x66,0xC0,0xC0,0xC0,0x66,0x3C,0x00];
    // D (68)
    f[36] = [0xF8,0x6C,0x66,0x66,0x66,0x6C,0xF8,0x00];
    // E (69)
    f[37] = [0xFE,0x62,0x68,0x78,0x68,0x62,0xFE,0x00];
    // F (70)
    f[38] = [0xFE,0x62,0x68,0x78,0x68,0x60,0xF0,0x00];
    // G (71)
    f[39] = [0x3C,0x66,0xC0,0xC0,0xCE,0x66,0x3E,0x00];
    // H (72)
    f[40] = [0xC6,0xC6,0xC6,0xFE,0xC6,0xC6,0xC6,0x00];
    // I (73)
    f[41] = [0x3C,0x18,0x18,0x18,0x18,0x18,0x3C,0x00];
    // J (74)
    f[42] = [0x1E,0x0C,0x0C,0x0C,0xCC,0xCC,0x78,0x00];
    // K (75)
    f[43] = [0xE6,0x66,0x6C,0x78,0x6C,0x66,0xE6,0x00];
    // L (76)
    f[44] = [0xF0,0x60,0x60,0x60,0x62,0x66,0xFE,0x00];
    // M (77)
    f[45] = [0xC6,0xEE,0xFE,0xFE,0xD6,0xC6,0xC6,0x00];
    // N (78)
    f[46] = [0xC6,0xE6,0xF6,0xDE,0xCE,0xC6,0xC6,0x00];
    // O (79)
    f[47] = [0x7C,0xC6,0xC6,0xC6,0xC6,0xC6,0x7C,0x00];
    // P (80)
    f[48] = [0xFC,0x66,0x66,0x7C,0x60,0x60,0xF0,0x00];
    // Q (81)
    f[49] = [0x7C,0xC6,0xC6,0xC6,0xD6,0xDE,0x7C,0x06];
    // R (82)
    f[50] = [0xFC,0x66,0x66,0x7C,0x6C,0x66,0xE6,0x00];
    // S (83)
    f[51] = [0x7C,0xC6,0xE0,0x7C,0x0E,0xC6,0x7C,0x00];
    // T (84)
    f[52] = [0x7E,0x7E,0x5A,0x18,0x18,0x18,0x3C,0x00];
    // U (85)
    f[53] = [0xC6,0xC6,0xC6,0xC6,0xC6,0xC6,0x7C,0x00];
    // V (86)
    f[54] = [0xC6,0xC6,0xC6,0xC6,0x6C,0x38,0x10,0x00];
    // W (87)
    f[55] = [0xC6,0xC6,0xD6,0xFE,0xFE,0xEE,0xC6,0x00];
    // X (88)
    f[56] = [0xC6,0x6C,0x38,0x38,0x38,0x6C,0xC6,0x00];
    // Y (89)
    f[57] = [0x66,0x66,0x66,0x3C,0x18,0x18,0x3C,0x00];
    // Z (90)
    f[58] = [0xFE,0xC6,0x8C,0x18,0x32,0x66,0xFE,0x00];
    // [ (91)
    f[59] = [0x3C,0x30,0x30,0x30,0x30,0x30,0x3C,0x00];
    // \ (92)
    f[60] = [0xC0,0x60,0x30,0x18,0x0C,0x06,0x02,0x00];
    // ] (93)
    f[61] = [0x3C,0x0C,0x0C,0x0C,0x0C,0x0C,0x3C,0x00];
    // ^ (94)
    f[62] = [0x10,0x38,0x6C,0xC6,0x00,0x00,0x00,0x00];
    // _ (95)
    f[63] = [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xFF];
    // ` (96)
    f[64] = [0x30,0x18,0x0C,0x00,0x00,0x00,0x00,0x00];
    // a (97)
    f[65] = [0x00,0x00,0x78,0x0C,0x7C,0xCC,0x76,0x00];
    // b (98)
    f[66] = [0xE0,0x60,0x7C,0x66,0x66,0x66,0xDC,0x00];
    // c (99)
    f[67] = [0x00,0x00,0x7C,0xC6,0xC0,0xC6,0x7C,0x00];
    // d (100)
    f[68] = [0x1C,0x0C,0x7C,0xCC,0xCC,0xCC,0x76,0x00];
    // e (101)
    f[69] = [0x00,0x00,0x7C,0xC6,0xFE,0xC0,0x7C,0x00];
    // f (102)
    f[70] = [0x1C,0x36,0x30,0x78,0x30,0x30,0x78,0x00];
    // g (103)
    f[71] = [0x00,0x00,0x76,0xCC,0xCC,0x7C,0x0C,0xF8];
    // h (104)
    f[72] = [0xE0,0x60,0x6C,0x76,0x66,0x66,0xE6,0x00];
    // i (105)
    f[73] = [0x18,0x00,0x38,0x18,0x18,0x18,0x3C,0x00];
    // j (106)
    f[74] = [0x06,0x00,0x06,0x06,0x06,0x66,0x66,0x3C];
    // k (107)
    f[75] = [0xE0,0x60,0x66,0x6C,0x78,0x6C,0xE6,0x00];
    // l (108)
    f[76] = [0x38,0x18,0x18,0x18,0x18,0x18,0x3C,0x00];
    // m (109)
    f[77] = [0x00,0x00,0xEC,0xFE,0xD6,0xD6,0xD6,0x00];
    // n (110)
    f[78] = [0x00,0x00,0xDC,0x66,0x66,0x66,0x66,0x00];
    // o (111)
    f[79] = [0x00,0x00,0x7C,0xC6,0xC6,0xC6,0x7C,0x00];
    // p (112)
    f[80] = [0x00,0x00,0xDC,0x66,0x66,0x7C,0x60,0xF0];
    // q (113)
    f[81] = [0x00,0x00,0x76,0xCC,0xCC,0x7C,0x0C,0x1E];
    // r (114)
    f[82] = [0x00,0x00,0xDC,0x76,0x60,0x60,0xF0,0x00];
    // s (115)
    f[83] = [0x00,0x00,0x7E,0xC0,0x7C,0x06,0xFC,0x00];
    // t (116)
    f[84] = [0x30,0x30,0x7C,0x30,0x30,0x36,0x1C,0x00];
    // u (117)
    f[85] = [0x00,0x00,0xCC,0xCC,0xCC,0xCC,0x76,0x00];
    // v (118)
    f[86] = [0x00,0x00,0xC6,0xC6,0xC6,0x6C,0x38,0x00];
    // w (119)
    f[87] = [0x00,0x00,0xC6,0xD6,0xD6,0xFE,0x6C,0x00];
    // x (120)
    f[88] = [0x00,0x00,0xC6,0x6C,0x38,0x6C,0xC6,0x00];
    // y (121)
    f[89] = [0x00,0x00,0xC6,0xC6,0xCE,0x76,0x06,0xFC];
    // z (122)
    f[90] = [0x00,0x00,0xFC,0x98,0x30,0x64,0xFC,0x00];
    // { (123)
    f[91] = [0x0E,0x18,0x18,0x70,0x18,0x18,0x0E,0x00];
    // | (124)
    f[92] = [0x18,0x18,0x18,0x00,0x18,0x18,0x18,0x00];
    // } (125)
    f[93] = [0x70,0x18,0x18,0x0E,0x18,0x18,0x70,0x00];
    // ~ (126)
    f[94] = [0x76,0xDC,0x00,0x00,0x00,0x00,0x00,0x00];
    // DEL placeholder (127) — not used but keeps array full
    f[95] = [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00];
    f
};

const FONT_GLYPH_W: u32 = 8;
const FONT_GLYPH_H: u32 = 8;
const FONT_COLS: u32 = 16; // glyphs per row in atlas
const FONT_ROWS: u32 = 6;  // 96 glyphs / 16 = 6 rows
const FONT_ATLAS_W: u32 = FONT_COLS * FONT_GLYPH_W; // 128
const FONT_ATLAS_H: u32 = FONT_ROWS * FONT_GLYPH_H; // 48

fn build_font_atlas(device: &wgpu::Device, queue: &wgpu::Queue) -> GpuTexture {
    let mut rgba = vec![0u8; (FONT_ATLAS_W * FONT_ATLAS_H * 4) as usize];
    for (idx, glyph) in FONT_8X8.iter().enumerate() {
        let col = (idx as u32) % FONT_COLS;
        let row = (idx as u32) / FONT_COLS;
        let ox = col * FONT_GLYPH_W;
        let oy = row * FONT_GLYPH_H;
        for y in 0..8u32 {
            let bits = glyph[y as usize];
            for x in 0..8u32 {
                if bits & (0x80 >> x) != 0 {
                    let px = ox + x;
                    let py = oy + y;
                    let off = ((py * FONT_ATLAS_W + px) * 4) as usize;
                    rgba[off] = 255;
                    rgba[off + 1] = 255;
                    rgba[off + 2] = 255;
                    rgba[off + 3] = 255;
                }
            }
        }
    }
    GpuTexture::new_2d(device, queue, FONT_ATLAS_W, FONT_ATLAS_H,
        wgpu::TextureFormat::Rgba8Unorm, &rgba, "font_atlas")
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

/// Build text mesh as overlay vertices. Returns vertex data.
/// `scale` is the pixel size per glyph (e.g. 16 for 2x scaling of 8px font).
fn build_overlay_text(text: &str, x0: f32, y0: f32, scale: f32) -> Vec<OverlayVertex> {
    let mut vertices = Vec::new();
    let mut cx = x0;
    let mut cy = y0;
    let u_step = FONT_GLYPH_W as f32 / FONT_ATLAS_W as f32;
    let v_step = FONT_GLYPH_H as f32 / FONT_ATLAS_H as f32;
    for ch in text.chars() {
        if ch == '\n' {
            cx = x0;
            cy += scale;
            continue;
        }
        let code = ch as u32;
        if code < 32 || code > 126 {
            cx += scale;
            continue;
        }
        let idx = code - 32;
        let col = idx % FONT_COLS;
        let row = idx / FONT_COLS;
        let u0 = col as f32 * u_step;
        let v0 = row as f32 * v_step;
        let u1 = u0 + u_step;
        let v1 = v0 + v_step;

        let x1 = cx + scale;
        let y1 = cy + scale;

        // Two triangles per glyph
        vertices.push(OverlayVertex { position: [cx, cy], uv: [u0, v0] });
        vertices.push(OverlayVertex { position: [x1, cy], uv: [u1, v0] });
        vertices.push(OverlayVertex { position: [cx, y1], uv: [u0, v1] });
        vertices.push(OverlayVertex { position: [cx, y1], uv: [u0, v1] });
        vertices.push(OverlayVertex { position: [x1, cy], uv: [u1, v0] });
        vertices.push(OverlayVertex { position: [x1, y1], uv: [u1, v1] });

        cx += scale;
    }
    vertices
}

impl ActionMode {
    fn help_text(&self) -> &'static str {
        match self {
            Self::TorusView => concat!(
                "[TorusView]\n",
                "Q/E:    Rotate\n",
                "Up/Dn:  Tilt\n",
                "WASD:   Pan terrain\n",
                "Space:  Center on blue\n",
                "B/V:    Next/Prev level\n",
                "N/M:    Next/Prev shader\n",
                "C:      Toggle curvature\n",
                "[/]:    Curvature +/-\n",
                "Scroll: Zoom\n",
                "Esc:    Quit\n",
                "P:      Next mode",
            ),
            Self::FreeCamera => concat!(
                "[FreeCamera]\n",
                "Arrows: Rotate X/Z\n",
                "WASD:   Move X/Y\n",
                "E/F:    Move Z\n",
                "I/O:    Rotate Y\n",
                "Scroll: Zoom\n",
                "Q:      Quit\n",
                "P:      Next mode",
            ),
            _ => concat!(
                "[Debug Mode]\n",
                "Arrows: Adjust\n",
                "Scroll: Zoom\n",
                "Q:      Quit\n",
                "P:      Next mode",
            ),
        }
    }
}

/******************************************************************************/

/// Create the bind group layout shared by all landscape shaders (group 0):
/// binding 0: mvp (mat4x4), binding 1: model_transform (mat4x4), binding 2: LandscapeParams
fn create_landscape_group0_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("landscape_group0_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Create the objects shader bind group layout (group 0): mvp + model_transform (2 bindings)
fn create_objects_group0_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("objects_group0_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Create the objects shader group 1 layout: params uniform + color storage buffer
fn create_objects_group1_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("objects_group1_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn make_storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/******************************************************************************/

struct AppConfig {
    base: Option<PathBuf>,
    level: Option<u8>,
    landtype: Option<String>,
    cpu: bool,
    cpu_full: bool,
    debug: bool,
    light: Option<(i16, i16)>,
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,

    // Landscape
    program_container: LandscapeProgramContainer,
    landscape_group0_layout: Option<wgpu::BindGroupLayout>,
    landscape_group0_bind_group: Option<wgpu::BindGroup>,
    model_main: Option<ModelEnvelop<LandscapeModel>>,

    // Selection lines
    select_pipeline: Option<wgpu::RenderPipeline>,
    objects_group0_bind_group: Option<wgpu::BindGroup>,
    objects_group1_bind_group: Option<wgpu::BindGroup>,
    model_select: Option<ModelEnvelop<DefaultModel>>,
    select_frag: i32,

    // Spawn markers (shaman sprites)
    spawn_pipeline: Option<wgpu::RenderPipeline>,
    spawn_group1_bind_group: Option<wgpu::BindGroup>,
    sprite_group1_layout: Option<wgpu::BindGroupLayout>,
    model_spawn: Option<ModelEnvelop<TexModel>>,
    spawn_cells: Vec<(f32, f32, u8)>,  // (cell_x, cell_y, tribe_index)
    sprite_texture: Option<GpuTexture>,
    shaman_atlas_info: Option<ShamanAtlasInfo>,
    // Overlay text
    overlay_pipeline: Option<wgpu::RenderPipeline>,
    overlay_bind_group: Option<wgpu::BindGroup>,
    overlay_vertex_buffer: Option<wgpu::Buffer>,
    overlay_vertex_count: u32,
    overlay_uniform_buffer: Option<GpuBuffer>,

    // Shared uniform buffers
    mvp_buffer: Option<GpuBuffer>,
    model_transform_buffer: Option<GpuBuffer>,
    landscape_params_buffer: Option<GpuBuffer>,
    select_params_buffer: Option<GpuBuffer>,

    // Storage buffers (level-dependent)
    heights_buffer: Option<GpuBuffer>,
    watdisp_buffer: Option<GpuBuffer>,

    // State
    landscape_mesh: LandscapeMeshS,
    camera: Camera,
    screen: Screen,
    mode: ActionMode,
    curvature_scale: f32,
    curvature_enabled: bool,
    zoom: f32,
    do_render: bool,
    mouse_pos: Point2<f32>,
    level_num: u8,
    sunlight: Vector4<f32>,
    wat_offset: i32,

    // Config
    config: AppConfig,
}

impl App {
    fn new(config: AppConfig) -> Self {
        let mut camera = Camera::new();
        camera.angle_x = -55;
        camera.angle_z = 65;
        camera.pos = Vector3 { x: -0.40, y: -3.70, z: 0.0 };

        let sunlight = {
            let (x, y) = config.light.unwrap_or((0x93, 0x93));
            Vector4::<f32>::new(x as f32, y as f32, 0x93 as f32, 0.0)
        };

        let landscape_mesh = LandscapeMesh::new(1.0 / 16.0, (1.0 / 16.0) * 4.0 / 1024.0);

        App {
            window: None,
            gpu: None,
            program_container: LandscapeProgramContainer::new(),
            landscape_group0_layout: None,
            landscape_group0_bind_group: None,
            model_main: None,
            select_pipeline: None,
            objects_group0_bind_group: None,
            objects_group1_bind_group: None,
            model_select: None,
            select_frag: -1,
            spawn_pipeline: None,
            spawn_group1_bind_group: None,
            sprite_group1_layout: None,
            model_spawn: None,
            spawn_cells: Vec::new(),
            sprite_texture: None,
            shaman_atlas_info: None,
            overlay_pipeline: None,
            overlay_bind_group: None,
            overlay_vertex_buffer: None,
            overlay_vertex_count: 0,
            overlay_uniform_buffer: None,
            mvp_buffer: None,
            model_transform_buffer: None,
            landscape_params_buffer: None,
            select_params_buffer: None,
            heights_buffer: None,
            watdisp_buffer: None,
            landscape_mesh,
            camera,
            screen: Screen { width: 800, height: 600 },
            mode: ActionMode::TorusView,
            curvature_scale: 0.0512,
            curvature_enabled: true,
            zoom: 1.0,
            do_render: true,
            mouse_pos: Point2::<f32>::new(0.0, 0.0),
            level_num: config.level.unwrap_or(1),
            sunlight,
            wat_offset: -1,
            config,
        }
    }

    fn build_landscape_params(&self) -> LandscapeUniformData {
        let shift = self.landscape_mesh.get_shift_vector();
        LandscapeUniformData {
            level_shift: [shift.x, shift.y, shift.z, shift.w],
            height_scale: self.landscape_mesh.height_scale(),
            step: self.landscape_mesh.step(),
            width: self.landscape_mesh.width() as i32,
            selected_frag: self.select_frag,
            selected_color: [1.0, 0.0, 0.0, 0.0],
            sunlight: [self.sunlight.x, self.sunlight.y, self.sunlight.z, self.sunlight.w],
            wat_offset: self.wat_offset,
            curvature_scale: if self.curvature_enabled { self.curvature_scale } else { 0.0 },
            camera_focus: {
                let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
                [center, center]
            },
            viewport_radius: {
                let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
                center * 0.9
            },
            _pad2: [0.0; 3],
        }
    }

    fn update_level(&mut self) {
        let base = self.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type = self.config.landtype.as_deref();
        let level_res = LevelRes::new(&base, self.level_num, level_type);

        self.landscape_mesh.set_heights(&level_res.landscape.height);

        {
            let gpu = self.gpu.as_ref().unwrap();

            // Update heights buffer
            let landscape = level_res.landscape.make_shores();
            let heights_vec = landscape.to_vec();
            let heights_bytes: &[u8] = bytemuck::cast_slice(&heights_vec);
            let heights_buffer = GpuBuffer::new_storage(&gpu.device, heights_bytes, "heights_buffer");
            self.heights_buffer = Some(heights_buffer);

            // Update watdisp buffer
            let watdisp_vec: Vec<u32> = level_res.params.watdisp.iter().map(|v| *v as u32).collect();
            let watdisp_bytes: &[u8] = bytemuck::cast_slice(&watdisp_vec);
            let watdisp_buffer = GpuBuffer::new_storage(&gpu.device, watdisp_bytes, "watdisp_buffer");
            self.watdisp_buffer = Some(watdisp_buffer);
        }

        // Rebuild all landscape variants
        self.rebuild_landscape_variants(&level_res);

        // Rebuild sprite atlas with new palette
        if let Some(ref gpu) = self.gpu {
            if let Some((w, h, data, info)) = build_shaman_atlas(&base, &level_res.params.palette) {
                if let Some(ref sprite_tex) = self.sprite_texture {
                    if sprite_tex.size.width == w && sprite_tex.size.height == h {
                        sprite_tex.update(&gpu.queue, &data);
                    } else {
                        let new_tex = GpuTexture::new_2d(
                            &gpu.device, &gpu.queue, w, h,
                            wgpu::TextureFormat::Rgba8UnormSrgb, &data, "shaman_atlas",
                        );
                        let sampler = GpuTexture::create_sampler(&gpu.device, true);
                        let layout = self.sprite_group1_layout.as_ref().unwrap();
                        self.spawn_group1_bind_group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("spawn_sprite_bg1"),
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&new_tex.view) },
                                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                            ],
                        }));
                        self.sprite_texture = Some(new_tex);
                    }
                }
                self.shaman_atlas_info = Some(info);
            }
        }

        // Rebuild spawn markers
        self.spawn_cells = extract_spawn_cells(&level_res);
        self.rebuild_spawn_model();
    }

    fn rebuild_spawn_model(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let cs = if self.curvature_enabled { self.curvature_scale } else { 0.0 };
            self.model_spawn = Some(build_spawn_model(
                &gpu.device, &self.spawn_cells, &self.landscape_mesh, cs,
                self.camera.angle_x, self.camera.angle_z,
                self.shaman_atlas_info.as_ref(),
            ));
        }
    }

    fn rebuild_overlay(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let text = self.mode.help_text();
            let verts = build_overlay_text(text, 10.0, 10.0, 16.0);
            self.overlay_vertex_count = verts.len() as u32;
            let data: &[u8] = bytemuck::cast_slice(&verts);
            let buf = GpuBuffer::new_vertex(&gpu.device, data, "overlay_vb");
            self.overlay_vertex_buffer = Some(buf.buffer);
            // Update screen size uniform
            let screen_data = [self.screen.width as f32, self.screen.height as f32, 0.0f32, 0.0f32];
            if let Some(ref buf) = self.overlay_uniform_buffer {
                buf.update(&gpu.queue, 0, bytemuck::bytes_of(&screen_data));
            }
        }
    }

    fn rebuild_landscape_variants(&mut self, level_res: &LevelRes) {
        let gpu = self.gpu.as_ref().unwrap();
        let device = &gpu.device;
        let group0_layout = self.landscape_group0_layout.as_ref().unwrap();
        let heights_buffer = self.heights_buffer.as_ref().unwrap();
        let watdisp_buffer = self.watdisp_buffer.as_ref().unwrap();

        let vertex_layouts = LandscapeModel::vertex_buffer_layouts();
        let surface_format = gpu.surface_format();

        self.program_container = LandscapeProgramContainer::new();

        // CPU palette index variant
        if self.config.cpu {
            let land_texture = make_texture_land(level_res, None);
            let size = (level_res.landscape.land_size() * 32) as u32;

            let cpu_tex = GpuTexture::new_2d(
                device, &gpu.queue, size, size,
                wgpu::TextureFormat::R8Uint, &land_texture, "cpu_land_texture",
            );

            let palette_packed = pack_palette_rgba(&level_res.params.palette);
            let palette_bytes: &[u8] = bytemuck::cast_slice(&palette_packed);
            let palette_buf = GpuBuffer::new_storage(device, palette_bytes, "palette_buffer");

            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_cpu_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    make_storage_entry(3), // palette
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_cpu_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&cpu_tex.view) },
                    wgpu::BindGroupEntry { binding: 3, resource: palette_buf.buffer.as_entire_binding() },
                ],
            });

            let shader_source = include_str!("../shaders/landscape_cpu.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_cpu");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }

        // CPU full texture variant
        if self.config.cpu_full {
            let land_texture = make_texture_land(level_res, None);
            let size = (level_res.landscape.land_size() * 32) as u32;
            let full_tex_data = draw_texture_u8(&level_res.params.palette, size as usize, &land_texture);

            // draw_texture_u8 returns RGB data; need RGBA for wgpu
            let rgba_data = rgb_to_rgba(&full_tex_data);
            let full_tex = GpuTexture::new_2d(
                device, &gpu.queue, size, size,
                wgpu::TextureFormat::Rgba8Unorm, &rgba_data, "cpu_full_land_texture",
            );
            let sampler = GpuTexture::create_sampler(device, false);

            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_full_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_full_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&full_tex.view) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });

            let shader_source = include_str!("../shaders/landscape_full.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_full");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }

        // Main GPU landscape
        {
            let palette_packed = pack_palette_rgba(&level_res.params.palette);
            let palette_bytes: &[u8] = bytemuck::cast_slice(&palette_packed);
            let palette_buf = GpuBuffer::new_storage(device, palette_bytes, "main_palette_buffer");

            let disp_vec: Vec<i32> = level_res.params.disp0.iter().map(|v| *v as i32).collect();
            let disp_bytes: &[u8] = bytemuck::cast_slice(&disp_vec);
            let disp_buf = GpuBuffer::new_storage(device, disp_bytes, "disp_buffer");

            let bigf_vec: Vec<u32> = level_res.params.bigf0.iter().map(|v| *v as u32).collect();
            let bigf_bytes: &[u8] = bytemuck::cast_slice(&bigf_vec);
            let bigf_buf = GpuBuffer::new_storage(device, bigf_bytes, "bigf_buffer");

            let sla_vec: Vec<u32> = level_res.params.static_landscape_array.iter().map(|v| *v as u32).collect();
            let sla_bytes: &[u8] = bytemuck::cast_slice(&sla_vec);
            let sla_buf = GpuBuffer::new_storage(device, sla_bytes, "sla_buffer");

            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_main_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                    make_storage_entry(2), // palette
                    make_storage_entry(3), // disp
                    make_storage_entry(4), // bigf
                    make_storage_entry(5), // sla
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_main_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: palette_buf.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: disp_buf.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: bigf_buf.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 5, resource: sla_buf.buffer.as_entire_binding() },
                ],
            });

            let shader_source = include_str!("../shaders/landscape.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_main");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }

        // Gradient variant
        {
            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_grad_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_grad_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                ],
            });

            let shader_source = include_str!("../shaders/landscape_grad.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_grad");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }
    }

    fn render(&mut self) {
        let gpu = self.gpu.as_ref().unwrap();

        // Update uniforms
        let mvp = MVP::with_zoom(&self.screen, &self.camera, self.zoom);
        let mvp_m = mvp.projection * mvp.view * mvp.transform;
        let mvp_raw: TransformRaw = mvp_m.into();
        self.mvp_buffer.as_ref().unwrap().update(&gpu.queue, 0, bytemuck::bytes_of(&mvp_raw));

        // Update model transform
        if let Some(ref model_main) = self.model_main {
            model_main.write_transform(&gpu.queue, &self.model_transform_buffer.as_ref().unwrap().buffer, 0);
        }

        // Update landscape params
        let params = self.build_landscape_params();
        self.landscape_params_buffer.as_ref().unwrap().update(&gpu.queue, 0, bytemuck::bytes_of(&params));

        // Update selection uniform
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct ObjectParams { selected_frag: i32, num_colors: i32 }
        let obj_params = ObjectParams { selected_frag: self.select_frag, num_colors: obj_colors().len() as i32 };
        self.select_params_buffer.as_ref().unwrap().update(&gpu.queue, 0, bytemuck::bytes_of(&obj_params));

        // Update select model vertex data
        if let Some(ref model_select) = self.model_select {
            model_select.write_transform(&gpu.queue, &self.model_transform_buffer.as_ref().unwrap().buffer, 0);
        }

        let output = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of GPU memory"),
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gpu.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Draw landscape
            if let Some(variant) = self.program_container.current() {
                render_pass.set_pipeline(&variant.pipeline);
                render_pass.set_bind_group(0, self.landscape_group0_bind_group.as_ref().unwrap(), &[]);
                render_pass.set_bind_group(1, &variant.bind_group_1, &[]);
                if let Some(ref model_main) = self.model_main {
                    model_main.draw(&mut render_pass);
                }
            }

            // Draw spawn markers (shaman sprites)
            if let (Some(ref spawn_pipeline), Some(ref spawn_bg1)) =
                (&self.spawn_pipeline, &self.spawn_group1_bind_group)
            {
                render_pass.set_pipeline(spawn_pipeline);
                render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                render_pass.set_bind_group(1, spawn_bg1, &[]);
                if let Some(ref model_spawn) = self.model_spawn {
                    model_spawn.draw(&mut render_pass);
                }
            }

            // Draw selection lines
            if let Some(ref select_pipeline) = self.select_pipeline {
                render_pass.set_pipeline(select_pipeline);
                render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                render_pass.set_bind_group(1, self.objects_group1_bind_group.as_ref().unwrap(), &[]);
                if let Some(ref model_select) = self.model_select {
                    model_select.draw(&mut render_pass);
                }
            }
        }

        // Overlay text pass (no depth, load existing color)
        if let (Some(ref pipeline), Some(ref bg), Some(ref vb)) =
            (&self.overlay_pipeline, &self.overlay_bind_group, &self.overlay_vertex_buffer)
        {
            if self.overlay_vertex_count > 0 {
                // Update screen size uniform for overlay
                let screen_data = [self.screen.width as f32, self.screen.height as f32, 0.0f32, 0.0f32];
                if let Some(ref buf) = self.overlay_uniform_buffer {
                    buf.update(&gpu.queue, 0, bytemuck::bytes_of(&screen_data));
                }

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("overlay_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, bg, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.draw(0..self.overlay_vertex_count, 0..1);
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

/// Pack palette from RGBA u8 slices into packed u32 for storage buffer.
fn pack_palette_rgba(palette: &[u8]) -> Vec<u32> {
    palette.chunks(4).map(|c| {
        let r = c.get(0).copied().unwrap_or(0) as u32;
        let g = c.get(1).copied().unwrap_or(0) as u32;
        let b = c.get(2).copied().unwrap_or(0) as u32;
        let a = c.get(3).copied().unwrap_or(0) as u32;
        r | (g << 8) | (b << 16) | (a << 24)
    }).collect()
}

/// Convert RGB byte data to RGBA byte data (adding alpha=255).
fn rgb_to_rgba(rgb: &[u8]) -> Vec<u8> {
    let pixel_count = rgb.len() / 3;
    let mut rgba = Vec::with_capacity(pixel_count * 4);
    for chunk in rgb.chunks(3) {
        rgba.push(chunk[0]);
        rgba.push(chunk[1]);
        rgba.push(chunk[2]);
        rgba.push(255);
    }
    rgba
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Faithful"))
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        let base = self.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type = self.config.landtype.as_deref();
        let level_res = LevelRes::new(&base, self.level_num, level_type);

        self.landscape_mesh.set_heights(&level_res.landscape.height);

        // Heights storage buffer
        let landscape = level_res.landscape.make_shores();
        let heights_vec = landscape.to_vec();
        let heights_bytes: &[u8] = bytemuck::cast_slice(&heights_vec);
        let heights_buffer = GpuBuffer::new_storage(device, heights_bytes, "heights_buffer");

        // Watdisp storage buffer
        let watdisp_vec: Vec<u32> = level_res.params.watdisp.iter().map(|v| *v as u32).collect();
        let watdisp_bytes: &[u8] = bytemuck::cast_slice(&watdisp_vec);
        let watdisp_buffer = GpuBuffer::new_storage(device, watdisp_bytes, "watdisp_buffer");

        // Shared uniform buffers
        let mvp_buffer = GpuBuffer::new_uniform(device, 64, "mvp_buffer");
        let model_transform_buffer = GpuBuffer::new_uniform(device, 64, "model_transform_buffer");
        let landscape_params_buffer = GpuBuffer::new_uniform_init(
            device,
            bytemuck::bytes_of(&self.build_landscape_params()),
            "landscape_params_buffer",
        );

        // Landscape group 0 layout and bind group
        let landscape_group0_layout = create_landscape_group0_layout(device);
        let landscape_group0_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("landscape_group0_bg"),
            layout: &landscape_group0_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: mvp_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: model_transform_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: landscape_params_buffer.buffer.as_entire_binding() },
            ],
        });

        // Objects (selection lines) setup
        let objects_group0_layout = create_objects_group0_layout(device);
        let objects_group0_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("objects_group0_bg"),
            layout: &objects_group0_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: mvp_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: model_transform_buffer.buffer.as_entire_binding() },
            ],
        });

        // Objects group 1: params uniform + color storage
        let objects_group1_layout = create_objects_group1_layout(device);

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct ObjectParams { selected_frag: i32, num_colors: i32 }

        let colors = obj_colors();
        let obj_params = ObjectParams { selected_frag: self.select_frag, num_colors: colors.len() as i32 };
        let select_params_buffer = GpuBuffer::new_uniform_init(device, bytemuck::bytes_of(&obj_params), "select_params_buffer");

        // Pack colors as vec4<u32> (RGBA, each channel widened to u32)
        let color_data: Vec<[u32; 4]> = colors.iter().map(|c| {
            [c.x as u32, c.y as u32, c.z as u32, 0u32]
        }).collect();
        let color_bytes: &[u8] = bytemuck::cast_slice(&color_data);
        let color_buffer = GpuBuffer::new_storage(device, color_bytes, "obj_color_buffer");

        let objects_group1_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("objects_group1_bg"),
            layout: &objects_group1_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: select_params_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: color_buffer.buffer.as_entire_binding() },
            ],
        });

        // Selection lines pipeline (LineList topology)
        let select_shader_source = include_str!("../shaders/objects.wgsl");
        let select_vertex_layouts = DefaultModel::vertex_buffer_layouts();
        let select_pipeline = create_pipeline(
            device, select_shader_source, &select_vertex_layouts,
            &[&objects_group0_layout, &objects_group1_layout],
            gpu.surface_format(), true,
            wgpu::PrimitiveTopology::LineList,
            "objects_pipeline",
        );

        // Landscape model
        let model_main = make_landscape_model(device, &self.landscape_mesh);

        // Selection model (2 vertices for a ray line)
        let model_select = {
            let mut model: DefaultModel = MeshModel::new();
            model.push_vertex(Vector3::new(0.0, 0.0, 0.0));
            model.push_vertex(Vector3::new(0.0, 0.0, 0.0));
            let m = vec![(RenderType::Lines, model)];
            ModelEnvelop::<DefaultModel>::new(device, m)
        };

        // Shaman sprite atlas and pipeline
        let sprite_group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite_group1_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let spawn_shader_source = include_str!("../shaders/shaman_sprite.wgsl");
        let spawn_vertex_layouts = TexModel::vertex_buffer_layouts();
        let spawn_pipeline = create_pipeline(
            device, spawn_shader_source, &spawn_vertex_layouts,
            &[&objects_group0_layout, &sprite_group1_layout],
            gpu.surface_format(), true,
            wgpu::PrimitiveTopology::TriangleList,
            "shaman_sprite_pipeline",
        );

        // Load shaman sprite atlas (5 dirs × 8 frames, palette-converted to RGBA sRGB)
        let (sprite_texture, shaman_atlas_info) = match build_shaman_atlas(&base, &level_res.params.palette) {
            Some((w, h, data, info)) => {
                let tex = GpuTexture::new_2d(device, &gpu.queue, w, h,
                    wgpu::TextureFormat::Rgba8UnormSrgb, &data, "shaman_atlas");
                (Some(tex), Some(info))
            }
            None => (None, None),
        };
        let sprite_sampler = GpuTexture::create_sampler(device, true);

        let spawn_group1_bind_group = if let Some(ref tex) = sprite_texture {
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("spawn_sprite_bg1"),
                layout: &sprite_group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex.view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sprite_sampler) },
                ],
            }))
        } else {
            log::warn!("HSPR0-0.DAT not found or frame 7578 missing — spawn sprites disabled");
            None
        };

        // Empty spawn model (will be populated when level loads)
        let model_spawn = {
            let model: TexModel = MeshModel::new();
            let m = vec![(RenderType::Triangles, model)];
            ModelEnvelop::<TexModel>::new(device, m)
        };

        // Overlay text pipeline
        let overlay_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay_bg_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let font_atlas = build_font_atlas(device, &gpu.queue);
        let font_sampler = GpuTexture::create_sampler(device, true);

        let screen_data = [self.screen.width as f32, self.screen.height as f32, 0.0f32, 0.0f32];
        let overlay_uniform_buffer = GpuBuffer::new_uniform_init(
            device, bytemuck::bytes_of(&screen_data), "overlay_uniforms");

        let overlay_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay_bg"),
            layout: &overlay_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: overlay_uniform_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&font_atlas.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&font_sampler) },
            ],
        });

        let overlay_shader_source = include_str!("../shaders/overlay_text.wgsl");
        let overlay_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay_shader"),
            source: wgpu::ShaderSource::Wgsl(overlay_shader_source.into()),
        });
        let overlay_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay_pipeline_layout"),
            bind_group_layouts: &[&overlay_bind_group_layout],
            immediate_size: 0,
        });
        let overlay_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay_pipeline"),
            layout: Some(&overlay_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &overlay_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<OverlayVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 8, shader_location: 1 },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &overlay_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.surface_format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Store everything
        self.heights_buffer = Some(heights_buffer);
        self.watdisp_buffer = Some(watdisp_buffer);
        self.mvp_buffer = Some(mvp_buffer);
        self.model_transform_buffer = Some(model_transform_buffer);
        self.landscape_params_buffer = Some(landscape_params_buffer);
        self.select_params_buffer = Some(select_params_buffer);
        self.landscape_group0_layout = Some(landscape_group0_layout);
        self.landscape_group0_bind_group = Some(landscape_group0_bind_group);
        self.objects_group0_bind_group = Some(objects_group0_bind_group);
        self.objects_group1_bind_group = Some(objects_group1_bind_group);
        self.select_pipeline = Some(select_pipeline);
        self.spawn_pipeline = Some(spawn_pipeline);
        self.spawn_group1_bind_group = spawn_group1_bind_group;
        self.sprite_group1_layout = Some(sprite_group1_layout);
        self.sprite_texture = sprite_texture;
        self.shaman_atlas_info = shaman_atlas_info;
        self.model_spawn = Some(model_spawn);
        self.model_main = Some(model_main);
        self.model_select = Some(model_select);
        self.overlay_pipeline = Some(overlay_pipeline);
        self.overlay_bind_group = Some(overlay_bind_group);
        self.overlay_uniform_buffer = Some(overlay_uniform_buffer);

        self.gpu = Some(gpu);

        // Build landscape variants (needs self.gpu, heights_buffer, etc.)
        let base2 = self.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type2 = self.config.landtype.as_deref();
        let level_res2 = LevelRes::new(&base2, self.level_num, level_type2);
        self.rebuild_landscape_variants(&level_res2);

        // Build spawn markers for initial level
        self.spawn_cells = extract_spawn_cells(&level_res2);
        self.rebuild_spawn_model();

        // Build initial overlay text
        self.rebuild_overlay();

        self.do_render = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.screen.width = physical_size.width;
                self.screen.height = physical_size.height;
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(physical_size);
                }
                self.rebuild_overlay();
                self.do_render = true;
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = Point2::<f32>::new(position.x as f32, position.y as f32);
            },
            WindowEvent::MouseInput { state, .. } => {
                if state == ElementState::Pressed {
                    let (v1, v2) = screen_to_scene_zoom(&self.screen, &self.camera, &self.mouse_pos, self.zoom);
                    if let Some(ref mut model_select) = self.model_select {
                        if let Some(m) = model_select.get(0) {
                            m.model.set_vertex(0, v1);
                            m.model.set_vertex(1, v2);
                        }
                        model_select.update_model_buffers(&self.gpu.as_ref().unwrap().device, 0);
                    }

                    let mvp_transform = self.model_main.as_mut()
                        .and_then(|mm| mm.get(0))
                        .map(|m| m.transform())
                        .unwrap_or(Matrix4::identity());
                    let iter = self.landscape_mesh.iter();
                    match intersect_iter(iter, &mvp_transform, v1, v2) {
                        Some((n, _)) => self.select_frag = n as i32,
                        None => self.select_frag = -1,
                    }
                    self.do_render = true;
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
                };
                self.zoom *= 1.1_f32.powf(scroll_y);
                self.zoom = self.zoom.clamp(0.3, 5.0);
                self.do_render = true;
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::Escape => {
                                event_loop.exit();
                                return;
                            },
                            KeyCode::KeyR => {
                                self.camera.angle_x = 0;
                                self.camera.angle_y = 0;
                                self.camera.angle_z = 0;
                                self.camera.pos = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
                            },
                            KeyCode::KeyT => {
                                self.camera.angle_x = -90;
                            },
                            KeyCode::KeyN => {
                                self.program_container.next();
                            },
                            KeyCode::KeyM => {
                                self.program_container.prev();
                            },
                            KeyCode::KeyB => {
                                self.level_num = (self.level_num + 1) % 26;
                                if self.level_num == 0 { self.level_num = 1; }
                                self.update_level();
                            },
                            KeyCode::KeyV => {
                                self.level_num = if self.level_num == 1 { 25 } else { self.level_num - 1 };
                                self.update_level();
                            },
                            KeyCode::KeyL => {
                                self.landscape_mesh.shift_y(1);
                            },
                            KeyCode::KeyH => {
                                self.landscape_mesh.shift_y(-1);
                            },
                            KeyCode::KeyJ => {
                                self.landscape_mesh.shift_x(1);
                            },
                            KeyCode::KeyK => {
                                self.landscape_mesh.shift_x(-1);
                            },
                            KeyCode::KeyY => {
                                self.sunlight.x -= 1.0;
                                self.sunlight.y -= 1.0;
                                log::debug!("sunlight = {:?}", self.sunlight);
                            },
                            KeyCode::KeyZ => {
                                self.wat_offset += 1;
                            },
                            KeyCode::KeyC => {
                                self.curvature_enabled = !self.curvature_enabled;
                                log::info!("curvature {}", if self.curvature_enabled { "on" } else { "off" });
                                self.rebuild_spawn_model();
                            },
                            KeyCode::BracketRight => {
                                self.curvature_scale *= 1.2;
                                log::info!("curvature_scale = {:.6}", self.curvature_scale);
                                self.rebuild_spawn_model();
                            },
                            KeyCode::BracketLeft => {
                                self.curvature_scale *= 0.8;
                                log::info!("curvature_scale = {:.6}", self.curvature_scale);
                                self.rebuild_spawn_model();
                            },
                            KeyCode::Space => {
                                // Center on blue shaman (tribe 0)
                                if let Some(&(cx, cy, _)) = self.spawn_cells.iter().find(|(_, _, t)| *t == 0) {
                                    let n = self.landscape_mesh.width() as i32;
                                    let sx = ((cx as i32 - n / 2) % n + n) % n;
                                    let sy = ((cy as i32 - n / 2) % n + n) % n;
                                    self.landscape_mesh.set_shift(sx as usize, sy as usize);
                                    self.rebuild_spawn_model();
                                }
                            },
                            _ => {
                                let prev_shift = self.landscape_mesh.get_shift_vector();
                                let prev_angle_z = self.camera.angle_z;
                                let prev_angle_x = self.camera.angle_x;
                                let prev_mode = self.mode;
                                self.mode.process_key(key, &mut self.camera, &mut self.landscape_mesh);
                                let new_shift = self.landscape_mesh.get_shift_vector();
                                let shift_changed = new_shift != prev_shift;
                                if shift_changed || self.camera.angle_z != prev_angle_z || self.camera.angle_x != prev_angle_x {
                                    self.rebuild_spawn_model();
                                }
                                if self.mode != prev_mode {
                                    self.rebuild_overlay();
                                }
                                if self.mode == ActionMode::FreeCamera {
                                    println!("camera: angle_x={} angle_y={} angle_z={} pos=({:.2}, {:.2}, {:.2})",
                                        self.camera.angle_x, self.camera.angle_y, self.camera.angle_z,
                                        self.camera.pos.x, self.camera.pos.y, self.camera.pos.z);
                                }
                            },
                        }
                        self.do_render = true;
                    }
                }
            },
            WindowEvent::RedrawRequested => {
                if self.do_render && self.gpu.is_some() {
                    self.render();
                    self.do_render = false;
                }
            },
            _ => (),
        }
        if self.do_render {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

/******************************************************************************/

fn parse_light(s: &str) -> Option<(i16, i16)> {
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() != 2 { return None; }
    Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
}

fn cli() -> Command {
    let args = [
        Arg::new("base")
            .long("base")
            .action(ArgAction::Set)
            .value_name("BASE_PATH")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to POP3 directory"),
        Arg::new("level")
            .long("level")
            .action(ArgAction::Set)
            .value_name("LEVEL")
            .value_parser(clap::value_parser!(u8).range(1..255))
            .help("Level number"),
        Arg::new("landtype")
            .long("landtype")
            .action(ArgAction::Set)
            .value_name("LAND_TYPE")
            .value_parser(clap::builder::StringValueParser::new())
            .help("Override land type"),
        Arg::new("cpu")
            .long("cpu")
            .action(ArgAction::SetTrue)
            .help("Enable CPU texture rendering"),
        Arg::new("cpu-full")
            .long("cpu-full")
            .action(ArgAction::SetTrue)
            .help("Enable full CPU texture rendering"),
        Arg::new("light")
            .long("light")
            .action(ArgAction::Set)
            .help("Light configuration x;y"),
        Arg::new("debug")
            .long("debug")
            .action(ArgAction::SetTrue)
            .help("Enable debug printing"),
    ];
    Command::new("faithful")
        .about("POP3 wgpu renderer")
        .args(&args)
}

fn main() {
    let matches = cli().get_matches();

    let config = AppConfig {
        base: matches.get_one("base").cloned(),
        level: matches.get_one("level").copied(),
        landtype: matches.get_one("landtype").cloned(),
        cpu: matches.get_flag("cpu"),
        cpu_full: matches.get_flag("cpu-full"),
        debug: matches.get_flag("debug"),
        light: matches.get_one::<String>("light").and_then(|s| parse_light(s)),
    };

    let log_level: &str = if config.debug { "debug" } else { "info" };
    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", log_level)
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(config);
    event_loop.run_app(&mut app).unwrap();
}

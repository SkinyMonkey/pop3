use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, ArgAction, Command};

use cgmath::{Point2, Point3, Vector2, Vector3, Vector4, Matrix4, SquareMatrix};

use pop3::model::{VertexModel, MeshModel};
use pop3::default_model::DefaultModel;
use pop3::tex_model::{TexModel, TexVertex};
use pop3::color_model::{ColorModel, ColorVertex};
use pop3::view::*;

use pop3::pop::psfb::ContainerPSFB;
use pop3::pop::types::BinDeserializer;

use pop3::intersect::intersect_iter;

use pop3::landscape::{LandscapeMesh, LandscapeModel};
use pop3::pop::level::{LevelRes, ObjectPaths};
use pop3::pop::units::{ModelType, object_3d_index};
use pop3::pop::objects::{Object3D, Shape, mk_pop_object};
use pop3::pop::bl320::make_bl320_texture_rgba;
use pop3::pop::landscape::{make_texture_land, draw_texture_u8};

use pop3::unit_control::{UnitCoordinator, DragState, Unit};
use pop3::unit_control::coords::{cell_to_world, triangle_to_cell, project_to_screen, nearest_screen_hit};

use pop3::gpu::context::GpuContext;
use pop3::gpu::pipeline::create_pipeline;
use pop3::gpu::buffer::GpuBuffer;
use pop3::gpu::texture::GpuTexture;
use pop3::envelop::*;

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

fn process_key(key: KeyCode, camera: &mut Camera, landscape_mesh: &mut LandscapeMeshS) {
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
                KeyCode::KeyW => (0.0f32, 1.0f32),
                KeyCode::KeyS => (0.0, -1.0),
                KeyCode::KeyA => (-1.0, 0.0),
                KeyCode::KeyD => (1.0, 0.0),
                _ => unreachable!(),
            };
            // Project screen direction through orbit camera orientation.
            // Camera forward on XY = (-sin(az), -cos(az)),
            // camera right on XY = (-cos(az), sin(az)).
            let az = (camera.angle_z as f32).to_radians();
            let gx = -dx * az.cos() - dy * az.sin();
            let gy =  dx * az.sin() - dy * az.cos();
            landscape_mesh.shift_x(gx.round() as i32);
            landscape_mesh.shift_y(gy.round() as i32);
        },
        _ => (),
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
    _pad_width: i32,
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

/// Landscape model transform: world = LANDSCAPE_SCALE * model + LANDSCAPE_OFFSET.
const LANDSCAPE_SCALE: f32 = 2.5;
const LANDSCAPE_OFFSET: f32 = -2.0;

fn make_landscape_model(device: &wgpu::Device, landscape_mesh: &LandscapeMeshS) -> ModelEnvelop<LandscapeModel> {
    let mut model: LandscapeModel = MeshModel::new();
    landscape_mesh.to_model(&mut model);
    log::debug!("Landscape mesh - vertices={:?}, indices={:?}", model.vertices.len(), model.indices.len());
    let m = vec![(RenderType::Triangles, model)];
    let mut model_main = ModelEnvelop::<LandscapeModel>::new(device, m);
    if let Some(m) = model_main.get(0) {
        m.location.x = LANDSCAPE_OFFSET;
        m.location.y = LANDSCAPE_OFFSET;
        m.scale = LANDSCAPE_SCALE;
    }
    eprintln!("[landscape] model transform: location=({0},{0},0) scale={1}", LANDSCAPE_OFFSET, LANDSCAPE_SCALE);
    model_main
}

/// Extract spawn cell coordinates from level data.
/// Returns (cell_x, cell_y, tribe_index) for each tribe's shaman (Person subtype 7).
fn extract_spawn_cells(level_res: &LevelRes) -> Vec<(f32, f32, u8)> {
    let mut found = [false; 4];
    let mut cells = Vec::new();
    let n = level_res.landscape.land_size() as f32;
    for unit in &level_res.units {
        let tribe = unit.tribe_index() as usize;
        if unit.model == 1 && unit.subtype == 7 && tribe < 4 && !found[tribe] {
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
    let sprite_h = step * 0.6;
    let half_w = if let Some(info) = atlas_info {
        let aspect = info.frame_width as f32 / info.frame_height as f32;
        sprite_h * aspect / 2.0
    } else {
        step * 0.6
    };

    let center = (w - 1.0) * step / 2.0;

    // Billboard orientation: extract screen-aligned right and up vectors from the
    // orbit camera's view matrix, matching MVP::with_zoom exactly.
    let az = (angle_z as f32).to_radians();
    let ax = (angle_x as f32).to_radians();
    let eye = Point3::new(
        center + ax.cos() * az.sin(),
        center + ax.cos() * az.cos(),
        -ax.sin(),
    );
    let target = Point3::new(center, center, 0.0);
    let view = Matrix4::look_at_rh(eye, target, Vector3::new(0.0, 0.0, 1.0));
    // World-space right = first row of view matrix, up = second row
    let right = Vector3::new(view.x.x, view.y.x, view.z.x);
    let up = Vector3::new(view.x.y, view.y.y, view.z.y);

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
// Level object markers

struct LevelObject {
    cell_x: f32,
    cell_y: f32,
    model_type: ModelType,
    #[allow(dead_code)]
    subtype: u8,
    tribe_index: u8,
    angle: u32,
}

fn extract_level_objects(level_res: &LevelRes) -> Vec<LevelObject> {
    let n = level_res.landscape.land_size() as f32;
    let mut objects = Vec::new();
    for unit in &level_res.units {
        let model_type = match unit.model_type() {
            Some(mt) if mt.is_visible() => mt,
            _ => continue,
        };
        if unit.loc_x() == 0 && unit.loc_y() == 0 {
            continue;
        }
        let bevy_x = ((unit.loc_x() >> 8) / 2) as f32 + 0.5;
        let bevy_z = ((unit.loc_y() >> 8) / 2) as f32 + 0.5;
        let cell_x = bevy_z;
        let cell_y = (n - 1.0) - bevy_x;
        eprintln!("[extract] type={:?} subtype={} tribe={} angle={} loc=({},{})",
            model_type, unit.subtype, unit.tribe_index(), unit.angle(),
            unit.loc_x(), unit.loc_y());
        objects.push(LevelObject {
            cell_x,
            cell_y,
            model_type,
            subtype: unit.subtype,
            tribe_index: unit.tribe_index(),
            angle: unit.angle(),
        });
    }
    objects
}

fn tribe_marker_color(tribe_index: u8) -> [f32; 3] {
    match tribe_index {
        0 => [0.2, 0.4, 1.0],   // Blue
        1 => [1.0, 0.2, 0.2],   // Red
        2 => [1.0, 1.0, 0.2],   // Yellow
        3 => [0.2, 1.0, 0.2],   // Green
        _ => [0.9, 0.9, 0.9],   // Unowned (tribe 255 = no owner)
    }
}

fn object_marker_color(model_type: &ModelType, tribe_index: u8) -> [f32; 3] {
    match model_type {
        // Tribe-owned units: use tribe color
        ModelType::Person | ModelType::Building | ModelType::Creature | ModelType::Vehicle
            if tribe_index < 4 => tribe_marker_color(tribe_index),
        // Unowned persons (wildmen): brown
        ModelType::Person   => [0.6, 0.4, 0.2],
        // Unowned buildings: dark orange
        ModelType::Building => [0.7, 0.5, 0.1],
        // Unowned creatures: magenta
        ModelType::Creature => [0.8, 0.2, 0.8],
        // Unowned vehicles: cyan
        ModelType::Vehicle  => [0.2, 0.6, 0.8],
        ModelType::Scenery  => [0.2, 0.5, 0.1],   // Dark green
        ModelType::General  => [1.0, 0.5, 0.0],   // Orange
        ModelType::Shape    => [0.5, 0.5, 0.5],   // Gray
        _ => [1.0, 1.0, 1.0],
    }
}

fn build_object_markers(
    device: &wgpu::Device, objects: &[LevelObject],
    landscape: &LandscapeMesh<128>, curvature_scale: f32,
    angle_x: i16, angle_z: i16,
) -> ModelEnvelop<ColorModel> {
    let mut model: ColorModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let center = (w - 1.0) * step / 2.0;

    let az = (angle_z as f32).to_radians();
    let ax = (angle_x as f32).to_radians();
    let eye = Point3::new(
        center + ax.cos() * az.sin(),
        center + ax.cos() * az.cos(),
        -ax.sin(),
    );
    let target = Point3::new(center, center, 0.0);
    let view = Matrix4::look_at_rh(eye, target, Vector3::new(0.0, 0.0, 1.0));
    let right = Vector3::new(view.x.x, view.y.x, view.z.x);
    let up = Vector3::new(view.x.y, view.y.y, view.z.y);

    for obj in objects {
        // Skip objects that have 3D meshes
        if object_3d_index(&obj.model_type, obj.subtype, obj.tribe_index).is_some() {
            continue;
        }

        let vis_x = ((obj.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((obj.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;

        let ix = (obj.cell_x as usize).min(127);
        let iy = (obj.cell_y as usize).min(127);
        let gz = landscape.height_at(ix, iy) as f32 * height_scale;

        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * curvature_scale;
        let z_base = gz - curvature_offset + 0.005;

        let (half_w, sprite_h) = match obj.model_type {
            ModelType::Person   => (step * 0.15, step * 0.4),
            ModelType::Scenery  => (step * 0.2, step * 0.25),
            _                   => (step * 0.2, step * 0.3),
        };

        let color_rgb = object_marker_color(&obj.model_type, obj.tribe_index);
        let color = Vector3::new(color_rgb[0], color_rgb[1], color_rgb[2]);

        let base_pos = Vector3::new(gx, gy, z_base);
        let bl = base_pos - right * half_w;
        let br = base_pos + right * half_w;
        let tl = bl + up * sprite_h;
        let tr = br + up * sprite_h;

        let v = |p: Vector3<f32>| ColorVertex { coord: p, color };

        model.push_vertex(v(bl));
        model.push_vertex(v(br));
        model.push_vertex(v(tr));
        model.push_vertex(v(bl));
        model.push_vertex(v(tr));
        model.push_vertex(v(tl));
    }
    if !model.vertices.is_empty() {
        let (mut min_x, mut min_y, mut min_z) = (f32::MAX, f32::MAX, f32::MAX);
        let (mut max_x, mut max_y, mut max_z) = (f32::MIN, f32::MIN, f32::MIN);
        for v in &model.vertices {
            min_x = min_x.min(v.coord.x); max_x = max_x.max(v.coord.x);
            min_y = min_y.min(v.coord.y); max_y = max_y.max(v.coord.y);
            min_z = min_z.min(v.coord.z); max_z = max_z.max(v.coord.z);
        }
        eprintln!("[markers] bbox x=[{:.3}..{:.3}] y=[{:.3}..{:.3}] z=[{:.3}..{:.3}] verts={}",
            min_x, max_x, min_y, max_y, min_z, max_z, model.vertices.len());
    }
    let m = vec![(RenderType::Triangles, model)];
    ModelEnvelop::<ColorModel>::new(device, m)
}

fn build_unit_markers(
    device: &wgpu::Device, units: &[pop3::unit_control::Unit],
    landscape: &LandscapeMesh<128>, curvature_scale: f32,
    angle_x: i16, angle_z: i16,
) -> Option<ModelEnvelop<ColorModel>> {
    if units.is_empty() { return None; }
    let mut model: ColorModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let center = (w - 1.0) * step / 2.0;

    let az = (angle_z as f32).to_radians();
    let ax = (angle_x as f32).to_radians();
    let eye = Point3::new(
        center + ax.cos() * az.sin(),
        center + ax.cos() * az.cos(),
        -ax.sin(),
    );
    let target = Point3::new(center, center, 0.0);
    let view = Matrix4::look_at_rh(eye, target, Vector3::new(0.0, 0.0, 1.0));
    let right = Vector3::new(view.x.x, view.y.x, view.z.x);
    let up = Vector3::new(view.x.y, view.y.y, view.z.y);

    for unit in units {
        let vis_x = ((unit.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((unit.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;

        let ix = (unit.cell_x as usize).min(127);
        let iy = (unit.cell_y as usize).min(127);
        let gz = landscape.height_at(ix, iy) as f32 * height_scale;

        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * curvature_scale;
        let z_base = gz - curvature_offset + 0.005;

        let half_w = step * 0.15;
        let sprite_h = step * 0.4;

        let color_rgb = object_marker_color(&unit.model_type, unit.tribe_index);
        let color = Vector3::new(color_rgb[0], color_rgb[1], color_rgb[2]);

        let base_pos = Vector3::new(gx, gy, z_base);
        let bl = base_pos - right * half_w;
        let br = base_pos + right * half_w;
        let tl = bl + up * sprite_h;
        let tr = br + up * sprite_h;

        let v = |p: Vector3<f32>| ColorVertex { coord: p, color };
        model.push_vertex(v(bl));
        model.push_vertex(v(br));
        model.push_vertex(v(tr));
        model.push_vertex(v(bl));
        model.push_vertex(v(tr));
        model.push_vertex(v(tl));
    }
    let m = vec![(RenderType::Triangles, model)];
    Some(ModelEnvelop::<ColorModel>::new(device, m))
}

fn build_selection_rings(
    device: &wgpu::Device, coordinator: &UnitCoordinator,
    landscape: &LandscapeMesh<128>, curvature_scale: f32,
) -> Option<ModelEnvelop<ColorModel>> {
    if coordinator.selection.selected.is_empty() { return None; }
    let mut model: ColorModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let center = (w - 1.0) * step / 2.0;
    let segments = 16;
    let radius = step * 0.3;
    let ring_width = step * 0.04;
    let color = Vector3::new(0.0, 1.0, 0.0); // Green

    for &unit_id in &coordinator.selection.selected {
        let unit = match coordinator.units.get(unit_id) {
            Some(u) => u,
            None => continue,
        };

        let vis_x = ((unit.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((unit.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;
        let ix = (unit.cell_x as usize).min(127);
        let iy = (unit.cell_y as usize).min(127);
        let gz = landscape.height_at(ix, iy) as f32 * height_scale;
        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * curvature_scale;
        let z_base = gz - curvature_offset + 0.003;

        for i in 0..segments {
            let a0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            let (c0, s0) = (a0.cos(), a0.sin());
            let (c1, s1) = (a1.cos(), a1.sin());

            // Inner and outer ring vertices (flat on ground plane)
            let inner0 = Vector3::new(gx + (radius - ring_width) * c0, gy + (radius - ring_width) * s0, z_base);
            let outer0 = Vector3::new(gx + radius * c0, gy + radius * s0, z_base);
            let inner1 = Vector3::new(gx + (radius - ring_width) * c1, gy + (radius - ring_width) * s1, z_base);
            let outer1 = Vector3::new(gx + radius * c1, gy + radius * s1, z_base);

            let v = |p: Vector3<f32>| ColorVertex { coord: p, color };
            model.push_vertex(v(inner0));
            model.push_vertex(v(outer0));
            model.push_vertex(v(outer1));
            model.push_vertex(v(inner0));
            model.push_vertex(v(outer1));
            model.push_vertex(v(inner1));
        }
    }
    let m = vec![(RenderType::Triangles, model)];
    Some(ModelEnvelop::<ColorModel>::new(device, m))
}

fn build_building_meshes(
    device: &wgpu::Device, objects: &[LevelObject], objects_3d: &[Option<Object3D>],
    shapes: &[Shape], landscape: &LandscapeMesh<128>, curvature_scale: f32,
) -> ModelEnvelop<TexModel> {
    let mut combined: TexModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let center = (w - 1.0) * step / 2.0;

    let mut building_count = 0;
    for obj in objects {
        let idx = match object_3d_index(&obj.model_type, obj.subtype, obj.tribe_index) {
            Some(i) => Some(i),
            None => continue,
        };
        building_count += 1;
        eprintln!("[3d-obj] type={:?} subtype={} tribe={} -> idx={:?}", obj.model_type, obj.subtype, obj.tribe_index, idx);
        let obj3d = match idx {
            Some(i) if i < objects_3d.len() => match &objects_3d[i] {
                Some(o) => o,
                None => { eprintln!("  -> object at {} is None", i); continue; },
            },
            _ => continue,
        };

        let local_model = mk_pop_object(obj3d);
        let scale = step * (obj3d.coord_scale() / 300.0);

        let vis_x = ((obj.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((obj.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;

        let ix = (obj.cell_x as usize).min(127);
        let iy = (obj.cell_y as usize).min(127);
        let gz = landscape.height_at(ix, iy) as f32 * height_scale;
        // TODO: height averaging across footprint — needs correct shapes_index mapping
        // (ObjectRaw.shapes_index may not be a direct SHAPES.DAT index)
        eprintln!("[bldg] subtype={} angle={} shape_idx={} gz={:.3}",
            obj.subtype, obj.angle, obj3d.shapes_index(), gz);

        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * curvature_scale;
        let z_base = gz - curvature_offset;

        // Rotate model vertices in the horizontal plane (model X/Z -> world X/Y)
        let angle_rad = (obj.angle as f32) * std::f32::consts::TAU / 2048.0;
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let base_idx = combined.vertices.len() as u16;
        let first_vert = combined.vertices.len();
        for v in &local_model.vertices {
            let rx = v.coord.x * cos_a - v.coord.z * sin_a;
            let rz = v.coord.x * sin_a + v.coord.z * cos_a;
            combined.push_vertex(TexVertex {
                coord: Vector3::new(
                    gx + rx * scale,
                    gy + rz * scale,
                    z_base + v.coord.y * scale,
                ),
                uv: v.uv,
                tex_id: v.tex_id,
            });
        }
        if let Some(fv) = combined.vertices.get(first_vert) {
            eprintln!("  -> verts={} pos=({:.3},{:.3},{:.3}) scale={:.4} gx={:.3} gy={:.3} gz={:.3} angle={} ({:.2} rad) tex_id={}",
                local_model.vertices.len(), fv.coord.x, fv.coord.y, fv.coord.z,
                scale, gx, gy, gz, obj.angle, angle_rad, fv.tex_id);
        }
        for &idx16 in &local_model.indices {
            combined.indices.push(base_idx + idx16);
        }
    }
    eprintln!("[buildings] total={} vertices={} indices={} step={:.4} center={:.4} h_scale={:.6}",
        building_count, combined.vertices.len(), combined.indices.len(), step, center, height_scale);
    // Print vertex bounding box for debugging
    if !combined.vertices.is_empty() {
        let (mut min_x, mut min_y, mut min_z) = (f32::MAX, f32::MAX, f32::MAX);
        let (mut max_x, mut max_y, mut max_z) = (f32::MIN, f32::MIN, f32::MIN);
        for v in &combined.vertices {
            min_x = min_x.min(v.coord.x); max_x = max_x.max(v.coord.x);
            min_y = min_y.min(v.coord.y); max_y = max_y.max(v.coord.y);
            min_z = min_z.min(v.coord.z); max_z = max_z.max(v.coord.z);
        }
        eprintln!("[buildings] bbox x=[{:.3}..{:.3}] y=[{:.3}..{:.3}] z=[{:.3}..{:.3}]",
            min_x, max_x, min_y, max_y, min_z, max_z);
    }
    let m = vec![(RenderType::Triangles, combined)];
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

fn help_text() -> &'static str {
    concat!(
        "Q/E:    Rotate\n",
        "Up/Dn:  Tilt\n",
        "WASD:   Pan terrain\n",
        "Space:  Center on blue\n",
        "B/V:    Next/Prev level\n",
        "N/M:    Next/Prev shader\n",
        "C:      Toggle curvature\n",
        "[/]:    Curvature +/-\n",
        "O:      Toggle objects\n",
        "Scroll: Zoom\n",
        "Esc:    Quit",
    )
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
    script: Option<PathBuf>,
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,

    // Landscape
    program_container: LandscapeProgramContainer,
    landscape_group0_layout: Option<wgpu::BindGroupLayout>,
    landscape_group0_bind_group: Option<wgpu::BindGroup>,
    model_main: Option<ModelEnvelop<LandscapeModel>>,

    // Object marker bind groups (shared by markers, unit markers, selection rings)
    objects_group0_bind_group: Option<wgpu::BindGroup>,
    objects_group1_bind_group: Option<wgpu::BindGroup>,

    // Spawn markers (shaman sprites)
    spawn_pipeline: Option<wgpu::RenderPipeline>,
    spawn_group1_bind_group: Option<wgpu::BindGroup>,
    sprite_group1_layout: Option<wgpu::BindGroupLayout>,
    model_spawn: Option<ModelEnvelop<TexModel>>,
    spawn_cells: Vec<(f32, f32, u8)>,  // (cell_x, cell_y, tribe_index)
    sprite_texture: Option<GpuTexture>,
    shaman_atlas_info: Option<ShamanAtlasInfo>,

    // Level object markers
    objects_marker_pipeline: Option<wgpu::RenderPipeline>,
    model_objects: Option<ModelEnvelop<ColorModel>>,
    level_objects: Vec<LevelObject>,
    show_objects: bool,

    // 3D building meshes
    objects_3d: Vec<Option<Object3D>>,
    shapes: Vec<Shape>,
    building_pipeline: Option<wgpu::RenderPipeline>,
    building_bind_group_1: Option<wgpu::BindGroup>,
    model_buildings: Option<ModelEnvelop<TexModel>>,

    // Sky
    sky_pipeline: Option<wgpu::RenderPipeline>,
    sky_bind_group: Option<wgpu::BindGroup>,
    sky_uniform_buffer: Option<GpuBuffer>,

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
    curvature_scale: f32,
    curvature_enabled: bool,
    zoom: f32,
    do_render: bool,
    mouse_pos: Point2<f32>,
    level_num: u8,
    sunlight: Vector4<f32>,
    wat_offset: i32,
    wat_interval: u32,
    frame_count: u32,

    // Config
    config: AppConfig,

    // Debug logging
    debug_log: BufWriter<File>,
    start_time: Instant,

    // Script replay
    script_commands: Vec<String>,
    script_index: usize,

    // Unit control
    unit_coordinator: UnitCoordinator,
    model_unit_markers: Option<ModelEnvelop<ColorModel>>,
    model_selection_rings: Option<ModelEnvelop<ColorModel>>,
    drag_state: DragState,
    last_tick: Instant,
    tick_interval: std::time::Duration,
    game_ticking: bool,
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

        let debug_log = BufWriter::new(
            File::create("/tmp/pop3_debug.jsonl").expect("failed to create debug log"),
        );

        let script_commands: Vec<String> = config.script.as_ref()
            .map(|path| {
                std::fs::read_to_string(path)
                    .unwrap_or_else(|e| panic!("failed to read script {:?}: {}", path, e))
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect()
            })
            .unwrap_or_default();

        App {
            window: None,
            gpu: None,
            program_container: LandscapeProgramContainer::new(),
            landscape_group0_layout: None,
            landscape_group0_bind_group: None,
            model_main: None,
            objects_group0_bind_group: None,
            objects_group1_bind_group: None,
            spawn_pipeline: None,
            spawn_group1_bind_group: None,
            sprite_group1_layout: None,
            model_spawn: None,
            spawn_cells: Vec::new(),
            sprite_texture: None,
            shaman_atlas_info: None,
            objects_marker_pipeline: None,
            model_objects: None,
            level_objects: Vec::new(),
            show_objects: true,
            objects_3d: Vec::new(),
            shapes: Vec::new(),
            building_pipeline: None,
            building_bind_group_1: None,
            model_buildings: None,
            sky_pipeline: None,
            sky_bind_group: None,
            sky_uniform_buffer: None,
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
            curvature_scale: 0.0512,
            curvature_enabled: true,
            zoom: 1.0,
            do_render: true,
            mouse_pos: Point2::<f32>::new(0.0, 0.0),
            level_num: config.level.unwrap_or(1),
            sunlight,
            wat_offset: -1,
            wat_interval: 5000,
            frame_count: 0,
            config,
            debug_log,
            start_time: Instant::now(),
            script_commands,
            script_index: 0,

            // Unit control
            unit_coordinator: UnitCoordinator::new(),
            model_unit_markers: None,
            model_selection_rings: None,
            drag_state: DragState::None,
            last_tick: Instant::now(),
            tick_interval: std::time::Duration::from_millis(50),
            game_ticking: false,
        }
    }

    fn build_landscape_params(&self) -> LandscapeUniformData {
        let shift = self.landscape_mesh.get_shift_vector();
        LandscapeUniformData {
            level_shift: [shift.x, shift.y, shift.z, shift.w],
            height_scale: self.landscape_mesh.height_scale(),
            step: self.landscape_mesh.step(),
            width: self.landscape_mesh.width() as i32,
            _pad_width: 0,
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

    /// The grid vertex that appears at the camera's focus point,
    /// accounting for the landscape model transform.
    fn camera_focus_vertex(&self) -> f32 {
        let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        // Camera focus is at world (center, center). Invert model transform to get model-space pos.
        let model_x = (center - LANDSCAPE_OFFSET) / LANDSCAPE_SCALE;
        model_x / self.landscape_mesh.step()
    }

    fn center_on_tribe0_shaman(&mut self) {
        if let Some(&(cx, cy, _)) = self.spawn_cells.iter().find(|(_, _, t)| *t == 0) {
            let n = self.landscape_mesh.width() as i32;
            let v = self.camera_focus_vertex() as i32;
            let sx = ((cx as i32 - v) % n + n) % n;
            let sy = ((cy as i32 - v) % n + n) % n;
            log::info!("[center] shaman at cell ({}, {}), camera_vertex={}, shift -> ({}, {})", cx, cy, v, sx, sy);
            self.landscape_mesh.set_shift(sx as usize, sy as usize);
            self.rebuild_spawn_model();
        } else {
            log::warn!("[center] no tribe 0 shaman in spawn_cells (len={})", self.spawn_cells.len());
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

        // Rebuild spawn markers and object markers
        self.spawn_cells = extract_spawn_cells(&level_res);
        self.level_objects = extract_level_objects(&level_res);

        // Extract person units into the coordinator (they become live entities)
        self.unit_coordinator.load_level(&level_res.units, level_res.landscape.land_size());
        // Remove persons from static markers — they're now rendered by the coordinator
        self.level_objects.retain(|obj| obj.model_type != ModelType::Person);

        self.rebuild_spawn_model();
        self.center_on_tribe0_shaman();
    }

    /// Compute the terrain height under the orbit camera eye position.
    fn camera_min_z(&self) -> f32 {
        let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        let az = (self.camera.angle_z as f32).to_radians();
        let ax = (self.camera.angle_x as f32).to_radians();
        let radius = 1.5 / self.zoom;
        let eye_x = center + radius * ax.cos() * az.sin();
        let eye_y = center + radius * ax.cos() * az.cos();
        let step = self.landscape_mesh.step();
        let n = self.landscape_mesh.width();
        let gx = (eye_x / step).clamp(0.0, (n - 1) as f32) as usize;
        let gy = (eye_y / step).clamp(0.0, (n - 1) as f32) as usize;
        // Add shift to match what the shader renders at this world position
        let shift = self.landscape_mesh.get_shift_vector();
        let sx = (gx + shift.x as usize) % n;
        let sy = (gy + shift.y as usize) % n;
        self.landscape_mesh.height_at(sx, sy) as f32 * self.landscape_mesh.height_scale() + 0.05
    }

    /// Ray-cast a screen click onto the landscape mesh and return cell coordinates.
    /// Returns None if the ray misses the terrain entirely.
    fn screen_to_cell(&self) -> Option<(f32, f32)> {
        let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        let focus = Vector3::new(center, center, 0.0);
        let min_z = self.camera_min_z();
        let (v1, v2) = screen_to_scene_zoom(&self.screen, &self.camera, &self.mouse_pos, self.zoom, focus, min_z);

        // Must use the actual landscape model transform (translate + scale)
        let mvp_transform = Matrix4::from_translation(Vector3::new(LANDSCAPE_OFFSET, LANDSCAPE_OFFSET, 0.0))
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

    /// Build the projection-view-model matrix for unit screen projection.
    fn unit_pvm(&self) -> Matrix4<f32> {
        let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        let focus = Vector3::new(center, center, 0.0);
        let min_z = self.camera_min_z();
        let mvp = MVP::with_zoom(&self.screen, &self.camera, self.zoom, focus, min_z);
        let model_transform = Matrix4::from_translation(Vector3::new(LANDSCAPE_OFFSET, LANDSCAPE_OFFSET, 0.0))
            * Matrix4::from_scale(LANDSCAPE_SCALE);
        mvp.projection * mvp.view * model_transform
    }

    /// Project a unit's billboard center to screen coordinates.
    /// Returns None if behind camera.
    fn unit_screen_pos(&self, unit: &Unit, pvm: &Matrix4<f32>) -> Option<(f32, f32)> {
        let step = self.landscape_mesh.step();
        let w = self.landscape_mesh.width() as f32;
        let shift = self.landscape_mesh.get_shift_vector();
        let height_scale = self.landscape_mesh.height_scale();
        let center = (w - 1.0) * step / 2.0;
        let cs = if self.curvature_enabled { self.curvature_scale } else { 0.0 };

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

        project_to_screen([gx, gy, z_base], pvm, self.screen.width as f32, self.screen.height as f32)
    }

    /// Find the unit whose billboard is closest to the screen click position.
    fn find_unit_at_screen_pos(&self, mouse: &Point2<f32>) -> Option<usize> {
        let pvm = self.unit_pvm();
        let candidates = self.unit_coordinator.units.iter().filter_map(|unit| {
            self.unit_screen_pos(unit, &pvm).map(|(sx, sy)| (unit.id, sx, sy))
        });
        nearest_screen_hit(candidates, mouse.x, mouse.y, 20.0)
    }

    /// Find all units whose billboard centers project into a screen rectangle.
    fn units_in_screen_rect(&self, corner_a: Point2<f32>, corner_b: Point2<f32>) -> Vec<usize> {
        let min_x = corner_a.x.min(corner_b.x);
        let max_x = corner_a.x.max(corner_b.x);
        let min_y = corner_a.y.min(corner_b.y);
        let max_y = corner_a.y.max(corner_b.y);

        let pvm = self.unit_pvm();
        let mut ids = Vec::new();
        for unit in &self.unit_coordinator.units {
            if let Some((sx, sy)) = self.unit_screen_pos(unit, &pvm) {
                if sx >= min_x && sx <= max_x && sy >= min_y && sy <= max_y {
                    ids.push(unit.id);
                }
            }
        }
        ids
    }

    fn log_camera_state(&mut self, event: &str) {
        let t = self.start_time.elapsed().as_secs_f64();
        let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        let az = (self.camera.angle_z as f32).to_radians();
        let ax = (self.camera.angle_x as f32).to_radians();
        let radius = 1.5 / self.zoom;
        let eye_x = center + radius * ax.cos() * az.sin();
        let eye_y = center + radius * ax.cos() * az.cos();
        let eye_z_orbit = -radius * ax.sin();
        let min_z = self.camera_min_z();
        let eye_z = eye_z_orbit.max(min_z);
        let shift = self.landscape_mesh.get_shift_vector();
        let _ = writeln!(
            self.debug_log,
            r#"{{"t":{:.3},"event":"{}","angle_x":{},"angle_z":{},"zoom":{:.3},"radius":{:.4},"eye":[{:.4},{:.4},{:.4}],"eye_z_orbit":{:.4},"min_z":{:.4},"focus":[{:.4},{:.4},0.0],"shift":[{},{}]}}"#,
            t, event,
            self.camera.angle_x, self.camera.angle_z,
            self.zoom, radius,
            eye_x, eye_y, eye_z, eye_z_orbit, min_z,
            center, center,
            shift.x, shift.y,
        );
        let _ = self.debug_log.flush();
    }

    fn is_script_mode(&self) -> bool {
        !self.script_commands.is_empty()
    }

    fn run_script_step(&mut self) -> bool {
        if self.script_index >= self.script_commands.len() {
            return false; // done
        }
        let cmd = self.script_commands[self.script_index].clone();
        self.script_index += 1;

        // Parse zoom command
        if let Some(val) = cmd.strip_prefix("zoom ") {
            if let Ok(z) = val.trim().parse::<f32>() {
                self.zoom = z.clamp(0.3, 5.0);
                self.log_camera_state("zoom");
                self.do_render = true;
                return true;
            }
        }

        // Parse key name to KeyCode
        let key = match cmd.as_str() {
            "W" => KeyCode::KeyW,
            "A" => KeyCode::KeyA,
            "S" => KeyCode::KeyS,
            "D" => KeyCode::KeyD,
            "Q" => KeyCode::KeyQ,
            "E" => KeyCode::KeyE,
            "R" => KeyCode::KeyR,
            "T" => KeyCode::KeyT,
            "N" => KeyCode::KeyN,
            "M" => KeyCode::KeyM,
            "B" => KeyCode::KeyB,
            "V" => KeyCode::KeyV,
            "C" => KeyCode::KeyC,
            "Space" => KeyCode::Space,
            "ArrowUp" => KeyCode::ArrowUp,
            "ArrowDown" => KeyCode::ArrowDown,
            "BracketLeft" => KeyCode::BracketLeft,
            "BracketRight" => KeyCode::BracketRight,
            other => {
                log::warn!("script: unknown command {:?}", other);
                return true; // skip, continue
            }
        };

        // Replay through the same logic as the keyboard handler
        match key {
            KeyCode::KeyR => {
                self.camera.angle_x = -55;
                self.camera.angle_y = 0;
                self.camera.angle_z = 0;
                self.camera.pos = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
                self.rebuild_spawn_model();
                self.log_camera_state("reset");
            },
            KeyCode::KeyT => {
                self.camera.angle_x = -90;
                self.log_camera_state("KeyT");
            },
            KeyCode::Space => {
                self.center_on_tribe0_shaman();
                self.log_camera_state("space_center");
            },
            KeyCode::KeyC => {
                self.curvature_enabled = !self.curvature_enabled;
                self.rebuild_spawn_model();
            },
            KeyCode::KeyO => {
                self.show_objects = !self.show_objects;
            },
            KeyCode::BracketRight => {
                self.curvature_scale *= 1.2;
                self.rebuild_spawn_model();
            },
            KeyCode::BracketLeft => {
                self.curvature_scale *= 0.8;
                self.rebuild_spawn_model();
            },
            _ => {
                let prev_shift = self.landscape_mesh.get_shift_vector();
                let prev_angle_z = self.camera.angle_z;
                let prev_angle_x = self.camera.angle_x;
                process_key(key, &mut self.camera, &mut self.landscape_mesh);
                let new_shift = self.landscape_mesh.get_shift_vector();
                let shift_changed = new_shift != prev_shift;
                if shift_changed || self.camera.angle_z != prev_angle_z || self.camera.angle_x != prev_angle_x {
                    self.rebuild_spawn_model();
                    self.log_camera_state(&format!("{:?}", key));
                }
            },
        }
        self.do_render = true;
        true
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
        self.rebuild_object_markers();
    }

    fn rebuild_unit_models(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let cs = if self.curvature_enabled { self.curvature_scale } else { 0.0 };
            self.model_unit_markers = build_unit_markers(
                &gpu.device, &self.unit_coordinator.units, &self.landscape_mesh, cs,
                self.camera.angle_x, self.camera.angle_z,
            );
            self.model_selection_rings = build_selection_rings(
                &gpu.device, &self.unit_coordinator,
                &self.landscape_mesh, cs,
            );
        }
    }

    fn rebuild_object_markers(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let cs = if self.curvature_enabled { self.curvature_scale } else { 0.0 };
            self.model_objects = Some(build_object_markers(
                &gpu.device, &self.level_objects, &self.landscape_mesh, cs,
                self.camera.angle_x, self.camera.angle_z,
            ));
            self.model_buildings = Some(build_building_meshes(
                &gpu.device, &self.level_objects, &self.objects_3d,
                &self.shapes, &self.landscape_mesh, cs,
            ));
        }
        self.rebuild_unit_models();
    }

    fn rebuild_overlay(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let text = help_text();
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
        let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        let focus = Vector3::new(center, center, 0.0);
        let min_z = self.camera_min_z();
        let mvp = MVP::with_zoom(&self.screen, &self.camera, self.zoom, focus, min_z);
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
        struct ObjectParams { num_colors: i32, _pad: i32 }
        let obj_params = ObjectParams { num_colors: obj_colors().len() as i32, _pad: 0 };
        self.select_params_buffer.as_ref().unwrap().update(&gpu.queue, 0, bytemuck::bytes_of(&obj_params));

        // Update sky yaw offset
        if let Some(ref sky_buf) = self.sky_uniform_buffer {
            // angle_z is in degrees; map to 0..1 range for UV offset
            let yaw = (self.camera.angle_z as f32) / 360.0;
            sky_buf.update(&gpu.queue, 0, bytemuck::bytes_of(&[yaw, 0.0f32, 0.0f32, 0.0f32]));
        }

        // Update select model vertex data
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

            // Draw sky background
            if let (Some(ref sky_pipe), Some(ref sky_bg)) =
                (&self.sky_pipeline, &self.sky_bind_group)
            {
                render_pass.set_pipeline(sky_pipe);
                render_pass.set_bind_group(0, sky_bg, &[]);
                render_pass.draw(0..3, 0..1);
            }

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

            // Draw 3D building meshes (identity model transform, same as markers)
            if self.show_objects {
                if let (Some(ref pipeline), Some(ref bg1)) =
                    (&self.building_pipeline, &self.building_bind_group_1)
                {
                    // model_select already wrote identity — no extra write needed
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                    render_pass.set_bind_group(1, bg1, &[]);
                    if let Some(ref model) = self.model_buildings {
                        model.draw(&mut render_pass);
                    }
                }
            }

            // Draw level object markers (non-building objects)
            if self.show_objects {
                if let Some(ref marker_pipeline) = self.objects_marker_pipeline {
                    render_pass.set_pipeline(marker_pipeline);
                    render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                    if let Some(ref model_objects) = self.model_objects {
                        model_objects.draw(&mut render_pass);
                    }
                }
            }

            // Draw live unit markers and selection rings
            if let Some(ref marker_pipeline) = self.objects_marker_pipeline {
                render_pass.set_pipeline(marker_pipeline);
                render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                if let Some(ref model) = self.model_unit_markers {
                    model.draw(&mut render_pass);
                }
                if let Some(ref model) = self.model_selection_rings {
                    model.draw(&mut render_pass);
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
        struct ObjectParams { num_colors: i32, _pad: i32 }

        let colors = obj_colors();
        let obj_params = ObjectParams { num_colors: colors.len() as i32, _pad: 0 };
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

        // Landscape model
        let model_main = make_landscape_model(device, &self.landscape_mesh);

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

        // Level objects marker pipeline (group 0 only, no group 1)
        let objects_marker_shader = include_str!("../shaders/level_objects.wgsl");
        let objects_marker_layouts = ColorModel::vertex_buffer_layouts();
        let objects_marker_pipeline = create_pipeline(
            device, objects_marker_shader, &objects_marker_layouts,
            &[&objects_group0_layout],
            gpu.surface_format(), true,
            wgpu::PrimitiveTopology::TriangleList,
            "level_objects_pipeline",
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

        // Load 3D objects (bank "0"), shapes, and BL320 texture atlas for building meshes
        let objects_3d = Object3D::from_file_all(&base, "0");
        let obj_paths = ObjectPaths::from_default_dir(&base, "0");
        let shapes: Vec<Shape> = Shape::from_file_vec(&obj_paths.shapes);
        eprintln!("[shapes] loaded {} entries", shapes.len());
        for (i, s) in shapes.iter().take(10).enumerate() {
            let sref = s.shape_ref;
            eprintln!("[shapes] [{}] {}x{} origin=({},{}) ref={}",
                i, s.width, s.height, s.origin_x, s.origin_z, sref);
        }

        let (bl320_w, bl320_h, mut bl320_data) = make_bl320_texture_rgba(
            &level_res.paths.bl320, &level_res.params.palette);

        // Mark transparent pixels (palette index 0) with alpha=255 so the shader
        // can discard them via `if (color.w > 0.0) { discard; }`.
        let key_r = level_res.params.palette[0];
        let key_g = level_res.params.palette[1];
        let key_b = level_res.params.palette[2];
        for pixel in bl320_data.chunks_exact_mut(4) {
            if pixel[0] == key_r && pixel[1] == key_g && pixel[2] == key_b && pixel[3] == 0 {
                pixel[3] = 255;
            }
        }

        let bl320_gpu_tex = GpuTexture::new_2d(
            device, &gpu.queue,
            bl320_w as u32, bl320_h as u32,
            wgpu::TextureFormat::Rgba8Unorm,
            &bl320_data,
            "bl320_texture",
        );
        let bl320_sampler = GpuTexture::create_sampler(device, false);

        let building_bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("building_bg1"),
            layout: &sprite_group1_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&bl320_gpu_tex.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&bl320_sampler) },
            ],
        });

        // Building pipeline (reuses objects_tex.wgsl shader, TexModel vertex layout)
        let building_shader_source = include_str!("../shaders/objects_tex.wgsl");
        let building_vertex_layouts = TexModel::vertex_buffer_layouts();
        let building_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("building_shader"),
            source: wgpu::ShaderSource::Wgsl(building_shader_source.into()),
        });
        let building_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("building_pipeline_layout"),
            bind_group_layouts: &[&objects_group0_layout, &sprite_group1_layout],
            immediate_size: 0,
        });
        let building_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("building_pipeline"),
            layout: Some(&building_pipe_layout),
            vertex: wgpu::VertexState {
                module: &building_shader,
                entry_point: Some("vs_main"),
                buffers: &building_vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &building_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.surface_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Sky texture and pipeline
        let sky_data = std::fs::read(&level_res.paths.sky).ok();
        let (sky_pipeline, sky_bind_group, sky_uniform_buffer) = if let Some(sky_raw) = sky_data {
            // sky0-{key}.dat is 512x512 palette indices (262144 bytes).
            // sky0-0.dat is 307200 bytes (600x512); just take first 512 rows.
            let sky_size = 512usize;
            let pixel_count = sky_size * sky_size;
            let sky_indices = if sky_raw.len() >= pixel_count {
                &sky_raw[..pixel_count]
            } else {
                &sky_raw[..]
            };
            let pal = &level_res.params.palette;
            // Game adds 0x70 to every sky byte, then uses result as direct palette index
            let mut sky_rgb = vec![0u8; sky_size * sky_size * 3];
            for (i, &idx) in sky_indices.iter().enumerate() {
                let pal_idx = idx.wrapping_add(0x70) as usize * 4;
                sky_rgb[i * 3]     = pal[pal_idx];
                sky_rgb[i * 3 + 1] = pal[pal_idx + 1];
                sky_rgb[i * 3 + 2] = pal[pal_idx + 2];
            }
            let sky_rgba = rgb_to_rgba(&sky_rgb);
            let sky_tex = GpuTexture::new_2d(
                device, &gpu.queue,
                sky_size as u32, sky_size as u32,
                wgpu::TextureFormat::Rgba8Unorm,
                &sky_rgba, "sky_texture",
            );
            let sky_sampler = GpuTexture::create_sampler(device, false);
            let sky_uniform = GpuBuffer::new_uniform_init(
                device, bytemuck::bytes_of(&[0.0f32; 4]), "sky_uniform",
            );

            let sky_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sky_bg_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

            let sky_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sky_bg"),
                layout: &sky_bg_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: sky_uniform.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&sky_tex.view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sky_sampler) },
                ],
            });

            let sky_shader_source = include_str!("../shaders/sky.wgsl");
            let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sky_shader"),
                source: wgpu::ShaderSource::Wgsl(sky_shader_source.into()),
            });
            let sky_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sky_pipeline_layout"),
                bind_group_layouts: &[&sky_bg_layout],
                immediate_size: 0,
            });
            let sky_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("sky_pipeline"),
                layout: Some(&sky_pipe_layout),
                vertex: wgpu::VertexState {
                    module: &sky_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &sky_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.surface_format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            (Some(sky_pipe), Some(sky_bg), Some(sky_uniform))
        } else {
            log::warn!("Sky texture not found: {:?}", level_res.paths.sky);
            (None, None, None)
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
        self.spawn_pipeline = Some(spawn_pipeline);
        self.spawn_group1_bind_group = spawn_group1_bind_group;
        self.sprite_group1_layout = Some(sprite_group1_layout);
        self.sprite_texture = sprite_texture;
        self.shaman_atlas_info = shaman_atlas_info;
        self.objects_marker_pipeline = Some(objects_marker_pipeline);
        self.objects_3d = objects_3d;
        self.shapes = shapes;
        self.building_pipeline = Some(building_pipeline);
        self.building_bind_group_1 = Some(building_bind_group_1);
        self.sky_pipeline = sky_pipeline;
        self.sky_bind_group = sky_bind_group;
        self.sky_uniform_buffer = sky_uniform_buffer;
        self.model_spawn = Some(model_spawn);
        self.model_main = Some(model_main);
        self.overlay_pipeline = Some(overlay_pipeline);
        self.overlay_bind_group = Some(overlay_bind_group);
        self.overlay_uniform_buffer = Some(overlay_uniform_buffer);

        self.gpu = Some(gpu);

        // Build landscape variants (needs self.gpu, heights_buffer, etc.)
        let base2 = self.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type2 = self.config.landtype.as_deref();
        let level_res2 = LevelRes::new(&base2, self.level_num, level_type2);
        self.rebuild_landscape_variants(&level_res2);

        // Build spawn markers and object markers for initial level
        self.spawn_cells = extract_spawn_cells(&level_res2);
        self.level_objects = extract_level_objects(&level_res2);

        // Extract person units into the coordinator (they become live entities)
        self.unit_coordinator.load_level(&level_res2.units, level_res2.landscape.land_size());
        self.level_objects.retain(|obj| obj.model_type != ModelType::Person);

        self.rebuild_spawn_model();
        self.center_on_tribe0_shaman();

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

                // Update drag state
                match self.drag_state {
                    DragState::PendingDrag { start } => {
                        let dx = self.mouse_pos.x - start.x;
                        let dy = self.mouse_pos.y - start.y;
                        if dx * dx + dy * dy > 25.0 { // 5px threshold
                            self.drag_state = DragState::Dragging { start, current: self.mouse_pos };
                            self.do_render = true;
                        }
                    }
                    DragState::Dragging { start, .. } => {
                        self.drag_state = DragState::Dragging { start, current: self.mouse_pos };
                        self.do_render = true;
                    }
                    DragState::None => {}
                }
            },
            WindowEvent::MouseInput { state, button, .. } => {
                match (button, state) {
                    (MouseButton::Left, ElementState::Pressed) => {
                        // Start potential drag
                        self.drag_state = DragState::PendingDrag { start: self.mouse_pos };
                    }
                    (MouseButton::Left, ElementState::Released) => {
                        match self.drag_state {
                            DragState::PendingDrag { .. } => {
                                // Short click (no drag) — single-select via billboard projection
                                match self.find_unit_at_screen_pos(&self.mouse_pos) {
                                    Some(id) => self.unit_coordinator.selection.select_single(id),
                                    None => self.unit_coordinator.selection.clear(),
                                }

                            }
                            DragState::Dragging { start, current } => {
                                // Drag release — box-select units in screen rectangle
                                let ids = self.units_in_screen_rect(start, current);
                                self.unit_coordinator.selection.select_multiple(ids);
                            }
                            DragState::None => {}
                        }
                        self.drag_state = DragState::None;
                        self.rebuild_unit_models();
                    }
                    (MouseButton::Right, ElementState::Pressed) => {
                        // Right-click: move order
                        if let Some((cx, cy)) = self.screen_to_cell() {
                            let target = cell_to_world(cx, cy, self.landscape_mesh.width() as f32);
                            self.unit_coordinator.order_move(target);
                        }
                    }
                    _ => {}
                }
                self.do_render = true;
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
                };
                self.zoom *= 1.1_f32.powf(scroll_y);
                self.zoom = self.zoom.clamp(0.3, 5.0);
                self.log_camera_state("zoom");
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
                                self.camera.angle_x = -55;
                                self.camera.angle_y = 0;
                                self.camera.angle_z = 0;
                                self.camera.pos = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
                                self.rebuild_spawn_model();
                                self.log_camera_state("reset");
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
                            KeyCode::KeyC => {
                                self.curvature_enabled = !self.curvature_enabled;
                                log::info!("curvature {}", if self.curvature_enabled { "on" } else { "off" });
                                self.rebuild_spawn_model();
                            },
                            KeyCode::KeyO => {
                                self.show_objects = !self.show_objects;
                                log::info!("objects {}", if self.show_objects { "on" } else { "off" });
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
                            KeyCode::F5 => {
                                self.game_ticking = !self.game_ticking;
                                if self.game_ticking {
                                    self.last_tick = Instant::now();
                                }
                                log::info!("game ticking {}", if self.game_ticking { "on" } else { "off" });
                            },
                            KeyCode::Space => {
                                self.center_on_tribe0_shaman();
                                self.log_camera_state("space_center");
                            },
                            _ => {
                                let prev_shift = self.landscape_mesh.get_shift_vector();
                                let prev_angle_z = self.camera.angle_z;
                                let prev_angle_x = self.camera.angle_x;
                                process_key(key, &mut self.camera, &mut self.landscape_mesh);
                                let new_shift = self.landscape_mesh.get_shift_vector();
                                let shift_changed = new_shift != prev_shift;
                                if shift_changed || self.camera.angle_z != prev_angle_z || self.camera.angle_x != prev_angle_x {
                                    self.rebuild_spawn_model();
                                    self.log_camera_state(&format!("{:?}", key));
                                }
                            },
                        }
                        self.do_render = true;
                    }
                }
            },
            WindowEvent::RedrawRequested => {
                // Tick game simulation (20Hz when enabled)
                if self.game_ticking {
                    let now = Instant::now();
                    if now - self.last_tick >= self.tick_interval {
                        self.unit_coordinator.tick();
                        self.rebuild_unit_models();
                        self.last_tick = now;
                        self.do_render = true;
                    }
                }

                // Auto-animate water
                self.frame_count = self.frame_count.wrapping_add(1);
                if self.frame_count % self.wat_interval == 0 {
                    self.wat_offset += 1;
                    self.do_render = true;
                }
                if self.do_render && self.gpu.is_some() {
                    self.render();
                    self.do_render = false;
                }
                // Script replay: process one command per frame
                if self.is_script_mode() {
                    if !self.run_script_step() {
                        event_loop.exit();
                        return;
                    }
                }
            },
            _ => (),
        }
        if let Some(window) = &self.window {
            window.request_redraw();
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
        Arg::new("script")
            .long("script")
            .action(ArgAction::Set)
            .value_name("SCRIPT_PATH")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Replay key events from a script file"),
    ];
    Command::new("pop3")
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
        script: matches.get_one("script").cloned(),
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

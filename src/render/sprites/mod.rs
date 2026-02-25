use cgmath::{Point3, Vector2, Vector3, Matrix4};

use crate::render::model::{VertexModel, MeshModel};
use crate::render::tex_model::{TexModel, TexVertex};
use crate::render::color_model::{ColorModel, ColorVertex};
use crate::render::envelop::{ModelEnvelop, RenderType};
use crate::render::terrain::LandscapeMesh;
use crate::render::gpu::texture::GpuTexture;

use crate::data::level::LevelRes;
use crate::data::units::{ModelType, object_3d_index};
use crate::data::animation::{NUM_TRIBES, STORED_DIRECTIONS};
use crate::engine::state::constants::*;

use crate::engine::units::{UnitCoordinator, Unit};

/******************************************************************************/

pub fn obj_colors() -> Vec<Vector3<u8>> {
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

/// Convert raw palette bytes (4 bytes per entry) to [[u8; 4]; 256].
pub fn convert_palette(raw: &[u8]) -> Vec<[u8; 4]> {
    let mut pal = Vec::with_capacity(256);
    for i in 0..256 {
        let off = i * 4;
        if off + 3 < raw.len() {
            pal.push([raw[off], raw[off + 1], raw[off + 2], 255]);
        } else {
            pal.push([0, 0, 0, 255]);
        }
    }
    pal
}

/// Returns (source_direction_row, is_mirrored) for display direction 0-7.
pub fn get_source_direction(dir: usize) -> (usize, bool) {
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
pub fn tribe_facing_direction(tribe_index: u8) -> u16 {
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
pub fn sprite_direction_from_angle(camera_angle_z: i16, unit_facing_game: u16) -> usize {
    // Convert angle_z (degrees) to game angle units (0-2047, where 2048 = 360°)
    let camera_rot = ((camera_angle_z as i32) * 2048 / 360 % 2048 + 2048) % 2048;
    let raw = (camera_rot - unit_facing_game as i32 - 0x380) & 0x700;
    (raw >> 8) as usize
}

pub fn tribe_marker_color(tribe_index: u8) -> [f32; 3] {
    match tribe_index {
        0 => [0.2, 0.4, 1.0],   // Blue
        1 => [1.0, 0.2, 0.2],   // Red
        2 => [1.0, 1.0, 0.2],   // Yellow
        3 => [0.2, 1.0, 0.2],   // Green
        _ => [0.9, 0.9, 0.9],   // Unowned (tribe 255 = no owner)
    }
}

pub fn object_marker_color(model_type: &ModelType, tribe_index: u8) -> [f32; 3] {
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

/// Pack palette from RGBA u8 slices into packed u32 for storage buffer.
pub fn pack_palette_rgba(palette: &[u8]) -> Vec<u32> {
    palette.chunks(4).map(|c| {
        let r = c.get(0).copied().unwrap_or(0) as u32;
        let g = c.get(1).copied().unwrap_or(0) as u32;
        let b = c.get(2).copied().unwrap_or(0) as u32;
        let a = c.get(3).copied().unwrap_or(0) as u32;
        r | (g << 8) | (b << 16) | (a << 24)
    }).collect()
}

/// Convert RGB byte data to RGBA byte data (adding alpha=255).
pub fn rgb_to_rgba(rgb: &[u8]) -> Vec<u8> {
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

/******************************************************************************/
// Data extraction

/// Extract all person unit positions from level data, grouped by subtype.
/// Returns a Vec of (subtype, cells) where cells is Vec<(cell_x, cell_y, tribe_index)>.
pub fn extract_all_unit_cells(level_res: &LevelRes) -> Vec<(u8, Vec<(f32, f32, u8)>)> {
    let n = level_res.landscape.land_size() as f32;
    let mut by_subtype: std::collections::HashMap<u8, Vec<(f32, f32, u8)>> = std::collections::HashMap::new();

    for unit in &level_res.units {
        let tribe = unit.tribe_index() as usize;
        // Person model, subtypes Brave..Shaman, valid tribes
        if unit.model == 1 && unit.subtype >= PERSON_SUBTYPE_BRAVE && unit.subtype <= PERSON_SUBTYPE_SHAMAN && tribe < 4 {
            if unit.loc_x() == 0 && unit.loc_y() == 0 { continue; }
            let bevy_x = ((unit.loc_x() >> 8) / 2) as f32 + 0.5;
            let bevy_z = ((unit.loc_y() >> 8) / 2) as f32 + 0.5;
            let cell_x = bevy_z;
            let cell_y = (n - 1.0) - bevy_x;
            by_subtype.entry(unit.subtype).or_default()
                .push((cell_x, cell_y, unit.tribe_index()));
        }
    }

    let mut result: Vec<(u8, Vec<(f32, f32, u8)>)> = by_subtype.into_iter().collect();
    result.sort_by_key(|(st, _)| *st);
    result
}

pub struct LevelObject {
    pub cell_x: f32,
    pub cell_y: f32,
    pub model_type: ModelType,
    #[allow(dead_code)]
    pub subtype: u8,
    pub tribe_index: u8,
    pub angle: u32,
}

pub fn extract_level_objects(level_res: &LevelRes) -> Vec<LevelObject> {
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

/******************************************************************************/
// Per-unit rendering data (engine → renderer boundary)

/// Per-unit rendering data passed from engine to renderer each tick.
pub struct UnitRenderData {
    pub cell_x: f32,
    pub cell_y: f32,
    pub tribe_index: u8,
    pub facing_angle: u16,
    pub frame_index: u8,
    pub animation_id: u16,
}

/******************************************************************************/
// Per-unit-type rendering state

/// Per-unit-type rendering state (texture atlas + bind group + model).
pub struct UnitTypeRender {
    pub subtype: u8,
    pub cells: Vec<UnitRenderData>,
    #[allow(dead_code)]
    pub texture: GpuTexture,  // kept alive for GPU bind group
    pub bind_group: wgpu::BindGroup,
    pub model: Option<ModelEnvelop<TexModel>>,
    pub shadow_model: Option<ModelEnvelop<TexModel>>,  // flat proxy for shadow depth pass
    pub frame_width: u32,
    pub frame_height: u32,
    pub frames_per_dir: u32,  // total columns in atlas
    /// Maps animation_id → (column_offset, frame_count) within the atlas.
    pub anim_offsets: Vec<(u16, u32, u32)>,
    /// Fraction of frame height below the foot anchor (0.0–1.0).
    pub foot_below_frac: f32,
}

/******************************************************************************/
// GPU billboard builders

/// Build camera-facing billboard quads for unit sprites.
/// Each unit gets a single quad (6 vertices) oriented to face the camera.
/// `frames_per_dir` and `frame_w`/`frame_h` come from the unit type's atlas.
pub fn build_spawn_model(device: &wgpu::Device, cells: &[UnitRenderData],
                     landscape: &LandscapeMesh<128>, curvature_scale: f32,
                     angle_x: i16, angle_z: i16,
                     frame_w: u32, frame_h: u32, frames_per_dir: u32,
                     anim_offsets: &[(u16, u32, u32)],
                     foot_below_frac: f32,
) -> ModelEnvelop<TexModel> {
    let mut model: TexModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();

    // Sprite sizing: use atlas aspect ratio
    let sprite_h = step * 0.6;
    let aspect = if frame_h > 0 { frame_w as f32 / frame_h as f32 } else { 1.0 };
    let half_w = sprite_h * aspect / 2.0;

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

    let fpd = frames_per_dir as f32;
    let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as f32;
    let uv_scale_x = 1.0 / fpd;
    let uv_scale_y = 1.0 / total_rows;

    for unit_data in cells {
        let cell_x = unit_data.cell_x;
        let cell_y = unit_data.cell_y;
        let tribe_index = unit_data.tribe_index;

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

        // Per-unit sprite direction from actual facing angle
        let display_dir = sprite_direction_from_angle(angle_z, unit_data.facing_angle);
        let (src_dir, mirrored) = get_source_direction(display_dir);

        // Per-unit frame UV offset, accounting for animation column offset
        let (col_offset, anim_frames) = anim_offsets.iter()
            .find(|(id, _, _)| *id == unit_data.animation_id)
            .map(|(_, off, fc)| (*off, *fc))
            .unwrap_or((0, frames_per_dir));
        let frame_idx = (unit_data.frame_index as u32).min(anim_frames.saturating_sub(1));
        let uv_off_x = (col_offset + frame_idx) as f32 / fpd;

        let tribe_row = tribe_index as usize * STORED_DIRECTIONS + src_dir;
        let uv_off_y = tribe_row as f32 / total_rows;
        let (u_left, u_right) = if mirrored {
            (uv_off_x + uv_scale_x, uv_off_x)
        } else {
            (uv_off_x, uv_off_x + uv_scale_x)
        };
        let v_bottom = uv_off_y + uv_scale_y;
        let v_top = uv_off_y;

        // Screen-facing billboard quad using right and up vectors.
        // Shift base down so the foot pixel row (not the cell bottom) aligns with ground.
        let foot_shift = foot_below_frac * sprite_h;
        let base = Vector3::new(gx, gy, z_base);
        let bl = base - right * half_w - up * foot_shift;
        let br = base + right * half_w - up * foot_shift;
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

/// Build flat horizontal quads at each sprite position for shadow depth casting.
/// Camera-facing billboards are edge-on from the light's POV, so we use flat
/// ground-level proxies instead.
pub fn build_shadow_proxy_model(device: &wgpu::Device, cells: &[UnitRenderData],
                     landscape: &LandscapeMesh<128>, curvature_scale: f32,
                     frame_w: u32, frame_h: u32, frames_per_dir: u32,
                     anim_offsets: &[(u16, u32, u32)],
) -> ModelEnvelop<TexModel> {
    let mut model: TexModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();

    let sprite_h = step * 0.6;
    let aspect = if frame_h > 0 { frame_w as f32 / frame_h as f32 } else { 1.0 };
    let half_w = sprite_h * aspect / 2.0;

    let center = (w - 1.0) * step / 2.0;

    let fpd = frames_per_dir as f32;
    let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as f32;
    let uv_scale_x = 1.0 / fpd;
    let uv_scale_y = 1.0 / total_rows;

    for unit_data in cells {
        let cell_x = unit_data.cell_x;
        let cell_y = unit_data.cell_y;
        let tribe_index = unit_data.tribe_index;

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
        let z_base = gz - curvature_offset + 0.001; // at ground level; depth bias handled in shader

        let tid = tribe_index as i16;

        // Per-unit UV offset (same as billboard)
        let (col_offset, anim_frames) = anim_offsets.iter()
            .find(|(id, _, _)| *id == unit_data.animation_id)
            .map(|(_, off, fc)| (*off, *fc))
            .unwrap_or((0, frames_per_dir));
        let frame_idx = (unit_data.frame_index as u32).min(anim_frames.saturating_sub(1));
        let uv_off_x = (col_offset + frame_idx) as f32 / fpd;

        let display_dir = sprite_direction_from_angle(0, unit_data.facing_angle);
        let (src_dir, mirrored) = get_source_direction(display_dir);
        let tribe_row = tribe_index as usize * STORED_DIRECTIONS + src_dir;
        let uv_off_y = tribe_row as f32 / total_rows;
        // Shadow proxy: world +X = screen-left (camera right ≈ -X), so swap U
        let (u_left, u_right) = if mirrored {
            (uv_off_x, uv_off_x + uv_scale_x)
        } else {
            (uv_off_x + uv_scale_x, uv_off_x)
        };
        let v_bottom = uv_off_y + uv_scale_y;
        let v_top = uv_off_y;

        // Flat quad on the ground plane (x/y spread, constant z)
        let bl = Vector3::new(gx - half_w, gy - sprite_h * 0.5, z_base);
        let br = Vector3::new(gx + half_w, gy - sprite_h * 0.5, z_base);
        let tl = Vector3::new(gx - half_w, gy + sprite_h * 0.5, z_base);
        let tr = Vector3::new(gx + half_w, gy + sprite_h * 0.5, z_base);

        let v = |p: Vector3<f32>, u: f32, v: f32| -> TexVertex {
            TexVertex { coord: p, uv: Vector2::new(u, v), tex_id: tid }
        };

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

pub fn build_object_markers(
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

        // Skip objects outside the visible globe disc
        let dx_cull = gx - center;
        let dy_cull = gy - center;
        let viewport_radius = center * 0.9;
        if dx_cull * dx_cull + dy_cull * dy_cull > viewport_radius * viewport_radius {
            continue;
        }

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

pub fn build_unit_markers(
    device: &wgpu::Device, units: &[Unit],
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

pub fn build_selection_rings(
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

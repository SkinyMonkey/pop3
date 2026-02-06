use std::path::{Path, PathBuf};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, ArgAction, Command};

use cgmath::{Point2, Vector3, Vector4, Matrix4, SquareMatrix};

use faithful::model::{VertexModel, MeshModel};
use faithful::default_model::DefaultModel;
use faithful::view::*;

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
                    // World rotation (camera stays fixed, world spins)
                    KeyCode::ArrowLeft => { camera.angle_z -= 5; },
                    KeyCode::ArrowRight => { camera.angle_z += 5; },
                    // Tilt (pitch) the view
                    KeyCode::ArrowUp => { camera.angle_x = (camera.angle_x + 5).min(-30); },
                    KeyCode::ArrowDown => { camera.angle_x = (camera.angle_x - 5).max(-90); },
                    // Scroll the world (toroidal panning)
                    KeyCode::KeyW => { landscape_mesh.shift_y(1); },
                    KeyCode::KeyS => { landscape_mesh.shift_y(-1); },
                    KeyCode::KeyA => { landscape_mesh.shift_x(-1); },
                    KeyCode::KeyD => { landscape_mesh.shift_x(1); },
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

fn spawn_tribe_colors() -> Vec<Vector3<u8>> {
    vec![
        Vector3 { x: 51, y: 128, z: 255 },  // Tribe 0: Blue
        Vector3 { x: 255, y: 51, z: 51 },    // Tribe 1: Red
        Vector3 { x: 255, y: 230, z: 51 },   // Tribe 2: Yellow
        Vector3 { x: 51, y: 230, z: 77 },    // Tribe 3: Green
    ]
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

/// Build spawn marker triangles at grid positions adjusted for the current landscape shift.
/// Applies the same curvature displacement as the landscape shader so markers sit on the terrain.
fn build_spawn_model(device: &wgpu::Device, cells: &[(f32, f32, u8)],
                     landscape: &LandscapeMesh<128>, curvature_scale: f32) -> ModelEnvelop<DefaultModel> {
    let mut model: DefaultModel = MeshModel::new();
    let step = landscape.step();
    let height_scale = landscape.height_scale();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let size = step * 4.0;

    // Camera focus = grid center (same as build_landscape_params)
    let center = (w - 1.0) * step / 2.0;

    for &(cell_x, cell_y, _) in cells {
        // Visual grid position: where this cell appears given the current shift
        let vis_x = ((cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;

        // Sample height at the original cell position
        let ix = (cell_x as usize).min(127);
        let iy = (cell_y as usize).min(127);
        let h = landscape.height_at(ix, iy) as f32 * height_scale;

        // Apply same curvature as landscape vertex shader
        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * curvature_scale;
        let z = h - curvature_offset + 0.05;

        model.push_vertex(Vector3::new(gx, gy - size, z));
        model.push_vertex(Vector3::new(gx - size, gy + size, z));
        model.push_vertex(Vector3::new(gx + size, gy + size, z));
    }
    let m = vec![(RenderType::Triangles, model)];
    ModelEnvelop::<DefaultModel>::new(device, m)
}

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

    // Spawn markers
    spawn_pipeline: Option<wgpu::RenderPipeline>,
    spawn_group1_bind_group: Option<wgpu::BindGroup>,
    model_spawn: Option<ModelEnvelop<DefaultModel>>,
    spawn_cells: Vec<(f32, f32, u8)>,  // (cell_x, cell_y, tribe_index)

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
            model_spawn: None,
            spawn_cells: Vec::new(),
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

        // Rebuild spawn markers
        self.spawn_cells = extract_spawn_cells(&level_res);
        self.rebuild_spawn_model();
    }

    fn rebuild_spawn_model(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let cs = if self.curvature_enabled { self.curvature_scale } else { 0.0 };
            self.model_spawn = Some(build_spawn_model(&gpu.device, &self.spawn_cells, &self.landscape_mesh, cs));
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
        let mvp = MVP::new(&self.screen, &self.camera);
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

            // Draw spawn markers
            if let Some(ref spawn_pipeline) = self.spawn_pipeline {
                render_pass.set_pipeline(spawn_pipeline);
                render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                render_pass.set_bind_group(1, self.spawn_group1_bind_group.as_ref().unwrap(), &[]);
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

        // Spawn markers pipeline (TriangleList topology, same shader as objects)
        let spawn_shader_source = include_str!("../shaders/objects.wgsl");
        let spawn_vertex_layouts = DefaultModel::vertex_buffer_layouts();
        let spawn_pipeline = create_pipeline(
            device, spawn_shader_source, &spawn_vertex_layouts,
            &[&objects_group0_layout, &objects_group1_layout],
            gpu.surface_format(), true,
            wgpu::PrimitiveTopology::TriangleList,
            "spawn_pipeline",
        );

        // Spawn colors bind group
        let spawn_colors = spawn_tribe_colors();
        let spawn_params = ObjectParams { selected_frag: -1, num_colors: spawn_colors.len() as i32 };
        let spawn_params_buffer = GpuBuffer::new_uniform_init(device, bytemuck::bytes_of(&spawn_params), "spawn_params_buffer");
        let spawn_color_data: Vec<[u32; 4]> = spawn_colors.iter().map(|c| {
            [c.x as u32, c.y as u32, c.z as u32, 0u32]
        }).collect();
        let spawn_color_bytes: &[u8] = bytemuck::cast_slice(&spawn_color_data);
        let spawn_color_buffer = GpuBuffer::new_storage(device, spawn_color_bytes, "spawn_color_buffer");

        let spawn_group1_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("spawn_group1_bg"),
            layout: &objects_group1_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: spawn_params_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: spawn_color_buffer.buffer.as_entire_binding() },
            ],
        });

        // Empty spawn model (will be populated when level loads)
        let model_spawn = {
            let model: DefaultModel = MeshModel::new();
            let m = vec![(RenderType::Triangles, model)];
            ModelEnvelop::<DefaultModel>::new(device, m)
        };

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
        self.spawn_group1_bind_group = Some(spawn_group1_bind_group);
        self.model_spawn = Some(model_spawn);
        self.model_main = Some(model_main);
        self.model_select = Some(model_select);

        self.gpu = Some(gpu);

        // Build landscape variants (needs self.gpu, heights_buffer, etc.)
        let base2 = self.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type2 = self.config.landtype.as_deref();
        let level_res2 = LevelRes::new(&base2, self.level_num, level_type2);
        self.rebuild_landscape_variants(&level_res2);

        // Build spawn markers for initial level
        self.spawn_cells = extract_spawn_cells(&level_res2);
        self.rebuild_spawn_model();

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
                self.do_render = true;
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = Point2::<f32>::new(position.x as f32, position.y as f32);
            },
            WindowEvent::MouseInput { state, .. } => {
                if state == ElementState::Pressed {
                    let (v1, v2) = screen_to_scene(&self.screen, &self.camera, &self.mouse_pos);
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::KeyQ => {
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
                            _ => {
                                let prev_shift = self.landscape_mesh.get_shift_vector();
                                self.mode.process_key(key, &mut self.camera, &mut self.landscape_mesh);
                                if self.landscape_mesh.get_shift_vector() != prev_shift {
                                    self.rebuild_spawn_model();
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

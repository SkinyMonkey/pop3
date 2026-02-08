//! Pathfinding Demo — Dual-arm wall-following visualizer
//!
//! Renders a 128×128 cell grid with terrain coloring, lets you place start/goal
//! points, and visualizes the pathfinder's search: visited cells colored by arm,
//! the final path, and obstacle walls.
//!
//! Controls:
//!   Left-click    - Set start position
//!   Right-click   - Set goal position (triggers pathfinding)
//!   G             - Toggle cell grid overlay
//!   R             - Reset (clear path and markers)
//!   +/-           - Zoom in/out
//!   Arrow keys    - Pan camera
//!   N/P           - Next/previous level
//!   Escape        - Quit

use std::path::PathBuf;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, Command};

use pop3::gpu::context::GpuContext;
use pop3::gpu::buffer::GpuBuffer;
use pop3::gpu::texture::GpuTexture;
use pop3::pop::level::{Landscape, LevelRes};
use pop3::movement::{
    self, RegionMap, TileCoord, PathfindDebug, PathfindResult,
};
use pop3::movement::constants::REGION_GRID_SIZE;

const MAP_SIZE: usize = 128;
/// World units per cell
const CELL_SIZE: f32 = 256.0;
/// Full world size in world units
const WORLD_SIZE: f32 = MAP_SIZE as f32 * CELL_SIZE;

/// Max overlay vertices for cell fills + grid lines + path lines
const MAX_OVERLAY_VERTS: usize = 200_000;

// ──────────────────────────────────────────────────────────────────────────────
// Vertex types
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TerrainVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayVertex {
    position: [f32; 2],
    color: [f32; 4],
}

// ──────────────────────────────────────────────────────────────────────────────
// Heightmap → RGBA texture
// ──────────────────────────────────────────────────────────────────────────────

fn heightmap_to_rgba(landscape: &Landscape<128>) -> Vec<u8> {
    let mut rgba = vec![0u8; MAP_SIZE * MAP_SIZE * 4];
    for z in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let h = landscape.height[z][x];
            let idx = (z * MAP_SIZE + x) * 4;
            if h == 0 {
                rgba[idx] = 20;
                rgba[idx + 1] = 40;
                rgba[idx + 2] = 100;
            } else {
                let norm = (h as f32 / 1024.0).min(1.0);
                let r = (40.0 + norm * 180.0) as u8;
                let g = (100.0 + (1.0 - norm) * 100.0) as u8;
                let b = (30.0 + norm * 40.0) as u8;
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
            }
            rgba[idx + 3] = 255;
        }
    }
    rgba
}

// ──────────────────────────────────────────────────────────────────────────────
// Build region map from landscape (water = unwalkable)
// ──────────────────────────────────────────────────────────────────────────────

fn build_region_map(landscape: &Landscape<128>) -> RegionMap {
    let mut map = RegionMap::new();
    // Mark water cells (height == 0) as unwalkable
    map.set_terrain_flags(1, 0x00); // terrain class 1 = water (no walkable bit)
    for z in (0..MAP_SIZE).step_by(2) {
        for x in (0..MAP_SIZE).step_by(2) {
            let tile = TileCoord::new(x as u8, z as u8);
            // Average the 2×2 tile block height
            let h = landscape.height[z][x];
            if h == 0 {
                map.get_cell_mut(tile).terrain_type = 1; // water
            }
        }
    }
    map
}

// ──────────────────────────────────────────────────────────────────────────────
// Simulation state
// ──────────────────────────────────────────────────────────────────────────────

struct Simulation {
    region_map: RegionMap,
    start: Option<TileCoord>,
    goal: Option<TileCoord>,
    debug: Option<PathfindDebug>,
    show_grid: bool,
}

impl Simulation {
    fn new(landscape: &Landscape<128>) -> Self {
        Self {
            region_map: build_region_map(landscape),
            start: None,
            goal: None,
            debug: None,
            show_grid: true,
        }
    }

    fn set_start(&mut self, cell_x: usize, cell_z: usize) {
        // Cell coords to tile coords (tile = cell * 2)
        let tx = ((cell_x * 2) as u8) & 0xFE;
        let tz = ((cell_z * 2) as u8) & 0xFE;
        self.start = Some(TileCoord::new(tx, tz));
        self.debug = None;
        println!("Start: cell ({}, {}), tile ({:#x}, {:#x})", cell_x, cell_z, tx, tz);
        self.try_pathfind();
    }

    fn set_goal(&mut self, cell_x: usize, cell_z: usize) {
        let tx = ((cell_x * 2) as u8) & 0xFE;
        let tz = ((cell_z * 2) as u8) & 0xFE;
        self.goal = Some(TileCoord::new(tx, tz));
        println!("Goal: cell ({}, {}), tile ({:#x}, {:#x})", cell_x, cell_z, tx, tz);
        self.try_pathfind();
    }

    fn try_pathfind(&mut self) {
        if let (Some(start), Some(goal)) = (self.start, self.goal) {
            let debug = movement::pathfind_debug(&self.region_map, start, goal);
            let found = matches!(debug.result, PathfindResult::Found(_));
            println!(
                "Pathfind: {} | arm0: {} steps, arm1: {} steps",
                if found { "FOUND" } else { "NOT FOUND" },
                debug.arm0_trace.len(),
                debug.arm1_trace.len(),
            );
            self.debug = Some(debug);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Overlay geometry builders
// ──────────────────────────────────────────────────────────────────────────────

/// Push a filled quad (2 triangles) for a cell at (cx, cz) in cell-space.
fn push_cell_quad(verts: &mut Vec<OverlayVertex>, cx: f32, cz: f32, color: [f32; 4]) {
    let x0 = cx * CELL_SIZE;
    let z0 = cz * CELL_SIZE;
    let x1 = x0 + CELL_SIZE;
    let z1 = z0 + CELL_SIZE;
    // Triangle 1
    verts.push(OverlayVertex { position: [x0, z0], color });
    verts.push(OverlayVertex { position: [x1, z0], color });
    verts.push(OverlayVertex { position: [x1, z1], color });
    // Triangle 2
    verts.push(OverlayVertex { position: [x0, z0], color });
    verts.push(OverlayVertex { position: [x1, z1], color });
    verts.push(OverlayVertex { position: [x0, z1], color });
}

fn build_cell_overlay(sim: &Simulation) -> Vec<OverlayVertex> {
    let mut verts = Vec::new();

    // Visited cells from pathfinding debug
    if let Some(debug) = &sim.debug {
        // Color visited cells: use arm traces to distinguish which arm explored each cell
        let arm0_color = [0.0, 0.8, 0.9, 0.25]; // Cyan for arm 0 (right-hand)
        let arm1_color = [0.9, 0.2, 0.8, 0.25]; // Magenta for arm 1 (left-hand)
        let visited_color = [0.5, 0.5, 0.5, 0.15]; // Gray for cells visited but not in either trace

        // Build lookup sets for arm traces
        let arm0_set: std::collections::HashSet<(i32, i32)> =
            debug.arm0_trace.iter().copied().collect();
        let arm1_set: std::collections::HashSet<(i32, i32)> =
            debug.arm1_trace.iter().copied().collect();

        // Draw visited cells
        for z in 0..REGION_GRID_SIZE as i32 {
            for x in 0..REGION_GRID_SIZE as i32 {
                if debug.visited.is_visited(x, z) {
                    let color = if arm0_set.contains(&(x, z)) && arm1_set.contains(&(x, z)) {
                        // Both arms visited — blend
                        [0.5, 0.5, 0.9, 0.3]
                    } else if arm0_set.contains(&(x, z)) {
                        arm0_color
                    } else if arm1_set.contains(&(x, z)) {
                        arm1_color
                    } else {
                        visited_color
                    };
                    push_cell_quad(&mut verts, x as f32, z as f32, color);
                }
            }
        }

        // Draw arm traces as connected line segments (via thin quads)
        // Arm 0 trace (cyan line)
        for pair in debug.arm0_trace.windows(2) {
            let (x0, z0) = pair[0];
            let (x1, z1) = pair[1];
            push_line_quad(
                &mut verts,
                (x0 as f32 + 0.5) * CELL_SIZE,
                (z0 as f32 + 0.5) * CELL_SIZE,
                (x1 as f32 + 0.5) * CELL_SIZE,
                (z1 as f32 + 0.5) * CELL_SIZE,
                CELL_SIZE * 0.08,
                [0.0, 1.0, 1.0, 0.8],
            );
        }
        // Arm 1 trace (magenta line)
        for pair in debug.arm1_trace.windows(2) {
            let (x0, z0) = pair[0];
            let (x1, z1) = pair[1];
            push_line_quad(
                &mut verts,
                (x0 as f32 + 0.5) * CELL_SIZE,
                (z0 as f32 + 0.5) * CELL_SIZE,
                (x1 as f32 + 0.5) * CELL_SIZE,
                (z1 as f32 + 0.5) * CELL_SIZE,
                CELL_SIZE * 0.08,
                [1.0, 0.2, 0.8, 0.8],
            );
        }

        // Draw final path waypoints (yellow)
        if let PathfindResult::Found(ref wps) = debug.result {
            for pair in wps.windows(2) {
                let x0 = (pair[0].tile_x as f32 + 1.0) * (CELL_SIZE / 2.0);
                let z0 = (pair[0].tile_z as f32 + 1.0) * (CELL_SIZE / 2.0);
                let x1 = (pair[1].tile_x as f32 + 1.0) * (CELL_SIZE / 2.0);
                let z1 = (pair[1].tile_z as f32 + 1.0) * (CELL_SIZE / 2.0);
                push_line_quad(&mut verts, x0, z0, x1, z1, CELL_SIZE * 0.12, [1.0, 0.9, 0.0, 0.9]);
            }
            // Waypoint markers
            for wp in wps {
                let cx = (wp.tile_x as f32 + 1.0) * (CELL_SIZE / 2.0);
                let cz = (wp.tile_z as f32 + 1.0) * (CELL_SIZE / 2.0);
                let s = CELL_SIZE * 0.15;
                push_cell_quad_centered(&mut verts, cx, cz, s, [1.0, 1.0, 0.0, 0.9]);
            }
        }
    }

    // Start marker (green)
    if let Some(start) = sim.start {
        let cx = (start.x as f32 / 2.0 + 0.5) * CELL_SIZE;
        let cz = (start.z as f32 / 2.0 + 0.5) * CELL_SIZE;
        push_cell_quad_centered(&mut verts, cx, cz, CELL_SIZE * 0.3, [0.0, 1.0, 0.0, 0.9]);
    }

    // Goal marker (red)
    if let Some(goal) = sim.goal {
        let cx = (goal.x as f32 / 2.0 + 0.5) * CELL_SIZE;
        let cz = (goal.z as f32 / 2.0 + 0.5) * CELL_SIZE;
        push_cell_quad_centered(&mut verts, cx, cz, CELL_SIZE * 0.3, [1.0, 0.2, 0.2, 0.9]);
    }

    verts
}

fn build_grid_overlay(sim: &Simulation) -> Vec<OverlayVertex> {
    if !sim.show_grid {
        return Vec::new();
    }

    let mut verts = Vec::new();
    let grid_color = [1.0, 1.0, 1.0, 0.08];
    let thickness = CELL_SIZE * 0.02;

    // Vertical lines
    for x in 0..=MAP_SIZE {
        let wx = x as f32 * CELL_SIZE;
        push_line_quad(
            &mut verts, wx, 0.0, wx, WORLD_SIZE, thickness, grid_color,
        );
    }
    // Horizontal lines
    for z in 0..=MAP_SIZE {
        let wz = z as f32 * CELL_SIZE;
        push_line_quad(
            &mut verts, 0.0, wz, WORLD_SIZE, wz, thickness, grid_color,
        );
    }

    // Unwalkable cells (darken)
    let wall_color = [0.0, 0.0, 0.0, 0.4];
    for z in (0..MAP_SIZE).step_by(2) {
        for x in (0..MAP_SIZE).step_by(2) {
            let tile = TileCoord::new(x as u8, z as u8);
            if !sim.region_map.is_walkable(tile) {
                push_cell_quad(&mut verts, (x / 2) as f32, (z / 2) as f32, wall_color);
            }
        }
    }

    verts
}

/// Push a thin quad along a line segment (for rendering lines via triangles).
fn push_line_quad(
    verts: &mut Vec<OverlayVertex>,
    x0: f32, z0: f32, x1: f32, z1: f32,
    thickness: f32,
    color: [f32; 4],
) {
    let dx = x1 - x0;
    let dz = z1 - z0;
    let len = (dx * dx + dz * dz).sqrt().max(0.001);
    let nx = -dz / len * thickness;
    let nz = dx / len * thickness;

    let a = [x0 + nx, z0 + nz];
    let b = [x0 - nx, z0 - nz];
    let c = [x1 - nx, z1 - nz];
    let d = [x1 + nx, z1 + nz];

    verts.push(OverlayVertex { position: a, color });
    verts.push(OverlayVertex { position: b, color });
    verts.push(OverlayVertex { position: c, color });
    verts.push(OverlayVertex { position: a, color });
    verts.push(OverlayVertex { position: c, color });
    verts.push(OverlayVertex { position: d, color });
}

/// Push a centered square quad.
fn push_cell_quad_centered(
    verts: &mut Vec<OverlayVertex>,
    cx: f32, cz: f32, half_size: f32, color: [f32; 4],
) {
    let x0 = cx - half_size;
    let z0 = cz - half_size;
    let x1 = cx + half_size;
    let z1 = cz + half_size;
    verts.push(OverlayVertex { position: [x0, z0], color });
    verts.push(OverlayVertex { position: [x1, z0], color });
    verts.push(OverlayVertex { position: [x1, z1], color });
    verts.push(OverlayVertex { position: [x0, z0], color });
    verts.push(OverlayVertex { position: [x1, z1], color });
    verts.push(OverlayVertex { position: [x0, z1], color });
}

// ──────────────────────────────────────────────────────────────────────────────
// Camera
// ──────────────────────────────────────────────────────────────────────────────

struct Camera {
    center: [f32; 2],
    zoom: f32, // world units per pixel
}

impl Camera {
    fn new() -> Self {
        Self {
            center: [WORLD_SIZE / 2.0, WORLD_SIZE / 2.0],
            zoom: 20.0, // Larger view — ~20 world units per pixel
        }
    }

    fn projection(&self, screen_w: f32, screen_h: f32) -> [[f32; 4]; 4] {
        let hw = screen_w * self.zoom / 2.0;
        let hh = screen_h * self.zoom / 2.0;
        let l = self.center[0] - hw;
        let r = self.center[0] + hw;
        let b = self.center[1] + hh;
        let t = self.center[1] - hh;
        [
            [2.0 / (r - l),       0.0,              0.0, 0.0],
            [0.0,                 2.0 / (t - b),    0.0, 0.0],
            [0.0,                 0.0,              1.0, 0.0],
            [-(r + l) / (r - l), -(t + b) / (t - b), 0.0, 1.0],
        ]
    }

    fn screen_to_world(&self, sx: f32, sy: f32, screen_w: f32, screen_h: f32) -> [f32; 2] {
        let ndc_x = (sx / screen_w) * 2.0 - 1.0;
        let ndc_y = (sy / screen_h) * 2.0 - 1.0;
        let hw = screen_w * self.zoom / 2.0;
        let hh = screen_h * self.zoom / 2.0;
        [
            self.center[0] + ndc_x * hw,
            self.center[1] + ndc_y * hh,
        ]
    }

    fn world_to_cell(&self, world: [f32; 2]) -> Option<(usize, usize)> {
        let cx = (world[0] / CELL_SIZE) as i32;
        let cz = (world[1] / CELL_SIZE) as i32;
        if cx >= 0 && cx < MAP_SIZE as i32 && cz >= 0 && cz < MAP_SIZE as i32 {
            Some((cx as usize, cz as usize))
        } else {
            None
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Application
// ──────────────────────────────────────────────────────────────────────────────

struct App {
    window: Option<Arc<Window>>,
    state: Option<ViewerState>,
    base_path: PathBuf,
    level_num: u8,
    landscape: Landscape<128>,
    sim: Simulation,
}

struct ViewerState {
    gpu: GpuContext,
    terrain_pipeline: wgpu::RenderPipeline,
    cell_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    terrain_texture: GpuTexture,
    sampler: wgpu::Sampler,
    uniform_buffer: GpuBuffer,
    terrain_vertex_buffer: GpuBuffer,
    cell_buffer: GpuBuffer,
    cell_vert_count: u32,
    camera: Camera,
    cursor_pos: [f32; 2],
}

impl App {
    fn new(base_path: PathBuf, level_num: u8) -> Self {
        let level = LevelRes::new(&base_path, level_num, None);
        let sim = Simulation::new(&level.landscape);
        Self {
            window: None,
            state: None,
            base_path,
            level_num,
            landscape: level.landscape,
            sim,
        }
    }

    fn load_level(&mut self, level_num: u8) {
        self.level_num = level_num;
        let level = LevelRes::new(&self.base_path, level_num, None);
        self.sim = Simulation::new(&level.landscape);
        self.landscape = level.landscape;

        if let Some(state) = &mut self.state {
            let rgba = heightmap_to_rgba(&self.landscape);
            state.terrain_texture = GpuTexture::new_2d(
                &state.gpu.device, &state.gpu.queue,
                MAP_SIZE as u32, MAP_SIZE as u32,
                wgpu::TextureFormat::Rgba8UnormSrgb, &rgba, "terrain_texture",
            );
            state.bind_group = state.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("pathfinding_bg"),
                layout: &state.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: state.uniform_buffer.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&state.terrain_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&state.sampler),
                    },
                ],
            });
            state.camera = Camera::new();
        }
        if let Some(window) = &self.window {
            window.set_title(&format!("Pathfinding Demo — Level {}", level_num));
        }
        println!("Loaded level {}", level_num);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(format!("Pathfinding Demo — Level {}", self.level_num))
                        .with_inner_size(winit::dpi::LogicalSize::new(1200, 1200)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        let rgba = heightmap_to_rgba(&self.landscape);
        let terrain_texture = GpuTexture::new_2d(
            device, &gpu.queue, MAP_SIZE as u32, MAP_SIZE as u32,
            wgpu::TextureFormat::Rgba8UnormSrgb, &rgba, "terrain_texture",
        );
        let sampler = GpuTexture::create_sampler(device, true);
        let uniform_buffer = GpuBuffer::new_uniform(device, 64, "pathfinding_uniforms");

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pathfinding_bgl"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pathfinding_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader_source = include_str!("../../shaders/pathfinding_demo.wgsl");

        // Terrain pipeline
        let terrain_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pathfinding_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let terrain_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TerrainVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            ],
        };

        let terrain_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &terrain_shader,
                entry_point: Some("vs_terrain"),
                buffers: &[terrain_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader,
                entry_point: Some("fs_terrain"),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Cell overlay pipeline — TriangleList with alpha blending
        let overlay_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OverlayVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x4 },
            ],
        };

        let overlay_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let cell_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cell_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &overlay_shader,
                entry_point: Some("vs_overlay"),
                buffers: &[overlay_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &overlay_shader,
                entry_point: Some("fs_overlay"),
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

        let terrain_quad = [
            TerrainVertex { position: [0.0, 0.0], uv: [0.0, 0.0] },
            TerrainVertex { position: [WORLD_SIZE, 0.0], uv: [1.0, 0.0] },
            TerrainVertex { position: [WORLD_SIZE, WORLD_SIZE], uv: [1.0, 1.0] },
            TerrainVertex { position: [0.0, 0.0], uv: [0.0, 0.0] },
            TerrainVertex { position: [WORLD_SIZE, WORLD_SIZE], uv: [1.0, 1.0] },
            TerrainVertex { position: [0.0, WORLD_SIZE], uv: [0.0, 1.0] },
        ];
        let terrain_vertex_buffer = GpuBuffer::new_vertex(
            device, bytemuck::cast_slice(&terrain_quad), "terrain_quad",
        );

        let cell_buffer = GpuBuffer::new_vertex(
            device,
            &vec![0u8; MAX_OVERLAY_VERTS * std::mem::size_of::<OverlayVertex>()],
            "cell_overlay",
        );

        self.state = Some(ViewerState {
            gpu,
            terrain_pipeline,
            cell_pipeline,
            bind_group_layout,
            bind_group,
            terrain_texture,
            sampler,
            uniform_buffer,
            terrain_vertex_buffer,
            cell_buffer,
            cell_vert_count: 0,
            camera: Camera::new(),
            cursor_pos: [0.0, 0.0],
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _wid: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.state {
                    state.gpu.resize(size);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(state) = &mut self.state {
                    state.cursor_pos = [position.x as f32, position.y as f32];
                }
            }
            WindowEvent::MouseInput { state: btn_state, button, .. } => {
                if btn_state == ElementState::Pressed {
                    if let Some(vs) = &self.state {
                        let sw = vs.gpu.size.width as f32;
                        let sh = vs.gpu.size.height as f32;
                        let world = vs.camera.screen_to_world(vs.cursor_pos[0], vs.cursor_pos[1], sw, sh);
                        if let Some((cx, cz)) = vs.camera.world_to_cell(world) {
                            match button {
                                MouseButton::Left => self.sim.set_start(cx, cz),
                                MouseButton::Right => self.sim.set_goal(cx, cz),
                                _ => {}
                            }
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::Escape => { event_loop.exit(); return; }
                            KeyCode::KeyG => {
                                self.sim.show_grid = !self.sim.show_grid;
                                println!("Grid: {}", if self.sim.show_grid { "ON" } else { "OFF" });
                            }
                            KeyCode::KeyR => {
                                self.sim = Simulation::new(&self.landscape);
                                println!("Reset");
                            }
                            KeyCode::Equal | KeyCode::NumpadAdd => {
                                if let Some(state) = &mut self.state {
                                    state.camera.zoom = (state.camera.zoom * 0.8).max(0.5);
                                }
                            }
                            KeyCode::Minus | KeyCode::NumpadSubtract => {
                                if let Some(state) = &mut self.state {
                                    state.camera.zoom = (state.camera.zoom * 1.25).min(200.0);
                                }
                            }
                            KeyCode::ArrowLeft => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[0] -= 200.0 * state.camera.zoom;
                                }
                            }
                            KeyCode::ArrowRight => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[0] += 200.0 * state.camera.zoom;
                                }
                            }
                            KeyCode::ArrowUp => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[1] -= 200.0 * state.camera.zoom;
                                }
                            }
                            KeyCode::ArrowDown => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[1] += 200.0 * state.camera.zoom;
                                }
                            }
                            KeyCode::KeyN => {
                                let next = self.level_num.wrapping_add(1).max(1);
                                self.load_level(next);
                            }
                            KeyCode::KeyP => {
                                let prev = if self.level_num <= 1 { 25 } else { self.level_num - 1 };
                                self.load_level(prev);
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.render(&self.sim);
                }
            }
            _ => {}
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ViewerState {
    fn render(&mut self, sim: &Simulation) {
        let sw = self.gpu.size.width as f32;
        let sh = self.gpu.size.height as f32;

        let proj = self.camera.projection(sw, sh);
        self.gpu.queue.write_buffer(&self.uniform_buffer.buffer, 0, bytemuck::bytes_of(&proj));

        let output = match self.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("pathfinding_encoder") },
        );

        // Build overlay geometry
        let mut overlay_verts = build_grid_overlay(sim);
        overlay_verts.extend(build_cell_overlay(sim));
        self.cell_vert_count = (overlay_verts.len() as u32).min(MAX_OVERLAY_VERTS as u32);
        if self.cell_vert_count > 0 {
            self.gpu.queue.write_buffer(
                &self.cell_buffer.buffer, 0,
                bytemuck::cast_slice(&overlay_verts[..self.cell_vert_count as usize]),
            );
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pathfinding_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02, g: 0.02, b: 0.05, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // Draw terrain
            pass.set_pipeline(&self.terrain_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.terrain_vertex_buffer.buffer.slice(..));
            pass.draw(0..6, 0..1);

            // Draw cell overlay
            if self.cell_vert_count > 0 {
                pass.set_pipeline(&self.cell_pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.cell_buffer.buffer.slice(..));
                pass.draw(0..self.cell_vert_count, 0..1);
            }
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// CLI & main
// ──────────────────────────────────────────────────────────────────────────────

fn cli() -> Command {
    Command::new("pathfinding-demo")
        .about("Pathfinding visualizer for Populous: The Beginning")
        .arg(
            Arg::new("base")
                .long("base")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Path to game install directory"),
        )
        .arg(
            Arg::new("level")
                .long("level")
                .short('l')
                .value_parser(clap::value_parser!(u8))
                .default_value("1")
                .help("Level number (1-25)"),
        )
}

fn main() {
    let matches = cli().get_matches();

    let base = matches
        .get_one::<PathBuf>("base")
        .cloned()
        .unwrap_or_else(|| {
            let candidates = [
                PathBuf::from("data/original_game"),
                PathBuf::from("../data/original_game"),
            ];
            for c in &candidates {
                if c.join("levels").exists() {
                    return c.clone();
                }
            }
            PathBuf::from("data/original_game")
        });

    let level_num = *matches.get_one::<u8>("level").unwrap();

    println!("Pathfinding Demo");
    println!("  Left-click:  set start");
    println!("  Right-click: set goal (triggers pathfinding)");
    println!("  G:           toggle grid");
    println!("  R:           reset");
    println!("  +/-:         zoom");
    println!("  Arrows:      pan");
    println!("  N/P:         next/prev level");
    println!();
    println!("  Cyan cells:    arm 0 (right-hand wall-follow)");
    println!("  Magenta cells: arm 1 (left-hand wall-follow)");
    println!("  Yellow line:   final path");

    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", "info")
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(base, level_num);
    event_loop.run_app(&mut app).unwrap();
}

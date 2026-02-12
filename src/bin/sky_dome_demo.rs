//! Sky Dome Demo
//!
//! Demonstrates the sky hemisphere projection from the original game.
//! Implements all 4 rendering modes discovered via Ghidra disassembly.
//!
//! Controls:
//!   Q/E         - Rotate camera yaw (sky parallax)
//!   W/S         - Tilt view (adjust V origin)
//!   Up/Down     - Adjust horizon curvature
//!   1/2/3/4     - Switch render mode (Dome/Simple/Parallax/Flat)
//!   N/P         - Next/Previous sky theme
//!   Space       - Toggle auto-rotation
//!   R           - Reset camera
//!   Escape      - Quit

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, Command};

use pop3::gpu::context::GpuContext;
use pop3::gpu::pipeline::create_pipeline;
use pop3::gpu::buffer::GpuBuffer;
use pop3::gpu::texture::GpuTexture;

/******************************************************************************/
// Data loading
/******************************************************************************/

const KEYS: &[&str] = &[
    "0","1","2","3","4","5","6","7","8","9",
    "a","b","c","d","e","f","g","h","i","j",
    "k","l","m","n","o","p","q","r","s","t",
    "u","v","w","x","y","z",
];

struct SkyVariant {
    key: String,
    sky_path: PathBuf,
    pal_path: PathBuf,
}

fn discover_variants(data_dir: &Path) -> Vec<SkyVariant> {
    let mut variants = Vec::new();
    for key in KEYS {
        let sky_path = data_dir.join(format!("sky0-{key}.dat"));
        let pal_path = data_dir.join(format!("pal0-{key}.dat"));
        if sky_path.exists() && pal_path.exists() {
            variants.push(SkyVariant {
                key: key.to_string(),
                sky_path,
                pal_path,
            });
        }
    }
    variants
}

fn load_sky_rgba(variant: &SkyVariant) -> (Vec<u8>, u32, u32) {
    let sky_raw = std::fs::read(&variant.sky_path).expect("Failed to read sky file");
    let pal = std::fs::read(&variant.pal_path).expect("Failed to read palette file");

    // Detect dimensions from file size
    let (width, height) = match sky_raw.len() {
        307200 => (640usize, 480usize),  // sky0-{c}.dat = 640×480
        262144 => (512usize, 512usize),  // MSKY0-{c}.DAT = 512×512
        other => {
            let w = 512usize;
            let h = other / w;
            eprintln!("  [DEBUG] Unknown sky size {}, assuming {}x{}", other, w, h);
            (w, h)
        }
    };

    let pixel_count = width * height;
    let indices = &sky_raw[..pixel_count.min(sky_raw.len())];

    // Debug: analyze raw index distribution
    let min_idx = indices.iter().copied().min().unwrap_or(0);
    let max_idx = indices.iter().copied().max().unwrap_or(0);
    let mut hist = [0u32; 256];
    for &idx in indices { hist[idx as usize] += 1; }
    let unique_count = hist.iter().filter(|&&c| c > 0).count();

    println!("  [DEBUG] sky0-{}.dat: {} bytes → {}x{}", variant.key, sky_raw.len(), width, height);
    println!("  [DEBUG] palette: {} bytes ({} entries)", pal.len(), pal.len() / 4);
    println!("  [DEBUG] index range: [{}-{}] (0x{:02x}-0x{:02x}), {} unique values",
        min_idx, max_idx, min_idx, max_idx, unique_count);

    // Debug: show palette colors at the REMAPPED index range (after offset)
    let pal_offset_preview: u8 = if width == 640 { 0 } else { 0x70 };
    if pal.len() >= 256 * 4 {
        let remap_min = min_idx.wrapping_add(pal_offset_preview);
        let remap_max = max_idx.wrapping_add(pal_offset_preview);
        print!("  [DEBUG] palette at [{}-{}] (0x{:02x}-0x{:02x}): ",
            remap_min, remap_max, remap_min, remap_max);
        for i in remap_min..=remap_max {
            let off = i as usize * 4;
            print!("#{:02x}{:02x}{:02x} ", pal[off], pal[off+1], pal[off+2]);
        }
        println!();
    }

    // Debug: sample first row pixel values
    print!("  [DEBUG] row 0 samples (every 64px): ");
    for x in (0..width).step_by(64) {
        print!("[x={}]={} ", x, indices[x]);
    }
    println!();

    // Debug: sample first column pixel values
    print!("  [DEBUG] col 0 samples (every 64px): ");
    for y in (0..height).step_by(64) {
        print!("[y={}]={} ", y, indices[y * width]);
    }
    println!();

    // Debug: check if data varies more by row or column
    let mut row_variance = 0u64;
    let mut col_variance = 0u64;
    for y in 0..height.min(100) {
        for x in 1..width {
            let diff = (indices[y * width + x] as i32 - indices[y * width + x - 1] as i32).unsigned_abs() as u64;
            row_variance += diff;
        }
    }
    for x in 0..width.min(100) {
        for y in 1..height {
            let diff = (indices[y * width + x] as i32 - indices[(y - 1) * width + x] as i32).unsigned_abs() as u64;
            col_variance += diff;
        }
    }
    println!("  [DEBUG] horizontal variance: {}, vertical variance: {} ({})",
        row_variance, col_variance,
        if row_variance > col_variance * 2 { "MORE horizontal variation → gradient is vertical"
        } else if col_variance > row_variance * 2 { "MORE vertical variation → gradient is horizontal"
        } else { "similar in both directions" });

    // 640×480 files have absolute palette indices (100-127).
    // 512×512 files have relative indices (1-14) needing +0x70 → palette 0x71-0x7E.
    let pal_offset: u8 = if width == 640 { 0 } else { 0x70 };
    println!("  [DEBUG] palette offset: +0x{:02x} ({})",
        pal_offset, if pal_offset == 0 { "absolute" } else { "relative, MSKY format" });

    let mut rgba = vec![0u8; pixel_count * 4];
    for (i, &idx) in indices.iter().enumerate() {
        let pal_idx = idx.wrapping_add(pal_offset) as usize;
        let off = pal_idx * 4;
        if off + 2 < pal.len() {
            rgba[i * 4]     = pal[off];
            rgba[i * 4 + 1] = pal[off + 1];
            rgba[i * 4 + 2] = pal[off + 2];
            rgba[i * 4 + 3] = 255;
        }
    }
    (rgba, width as u32, height as u32)
}

/******************************************************************************/
// Uniform data — matches SkyDomeParams in sky_dome.wgsl
/******************************************************************************/

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyDomeParams {
    u_origin: f32,
    v_origin: f32,
    cos_angle: f32,
    sin_angle: f32,
    u_scale: f32,
    v_scale: f32,
    horizon: f32,
    mode: u32,
}

const MODE_NAMES: &[&str] = &["Dome (Mode 0)", "Simple (Mode 1)", "Parallax (Mode 2)", "Flat (Mode 3)"];

/******************************************************************************/
// Camera state — implements Sky_UpdateRotation / Sky_ComputeParams logic
/******************************************************************************/

struct SkyCamera {
    yaw: f32,           // camera yaw in radians
    u_acc: f32,         // accumulated U panning
    v_acc: f32,         // accumulated V panning
    angle: f32,         // rotation angle in radians
    horizon: f32,       // hemisphere curvature
    mode: u32,          // render mode 0-3
    auto_rotate: bool,
}

impl SkyCamera {
    fn new() -> Self {
        Self {
            yaw: 0.0,
            u_acc: 0.0,
            v_acc: 0.0,
            angle: 0.0,
            // Original game: horizon is param[0xb] from the projection struct.
            // A reasonable default that gives visible dome curvature.
            horizon: 1.5,
            mode: 0,
            auto_rotate: true,
        }
    }

    fn reset(&mut self) {
        self.yaw = 0.0;
        self.u_acc = 0.0;
        self.v_acc = 0.0;
        self.angle = 0.0;
        self.horizon = 1.5;
    }

    /// Matches Sky_UpdateRotation (0x004dc710):
    /// Rotate the UV delta by current angle, accumulate.
    fn update_rotation(&mut self, dyaw: f32, du: f32, dv: f32) {
        let cos_a = self.angle.cos();
        let sin_a = self.angle.sin();

        self.u_acc += du * cos_a + dv * sin_a;
        self.v_acc += dv * cos_a - du * sin_a;
        self.angle += dyaw * 0.5;
    }

    /// Matches Sky_ComputeParams (0x004dc850):
    /// Convert accumulated state into shader params.
    fn to_params(&self) -> SkyDomeParams {
        SkyDomeParams {
            u_origin: self.u_acc,
            v_origin: self.v_acc,
            cos_angle: self.angle.cos(),
            sin_angle: self.angle.sin(),
            // Original: 0x2800000 / visible_width and 0xC80000 / visible_height
            // Normalized to floating point; these control the "zoom" of the dome
            u_scale: 2.5,
            v_scale: 1.8,
            horizon: self.horizon,
            mode: self.mode,
        }
    }
}

/******************************************************************************/
// Application
/******************************************************************************/

struct App {
    window: Option<Arc<Window>>,
    state: Option<DemoState>,
    variants: Vec<SkyVariant>,
    current_index: usize,
    camera: SkyCamera,
    start_time: Instant,
}

struct DemoState {
    gpu: GpuContext,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    sky_texture: GpuTexture,
    sampler: wgpu::Sampler,
    uniform_buffer: GpuBuffer,
}

impl App {
    fn new(variants: Vec<SkyVariant>) -> Self {
        Self {
            window: None,
            state: None,
            variants,
            current_index: 0,
            camera: SkyCamera::new(),
            start_time: Instant::now(),
        }
    }

    fn load_variant(&mut self, index: usize) {
        self.current_index = index;
        let variant = &self.variants[index];
        println!("Loading sky0-{}.dat", variant.key);
        let (rgba, w, h) = load_sky_rgba(variant);

        if let Some(state) = &mut self.state {
            state.sky_texture = GpuTexture::new_2d(
                &state.gpu.device,
                &state.gpu.queue,
                w, h,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                &rgba,
                "sky_texture",
            );
            state.bind_group = state.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sky_dome_bg"),
                layout: &state.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: state.uniform_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&state.sky_texture.view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&state.sampler) },
                ],
            });
        }

        self.update_title();
    }

    fn update_title(&self) {
        if let Some(window) = &self.window {
            let variant = &self.variants[self.current_index];
            window.set_title(&format!(
                "Sky Dome Demo — sky0-{}.dat [{}/{}] — {} — horizon={:.1} {}",
                variant.key,
                self.current_index + 1,
                self.variants.len(),
                MODE_NAMES[self.camera.mode as usize],
                self.camera.horizon,
                if self.camera.auto_rotate { "[auto]" } else { "" },
            ));
        }
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
                        .with_title("Sky Dome Demo")
                        .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        // Load initial sky
        let variant = &self.variants[self.current_index];
        let (rgba, w, h) = load_sky_rgba(variant);

        let sky_texture = GpuTexture::new_2d(
            device, &gpu.queue, w, h,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &rgba, "sky_texture",
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sky_dome_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = GpuBuffer::new_uniform_init(
            device,
            bytemuck::bytes_of(&self.camera.to_params()),
            "sky_dome_uniform",
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky_dome_bgl"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky_dome_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&sky_texture.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        let shader_source = include_str!("../../shaders/sky_dome.wgsl");
        let pipeline = create_pipeline(
            device, shader_source, &[], &[&bind_group_layout],
            gpu.surface_format(), false,
            wgpu::PrimitiveTopology::TriangleList,
            "sky_dome_pipeline",
        );

        self.state = Some(DemoState {
            gpu, pipeline, bind_group_layout, bind_group,
            sky_texture, sampler, uniform_buffer,
        });

        self.update_title();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _wid: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.state {
                    state.gpu.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::Escape => { event_loop.exit(); return; }

                            // Camera yaw (Sky_UpdateRotation: yaw drives parallax)
                            KeyCode::KeyQ => self.camera.update_rotation(-0.08, 0.0, 0.0),
                            KeyCode::KeyE => self.camera.update_rotation(0.08, 0.0, 0.0),

                            // V panning
                            KeyCode::KeyW => self.camera.v_acc -= 0.02,
                            KeyCode::KeyS => self.camera.v_acc += 0.02,

                            // Horizon curvature
                            KeyCode::ArrowUp => {
                                self.camera.horizon += 0.1;
                                self.update_title();
                            }
                            KeyCode::ArrowDown => {
                                self.camera.horizon = (self.camera.horizon - 0.1).max(0.1);
                                self.update_title();
                            }

                            // Mode switching
                            KeyCode::Digit1 => { self.camera.mode = 0; self.update_title(); }
                            KeyCode::Digit2 => { self.camera.mode = 1; self.update_title(); }
                            KeyCode::Digit3 => { self.camera.mode = 2; self.update_title(); }
                            KeyCode::Digit4 => { self.camera.mode = 3; self.update_title(); }

                            // Theme cycling
                            KeyCode::KeyN | KeyCode::ArrowRight => {
                                let next = (self.current_index + 1) % self.variants.len();
                                self.load_variant(next);
                            }
                            KeyCode::KeyP | KeyCode::ArrowLeft => {
                                let prev = if self.current_index == 0 {
                                    self.variants.len() - 1
                                } else {
                                    self.current_index - 1
                                };
                                self.load_variant(prev);
                            }

                            // Auto-rotate toggle
                            KeyCode::Space => {
                                self.camera.auto_rotate = !self.camera.auto_rotate;
                                self.update_title();
                            }

                            // Reset
                            KeyCode::KeyR => {
                                self.camera.reset();
                                self.update_title();
                            }

                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Auto-rotation (simulates camera yaw changing over time)
                if self.camera.auto_rotate {
                    let t = self.start_time.elapsed().as_secs_f32();
                    let rate = 0.003;
                    self.camera.update_rotation(rate, rate * 0.5, 0.0);
                    // Gentle V oscillation for visual interest
                    self.camera.v_acc = 0.1 * (t * 0.2).sin();
                }

                if let Some(state) = &mut self.state {
                    // Update uniform
                    let params = self.camera.to_params();
                    state.gpu.queue.write_buffer(
                        &state.uniform_buffer.buffer, 0,
                        bytemuck::bytes_of(&params),
                    );

                    // Render
                    let output = match state.gpu.surface.get_current_texture() {
                        Ok(t) => t,
                        Err(_) => return,
                    };
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = state.gpu.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor { label: Some("sky_dome_enc") },
                    );
                    {
                        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("sky_dome_pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            ..Default::default()
                        });
                        pass.set_pipeline(&state.pipeline);
                        pass.set_bind_group(0, &state.bind_group, &[]);
                        pass.draw(0..3, 0..1);
                    }
                    state.gpu.queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                }
            }
            _ => {}
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/******************************************************************************/
// CLI & main
/******************************************************************************/

fn main() {
    let matches = Command::new("sky-dome-demo")
        .about("Sky hemisphere projection demo — all 4 original game modes")
        .arg(
            Arg::new("base")
                .long("base")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Path to game data directory"),
        )
        .get_matches();

    let base = matches
        .get_one::<PathBuf>("base")
        .cloned()
        .unwrap_or_else(|| PathBuf::from("data/original_game"));

    let data_dir = base.join("data");
    let variants = discover_variants(&data_dir);

    if variants.is_empty() {
        eprintln!("No sky variants found in {:?}", data_dir);
        eprintln!("Expected files like sky0-0.dat + pal0-0.dat");
        std::process::exit(1);
    }

    println!("Sky Dome Demo");
    println!("Found {} sky variants", variants.len());
    println!();
    println!("Controls:");
    println!("  Q/E       Rotate camera yaw (parallax)");
    println!("  W/S       Pan vertically");
    println!("  Up/Down   Adjust horizon curvature");
    println!("  1/2/3/4   Switch mode (Dome/Simple/Parallax/Flat)");
    println!("  N/P       Next/Previous sky theme");
    println!("  Space     Toggle auto-rotation");
    println!("  R         Reset camera");
    println!("  Escape    Quit");

    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", "info")
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(variants);
    event_loop.run_app(&mut app).unwrap();
}

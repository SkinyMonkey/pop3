//! Spell Effect Demo
//!
//! Tests the engine spell-effect pipeline: pick an effect type, spawn it into the
//! real `EffectPool`, and watch the actual spawn -> tick -> gravity/loop/expire
//! lifecycle, drawn with real HSPR sprite frames.
//!
//! Spells in PopRe are Things selected by `EffectType` (+0x14) / `EffectModel`
//! (+0x15) into the shared model table (see docs/specs/v2/effects.md). We don't
//! yet have the per-spell `EffectModel` constants (deferred to a runtime
//! capture), so this is also a *sprite-bank browser*: scrub the sprite range
//! live to find each effect's frames, then spawn to verify the lifecycle.
//!
//! Controls:
//!   1-9,0,Q,W,E  - select EffectType 0x01..0x0C (Burn..Shield)
//!   Space        - spawn one effect at center with current type + sprite range
//!   [ / ]        - sprite start -1 / +1     (hunt for effect frames)
//!   - / =        - sprite start -10 / +10
//!   , / .        - frame count -1 / +1
//!   Backspace    - clear all active effects
//!   P            - pause/resume the effect tick
//!   Esc          - quit

use std::path::{Path, PathBuf};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, Command};

use pop3::render::gpu::buffer::GpuBuffer;
use pop3::render::gpu::context::GpuContext;
use pop3::render::gpu::pipeline::create_pipeline;
use pop3::render::gpu::texture::GpuTexture;

use pop3::data::psfb::ContainerPSFB;
use pop3::data::types::BinDeserializer;

use pop3::engine::effects::types::effect_defaults;
use pop3::engine::effects::{spawn::spawn_at, EffectPool, EFFECT_GRAVITY, EFFECT_LOOP, MAX_EFFECTS};

/// Engine effect type id -> spell name (see effects::spawn::spawn_on_spell_impact).
const SPELL_NAMES: [(u8, &str); 12] = [
    (0x01, "Burn"),
    (0x02, "Blast"),
    (0x03, "Lightning"),
    (0x04, "Tornado"),
    (0x05, "Swamp"),
    (0x06, "Flatten"),
    (0x07, "Earthquake"),
    (0x08, "Erosion"),
    (0x09, "Volcano"),
    (0x0A, "Firestorm"),
    (0x0B, "AngelOfDeath"),
    (0x0C, "Shield"),
];

fn spell_name(effect_type: u8) -> &'static str {
    SPELL_NAMES
        .iter()
        .find(|(t, _)| *t == effect_type)
        .map(|(_, n)| *n)
        .unwrap_or("?")
}

/// Max effects drawn per frame (one bind group / uniform buffer each).
const MAX_DRAW: usize = 128;
/// World units per screen pixel (calibration knob — effects spawn in world space).
// ponytail: hand-tuned so a spawned arc stays on screen; adjust if motion clips.
const WORLD_PER_PX: f32 = 6.0;
/// Base on-screen half-height (px) of an effect at scale 0x100.
const BASE_SPRITE_PX: f32 = 48.0;

/******************************************************************************/
// Palette + atlas
/******************************************************************************/

fn load_palette(path: &Path) -> Option<Vec<[u8; 4]>> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 1024 {
        return None;
    }
    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let off = i * 4;
        palette.push([data[off], data[off + 1], data[off + 2], 255]);
    }
    Some(palette)
}

struct SpriteAtlas {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    frame_width: u32,
    frame_height: u32,
    frames: u32,
}

/// Build a single-row atlas of `count` sprites starting at `start`.
fn build_atlas(container: &ContainerPSFB, palette: &[[u8; 4]], start: usize, count: usize) -> Option<SpriteAtlas> {
    let count = count.max(1);
    let mut max_w: u16 = 0;
    let mut max_h: u16 = 0;
    for f in 0..count {
        if let Some(info) = container.get_info(start + f) {
            max_w = max_w.max(info.width);
            max_h = max_h.max(info.height);
        }
    }
    if max_w == 0 || max_h == 0 {
        return None;
    }

    let fw = max_w as u32;
    let fh = max_h as u32;
    let atlas_w = fw * count as u32;
    let atlas_h = fh;
    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    for f in 0..count {
        let idx = start + f;
        if let (Some(image), Some(info)) = (container.get_image(idx), container.get_info(idx)) {
            let sw = info.width as u32;
            let sh = info.height as u32;
            let ox = (fw - sw) / 2;
            let oy = (fh - sh) / 2;
            let cell_x = f as u32 * fw;
            for y in 0..sh {
                for x in 0..sw {
                    let src = image.data[(y * sw + x) as usize];
                    if src == 0 {
                        continue; // transparent (rgba already 0)
                    }
                    let c = palette.get(src as usize).unwrap_or(&[255, 0, 255, 255]);
                    let dst_x = cell_x + ox + x;
                    let dst_y = oy + y;
                    let dst = ((dst_y * atlas_w + dst_x) * 4) as usize;
                    rgba[dst] = c[0];
                    rgba[dst + 1] = c[1];
                    rgba[dst + 2] = c[2];
                    rgba[dst + 3] = 255;
                }
            }
        }
    }

    Some(SpriteAtlas {
        rgba,
        width: atlas_w,
        height: atlas_h,
        frame_width: fw,
        frame_height: fh,
        frames: count as u32,
    })
}

/******************************************************************************/
// GPU types
/******************************************************************************/

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteUniforms {
    projection: [[f32; 4]; 4],
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
    mirror: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

/// Unit quad centered at origin, corners +/-1.
fn unit_quad() -> [SpriteVertex; 6] {
    [
        SpriteVertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
        SpriteVertex { position: [1.0, -1.0], uv: [1.0, 1.0] },
        SpriteVertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
        SpriteVertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
        SpriteVertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
        SpriteVertex { position: [-1.0, 1.0], uv: [0.0, 0.0] },
    ]
}

/// Affine placement baked into the projection matrix (column-major for WGSL):
/// clip = (sx*x + tx, sy*y + ty).  px/py are pixels from screen center.
fn place_matrix(px: f32, py: f32, half_w_px: f32, half_h_px: f32, w: f32, h: f32) -> [[f32; 4]; 4] {
    let sx = half_w_px / (w * 0.5);
    let sy = half_h_px / (h * 0.5);
    let tx = px / (w * 0.5);
    let ty = py / (h * 0.5);
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [tx, ty, 0.0, 1.0],
    ]
}

/******************************************************************************/
// Application
/******************************************************************************/

struct App {
    window: Option<Arc<Window>>,
    state: Option<DemoState>,
    container: ContainerPSFB,
    palette: Vec<[u8; 4]>,
}

struct DemoState {
    gpu: GpuContext,
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    bind_groups: Vec<wgpu::BindGroup>,
    uniform_buffers: Vec<GpuBuffer>,
    quad: GpuBuffer,
    sampler: wgpu::Sampler,
    atlas_tex: GpuTexture,
    atlas: SpriteAtlas,

    pool: EffectPool,
    effect_type: u8,
    sprite_start: usize,
    frame_count: usize,
    paused: bool,
    rng: u32,
    tick_timer: f32,
    last: std::time::Instant,
}

fn xorshift(s: &mut u32) -> u32 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *s = x;
    x
}

impl DemoState {
    fn rebuild_atlas(&mut self, container: &ContainerPSFB, palette: &[[u8; 4]]) {
        match build_atlas(container, palette, self.sprite_start, self.frame_count) {
            Some(atlas) => {
                self.atlas_tex = GpuTexture::new_2d(
                    &self.gpu.device,
                    &self.gpu.queue,
                    atlas.width,
                    atlas.height,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    &atlas.rgba,
                    "effect_atlas",
                );
                self.rebuild_bind_groups();
                println!(
                    "sprites {}..{} ({} frames, cell {}x{})",
                    self.sprite_start,
                    self.sprite_start + atlas.frames as usize,
                    atlas.frames,
                    atlas.frame_width,
                    atlas.frame_height
                );
                self.atlas = atlas;
            }
            None => println!("(no drawable sprites at {} x{})", self.sprite_start, self.frame_count),
        }
    }

    fn rebuild_bind_groups(&mut self) {
        self.bind_groups.clear();
        for i in 0..MAX_DRAW {
            let bg = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("effect_bg"),
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.uniform_buffers[i].buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&self.atlas_tex.view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                ],
            });
            self.bind_groups.push(bg);
        }
    }

    fn spawn(&mut self) {
        // Spawn via the real engine path: spawn_at applies effect_defaults
        // (max_frame, flags, scale, alpha) for the chosen effect type.
        if let Some(id) = spawn_at(&mut self.pool, self.effect_type, 0, 0, 0, 0) {
            // Give it some launch velocity so the lifecycle/arc is visible.
            let vx = (xorshift(&mut self.rng) % 2048) as i32 - 1024;
            if let Some(e) = self.pool.get_mut(id) {
                e.velocity_x = vx; // fixed-point >>8 per tick
                e.velocity_z = 1400; // upward; gravity (if flagged) pulls it back
            }
            let (max_frame, flags, _, _) = effect_defaults(self.effect_type);
            println!(
                "spawn {} (type 0x{:02x}) max_frame={} gravity={} loop={} active={}",
                spell_name(self.effect_type),
                self.effect_type,
                max_frame,
                flags & EFFECT_GRAVITY != 0,
                flags & EFFECT_LOOP != 0,
                self.pool.active_count(),
            );
        }
    }

    fn clear(&mut self) {
        for i in 0..MAX_EFFECTS {
            self.pool.destroy(i as u16);
        }
        println!("cleared effects");
    }

    fn update(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        self.last = now;
        if self.paused {
            return;
        }
        // Tick the engine pool at ~60Hz regardless of frame rate.
        self.tick_timer += dt;
        while self.tick_timer >= 1.0 / 60.0 {
            self.tick_timer -= 1.0 / 60.0;
            self.pool.update_all();
        }
    }

    fn render(&mut self) {
        let output = match self.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let w = self.gpu.size.width as f32;
        let h = self.gpu.size.height as f32;
        let aspect = self.atlas.frame_width as f32 / self.atlas.frame_height as f32;
        let uv_scale_x = 1.0 / self.atlas.frames as f32;

        // Collect active effects and write their uniforms before the pass.
        let mut draws = 0usize;
        for i in 0..MAX_EFFECTS {
            if draws >= MAX_DRAW {
                break;
            }
            let Some(e) = self.pool.get(i as u16) else { continue };
            let px = e.x as f32 / WORLD_PER_PX;
            let py = e.z as f32 / WORLD_PER_PX;
            let scale = e.scale as f32 / 256.0;
            let half_h = BASE_SPRITE_PX * scale;
            let half_w = half_h * aspect;
            let frame = (e.frame.max(0) as u32) % self.atlas.frames;
            let uniforms = SpriteUniforms {
                projection: place_matrix(px, py, half_w, half_h, w, h),
                uv_offset: [frame as f32 * uv_scale_x, 0.0],
                uv_scale: [uv_scale_x, 1.0],
                mirror: [0.0; 4],
            };
            self.gpu.queue.write_buffer(&self.uniform_buffers[draws].buffer, 0, bytemuck::bytes_of(&uniforms));
            draws += 1;
        }

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("effect_encoder") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("effect_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.quad.buffer.slice(..));
            for d in 0..draws {
                pass.set_bind_group(0, &self.bind_groups[d], &[]);
                pass.draw(0..6, 0..1);
            }
        }
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn handle_key(&mut self, key: KeyCode, container: &ContainerPSFB, palette: &[[u8; 4]]) {
        let mut rebuild = false;
        match key {
            KeyCode::Space => self.spawn(),
            KeyCode::Backspace | KeyCode::Delete => self.clear(),
            KeyCode::KeyP => {
                self.paused = !self.paused;
                println!("tick {}", if self.paused { "paused" } else { "running" });
            }
            KeyCode::BracketLeft => { self.sprite_start = self.sprite_start.saturating_sub(1); rebuild = true; }
            KeyCode::BracketRight => { self.sprite_start += 1; rebuild = true; }
            KeyCode::Minus => { self.sprite_start = self.sprite_start.saturating_sub(10); rebuild = true; }
            KeyCode::Equal => { self.sprite_start += 10; rebuild = true; }
            KeyCode::Comma => { self.frame_count = self.frame_count.saturating_sub(1).max(1); rebuild = true; }
            KeyCode::Period => { self.frame_count += 1; rebuild = true; }
            _ => {
                if let Some(t) = effect_type_for_key(key) {
                    self.effect_type = t;
                    println!("selected {} (type 0x{:02x})", spell_name(t), t);
                }
            }
        }
        if rebuild {
            self.rebuild_atlas(container, palette);
        }
    }
}

/// Map a number/letter key to an EffectType id 0x01..0x0C.
fn effect_type_for_key(key: KeyCode) -> Option<u8> {
    Some(match key {
        KeyCode::Digit1 => 0x01,
        KeyCode::Digit2 => 0x02,
        KeyCode::Digit3 => 0x03,
        KeyCode::Digit4 => 0x04,
        KeyCode::Digit5 => 0x05,
        KeyCode::Digit6 => 0x06,
        KeyCode::Digit7 => 0x07,
        KeyCode::Digit8 => 0x08,
        KeyCode::Digit9 => 0x09,
        KeyCode::Digit0 => 0x0A,
        KeyCode::KeyQ => 0x0B,
        KeyCode::KeyW => 0x0C,
        _ => return None,
    })
}

impl App {
    fn new(container: ContainerPSFB, palette: Vec<[u8; 4]>) -> Self {
        Self { window: None, state: None, container, palette }
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
                        .with_title("Spell Effect Demo")
                        .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        let sprite_start = 0usize;
        let frame_count = 4usize;
        let atlas = build_atlas(&self.container, &self.palette, sprite_start, frame_count)
            .unwrap_or(SpriteAtlas { rgba: vec![0, 0, 0, 0], width: 1, height: 1, frame_width: 1, frame_height: 1, frames: 1 });
        let atlas_tex = GpuTexture::new_2d(
            device,
            &gpu.queue,
            atlas.width,
            atlas.height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &atlas.rgba,
            "effect_atlas",
        );
        let sampler = GpuTexture::create_sampler(device, true);

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("effect_bgl"),
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

        let mut uniform_buffers = Vec::with_capacity(MAX_DRAW);
        for i in 0..MAX_DRAW {
            uniform_buffers.push(GpuBuffer::new_uniform(
                device,
                std::mem::size_of::<SpriteUniforms>() as u64,
                &format!("effect_uniforms_{}", i),
            ));
        }

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            ],
        };
        let pipeline = create_pipeline(
            device,
            include_str!("../../shaders/sprite.wgsl"),
            &[vertex_layout],
            &[&bgl],
            gpu.surface_format(),
            false,
            wgpu::PrimitiveTopology::TriangleList,
            "effect_pipeline",
        );
        let quad = GpuBuffer::new_vertex(device, bytemuck::cast_slice(&unit_quad()), "unit_quad");

        let mut state = DemoState {
            gpu,
            pipeline,
            bgl,
            bind_groups: Vec::new(),
            uniform_buffers,
            quad,
            sampler,
            atlas_tex,
            atlas,
            pool: EffectPool::new(),
            effect_type: 0x01,
            sprite_start,
            frame_count,
            paused: false,
            rng: 0x1234_5678,
            tick_timer: 0.0,
            last: std::time::Instant::now(),
        };
        state.rebuild_bind_groups();

        println!("Spell Effect Demo — Space=spawn  [ ]/-=+ scrub sprites  , . frames  P pause  Backspace clear");
        println!("EffectType keys: 1-9,0,Q,W -> {:?}", SPELL_NAMES.map(|(_, n)| n));
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(s) = &mut self.state {
                    s.gpu.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        if key == KeyCode::Escape {
                            event_loop.exit();
                            return;
                        }
                        if let Some(s) = &mut self.state {
                            s.handle_key(key, &self.container, &self.palette);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(s) = &mut self.state {
                    s.update();
                    s.render();
                }
            }
            _ => {}
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_matrix_maps_center_and_size_to_ndc() {
        // 800x600 surface, sprite at center, 100px half-width / 50px half-height.
        let m = place_matrix(0.0, 0.0, 100.0, 50.0, 800.0, 600.0);
        // Column-major: m[3] is translation (center -> NDC origin).
        assert_eq!([m[3][0], m[3][1]], [0.0, 0.0]);
        // A unit-quad corner at x=+1 lands at half_w / (w/2) in NDC.
        assert!((m[0][0] - 100.0 / 400.0).abs() < 1e-6);
        assert!((m[1][1] - 50.0 / 300.0).abs() < 1e-6);
        // Off-center translation is pixels / (dim/2).
        let m2 = place_matrix(200.0, -150.0, 10.0, 10.0, 800.0, 600.0);
        assert!((m2[3][0] - 0.5).abs() < 1e-6);
        assert!((m2[3][1] + 0.5).abs() < 1e-6);
    }
}

/******************************************************************************/
// CLI & main
/******************************************************************************/

fn main() {
    let matches = Command::new("spell-demo")
        .about("Spell effect lifecycle demo for Populous: The Beginning")
        .arg(Arg::new("base").long("base").value_parser(clap::value_parser!(PathBuf)).help("Path to game data directory"))
        .get_matches();

    let base = matches
        .get_one::<PathBuf>("base")
        .cloned()
        .unwrap_or_else(|| PathBuf::from("data/original_game"));
    let data_dir = base.join("data");

    let palette = load_palette(&data_dir.join("pal0-0.dat")).expect("Failed to load palette pal0-0.dat");
    let container = ContainerPSFB::from_file(&data_dir.join("HSPR0-0.DAT")).expect("Failed to load HSPR0-0.DAT");
    println!("Loaded {} sprites from HSPR0-0.DAT", container.len());

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(container, palette);
    event_loop.run_app(&mut app).unwrap();
}

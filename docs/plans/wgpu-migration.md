# Migration from OpenGL 4.6 to wgpu + winit

## Motivation

The original codebase used OpenGL 4.6 via `gl46` (raw bindings) and `glutin` (windowing + GL context). This tied the project to desktop OpenGL, which is deprecated on macOS and unavailable on the web.

The replacement uses **wgpu** (Rust-native GPU abstraction) and **winit** (cross-platform windowing). wgpu targets Metal on macOS, Vulkan on Linux, DX12 on Windows, and WebGPU in browsers -- making this the most portable option.

## Dependency Changes

### Removed
- `glutin` -- OpenGL windowing/context
- `gl46` -- OpenGL 4.6 raw bindings
- `takeable-option` -- used in old envelop code
- `gl_generator` (build-dep) -- generated GL bindings at build time
- `build.rs` -- only existed to run `gl_generator`

### Added
- `wgpu = "28.0"` -- GPU abstraction (Metal/Vulkan/DX12/WebGPU)
- `winit = "0.30"` -- cross-platform windowing
- `pollster = "0.4"` -- blocking on async wgpu initialization
- `bytemuck = { version = "1", features = ["derive"] }` -- safe byte casting for GPU buffers

## Architecture

### Old structure
```
src/opengl/
  gl.rs        -- GlCtx wrapper around raw GL context
  program.rs   -- GlProgram, GlShader (SPIR-V loading)
  buffer.rs    -- VAO/VBO/EBO management
  vertex.rs    -- vertex attribute setup
  uniform.rs   -- uniform + SSBO wrappers
  texture.rs   -- GL texture + TBO management
```

### New structure
```
src/gpu/
  context.rs   -- GpuContext: device, queue, surface, depth texture
  pipeline.rs  -- create_pipeline() helper
  buffer.rs    -- GpuBuffer: vertex/index/uniform/storage wrappers
  texture.rs   -- GpuTexture: 2D textures + samplers
```

### Key design decisions

1. **ModelEnvelop manages geometry only.** The old `ModelEnvelop` owned VAOs, VBOs, uniforms, and knew about the GL program. The new one only manages vertex/index buffers. Bind groups, pipelines, and uniform buffers are managed by the application (each binary). This gives more flexibility since the landscape viewer and object viewer have very different bind group layouts.

2. **All shaders use a 2-group layout.** Group 0 holds transform uniforms (MVP, model transform, and optionally LandscapeParams). Group 1 holds shader-specific resources (textures, storage buffers, samplers).

3. **WGSL shaders duplicate shared code.** WGSL has no `#include` mechanism, so the landscape vertex shader code is duplicated across all 4 landscape shader files. This is the standard approach for WGSL.

4. **Old GLSL/SPIR-V shaders are kept for reference** alongside the new `.wgsl` files.

## File-by-file changes

### Deleted
- `build.rs`
- `src/opengl/` (entire directory)

### Created
- `src/gpu/mod.rs` -- module declaration
- `src/gpu/context.rs` -- `GpuContext` struct (device, queue, surface, depth texture, resize)
- `src/gpu/pipeline.rs` -- `create_pipeline()` helper function
- `src/gpu/buffer.rs` -- `GpuBuffer` with factory methods (vertex, index, uniform, storage)
- `src/gpu/texture.rs` -- `GpuTexture` with 2D creation, update, and sampler factory
- `shaders/objects.wgsl` -- selection lines shader
- `shaders/objects_tex.wgsl` -- textured object shader (OBJ viewer)
- `shaders/landscape.wgsl` -- full GPU texture generation
- `shaders/landscape_cpu.wgsl` -- CPU palette index variant
- `shaders/landscape_full.wgsl` -- CPU full texture variant
- `shaders/landscape_grad.wgsl` -- height gradient variant

### Rewritten
- `src/envelop.rs` -- `GlModel` trait replaced with `GpuModel`, `ModelEnvelop` simplified
- `src/default_model.rs` -- implements `GpuModel` for `DefaultModel`
- `src/tex_model.rs` -- implements `GpuModel` for `TexModel` with new `TexVertexGpu` AoS layout
- `src/landscape.rs` -- implements `GpuModel` for `LandscapeModel`
- `src/lib.rs` -- `pub mod opengl` replaced with `pub mod gpu`
- `src/main.rs` -- full rewrite with winit `ApplicationHandler` pattern
- `src/bin/pop_obj_view.rs` -- full rewrite with winit `ApplicationHandler` pattern

### Unchanged
- `src/model.rs`, `src/view.rs`, `src/intersect.rs`
- `src/pop/` (all game data parsing)
- `src/geometry/` (procedural mesh generation)

## OpenGL to wgpu translation guide

### Concepts mapping

| OpenGL | wgpu |
|--------|------|
| GL context | `wgpu::Device` + `wgpu::Queue` |
| Swap buffers | `surface.get_current_texture()` + `present()` |
| VAO/VBO | `wgpu::Buffer` (vertex) |
| EBO | `wgpu::Buffer` (index) |
| Uniform buffer | `wgpu::Buffer` (uniform) + bind group |
| SSBO (shader storage) | `wgpu::Buffer` (storage) + bind group |
| `glUseProgram` | `render_pass.set_pipeline()` |
| `glUniform*` | `queue.write_buffer()` |
| `glBindTexture` | Bind group with texture view |
| `GL_TEXTURE_1D` | 2D texture with height=1 |
| Texture buffer object (TBO) | Storage buffer with array indexing |
| `gl_PrimitiveID` | `vertex_index / 3` as flat-interpolated varying |
| `glLineWidth(3.0)` | Not supported (Metal/WebGPU limitation, uses 1px) |
| glutin event loop closure | winit `ApplicationHandler` trait |

### Shader translation (GLSL 4.60 to WGSL)

| GLSL | WGSL |
|------|------|
| `layout(location=0) in vec3 coord3d` | `@location(0) coord3d: vec3<f32>` |
| `layout(binding=0) uniform mat4 m` | `@group(0) @binding(0) var<uniform> t: Transforms` |
| `layout(binding=9, std430) buffer Heights { uint h[]; }` | `@group(1) @binding(0) var<storage, read> heights: array<u32>` |
| `texelFetch(sampler1D, idx, 0)` | `palette[idx]` (storage buffer lookup) |
| `texture(sampler2D, uv)` | `textureSample(tex, samp, uv)` |
| `gl_PrimitiveID` | Pass from vertex shader as flat varying |
| `discard` | `discard` (same) |

## Runtime fixes

### Vertex alignment (VERTEX_ALIGNMENT = 4)

wgpu requires all vertex buffer strides to be a multiple of 4 bytes. The landscape mesh uses `Vector2<u8>` vertices (2 bytes each). OpenGL had no such restriction.

**Fix:** Pad each vertex from 2 bytes to 4 bytes:
- Stride changed from `2` to `4`
- `vertex_data()` emits `[x, y, 0, 0]` instead of `[x, y]`
- The shader format remains `Uint8x2` (reads the first 2 bytes, ignores padding)

### Vertex layout: SoA to AoS

The old `TexModel` used Structure-of-Arrays (SoA) layout in OpenGL: all positions contiguous, then all UVs, then all tex_ids. This was done with multiple `glVertexAttribPointer` calls into different regions of one VBO.

wgpu strongly favors Array-of-Structures (AoS) / interleaved layout. A new `TexVertexGpu` struct packs all attributes per vertex:

```rust
#[repr(C)]
struct TexVertexGpu {
    coord: [f32; 3],  // 12 bytes
    uv: [f32; 2],     // 8 bytes
    tex_id: i32,      // 4 bytes (widened from i16)
}
// Total stride: 24 bytes
```

### Windowing: glutin closure to winit ApplicationHandler

winit 0.30 replaced the closure-based event loop with a trait-based pattern:

```rust
// Old (glutin)
el.run(move |event, _, control_flow| {
    match event { ... }
});

// New (winit 0.30)
struct App { ... }
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { /* init GPU */ }
    fn window_event(&mut self, ..., event: WindowEvent) { /* handle input */ }
}
event_loop.run_app(&mut app).unwrap();
```

GPU initialization happens in `resumed()` because the window must exist before creating the wgpu surface.

### Keyboard mapping

| Old (glutin) | New (winit 0.30) |
|-------------|-----------------|
| `VKC::Q` | `KeyCode::KeyQ` |
| `VKC::Up` | `KeyCode::ArrowUp` |
| `VKC::Left` | `KeyCode::ArrowLeft` |
| `KI { state, virtual_keycode, .. }` | `event.state` + `PhysicalKey::Code(key)` |

## wgpu 28 API specifics

These differ from most online examples (which target wgpu ~22-24):

- `PipelineLayoutDescriptor` uses `immediate_size: 0` instead of `push_constant_ranges: &[]`
- `RenderPipelineDescriptor` uses `multiview_mask: None` instead of `multiview: None`
- `RenderPassColorAttachment` requires a `depth_slice: None` field
- `SamplerDescriptor.mipmap_filter` takes `MipmapFilterMode` not `FilterMode`
- `write_texture` uses `TexelCopyTextureInfo` and `TexelCopyBufferLayout` (renamed from older types)

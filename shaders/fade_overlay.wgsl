// fade_overlay.wgsl — Fullscreen fade overlay pass.
//
// Renders a single-screen quad that applies the fade table to the
// entire screen image. This replaces the C++ LbDraw_SetFadeStep()
// mechanic which sets a row in the 256-row fade table and applies
// it to every pixel of the 8-bit framebuffer.
//
// The fade_step uniform (0.0–1.0) selects the row of the fade table.

struct FadeParams {
    fade_step: f32,    // 0.0 = black, 1.0 = normal
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(0) @binding(0) var<uniform> params: FadeParams;
@group(0) @binding(1) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var screen_sampler: sampler;
@group(0) @binding(3) var index_texture: texture_2d<u32>;
@group(0) @binding(4) var index_sampler: sampler;
@group(0) @binding(5) var fade_table: texture_2d<u32>;
@group(0) @binding(6) var fade_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Fullscreen triangle (3 vertices, no vertex buffer)
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u)) * 2.0;
    let y = f32(i32(vertex_index >> 1u)) * 2.0;
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(screen_texture, screen_sampler, in.uv);
    let base_alpha = base_color.a;

    // When fade_step is 1.0, pass through normally
    if (params.fade_step >= 0.999) {
        return base_color;
    }
    // When fade_step is 0.0, fully black
    if (params.fade_step <= 0.001) {
        return vec4<f32>(0.0, 0.0, 0.0, base_alpha);
    }

    // Linear fade to black using the step
    let faded = base_color.rgb * params.fade_step;
    return vec4<f32>(faded, base_alpha);
}
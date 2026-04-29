// fade_overlay.wgsl — Fullscreen fade overlay pass.
//
// Renders a fullscreen black quad with variable opacity.
// fade_step = 1.0 means no fade (fully transparent).
// fade_step = 0.0 means fully black.
// This replaces the C++ LbDraw_SetFadeStep() mechanic.

struct FadeParams {
    fade_step: f32,    // 0.0 = black, 1.0 = normal
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(0) @binding(0) var<uniform> params: FadeParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Fullscreen triangle (covers entire clip space)
    let x = f32(i32(vertex_index & 1u)) * 4.0 - 1.0;
    let y = f32(i32(vertex_index >> 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = 1.0 - params.fade_step;
    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}
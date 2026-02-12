// hud_sprite.wgsl — Screen-space 2D sprite/HUD rendering with color tint.

struct Uniforms {
    screen_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var sprite_texture: texture_2d<f32>;
@group(0) @binding(2) var sprite_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert pixel coordinates to clip space: (0,0)=top-left, (w,h)=bottom-right
    let x = in.position.x / uniforms.screen_size.x * 2.0 - 1.0;
    let y = 1.0 - in.position.y / uniforms.screen_size.y * 2.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(sprite_texture, sprite_sampler, in.uv);
    let final_color = tex * in.color;
    if (final_color.a < 0.01) {
        discard;
    }
    return final_color;
}

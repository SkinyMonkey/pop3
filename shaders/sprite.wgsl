struct Uniforms {
    projection: mat4x4<f32>,
    uv_offset: vec2<f32>,
    uv_scale: vec2<f32>,
    mirror: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var sprite_texture: texture_2d<f32>;
@group(0) @binding(2) var sprite_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    var u = in.uv.x;
    if (uniforms.mirror.x > 0.5) {
        u = 1.0 - u;
    }
    out.uv = uniforms.uv_offset + vec2<f32>(u, in.uv.y) * uniforms.uv_scale;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(sprite_texture, sprite_sampler, in.uv);
    if (color.a < 0.5) {
        discard;
    }
    return color;
}

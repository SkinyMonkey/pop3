// shadow_depth_sprite.wgsl — Shadow pass for sprites: standard alpha test.
// Same vertex transform as building depth shader, but discards transparent sprite pixels.

struct LightMVP {
    matrix: mat4x4<f32>,
};

struct ModelTransform {
    matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> light_mvp: LightMVP;
@group(0) @binding(1) var<uniform> model_transform: ModelTransform;

@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_id: i32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = light_mvp.matrix * model_transform.matrix * vec4<f32>(in.coord3d, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) {
    let color = textureSample(sprite_texture, sprite_sampler, in.uv);
    if (color.a < 0.5) {
        discard;
    }
}

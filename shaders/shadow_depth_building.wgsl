// shadow_depth_building.wgsl — Shadow pass: render building from light's POV into depth texture.
// Uses TexVertex layout. BL320 alpha test ensures transparent pixels don't cast shadow.

struct LightMVP {
    matrix: mat4x4<f32>,
};

struct ModelTransform {
    matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> light_mvp: LightMVP;
@group(0) @binding(1) var<uniform> model_transform: ModelTransform;

@group(1) @binding(0) var texture_main: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_id: i32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) tex_id: i32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = light_mvp.matrix * model_transform.matrix * vec4<f32>(in.coord3d, 1.0);
    out.uv = in.uv;
    out.tex_id = in.tex_id;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) {
    if (in.tex_id < 0 || in.tex_id > 255) {
        return;
    }
    let row = in.tex_id / 8;
    let column = in.tex_id % 8;
    let hor_k = 1.0 / 8.0;
    let ver_k = 1.0 / 32.0;
    let u = hor_k * f32(column) + hor_k * in.uv.x;
    let v = ver_k * f32(row) + ver_k * in.uv.y;
    let color = textureSample(texture_main, texture_sampler, vec2<f32>(u, v));
    if (color.w > 0.0) {
        discard;
    }
}

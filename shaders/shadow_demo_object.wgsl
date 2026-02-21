// shadow_demo_object.wgsl — Building rendering with Lambert lighting for shadow demo.

struct CameraMVP {
    matrix: mat4x4<f32>,
};

struct ModelTransform {
    matrix: mat4x4<f32>,
};

struct SunDir {
    dir: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera_mvp: CameraMVP;
@group(0) @binding(1) var<uniform> model_transform: ModelTransform;
@group(0) @binding(2) var<uniform> sun: SunDir;

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
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world = model_transform.matrix * vec4<f32>(in.coord3d, 1.0);
    out.position = camera_mvp.matrix * world;
    out.uv = in.uv;
    out.tex_id = in.tex_id;
    out.world_pos = world.xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_normal = normalize(cross(dpdx(in.world_pos), dpdy(in.world_pos)));
    let ndotl = max(dot(world_normal, sun.dir.xyz), 0.0);
    let ambient = 0.3;
    let brightness = ambient + (1.0 - ambient) * ndotl;

    if (in.tex_id < 0 || in.tex_id > 255) {
        return vec4<f32>(vec3<f32>(0.6) * brightness, 1.0);
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
    return vec4<f32>(color.rgb * brightness, 1.0);
}

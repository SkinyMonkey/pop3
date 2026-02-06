// objects_tex.wgsl — OBJ viewer with textured objects.
// Replaces objects_1.vert + objects_1.frag.

// Group 0: Transform matrices
struct Transforms {
    m_transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> transforms: Transforms;

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

@group(0) @binding(1) var<uniform> transforms1: Transforms1;

// Group 1: Texture and sampler
@group(1) @binding(0) var texture_main: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

// Vertex input
struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_id: i32,
};

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) tex_id: i32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(in.coord3d, 1.0);
    out.tex_id = in.tex_id;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.tex_id < 0 || in.tex_id > 255) {
        return vec4<f32>(0.6, 0.6, 0.6, 1.0);
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
    return color;
}

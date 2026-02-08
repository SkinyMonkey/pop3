// objects.wgsl — Selection lines in the landscape viewer.
// Replaces objects.vert + objects.frag.

// Group 0: Transform matrices
struct Transforms {
    m_transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> transforms: Transforms;

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

@group(0) @binding(1) var<uniform> transforms1: Transforms1;

// Group 1: Fragment uniforms and color storage buffer
struct ObjectParams {
    num_colors: i32,
};

@group(1) @binding(0) var<uniform> params: ObjectParams;

// Colors stored as vec4<u32> (RGBA u8 values, each widened to u32)
@group(1) @binding(1) var<storage, read> colors: array<vec4<u32>>;

// Vertex input
struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @builtin(vertex_index) vertex_index: u32,
};

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) primitive_id: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(in.coord3d, 1.0);
    out.primitive_id = in.vertex_index / 3u;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = params.num_colors;
    let idx = prim_id % size;
    let color = colors[idx];
    return vec4<f32>(
        f32(color.r) / 255.0,
        f32(color.g) / 255.0,
        f32(color.b) / 255.0,
        0.0,
    );
}

// shadow.wgsl — Ground shadow blobs for units and objects.
// Renders flat quads on terrain with a circular shadow texture.
// Uses alpha blending to darken the terrain underneath.

struct Transforms {
    m_transform: mat4x4<f32>,
};

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> transforms: Transforms;
@group(0) @binding(1) var<uniform> transforms1: Transforms1;

@group(1) @binding(0) var shadow_texture: texture_2d<f32>;
@group(1) @binding(1) var shadow_sampler: sampler;

struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) _unused: i32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(in.coord3d, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let shadow = textureSample(shadow_texture, shadow_sampler, in.uv);
    // Output black with shadow alpha — blending darkens the terrain
    return vec4<f32>(0.0, 0.0, 0.0, shadow.r * 0.48);
}

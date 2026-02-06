// shaman_sprite.wgsl — Shaman sprite billboards at spawn positions.

// Group 0: Transform matrices (shared with landscape/objects)
struct Transforms {
    m_transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> transforms: Transforms;

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

@group(0) @binding(1) var<uniform> transforms1: Transforms1;

// Group 1: Sprite texture and sampler
@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

// Vertex input — uses TexVertex layout (coord3d + uv + tex_id)
struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tribe_id: i32,
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
    let color = textureSample(sprite_texture, sprite_sampler, in.uv);
    if (color.a < 0.5) {
        discard;
    }
    return vec4<f32>(color.rgb, 1.0);
}

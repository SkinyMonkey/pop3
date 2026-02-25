// shadow_demo_sprite.wgsl — Main pass sprite billboard with lighting.

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
    out.position = camera_mvp.matrix * model_transform.matrix * vec4<f32>(in.coord3d, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(sprite_texture, sprite_sampler, in.uv);
    if (color.a < 0.5) {
        discard;
    }
    // Sprite brightness from sun elevation (z component)
    let ambient = 0.3;
    let brightness = ambient + (1.0 - ambient) * max(sun.dir.z, 0.0);
    return vec4<f32>(color.rgb * brightness, 1.0);
}

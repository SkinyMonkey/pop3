// shadow_demo_ground.wgsl — Ground plane that receives shadows from the shadow map.

struct CameraMVP {
    matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera_mvp: CameraMVP;

@group(1) @binding(0) var shadow_map: texture_depth_2d;
@group(1) @binding(1) var shadow_samp: sampler_comparison;
@group(1) @binding(2) var<uniform> light_mvp: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera_mvp.matrix * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Project world position into light space
    let light_pos = light_mvp * vec4<f32>(in.world_pos, 1.0);
    let shadow_uv = light_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let shadow_depth = light_pos.z - 0.005;

    // Sample shadow map (1.0 = lit, 0.0 = shadowed)
    let shadow = textureSampleCompare(shadow_map, shadow_samp, shadow_uv, shadow_depth);
    let shadow_factor = 0.3 + 0.7 * shadow;

    // Bright green checkerboard — shadows clearly visible
    let cx = floor(in.world_pos.x * 4.0);
    let cy = floor(in.world_pos.y * 4.0);
    let even = ((i32(cx) + i32(cy)) % 2) == 0;
    let base_color = select(vec3<f32>(0.35, 0.6, 0.25), vec3<f32>(0.45, 0.7, 0.3), even);

    return vec4<f32>(base_color * shadow_factor, 1.0);
}

// shadow.wgsl — Ground shadow blobs for units and objects.
// Renders flat quads on terrain with a circular shadow texture.
// Uses alpha blending to darken the terrain underneath.

struct Transforms {
    m_transform: mat4x4<f32>,
};

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

struct LightParams {
    sun_dir: vec3<f32>,
    ambient: f32,
    camera_focus: vec2<f32>,
    viewport_radius: f32,
    game_tick: f32,
};

@group(0) @binding(0) var<uniform> transforms: Transforms;
@group(0) @binding(1) var<uniform> transforms1: Transforms1;
@group(0) @binding(2) var<uniform> light: LightParams;

@group(1) @binding(0) var shadow_texture: texture_2d<f32>;
@group(1) @binding(1) var shadow_sampler: sampler;

struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) shadow_type: i32, // 0 = building (static), 1 = unit (shimmer)
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) viewport_fade: f32,
    @location(2) @interpolate(flat) shadow_type: i32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(in.coord3d, 1.0);
    out.uv = in.uv;
    out.shadow_type = in.shadow_type;

    // Viewport fade: smooth circular falloff at edges
    let dx = in.coord3d.x - light.camera_focus.x;
    let dy = in.coord3d.y - light.camera_focus.y;
    let dist = sqrt(dx * dx + dy * dy);
    let fade_start = light.viewport_radius * 0.85;
    let fade_end = light.viewport_radius;
    out.viewport_fade = clamp(1.0 - (dist - fade_start) / (fade_end - fade_start), 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.viewport_fade < 0.01) {
        discard;
    }
    let shadow = textureSample(shadow_texture, shadow_sampler, in.uv);

    // Only unit shadows (type 1) shimmer; building shadows are static
    var intensity = 0.48;
    if (in.shadow_type == 1) {
        let tick = u32(light.game_tick);
        let shimmer = abs(f32(tick % 12u) - f32(tick % 6u) * 2.0) / 5.0;
        intensity = 0.48 * (1.0 - shimmer * 0.33);
    }

    return vec4<f32>(0.0, 0.0, 0.0, shadow.r * intensity * in.viewport_fade);
}

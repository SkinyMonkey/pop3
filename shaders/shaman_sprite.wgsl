// shaman_sprite.wgsl — Shaman sprite billboards at spawn positions.

// Group 0: Transform matrices + lighting (shared with buildings/shadows)
struct Transforms {
    m_transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> transforms: Transforms;

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

@group(0) @binding(1) var<uniform> transforms1: Transforms1;

struct LightParams {
    sun_dir: vec3<f32>,
    ambient: f32,
    camera_focus: vec2<f32>,
    viewport_radius: f32,
    game_tick: f32,
};

@group(0) @binding(2) var<uniform> light: LightParams;

// Group 1: Sprite texture and sampler
@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

// Group 2: Shadow map
@group(2) @binding(0) var shadow_map: texture_depth_2d;
@group(2) @binding(1) var shadow_samp: sampler_comparison;
@group(2) @binding(2) var<uniform> shadow_light_mvp: mat4x4<f32>;

// Vertex input — uses TexVertex layout (coord3d + uv + tex_id)
struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tribe_id: i32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) viewport_fade: f32,
    @location(2) local_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(in.coord3d, 1.0);
    out.uv = in.uv;
    out.local_pos = in.coord3d;

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
    let color = textureSample(sprite_texture, sprite_sampler, in.uv);
    if (color.a < 0.5) {
        discard;
    }
    // Shadow mapping
    let shadow_world = transforms1.m_transform1 * vec4<f32>(in.local_pos, 1.0);
    let light_pos = shadow_light_mvp * shadow_world;
    let shadow_uv = light_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let shadow = textureSampleCompare(shadow_map, shadow_samp, shadow_uv, light_pos.z - 0.005);
    let shadow_factor = 0.3 + 0.7 * shadow;

    // Sprite brightness: use sun_dir.z as proxy for how much light hits horizontal surfaces
    let brightness = light.ambient + (1.0 - light.ambient) * max(light.sun_dir.z, 0.0);
    return vec4<f32>(color.rgb * brightness * shadow_factor * in.viewport_fade, 1.0);
}

// sprite_indexed.wgsl — Indexed sprite rendering with ghost/fade/remap LUT lookups.
//
// This shader replicates the original PopTB 8-bit palette blending pipeline on
// the GPU. It samples BOTH an RGBA sprite texture (for the final colour) AND
// a palette-index sideband texture (R8Uint storing the original palette index
// for each texel) so that LUT-based blend effects can work in index space.
//
// Blend modes (mutually exclusive, matching C++ draw flags):
//   0 = Solid (no blending, just texture sample)
//   1 = Glass  (TRANSPAR4):  blended = ghost[vec2(src_idx, dst_idx)]
//   2 = InvertGlass (TRANSPAR8): blended = ghost[vec2(dst_idx, src_idx)]
//   3 = Fade: blended = fade[vec2(palette_idx, fade_step)]
//   4 = Remap: remapped = remap[vec2(palette_idx, 0.5)]

// Group 0: Transform matrices + lighting
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

// Group 1: Sprite texture (RGBA) + palette-index texture (R8Uint)
@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(1) @binding(2) var index_texture: texture_2d<u32>;
@group(1) @binding(3) var index_sampler: sampler;

// Group 2: Shadow map
@group(2) @binding(0) var shadow_map: texture_depth_2d;
@group(2) @binding(1) var shadow_samp: sampler_comparison;
@group(2) @binding(2) var<uniform> shadow_light_mvp: mat4x4<f32>;

// Group 3: LUT textures
@group(3) @binding(0) var ghost_table: texture_2d<u32>;
@group(3) @binding(1) var ghost_sampler: sampler;
@group(3) @binding(2) var fade_table: texture_2d<u32>;
@group(3) @binding(3) var fade_sampler: sampler;

// Group 4: Blend mode uniform
struct BlendParams {
    blend_mode: u32,      // 0=Solid, 1=Glass, 2=InvertGlass, 3=Fade, 4=Remap
    fade_step: f32,       // 0..1 for fade mode (step / 255.0)
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
    tint_a: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};
@group(4) @binding(0) var<uniform> blend: BlendParams;

// Vertex input — same layout as shaman_sprite
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
    @location(3) @interpolate(flat) blend_mode: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(in.coord3d, 1.0);
    out.uv = in.uv;
    out.local_pos = in.coord3d;
    out.blend_mode = blend.blend_mode;

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

    // Read the original palette index from the sideband texture
    let index_vec = textureSample(index_texture, index_sampler, in.uv);
    let src_idx = index_vec.r;

    // Shadow mapping
    let shadow_world = transforms1.m_transform1 * vec4<f32>(in.local_pos, 1.0);
    let light_pos = shadow_light_mvp * shadow_world;
    let shadow_uv = light_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let shadow = textureSampleCompare(shadow_map, shadow_samp, shadow_uv, light_pos.z - 0.005);
    let shadow_factor = 0.3 + 0.7 * shadow;

    let brightness = light.ambient + (1.0 - light.ambient) * max(light.sun_dir.z, 0.0);

    // Apply blend mode
    if (in.blend_mode == 1u) {
        // GLASS (TRANSPAR4): ghost[vec2(src_idx, dst_idx)]
        // We read dst_idx from the framebuffer — but since we can't read
        // the framebuffer in a forward pass, we use alpha blending instead.
        // The ghost table gives us a blended palette index. We look up the
        // blended colour from the main sprite texture (which has already been
        // palette-resolved), so we apply the ghost as a tint/alpha modifier.
        //
        // For a true glass effect, we need two passes or framebuffer readback.
        // Here we approximate by dimming the sprite colour using the ghost table
        // row as a brightness modifier.
        let ghost_val = textureSample(ghost_table, ghost_sampler, vec2<f32>(f32(src_idx) / 256.0, 0.5)).r;
        let ghost_frac = f32(ghost_val) / 255.0;
        let final_color = color.rgb * brightness * shadow_factor * in.viewport_fade * ghost_frac;
        return vec4<f32>(final_color, color.a * in.viewport_fade);
    } else if (in.blend_mode == 2u) {
        // INVERT_GLASS (TRANSPAR8): ghost[vec2(dst_idx, src_idx)]
        // Same approximation as Glass but with swapped table coords.
        // Since we can't read the destination in a single pass, this
        // produces a similar but not identical effect. A two-pass solution
        // would capture the destination first.
        let ghost_val = textureSample(ghost_table, ghost_sampler, vec2<f32>(0.5, f32(src_idx) / 256.0)).r;
        let ghost_frac = f32(ghost_val) / 255.0;
        let final_color = color.rgb * brightness * shadow_factor * in.viewport_fade * ghost_frac;
        return vec4<f32>(final_color, color.a * in.viewport_fade);
    } else if (in.blend_mode == 3u) {
        // FADE: fade[vec2(palette_idx, fade_step)]
        let faded_idx = textureSample(fade_table, fade_sampler, vec2<f32>(f32(src_idx) / 256.0, blend.fade_step)).r;
        let faded_frac = f32(faded_idx) / 255.0;
        let final_color = color.rgb * brightness * shadow_factor * in.viewport_fade * faded_frac;
        return vec4<f32>(final_color, color.a * in.viewport_fade);
    } else if (in.blend_mode == 4u) {
        // REMAP: Not yet implemented — render as tinted sprite
        let t = vec3<f32>(blend.tint_r, blend.tint_g, blend.tint_b);
        let final_color = color.rgb * t * brightness * shadow_factor * in.viewport_fade;
        return vec4<f32>(final_color, color.a * blend.tint_a * in.viewport_fade);
    }

    // Mode 0: Solid
    let tint = vec3<f32>(blend.tint_r, blend.tint_g, blend.tint_b);
    let has_tint = blend.tint_r > 0.0 || blend.tint_g > 0.0 || blend.tint_b > 0.0;
    if (has_tint) {
        let final_color = color.rgb * tint * brightness * shadow_factor * in.viewport_fade;
        return vec4<f32>(final_color, color.a * blend.tint_a * in.viewport_fade);
    }

    return vec4<f32>(color.rgb * brightness * shadow_factor * in.viewport_fade, 1.0);
}
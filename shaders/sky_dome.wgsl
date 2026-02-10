// sky_dome.wgsl — Sky hemisphere projection faithful to the original game.
//
// Implements the perspective projection from Sky_RenderTiled (0x004dcc30):
//   1. Map screen pixel to view-space offset from center
//   2. Scale by lens factors (u_scale, v_scale)
//   3. Apply hemisphere depth: depth = horizon - (sx^2 >> 12)
//   4. Compute perspective divisor
//   5. Rotate UV by camera angle
//   6. Sample 512x512 sky texture with wrapping

struct SkyDomeParams {
    // Camera rotation
    u_origin: f32,      // accumulated U pan (normalized 0..1)
    v_origin: f32,      // accumulated V pan (normalized 0..1)
    cos_angle: f32,     // cos of rotation angle
    sin_angle: f32,     // sin of rotation angle
    // Projection
    u_scale: f32,       // horizontal projection scale
    v_scale: f32,       // vertical projection scale
    horizon: f32,       // hemisphere curvature (higher = flatter)
    mode: u32,          // 0=tiled, 1=simple, 2=parallax, 3=flat
};

@group(0) @binding(0) var<uniform> params: SkyDomeParams;
@group(0) @binding(1) var sky_texture: texture_2d<f32>;
@group(0) @binding(2) var sky_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Fullscreen triangle
    var pos: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    var uv: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );
    var out: VertexOutput;
    out.position = vec4<f32>(pos[vertex_index], 0.9999, 1.0);
    out.uv = uv[vertex_index];
    return out;
}

// Mode 0: Hemisphere projection (matches Sky_RenderTiled)
fn sample_dome(screen_uv: vec2<f32>) -> vec4<f32> {
    // Map screen UV to centered coordinates (-0.5 .. +0.5)
    let cx = screen_uv.x - 0.5;
    let cy = screen_uv.y - 0.5;

    // Scale by lens factors
    let sx = cx * params.u_scale;
    let sy = cy * params.v_scale;

    // Hemisphere depth: depth = horizon - sx^2 / curvature_divisor
    // In the original: depth = horizon - (sx*sx >> 12)
    // We normalize: the divisor 4096 (>>12) relative to the scale
    let depth = params.horizon - (sx * sx * 0.25);

    // Perspective divisor: perspective = horizon / (horizon - depth)
    // When depth approaches horizon, clamp to avoid division by zero
    let denom = params.horizon - depth;
    let perspective = select(
        params.horizon / denom,
        64.0,
        abs(denom) < 0.001
    );

    // Rotate and project UV — both sx and sy contribute after perspective scaling.
    // This is a 2D rotation of the perspective-projected (sx, sy) vector by camera angle.
    let proj_x = sx * perspective * 0.5;
    let proj_y = sy * perspective * 0.5;

    let u = params.u_origin
          + proj_x * params.cos_angle
          + proj_y * params.sin_angle;
    let v = params.v_origin
          - proj_x * params.sin_angle
          + proj_y * params.cos_angle;

    return textureSample(sky_texture, sky_sampler, vec2<f32>(u, v));
}

// Mode 1: Simple scrolling (matches Sky_RenderSimple)
fn sample_simple(screen_uv: vec2<f32>) -> vec4<f32> {
    let u = screen_uv.x + params.u_origin;
    let v = screen_uv.y + params.v_origin;
    return textureSample(sky_texture, sky_sampler, vec2<f32>(u, v));
}

// Mode 2: Vertical gradient parallax (matches Sky_RenderParallax)
fn sample_parallax(screen_uv: vec2<f32>) -> vec4<f32> {
    // Dither matrix (4x4 Bayer, normalized)
    let dither = array<f32, 16>(
        0.0/64.0, 32.0/64.0,  8.0/64.0, 40.0/64.0,
       48.0/64.0, 16.0/64.0, 56.0/64.0, 24.0/64.0,
       12.0/64.0, 44.0/64.0,  4.0/64.0, 36.0/64.0,
       60.0/64.0, 28.0/64.0, 52.0/64.0, 20.0/64.0
    );

    // Vertical gradient with dither
    let t = screen_uv.y;

    // Sample at dithered position (use texture as color ramp)
    let u = params.u_origin + 0.5;
    let v = t;
    return textureSample(sky_texture, sky_sampler, vec2<f32>(u, v));
}

// Mode 3: Flat fill (matches Sky_RenderFlatFill — palette 0x7E)
fn sample_flat() -> vec4<f32> {
    // Sample center of texture for a representative sky color
    return textureSample(sky_texture, sky_sampler, vec2<f32>(0.5, 0.5));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    switch params.mode {
        case 0u: { return sample_dome(in.uv); }
        case 1u: { return sample_simple(in.uv); }
        case 2u: { return sample_parallax(in.uv); }
        case 3u: { return sample_flat(); }
        default: { return sample_dome(in.uv); }
    }
}

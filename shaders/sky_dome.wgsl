// sky_dome.wgsl — Sky hemisphere projection demo.
//
// Approximates the 4 rendering modes from the original game:
//   Mode 0: Hemisphere dome projection (Sky_RenderTiled)
//   Mode 1: Simple scrolling (Sky_RenderSimple)
//   Mode 2: Vertical gradient parallax (Sky_RenderParallax)
//   Mode 3: Flat fill (Sky_RenderFlatFill)

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

// Mode 0: Hemisphere dome projection (approximates Sky_RenderTiled).
//
// Original uses depth = horizon - sx² with only horizontal curvature and
// skylens.dat for vertical distortion. We approximate with radial hemisphere:
// depth = horizon - r² where r² = sx² + sy², giving proper dome curvature.
// Perspective = horizon / (horizon - depth), clamped to 1.0 (matching original
// 0x10000 clamp in 16.16 fixed-point). The projected (sx, sy) vector is rotated
// by camera angle to produce final UV.
fn sample_dome(screen_uv: vec2<f32>) -> vec4<f32> {
    let cx = screen_uv.x - 0.5;
    let cy = screen_uv.y - 0.5;

    let sx = cx * params.u_scale;
    let sy = cy * params.v_scale;

    // Radial hemisphere: curvature from both axes
    let r_sq = sx * sx + sy * sy;
    let depth = params.horizon - r_sq * 0.25;

    // Perspective: clamped to 1.0 (original clamps to 0x10000 in 16.16 = 1.0)
    let denom = max(params.horizon - depth, 0.01);
    let perspective = min(params.horizon / denom, 1.0);

    // 2D rotation of perspective-projected screen vector by camera angle
    let proj_x = sx * perspective;
    let proj_y = sy * perspective;

    let u = params.u_origin
          + proj_x * params.cos_angle
          + proj_y * params.sin_angle;
    let v = params.v_origin
          - proj_x * params.sin_angle
          + proj_y * params.cos_angle;

    return textureSample(sky_texture, sky_sampler, vec2<f32>(u, v));
}

// Mode 1: Simple scrolling (matches Sky_RenderSimple).
// Original: tex_u = (origin_u >> 16) + origin_v_offset*2 + x
//           tex_v = (origin_v >> 18) + y * 2
// Note the 2x vertical scale — the 512×512 texture is sampled at double
// density vertically to fill the 384-line viewport.
fn sample_simple(screen_uv: vec2<f32>) -> vec4<f32> {
    let u = screen_uv.x + params.u_origin;
    let v = screen_uv.y * 2.0 + params.v_origin;
    return textureSample(sky_texture, sky_sampler, vec2<f32>(u, v));
}

// Mode 2: Vertical gradient parallax (matches Sky_RenderParallax).
// Original fills each scanline with a SINGLE color derived from a vertical
// brightness gradient through the palette LUT, with 4×4 ordered dithering
// to reduce banding. We approximate by sampling the texture along the
// vertical center, dithered.
fn sample_parallax(screen_uv: vec2<f32>) -> vec4<f32> {
    // Original dither matrix from g_dither_matrix (0x0056d2f0):
    // 00 20 08 28 30 10 38 18 0C 2C 04 24 3C 1C 34 14
    // Normalized to [0, 1) range (divide by 64)
    let dither = array<f32, 16>(
         0.0/64.0, 32.0/64.0,  8.0/64.0, 40.0/64.0,
        48.0/64.0, 16.0/64.0, 56.0/64.0, 24.0/64.0,
        12.0/64.0, 44.0/64.0,  4.0/64.0, 36.0/64.0,
        60.0/64.0, 28.0/64.0, 52.0/64.0, 20.0/64.0
    );

    // Pixel coordinates for dither lookup
    let tex_size = vec2<f32>(textureDimensions(sky_texture, 0));
    let px = vec2<i32>(screen_uv * tex_size);
    let dither_idx = (px.y & 3) * 4 + (px.x & 3);
    let dither_val = dither[dither_idx];

    // Vertical gradient: t goes from 0 (top = dark/zenith) to 1 (bottom = bright/horizon)
    // Apply dither to break banding — original adds dither to brightness then quantizes
    let t = clamp(screen_uv.y + dither_val * 0.04, 0.0, 1.0);

    // Sample texture center column at dithered V — the sky texture's vertical
    // center naturally provides a brightness gradient from dark (top) to bright (bottom)
    return textureSample(sky_texture, sky_sampler, vec2<f32>(0.5, t));
}

// Mode 3: Flat fill (matches Sky_RenderFlatFill).
// Original fills entire sky with palette index 0x7E (next-to-brightest sky color).
// We sample the brightest area of the texture as approximation.
fn sample_flat() -> vec4<f32> {
    // Sample near bottom-center — the brightest sky color in the gradient
    return textureSample(sky_texture, sky_sampler, vec2<f32>(0.5, 0.9));
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

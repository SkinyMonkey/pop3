// sky.wgsl — Fullscreen sky background matching Sky_RenderSimple (Mode 1).
//
// Original algorithm (0x004dd710):
//   tex_u = (origin_u >> 16) + x          — horizontal scroll by camera yaw
//   tex_v = (origin_v >> 18) + y * 2      — 2x vertical scale (zoom into clouds)
//   pixel = texture[(tex_u & 0x1FF) + (tex_v & 0x1FF) * 512]
//
// The 2x vertical factor makes cloud patterns appear at natural scale when the
// 512×512 texture covers the ~384-line visible sky area.

struct SkyParams {
    yaw_offset: f32,    // camera yaw mapped to 0..1 UV offset
};

@group(0) @binding(0) var<uniform> sky_params: SkyParams;
@group(0) @binding(1) var sky_texture: texture_2d<f32>;
@group(0) @binding(2) var sky_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Fullscreen triangle (3 vertices, covers entire screen)
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let u = in.uv.x + sky_params.yaw_offset;
    // Original uses y * 2 to zoom into the cloud texture vertically
    let v = in.uv.y * 2.0;
    return textureSample(sky_texture, sky_sampler, vec2<f32>(u, v));
}

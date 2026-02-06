// landscape_grad.wgsl — Simple height gradient visualization.
// Replaces landscape.vert + landscape_grad.frag.

// ---------- Shared types ----------

struct LandscapeParams {
    level_shift: vec4<i32>,
    height_scale: f32,
    step: f32,
    width: i32,
    selected_frag: i32,
    selected_color: vec4<f32>,
    sunlight: vec4<f32>,
    wat_offset: i32,
    curvature_scale: f32,
    camera_focus: vec2<f32>,
    viewport_radius: f32,
};

struct Transforms {
    m_transform: mat4x4<f32>,
};

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

// ---------- Bindings ----------

// Group 0: Uniforms
@group(0) @binding(0) var<uniform> transforms: Transforms;
@group(0) @binding(1) var<uniform> transforms1: Transforms1;
@group(0) @binding(2) var<uniform> params: LandscapeParams;

// Group 1: Storage buffers (needed by vertex shader)
@group(1) @binding(0) var<storage, read> heights: array<u32>;
@group(1) @binding(1) var<storage, read> watdisp: array<u32>;

// ---------- Vertex ----------

struct VertexInput {
    @location(0) coord_in: vec2<u32>,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) coord3d_out: vec3<f32>,
    @location(1) height_out: f32,
    @location(2) brightness: f32,
    @location(3) @interpolate(flat) primitive_id: u32,
    @location(4) viewport_fade: f32,
};

fn wat_height(x: u32, y: u32) -> u32 {
    let x_wat = x * 2u;
    let y_wat = y * 2u;
    let index = (y_wat * 256u + x) * 8u;
    let wat_offset = u32(params.wat_offset) & 0xffu;
    let index1 = (index + wat_offset * 0x101u) & 0xffffu;
    let index2 = (index + 0x4cu - wat_offset * 0x101u) & 0xffffu;
    return (watdisp[index1] + watdisp[index2]) / 8u;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let coord3d = vec3<f32>(f32(in.coord_in.x) * params.step, f32(in.coord_in.y) * params.step, 0.0);

    let w = u32(params.width);
    let x = (in.coord_in.x + u32(params.level_shift.x)) % w;
    let y = (in.coord_in.y + u32(params.level_shift.y)) % w;
    let index = y * w + x;
    var height = heights[index];

    out.height_out = f32(height);

    if (params.wat_offset > -1 && height == 0u) {
        height = wat_height(x, y);
    }

    let coordf = vec3<f32>(coord3d.x, coord3d.y, f32(height) * params.height_scale);

    // Curvature: pull Z down by distance² from camera focus (planet illusion)
    let dx = coordf.x - params.camera_focus.x;
    let dy = coordf.y - params.camera_focus.y;
    let dist_sq = dx * dx + dy * dy;
    let curvature_offset = dist_sq * params.curvature_scale;
    let curved = vec3<f32>(coordf.x, coordf.y, coordf.z - curvature_offset);

    // Viewport fade: smooth circular falloff at edges
    let dist = sqrt(dist_sq);
    let fade_start = params.viewport_radius * 0.85;
    let fade_end = params.viewport_radius;
    var vp_fade = 1.0;
    if (dist > fade_end) {
        vp_fade = 0.0;
    } else if (dist > fade_start) {
        vp_fade = 1.0 - (dist - fade_start) / (fade_end - fade_start);
    }
    out.viewport_fade = vp_fade;

    let coord = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(curved, 1.0);
    out.position = coord;
    out.coord3d_out = vec3<f32>(coord3d.xy, coordf.z);

    // Brightness calculation
    let index1 = ((in.coord_in.y + u32(params.level_shift.y) + 1u) % w) * w + ((in.coord_in.x + u32(params.level_shift.x)) % w);
    let index2 = ((in.coord_in.y + u32(params.level_shift.y)) % w) * w + ((in.coord_in.x + u32(params.level_shift.x) + 1u) % w);
    let ch = i32(heights[index]);
    let br_i = i32(params.sunlight.z) + i32(params.sunlight.y) * (i32(heights[index1]) - ch) - i32(params.sunlight.x) * (ch - i32(heights[index2]));
    let br_f = f32(br_i) / f32(0x15e) + f32(0x80);
    out.brightness = clamp(br_f, 0.0, 255.0);

    out.primitive_id = in.vertex_index / 3u;

    return out;
}

// ---------- Fragment ----------

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.viewport_fade < 0.01) {
        discard;
    }

    let prim_id = i32(in.primitive_id);

    if (params.selected_frag > 0 && params.selected_frag == prim_id) {
        return params.selected_color;
    }

    let height = u32(in.coord3d_out.z / params.height_scale);
    if (height <= 0u) {
        return vec4<f32>(vec3<f32>(0.0, 0.1, 1.0) * in.viewport_fade, 0.0);
    } else {
        let color_ratio = 0.2 * in.coord3d_out.z * 5.0;
        return vec4<f32>(vec3<f32>(0.3 + color_ratio, 0.5 - color_ratio, 0.0) * in.viewport_fade, 0.0);
    }
}

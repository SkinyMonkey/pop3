// landscape.wgsl — GPU texture generation landscape shader.
// Replaces landscape.vert + landscape.frag.

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

// Group 1: Storage buffers
@group(1) @binding(0) var<storage, read> heights: array<u32>;
@group(1) @binding(1) var<storage, read> watdisp: array<u32>;
@group(1) @binding(2) var<storage, read> palette: array<u32>;
@group(1) @binding(3) var<storage, read> disp: array<i32>;
@group(1) @binding(4) var<storage, read> bigf: array<u32>;
@group(1) @binding(5) var<storage, read> sla: array<u32>;

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

fn mk_tex(val: u32) -> vec3<f32> {
    let packed = palette[val % 128u];
    let r = f32((packed >> 0u) & 0xffu) / 255.0;
    let g = f32((packed >> 8u) & 0xffu) / 255.0;
    let b = f32((packed >> 16u) & 0xffu) / 255.0;
    return vec3<f32>(r, g, b);
}

fn mk_height(z: f32) -> u32 {
    let height = u32(z);
    if (height > 0u) {
        return min(height + 0x96u, 0x400u);
    }
    if (z > 0.0) {
        let h = u32(z * f32(0x4b));
        return min(h + 0x4bu, 0x400u);
    }
    return min(height + 0x4bu, 0x400u);
}

fn get_wat_color(z: f32, z_current: f32) -> vec3<f32> {
    if (z <= 1.0 && z_current > 0.0) {
        let c = -((z_current / params.height_scale) / 512.0) / 1.0;
        return vec3<f32>(c, c, c);
    }
    return vec3<f32>(0.0, 0.0, 0.0);
}

fn get_disp(x: i32, y: i32) -> i32 {
    let sx = params.level_shift.x * 32;
    let sy = params.level_shift.y * 32;
    let dx = ((x + sx) % 256) * 256;
    let dy = (y + sy) % 256;
    return disp[dx + dy];
}

fn get_disp_2(x: i32, y: i32) -> i32 {
    let sx = params.level_shift.x * 32;
    let sy = params.level_shift.y * 32;
    let ly = (y + sy) % 32;
    var dx: i32;
    if (ly == 31) {
        dx = 0;
    } else {
        dx = 1;
    }
    let x1 = ((x + dx + sx) % 256) * 256;
    let y1 = (y + 1 + sy) % 256;
    return disp[x1 + y1];
}

fn land_tex(coord: vec3<f32>, height_in: f32, brightness: f32) -> vec3<f32> {
    let height = mk_height(height_in);

    let disp_val = get_disp(i32(coord.x), i32(coord.y) + 32);
    let disp_val_2 = get_disp_2(i32(coord.x), i32(coord.y) + 32);

    var disp_param = i32((f32(disp_val_2) - f32(disp_val)) / 4.0) + i32(brightness);
    disp_param = clamp(disp_param, 0, 255);

    let sla_val = sla[height];
    var static_component = i32(sla_val) * disp_val;
    var static_component_u = u32(static_component);
    static_component_u = static_component_u & 0xfffffc03u;
    static_component = i32(static_component_u);
    static_component = static_component >> 2u;

    let height_component = i32(height * 256u) & 0x7fffff00;
    let index = static_component + height_component + disp_param;

    let bigf_index = min(bigf[index], 128u);
    let res_color = mk_tex(bigf_index);
    let wat_color = get_wat_color(height_in, coord.z);
    return res_color + wat_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.viewport_fade < 0.01) {
        discard;
    }

    let prim_id = i32(in.primitive_id);

    if (params.selected_frag > 0 && params.selected_frag == prim_id) {
        return params.selected_color;
    }

    let coordi = vec3<f32>(
        in.coord3d_out.x / 8.0 * 4096.0,
        in.coord3d_out.y / 8.0 * 4096.0,
        in.coord3d_out.z,
    );
    let c = land_tex(coordi, in.height_out, in.brightness);
    return vec4<f32>(c * in.viewport_fade, 0.0);
}

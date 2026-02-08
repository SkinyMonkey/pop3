// Pathfinding demo shader — 2D top-down terrain + cell overlay + line overlay.
//
// Three pipelines share this shader file:
// 1. Terrain: fullscreen quad textured from heightmap RGBA
// 2. Cell overlay: colored quads for visited cells, grid lines
// 3. Line overlay: colored vertices for path lines, markers

struct Uniforms {
    // Orthographic projection mapping world coords to clip space
    projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// === Terrain pipeline (uses texture) ===

@group(0) @binding(1) var terrain_texture: texture_2d<f32>;
@group(0) @binding(2) var terrain_sampler: sampler;

struct TerrainVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct TerrainVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_terrain(in: TerrainVertexInput) -> TerrainVertexOutput {
    var out: TerrainVertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_terrain(in: TerrainVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(terrain_texture, terrain_sampler, in.uv);
}

// === Overlay pipeline (vertex color, used for both cells and lines) ===

struct OverlayVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct OverlayVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_overlay(in: OverlayVertexInput) -> OverlayVertexOutput {
    var out: OverlayVertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_overlay(in: OverlayVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

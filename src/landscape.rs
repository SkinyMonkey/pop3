use std::iter::Zip;
use std::ops::RangeFrom;
use std::slice::Chunks;

use cgmath::{Vector4, Vector3, Vector2};

use crate::model::{Triangle, VertexModel, MeshModel};
use crate::envelop::{GpuModel, ModelEnvelop, RenderType};
use crate::pop::objects::Shape;

pub type LandscapeModel = MeshModel<Vector2<u8>, u16>;

impl GpuModel for LandscapeModel {
    fn vertex_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        // wgpu requires vertex stride to be a multiple of 4 (VERTEX_ALIGNMENT)
        vec![wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint8x2,
                offset: 0,
                shader_location: 0,
            }],
        }]
    }

    fn vertex_data(&self) -> Vec<u8> {
        // Pad each 2-byte vertex to 4 bytes for alignment
        self.vertices
            .iter()
            .flat_map(|v| [v.x, v.y, 0, 0])
            .collect()
    }

    fn index_data(&self) -> Option<Vec<u8>> {
        None
    }

    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }

    fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    fn index_count(&self) -> u32 {
        0
    }

    fn is_indexed(&self) -> bool {
        false
    }
}

pub struct LandscapeMesh<const N: usize> {
    vertices: Vec<Vector2<u8>>,
    step: f32,
    height_scale: f32,
    shift_x: usize,
    shift_y: usize,
    heights: [[u16; N]; N],
}

impl<const N: usize> LandscapeMesh<N> {
    fn gen_mesh() -> Vec<Vector2<u8>> {
        let vertices_num: usize = N * N * 6;
        let mut vertices = vec![Vector2 { x: 0, y: 0 }; vertices_num];
        for i in 0..(N - 1) {
            for j in 0..(N - 1) {
                let index = (i * N + j) * 6;
                let i_u8 = i as u8;
                let j_u8 = j as u8;
                vertices[index] = Vector2 { x: i_u8, y: j_u8 };
                vertices[index + 1] = Vector2 { x: i_u8, y: j_u8 + 1 };
                vertices[index + 2] = Vector2 { x: i_u8 + 1, y: j_u8 };
                vertices[index + 3] = Vector2 { x: i_u8 + 1, y: j_u8 + 1 };
                vertices[index + 4] = Vector2 { x: i_u8, y: j_u8 + 1 };
                vertices[index + 5] = Vector2 { x: i_u8 + 1, y: j_u8 };
            }
        }
        vertices
    }

    fn shift_n(n: usize, max_n: usize, shift: i32) -> usize {
        let shift_n = (n as i32) + shift;
        if shift_n >= 0 {
            (shift_n as usize) % max_n
        } else {
            (((max_n as i32) + shift_n) as usize) % max_n
        }
    }

    pub fn width(&self) -> usize {
        N
    }

    pub fn new(step: f32, height_scale: f32) -> Self {
        let vertices = Self::gen_mesh();
        Self {
            vertices,
            step,
            height_scale,
            shift_x: 0,
            shift_y: 0,
            heights: [[0u16; N]; N],
        }
    }

    pub fn set_heights(&mut self, heights: &[[u16; N]; N]) {
        self.heights = *heights;
    }

    pub fn height_at(&self, x: usize, y: usize) -> u16 {
        self.heights[y % N][x % N]
    }

    pub fn set_height_at(&mut self, x: usize, y: usize, h: u16) {
        self.heights[y % N][x % N] = h;
    }

    /// Bilinear height interpolation at fractional cell position.
    /// cell_x/cell_y are in cell units (0.0 to N-1). Handles toroidal wrapping.
    /// Returns height in world units (already multiplied by height_scale).
    ///
    /// Matches the original game's Terrain_GetHeightAtPoint (0x004E8E50).
    /// Uses "/" diagonal split consistent with gen_mesh() triangle layout.
    pub fn interpolate_height_at(&self, cell_x: f32, cell_y: f32) -> f32 {
        let ix = cell_x.floor() as i32;
        let iy = cell_y.floor() as i32;
        let fx = cell_x - ix as f32; // fractional 0..1
        let fy = cell_y - iy as f32;

        let x0 = ((ix % N as i32 + N as i32) as usize) % N;
        let y0 = ((iy % N as i32 + N as i32) as usize) % N;
        let x1 = (x0 + 1) % N;
        let y1 = (y0 + 1) % N;

        let h00 = self.heights[y0][x0] as f32;
        let h10 = self.heights[y0][x1] as f32;
        let h01 = self.heights[y1][x0] as f32;
        let h11 = self.heights[y1][x1] as f32;

        // "/" diagonal split: same as gen_mesh() triangle layout
        let h = if fx + fy <= 1.0 {
            // Lower-left triangle: (0,0)-(0,1)-(1,0)
            h00 + (h10 - h00) * fx + (h01 - h00) * fy
        } else {
            // Upper-right triangle: (1,1)-(0,1)-(1,0)
            h11 + (h01 - h11) * (1.0 - fx) + (h10 - h11) * (1.0 - fy)
        };

        h * self.height_scale
    }

    /// Flatten terrain under a building footprint.
    ///
    /// Matches Building_FlattenTerrain (0x0042F2A0):
    /// 1. Samples all cell heights in footprint
    /// 2. Computes average (or min if use_average=false)
    /// 3. Writes uniform height to all footprint cells
    /// 4. Smooths surrounding terrain
    pub fn flatten_building_footprint(
        &mut self,
        cell_x: usize, cell_y: usize, // building center cell
        shape: &Shape,
        use_average: bool,
    ) {
        let w = shape.width as usize;
        let h = shape.height as usize;
        if w == 0 || h == 0 { return; }

        // Corner = center - origin (with toroidal wrap)
        let corner_x = (cell_x + N - shape.origin_x as usize) % N;
        let corner_y = (cell_y + N - shape.origin_z as usize) % N;

        // Pass 1: Sample heights and compute target
        let mut sum: u64 = 0;
        let mut min_h: u16 = u16::MAX;
        let mut count: u32 = 0;
        for dy in 0..h {
            for dx in 0..w {
                let mask_idx = dy * w + dx;
                if mask_idx < 40 && (shape.cell_mask[mask_idx] & 0x01) != 0 {
                    let cx = (corner_x + dx) % N;
                    let cy = (corner_y + dy) % N;
                    // Sample all 4 corners of this cell
                    let cx1 = (cx + 1) % N;
                    let cy1 = (cy + 1) % N;
                    let h00 = self.heights[cy][cx];
                    let h10 = self.heights[cy][cx1];
                    let h01 = self.heights[cy1][cx];
                    let h11 = self.heights[cy1][cx1];
                    sum += h00 as u64 + h10 as u64 + h01 as u64 + h11 as u64;
                    min_h = min_h.min(h00).min(h10).min(h01).min(h11);
                    count += 4;
                }
            }
        }

        if count == 0 { return; }

        let target = if use_average {
            (sum / count as u64) as u16
        } else {
            min_h
        };
        let target = target.max(1);

        // Pass 2: Write target height to all footprint cells
        for dy in 0..h {
            for dx in 0..w {
                let mask_idx = dy * w + dx;
                if mask_idx < 40 && (shape.cell_mask[mask_idx] & 0x01) != 0 {
                    let cx = (corner_x + dx) % N;
                    let cy = (corner_y + dy) % N;
                    let cx1 = (cx + 1) % N;
                    let cy1 = (cy + 1) % N;
                    self.heights[cy][cx] = target;
                    self.heights[cy][cx1] = target;
                    self.heights[cy1][cx] = target;
                    self.heights[cy1][cx1] = target;
                }
            }
        }

        // Pass 3: Smooth surrounding terrain
        let radius = (w.max(h) / 2) + 1;
        self.smooth_terrain_area(cell_x, cell_y, radius, corner_x, corner_y, w, h, shape);
    }

    /// Smooth terrain transitions around a flattened area.
    /// Averages heights at border cells to create gradual transitions.
    fn smooth_terrain_area(
        &mut self,
        center_x: usize, center_y: usize,
        radius: usize,
        corner_x: usize, corner_y: usize,
        fp_w: usize, fp_h: usize,
        shape: &Shape,
    ) {
        let r = radius + 1;
        // Iterate over a ring around the footprint
        let start_x = (center_x + N - r) % N;
        let start_y = (center_y + N - r) % N;
        let diameter = r * 2 + 1;

        for dy in 0..diameter {
            for dx in 0..diameter {
                let cx = (start_x + dx) % N;
                let cy = (start_y + dy) % N;

                // Skip cells inside the footprint
                let rel_x = (cx + N - corner_x) % N;
                let rel_y = (cy + N - corner_y) % N;
                if rel_x < fp_w && rel_y < fp_h {
                    let mask_idx = rel_y * fp_w + rel_x;
                    if mask_idx < 40 && (shape.cell_mask[mask_idx] & 0x01) != 0 {
                        continue;
                    }
                }

                // Average with neighbors
                let cx1 = (cx + 1) % N;
                let cy1 = (cy + 1) % N;
                let cxm = (cx + N - 1) % N;
                let cym = (cy + N - 1) % N;

                let h_center = self.heights[cy][cx] as u32;
                let h_right = self.heights[cy][cx1] as u32;
                let h_left = self.heights[cy][cxm] as u32;
                let h_down = self.heights[cy1][cx] as u32;
                let h_up = self.heights[cym][cx] as u32;

                let avg = (h_center * 4 + h_right + h_left + h_down + h_up) / 8;
                self.heights[cy][cx] = avg as u16;
            }
        }
    }

    pub fn heights(&self) -> &[[u16; N]; N] {
        &self.heights
    }

    /// Export heights as Vec<u32> for GPU buffer upload (matches Landscape::to_vec format).
    pub fn heights_to_gpu_vec(&self) -> Vec<u32> {
        let mut vec = vec![0u32; N * N];
        for i in 0..N {
            for j in 0..N {
                vec[i * N + j] = self.heights[i][j] as u32;
            }
        }
        vec
    }

    pub fn step(&self) -> f32 {
        self.step
    }

    pub fn height_scale(&self) -> f32 {
        self.height_scale
    }

    pub fn shift_x(&mut self, shift: i32) -> usize {
        self.shift_x = Self::shift_n(self.shift_x, N, shift);
        self.shift_x
    }

    pub fn shift_y(&mut self, shift: i32) -> usize {
        self.shift_y = Self::shift_n(self.shift_y, N, shift);
        self.shift_y
    }

    pub fn set_shift(&mut self, sx: usize, sy: usize) {
        self.shift_x = sx % N;
        self.shift_y = sy % N;
    }

    pub fn get_shift_vector(&self) -> Vector4<i32> {
        Vector4::new(self.shift_x as i32, self.shift_y as i32, 0, 0)
    }

    pub fn to_model(&self, m: &mut LandscapeModel) {
        for v2 in &self.vertices {
            m.push_vertex(*v2);
        }
    }

    pub fn iter(&self) -> LandscapeTriangleIterator<N> {
        let iter_internal = (0..).zip(self.vertices.chunks(3));
        LandscapeTriangleIterator {
            landscape: self,
            iter_internal,
        }
    }

    fn make_vec3(&self, v: &Vector2<u8>) -> Vector3<f32> {
        let x = v.x as f32 * self.step;
        let y = v.y as f32 * self.step;
        let index_x = (v.x as usize + self.shift_x) % N;
        let index_y = (v.y as usize + self.shift_y) % N;
        let z = self.heights[index_y][index_x] as f32 * self.height_scale;
        Vector3 { x, y, z }
    }
}

pub struct LandscapeTriangleIterator<'a, const N: usize> {
    landscape: &'a LandscapeMesh<N>,
    iter_internal: Zip<RangeFrom<usize>, Chunks<'a, Vector2<u8>>>,
}

impl<'a, const N: usize> Iterator for LandscapeTriangleIterator<'a, N> {
    type Item = (usize, Triangle<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter_internal.next() {
            Some((n, c)) => {
                if c.len() != 3 {
                    return None;
                }
                let a = self.landscape.make_vec3(&c[0]);
                let b = self.landscape.make_vec3(&c[1]);
                let c = self.landscape.make_vec3(&c[2]);
                let t: Triangle<f32> = Triangle { a, b, c };
                Some((n, t))
            }
            None => None,
        }
    }
}

/******************************************************************************/

/// Landscape model transform: world = LANDSCAPE_SCALE * model + LANDSCAPE_OFFSET.
pub const LANDSCAPE_SCALE: f32 = 2.5;
pub const LANDSCAPE_OFFSET: f32 = -2.0;

/// Packed landscape uniform data matching the WGSL LandscapeParams struct.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LandscapeUniformData {
    pub level_shift: [i32; 4],
    pub height_scale: f32,
    pub step: f32,
    pub width: i32,
    pub _pad_width: i32,
    pub sunlight: [f32; 4],
    pub wat_offset: i32,
    pub curvature_scale: f32,
    pub camera_focus: [f32; 2],
    pub viewport_radius: f32,
    pub _pad2: [f32; 3],
}

/// A landscape program variant with its own pipeline and group-1 bind group.
pub struct LandscapeVariant {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_1: wgpu::BindGroup,
}

pub struct LandscapeProgramContainer {
    variants: Vec<LandscapeVariant>,
    index: usize,
}

impl LandscapeProgramContainer {
    pub fn new() -> Self {
        Self { variants: Vec::new(), index: 0 }
    }

    pub fn add(&mut self, variant: LandscapeVariant) {
        self.variants.push(variant);
    }

    pub fn next(&mut self) {
        if !self.variants.is_empty() {
            self.index = (self.index + 1) % self.variants.len();
        }
    }

    pub fn prev(&mut self) {
        if self.variants.is_empty() { return; }
        self.index = if self.index == 0 {
            self.variants.len() - 1
        } else {
            self.index - 1
        };
    }

    pub fn current(&self) -> Option<&LandscapeVariant> {
        self.variants.get(self.index)
    }
}

pub fn make_landscape_model<const N: usize>(device: &wgpu::Device, landscape_mesh: &LandscapeMesh<N>) -> ModelEnvelop<LandscapeModel> {
    let mut model: LandscapeModel = MeshModel::new();
    landscape_mesh.to_model(&mut model);
    log::debug!("Landscape mesh - vertices={:?}, indices={:?}", model.vertices.len(), model.indices.len());
    let m = vec![(RenderType::Triangles, model)];
    let mut model_main = ModelEnvelop::<LandscapeModel>::new(device, m);
    if let Some(m) = model_main.get(0) {
        m.location.x = LANDSCAPE_OFFSET;
        m.location.y = LANDSCAPE_OFFSET;
        m.scale = LANDSCAPE_SCALE;
    }
    eprintln!("[landscape] model transform: location=({0},{0},0) scale={1}", LANDSCAPE_OFFSET, LANDSCAPE_SCALE);
    model_main
}

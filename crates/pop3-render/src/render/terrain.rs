use crate::data::model::MeshModel;
use crate::render::envelop::{GpuModel, ModelEnvelop, RenderType};

pub use crate::engine::terrain::landscape_mesh::{
    LandscapeModel, LandscapeMesh, LandscapeTriangleIterator,
    LandscapeUniformData, LANDSCAPE_SCALE, LANDSCAPE_OFFSET,
};


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


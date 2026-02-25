use cgmath::Vector3;

use crate::model::{VertexModel, MeshModel};
use crate::tex_model::{TexModel, TexVertex};
use crate::envelop::{ModelEnvelop, RenderType};
use crate::landscape::LandscapeMesh;
use crate::pop::objects::{Object3D, Shape, mk_pop_object};
use crate::pop::units::object_3d_index;

use crate::sprites::LevelObject;

pub fn build_building_meshes(
    device: &wgpu::Device, objects: &[LevelObject], objects_3d: &[Option<Object3D>],
    _shapes: &[Shape], landscape: &LandscapeMesh<128>, curvature_scale: f32,
) -> ModelEnvelop<TexModel> {
    let mut combined: TexModel = MeshModel::new();
    let step = landscape.step();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let center = (w - 1.0) * step / 2.0;

    let mut building_count = 0;
    for obj in objects {
        let idx = match object_3d_index(&obj.model_type, obj.subtype, obj.tribe_index) {
            Some(i) => Some(i),
            None => continue,
        };
        building_count += 1;
        eprintln!("[3d-obj] type={:?} subtype={} tribe={} -> idx={:?}", obj.model_type, obj.subtype, obj.tribe_index, idx);
        let obj3d = match idx {
            Some(i) if i < objects_3d.len() => match &objects_3d[i] {
                Some(o) => o,
                None => { eprintln!("  -> object at {} is None", i); continue; },
            },
            _ => continue,
        };

        let local_model = mk_pop_object(obj3d);
        let scale = step * (obj3d.coord_scale() / 300.0);

        let vis_x = ((obj.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((obj.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;

        // Skip buildings outside the visible globe disc (matching landscape viewport fade)
        let dx_cull = gx - center;
        let dy_cull = gy - center;
        let viewport_radius = center * 0.9;
        if dx_cull * dx_cull + dy_cull * dy_cull > viewport_radius * viewport_radius {
            continue;
        }

        // Rotate model vertices in the horizontal plane (model X/Z -> world X/Y)
        let angle_rad = (obj.angle as f32) * std::f32::consts::TAU / 2048.0;
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let base_idx = combined.vertices.len() as u16;
        for v in &local_model.vertices {
            let rx = v.coord.x * cos_a - v.coord.z * sin_a;
            let rz = v.coord.x * sin_a + v.coord.z * cos_a;

            let vx_gpu = gx + rx * scale;
            let vy_gpu = gy + rz * scale;

            // Per-vertex curvature (matching landscape shader: dist_sq * curvature_scale)
            let vdx = vx_gpu - center;
            let vdy = vy_gpu - center;
            let vertex_curvature = (vdx * vdx + vdy * vdy) * curvature_scale;

            // Per-vertex terrain height sampling (matching Model3D_RenderObject Phase 4)
            let vert_cell_x = vis_x + rx * scale / step;
            let vert_cell_y = vis_y + rz * scale / step;
            let abs_cell_x = ((vert_cell_x % w + w) % w) + shift.x as f32;
            let abs_cell_y = ((vert_cell_y % w + w) % w) + shift.y as f32;
            let vertex_gz = landscape.interpolate_height_at(abs_cell_x, abs_cell_y);
            let vertex_z = vertex_gz - vertex_curvature + v.coord.y * scale;

            combined.push_vertex(TexVertex {
                coord: Vector3::new(vx_gpu, vy_gpu, vertex_z),
                uv: v.uv,
                tex_id: v.tex_id,
            });
        }
        for &idx16 in &local_model.indices {
            combined.indices.push(base_idx + idx16);
        }
    }
    eprintln!("[buildings] total={} vertices={} indices={} step={:.4} center={:.4}",
        building_count, combined.vertices.len(), combined.indices.len(), step, center);
    // Print vertex bounding box for debugging
    if !combined.vertices.is_empty() {
        let (mut min_x, mut min_y, mut min_z) = (f32::MAX, f32::MAX, f32::MAX);
        let (mut max_x, mut max_y, mut max_z) = (f32::MIN, f32::MIN, f32::MIN);
        for v in &combined.vertices {
            min_x = min_x.min(v.coord.x); max_x = max_x.max(v.coord.x);
            min_y = min_y.min(v.coord.y); max_y = max_y.max(v.coord.y);
            min_z = min_z.min(v.coord.z); max_z = max_z.max(v.coord.z);
        }
        eprintln!("[buildings] bbox x=[{:.3}..{:.3}] y=[{:.3}..{:.3}] z=[{:.3}..{:.3}]",
            min_x, max_x, min_y, max_y, min_z, max_z);
    }
    let m = vec![(RenderType::Triangles, combined)];
    ModelEnvelop::<TexModel>::new(device, m)
}

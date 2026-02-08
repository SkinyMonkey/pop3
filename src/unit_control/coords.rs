// Coordinate conversions between world, cell, and GPU space.
//
// World coords: i16 (0-65535 range), used by movement system.
// Cell coords:  f32 (0-127), used by renderer for landscape grid position.
// GPU coords:   f32, cell * step with shift applied.
//
// The axis mapping (from main.rs extract_level_objects):
//   bevy_x = (loc_x >> 8) / 2 + 0.5
//   bevy_z = (loc_y >> 8) / 2 + 0.5
//   cell_x = bevy_z            (swap!)
//   cell_y = (n-1) - bevy_x    (flip!)
//
// So: world.x → cell_y (flipped), world.z → cell_x

use cgmath::{Matrix4, Vector4};
use crate::movement::WorldCoord;

/// Convert world coordinates to cell coordinates for rendering.
/// `n` is landscape size (typically 128.0).
pub fn world_to_cell(pos: &WorldCoord, n: f32) -> (f32, f32) {
    let bevy_x = ((pos.x as u16) >> 8) as f32 / 2.0 + 0.5;
    let bevy_z = ((pos.z as u16) >> 8) as f32 / 2.0 + 0.5;
    let cell_x = bevy_z;
    let cell_y = (n - 1.0) - bevy_x;
    (cell_x, cell_y)
}

/// Convert cell coordinates back to world coordinates.
/// Inverse of world_to_cell. `n` is landscape size (typically 128.0).
pub fn cell_to_world(cell_x: f32, cell_y: f32, n: f32) -> WorldCoord {
    // cell_x = bevy_z = (loc_y >> 8) / 2 + 0.5
    // cell_y = (n-1) - bevy_x = (n-1) - ((loc_x >> 8) / 2 + 0.5)
    let bevy_x = (n - 1.0) - cell_y;
    let bevy_z = cell_x;
    let loc_x = ((bevy_x - 0.5) * 2.0) as u16;
    let loc_y = ((bevy_z - 0.5) * 2.0) as u16;
    WorldCoord::new((loc_x << 8) as i16, (loc_y << 8) as i16)
}

/// Convert GPU-space hit point to cell coordinates.
/// `step` is landscape mesh step size, `shift` is current view shift,
/// `w` is landscape width (128.0).
pub fn gpu_to_cell(gx: f32, gy: f32, step: f32, shift_x: f32, shift_y: f32, w: f32) -> (f32, f32) {
    let vis_x = gx / step;
    let vis_y = gy / step;
    let cell_x = ((vis_x + shift_x) % w + w) % w;
    let cell_y = ((vis_y + shift_y) % w + w) % w;
    (cell_x, cell_y)
}

/// Convert a landscape triangle index to cell coordinates.
/// The mesh has `mesh_width * mesh_width * 6` vertices, 2 triangles per cell,
/// with stride `mesh_width` (not mesh_width-1) in the vertex array.
pub fn triangle_to_cell(triangle_id: usize, mesh_width: usize, shift_x: usize, shift_y: usize) -> (f32, f32) {
    let cell_idx = triangle_id / 2;
    let vis_i = cell_idx / mesh_width;
    let vis_j = cell_idx % mesh_width;
    let cell_x = ((vis_i + shift_x) % mesh_width) as f32 + 0.5;
    let cell_y = ((vis_j + shift_y) % mesh_width) as f32 + 0.5;
    (cell_x, cell_y)
}

/// Project a model-space 3D point to screen coordinates via a PVM matrix.
/// Returns `None` if the point is behind the camera (clip.w <= 0).
pub fn project_to_screen(
    pos: [f32; 3],
    pvm: &Matrix4<f32>,
    screen_w: f32,
    screen_h: f32,
) -> Option<(f32, f32)> {
    let clip = *pvm * Vector4::new(pos[0], pos[1], pos[2], 1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc_x = clip.x / clip.w;
    let ndc_y = clip.y / clip.w;
    let sx = (ndc_x + 1.0) * 0.5 * screen_w;
    let sy = (1.0 - ndc_y) * 0.5 * screen_h;
    Some((sx, sy))
}

/// Find the nearest candidate to a screen point within a pixel threshold.
/// `candidates` yields `(id, screen_x, screen_y)` tuples.
/// Returns the `id` of the closest candidate, or `None` if none are within range.
pub fn nearest_screen_hit(
    candidates: impl Iterator<Item = (usize, f32, f32)>,
    mouse_x: f32,
    mouse_y: f32,
    threshold: f32,
) -> Option<usize> {
    let thresh_sq = threshold * threshold;
    let mut best: Option<(usize, f32)> = None;
    for (id, sx, sy) in candidates {
        let dist_sq = (sx - mouse_x).powi(2) + (sy - mouse_y).powi(2);
        if dist_sq < thresh_sq {
            if best.is_none() || dist_sq < best.unwrap().1 {
                best = Some((id, dist_sq));
            }
        }
    }
    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_cell_roundtrip() {
        // A unit at world (0x2000, 0x3000) = loc_x=0x2000, loc_y=0x3000
        let world = WorldCoord::new(0x2000, 0x3000);
        let n = 128.0;
        let (cx, cy) = world_to_cell(&world, n);

        // bevy_x = (0x2000 >> 8) / 2 + 0.5 = 32/2 + 0.5 = 16.5
        // bevy_z = (0x3000 >> 8) / 2 + 0.5 = 48/2 + 0.5 = 24.5
        // cell_x = 24.5, cell_y = 127 - 16.5 = 110.5
        assert!((cx - 24.5).abs() < 0.01);
        assert!((cy - 110.5).abs() < 0.01);

        // Convert back
        let back = cell_to_world(cx, cy, n);
        assert_eq!(back.x, world.x);
        assert_eq!(back.z, world.z);
    }

    #[test]
    fn world_to_cell_origin() {
        // World (0x100, 0x100) = loc=256 for both axes
        let world = WorldCoord::new(0x100, 0x100);
        let (cx, cy) = world_to_cell(&world, 128.0);
        // bevy_x = 1/2 + 0.5 = 1.0, bevy_z = 1/2 + 0.5 = 1.0
        // cell_x = 1.0, cell_y = 127 - 1.0 = 126.0
        assert!((cx - 1.0).abs() < 0.01);
        assert!((cy - 126.0).abs() < 0.01);
    }

    #[test]
    fn gpu_to_cell_with_shift() {
        let step = 1.0 / 16.0; // 0.0625
        let gx = 2.0 * step; // vis_x = 2.0
        let gy = 3.0 * step; // vis_y = 3.0
        let (cx, cy) = gpu_to_cell(gx, gy, step, 10.0, 20.0, 128.0);
        assert!((cx - 12.0).abs() < 0.01); // 2 + 10
        assert!((cy - 23.0).abs() < 0.01); // 3 + 20
    }

    #[test]
    fn gpu_to_cell_wraps() {
        let step = 1.0 / 16.0;
        let gx = 120.0 * step;
        let gy = 0.0;
        let (cx, _) = gpu_to_cell(gx, gy, step, 20.0, 0.0, 128.0);
        // 120 + 20 = 140 % 128 = 12
        assert!((cx - 12.0).abs() < 0.01);
    }

    // gen_mesh uses index = (i * N + j) * 6 where N=128.
    // So cell (i, j) has triangle_ids (i*128+j)*2 and (i*128+j)*2+1.
    // triangle_to_cell must use stride 128 (not 127) to recover (i, j).

    #[test]
    fn triangle_to_cell_origin() {
        // Cell (0,0): triangle_ids 0, 1
        let (cx, cy) = triangle_to_cell(0, 128, 0, 0);
        assert!((cx - 0.5).abs() < 0.01);
        assert!((cy - 0.5).abs() < 0.01);
        let (cx, cy) = triangle_to_cell(1, 128, 0, 0);
        assert!((cx - 0.5).abs() < 0.01);
        assert!((cy - 0.5).abs() < 0.01);
    }

    #[test]
    fn triangle_to_cell_row_boundary() {
        // Cell (1,0): triangle_ids (1*128+0)*2 = 256, 257
        // This is the case that breaks with stride 127.
        let (cx, cy) = triangle_to_cell(256, 128, 0, 0);
        assert!((cx - 1.5).abs() < 0.01);
        assert!((cy - 0.5).abs() < 0.01);
    }

    #[test]
    fn triangle_to_cell_last_in_row() {
        // Cell (0,126): triangle_ids (0*128+126)*2 = 252, 253
        let (cx, cy) = triangle_to_cell(252, 128, 0, 0);
        assert!((cx - 0.5).abs() < 0.01);
        assert!((cy - 126.5).abs() < 0.01);
    }

    #[test]
    fn triangle_to_cell_interior() {
        // Cell (5,10): triangle_ids (5*128+10)*2 = 1300
        let (cx, cy) = triangle_to_cell(1300, 128, 0, 0);
        assert!((cx - 5.5).abs() < 0.01);
        assert!((cy - 10.5).abs() < 0.01);
    }

    #[test]
    fn triangle_to_cell_with_shift() {
        // Cell (0,0) at shift=(40,50) → cell (40.5, 50.5)
        let (cx, cy) = triangle_to_cell(0, 128, 40, 50);
        assert!((cx - 40.5).abs() < 0.01);
        assert!((cy - 50.5).abs() < 0.01);
    }

    #[test]
    fn triangle_to_cell_shift_wraps() {
        // Cell (120,0) at shift=(20,0): (120+20)%128 = 12
        let tri_id = (120 * 128 + 0) * 2;
        let (cx, cy) = triangle_to_cell(tri_id, 128, 20, 0);
        assert!((cx - 12.5).abs() < 0.01);
        assert!((cy - 0.5).abs() < 0.01);
    }

    // --- project_to_screen tests ---

    use cgmath::{Matrix4, SquareMatrix};

    #[test]
    fn project_identity_center() {
        // Identity PVM: NDC = model coords directly.
        // Point at origin (0,0,0) → NDC (0,0) → screen center.
        let pvm = Matrix4::identity();
        let (sx, sy) = project_to_screen([0.0, 0.0, 0.0], &pvm, 800.0, 600.0).unwrap();
        assert!((sx - 400.0).abs() < 0.01);
        assert!((sy - 300.0).abs() < 0.01);
    }

    #[test]
    fn project_identity_corners() {
        // NDC (-1,-1) → screen (0, h), NDC (1,1) → screen (w, 0)
        let pvm = Matrix4::identity();
        // Point at (-1, -1, 0): NDC (-1,-1) → sx=0, sy=600
        let (sx, sy) = project_to_screen([-1.0, -1.0, 0.0], &pvm, 800.0, 600.0).unwrap();
        assert!((sx - 0.0).abs() < 0.01);
        assert!((sy - 600.0).abs() < 0.01);
        // Point at (1, 1, 0): NDC (1,1) → sx=800, sy=0
        let (sx, sy) = project_to_screen([1.0, 1.0, 0.0], &pvm, 800.0, 600.0).unwrap();
        assert!((sx - 800.0).abs() < 0.01);
        assert!((sy - 0.0).abs() < 0.01);
    }

    #[test]
    fn project_behind_camera_returns_none() {
        // A PVM that puts clip.w <= 0.
        // Simple: scale w by -1 via a matrix that negates the w component.
        #[rustfmt::skip]
        let pvm = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, -1.0, // w = -1 * 1.0 = -1
        );
        assert!(project_to_screen([0.0, 0.0, 0.0], &pvm, 800.0, 600.0).is_none());
    }

    #[test]
    fn project_with_scale() {
        // Scale x by 2: point (0.25, 0, 0) → NDC (0.5, 0) → screen (600, 300)
        #[rustfmt::skip]
        let pvm = Matrix4::new(
            2.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let (sx, sy) = project_to_screen([0.25, 0.0, 0.0], &pvm, 800.0, 600.0).unwrap();
        assert!((sx - 600.0).abs() < 0.01); // (0.5+1)*0.5*800 = 600
        assert!((sy - 300.0).abs() < 0.01);
    }

    // --- nearest_screen_hit tests ---

    #[test]
    fn nearest_hit_empty() {
        let result = nearest_screen_hit(std::iter::empty(), 100.0, 100.0, 20.0);
        assert!(result.is_none());
    }

    #[test]
    fn nearest_hit_within_threshold() {
        let candidates = vec![(0, 105.0, 100.0), (1, 200.0, 200.0)];
        let result = nearest_screen_hit(candidates.into_iter(), 100.0, 100.0, 20.0);
        assert_eq!(result, Some(0)); // dist=5, within 20px
    }

    #[test]
    fn nearest_hit_none_outside_threshold() {
        let candidates = vec![(0, 150.0, 100.0), (1, 200.0, 200.0)];
        let result = nearest_screen_hit(candidates.into_iter(), 100.0, 100.0, 20.0);
        assert!(result.is_none()); // dist=50, outside 20px
    }

    #[test]
    fn nearest_hit_picks_closest() {
        let candidates = vec![
            (0, 110.0, 100.0), // dist=10
            (1, 105.0, 100.0), // dist=5  ← closest
            (2, 108.0, 100.0), // dist=8
        ];
        let result = nearest_screen_hit(candidates.into_iter(), 100.0, 100.0, 20.0);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn nearest_hit_diagonal_distance() {
        // dist = sqrt(10^2 + 10^2) = 14.14, within 20px threshold
        let candidates = vec![(7, 110.0, 110.0)];
        let result = nearest_screen_hit(candidates.into_iter(), 100.0, 100.0, 20.0);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn nearest_hit_exact_threshold_excluded() {
        // dist = 20.0 exactly: threshold check is strict (<, not <=)
        let candidates = vec![(0, 120.0, 100.0)];
        let result = nearest_screen_hit(candidates.into_iter(), 100.0, 100.0, 20.0);
        assert!(result.is_none());
    }
}

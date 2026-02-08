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
}

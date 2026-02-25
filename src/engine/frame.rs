use cgmath::Vector4;

use crate::hud::HudState;
use crate::landscape::LandscapeMesh;
use crate::data::objects::{Object3D, Shape};
use crate::sprites::LevelObject;
use crate::engine::units::{UnitCoordinator, DragState};
use crate::view::{Camera, Screen};

/// Output boundary — everything the renderer needs to produce one frame.
/// Produced by GameEngine, consumed by Renderer.
pub struct FrameState<'a> {
    // View
    pub camera: &'a Camera,
    pub screen: &'a Screen,
    pub zoom: f32,

    // Landscape
    pub landscape: &'a LandscapeMesh<128>,
    pub curvature_scale: f32, // 0.0 if disabled
    pub sunlight: Vector4<f32>,
    pub wat_offset: i32,

    // Objects
    pub show_objects: bool,
    pub unit_coordinator: &'a UnitCoordinator,
    pub level_objects: &'a [LevelObject],
    pub objects_3d: &'a [Option<Object3D>],
    pub shapes: &'a [Shape],

    // HUD
    pub hud_state: HudState,
    pub drag_state: &'a DragState,

    // Dirty flags (set by apply_command, cleared after renderer processes them)
    pub needs_spawn_rebuild: bool,
    pub needs_unit_rebuild: bool,
    pub needs_level_reload: bool,
}

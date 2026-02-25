use cgmath::Vector4;
use winit::keyboard::KeyCode;

use crate::hud::{HudTab, HudState};
use crate::landscape::LandscapeMesh;
use crate::pop::objects::{Object3D, Shape};
use crate::sprites::LevelObject;
use crate::unit_control::{UnitCoordinator, DragState};
use crate::view::{Camera, Screen};

/// Input boundary — every action the game engine can process,
/// regardless of source (keyboard, mouse, script, network, test harness).
#[derive(Debug, Clone)]
pub enum GameCommand {
    // Camera / view
    RotateCamera { delta_z: i16 },
    TiltCamera { delta_x: i16 },
    /// Screen-relative pan: forward/right in screen space.
    /// Resolved to grid shifts using camera.angle_z in apply_command.
    PanScreen { forward: f32, right: f32 },
    /// Direct grid shift (HJKL-style, not screen-relative).
    PanTerrain { dx: i32, dy: i32 },
    ResetCamera,
    TopDownView,
    CenterOnShaman,
    SetZoom(f32),

    // Curvature
    ToggleCurvature,
    AdjustCurvature { factor: f32 },

    // Level navigation
    NextLevel,
    PrevLevel,

    // Shader / rendering toggles
    NextShader,
    PrevShader,
    ToggleObjects,

    // Sunlight
    AdjustSunlight { dx: f32, dy: f32 },

    // Unit interaction (resolved game-level concepts, not raw screen coords)
    SelectUnit(usize),
    SelectMultiple(Vec<usize>),
    ClearSelection,
    OrderMove { x: f32, z: f32 },

    // Game state
    ToggleSimulation,

    // HUD
    SetHudTab(HudTab),

    // Lifecycle
    Quit,
}

/// Translate a winit KeyCode into a GameCommand.
/// Returns None for keys that have no game-command mapping.
pub fn translate_key(key: KeyCode) -> Option<GameCommand> {
    match key {
        // Orbit rotation
        KeyCode::KeyQ => Some(GameCommand::RotateCamera { delta_z: -5 }),
        KeyCode::KeyE => Some(GameCommand::RotateCamera { delta_z: 5 }),

        // Tilt
        KeyCode::ArrowUp => Some(GameCommand::TiltCamera { delta_x: 5 }),
        KeyCode::ArrowDown => Some(GameCommand::TiltCamera { delta_x: -5 }),

        // Screen-relative panning (WASD)
        KeyCode::KeyW => Some(GameCommand::PanScreen { forward: 1.0, right: 0.0 }),
        KeyCode::KeyS => Some(GameCommand::PanScreen { forward: -1.0, right: 0.0 }),
        KeyCode::KeyA => Some(GameCommand::PanScreen { forward: 0.0, right: -1.0 }),
        KeyCode::KeyD => Some(GameCommand::PanScreen { forward: 0.0, right: 1.0 }),

        // Direct grid panning (HJKL)
        KeyCode::KeyH => Some(GameCommand::PanTerrain { dx: 0, dy: -1 }),
        KeyCode::KeyL => Some(GameCommand::PanTerrain { dx: 0, dy: 1 }),
        KeyCode::KeyJ => Some(GameCommand::PanTerrain { dx: 1, dy: 0 }),
        KeyCode::KeyK => Some(GameCommand::PanTerrain { dx: -1, dy: 0 }),

        // Camera presets
        KeyCode::KeyR => Some(GameCommand::ResetCamera),
        KeyCode::KeyT => Some(GameCommand::TopDownView),
        KeyCode::Space => Some(GameCommand::CenterOnShaman),

        // Level navigation
        KeyCode::KeyB => Some(GameCommand::NextLevel),
        KeyCode::KeyV => Some(GameCommand::PrevLevel),

        // Shader cycling
        KeyCode::KeyN => Some(GameCommand::NextShader),
        KeyCode::KeyM => Some(GameCommand::PrevShader),

        // Toggles
        KeyCode::KeyC => Some(GameCommand::ToggleCurvature),
        KeyCode::KeyO => Some(GameCommand::ToggleObjects),

        // Curvature adjustment
        KeyCode::BracketRight => Some(GameCommand::AdjustCurvature { factor: 1.2 }),
        KeyCode::BracketLeft => Some(GameCommand::AdjustCurvature { factor: 0.8 }),

        // Sunlight
        KeyCode::KeyY => Some(GameCommand::AdjustSunlight { dx: -1.0, dy: -1.0 }),

        // Game simulation
        KeyCode::F5 => Some(GameCommand::ToggleSimulation),

        // Quit
        KeyCode::Escape => Some(GameCommand::Quit),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_camera_rotation() {
        match translate_key(KeyCode::KeyQ) {
            Some(GameCommand::RotateCamera { delta_z: -5 }) => {}
            other => panic!("expected RotateCamera -5, got {:?}", other),
        }
        match translate_key(KeyCode::KeyE) {
            Some(GameCommand::RotateCamera { delta_z: 5 }) => {}
            other => panic!("expected RotateCamera 5, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_tilt() {
        match translate_key(KeyCode::ArrowUp) {
            Some(GameCommand::TiltCamera { delta_x: 5 }) => {}
            other => panic!("expected TiltCamera 5, got {:?}", other),
        }
        match translate_key(KeyCode::ArrowDown) {
            Some(GameCommand::TiltCamera { delta_x: -5 }) => {}
            other => panic!("expected TiltCamera -5, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_wasd_pan() {
        match translate_key(KeyCode::KeyW) {
            Some(GameCommand::PanScreen { forward, right }) => {
                assert_eq!(forward, 1.0);
                assert_eq!(right, 0.0);
            }
            other => panic!("expected PanScreen forward, got {:?}", other),
        }
        match translate_key(KeyCode::KeyA) {
            Some(GameCommand::PanScreen { forward, right }) => {
                assert_eq!(forward, 0.0);
                assert_eq!(right, -1.0);
            }
            other => panic!("expected PanScreen left, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_hjkl_pan() {
        match translate_key(KeyCode::KeyH) {
            Some(GameCommand::PanTerrain { dx: 0, dy: -1 }) => {}
            other => panic!("expected PanTerrain dy=-1, got {:?}", other),
        }
        match translate_key(KeyCode::KeyL) {
            Some(GameCommand::PanTerrain { dx: 0, dy: 1 }) => {}
            other => panic!("expected PanTerrain dy=1, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_level_nav() {
        assert!(matches!(translate_key(KeyCode::KeyB), Some(GameCommand::NextLevel)));
        assert!(matches!(translate_key(KeyCode::KeyV), Some(GameCommand::PrevLevel)));
    }

    #[test]
    fn test_translate_toggles() {
        assert!(matches!(translate_key(KeyCode::KeyC), Some(GameCommand::ToggleCurvature)));
        assert!(matches!(translate_key(KeyCode::KeyO), Some(GameCommand::ToggleObjects)));
        assert!(matches!(translate_key(KeyCode::F5), Some(GameCommand::ToggleSimulation)));
    }

    #[test]
    fn test_translate_curvature_adjust() {
        match translate_key(KeyCode::BracketRight) {
            Some(GameCommand::AdjustCurvature { factor }) => assert_eq!(factor, 1.2),
            other => panic!("expected AdjustCurvature 1.2, got {:?}", other),
        }
        match translate_key(KeyCode::BracketLeft) {
            Some(GameCommand::AdjustCurvature { factor }) => assert_eq!(factor, 0.8),
            other => panic!("expected AdjustCurvature 0.8, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_presets() {
        assert!(matches!(translate_key(KeyCode::KeyR), Some(GameCommand::ResetCamera)));
        assert!(matches!(translate_key(KeyCode::KeyT), Some(GameCommand::TopDownView)));
        assert!(matches!(translate_key(KeyCode::Space), Some(GameCommand::CenterOnShaman)));
    }

    #[test]
    fn test_translate_quit() {
        assert!(matches!(translate_key(KeyCode::Escape), Some(GameCommand::Quit)));
    }

    #[test]
    fn test_translate_unmapped_returns_none() {
        assert!(translate_key(KeyCode::F1).is_none());
        assert!(translate_key(KeyCode::Enter).is_none());
        assert!(translate_key(KeyCode::Tab).is_none());
    }
}

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

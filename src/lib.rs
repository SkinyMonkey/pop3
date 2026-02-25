pub mod model;
pub mod default_model;
pub mod tex_model;
pub mod color_model;
pub mod view;
pub mod intersect;
pub mod envelop;
pub mod geometry;
pub mod gpu;
pub mod data;
pub mod landscape;
pub mod movement;
pub mod unit_control;
pub mod game_state;
pub mod hud;
pub mod sprites;
pub mod buildings;
pub mod engine;
/// Backward compat — use `engine::command` directly.
pub mod game_command {
    pub use crate::engine::command::*;
    pub use crate::engine::frame::*;
}

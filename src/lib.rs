pub mod data;
pub mod engine;
pub mod render;
pub mod hud;
pub mod sprites;

// Backward compat re-exports — old paths still work
pub mod model { pub use crate::render::model::*; }
pub mod default_model { pub use crate::render::default_model::*; }
pub mod tex_model { pub use crate::render::tex_model::*; }
pub mod color_model { pub use crate::render::color_model::*; }
pub mod view { pub use crate::render::camera::*; }
pub mod intersect { pub use crate::render::picking::*; }
pub mod envelop { pub use crate::render::envelop::*; }
pub mod geometry {
    pub use crate::render::geometry::*;
}
pub mod gpu {
    pub use crate::render::gpu::*;
}
pub mod landscape { pub use crate::render::terrain::*; }
pub mod buildings { pub use crate::render::buildings::*; }
pub mod game_command {
    pub use crate::engine::command::*;
    pub use crate::engine::frame::*;
}
pub mod game_state {
    pub use crate::engine::state::*;
}
pub mod movement {
    pub use crate::engine::movement::*;
}
pub mod unit_control {
    pub use crate::engine::units::*;
}

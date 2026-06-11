//! Everything visual + GPU (wgpu/winit).
//! `crate::data` / `crate::engine` resolve via the re-exports below so module
//! paths survive the workspace split unchanged.
pub use pop3_data::data;
pub use pop3_engine::engine;
pub mod render;

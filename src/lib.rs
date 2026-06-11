//! Thin facade: re-exports the workspace crates under the historical
//! `pop3::data` / `pop3::engine` / `pop3::render` paths used by main.rs
//! and the viewer binaries.
pub use pop3_data::data;
pub use pop3_engine::engine;
pub use pop3_render::render;

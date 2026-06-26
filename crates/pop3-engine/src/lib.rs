//! Game simulation — no GPU types.
//! `crate::data` resolves via the re-export below so module paths survive
//! the workspace split unchanged; normalize paths during the engine rewrite.
pub use pop3_data::data;
pub mod engine;

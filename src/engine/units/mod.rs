// Unit control system — selection, movement orders, and per-tick updates.
//
// Provides click-to-select, right-click-to-move, and drag-box multi-select
// for person units, wired to the movement system's pathfinding.

pub mod coords;
pub mod unit;
pub mod selection;
pub mod person_state;
pub mod coordinator;

pub use unit::{Unit, UnitId};
pub use selection::{SelectionState, DragState, find_unit_at_cell};
pub use coordinator::UnitCoordinator;
pub use coords::{world_to_cell, cell_to_world, gpu_to_cell};

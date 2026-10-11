//! SEA Forge SeaCell federation (spec-full §7.4).
//!
//! Cell identity (`cell.json`), tar bundle export/import with atomic-reject
//! (§14.8), and template adoption gating. Kernel crate: synchronous, no
//! network, no async.

#![forbid(unsafe_code)]

pub mod bundle;
pub mod cell;
pub mod template;

pub use bundle::{export, import, imported_template_path, read_manifest};
pub use cell::{ensure, is_known_schema, read as read_cell, CellRecord};
pub use template::adopt;

//! Synchronous governed capability-execution kernel.

#![forbid(unsafe_code)]

pub mod authority;
pub mod capability;
pub mod domain;
pub mod errors;
pub mod evidence;
pub mod ids;
pub mod pipeline;
pub mod planner;
pub mod runtime;
pub mod sandbox;
pub mod settlement;
pub mod trace;
pub mod types;

pub const RECORD_VERSION: &str = "0.1";

pub use errors::ForgeError;
pub use pipeline::{run_intent, RunOptions, RunOutcome};

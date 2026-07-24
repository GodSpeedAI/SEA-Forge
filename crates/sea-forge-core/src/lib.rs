//! Synchronous governed capability-execution kernel types.

#![forbid(unsafe_code)]

pub mod errors;
pub mod ids;
pub mod types;

pub const RECORD_VERSION: &str = "0.1";

pub use errors::ForgeError;

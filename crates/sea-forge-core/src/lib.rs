//! Synchronous governed capability-execution kernel types.

#![forbid(unsafe_code)]

pub mod approvals;
pub mod canonical;
pub mod errors;
pub mod ids;
pub mod path;
pub mod pipe_name;
pub mod types;
pub mod yaml;

pub use pipe_name::pipe_name_for;

pub const RECORD_VERSION: &str = "0.1";

pub use errors::ForgeError;

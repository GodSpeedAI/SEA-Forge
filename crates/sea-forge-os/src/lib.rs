//! Operating-system bindings that have no safe std equivalent.
//!
//! This is the only crate in the workspace allowed to contain `unsafe` code.
//! Every `unsafe` block states the invariant it relies on in a `SAFETY`
//! comment. The bodies are Windows-only; other platforms compile this crate
//! to an empty library, and their code paths never reference it.

#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(windows)]
mod imp;

#[cfg(windows)]
pub use imp::*;

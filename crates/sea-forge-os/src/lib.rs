//! Operating-system bindings that have no safe std equivalent.
//!
//! This crate holds all Windows FFI `unsafe` code. The only other `unsafe`
//! in the workspace is the pre-existing Linux sandbox code (seccomp notify in
//! `sea-forge-sandbox/src/socket_policy.rs`, raw syscalls in
//! `sea-forge-workload-exec`).
//! Every `unsafe` block states the invariant it relies on in a `SAFETY`
//! comment. The bodies are Windows-only; other platforms compile this crate
//! to an empty library, and their code paths never reference it.

#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(windows)]
mod imp;

#[cfg(windows)]
pub use imp::*;

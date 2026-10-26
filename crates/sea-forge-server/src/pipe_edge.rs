//! Windows named-pipe adapter at the server edge (review F33).
//!
//! `sea-forge-os` owns the synchronous pipe primitives (`PipeSecurity`,
//! `current_sid`, `pipe_client_sid`) and stays tokio-free, so tokio can
//! never enter a kernel crate's dependency graph. The one async piece —
//! handing those security attributes to tokio when it creates a named-pipe
//! server instance — lives here, in this approved adapter crate.
//!
//! This module carries the adapter's own `unsafe` under the same discipline
//! as `sea-forge-os`: every `unsafe` block states the invariant it relies on
//! in a `SAFETY` comment. The crate-root lint stays `deny(unsafe_code)`;
//! this module is the single documented opt-out.

#![allow(unsafe_code)]

use std::io;

/// Creates one named pipe server instance. `first` maps to
/// `FILE_FLAG_FIRST_PIPE_INSTANCE`: an existing pipe of that name makes the
/// call fail, so a squatter cannot be silently joined.
pub fn create_pipe_server(
    name: &str,
    security: &sea_forge_os::PipeSecurity,
    first: bool,
) -> io::Result<tokio::net::windows::named_pipe::NamedPipeServer> {
    let mut options = tokio::net::windows::named_pipe::ServerOptions::new();
    options
        .first_pipe_instance(first)
        .reject_remote_clients(true);
    // SAFETY: `attributes_ptr` points at a SECURITY_ATTRIBUTES whose descriptor stays alive for the borrow of `security`; tokio passes it to CreateNamedPipeW synchronously and keeps no reference.
    unsafe { options.create_with_security_attributes_raw(name, security.attributes_ptr()) }
}

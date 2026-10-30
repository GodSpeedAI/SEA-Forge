# sea-forge-os

## Overview
SEA Forge crate for OS bindings that have no safe `std` equivalent (`Cargo.toml` description). It is the only crate in the workspace that may contain `unsafe` code: all Windows FFI lives here — named-pipe security descriptors and client-SID impersonation, owner-only private-file writes, reparse-point-refusing pinned directories, and kill-on-close Job Object process spawning. On non-Windows platforms the crate compiles to an empty library and other code paths never reference it.

## Interface
- Source contract: [./src/README.md] — `pipe_client_sid`/`current_sid`, `PipeSecurity`, `create_pipe_server`, `write_private`, `PinnedDir`, `spawn_in_job`/`Job` [src/lib.rs:pub use imp::*], [src/imp.rs:PipeSecurity], [src/imp.rs:spawn_in_job]
- Workspace manifest: `Cargo.toml` (package `sea-forge-os`; `windows-sys` under `cfg(windows)`; optional `tokio` behind the `tokio-pipes` feature, enabled only by `sea-forge-server`)

## Invariants
- Sole holder of workspace `unsafe` code; every `unsafe` block documents its invariant in a `SAFETY` comment; `#![deny(unsafe_op_in_unsafe_fn)]` [src/lib.rs:module doc], [src/imp.rs:SAFETY comments]
- SID strings are grammar-checked before splicing into SDDL (no ACE/flag injection) [src/imp.rs:check_sid_grammar]
- Pipe clients get `CLIENT_PIPE_RIGHTS`, deliberately excluding `FILE_CREATE_PIPE_INSTANCE` so a bound client cannot start a competing server instance [src/imp.rs:CLIENT_PIPE_RIGHTS]
- Pinned directories never follow reparse points and hold a no-`FILE_SHARE_DELETE` handle so the pinned path keeps naming the same directory [src/imp.rs:PinnedDir]
- Children spawn suspended into a kill-on-close job and are resumed only after assignment, so no grandchild escapes the job; post-spawn failures terminate the tree [src/imp.rs:spawn_in_job]
- A failed `RevertToSelf` on drop aborts the process (privilege-leak policy) [src/imp.rs:Impersonation]

## Side Effects
- Windows kernel/FFI: named-pipe creation and impersonation, token/process/thread queries, Job Objects, process-tree termination [src/imp.rs]
- Filesystem (Windows): owner-only-DACL file writes (`write_private`), containment-checked directory operations via `PinnedDir` [src/imp.rs:write_private], [src/imp.rs:PinnedDir]

## Verification
- Run: `cargo test -p sea-forge-os` (repo-wide: `just test`; non-Windows builds compile the crate empty, so behavioral coverage exists only on Windows)

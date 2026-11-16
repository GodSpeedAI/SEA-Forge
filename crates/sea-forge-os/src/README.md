# sea-forge-os/src

## Overview
Source for the `sea-forge-os` crate: operating-system bindings that have no safe `std` equivalent. This crate holds SEA Forge's Windows FFI `unsafe` code (named-pipe security and impersonation, owner-only private-file writes, reparse-point-refusing pinned directories, kill-on-close job objects); narrow `#![allow(unsafe_code)]` scopes also exist in `sea-forge-sandbox`'s workload-exec helper and socket policy. `lib.rs` is a thin platform gate; `imp.rs` is the Windows implementation. On non-Windows platforms the crate compiles to an empty library and other code paths never reference it.

## Interface
- Module gate: `imp` is compiled only under `#[cfg(windows)]` and re-exported via `pub use imp::*` [lib.rs:cfg attribute], [lib.rs:pub use]
- SID queries: `pipe_client_sid(pipe)`, `pipe_server_sid(pipe)`, `current_sid()` — SIDs are read from the kernel token, so a client SID cannot be spoofed by a reused process id [imp.rs:pipe_client_sid], [imp.rs:current_sid]
- `CLIENT_PIPE_RIGHTS: u32 = 0x0012_018B` — rights granted to each bound client SID on the pipe [imp.rs:CLIENT_PIPE_RIGHTS]
- `PipeSecurity` with `PipeSecurity::new(sids)` — builds the pipe DACL from grammar-checked SIDs [imp.rs:PipeSecurity]
- `create_pipe_server(name, security, first)` — tokio named-pipe server constructor (requires the `tokio-pipes` feature, enabled only by `sea-forge-server`) [imp.rs:create_pipe_server], [Cargo.toml:tokio-pipes]
- `write_private(path, bytes)` — writes a file with an owner-only DACL (SYSTEM + current user full control, everyone else denied) [imp.rs:write_private]
- `PinnedDir`, `EntryState`, `PinError` — containment-checked directory writes: entries are opened without following reparse points; symlinks/junctions are refused, and every step must be a direct child of its pinned parent [imp.rs:PinnedDir], [imp.rs:PinError], [imp.rs:EntryState]
- `spawn_in_job(cmd) -> (Child, Job)` and `Job` (with `Job::kill_on_close`) — spawn a child suspended into a kill-on-close job object, then resume its threads [imp.rs:spawn_in_job], [imp.rs:Job]

## Invariants
- Every `unsafe` block states the invariant it relies on in a `SAFETY` comment [lib.rs:module doc], [imp.rs:SAFETY comments]
- `#![deny(unsafe_op_in_unsafe_fn)]` [lib.rs:deny attribute]
- SID strings are grammar-checked (`S-1-<digits>-...`) before being spliced into SDDL, so a SID cannot inject ACE or flag syntax [imp.rs:check_sid_grammar]
- `GENERIC_WRITE` is deliberately not granted to pipe clients: its mapping includes `FILE_APPEND_DATA`, which is `FILE_CREATE_PIPE_INSTANCE` on a named pipe and would let a bound client start a competing server instance [imp.rs:CLIENT_PIPE_RIGHTS]
- The pipe DACL grants full control only to SYSTEM and the current user; each bound SID gets only `CLIENT_PIPE_RIGHTS`; everyone else is denied by omission [imp.rs:PipeSecurity]
- `pipe_client_sid` requires the client to have written at least one byte (impersonation fails with `ERROR_CANNOT_IMPERSONATE` before the first read) [imp.rs:pipe_client_sid]
- A `PinnedDir` handle is opened without `FILE_SHARE_DELETE`, so the path recorded at open time keeps naming the same directory for the handle's lifetime; reparse points are opened with `FILE_FLAG_OPEN_REPARSE_POINT` and refused, never followed [imp.rs:PinnedDir], [imp.rs:PinError::ReparsePoint]
- `spawn_in_job` assigns the suspended child to the job before resuming threads, so a grandchild cannot be created outside the job; on any failure after spawn the process tree is terminated and the error returned [imp.rs:spawn_in_job]
- `CREATE_SUSPENDED` set by `spawn_in_job` replaces any flags set earlier on the `Command` (std does not expose merging); workspace callers set no other creation flags [imp.rs:spawn_in_job]
- If `RevertToSelf` fails on drop (a thread left impersonating a remote client is a privilege leak), the process aborts — there is no safe way to continue [imp.rs:Impersonation]

## Side Effects
- Windows FFI / kernel effects: named-pipe creation with security attributes, client impersonation, process/thread and token queries, `RevertToSelf`, `process::abort` on privilege-leak risk [imp.rs]
- Filesystem (Windows): creates files with an owner-only DACL (`write_private`); opens/pins directories; renames/deletes via pinned handles; may refuse operations on reparse points [imp.rs:write_private], [imp.rs:PinnedDir]
- Process management: creates Job Objects, assigns and terminates process trees (`spawn_in_job`, `Job`) [imp.rs:spawn_in_job]

## Verification
- Run: `cargo test -p sea-forge-os` (repo-wide: `just test`; on non-Windows the crate compiles empty, so behavioral coverage only exists on Windows)

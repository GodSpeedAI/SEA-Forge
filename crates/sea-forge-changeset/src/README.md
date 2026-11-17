# sea-forge-changeset — src

## Overview
Single-module source tree for git-backed transactional workspaces. Untrusted
execution proposes changes; it never writes the authoritative repository. The
lifecycle: `capture_base` pins the start commit/tree, `export_base` materializes
it as a plain directory (no `.git`) for sandbox upload, `collect` turns the
returned directory into a proposed tree and explicit `Changeset` in a separate
staging repository, `validate` applies path/file-type/protected-path rules, and
`promote` moves the authoritative ref with a compare-and-swap
[lib.rs:module docs].

## Interface
All symbols in [lib.rs]:
- Lifecycle functions: `capture_base`, `export_base`, `collect`, `validate`, `promote`.
- Types: `BaseIdentity`, `ChangeStatus`, `ChangeEntry`, `Changeset`, `ProtectedDecision`, `ChangeRules`, `Finding`, `Validation`, `PromotionAuthorization`, `Promotion`, `ChangesetError`.
- Schema constant: `CHANGESET_SCHEMA = "sea-forge.changeset/v1"` [lib.rs:CHANGESET_SCHEMA].

## Invariants
- Untrusted execution never writes the authoritative repository; only `promote` moves the authoritative ref, and only for the exact changeset that was authorized [lib.rs:module docs], [lib.rs:promote].
- `promote` is compare-and-swap via `update-ref <ref> <new> <old>`: a moved ref is a conflict [lib.rs:module docs].
- Atomicity means the ref update only; effects outside the repository are not covered [lib.rs:module docs].
- Every Git invocation ignores user and system configuration: env is cleared, `GIT_CONFIG_GLOBAL`/`GIT_CONFIG_SYSTEM` point to `/dev/null`, hooks path is `/dev/null`, and autocrlf/fsmonitor/excludesFile are disabled, so hooks, filters, aliases, and global ignores cannot change what is staged [lib.rs:Git::command].
- The repository itself is never exposed to the sandbox: `export_base` materializes the tree without `.git` [lib.rs:export_base].
- A workspace `.gitignore` is honoured: ignored new files are never promoted; a base file that became ignored appears as an explicit deletion [lib.rs:module docs].

## Side Effects
- Spawns external `git` processes (with sanitized environment and null stdin) [lib.rs:Git::command].
- Creates staging directories and index files, writes tree/state files, and reads/writes workspace files via `std::fs` [lib.rs:export_base], [lib.rs:collect].

## Verification
- Run: `cargo test -p sea-forge-changeset` (repo-wide: `just test`)

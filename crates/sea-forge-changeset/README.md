# sea-forge-changeset

## Overview
SEA Forge git-backed transactional workspaces and changesets (Cargo.toml).
Untrusted execution proposes changes through a staging repository; the
authoritative ref moves only via a compare-and-swap promotion of the exact
authorized changeset. Git's object store and `update-ref` provide identity and
atomicity; the crate adds no store of its own [src/lib.rs:module docs].

## Interface
- Single module `lib.rs`; lifecycle: `capture_base`, `export_base`, `collect`, `validate`, `promote` [src/lib.rs].
- Types: `BaseIdentity`, `Changeset`, `ChangeEntry`, `ChangeStatus`, `ChangeRules`, `Finding`, `Validation`, `ProtectedDecision`, `PromotionAuthorization`, `Promotion`, `ChangesetError`; schema `CHANGESET_SCHEMA = "sea-forge.changeset/v1"` [src/lib.rs].
- Source tree contract: [src/README.md]

## Invariants
- Untrusted execution never writes the authoritative repository [src/lib.rs:module docs].
- Promotion is compare-and-swap; a moved ref is a conflict [src/lib.rs:promote].
- All Git invocations run with sanitized environment (no user/system config, no hooks, no filters, no terminal prompt) [src/lib.rs:Git::command].
- The sandbox only ever receives a `.git`-free tree export [src/lib.rs:export_base].

## Side Effects
- Spawns `git` processes; creates staging repositories and index files; reads/writes workspace files [src/lib.rs:Git::command], [src/lib.rs:collect].

## Verification
- Run: `cargo test -p sea-forge-changeset` (repo-wide: `just test`)

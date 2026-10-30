# Crates Blueprint

## Overview
All 25 workspace crates of the governed capability-execution kernel. The kernel is synchronous end-to-end; the only async crates are the two edges (`sea-forge-server`, `sea-forge-agent`). Every crate root and `src/` tree carries its own EGM contract one level down.

## Subdirectories & Boundaries
- [sea-forge-agent](sea-forge-agent/README.md) - Governed HTTP agent provider adapters crate (OpenAI-compatible, Anthropic, ACP driver); owns transport only while authority, evidence, and settlement stay in the server.
- [sea-forge-artifact-ip](sea-forge-artifact-ip/README.md) - SEA Forge artifact-to-IP registration and transition validation crate (M8 overlay turning settled artifacts into recognized IP with lifecycle, rights, evidence, and attestation).
- [sea-forge-authority](sea-forge-authority/README.md) - SEA Forge authority fabric: policy, identity, and governance verdicts (engine, grant minting, CEP loop, OPA candidate).
- [sea-forge-capability](sea-forge-capability/README.md) - SEA Forge capability memory and extension validation crate (memory store plus promotion of settled declarations into capability records).
- [sea-forge-case-runner](sea-forge-case-runner/README.md) - Synchronous lifecycle primitives for SEA Forge cases: governed mutations shared by CLI and server, advancement engine, and governed cell episodes with approved promotion.
- [sea-forge-cell](sea-forge-cell/README.md) - SEA Forge SeaCell federation crate: cell identity, bundle export/import (spec-full §7.4); synchronous kernel crate, no network, no async.
- [sea-forge-changeset](sea-forge-changeset/README.md) - SEA Forge git-backed transactional workspaces and changesets crate; git's object store and update-ref provide identity and atomicity, the crate adds no store of its own.
- [sea-forge-cli](sea-forge-cli/README.md) - SEA Forge one-shot CLI (binary sea-forge): operator command surface over governed kernel paths, exiting with the governed dispatch code.
- [sea-forge-core](sea-forge-core/README.md) - Synchronous kernel foundation crate: governed types and lifecycle, IDs, canonical hashing, approvals journal, path validation, strict YAML; dependency of every other kernel crate.
- [sea-forge-domain](sea-forge-domain/README.md) - SEA Forge crate for the intent vocabulary and domain planning primitives; classifies intent summaries into a closed set of IntentPattern values for the planner.
- [sea-forge-domainforge](sea-forge-domainforge/README.md) - SEA Forge's first-party DomainForge semantic adapter over pinned domainforge-core 0.21.1: validate .sea source sets, project to CALM/RDF/KG, bind semantic worlds.
- [sea-forge-evidence](sea-forge-evidence/README.md) - SEA Forge crate for evidence and artifact capture: shared hashing/canonicalization primitives, sequence-checked JSONL evidence journal, symlink-safe artifact capture.
- [sea-forge-extension](sea-forge-extension/README.md) - SEA Forge crate for extension descriptor and registry contracts (spec-full 7.0b): persisted registry, trust levels, projection-adapter ABI, and the pure CEP-0008 v1 flat-profile adapter.
- [sea-forge-ledger](sea-forge-ledger/README.md) - SEA Forge integrity ledger crate: append-only, hash-chained, MMR-committed, signed streams with bounded reader, checkpoints, crash repair, Ed25519 signing, and the exposure registry (SEC-09/10/11).
- [sea-forge-os](sea-forge-os/README.md) - SEA Forge OS bindings crate: holds all Windows FFI (pipe security/impersonation, owner-only file writes, pinned dirs, job objects); the FFI `unsafe` holder (narrow `allow(unsafe_code)` scopes also exist in sea-forge-sandbox).
- [sea-forge-planner](sea-forge-planner/README.md) - SEA Forge case plan construction crate: intent-driven plan(), canonical spec-pipeline stage plans, deterministic case engine, and settlement-criteria derivation/verification.
- [sea-forge-realitytrace](sea-forge-realitytrace/README.md) - SEA Forge crate providing ledger-derived security evidence (sea-forge.security-trace/v1) for RealityTrace and auditors; evidence consumer only, no execution crate depends on it.
- [sea-forge-runtime](sea-forge-runtime/README.md) - SEA Forge command execution runtime: the seam where an authorized ActionGrant becomes a sandboxed process run with an ExecutionResult.
- [sea-forge-sandbox](sea-forge-sandbox/README.md) - SEA Forge sandbox backends and workspace path safety: the single choke point through which every governed command execution reaches a process.
- [sea-forge-self-model](sea-forge-self-model/README.md) - SEA Forge Genesis self-model crate (spec-adlc-thoth E11, M9): bundled models, deterministic realizations, projections, and immutable hash-linked snapshots.
- [sea-forge-server](sea-forge-server/README.md) - SEA Forge concurrent case server (M3): async-edge crate serving NDJSON over a Unix socket, dispatching case runs, with SFWP, identity, delegation, CEP, MCP, and Postgres subsystems.
- [sea-forge-settlement](sea-forge-settlement/README.md) - SEA Forge settlement evaluation crate: settles claims into SettlementEvents and owns the hash-chained settlement declaration ledger.
- [sea-forge-spec-pipeline](sea-forge-spec-pipeline/README.md) - SEA Forge spec-to-code pipeline and DomainForge projections crate: pure deterministic pipeline logic run as ordinary governed work.
- [sea-forge-thoth](sea-forge-thoth/README.md) - SEA Forge Thoth crate (spec-adlc-thoth E13): typed governed epistemic interface answering from the self-model snapshot under disclosure policy with a ledgered evidence chain.
- [sea-forge-trace](sea-forge-trace/README.md) - SEA Forge trace event recording crate: per-run JSONL trace journals and the external event-sink bridge contract.

## Global Invariants
- Synchronous kernel boundary: only `sea-forge-server` and `sea-forge-agent` may be async (see `crates/AGENTS.md`); enforce with `just no-async-kernel` after dependency-boundary changes.
- `#![forbid(unsafe_code)]` is declared in multiple crates (e.g. `sea-forge-domainforge`, `sea-forge-extension`, `sea-forge-ledger`); do not weaken it.
- Package-scoped feedback loop: `just crate-check` / `just crate-test` per crate; repo-wide gates are `just check` and `just test`.

## Global Side Effects
- `None` at this level: the blueprint groups child crates; all side effects are declared in the child contracts.

## Verification
- Run: `just test` (repo-wide); per crate `cargo test -p <crate_name>`.

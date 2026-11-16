# sea-forge-thoth

## Overview

SEA Forge Thoth: typed governed epistemic interface (spec-adlc-thoth E13).
Thoth answers typed questions about kernel state from the self-model snapshot
under a per-actor disclosure policy, committing the full
question → disclosure-plan → decision → answer evidence chain to the ledger.
Deterministic engine, pure protocol types, a mediated service entrypoint, and
a deterministic manager-loop judgment helper.

## Interface

- Crate contract lives in [./src/README.md](./src/README.md) (module tree:
  `engine.rs`, `manager.rs`, `protocol.rs`, `service.rs`).
- Key entry points, cited from source:
  - `ask(root, actor_id, kind, subject, purpose, case_id)` — the one mediated
    ask entrypoint; CLI and server adapters (T14B) call this and do nothing
    else governance-related (`src/service.rs:ask`).
  - `answer_question(question, snapshot, policy, answer_id, answered_at)` —
    pure disclosure derivation (`src/engine.rs:answer_question`).
  - Extension traits: `SnapshotView` (`src/engine.rs:SnapshotView`),
    `DisclosurePolicy` (`src/engine.rs:DisclosurePolicy`).
  - Manager judgment: `judge(view)` over `ManagerView`
    (`src/manager.rs:judge`).
  - Protocol types: `ThothQuestion`, `ThothAnswer`, `ClaimClass`,
    `QuestionKind`, `AUTHORITY_NOTICE` (`src/protocol.rs:ThothAnswer`).

## Invariants

- Knowledge confers no execution authority: every answer discloses freshness,
  assurance, limitations, omitted claim classes, and the fixed
  `AUTHORITY_NOTICE` (`src/protocol.rs:AUTHORITY_NOTICE`).
- Answers are deterministic over (question, snapshot, policy); required-fresh
  policies refuse before content queries (§7.4, T13.5;
  `src/engine.rs:answer_question`).
- The full evidence chain is committed before `ask` returns (§10.3, T14A;
  `src/service.rs:ask`).

## Side Effects

- Appends to the `thoth-asks` ledger stream under the cell root on every ask
  (`src/service.rs:ASK_LEDGER_STREAM`).
- Reads ledgers, cell realizations, and surface policy from the cell root
  (`src/service.rs:LedgerSnapshotView`).

## Verification

- Run: `cargo test -p sea-forge-thoth` (repo-wide: `just test`)

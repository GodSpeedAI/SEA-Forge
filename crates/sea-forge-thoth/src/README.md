# sea-forge-thoth — src

## Overview

Implementation of the `sea-forge-thoth` crate: SEA Forge Thoth, the typed
governed epistemic interface (spec-adlc-thoth E13). Thoth answers questions
about the kernel's state from an immutable self-model snapshot under a
disclosure policy, and commits the complete question → disclosure-plan →
decision → answer evidence chain to a ledger. Answers confer no execution
authority.

## Interface

- Module tree: `engine`, `manager`, `protocol`, `service`
  (`src/lib.rs:engine`, `src/lib.rs:manager`, `src/lib.rs:protocol`,
  `src/lib.rs:service`); `engine::*` and `protocol::*` are re-exported by
  `lib.rs`.
- Protocol types (`protocol.rs`):
  - `ClaimStatus`, `ClaimFlags`, `ClaimClass`, `GroundedClaim`
    (`protocol.rs:ClaimClass`, `protocol.rs:GroundedClaim`).
  - `claim_class_to_surface_str(c)` (`protocol.rs:claim_class_to_surface_str`);
    `is_high_risk(class)` (`protocol.rs:is_high_risk`).
  - Questions: `QuestionKind`, `parse_question_kind(s)`, `ThothQuestion`,
    `candidate_classes(kind)` (`protocol.rs:QuestionKind`,
    `protocol.rs:candidate_classes`).
  - Answers: `DisclosurePlan`, `Disposition`, `Freshness`, `ThothAnswer`,
    and `AUTHORITY_NOTICE` — the fixed notice "This answer confers no
    execution authority." (`protocol.rs:AUTHORITY_NOTICE`,
    `protocol.rs:ThothAnswer`).
- Engine (`engine.rs`): pure disclosure derivation —
  - `answer_question(question, snapshot, policy, answer_id, answered_at) ->
    ThothAnswer` — deterministic over (question, snapshot, policy): classify
    → policy derivation → answer assembly (`engine.rs:answer_question`).
  - Extension points: trait `SnapshotView` (`engine.rs:SnapshotView`), trait
    `DisclosurePolicy` (`engine.rs:DisclosurePolicy`); view types
    `CapabilityInfo`, `EnvironmentInfo`, `RecordedAuthorityDecision`;
    `THOTH_ACTOR_ID = "thoth"` (`engine.rs:THOTH_ACTOR_ID`).
- Manager judgment (`manager.rs`):
  - `ManagerView` — deterministic case-state facts the manager loop judges
    against (`manager.rs:ManagerView`).
  - `judge(view) -> ManagerJudgment` — classify a case per §9.5's rule table;
    never invents free-form judgment, purely a function of the observed facts
    (`manager.rs:judge`).
- Service (`service.rs`): the one mediated ask entrypoint —
  - `ask(root, actor_id, kind, subject, purpose, case_id) -> Result<ThothAnswer,
    ForgeError>` (`service.rs:ask`): validates inputs, opens the real snapshot
    view, derives the disclosure policy, resolves the answer, and commits the
    full chain before returning (§10.3, T14A). CLI and server adapters (T14B)
    call this and do nothing else governance-related.
  - `LedgerSnapshotView` — the real snapshot view over the ledger
    (`service.rs:LedgerSnapshotView`).
  - Limits: `MAX_PURPOSE_LEN = 500` (`service.rs:MAX_PURPOSE_LEN`); ledger
    stream `thoth-asks` (`service.rs:ASK_LEDGER_STREAM`).

## Invariants

- Answers confer no execution authority; every answer carries
  `AUTHORITY_NOTICE` (`protocol.rs:AUTHORITY_NOTICE`).
- The engine is deterministic over (question, snapshot, policy) and IO-free;
  a required-fresh policy refuses before any bounded query method on the
  snapshot is invoked — only metadata (`snapshot_ref`/`is_stale`) is ever
  called before that gate (§7.4, T13.5; `engine.rs:answer_question`).
- Denial names claim classes only, never free-form reasons (§10.3;
  `engine.rs:answer_question`).
- A read-only disclosure query never writes a materialized capability record
  as a side effect (`service.rs`, rebuild_capability comments).
- `ask` requires a non-empty actor identity and typed subject; purpose is
  capped at `MAX_PURPOSE_LEN`; `case_id`, when given, must be non-empty
  (`service.rs:ask`).
- Manager judgment is a pure function of observed facts — never free-form
  (`manager.rs:judge`).

## Side Effects

- Ledger writes: `ask` commits the question, disclosure plan, decision, and
  answer to the `thoth-asks` ledger stream via `LedgerStream::commit_typed`
  (`service.rs:ask`, `service.rs:ASK_LEDGER_STREAM`).
- Ledger/snapshot reads: `LedgerSnapshotView::open` reads the ledger and cell
  realizations under the cell root (`service.rs:LedgerSnapshotView`,
  `service.rs` reads `store::read_cell_realization`).
- Surface policy is loaded from the cell root per actor
  (`service.rs:ask` → `load_surface_policy`).

## Verification

- Run: `cargo test -p sea-forge-thoth` (repo-wide: `just test`)

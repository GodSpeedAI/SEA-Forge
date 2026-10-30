# sea-forge-case-runner — src/case_ops

## Overview
Governed case mutation operations shared by the CLI and the SFWP server.
Extracted verbatim from the CLI so case mutations have exactly one
implementation: the CLI renders results, the server calls them in-process, and
neither carries a private copy of a governed write. The library performs no
printing — callers render typed results [mod.rs:module docs].

Submodules:
- `advance` ([advance.rs]) — in-process case advancement (T04 step 2, decision D-3); one engine drives operator verbs and re-derives next actions from persisted case events alone.
- `authority_views` ([authority_views.rs]) — rebuildable authority mirror projections (`authority/decisions.jsonl`, `authority/opaque-constraints.json`, `authority/audit.jsonl`) from the append-only ledger.
- `mediation` ([mediation.rs]) — mediated authority checks for governed case mutations.

## Interface
- Notifier contract: `CaseEventNotifier<'a> = &mut dyn FnMut(&TraceEvent)` — host callback invoked after each case trace event is durably appended; every appending function has a `_with` spelling taking it, plain spellings use a no-op sink [mod.rs:CaseEventNotifier].
- Case mutations ([mod.rs]): `append_case_event`, `append_case_event_with`, `reopen`, `reopen_with`, `reopen_with_reason`, `terminate`, `terminate_with`, `propose_item`, `propose_item_with`, `complete_human_task`, `complete_human_task_with`, `load_case_plan`, `save_case`, `resolve_approval`.
- Advancement ([advance.rs]): `advance_case`, `AdvanceScope`, `EpisodeOutcome`, `AdvanceOutcome`, `EpisodeExecutor`, `MAX_EPISODES_PER_ADVANCE = 32`.
- Authority mirrors ([authority_views.rs]): `rebuild_authority_mirrors`, `load_opaque_constraints`.
- Mediation ([mediation.rs]): `authorize_action`, `with_authorized_action`, `AuthorityContext`, `with_authorized_action_in_context`, `with_authorized_action_binding`, `authorize_read`, `authorize_read_with_grant`.

## Invariants
- No printing inside the library; callers render typed results [mod.rs:module docs].
- Every append to `case-events.jsonl` goes through a `_with` spelling so a durable append always publishes a host event frame; no append lands without one [mod.rs:module docs].
- Advancement re-derives next actions from persisted case events alone (`next_case_actions`/`replay_case` from `sea-forge-planner`) [advance.rs:module docs].
- Enabled manual-activation items are activated here on purpose: `next_case_actions` returns `Enable` but never `Activate` for them — enabling is the sentry's half, activating the operator's (D-3) [advance.rs:module docs].
- Mediation contract: a mutation is not authorized until its decision is ledgered; stream, evidence, and mirror writes are part of the contract [mediation.rs:module docs].
- Episodes per advance are bounded by `MAX_EPISODES_PER_ADVANCE = 32` [advance.rs:MAX_EPISODES_PER_ADVANCE].

## Side Effects
- Appends to `case-events.jsonl` and case ledgers via `OpenOptions` append and `LedgerStream` [mod.rs:append_case_event_with].
- Case state is written atomically via temp file + `fs::rename` [mod.rs:save_case].
- Invokes host-supplied `EpisodeExecutor` callbacks which run SandboxedTask episodes [advance.rs:EpisodeExecutor].
- Writes authority mirror files and reads opaque constraints from disk [authority_views.rs:rebuild_authority_mirrors].
- Evaluates and commits authority decisions through `sea-forge-authority`'s engine and ledger streams [mediation.rs:authorize_action].

## Verification
- Run: `cargo test -p sea-forge-case-runner` (repo-wide: `just test`)

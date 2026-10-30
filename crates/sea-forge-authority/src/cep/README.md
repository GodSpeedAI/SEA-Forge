# sea-forge-authority — src/cep

## Overview
Obligation settlement module for the CEP (central authority) service. Settles
operations whose allow decision recorded `claims.modality = "obligation"` from
RealityTrace's own ledger (Horizon HORIZON-001 1.1.0, REQ-NAV-011/012), instead
of the evidence-item rule in `settle_with` [obligation.rs:module docs].

## Interface
Module tree: the single module `obligation` ([obligation.rs]), declared by
[src/cep.rs]. Public symbols in [obligation.rs]:
- `SourceRow` — one row of an evidence source's own ledger, read back after chain verification.
- `ObligationSource` — read access to RealityTrace's ledger (`ledger_rows`).
- `Obligation`, `obligation_of` — obligation reconstructed from the frozen allow-decision claims.
- `argv_digest` — digest of the expected verifier argv.
- `classify`, `RunResult`, `Frame` — admissibility classification of a verification run.
- `judge` — maps admissible known results to `(epistemic_status, settlement)` pairs.
- `ClosureAck`, `CLOSURE_KIND = "settlement_closure"`, `DEFAULT_CLOSE_GRACE_S = 30` [obligation.rs:CLOSURE_KIND], [obligation.rs:DEFAULT_CLOSE_GRACE_S]
- `CepAuthorityService::settle_obligation` (impl on the parent crate's service) [obligation.rs:CepAuthorityService].

## Invariants
- The threshold must exist in RealityTrace's ledger at exactly position T [obligation.rs:module docs].
- A closure later than deadline + grace cannot prove C on time; result is `unresolved`/`rejected` [obligation.rs:module docs].
- The discrimination run (position B > T) must not have observed the statement before acting [obligation.rs:check_discrimination].
- A verification run counts only if RealityTrace recorded it starting after SEA-Forge received the operation's first execution evidence packet [obligation.rs:execution_floor].
- Runs are admissible only at ledger positions in (B, C]; only "known" results (completion at/before deadline, exit code, no crash, expected argv) decide the outcome [obligation.rs:classify].
- Outcome mapping: 0-support-only → `supported`/settled; contradiction-only → `contradicted`/rejected; both → `inconsistent`/unsettled; none → `unresolved`/rejected [obligation.rs:judge].
- The window is closed and evidence frozen at C, so every outcome here is final [obligation.rs:module docs].

## Side Effects
- ["None"] — the module reads ledgers through the injected `ObligationSource` trait and returns results; no direct disk, network, or process effects.

## Verification
- Run: `cargo test -p sea-forge-authority` (repo-wide: `just test`)

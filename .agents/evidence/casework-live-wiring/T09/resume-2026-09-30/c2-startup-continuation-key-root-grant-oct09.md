# C2 startup continuation key: root scope grant

Date: 2026-10-09. Status: design approved; implementation held until root explicitly releases it after the private codec milestone commit.

## Authority and boundary

Implements the already approved startup policy in `.agents/specs/casework-live-cursor-v4-spec.yaml`, REQ-C2-RANGE-004 and V-C2-RANGE-01. This is a separate grant from the codec-only grant. T09 stays partial. It introduces no public wire, identity, persistence, dependency, or authorization policy change.

Allowed production/test file: `crates/sea-forge-server/src/lib.rs` only. Read applicable instructions and nearby tests first. Use native apply_patch. No Git, compilation, tests, formatting commands, or other gates by the builder; root serializes gates with available-memory preflight. Report new debt for root to route into `.agents/DEBT.md`.

## Required behavior

- Every `ServerState::new` constructs one fresh process-memory Ed25519 signing key using the existing `getrandom::fill` and ledger `SigningKey`.
- Keep the key in a private state field. Do not use the ledger's persisted-key loader, log/serialize key material, or add configuration.
- Validate configuration first, then acquire entropy before existing ServerState recovery/reconciliation/correlation/events-ledger work. Entropy failure returns `ForgeError::Internal` with a fixed nonsensitive message, so state construction fails before `run` binds/publishes its listener.
- Do not claim the whole `run` has no prior side effects: its existing path/lock preparation may precede state construction.
- Hold the 32-byte temporary seed in existing `zeroize::Zeroizing`; clear it immediately after key construction. Error and unwinding paths retain RAII cleanup. Use existing zeroization support for the retained signing key.
- Use a private one-shot entropy closure, without a mutable global hook. Prefer private `new_with_continuation_entropy` delegating through the same constructor body so tests exercise ordering. Public `new` remains the same signature and supplies `getrandom::fill`.
- Limit temporary dead-code allowance to the new private field with a comment explaining later reader integration. No codec or dispatch wiring in this unit.

## Tests first and review

First deliver tests plus the minimum compiling placeholder needed for a meaningful failing run; stop for root's RED gate before production implementation. Exercise deterministic success, distinct seed/key signature isolation, entropy failure after partial buffer fill, and configuration failure before requesting entropy. Prove entropy failure prevents ServerState's durable recovery/ledger work using a valid isolated test configuration and unchanged root inventory. Never print seed/key bytes. Source ordering establishes the existing listener boundary; no new public run hook is required.

After root observes the expected RED, implement the smallest coherent change. Independent critic receives this original grant and the implementation, checks exact scope and all error/success paths, cites source and actual gate captures, and lists every material deviation. Approval applies only to this startup unit. Root then integrates the separately granted bounded page reader.

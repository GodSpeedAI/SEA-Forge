# Stage 10 plan — world transitions

Governing spec: `.agents/specs/cep-world-ref-migration.spec.md` § Stage 10.

1. `sea-forge-authority/src/cep.rs`: `OperationKind::WorldTransition`; parse and verify the transition extension (T1-T3, T7); the approval floor (T4); the ledger record and `transitions()` (T5).
2. Tests in `sea-forge-authority/tests/cep_transition.rs`: equal-closure allow, unequal-closure floor, approval consumption, approval bound to the target, liar claims, same world, unknown world, deny stays deny, record validates against the cep schema (gated on `CEP_REPO`).
3. Server: expose `authority_transitions` (read-only) next to `authority_approvals`; conformance test over the socket.
4. Cognate: `SeaForgeAuthority.requestWorldTransition` builds the request; no world selection by Cognate.
5. Docs, report, status, DEBT.md; commit; push; PR.

Risk: the policy engine's handling of an unknown reserved resource type. Verify it denies by default before relying on it.

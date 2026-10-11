# Stage 10 settlement — world transitions

Date: 2026-10-05. Spec: `specs/cep-world-ref-migration.spec.md` § Stage 10. Plan: `plans/2026-10-05-stage10-world-transitions.md` (both written before coding). Normative source: cep `spec/profiles/GODSPEED-PROFILES-v1.md` §5 and `world-transition.v1`.

## What a transition is
A `world_ref` is immutable and any source edit, even a comment, mints a new one. Work pinned to the old world moves by one explicit, governed, recorded act. Never by retargeting an alias.

## Delivered
- **SEA-Forge** (`sea-forge-authority/src/cep.rs`, server verb `authority_transitions`, policy kind `world_transition`):
  - A transition is an `authority_request` with `operation_kind: world_transition`, pinned to the SOURCE world; `resource_id` is the TARGET, so an approval is for one source, one target and one requester.
  - SEA-Forge recomputes `semantic_closure_equal` from the two registered identities and refuses claims that contradict them (`source_edit_only` and `compiler_upgrade` need equal closure, `semantic_change` needs unequal). Unknown, identical or alias worlds are refused before any ledger write.
  - Floor: a move across a semantic change can never be allowed without an approved escalation, whatever policy says. A policy deny stays a deny. A meaning-preserving move follows policy.
  - Compatibility is recorded `compatible` only when the closures are equal; otherwise it is the sender's claim (`unknown` if none), because DomainForge has no world-level diff.
  - An allowed move is derived from its allow decision in the ledger (no second fact to drift) and read back as a record that validates against cep's `world-transition.v1`.
- **Cognate**: `requestWorldTransition` and `listTransitions`; Cognate never picks a world (record: cognate `.agents/evidence/2026-10-05-stage10-transitions.md`).

## Findings
- A world's semantic closure includes its logical file URIs, so moving or renaming a file is a semantic change and always needs approval. My first socket test used different file names for a "comment-only" edit and the closure differed.
- Existing rules already keep pending approvals in their world; a transition does not carry them across.

## Verification
- sea-rs: `just test` 1195 passed, 0 failed (1180 before; +15: 14 authority tests in `cep_transition.rs`, 1 socket conformance test); `just check` green.
- cep's validator accepts the recorded record (`CEP_REPO` gate, run).
- Cognate: `just verify` 78/78; `just sea-forge-live` 33 pass, 0 fail, against the real server with DomainForge-derived world refs.

## Debt
M-45 to M-47 in `.agents/DEBT.md`.

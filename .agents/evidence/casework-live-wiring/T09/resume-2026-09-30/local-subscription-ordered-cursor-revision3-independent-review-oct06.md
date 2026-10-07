# Local ordered-cursor proposal revision 3 — independent review

Date: 2026-10-06  
Verdict: **REJECT for citation repair; semantics otherwise address the assigned omission and prior findings.** This is a document-only review; no source, tests, compiler, or runtime gates were changed or run.

## Frozen identities and scope

- Original architecture assignment: `local-subscription-ordered-cursor-proposal-original-assignment-oct06.md`, SHA-256 `6028c2e99831f67a5ab3b974e955fd9b1fc6d7e1ac7c012ef93944bcd409d4c8`.
- Revision 2 assignment: `local-subscription-ordered-cursor-revision2-original-assignment-oct06.md`, SHA-256 `6375957abaffb8c9b051133776869ab8d63b01405b6eb35cdfe7a56418ecabde`.
- Revision 2 proposal: `local-subscription-ordered-cursor-concrete-proposal-revision2-oct06.md`, SHA-256 `a46ece1d0feb86ca679f485c4cf32ee566bff531f7adcc8fa1d630503d248456`.
- Revision 2 independent review: `local-subscription-ordered-cursor-revision2-independent-review-oct06.md`, SHA-256 `2d5f0eb696cb5f959dcadee76e1d32a7771db22f5b9f2bceca69da4d42de9869`.
- Revision 3 assignment: `local-subscription-ordered-cursor-revision3-original-assignment-oct06.md`.
- Revision 3 proposal: `local-subscription-ordered-cursor-concrete-proposal-revision3-oct06.md`, SHA-256 `ab63cc8bd5b561a72419899eb694df203c66e1baf48d81fd3689d08bd5d8d43e`.

This review checks the complete revision 3 against both architecture assignments, revision 2 and its review, and actual UI/normative source files. It is not source or runtime approval.

## Revision 3 behavior review

The sole revision 2 finding is resolved. The port permits `sinceCursor: string | undefined` (`apps/godspeed-cognitive-ui/src/ports/contract.ts:204-216`); revision 3 now defines `undefined` as no supplied floor and uses the captured allocator frontier `F`, with no replay, no input error, and an unsubscribe closure (`revision3:65-72`). Its controlled-timer plan queues H+1 before subscribing with `undefined`, requires H+1 suppression, then later genuine H+2 delivery. It keeps the separate supplied-H case and requires that queued H+1 be suppressed there too (`:72,74-75,87-90`). This meets revision 3 assignment's distinct test obligations.

The four findings from the earlier rejection remain repaired in the complete proposal: candidate is compared with the *previous* frontier, prepared snapshot state commits with that cursor before publication; per-subscriber pending FIFO capacity is 64 with explicit 65th-event disposal/error, isolation and callback-failure behavior; the ten-digit sequence ceiling is described as a new local proposal rather than existing grammar; and malformed/unsafe/different-epoch/over-limit/future/leading-zero inputs and seed validation have defined behavior (`revision3:32-40,42-58,83-96,108-119`). The proposal also maintains the allocator per case, no epoch wrap, stable order, immutable prior snapshots and points, visible progress and settlement, snapshot-before-side-event ordering, exhaustion atomicity, a separate execution-observation exception, controlled timers, and retained-replay branch preservation (`:8-20,42-69,76-96,98-120`).

The retained-trajectory-head/ordinary-stream-frontier split is consistent with actual sources and the root-directed local behavior. `getSnapshotAt` resolves an exact retained snapshot, and `queryTemporalTrajectory` derives head from the final point (`apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:146-166`). The port defines event subscription but does not require every ordinary event cursor to resolve to a snapshot (`ports/contract.ts:204-216`). Normative §6.1 shows `execution_progress` advancing after a snapshot; §6.3 separately requires `execution_observation` to copy the latest real case cursor without advancing revision state (`.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`). The actual conformance fixture separates future-only and retained-replay paths (`apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-142`). Revision 3 neither changes replay behavior nor proposes a public frontier getter.

## Material source-anchor errors and correction

Although the design assertions above are supported by real source, several full source paths in revision 3 point to files that do not exist. The assignment requires source-backed review; a future implementer cannot follow these anchors as written, so the proposal needs a new corrected immutable revision before its source record is complete.

1. Revision 3 cites `apps/godspeed-cognitive-ui/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235` (`:10`). The actual normative file is `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`.
2. It cites `apps/godspeed-cognitive-ui/src/ports/types.ts` for cursor grammar and event types (`:35,89`). That file does not exist. The actual local cursor comparison/comment is `apps/godspeed-cognitive-ui/src/ports/project.ts:145-150`; the `StreamEventType`, `StreamEvent` and observation-event definitions are in `.agents/reports/interface-contracts/typescript/types.ts:426-448`, with the `<epoch>.<seq>` cursor field at `:104`. The decimal-regex conformance is `apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts:416` (with its cursor assertions at `:494,609,636`), not `src/ports/types.ts:104`. The `model/types.ts` file that does exist has an unrelated `Relationship.to` field at line 104, so it cannot substitute for the cited cursor grammar.
3. Revision 3 cites `apps/godspeed-cognitive-ui/src/ports/caseworkPortConformance.ts:109-142` (`:89`). That file does not exist. The actual fixture is `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-142`.

I also need to clarify the prior review record. The revision 2 independent review cited the actual normative report path and actual local/port sources, but referred to the conformance file by basename only (`local-subscription-ordered-cursor-revision2-independent-review-oct06.md:22`) and did not flag that the revision 2 proposal itself used unqualified or invalid `types.ts`/conformance anchors (`revision2 proposal:36,89,91`). Those prior review claims were independently checked against real source; however, the review failed to call out the proposal's citation defects. This revision review corrects the full source paths and does not rely on nonexistent paths. Preserve the prior review as written; do not silently amend or relabel it.

The source was directly checked at the corrected locations: cursor comparator in `ports/project.ts:145-150`; actual port signature in `ports/contract.ts:204-216`; normative progress/observation rules at `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`; event types in `.agents/reports/interface-contracts/typescript/types.ts:426-448`; and the actual future-only/replay branch at `adapters/conformance/caseworkPortConformance.ts:109-142`. No alternate path under `apps/godspeed-cognitive-ui/reports` or `src/ports/types.ts` / `src/ports/caseworkPortConformance.ts` was found.

## Verdict and bounded next step

**Reject for citation repair.** Revision 3 fixes the sole semantic omission and preserves all previously reviewed behavior. The unresolved issue is that its normative, cursor-type, and conformance source anchors are invalid. A fresh corrected proposal should update only those citations, keep the verified semantics and test matrix intact, and preserve all earlier records. This verdict grants no local source/test changes, runtime proof, public cursor behavior, or architecture release.

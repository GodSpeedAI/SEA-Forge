# Local ordered-cursor proposal revision 4 — independent review

Date: 2026-10-06  
Verdict: **ACCEPT as a complete local architecture proposal for separate root review.** This is document approval only. It does not release source changes, tests, or runtime behavior.

## Frozen inputs and review boundary

- Original architecture assignment: `local-subscription-ordered-cursor-proposal-original-assignment-oct06.md`, SHA-256 `6028c2e99831f67a5ab3b974e955fd9b1fc6d7e1ac7c012ef93944bcd409d4c8`.
- Revision 2 assignment: `local-subscription-ordered-cursor-revision2-original-assignment-oct06.md`, SHA-256 `6375957abaffb8c9b051133776869ab8d63b01405b6eb35cdfe7a56418ecabde`.
- Revision 3 assignment: `local-subscription-ordered-cursor-revision3-original-assignment-oct06.md`, SHA-256 `e7471cdfa3b8326410674ce97fa146e1b8e10a098b814861029c831ea9d54aa9`.
- Revision 4 assignment: `local-subscription-ordered-cursor-revision4-original-assignment-oct06.md`, SHA-256 `0af4db3bfee9d17621eb8f6a1e1f8ebb18c11b456b2f6140accb2a290b31d7b8`.
- Revision 2 proposal and review: SHA-256 `a46ece1d0feb86ca679f485c4cf32ee566bff531f7adcc8fa1d630503d248456` and `2d5f0eb696cb5f959dcadee76e1d32a7771db22f5b9f2bceca69da4d42de9869`.
- Revision 3 proposal and review: SHA-256 `ab63cc8bd5b561a72419899eb694df203c66e1baf48d81fd3689d08bd5d8d43e` and `c3f84f279c9a14fd3bc5ac754689d99607296d4d44579f561407ab732b7770b3`.
- Revision 4 proposal: `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md`, SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`.

I compared the complete revision 4 with every original assignment and the prior independent reviews, then verified the cited paths and line claims against actual source/spec files. Relative shorthand in the proposal (for example `localAdapter.ts` or `src/ports/project.ts`) resolves to the full UI source path named in the surrounding section. No invalid source path remains.

## Revision 2/3 findings and assigned repair

All four original findings remain resolved: (1) candidate cursor is checked against previous frontier before advancing, and the same cursor is committed and published after snapshot preparation; (2) each subscriber has a 64-pending-event FIFO with explicit overflow disposal and exactly-once error behavior; (3) the ten-digit sequence ceiling is correctly identified as a new local proposal rather than existing grammar; and (4) malformed, unsafe, over-limit, other-epoch, future, and leading-zero inputs plus seed validation have defined behavior (`revision4:29-53,55-69,84-97`). The only revision 2 omission, `sinceCursor === undefined`, is fixed: it means no supplied floor, so registration uses captured `F`, does not replay or signal an input error, and returns the normal unsubscribe closure (`:71-78`). Its test explicitly suppresses queued H+1 and delivers a later genuine H+2 (`:85-91`).

The proposal keeps this omitted-boundary case separate from the supplied-H case, and keeps invalid input separate from both. It maintains local per-case ordering, previous-frontier allocation, preparation-before-mutation, no cursor reuse/wrap, immutable revisions, exact `getSnapshotAt`, healthy progress and settlement delivery, and terminal side-event ordering. Callback throws, `onError` throws, timer scheduling failure, reentrant enqueue, self-disposal, and queue overflow are addressed; errors do not roll back a committed event cursor (`:43-69,93-108`). Existing visible progress/settlement assertions remain required (`:117-119`).

## Current conformance behavior is accurately distinguished from the proposal

The actual shared conformance file is `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts`. At line 109 it captures `headCursorBeforeResume`, but at line 113 subscribes with `snapshot.cursor`; that call is before the replay/future-only branch at lines 118-139. Revision 4 explicitly states this current behavior and says the future-only branch still needs a separate branch-only adjustment to pass the captured head, while replay retains its original snapshot/resume cursor (`revision4:79-83,91-92`). The existing assertion only checks received events against captured head (`caseworkPortConformance.ts:134-139`); the proposal correctly requires additional controlled-timer tests proving suppression of pre-subscription queued H+1 for both omitted and supplied boundaries. This is not an assertion that conformance has already been fixed.

The retained revision head / ordinary stream frontier distinction remains well-grounded and bounded to the local simulation. Exact `getSnapshotAt` and retained trajectory head behavior are at `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts:146-166`; the port signature imposes no promise that each event cursor resolves to a snapshot (`src/ports/contract.ts:204-216`). Normative §6.1 and §6.3 describe distinct progress and informational-observation cursor behavior (`.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`). The proposal does not change the public frontier or observation rules.

## Citation corrections and prior-review record

Revision 3 was rejected for three citation errors, not for its repaired semantics. Revision 4 corrects each with actual sources:

1. Normative spec is `.agents/reports/interface-contracts/04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:195-235`, not a path under `apps/godspeed-cognitive-ui/reports`.
2. Canonical TS snapshot cursor field/comment and stream event definitions are `.agents/reports/interface-contracts/typescript/types.ts:101-110,426-448`. The executable regular expression is test code at `apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts:416`, with assertions at `:489-495,604-624`; it is not a production subscription parser. Numeric cursor comparison is `apps/godspeed-cognitive-ui/src/ports/project.ts:145-150`, which converts to numbers but does not validate syntax or safe-integer range. The named `apps/godspeed-cognitive-ui/src/ports/types.ts` does not exist.
3. Conformance is `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:109-150`, not `src/ports/caseworkPortConformance.ts`.

The previous revision 2 review used real port/spec/local source paths and its conformance basename could be resolved, but it did not flag the underlying proposal's bad/unqualified type and conformance citations. The revision 3 review recorded that omission and corrected the paths. Revision 4 accurately preserves that review history and does not silently rewrite prior records (`revision4:120-135`). The exact source identity of the actual conformance branch was rechecked here rather than inferred from earlier review text.

Other cited claims were also verified: adapter state and padding at `localAdapter.ts:52-99`; subscription and snapshot lookup at `:140-173`; template-created case at `:276-343`; execution/progress/settlement at `:347-399`; append and asynchronous emit at `:401-435`; and visible progress/settlement tests at `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts:162-192`. Each cited file exists and each referenced span supports the statement made.

## Deviations, limits, and verdict

The only material revision 4 changes relative to revision 3 are corrected full source paths and explicit distinction between current conformance and its proposed future-only change. It preserves the undefined-floor repair, separate H/H+1 test, four earlier findings, and all assigned behavior/test obligations. Its operational sequence ceiling and queue limit are still proposals only; they are not schema constraints, implemented behavior, or process-wide memory limits. It accurately leaves catastrophic native allocation failure unrecoverable and identifies the optional `onError` fallback/callsite compatibility as something to verify if implementation is later authorized.

**Accept as a complete local design proposal for root's separate architecture decision.** No source, conformance fixture, public interface, normative spec, or generated file was changed. No tests, compiler, scanner, gate, or Git operation was run. This verdict is not implementation authorization, actual conformance approval, source approval, or runtime proof.

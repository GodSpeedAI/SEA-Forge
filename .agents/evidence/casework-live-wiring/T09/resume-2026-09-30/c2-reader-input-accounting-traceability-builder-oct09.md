# C2 reader input-accounting traceability update

Date: 2026-10-09. This is a narrowly scoped normative traceability change. It
does not authorize Rust implementation, schema changes, tests, runtime work,
readiness, or T09 settlement.

## Full original recon grant

> Read-only tightly scoped recon in /home/sprime01/projects/sea-rs. No
> edits/compiling/testing/git mutation. Read applicable AGENTS and graft skill;
> use Graft first. Determine smallest implementable NEXT bounded-ledger-reader
> slice after private Next GREEN (currently another builder editing only 3 Go
> Next files; avoid inspecting moving files). Active plan
> .agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml T09,
> governing spec .agents/specs/godspeed.casework-cognitive-environment-spec.yaml,
> evidence base .agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-reader-registration-input-accounting-root-decisions-oct09.md + independent
> review and latest bounded-reader proposal/reviews/approvals. Identify exact
> Rust source/test scopes, existing decoder/lock/signer facilities, required
> normative traceability gaps, and proposed bounded builder specification with
> named tests and blockers. Do not invent policy, dependencies or public
> contract changes. Return concise paths+source anchors and outstanding approved
> vs unapproved distinctions. Runtime implementation is HELD until root
> explicitly releases after private Next checkpoint.

## Full normative traceability grant

> Good. Prepare bounded normative TRACEABILITY ONLY doc change now, runtime held.
> Read governing .agents instructions and exact current specs/ADR approval refs.
> Allowed edits ONLY .agents/specs/casework-live-cursor-v4-spec.yaml and parent
> .agents/specs/godspeed.casework-cognitive-environment-spec.yaml as strictly
> necessary to reflect ALREADY APPROVED Oct09
> registration/input-accounting-root-decisions-oct09.md plus independentreview.
> Add explicit every actual raw byte including
> repeated head/pin/predecessor reads charged each time to same4MiB page budget;
> metadata/seek not raw; cap before alloc/read; conformance vectors combined
> auxiliary+forward exactcap/capplusone. Preserve all existing
> caps/policies/public schemas and implementation_status held. For 'status:
> review', do NOT change lifecycle status blindly: inspect local allowed
> lifecycle+independent acceptance; report evidence/recommendation if status
> change requires additional authority. Do not run gates/test/compile/commit or
> edit otherdocs/status/debt/source. Persistent writes nativeapply_patch only.
> New immutable record under BASE
> c2-reader-input-accounting-traceability-builder-oct09.md containing full
> original grant/reconrefs, actual diffscope/hashes, deviations, policy
> approvalsupport and statusrecommendation. Root will review then assign
> independent critic. Ask root if anything extends approved clarification. No
> new public interface or architecture boundary.

## Recon and authority references

- T09 plan: `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml`.
- Parent and supplemental specs: `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml` and `.agents/specs/casework-live-cursor-v4-spec.yaml`.
- Decision: `docs/decisions/ADR-008-casework-live-cursor-v4.md`.
- Approved reader design: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-bounded-ledger-reader-proposal-revision4-independent-review-oct08.md` (proposal only), based on `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-bounded-ledger-reader-design-proposal-revision2-oct08.md` plus its recorded corrections/addenda.
- Operator policy receipt: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-bounded-reader-additional-policy-operator-approval-oct08.md` (policy approval, not implementation authorization).
- Oct. 9 architecture clarification and independent review: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-reader-registration-input-accounting-root-decisions-oct09.md` and `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-reader-registration-input-accounting-independent-review-oct09.md` (clarification-only approval; review requires normative traceability before the future reader grant).
- The independent normative-package re-review `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/c2-bounded-reader-normative-repair-independent-review-oct08.md` approved the then-current spec hash `4bb9730c25e582be159a35c5febe0f09f8af8ef4c17b59683599d147ff5e1b67` and ADR hash `c66912baee08317e4819ad211a208f5e2abff5d3369053a743a0f384737cc4e5`. Its approval predates this Oct. 9 traceability addition.

The operator-approved page input ceiling remains 4 MiB, with the existing 2 MiB
row, 500 scanned-row, 500 emitted-frame, 1 MiB complete serialized-response,
and 50 ms lock-wait limits unchanged. The Oct. 9 root clarification resolves
how bytes are charged inside that existing page ceiling; it creates no second
allowance, new interface, dependency, or authority boundary.

## Exact change scope and identities

Only `.agents/specs/casework-live-cursor-v4-spec.yaml` was changed. The parent
spec and ADR were read and left unchanged: the parent delegates C2 V4 details to
the supplement and already retains `status: review`; the ADR already describes
the approved 4 MiB page cap, and this change makes no architecture decision.

| Artifact | Identity |
|---|---|
| Supplemental spec before this edit, per prior independent review | `4bb9730c25e582be159a35c5febe0f09f8af8ef4c17b59683599d147ff5e1b67` |
| Supplemental spec after this edit, measured with `sha256sum` | `805f50bfe39e2c3aded66e0963c23de1d26033c4cd6b4acbbbf0160788b2b561` |
| Parent spec, unchanged, measured with `sha256sum` | `dc9ea678bab7e24d02072beb46d4079ac3181a790c91f1146919e4bb4624601c` |
| ADR-008, unchanged, measured with `sha256sum` | `c66912baee08317e4819ad211a208f5e2abff5d3369053a743a0f384737cc4e5` |

The supplement now cites the Oct. 9 clarification and its independent review,
and states in `REQ-C2-RANGE-004` that the single 4 MiB page budget charges every
actual raw ledger byte read: head/tail pinning, continuation-predecessor
revalidation, forward scan, non-events, and lookahead. Repeated reads count
again; metadata checks and seeks do not count. Read requests and row-buffer
growth are limited by the remaining budget before read/allocation. Exhaustion
cannot acknowledge partial rows, undelivered lookahead, or incomplete
validation.

`V-C2-RANGE-03` now requires combined auxiliary-plus-forward exact-cap and
cap-plus-one vectors, charging repeated reads again and excluding metadata and
seeks. Exact-cap progress may acknowledge only complete validated rows;
cap-plus-one may read no bytes beyond the remaining budget and may return only
previously acknowledged progress or typed unavailable. A failed page, partial
row, or undelivered lookahead is never acknowledged.

No policy, limit, public schema, status field, implementation status, or
architecture boundary was otherwise changed. No tests, gates, compiler,
formatter, or Git mutation were run. The source/runtime implementation hold is
preserved.

## Status recommendation and deviations

The C2 supplement remains `status: review`, its implementation status remains
`held`, and the parent continues to refer to the supplement as `review`. The
local specification template allows `draft | review | approved | deprecated`;
however, the earlier independent approval reviewed a different spec hash, and
the current root instruction calls for root review followed by an independent
critic on this changed spec. Keep `review` until that critic accepts the
updated normative artifact. No lifecycle-status authority is inferred from the
approval of the Oct. 9 clarification.

No deviations from the scoped grant. The parent spec was not edited because it
does not restate the C2 raw-page accounting rule; duplicating the new rule there
would be unnecessary and the existing parent-to-supplement status reference
already remains accurate. This record does not claim independent acceptance of
the changed spec or authorize reader implementation.

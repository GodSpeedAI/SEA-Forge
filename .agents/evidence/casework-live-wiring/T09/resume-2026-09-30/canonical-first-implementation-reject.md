# T09 canonical first implementation review — REJECT

This is the immutable result of the first independent source review against
`canonical-implementation-assignment.md`, `test-first-builder-assignments.md`, the approved
proposal, source preparation, and fixture review. **REJECT; repair is required before GREEN.**
No compiler token was used and no build/test/compile/runtime command was run. The original
fixture files were not edited. Expanded audit and requirement-by-requirement source anchors
are preserved in `canonical-implementation-source-review.md`.

## Blocking findings for this implementation

1. **Ask schema root is unconstrained.** At
   `.agents/reports/interface-contracts/schemas/thoth-ask.schema.json:1-7`, root asserts only
   `type: object` and declares `definitions`; it never references `AskRequest` or
   `ThothAnswerView`. Thus `{}` and arbitrary objects validate against the root schema.
   Connect the root to both payload shapes (e.g. `oneOf` refs) and assert that connection in
   the focused contract test.
2. **Observation schema omits several expressible state conditions.** At
   `schemas/event-stream.schema.json:70-87`, schema documentation says truncated iff omitted
   is positive, but no assertion encodes it. At `:117-152`, an unavailable run list may still
   contain runs; `no_runs` permits a nonzero `reads_attempted`; and `complete` permits a
   zero `listed_run_count`. Constrain these cases in JSON Schema and pin them in tests.

## Limits and non-findings

Draft 2020-12 cannot express the cross-property arithmetic
`total_frame_count = retained_frame_count + omitted_frame_count`, or dynamically equate
`frames.length` to `retained_frame_count`, without extensions. Those must stay documented and
be checked by runtime validators/tests; this review does not require a dependency or claim that
the schema alone proves count truth. Likewise, the new SSE JSONL row is only an illustrative
envelope; it does not prove native no-`id:` framing or cursor reuse. Its `...0416` cursor is not
identified as fabricated because its case ID differs from preceding illustrative rows.

## Assignment coverage and material difference

The source does implement the specified Ask field/literal sets and complete disclosure DTOs;
the cohort/run/frame DTOs, event literal and enum parity, separate run execution/settlement
standings, bounded array/read values, allowlisted frame fields, command-finished-only optional
metadata, and normative runtime-pending limits/exclusions. Detailed evidence and actual Rust
view anchors are in the expanded audit file cited above.

The added `ThothNarrationResultMetadata` (`typescript/types.ts:540-543`) is detached from the
actual `NarrationBeat` port (`apps/godspeed-cognitive-ui/src/ports/contract.ts:99-110`); Spec
04 documents the new field on `NarrationBeat`, but the helper is not a usable extension of
that owning type. This is recorded as a material scope limitation for the later narration/UI
unit, not the basis for rejecting this bounded contract unit.

Repair only the two authored schemas and focused conformance assertions per root's transfer.
Keep original fixtures and all unrelated mirrors unchanged. A fresh independent source review
and later focused GREEN plus explicit TypeScript pins remain required.

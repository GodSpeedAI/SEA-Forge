# T09 canonical implementation source review

## Verdict

**REJECT source implementation pending repair.** This is an independent, source-only
review against `canonical-implementation-assignment.md`, the original test-first assignment,
approved proposal, source preparation, fixture review, and current canonical sources. No
compiler token was used; no build, test, compile, or runtime command was run. The frozen
fixtures remain unchanged.

## Blocking findings

1. **The Thoth Ask schema does not constrain its root payload.**
   `schemas/thoth-ask.schema.json:1-7` sets only `type: object` and declares definitions.
   The root has no `$ref`, `oneOf`, `properties`, or other assertion connecting it to
   `#/definitions/AskRequest` or `#/definitions/ThothAnswerView`. A validator applying this
   schema to `{}` accepts it; arbitrary objects also pass. The route request and complete
   answer are therefore disconnected lookup definitions rather than a root contract. Link
   the schema root to both supported payload shapes (for example, a `oneOf` over those two
   definitions) and assert that linkage in the conformance test.

2. **Several expressible observation invariants are only descriptions or absent.**
   `event-stream.schema.json:70-87` describes `total_frame_count = retained + omitted` and
   `truncated iff omitted > 0`, but has no constraints implementing the latter boolean
   relation. Draft 2020-12 cannot express the arithmetic equality or dynamically compare
   `frames.length` with a count; those belong in runtime checks/tests and the existing
   fixture helper. The schema can and should still enforce the boolean equivalence.
   Additionally, `:117-152` permits all of these semantically contradictory shapes:

   - `run_list_state: unavailable` with a non-empty `runs` array (counts are correctly
     forbidden, but a failed initial list has no candidates to hydrate);
   - `observation_state: no_runs` with `hydration_read_budget.reads_attempted > 0`;
   - `observation_state: complete` with `listed_run_count: 0`, even though that is the
     `no_runs` state.

   These are expressible in the existing schema dialect. Add constraints and focused
   assertions. Cross-field arithmetic/count sums, dynamic equality to array lengths, and
   truth of kernel-derived values remain runtime responsibilities; no custom validator or
   dependency is requested.

## Requirement audit

- **Ask DTOs and literals:** `typescript/types.ts:473-538` defines all nine kernel question
  kinds, twelve claim classes, seven statuses, three dispositions, freshness values, the
  actor-free `{ kind, subject, purpose?, case? }` request, and full claim/answer disclosure
  fields. Assurance and authority notice remain strings. The schema definitions preserve
  optional `capability_record_ref`; actual `ClaimView` uses
  `skip_serializing_if = Option::is_none` in `crates/sea-forge-server/src/sfwp/thoth.rs:14-38`.
  Ask schema additionally documents the raw-body and UTF-8 purpose limits rather than
  pretending JSON Schema `maxLength` implements them.
- **Observation DTOs:** `types.ts:302-380` distinguishes the existing invocation
  `ExecutionObservation` from the cohort `RunTraceObservation` and nested
  `RunTraceRunObservation`; separates six execution and four settlement standings; makes
  run-list counts optional; and includes the eight-read budget. The stream enum includes
  `interrupted`, `error`, and `execution_observation`. The event schema links its literal
  observation arm to the typed payload at `event-stream.schema.json:37-40,154-166`, and
  includes bounded eight-run/eight-read and 1,024-frame shapes.
- **Safe frame constraints:** `RunTraceFrame` contains only event ID, kind, timestamp, and
  optional command metadata. The schema's condition at
  `event-stream.schema.json:64-68` rejects command status/exit code on every non-
  `command_finished` kind and limits command status to the five approved values. The TypeScript
  type and schema carry the same allowlisted ten frame kinds. The JSONL golden is an envelope
  example only; it proves neither native SSE no-`id:` framing nor cursor reuse.
- **Normative boundaries and budgets:** Spec 04 sections 6.3 and 8.1 and governing spec
  amendment `t09_additive_contract_amendment` document informational real case cursor/no SSE
  ID/no history mutation, hydration/poller/UI bounds, Ask auth/body/rate/disclosure
  requirements, excluded aggregate bounds, and pending runtime status. This matches the
  assignment's contract-only scope; no Go/UI mirrors or runtime behavior are claimed here.

## Material difference: narration metadata

The implementation adds `ThothNarrationResultMetadata` with an optional `grounded_answer`
(`types.ts:540-543`) because this canonical file has no `NarrationBeat`. This helper is
detached: the actual UI port owns `NarrationBeat` at
`apps/godspeed-cognitive-ui/src/ports/contract.ts:99-110`, and the new field is not attached
to that type or to a canonical narration result. Spec 04 documents `grounded_answer` on its
`NarrationBeat` interface. This is not a blocker for the bounded contract unit's Ask and
observation DTO requirements, but it does not constitute a usable narration type extension.
The later narration/UI unit must add the field to the actual owning port type and verify the
returned references and complete answer are preserved without invented claims, citations, or
directives. Do not represent this helper alone as completed narration integration.

The observation golden uses cursor `1.0000000416` after an illustrative `...0415` event,
but those rows have different case IDs and are independent examples. This review does not
infer a false cursor advancement from that ordering. A runtime/server framing test must prove
that an observation copies the latest real cursor and emits no SSE `id:`.

## Required next step

Fresh bounded builder repairs only the authored canonical implementation files and any
authorized source-review follow-up. Preserve the approved fixtures and existing UI/Go mirrors.
After a fresh independent source approval, root may transfer the compiler token for focused
GREEN and the explicit TypeScript pins; both remain unverified and pending.

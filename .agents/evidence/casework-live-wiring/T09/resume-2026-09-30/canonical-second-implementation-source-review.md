# T09 canonical second implementation source review

## Verdict

**APPROVE source only; GREEN remains pending.** This fresh independent review considered the
original `canonical-implementation-assignment.md`, `test-first-builder-assignments.md`,
approved proposal and source preparation, prior source rejection, and the frozen repair diff.
The repair is limited to `schemas/thoth-ask.schema.json`,
`schemas/event-stream.schema.json`, and focused assertions in
`tests/contract-conformance.test.ts`. It preserves the already approved fixtures and
implementation DTO/spec scope. I did not use the compiler token or run tests/builds/runtime.

## Rejection findings resolved

1. `thoth-ask.schema.json:6-11` now has a root `oneOf` referencing both
   `#/definitions/AskRequest` and `#/definitions/ThothAnswerView`. The shapes are disjoint:
   request requires `kind`/`subject`, answer requires its own required answer fields, and
   both disallow unknown fields. The focused contract test pins the exact root refs and the
   schema inventory.
2. `event-stream.schema.json` now constrains `truncated` to `true` when
   `omitted_frame_count >= 1`, and to `false` otherwise (`RunTraceRunObservation.allOf`).
   The test pins that condition. The required count field makes the conditional exhaustive.
3. An unavailable run list now requires an empty `runs` array while leaving all six count
   fields absent. The test pins both assertions.
4. `no_runs` now requires `reads_attempted: 0`, a complete list, zero counts, and no run
   entries. The focused assertion pins that condition.
5. `complete` now requires a complete list and `listed_run_count >= 1`, with zero error and
   omission counts. The focused assertion pins the positive lower bound.

These additions are valid Draft 2020-12 conditionals and fit the contract's stated states.
No custom validator or dependency was introduced.

## Original assignment coverage

- **Ask types:** `typescript/types.ts` carries the exact nine question kinds, twelve claim
  classes, seven claim statuses, actual disposition/freshness values, and actor-free request
  fields. `ThothAnswerView` and `ThothClaimView` retain every approved disclosure and returned
  reference. Assurance and authority notice are strings. The Ask schema definitions impose
  the same finite sets and complete fields, disallow extra keys, preserve optional
  `capability_record_ref`, and explicitly place the 8 KiB body and 500 UTF-8-byte purpose
  checks at the route boundary. The schema root now applies those definitions to either
  payload shape.
- **Trace DTO and safe frame:** `types.ts` distinguishes the existing invocation
  `ExecutionObservation`, cohort `RunTraceObservation`, and nested
  `RunTraceRunObservation`. Run execution and settlement remain separate six/four-value
  unions. Frames carry only actual ID/kind/time and optional command-finished status/exit
  code. The schema allows the five core command statuses only on `command_finished`, and
  rejects extra/raw fields. Existing `interrupted` and `error` are preserved in the exact
  stream enum alongside `execution_observation`; the event's literal type and payload are
  connected through the event-stream root condition.
- **Budgets and states:** the typed/schema cohort remains bounded at eight runs, eight initial
  reads, and 1,024 frames per run. Counts are optional in TypeScript and absent when the list
  is unavailable. Schema conditionals require count presence for a complete list, restrict
  `no_runs`, `capacity_limited`, `complete`, and `unavailable` state combinations, and now
  close the prior expressible gaps listed above.
- **Normative scope:** Spec 04 and the governing casework spec describe informational use of
  the latest real case cursor, no SSE `id:`/logical advancement, immutable history, protected
  Ask disclosures, poll/read/cache limits, and aggregate-bound exclusions. Both mark runtime
  behavior pending. They make no claim that these schemas prove native SSE framing or actual
  runtime count truth.

## Standard-schema/runtime boundary

Draft 2020-12 cannot express cross-property arithmetic equality (`total_frame_count =
retained_frame_count + omitted_frame_count`), dynamic equality between array length and a
count, or actual truth of kernel-derived counts. Existing focused sample checks cover count
truth for the golden/helper paths; later runtime tests remain responsible for real outputs.
This limitation is explicit in the prior immutable rejection record and is not a remaining
source blocker. The fixture/golden still does not prove that native SSE lacks an `id:` line or
that the event cursor equals the latest cursor at runtime; the later server framing/runtime
unit must prove those behaviors.

## Material deviation retained

The canonical file adds detached `ThothNarrationResultMetadata` with optional
`grounded_answer` (`typescript/types.ts:540-543`), while the actual current `NarrationBeat`
owner is `apps/godspeed-cognitive-ui/src/ports/contract.ts:99-110`. This helper is not itself
a usable extension of that port. Narration wiring is a later prepared unit, so this is a
documented integration limit rather than a blocker for this source unit. The later mirror/
narration unit must extend the actual owner and preserve complete Thoth references without
invented claims/citations/directives. No UI, Go, or production runtime file is part of this
repair.

## Pending evidence

This verdict approves source completeness only. The original focused Bun GREEN run and
explicit TypeScript CLI check of the `.agents` conformance test are not run or approved here.
Root must transfer the compile token after cap Go gates finish, perform the required RAM and
process preflight, serialize compiler work, retain the exact command outputs, and keep the
existing UI typecheck limitation visible.

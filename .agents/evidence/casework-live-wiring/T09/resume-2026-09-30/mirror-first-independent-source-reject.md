# T09 Go/UI mirror: first independent source rejection

Date: 2026-10-01
Result: **REJECT — source review only. No compilation or tests were run.**
Builder: `t09_contract_mirror_builder` (first implementation, frozen four-file source).
Compiler token: released to ROOT; no further compiler work by this critic.

## Scope and exact reviewed source

Reviewed against the original assignment at
[`mirror-implementation-assignment.md`](mirror-implementation-assignment.md), especially
its requirements to pin full field/type/optional count and literal parity (lines 20–22),
and attach grounded answer metadata to the `NarrationBeat` port (lines 23–25).

The four frozen owned files and SHA-256 identities at review:

| File | SHA-256 |
| --- | --- |
| `apps/godspeed-casework-go/internal/contract/contract.go` | `f0b51bfb61698e56faf16cf4a0216010e52e9c2f447f0ff673b78980964363c6` |
| `apps/godspeed-casework-go/internal/contract/golden_test.go` | `e60fe3a575c7f926129b842afe024cccb79238452f522ef51f0fb1d1605dba24` |
| `apps/godspeed-cognitive-ui/src/ports/contract.ts` | `042970baaa5dc131eb2ee2ab00a29a6fc5fcf62abffa1f11b6f5d9b253336bf4` |
| `apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts` | `7e66bb5a835bdc5ccfd333009e14c87a498b20825587b444d6b78ec93b7b1aac` |

These identities were produced by:

```text
sha256sum apps/godspeed-casework-go/internal/contract/contract.go apps/godspeed-casework-go/internal/contract/golden_test.go apps/godspeed-cognitive-ui/src/ports/contract.ts apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts
```

## Material findings

1. **Go field types are not fully pinned.** At `golden_test.go:273–303`,
   `TestCanonicalMirrorFieldSets` checks the JSON tag names and the `omitempty` set.
   At `golden_test.go:334–341`, the only reflected field-type assertion checks that the
   six cohort count fields are `*int`. There are no per-field `reflect.Type` assertions
   for the other Ask or observation fields. Therefore the test can miss an unintended
   Go representation change that preserves the JSON field inventory and optional tags,
   particularly on absent/empty fixture members. This does not satisfy the assignment's
   explicit full field/type/optional pin requirement.

   Evidence commands and source ranges:

   ```text
   nl -ba apps/godspeed-casework-go/internal/contract/golden_test.go | sed -n '270,370p'
   nl -ba apps/godspeed-casework-go/internal/contract/contract.go | sed -n '180,300p'
   ```

2. **UI runtime validation does not close the nested hydration-budget object.** At
   `wireContract.test.ts:432–438`, the test confirms the object exists and checks its three
   known values, but never checks its keys with `hasOnlyKeys`. Extra properties therefore
   pass UI fixture validation even though the surrounding event, cohort, nested run, and
   safe frame validators do exact-key checks. The Go observation decoder uses
   `DisallowUnknownFields`, so this nested shape currently has asymmetric validation.

   Evidence command and source range:

   ```text
   nl -ba apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts | sed -n '430,475p'
   ```

## Other material differences and covered scope

- The UI port imports and re-exports the canonical TypeScript DTOs directly
  (`contract.ts:15–40, 42–71`) rather than defining a second local DTO copy. This avoids
  duplicate type drift; the `wireContract.test.ts:247–310` type pins check canonical
  literals, selected property types, exact key/optionality inventories, and direct port
  re-export equality. `apps/godspeed-cognitive-ui/tsconfig.json` includes `src`, so these
  checks are in the configured typecheck graph. The selected UI property pins do not
  compensate for the missing Go per-field type pins above.
- The port adds `NarrationBeat.grounded_answer?: ThothAnswerView` at
  `contract.ts:133–140`; the UI test pins that property to the canonical answer view.
  Subsequent narration rendering remains outside this four-file unit as assigned.
- Source inspection found Ask request/claim/answer structures and the finite vocabulary
  arrays; separate six-value execution and four-value settlement standings; the five
  command statuses; nested run/cohort/list observation states; pointer-backed optional
  cohort counts; optional pointer-backed command status/exit metadata; and a typed named
  observation event. The observation event is decoded as `RunTraceObservationEvent` with
  `DisallowUnknownFields`, not `json.RawMessage`. The existing stream arms and fields are
  retained, with the new event added. All four Ask goldens are included in the Go round-trip
  dispatch and UI fixture checks. No Ask operation method or unrelated runtime rendering
  was introduced.

## Commands observed

```text
git diff --check -- apps/godspeed-casework-go/internal/contract/contract.go apps/godspeed-casework-go/internal/contract/golden_test.go apps/godspeed-cognitive-ui/src/ports/contract.ts apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts
```

Result: exit 0, no output.

```text
gofmt -d apps/godspeed-casework-go/internal/contract/contract.go apps/godspeed-casework-go/internal/contract/golden_test.go
```

Result: exit 0, no output.

No compiler, typecheck, or test command was run for this review. Source rejection makes
implementation GREEN claims unsupported; the frozen source should go to a fresh builder.

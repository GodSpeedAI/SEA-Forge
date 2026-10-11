# T01 evidence summary — single canonical UI<->gateway wire contract (GAP-A)

Date: 2026-09-23. Branch: `casework/live-wiring`. Not committed (per task instruction);
`git status` shows the T01 file set below plus the pre-existing parallel rust_kernel work
(T02-T04 track), which T01 did not touch.

## Gates (all green; logs in ./gates/)

| Gate | Command | Exit |
|---|---|---|
| G1 | `cd apps/godspeed-casework-go && go vet ./... && go test -race -count=1 ./...` | 0 |
| G2 (task gate) | `cd apps/godspeed-casework-go && go test -race -count=1 ./internal/contract/...` | 0 |
| G3 | `cd apps/godspeed-cognitive-ui && bun run typecheck && bun test` | 0 (195 pass / 0 fail) |
| G4 (build guard) | `cd apps/godspeed-cognitive-ui && bun run build` | 0 (dist builds) |

`-count=1` on the Go contract tests is deliberate: Go's test cache keys on package sources, not
on runtime-read files, so a fixture-only change could otherwise be masked by a cached green
(demonstrated live during the teeth; see teeth/tooth-2b-followed-through.log).

**Skipped:** `bun e2e/run.ts` (local ladder). It is deterministically RED at baseline (J1,
decision-log B-1 / OBSERVED_DEBT); T01 is type-only and additive to the UI, and T09 owns
restoring the ladder. Not a T01 regression.

## Teeth (transcripts in ./teeth/)

1. `tooth-1-visible-objects-rename.log` — renamed the Go `visible_objects` json tag to
   `objects`: `TestGoldenRoundTrip` FAILED (exit 1). Reverted; green again.
2. `tooth-2a-ts-union-only.log` — added `BOGUS_KIND` to the TS `ConsequentialIntentName` union
   only: `bun run typecheck` FAILED (exit 2, `WireContractPins`); runtime `bun test` still
   passed, honestly noted (a union-only addition has no fixture to detect at runtime).
3. `tooth-2b-followed-through.log` — followed `BOGUS_KIND` fully through the TS side (all three
   unions, both hardcoded exhaustive lists, golden fixture): TS typecheck+test PASS (exit 0),
   and the Go `TestGoldenCoversEveryIntentKind` FAILED (exit 1) until the Go side agrees.
   Reverted; no `BOGUS_KIND` remains anywhere outside the transcripts.

## Kebab dialect retirement

`kebab-import-search.txt`: repo-wide `rg` for importers of
`apps/godspeed-cognitive-ui/contracts/*.ts` found ZERO code importers (only doc/config
mentions). The directory was deleted via `git rm` (6 files) and dropped from tsconfig include.
`reference/WIRE.md` still documents the still-live Go fixture wire (projection.go tags); it is
superseded by the ADR and is T06's to replace.

## Files (T01 only)

- `.agents/reports/interface-contracts/typescript/types.ts` — additive: 6 new intent kinds in
  two unions, `ConsequentialIntentName`, typed request payloads + `IntentRequestPayloads`,
  `IntentRefusalKind`/`IntentRefusal` (+ optional `refusal` on `IntentResponse`),
  template/preflight DTOs, `interrupted`/`error` SSE kinds + payloads.
- `.agents/reports/interface-contracts/golden/` — 15 canonical fixtures (world snapshot, 10
  intent request/response pairs covering all 7 refusal kinds, template entry options, preflight
  pass/fail, 9-line SSE jsonl).
- `apps/godspeed-casework-go/internal/contract/` — NEW stdlib-only package: contract structs
  (json tags = TS field names), exhaustive kind lists, golden round-trip + exhaustive-kind
  tests, `GOLDEN_UPDATE=1` canonicalization affordance.
- `apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts` — NEW: golden imports typechecked
  against the canonical types, runtime structural validators, compile-time exhaustive-kind
  pins (`WireContractPins`), set-equality against fixtures.
- `apps/godspeed-cognitive-ui/tsconfig.json` — removed the retired `contracts` include entry.
- `apps/godspeed-cognitive-ui/contracts/` — DELETED (git rm).
- `.agents/reports/casework-live-wiring/adr-wire-contract.md` — the ADR (D-1, ships/retires,
  drift prevention, Go vocabulary mapping note).

# ADR: One canonical UI<->gateway wire contract (T01, resolves GAP-A)

- **Status:** accepted 2026-09-23 (implements operator decision D-1, approved 2026-09-23; see
  `.agents/reports/casework-live-wiring/decision-log.yaml`)
- **Plan:** `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml` T01
- **Scope:** UI<->gateway boundary only. The SFWP (gateway<->kernel) wire is ADR-003 territory
  and is not changed by this ADR.

## Decision

Exactly one contract ships: the **spec-04 CognitiveWorldSnapshot package** at
`.agents/reports/interface-contracts/typescript/types.ts`, extended additively with the journey
vocabulary below. The UI port (`apps/godspeed-cognitive-ui/src/ports/contract.ts`) already
consumes it. The Go gateway grows a mirror of it in
`apps/godspeed-casework-go/internal/contract/` (stdlib-only Go structs whose JSON tags match the
TypeScript field names exactly).

The competing kebab-case dialect `apps/godspeed-cognitive-ui/contracts/*.ts` is **retired by
deletion** (verified first: zero importers anywhere in the repo — the only `contracts/`-shaped
import in the UI is the canonical `interface-contracts` import in `src/ports/contract.ts`; see
evidence `T01/kebab-import-search.txt`). It is deleted rather than aliased so it cannot drift
back. `tsconfig.json` no longer includes the directory.

## What ships (all additive on types.ts; no existing shape reshaped)

1. **Journey intent kinds.** `ActionIntentKind` and `InteractionActionName` gain `PROPOSE_CASE`,
   `EXECUTE_ITEM`, `COMPLETE_HUMAN_TASK`, `REOPEN_CASE`, `TERMINATE_CASE` (and
   `ADD_DISCRETIONARY_WORK` joins `InteractionActionName`; it already existed in
   `ActionIntentKind`). `APPROVE_HUMAN_TASK`, `REJECT_HUMAN_TASK`, `ESCALATE_OR_OVERRIDE` and
   `OPEN_ARTIFACT` already existed and were not duplicated. Legacy members (`BEGIN_WORK`,
   `REOPEN_WORK`, `RESOLVE_SOURCE`, `EXPORT_AUDIT_BUNDLE`, `RESUME_LIVE_STREAM`, the read-only
   interaction names) remain untouched. `ConsequentialIntentName` names the 10-kind consequential
   journey set (the plan's T01 list).
2. **Typed request payloads** riding in `InteractionIntent.parameters`, keyed by action name
   (`IntentRequestPayloads`): `PROPOSE_CASE` carries `template_ref`/`params`/`preflight_digest`;
   `ADD_DISCRETIONARY_WORK` carries `case_id`/`stage_id`(+optional `anchor_item_id`)/`kind`/
   `title`(/optional `summary`)/`justification`; `EXECUTE_ITEM` carries `item_id`;
   `COMPLETE_HUMAN_TASK` carries `item_id`/`result`/mandatory `justification`;
   `REOPEN_CASE` and `TERMINATE_CASE` carry `case_id`/`reason` (`CaseLifecyclePayload`). Intents
   without a typed payload carry envelope fields only (`target_object_id`, `case_id`,
   `justification`).
3. **Typed refusal union** `IntentRefusalKind`: `AUTHORITY_DENIED`, `UNAUTHORIZED_ROLE`,
   `SOD_VIOLATION`, `STALE_PROJECTION`, `JUSTIFICATION_REQUIRED`, `UNAVAILABLE`, `INVALID` —
   carried as a new optional `refusal` envelope on `IntentResponse` (`refusal_kind`, `message`,
   optional `current_cursor` for `STALE_PROJECTION`). The legacy `error_code`/`error_message`
   fields stay (additive; the UI's local adapter still uses them); refusals use the typed
   envelope. A refusal is an outcome (HTTP 200), not a transport error, same rule as the
   coordinator today.
4. **Template + preflight DTOs** (gateway-facing, same file): `TemplateEntryOption`
   (`template_ref`, `title`/`description`, `parameters` of `TemplateParameter`: name/title/
   description/type `string|number|boolean|enum`/required/default_value/options) — what
   `case.entry_options` / `GET /api/templates` returns; and `TemplatePreflightResult`
   (`template_ref`, `params`, `passed`, `reasons` empty exactly when passed, `digest` present
   exactly when passed — `PROPOSE_CASE` must echo it).
5. **SSE vocabulary.** `StreamEventType` gains `interrupted` (connection/interruption signal,
   payload `InterruptedPayload`: `reason`, optional `last_cursor` for resume) and `error`
   (stream failure, payload `ErrorPayload`: `error_code`, `message`; spec-04 §6.2's
   `gap_exceeded`). `snapshot`/`patch` remain the revision events, keyed by the kernel cursor
   (`cursor` is `"<epoch>.<seq>"`, doubles as SSE `Last-Event-ID`/`?last=`), alongside the
   existing `execution_progress`, `settlement_recorded`, `lease_expired`, `resync_required`,
   `heartbeat`.

## What is retired

- `apps/godspeed-cognitive-ui/contracts/` (agent.ts, artifact.ts, index.ts, interaction.ts,
  temporal.ts, world.ts) — deleted via `git rm` (this change). Its vocabulary
  (`focus-object`, `decide-approval`, `authority_denied`, `status: accepted/refused`, ...) does
  not survive on the UI boundary.
- **Mapping note (Go side).** The Go gateway's current fixture-stage vocabulary coincided with
  that dialect: `internal/coordinator/coordinator.go` accepted kebab-case kinds
  (`focus-object`, `decide-approval`, `propose-consequence`, `persist-artifact`, ...) with
  lowercase refusal reasons (`invalid`, `stale_projection`, `authority_denied`, `unavailable`)
  in an `Outcome{status, reason, note, lease}` envelope, and `internal/projection/projection.go`
  served the Northstar fixture shape (`cursor` int64, `surfaces`/`objects`/`relationships`,
  camelCase tags per `reference/WIRE.md`). **T06 moves the gateway to the spec-04 names**: the
  `internal/contract` structs become the served world/intent/SSE shapes, intent kinds become the
  SCREAMING_SNAKE consequential set, refusals become the typed `refusal_kind` envelope, and the
  projection cursor becomes the kernel `"<epoch>.<seq>"` string. Until T06 lands, the fixture
  packages keep their current shapes for their existing tests and MUST NOT be treated as the
  contract; `reference/WIRE.md` documents that fixture-stage wire and is superseded by this ADR.
  Vocabulary mapping (retired -> canonical): `focus-object`/`resolve-object` ->
  `SELECT_OBJECT`-class reads (local) or targeted intents; `inspect-artifact` -> `OPEN_ARTIFACT`;
  `request-explanation` -> narration port (spec-04 §8); `propose-consequence` (action
  `implement`) -> `EXECUTE_ITEM`; `decide-approval` -> `APPROVE_HUMAN_TASK`/`REJECT_HUMAN_TASK`;
  `persist-artifact` -> kernel artifact capture (T04), not a UI intent; `invalid` -> `INVALID`,
  `stale_projection` -> `STALE_PROJECTION`, `authority_denied` -> `AUTHORITY_DENIED` (or the
  finer `UNAUTHORIZED_ROLE`/`SOD_VIOLATION`), `unavailable` -> `UNAVAILABLE`; the coordinator's
  lease stages become SSE `execution_progress`/`settlement_recorded` events on the kernel cursor.

## How drift is prevented

Golden fixtures under `.agents/reports/interface-contracts/golden/` are the shared source of
truth, exercised by BOTH sides:

- `world-snapshot.json` (one canonical CognitiveWorldSnapshot), one `intent-<kind>.json` per
  consequential kind (request + success response + at least one meaningful refusal example;
  together they exercise all 7 refusal kinds), `templates-entry-options.json`,
  `template-preflight-pass.json` / `-fail.json`, and `sse-events.jsonl` (one line per stream
  event kind). Files are canonical JSON (Go struct field order; map keys sorted), generated and
  byte-pinned by the Go test (`GOLDEN_UPDATE=1 go test ./internal/contract/` rewrites them after
  an intentional, two-sided change only).
- **Go side** (`apps/godspeed-casework-go/internal/contract/golden_test.go`):
  `TestGoldenRoundTrip` reads EVERY file under golden/ (unknown files fail), unmarshals into the
  matching struct, re-marshals canonically and requires byte-stable round-trips (a json-tag
  rename, e.g. `visible_objects` -> `objects`, fails immediately); `TestGoldenCoversEveryIntentKind`
  / `TestGoldenCoversEveryRefusalKind` / `TestStreamEventKindsExhaustive` pin set-equality
  between the goldens and the exhaustive kind lists, so a kind added or removed on only one side
  fails; payload-carrying fixtures must decode into the typed payload structs. Run this package
  with `-count=1` after fixture-only changes: Go's test cache keys on package sources, not on
  files read at runtime (a cached green can otherwise mask a fixture-only edit; proven during
  the T01 teeth and recorded in the transcripts).
- **TS side** (`apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts`, bun test +
  `tsc --noEmit`): the golden JSONs are imported (so they typecheck) and validated against the
  canonical shapes field by field; hardcoded exhaustive kind lists are pinned to the types.ts
  unions at compile time (`WireContractPins`), and set-equality against the goldens is asserted
  at runtime. A union member added in TS only fails `bun run typecheck`; once its list and
  fixture follow, the Go exhaustive-kind test still fails until the Go side agrees.

Drift chain, end to end: change a union or a field name on either side -> typecheck (TS) or
byte-stability (Go) fails -> fixtures cannot be updated one-sidedly without the other side's
test failing.

## Consequences

- The gateway's future endpoints (`/api/world`, `/api/intents`, `/api/templates`,
  `/api/templates/preflight`, `/api/events`) serve these shapes (T06/T07/T08).
- `GOLDEN_UPDATE=1` is the only sanctioned way to rewrite fixtures, after the TS and Go types
  have both been updated. It is an intentional-contract-change affordance, not a way to make a
  one-sided change pass.
- No new dependencies anywhere (Go package is stdlib-only; TS uses only bun:test, node:fs,
  node:path).

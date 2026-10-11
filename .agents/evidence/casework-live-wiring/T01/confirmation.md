# T01 independent confirmation — single canonical UI<->gateway wire contract (GAP-A)

- **Verdict: APPROVE**
- Independent verifier, 2026-09-23, branch `casework/live-wiring`. All evidence below was
  personally observed by re-running every gate, both teeth, and targeted probes. No file was
  modified by the verifier except this note (all tooth/probe edits were reverted; final tree
  re-verified green).
- Scope honored: crates/* churn (T04A's in-flight work) was excluded from judgment.

## Gates re-run by the verifier (commands + exit codes)

| # | Command (cwd) | Result |
|---|---|---|
| G1a | `go vet ./...` (apps/godspeed-casework-go) | exit 0 |
| G1b | `go test -race -count=1 ./...` (apps/godspeed-casework-go) | exit 0 (see finding 9 for one intermittent pre-existing flake) |
| G2 | `go test -race -count=1 ./internal/contract/...` (task gate) | exit 0 — TestGoldenRoundTrip, TestGoldenCoversEveryIntentKind, TestGoldenCoversEveryRefusalKind, TestStreamEventKindsExhaustive, TestKindListsAreWellFormed all PASS |
| G3a | `bun run typecheck` (apps/godspeed-cognitive-ui) | exit 0 |
| G3b | `bun test` (apps/godspeed-cognitive-ui) | exit 0 — **195 pass / 0 fail** across 14 files (baseline 185 + the 10 new `src/ports/wireContract.test.ts` tests, confirmed by running that file alone: 10 pass) |
| G4 | `bun run build` (apps/godspeed-cognitive-ui) | exit 0 |

Skipped by design: `bun e2e/run.ts` — verified honestly recorded as deterministically red at
baseline J1 in `.agents/OBSERVED_DEBT.md:1005-1019` and
`.agents/reports/casework-live-wiring/decision-log.yaml:121`; T01 is type-only; T09 owns the
ladder. Acceptable per the task instruction.

## Teeth re-run verbatim by the verifier

1. **Tag rename (a):** edited `contract.go` tag `visible_objects` -> `objects`, ran
   `go test -count=1 ./internal/contract/`:
   `--- FAIL: TestGoldenRoundTrip ... world-snapshot.json: round-trip is not byte-stable; the Go json tags and the fixture disagree.` — exit 1.
   Reverted; `go test -race -count=1 ./internal/contract/` -> ok, exit 0; `grep '"objects"' internal/contract/contract.go` -> no match (no residue).
2. **Bogus TS-union-only kind (b, layer 1):** added `'BOGUS_KIND'` to `ConsequentialIntentName`
   only; `bun run typecheck` -> exit 2 with
   `src/ports/wireContract.test.ts(163,10): error TS2344: Type 'false' does not satisfy the constraint 'true'` (WireContractPins). Runtime `bun test` still 195 pass — matches the builder's honest note; the
   typecheck IS a T01 gate, so "drift detection fails somewhere" holds. Reverted.
3. **Bogus kind fully followed through TS (b, layer 2):** added `'BOGUS_KIND'` to all three unions
   (`ActionIntentKind`, `InteractionActionName`, `ConsequentialIntentName`), both hardcoded lists
   in `wireContract.test.ts`, plus a new golden `intent-bogus-kind.json` imported into
   `INTENT_FIXTURES`: TS typecheck exit 0 AND TS tests 10/10 pass; then
   `go test -count=1 ./internal/contract/` -> exit 1:
   `--- FAIL: TestGoldenCoversEveryIntentKind ... action_name "BOGUS_KIND" is not in the Go AllIntentKinds list`. Reverted all edits; `rg BOGUS_KIND` repo-wide -> clean; both suites re-green.

## Numbered findings (all personally observed)

1. **types.ts is purely additive — CONFIRMED.** `git diff HEAD -- types.ts` contains **0 removed
   lines**; the new content is new union members (`PROPOSE_CASE`, `EXECUTE_ITEM`,
   `COMPLETE_HUMAN_TASK`, `REOPEN_CASE`, `TERMINATE_CASE` in `ActionIntentKind`; those five plus
   `ADD_DISCRETIONARY_WORK` in `InteractionActionName`), new types (`ConsequentialIntentName`,
   `IntentRefusalKind`, `IntentRefusal`, payload interfaces, `TemplateParameter`,
   `TemplateEntryOption`, `TemplatePreflightResult`, `ErrorPayload`, `InterruptedPayload`), new
   optional fields (`refusal?` on `IntentResponse`), and two new `StreamEventType` members
   (`interrupted`, `error`). No existing shape reshaped. Pre-existing kinds verified not
   duplicated: `APPROVE_HUMAN_TASK`, `REJECT_HUMAN_TASK`, `ESCALATE_OR_OVERRIDE`, `OPEN_ARTIFACT`
   (and `ADD_DISCRETIONARY_WORK` in `ActionIntentKind`) were already at HEAD (`git show HEAD:...`).
2. **Kind sets exact.** `ConsequentialIntentName`/Go `AllIntentKinds` = exactly the required 10
   journey intents; `IntentRefusalKind`/`AllRefusalKinds` = exactly the required 7 refusals;
   `PayloadCarryingKinds` mirrors `PayloadCarryingIntentName` (6 kinds). SSE union covers
   revision/snapshot (pre-existing `snapshot`+`patch`, keyed by the `"<epoch>.<seq>"` cursor that
   doubles as Last-Event-ID), connection/interruption (new `interrupted`) and `error` (new).
3. **Payload fields complete per spec.** `ProposeCasePayload{template_ref, params,
   preflight_digest}`; `AddDiscretionaryWorkPayload{case_id, stage_id, anchor_item_id?, kind,
   title, summary?, justification}` (stage/item anchor, kind, title/summary, justification);
   `ExecuteItemPayload{item_id}`; `CompleteHumanTaskPayload{item_id, result, mandatory
   justification}`; `CaseLifecyclePayload{case_id, reason}` for REOPEN_CASE/TERMINATE_CASE.
   Enforced in both suites (Go `checkIntentFixture` required-field checks; TS `checkTypedPayload`).
4. **Go json tags match TS field names exactly — CONFIRMED by full field-by-field comparison**
   (60+ fields, far beyond the 10 required spot-checks): world
   (`world_id/case_id/cursor/timestamp/perspective/summary/visible_objects/available_actions/attention_focus`),
   objects (`id/kind/name/status/badge/explanation/salience/parent_id/depends_on/spatial_layout/actions`),
   actions (`id/label/intent/variant/consequential/requires_justification`), intent envelope
   (`intent_id/kind/action_name/target_object_id/case_id/client_cursor/actor{actor_id,role}/parameters/justification`),
   response (`intent_id/success/new_cursor/refusal/error_code/error_message/resulting_object`),
   refusal (`refusal_kind/message/current_cursor`), templates
   (`template_ref/title/description/parameters{name,title,description,type,required,default_value,options}`),
   preflight (`template_ref/params/passed/reasons/digest`), SSE
   (`event_type/cursor/timestamp/payload`) and all typed payloads. Byte-stability of the round-trip
   proves the tags agree with the fixtures; the fixture tag-rename tooth proves the tags agree with the contract.
5. **Golden coverage complete — CONFIRMED.** 15 files: 1 world snapshot; 10 intent kinds each with
   request + responses (every fixture has a success; 7 kinds also carry a refusal); all 7 refusal
   kinds exercised (STALE_PROJECTION, JUSTIFICATION_REQUIRED, AUTHORITY_DENIED, SOD_VIOLATION,
   UNAUTHORIZED_ROLE, UNAVAILABLE, INVALID); `templates-entry-options.json`,
   `template-preflight-pass.json`/`-fail.json`; `sse-events.jsonl` with exactly 9 lines, one per
   stream event kind (TS asserts `lines.length === STREAM_EVENT_KINDS.length` plus set-equality).
   Canonical key order (Go struct order; sorted map keys) is pinned by the byte-stable round-trip.
   Both suites read every file: Go via `os.ReadDir` walk; TS imports all 14 .json + reads the jsonl.
   *Observation (non-blocking):* 3 kinds (`REJECT_HUMAN_TASK`, `OPEN_ARTIFACT`, `REOPEN_CASE`) have
   success-only fixtures. The instruction's "success + refusal examples" is satisfied at set level
   (all 7 refusals covered across the goldens) and the plan's own gate wording is only "intent
   request and response per kind"; no refusal-kind or intent-kind is unexercised.
6. **The golden test READS the directory, not a hardcoded list — PROVEN.** Adding
   `junk-probe.json` to golden/ made `go test -count=1 ./internal/contract/` fail with
   `unrecognized golden fixture "junk-probe.json": every file under golden/ must map to a contract
   struct` (exit 1); `requiredFixtures` additionally fails on deletions. Probe removed; green.
7. **Kebab retirement complete.** Repo-wide searches found **zero code importers** of
   `apps/godspeed-cognitive-ui/contracts/*` (src/, e2e/, scripts, tests clean; the only UI import
   of the word "contracts" is the canonical `interface-contracts` import in `src/ports/contract.ts`).
   The directory is gone from disk and staged as `git rm` (6 files, 279 deletions); tsconfig no
   longer includes it. Remaining references are non-code: `reference/WIRE.md` (fixture-stage wire
   doc, explicitly superseded by the ADR; T06 replaces it), the Go-side fixture vocabulary in
   `internal/coordinator`/`projection` + Go README (still-live fixture wire that T06 migrates per
   the ADR's mapping note — correct sequencing, since T01's plan scope is "generates Go structs
   from the contract, or checks them against it", which the mirror+goldens satisfy),
   `GraphRenderer.tsx:327` (a `data-testid="graph-focus-object"` DOM id, not wire vocabulary),
   `.agents` reports/specs (historical), and a stale `graft/` local index entry (untracked,
   generated, gitignored — not shipped).
8. **Scope and constraints clean.** No crates/ file is in T01's changeset (the crates churn is
   T04A's; `.agents/evidence/casework-live-wiring/T04A/` exists with its own gate logs). No new
   dependencies: `go.mod` has zero `require` directives (stdlib only); `package.json`, `bun.lock`,
   `go.sum` unmodified (`git status` clean on all four). No commits (HEAD is T00 `8ad72ac`). No
   existing test weakened: the only UI test change is the new file; 195 = 185 baseline + 10 new.
   The contract package is not yet imported anywhere (correct — wiring is T06).
9. **Pre-existing flake observed (NOT a T01 defect; debt for the owning unit).**
   `TestIdempotencyReplayAndConflict` in `internal/coordinator` failed once during my first full
   `go test -race -count=1 ./...` (`coordinator_test.go:190: replay appended another revision:
   1151 -> 1153`), then passed on re-run; it also fails ~1/5 **in isolation** (`go test -race
   -count=1 -run TestIdempotencyReplayAndConflict ./internal/coordinator/`), with **zero local
   modifications** to that package (`git status --porcelain -- internal/coordinator/` empty). It is
   timing-flaky independent of T01 (the builder's gate log also caught a green roll). It makes the
   global gate `go test -race ./...` intermittently red and should be recorded in
   `.agents/OBSERVED_DEBT.md` by the gateway-owning unit (this verifier was instructed to modify
   no file but this one).

## Judgment of the four reported deviations

1. **`-count=1` on Go gates — SOUND, in fact necessary.** Personally demonstrated: with a
   key-broken fixture (`visible_objects` -> `objects` in world-snapshot.json), `go test
   ./internal/contract/` reports `ok ... (cached)` exit 0, while `go test -count=1 ...` fails exit
   1. Go's test cache keys on package sources, not runtime-read files; the flag strengthens, never
   weakens, the gate. The builder documented it in the test header and ADR.
2. **Success without `new_cursor` allowed; refusals never carry `new_cursor` — SOUND.** `new_cursor`
   was already optional on `IntentResponse` at HEAD, and the additive-only constraint forbids
   tightening it; `OPEN_ARTIFACT`'s success fixture without `new_cursor` is semantically correct
   (non-mutating read). Both suites enforce the invariants that matter: refusal ⇒ typed envelope
   present AND `new_cursor` absent; success ⇒ no refusal. The relaxation (not REQUIRING
   `new_cursor` on mutating successes) was not prohibited by the instructions.
3. **`GOLDEN_UPDATE=1` canonical rewrite affordance — SOUND with a bounded caveat.** Standard
   golden-test pattern: env-guarded, off by default, rewrites from the Go structs only, documented
   in the ADR with a two-sided-change protocol, and used once for key-order-only canonicalization.
   It cannot launder a one-sided change: a kind added only via GOLDEN_UPDATE'd fixtures still fails
   `TestGoldenCoversEveryIntentKind` (proven by tooth 3) and a TS-side pin failure still blocks
   `bun run typecheck`. Caveat: because it rewrites any byte mismatch, it must never be run as a
   reflex to a red test — the ADR says exactly this. Current fixtures are byte-stable without it.
4. **`tsconfig.json` dropping the `contracts` include (1 line) — SOUND.** The necessary and minimal
   consequence of deleting the directory; no other tsconfig change (`git diff` = exactly that one line).

## Minor observations (non-blocking)

- TS-side field-name drift detection is strongest where directly typed (`WorldSummary`,
  `AttentionFocus` assignments, all unions via `WireContractPins`); other interface field renames
  in types.ts are caught indirectly (Go golden byte-stability + fixtures remain the wire truth)
  rather than by a direct TS compile error. Every drift the task REQUIRED to be detectable (kind
  additions TS-only or followed-through; Go tag renames) was proven detectable by the teeth above.
- The ADR fully covers D-1, what ships, what is retired, drift prevention, and the Go vocabulary
  mapping note (`focus-object`/`decide-approval`/`propose-consequence`/... -> canonical names,
  coordinator lease stages -> SSE events), as required.

**Conclusion:** every deliverable (ADR, additive types.ts extension, golden fixtures, Go mirror
package with round-trip + exhaustive-kind tests, TS pinning tests, kebab retirement) is present,
correct against the original instructions, and proven by gates and teeth re-executed by this
verifier. Verdict: **APPROVE**.

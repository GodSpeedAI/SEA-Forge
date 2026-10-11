# T08 F4 shared conformance result

## Implementation

- Added `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts` as the single assertion implementation.
- `src/adapters/local/localAdapter.test.ts` invokes that runner against a fresh Northstar local adapter.
- `e2e/live-conformance.ts` invokes the same runner against `HttpCaseworkAdapter` connected to the script's fresh kernel/gateway cell built from `fixtures/cells/e2e`.
- The shared assertions cover current snapshot identity/shape; a retained snapshot read before and after new activity; trajectory case scope, cursor bounds, history retention, and the accepted revision; template listing and preflight digest; accepted and stale dispatch; event delivery and resume behavior; artifact content, digest, and provenance; and a before/after refusal state fingerprint.
- The live refusal fingerprint is a byte-for-byte snapshot of `cases/<case>/case-events.jsonl` and `approvals.jsonl`. This replaces the existing `|| true` assertion, which could not establish absence of writes.
- Existing local adapter tests for duplicate intents, role refusals, justification, invalid targets/actions, clone isolation, corrupt/unavailable artifacts, agent action standing, and event phase/order remain. The separate live checks for session bootstrapping, template-backed case proposal, operator standing, durable execution, and second-user identity also remain.

## Explicit fixture differences

| Behavior | Local fixture | Real HTTP adapter |
| --- | --- | --- |
| Case and accepted action | Northstar; `APPROVE_HUMAN_TASK` on `ns-release` | Newly proposed T00 E2E case; `EXECUTE_ITEM` on `task_prepare` |
| Initial retained history | Six authored Northstar points required | At least one retained point; only bounded relay history is claimed |
| Artifact locator and digest | `evi-assumption`; the authored marker is syntactically SHA-256-shaped but is not a content hash | Digest is read from the executed run's `evidence.jsonl`; returned content must hash to it |
| Artifact provenance | Requires the fixture's `case_id`; other fixture fields remain source-defined | Requires nonempty evidence/run metadata and checks the case/item identity supplied by this scenario |
| Event resume | The in-memory local adapter is future-only; the assertion rejects replay of cursors at or before the head at subscription time while allowing its pending execution events | Re-subscribes from the pre-mutation cursor and requires retained delivery through the HTTP event endpoint |
| Refusal no-write boundary | In-memory trajectory cursor list | Consequential case event and approval ledger bytes |
| Template data | `tpl-release-rollout` with its required local parameter | `e2e-sentry-chain@0.1.0` with the T00 fixture parameters |

These differences are scenario data passed to the shared runner; the behavioral assertions are not duplicated in the test and live script.

## Verification and limits

- Focused local command passed: `bun test src/adapters/local/localAdapter.test.ts` — exit 0, 13 tests, 21 assertions. Both the initial failing run and corrected passing run are preserved in `local-test-runs.md`.
- After that passing run, the artifact assertion was narrowed to the contract's honest semantics: provenance fields must be strings, `run_id` must be nonempty, and configured case/item values must match; empty invocation ID is allowed. This final source adjustment was not rerun because the focused compile token had been released.
- No UI-wide tests, typecheck, production build, live run, Go commands, or Cargo commands were run. The compile token authorized only the focused local Bun command and is released.
- The live runner was not run because its gateway binary is stale and the separate Go source changes need a fresh build authorization.
- The live artifact check requires source-backed case and plan-item attribution, validates that every provenance field is a string, and requires a nonempty run ID. The current route returns only `provenance.run_id`; a fresh builder is deriving case/item from canonical `run.get` data. The interface contract does not require a nonempty invocation ID and no separate invocation identity exists in the live kernel, so an empty invocation string remains honest and is not fabricated to satisfy the test.
- Trajectory assertions concern only revisions actually retained by the bounded live relay. The suite does not infer missing past events or substitute the viewer's identity for event attribution.

## Source diff summary

- Added the shared scenario type and `runCaseworkPortConformance` assertion function.
- Replaced duplicated local snapshot/history/dispatch/stale/event/artifact-basics tests with the shared runner; retained the adapter-specific tests listed above.
- Replaced the live script's handwritten snapshot/SSE/dispatch/stale subset with the shared runner, passed the live fixture parameters, located artifact digests from run evidence, and checked no-write against both consequential ledgers.

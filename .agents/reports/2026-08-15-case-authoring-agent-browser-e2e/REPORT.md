# Case-authoring proof journeys — agent-browser mocked-IPC evidence (2026-08-15)

Resolves the `OBSERVED_DEBT.md` entry "Case-authoring proof scenarios (6, 7)
have no Playwright e2e coverage". Per the owner's direction, the coverage is
driven by **agent-browser** (Vercel's browser-automation CLI) instead of
Playwright.

## Claim and its limits

**Claimed:** the real `CaseCreationWorkbench` UI — React components, TanStack
Query hooks, the `caseAuthoringMachine` XState machine, and the generated AJV
contract validators, all served by the real Vite dev build — renders the
correct affordances and enforces the two Task-6 proof scenarios end to end in
a real Chromium page.

**Not claimed:** anything about a Tauri host or a `sea-forge-server`. The
bridge is the agent-browser counterpart of the Playwright `e2e/tauriMock.ts`
shim (`workbench/apps/desktop/e2e-agent-browser/sfwp-case-authoring-mock.js`):
it answers `sfwp_query`/`sfwp_command`/`sfwp_request_status` with scripted
fixtures and logs every `case_commit`/`case_preflight` call with the
precondition digests actually carried on the wire. This is the same epistemic
class as the existing mocked Playwright suite.

## How to reproduce

```sh
just workbench-check        # installs the Bun workspace the runner needs
just workbench-e2e-case-authoring
```

Runner: `scripts/workbench-e2e-case-authoring.sh`. It boots an ephemeral
Vite server, opens the renderer bridge-less (fail-closed, per the
`workbench-e2e-agent-browser` contract), installs the shim into the live
document, and reaches `/cases/new` through SPA navigation only — a full page
load would wipe the shim, which is why the harness never reloads. Viewport is
1600×1000: below 1420px the evidence drawer overlays the attention rail and
intercepts clicks on the Commit control.

## Journey 7 — stale precondition repairs via re-preflight

Scripted transport: `case_commit` #1 returns `rejected_as_stale`; preflights
pin digest A then digest B (the template "changed on disk"); `case_commit` #2
succeeds.

Verified, in order:

1. Draft → preflight passes → Commit enabled.
2. Commit #1 → the stale alert ("The template changed since preflight. Re-run
   preflight to refresh before committing.") and the button relabels to
   "Re-run preflight" — the only repair affordance rendered.
3. The Commit control is `disabled=true` in the stale state (DOM-level).
4. Exactly one `case_commit` crossed the bridge so far, carrying
   `precondition.digest == A` (read from the mock's wire log, not the UI).
5. Re-preflight pins digest B; Commit re-enables.
6. Commit #2 succeeds carrying digest B (proving the repair really re-pinned
   the precondition rather than re-sending the stale one); the app navigates
   to `/cases`; `request.get_status` is never called (`statusCalls == 0`).

Screenshot: `journey7-rejected-as-stale.png` (stale state, disabled reason
visible), `journey7-repaired-and-committed.png` (horizon after commit).

## Journey 6 — dropped commit response recovers, never resubmits

Scripted transport: `case_commit` #1's promise rejects (connection closed
before response); `sfwp_request_status` later reports the outcome.

Verified, in order:

1. Draft → preflight passes → Commit #1 → the alert ("The commit response
   was lost. Recover the outcome instead of resubmitting.") and a "Recover
   outcome" button — no resubmit affordance anywhere.
2. The Commit control is `disabled=true`; a forced programmatic
   `button.click()` on it does not reach the bridge (`commits.length` stays
   1). Duplicate commit is unreachable at the UI layer; the machine-level
   unreachability (no `COMMIT` transition from `ambiguous`) was already
   pinned by `caseAuthoringMachine.test.ts`.
3. Zero axe violations on `#main-content` in the ambiguous state
   (`journey6-ambiguous.a11y.json`).
4. "Recover outcome" → one `request.get_status` call → the machine folds the
   recorded outcome and navigates to `/cases`.
5. End state: exactly **one** `case_commit` for the whole journey, carrying
   digest A; `statusCalls == 1`.

Screenshot: `journey6-ambiguous.png` (recovery state),
`journey6-recovered.png` (horizon after recovery).

## Cross-cutting checks

- `agent-browser errors`: zero page errors across both journeys.
- Assertion teeth were demonstrated during development, not assumed: every
  commit-count, digest, and disabled-state assertion fired against a real
  defect at least once while the harness was being built (the per-journey
  mock `reset` exists precisely because a shared counter let journey 6's
  commit succeed silently).

## Findings filed during this work (not fixed here)

1. The authoring status pill keeps the label "Preflight passed" while the
   machine is in `rejected_as_stale` — `CaseCreationWorkbench`'s label
   ternary tests `preflight?.ok` before `state`, so the stale state's
   "Rejected as stale" label is unreachable after a successful preflight
   (see new `OBSERVED_DEBT.md` entry).
2. The Playwright `e2e/tauriMock.ts` readiness fixtures fail the current
   generated `ReadinessView` contract (`next_lawful_action` is now required
   on every readiness item) — surfaced when this harness's first readiness
   fixture was validated with the real AJV validator (see new
   `OBSERVED_DEBT.md` entry).

## Harness notes for future journeys

- `agent-browser` has no `addInitScript`; the shim is `eval`'d post-load, so
  every post-install navigation must be a SPA route change.
- Nav links must be located by text (their accessible names carry hotkey
  suffixes, e.g. "Readiness R", which defeats role+name matching in
  agent-browser 0.19.0); buttons locate fine by role+name.
- `agent-browser eval` prints results JSON-encoded exactly once; strip the
  quotes before comparing strings.
- Shell-level queries with `staleTime: Infinity` (cell info) never recover
  from the bridge-less boot because the shell never remounts — cosmetic
  footer noise ("Cell unreachable") that does not affect the journeys; the
  identity query recovers because `/cases/new` mounts its own observer.

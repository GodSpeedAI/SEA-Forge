# Casework live wiring: delivery summary (documentation and GTM source)

Status: T09-T12 complete; T12 independent fresh-clone acceptance = CONFIRM WITH CONDITIONS
(`.agents/evidence/casework-live-wiring/T12/independent-confirmation.md`). T13 not started.

## What exists now
The GodSpeed casework UI runs against the real SEA Forge kernel, not fixtures:
React/Bun production build -> Go gateway (sessions, roles, SSE, metrics) -> SFWP over a unix socket ->
Rust kernel with a governed ledger. The gateway acts with delegated identity (`on_behalf_of`), so the
ledger records who did what.

## Claims the evidence supports
- No fixture adapter ships in the production bundle (scanned in every live run and on a fresh build).
- A full case lifecycle works end to end on a fresh cell, 3 consecutive times: design from template,
  preflight, commit, discretionary work, execute, two-person sign-off, settlement with evidence,
  close, reopen with reason, terminate. 11 journeys, 70 steps per run, each checked against durable
  files (case events, approvals, ledger, artifact hashes), not only the screen.
- Two distinct users are ledgered separately; the approver is a different principal from the requester.
- Evidence artifacts are hash-verified; a corrupted artifact produces a typed integrity error.
- Recovery: killing the gateway or kernel under 50 live SSE clients, all clients resync; memory stayed
  30-47 MB.
- Measured on one shared dev host: intent p95 8.0 s, event lag p95 7.75 s, 0 errors (budgets 20 s / 40 s).
- Tests: UI 373, Go 12 packages (plain and live tag), Rust 1158 pass / 0 fail.
- Packaging: systemd units, Prometheus-text metrics on a loopback listener, offline cell backup/restore.

## Do NOT claim (known limits)
- Production auth was not exercised in a browser run (live ladder uses dev auth); covered by Go tests only.
- No video or trace evidence exists (CW-44); only screenshots and an L6 HAR.
- An approved escalation does not resume the case in the kernel (CW-45).
- No execution progress frames and no typed settlement object from the live gateway.
- After a gateway restart the page shows Reconnecting until reload (CW-46).
- Backup is offline only; load budgets are single-host regression guards, not capacity numbers.
- The casework-live GitHub workflow has never been dispatched (CW-53); no external security review.
- Open operator decisions: CW-47, CW-49, CW-51. DEBT CW-43..CW-54 open.

## Demo script (repo root)
`just casework-e2e-live` (full ladder, ~15 min), `just casework-load`, `just casework-cell-backup` /
`casework-cell-restore`. Fixture ladder for UI-only demos: `just casework-ui-up` then `bun e2e/run.ts`.

## Documentation index
- Operations: `.agents/reports/casework-live-wiring/runbook.md`, `config-reference.md`, `load-budgets.md`
- Security: `security-review.md` (self-run by a delegated agent)
- Decisions and deviations: `decision-log.yaml` (T10-DEV-1..7, T11-DEV-1..4)
- Debt: `.agents/DEBT.md` CW-43..CW-54
- Component READMEs: `apps/godspeed-casework-go/README.md`, `apps/godspeed-cognitive-ui/README.md`,
  `apps/godspeed-cognitive-ui/e2e/README.md`
- Evidence: `.agents/evidence/casework-live-wiring/{T09,T10,T11,T12}`

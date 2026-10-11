# T00 baseline summary — casework-live-wiring-production (2026-09-23)

HEAD at capture: 44ffadd (branch casework/live-wiring created from it).
All gates below ran BEFORE any code change by this effort (justfile recipes and
live-serve.json landed after; they cannot affect these results — they change no
Rust, Go, or UI source).

## Global gates (plan verification.global_gates 1-4)

| Gate | Result | Log |
|---|---|---|
| `cargo test -p sea-forge-server -p sea-forge-planner -p sea-forge-case-runner -p sea-forge-cli` | GREEN (exit 0) | baseline/gate-cargo.log |
| `cd apps/godspeed-casework-go && go vet ./... && go test -race ./...` | GREEN (vet 0, race 0) | baseline/gate-go.log |
| `cd apps/godspeed-cognitive-ui && bun run typecheck && bun test` | GREEN (typecheck 0; 185 tests, 13 files) | baseline/gate-bun.log |
| `cd apps/godspeed-cognitive-ui && bun e2e/run.ts` | RED (deterministic) — see below | baseline/gate-e2e-ladder.log |
| `just casework-e2e-live` | n/a — recipe does not exist until T10 | — |

## Divergences from the plan's baseline_warnings (recorded, none blocking)

1. The archived warnings expected pre-existing cargo reds: "sea-forge-server
   exits 101 on a pre-existing sfwp trio, and sea-forge-cli exits 101 on
   pre-existing t13_1." Neither reproduced: the four-crate cargo gate is GREEN
   at 44ffadd. No test IDs to flag; the warning is stale (it described an
   earlier tree).
2. The local ladder was described as "11/11 PASS per CURRENT_STATUS.md ...
   regression gate for T08 and T09." It is RED at baseline, deterministically:
   run 1 (gate-e2e-ladder.log): J0 PASS; J1 FAIL at step "focus again: identity
   and state survive" (42075ms); J2..J9 + RECOVERY BLOCKED by dependency.
   run 2 (`--only J1`, gate-e2e-ladder-j1-rerun.log): J1 FAIL at the same step
   (122372ms). Recorded as OBSERVED_DEBT ("local UI e2e ladder deterministically
   red at J1"); T09 owns restoring it. The 11/11 evidence committed in 44ffadd
   itself is stale relative to that commit's UI rework.

## Environment incidents

- E-1 (decision log): repo `target` was a tracked, broken, self-referential
  symlink (committed in 118761a) that made every cargo command fail ("Not a
  directory"). Removed before the cargo baseline re-run (env-notes.txt). The
  tracked symlink deletion is included in T00's commit.
- Sibling evidence files: dirty-inventory.txt (tree was clean at branch
  creation; the 109-entry dirty baseline had been committed as 44ffadd),
  c1-git-evidence.txt (correction C-1).

## T00 task gate + teeth (after the recipe changes)

- Gate `just casework-cell-init && just casework-server-up && just
  casework-live-go-up && just casework-stack-down`: PASS (two full cycles —
  see gate-teeth/ logs; independent critic re-ran the chain and all teeth and
  approved: confirmation.md).
- Teeth "start casework-server-up twice": second call exits 1 with
  "casework-server-up: server already running (pid N, socket ...)"; exactly one
  listener. Verified by builder and independently.

## Builder/verifier chain for this task

- Builder: subagent (bounded spec: recipes + live-serve.json + self-verification).
- Independent critic: subagent re-ran gates/teeth/idempotency/crash-safety and
  APPROVED (confirmation.md). T00's plan-level confirmation requirement is
  `builder`; the independent pass exceeds it.

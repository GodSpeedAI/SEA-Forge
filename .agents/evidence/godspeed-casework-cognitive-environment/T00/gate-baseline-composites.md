# T00 evidence — SEA Forge composite gate baseline (`just check`, `just ci`)

Revision `6ce518fcd9b01bc5a7037f80f5d8986a33cd2924`, dedicated worktree,
`CARGO_BUILD_JOBS=1`, `CARGO_TARGET_DIR=/home/sprime01/projects/sea-rs/target`, one heavy gate
at a time. Raw logs: `raw-logs/round2-{proof,check,ci}.log`, `raw-logs/round2-status.txt`.

This file exists because the round-0 handoff recorded the composites **by pointer to a file that
did not exist yet** — an unsupported claim an independent verifier caught (finding F5 of
`.agents/evidence/godspeed-casework-cognitive-environment/T00/T00-verification-round-1-NOT_CONFIRM.md`).
The gates below were then actually run, and this is their result.

| Gate (verbatim) | Exit | Wall | Failing step | Verdict |
|---|---|---|---|---|
| `just proof` (re-run, now with a `/usr/bin/time -v` block) | **0** | 3 s | — | PASS (`[proof] P1-P4b passed`); max RSS 72784 kB |
| `just check` | **1** | 101 s | `just security` | **RED**, and red *only* at security |
| `just ci` | **1** | 41 s | `just security` | **RED**, and red *only* at security |

## `just check` — step-by-step evidence

`just check` runs `context-check → fmt-check → lint → typecheck → security` under `set -e`.
Log evidence (`raw-logs/round2-check.log`):

- line 2: `context check passed` — the handoff contract is satisfied.
- line 3: `cargo fmt --all -- --check` — no diff.
- line 4: `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`;
  line 69: `` Finished `dev` profile [unoptimized] target(s) in 29.53s `` — clippy clean under
  `-D warnings`.
- line 70: `cargo check --workspace --all-targets --locked`; line 129: `` Finished … in 14.66s ``.
- lines 1968–1978: the security step, then
  `error: recipe \`security\` failed with exit code 1` → `error: recipe \`check\` failed with exit code 1`.

So the composite is red **solely** because of the pre-existing gitleaks baseline analysed in
`gate-baseline-seafoerge.md` (prerequisite B2); every other `check` component is green at this
revision.

## `just ci` — step-by-step evidence

`just ci` runs `context-check → fmt-check → lint → typecheck → security → test → no-async-kernel
→ build`. Because `security` fails, the recipe stops there and `test` is never reached in this
composite. Log evidence (`raw-logs/round2-ci.log`): line 2 `context check passed`; line 1968
`error: recipe \`security\` failed with exit code 1`; line 1969 `error: recipe \`ci\` failed with
exit code 1`.

**Proof-boundary statement:** a red `just ci` here proves *only* that the composite stops at the
already-recorded security baseline. It says nothing about `test`/`build` inside `ci`, which are
covered by the separate `just test` (exit 0, 290 s) and `just build` (exit 0, 70 s) baselines in
`gate-baseline-seafoerge.md`. `no-async-kernel` was not run and is **not** claimed.

## Consequence for the plan

`GATE_SEAFORGE` is `just check && just test && just ci && just proof`. At this revision it cannot
pass: `just check` and `just ci` both fail at `security`. That is prerequisite **B2**, and it is
recorded rather than worked around. `just test` and `just proof` are green, so resolving B2 would
leave the whole `GATE_SEAFORGE` chain green at this revision — which is the strongest available
statement, and it is deliberately not stated as "the gate passes".
# SEA-Forge security round 4 — handoff prompt

You are continuing the SEA-Forge security program (SEC-01–SEC-12) in
`~/projects/sea-rs` (GitHub `GodSpeedAI/SEA-Forge`). This is my own project. The
work is defensive: closing gaps in my sandbox, and adversarially testing my own
containment against fixtures the tests create themselves. Round 3 was merged to
`main` as squash commit `257b622` (PR #32). Read these first:
`.agents/CURRENT_STATUS.yaml` (revision 112: verified, limits, next) and
`.agents/evidence/security-v2/round3-20261017/README.md`, along with the
repository's `AGENTS.md` / `CLAUDE.md`.

## Preconditions — check, don't fix

- The local `main` of `~/projects/sea-rs` should be fast-forwarded to `origin/main`
  (≥ `257b622`) with a clean tree. If it isn't, stop and tell me. Don't restore,
  reset or delete my files to make it clean.
- Work in a new worktree under `~/projects/_wt/` on a new branch
  `security/round4-<yyyymmdd>`. Give the worktree its own `target/`; never symlink
  `target/debug` into the main checkout.

## Hard rules

1. **Frozen v1 is immutable.** Never modify `tests/security-v1` or anything
   pinned by tag `sea-forge-security-v1-2026-10-16`
   (`af41ea2465e38c87dfe86471c33c6b54ea8564e1`). The v1 runner must still report
   6 PASS / 10 NOT-YET-EXECUTABLE, or more PASS. Any change of verdict needs evidence.
2. **No secrets** in code, logs, evidence or commits. Generate any synthetic
   tokens at run time.
3. **Git:** never fetch, pull, reset, clean, rebase, merge, push or delete
   branches without my explicit go. Make path-scoped commits (`git add <paths>`),
   in Conventional Commit format, ending with the co-author line your harness
   gives you. A commit that changes project files must also update
   `.agents/CURRENT_STATUS.yaml` and `.md` (add a new revision, 113+). The
   pre-commit and pre-push hooks enforce this.
4. **Test scripts that create throwaway git repositories** must first run
   `git rev-parse --local-env-vars` and unset every variable it lists. Under a
   hook, `GIT_DIR` otherwise points every `git -C <tmp>` at the real repository.
   In round 3 this flipped `core.bare` and moved a branch.
5. **WSL host limits:** run one cargo process at a time with `-j 1` and
   `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`, and keep targets narrow
   (`-p <crate> --test <name>`). Check free memory before full suites.
6. **Isolation tests** need `SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1` on this
   host, which allows mapped user namespaces. GitHub Ubuntu runners refuse them,
   so CI reports those tests NOT-EXECUTABLE.
7. **Mutation control for every fix:** break the fix, show its test fails,
   restore the source and `touch` it so cargo rebuilds, then record the result
   under `.agents/evidence/security-v2/round4-<date>/`.
8. Label claims honestly: LOCALLY TESTED, CI-VERIFIED, or INDEPENDENTLY
   RE-VERIFIED. Nothing is SETTLED without an independent re-attack.

## Work items

### A. Pathname Unix-socket containment (SEC-06 open limit) — main item

**Current state:** `crates/sea-forge-sandbox/tests/unix_socket_reach.rs` is
`#[ignore]` and fails when run. Landlock confines opening files, but `connect(2)`
to a pathname socket is not an open. An isolated workload can therefore reach a
listening pathname socket outside its declared trees, provided DAC allows it.

**Test rules:**
- Use only listener sockets the test creates in its own temp directories, as the
  existing test does.
- Do not enumerate, probe or connect to sockets the host already has, and don't
  write tooling that does.
- Containment is proven against controlled canaries placed where real
  services would sit.

1. **Design.** Write `docs/decisions/ADR-0xx-unix-socket-containment.md`
   comparing the candidate mechanisms against this host's kernel (6.18 WSL2).
   Read the current kernel and Landlock documentation; don't rely on memory for
   which Landlock ABI scopes or access rights exist, or what they cover.
   Candidates:
   - Landlock scoping or socket-resolve rights, if the running kernel's ABI
     provides them. Note abstract-socket scoping is a separate guarantee from
     pathname sockets.
   - Building the workload's mount view so that only declared trees exist in it.
   - A seccomp user-notification supervisor for `connect`/`sendto`/`sendmsg` on
     `AF_UNIX`, checking the resolved path against declared trees. Address TOCTOU.
   - DAC/ownership hardening as defence in depth only, not as the guarantee.

   State what each guarantees, its residual risk and its availability.
   Explicitly handle the declared PostgreSQL socket directory, which must stay
   reachable only when granted, and abstract sockets. Then **stop and ask me to
   pick** before implementing.
2. **Implement** the chosen mechanism behind the existing workload-isolation
   availability probe. If the kernel lacks the feature, fail closed with a
   NOT-EXECUTABLE or unavailable report instead of silently degrading.
3. **Adversarial test suite**, all against test-owned canary listeners:
   - socket outside every tree (must be refused);
   - socket under a declared read path (refused unless explicitly socket-granted);
   - granted PostgreSQL socket directory (allowed);
   - an abstract-namespace canary (refused);
   - a symlink or bind-mount alias to a canary outside the trees (refused);
   - a rename or swap race between check and connect, if using a supervisor (refused);
   - `sendto`/`sendmsg` on an unconnected datagram socket to a canary (refused);
   - a descriptor obtained before the restriction applied must not leak into
     the workload (check inherited descriptors);
   - socket created by the workload itself inside its own run tree (allowed),
     so the fix doesn't break legitimate IPC.

   Un-ignore `unix_socket_reach.rs` once it passes, and add a mutation control
   per mechanism.
4. Re-run the seccomp before/after syscall evidence and the frozen v1 runner.

### B. Admission race (pre-existing on `main`)

`crates/sea-forge-server/tests/conformance_sfwp.rs`,
`request_admission_refuses_overflow_and_cancels_waiters_without_effects`, fails
about 1 run in 13 to 1 in 40, on `main` too, with "pre-admission timeout reached
provider" (5 vs 4). There is also a fast-failure variant that fails within about
0.2 s. The test holds all four `max_concurrent_runs` permits on a hanging
endpoint, so a fifth provider connection means either a waiter was admitted
while four were held, or the test is miscounting.

- Find the root cause in the admission semaphore and the `ADMISSION_WAITING` /
  `ADMISSION_CANCELLED` / `ADMISSION_GRANTED` handoff in
  `crates/sea-forge-server/src/lib.rs` (`dispatch_bounded`, the bounded admission
  wrapper).
- Reproduce by looping the built test binary until it fails.
- Treat it as a real bug until proven otherwise. If it's a server bug, fix it
  and add a deterministic test. If it's the test, fix the test and explain why
  it can't hide a real overflow.
- Show at least 200 consecutive passes after the fix.

Also check `conformance_m12` `t12_endpoint_registration_marks_self_model_snapshot_stale`.
It failed once under full-suite load (5 s fixture timeout); only fix it if you
can show a cause.

### C. Remaining round-3 limits — report, then ask

For each of these, write a short options note and wait for my decision rather
than changing policy yourself:
- Federation export/import/adopt are denied as unclassified because
  `export_bundle` is not on the reserved-surface list. Should they be enabled,
  and under what exposure rules?
- A depth-1 CI gitleaks scan reports about 22 generic-api-key matches in
  committed round-1/2 evidence, which pass history scans only through
  commit-pinned exceptions. Should those become value-scoped exceptions, or
  should the files be redacted?
- New generic-api-key matches under casework evidence paths are still admitted.
  Should that rule be narrowed?
- RealityTrace E8 field-level differences are bounded at 256 entries, and
  ledger observer attribution isn't a signature (ADR-012 obligation 4). Propose
  a path.

### D. Independent re-attack (after A and B)

Spawn a context-isolated reviewer that hasn't seen your implementation notes.
It re-runs all 38 scenarios in `.agents/evidence/security-v2/adversarial-20261017`
plus new attacks on round-3 and round-4 changes:
- PostgreSQL session supervisor and exposure gate;
- state-tree DAC;
- seccomp filter (including the x32 ABI);
- host-symlink trust in `open_observation_root`;
- startup ledger repair (`recover_interrupted_appends`: can a crafted tail or
  `mmr.json` make repair hide tampering?);
- Unix-socket containment.

Record verdicts as HELD / PARTIAL / BYPASS / INCONCLUSIVE in
`.agents/evidence/security-v2/adversarial-round4-<date>/`.

## Gates before asking me to push

Run these in the worktree and save logs under the round-4 evidence directory:
- `just ci`, the full suite, one cargo at a time;
- the frozen v1 runner, with `SEA_FORGE_SECURITY_MCP_COMMAND` provisioned;
- the staged-secret canaries;
- gitleaks history;
- every mutation control.

Then summarize what changed and its status level, and ask me before pushing,
opening a PR or merging.

# SEA Forge Server Restart-Lock Remediation Plan

**Date:** 2026-09-01  
**Status:** Planned  
**Scope:** Unblock `just crate-test sea-forge-server` by correcting the restart lifecycle exercised by `conformance_run_locator::both_layouts_still_resolve_after_a_restart` without weakening cell or socket exclusivity.

## 1. Problem statement

The restart conformance test starts the server with a detached Tokio task. It drops the client but neither stops nor awaits the first server task before starting a second server against the same cell root. The first server therefore still owns the lifetime-scoped cell lock. The second start correctly fails closed, and the helper later reports a missing socket.

The implementation must distinguish these cases:

1. **Live-owner contention:** a second server for the same cell must fail closed.
2. **Real restart:** after the first server task has terminated and released all guards, a second server must start and read persisted records.
3. **Stale filesystem artifacts:** a dead process's socket artifact must not be treated as a live lock owner.

Do not “fix” the test by weakening `lock_cell_root`, deleting a live lock, ignoring startup errors, adding sleeps, or allowing two writers.

## 2. Governing invariants

- One live server process/task may write a cell root at a time.
- Explicitly different socket paths do not bypass cell-root exclusivity.
- The server owns the cell and socket locks for its full serving lifetime.
- Shutdown completion is observable: restart begins only after the first server future has ended and its RAII lock guards have dropped.
- Persistent run resolution does not depend on memory from the first server.
- Startup failures are returned to the test. Helpers must not discard `run(config).await` results.
- No production API or lock semantics change unless focused evidence shows the test harness is not the sole defect.

## 3. Scope and non-goals

### In scope

- `crates/sea-forge-server/tests/conformance_run_locator.rs` test-server lifecycle.
- A reusable test harness in that integration test, or an existing test-support module if one already owns this concern.
- Focused regression tests for lock ownership, shutdown, and restart.
- The smallest production correction only if an explicit shutdown-and-await reproducer proves that the lock survives server termination.

### Out of scope

- Weakening authority, lock, socket, or request-admission behavior.
- Adding a public shutdown API merely for this test without separate approval.
- Broad refactoring of server startup.
- Cleaning `target/`, increasing Cargo concurrency, or changing toolchain/build configuration.
- Unrelated worktree changes.

## 4. Execution plan

### Phase 0 — Capture the baseline

1. Confirm no compile-heavy job currently owns the shared build directory.
2. Record `git status --short`, the exact Rust toolchain, and `du -sh target`.
3. Run only:

   ```sh
   just crate-test sea-forge-server both_layouts_still_resolve_after_a_restart
   ```

4. Preserve the failing output. It must show that the second server cannot acquire the still-live cell lock and that the detached helper hides the startup result.
5. Inspect `run`, `lock_cell_root`, `lock_socket_path`, and nearby transport-hardening tests. Record whether lock guards are ordinary lifetime-scoped RAII values.

**Exit criterion:** the failure mechanism is proven, not inferred from the final `NotFound` symptom.

### Phase 1 — Make test-server lifetime explicit

Replace the socket-only `serve` helper with a bounded test harness that owns at least:

- the socket path;
- the `JoinHandle<Result<...>>` for `run(config)`;
- a readiness operation that fails if the task exits before publishing the socket;
- an explicit `stop(self)` that terminates the task, awaits task completion, and does not return until lock guards have dropped.

Requirements:

- Do not detach and discard the server result.
- Do not use fixed sleeps as proof of shutdown.
- Bound readiness and shutdown with Tokio timeouts so failures cannot hang CI.
- On early server exit, expose the actual startup error.
- Ensure cleanup executes on assertion failure, preferably through a small guard or an explicit cleanup path.
- Use task abortion only as a test equivalent of process termination, and always await the aborted handle. If the codebase already has a production-supported graceful shutdown mechanism, use it instead.

**Exit criterion:** a test can prove “first server ended” before attempting the second start.

### Phase 2 — Correct the restart test

Update `both_layouts_still_resolve_after_a_restart` to:

1. seed both layouts;
2. start server A and verify a read;
3. drop the client;
4. explicitly stop and await server A;
5. start server B against the same root with a distinct socket;
6. verify both the case-owned and flat run layouts;
7. explicitly stop and await server B.

The assertion must preserve the semantic claim: resolution survives a completed server lifetime and comes from durable records.

**Exit criterion:** the focused restart test passes without changing lock behavior.

### Phase 3 — Add adversarial regression tests

Add focused tests that discriminate the correct fix from unsafe shortcuts:

1. **Client-drop is not shutdown:** dropping all clients while server A remains live does not release its cell lock.
2. **Same root, different socket:** server B fails closed while server A owns the root, proving a socket override cannot bypass the cell lock.
3. **Shutdown then restart:** after `stop().await`, server B starts promptly and resolves both persisted layouts.
4. **Stale socket recovery:** after the owner has ended, a stale socket path does not block lawful restart, subject to the existing socket-safety contract.
5. **Different roots:** two servers on separate cell roots may run concurrently; the test harness must not accidentally serialize all servers globally.
6. **Repeated lifecycle:** run a bounded sequence of at least 20 start/ready/stop/restart cycles against one root. Every cycle must terminate and reacquire locks without a sleep-based retry.
7. **Early startup failure visibility:** deliberately create live-owner contention and assert that the harness reports the typed/actual startup failure rather than timing out waiting for a nonexistent socket.
8. **Persistence, not cache:** mutate no in-memory state between server A and B; verify server B reads seeded/persisted records only.

Where timing is relevant, use explicit task completion, channels, or lock acquisition outcomes. Avoid probabilistic races and generous sleeps.

**Exit criterion:** tests fail if lock exclusivity is weakened, shutdown is not awaited, or startup errors are swallowed.

### Phase 4 — Decide whether production code needs a change

Run the adversarial tests with the test-harness correction first.

- If explicit task termination and awaiting releases both locks, make no production lock change.
- If either lock survives completed task termination, add a minimal production regression test around the precise owner and fix its drop/lifetime path.
- If a graceful production shutdown API appears necessary, stop and request approval because that changes the public server lifecycle interface.

Review any production diff for:

- lock guard lifetime shortening;
- socket unlink races;
- a second writer becoming possible;
- task/connection leakage after listener shutdown;
- startup side effects before lock acquisition;
- errors converted into silent success.

## 5. Verification ladder

Run compile-heavy commands serially:

```sh
# Fast semantic/build check
just crate-check sea-forge-server

# Focused restart and adversarial cases
just crate-test sea-forge-server both_layouts_still_resolve_after_a_restart
just crate-test sea-forge-server live_owner
just crate-test sea-forge-server shutdown_then_restart
just crate-test sea-forge-server repeated_restart

# The originally blocked scope
just crate-test sea-forge-server

# Required handoff gates
just check-fast
just context-check
git diff --check
```

Use the exact final test names if they differ from the plan labels. Do not claim the crate suite is green unless the unfiltered `just crate-test sea-forge-server` exits zero.

Compilation expectations:

- Editing the integration test compiles/relinks that test target and any invalidated dependencies.
- Editing `src/lib.rs` recompiles `sea-forge-server` and dependent test targets.
- Incremental artifacts should remain hot. Do not run `cargo clean`.
- Observe the single-writer build rule. Never overlap these Cargo/Just checks with another compile-heavy job.

## 6. Concrete evidence package

Create `.agents/evidence/server-restart-lock/2026-09-01/` and store:

- `baseline-targeted-test.log` — original focused failure and exit code;
- `diagnosis.md` — lock owner, task owner, and causal sequence with source locations;
- `targeted-restart-test.log` — corrected restart test, command, exit code, and test count;
- `adversarial-tests.log` — all adversarial cases and results;
- `server-crate-suite.log` — full unfiltered crate test output and exit code;
- `check-fast.log`, `context-check.log`, and `diff-check.log`;
- `manifest.yml` — commit/worktree revision, toolchain, platform, commands, timestamps, exit codes, and SHA-256 digest for every evidence file.

Evidence rules:

- Capture stdout and stderr without editing away warnings or failures.
- Include the exact command and exit code in each log.
- Do not include secrets, environment contents, or sensitive payloads.
- If a gate fails for an unrelated reason, preserve the failure and classify it; do not relabel the gate as passing.
- Concrete acceptance evidence is the passing full crate command plus adversarial tests that would catch lock weakening. A passing targeted happy-path test alone is insufficient.

## 7. Review and acceptance

Before completion, review the diff adversarially for test-only false confidence and request an independent reviewer to answer:

1. Does the test now perform a real completed restart?
2. Can two live servers ever own one cell root?
3. Can an alternate socket bypass root exclusivity?
4. Are server startup errors observable rather than swallowed?
5. Are shutdown and lock release synchronized without sleeps?
6. Do the adversarial tests fail under the obvious unsafe mutations: removing the root lock, not awaiting shutdown, or ignoring the server result?

Completion requires all of the following:

- focused restart test passes;
- adversarial lock and lifecycle tests pass;
- `just crate-test sea-forge-server` exits zero;
- `just check-fast`, `just context-check`, and `git diff --check` pass;
- evidence manifest is complete and digest-valid;
- independent review returns **CONFIRM**;
- `.agents/current_status.yml` and `.agents/CURRENT_STATUS.md` record the resolved blocker and exact verification.

## 8. Rollback condition

Rollback or redesign if the proposed change makes a second same-root server start while the first is live, relies on delay-based lock release, hides startup errors, changes public lifecycle contracts without approval, or causes any no-effect/authority regression.

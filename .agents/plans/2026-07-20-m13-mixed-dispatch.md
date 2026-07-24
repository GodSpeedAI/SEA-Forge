# M13 Mixed Dispatch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `subagent-driven-development` to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Dispatch mixed `sandboxed_task` and `agent_task` case episodes from the server under its single semaphore and replay their persisted dispatch/settlement order.

**Architecture:** Extract the synchronous, impure plan-run lifecycle from the CLI into one workspace runner crate. The server derives ready items from the pure planner reducer, assigns each executable episode a permit, and is the only writer of ordered case lifecycle events. The CLI remains a serial facade over the same runner primitives.

**Tech Stack:** Rust 2021, existing Tokio server runtime, existing SEA Forge planner, ledger, sandbox, authority, and agent crates. No third-party dependencies.

## Global Constraints

- Keep the planner/case reducer synchronous and pure.
- Use `ServerState.semaphore` as the only concurrency cap; add no queue, pool, semaphore, or coordination ledger.
- Commit dispatch order before episode execution and settlement order after it returns.
- Preserve existing CLI and direct `Request::Delegate` behavior.
- Never execute a side effect before its existing authority decision.
- Keep kernel crates free of async and HTTP dependencies.

---

### Task 1: Extract shared synchronous case-runner primitives

**Files:**
- Create: `crates/sea-forge-case-runner/Cargo.toml`
- Create: `crates/sea-forge-case-runner/src/lib.rs`
- Modify: `Cargo.toml`
- Modify: `crates/sea-forge-cli/Cargo.toml`
- Modify: `crates/sea-forge-cli/src/plan_pipeline.rs`
- Test: existing `crates/sea-forge-cli/tests/conformance_m13.rs`

**Consumes:** the synchronous setup, event append, sandbox execution, and completion code currently in `crates/sea-forge-cli/src/plan_pipeline.rs`.

**Produces:** a runner API that initializes a case, derives ready actions through `sea_forge_planner::next_case_actions`, commits a supplied case event, runs one sandboxed episode, and applies one episode completion. It owns no Tokio type or concurrency state.

- [ ] **Step 1: Write a CLI conformance assertion that invokes the unchanged serial plan path through the runner facade.**

```rust
let outcome = sea_forge_cli::plan_pipeline::run_plan(&options).unwrap();
assert_eq!(outcome.state, "completed");
```

- [ ] **Step 2: Run the focused CLI test and confirm it passes before extraction.**

Run: `cargo test -p sea-forge-cli --test conformance_m13 --offline`

Expected: PASS.

- [ ] **Step 3: Create the runner crate and move only impure lifecycle primitives out of `plan_pipeline.rs`.**

```toml
[package]
name = "sea-forge-case-runner"
version.workspace = true
edition.workspace = true

[dependencies]
sea-forge-core = { path = "../sea-forge-core" }
sea-forge-planner = { path = "../sea-forge-planner" }
sea-forge-ledger = { path = "../sea-forge-ledger" }
sea-forge-authority = { path = "../sea-forge-authority" }
sea-forge-sandbox = { path = "../sea-forge-sandbox" }
```

Keep `PlanRunOptions`, `PlanRunOutcome`, `run_plan`, and `run_plan_with_approval_extension` as CLI compatibility functions; delegate their synchronous work to the runner. Do not move reducer types or add async code to the runner.

- [ ] **Step 4: Run focused CLI and runner checks.**

Run: `cargo test -p sea-forge-cli --test conformance_m13 --offline && cargo check -p sea-forge-case-runner --offline`

Expected: PASS.

- [ ] **Step 5: Commit the extraction.**

```sh
git add Cargo.toml crates/sea-forge-case-runner crates/sea-forge-cli
git commit -m "refactor: extract synchronous case runner"
```

### Task 2: Bind agent delegation to a planned episode

**Files:**
- Modify: `crates/sea-forge-server/src/delegation.rs`
- Modify: `crates/sea-forge-server/src/lib.rs`
- Test: `crates/sea-forge-server/tests/conformance_m13.rs`

**Consumes:** the runner case ID, item ID, allocated run ID, and case ledger context.

**Produces:** an internal `DelegationEpisodeContext` accepted by `execute_with_control` so planned agent evidence and settlement reference the submitted case/item/run. The direct `Request::Delegate` path creates its existing standalone context.

- [ ] **Step 1: Write a failing conformance assertion for a planned agent episode.**

```rust
assert_eq!(settlement_payload["case_id"], submitted_case_id);
assert_eq!(settlement_payload["item_id"], "agent");
assert_eq!(settlement_payload["run_id"], dispatched_run_id);
```

- [ ] **Step 2: Run the focused test and confirm it fails because delegation generates a new case/item identity.**

Run: `cargo test -p sea-forge-server --test conformance_m13 planned_agent_episode_uses_case_context --offline`

Expected: FAIL on case or item identity.

- [ ] **Step 3: Add the internal episode context and use it for agent authority, evidence, artifact, and settlement paths.**

```rust
struct DelegationEpisodeContext<'a> {
    case_id: &'a str,
    item_id: &'a str,
    run_id: &'a str,
}
```

`execute_with_control` returns its single real completion result. The future dispatcher appends the one case-level completion event; do not retain the synthetic `agent_task_delegated` path for planned episodes.

- [ ] **Step 4: Run the focused test and direct-delegation regression test.**

Run: `cargo test -p sea-forge-server --test conformance_m13 planned_agent_episode_uses_case_context --offline && cargo test -p sea-forge-server --test conformance_m13 t13_3_cancel_one_of_three --offline`

Expected: PASS.

- [ ] **Step 5: Commit the episode context.**

```sh
git add crates/sea-forge-server/src/delegation.rs crates/sea-forge-server/src/lib.rs crates/sea-forge-server/tests/conformance_m13.rs
git commit -m "feat(server): bind delegated episodes to case context"
```

### Task 3: Add the server-owned episode dispatcher

**Files:**
- Create: `crates/sea-forge-server/src/case_dispatch.rs`
- Modify: `crates/sea-forge-server/src/lib.rs`
- Modify: `crates/sea-forge-server/Cargo.toml`
- Test: `crates/sea-forge-server/tests/conformance_m13.rs`

**Consumes:** runner initialization/readiness/completion primitives and context-aware delegation.

**Produces:** `case_dispatch::submit`, called by `Request::Submit`, which derives ready work, obtains one `ServerState.semaphore` permit per episode, executes sandbox work via `spawn_blocking` and agent work through the existing delegation service, then applies completions before deriving further ready work.

- [ ] **Step 1: Write the failing five-episode mixed-load test.**

```rust
let response = handle_request(Request::Submit { payload }, &state).await;
assert_eq!(response["state"], "completed");
assert_eq!(case_events.dispatches().len(), 5);
assert_eq!(case_events.settlements().len(), 5);
assert!(case_events.max_unsettled_dispatches() <= 2);
assert!(case_events.contains_kind("sandboxed_task"));
assert!(case_events.contains_kind("agent_task"));
```

Use a five-item valid plan containing delayed sandbox commands and a local delayed HTTP agent stub. Configure `max_concurrent_runs: 2`. Submit through `Request::Submit`, not `Request::Delegate`.

- [ ] **Step 2: Run the test and confirm it fails because `Submit` invokes one serial CLI subprocess.**

Run: `cargo test -p sea-forge-server --test conformance_m13 t13_2_mixed_episodes_share_server_cap --offline`

Expected: FAIL: no two episodes dispatch before a settlement.

- [ ] **Step 3: Implement `case_dispatch::submit`.**

```rust
while let Some(action) = runner.next_ready_action()? {
    let permit = state.semaphore.clone().acquire_owned().await?;
    runner.record_dispatch(&action, next_dispatch_ordinal)?;
    active.push(spawn_episode(action, permit));
}
while let Some(completion) = active.next().await {
    runner.record_completion(&completion, next_settlement_ordinal)?;
    runner.apply_completion(completion)?;
}
```

Derive ready actions again only after completion. Store active handles only in the dispatcher. The permit lives in the episode task until it returns. Replace the whole-case permit/subprocess branch in `Request::Submit`; retain `Request::Delegate` as a direct compatible endpoint using the same semaphore.

- [ ] **Step 4: Run the T13.2 test and existing server conformance suite.**

Run: `cargo test -p sea-forge-server --test conformance_m13 t13_2_mixed_episodes_share_server_cap --offline && cargo test -p sea-forge-server --test conformance_m13 --offline`

Expected: PASS.

- [ ] **Step 5: Commit the dispatcher.**

```sh
git add crates/sea-forge-server
git commit -m "feat(server): dispatch mixed case episodes under shared cap"
```

### Task 4: Persist and replay dispatch/settlement order

**Files:**
- Modify: `crates/sea-forge-case-runner/src/lib.rs`
- Modify: `crates/sea-forge-cli/src/commands/ledger.rs`
- Modify: `crates/sea-forge-cli/src/main.rs`
- Modify: `crates/sea-forge-cli/tests/conformance_m13.rs`
- Modify: `crates/sea-forge-server/tests/conformance_m13.rs`
- Modify: `.agents/specs/spec-agent-orchestration.md`
- Modify: `.agents/CURRENT_STATUS.md`

**Consumes:** ordered case events emitted by the dispatcher.

**Produces:** additive `dispatch_ordinal` on dispatch/activation event payloads, additive `settlement_ordinal` on settlement payloads, and `sea-forge ledger replay --case <case_id>` output ordered by those persisted records.

- [ ] **Step 1: Write a failing replay assertion.**

```rust
let replay = run_cli(["ledger", "replay", "--case", &case_id]);
assert_eq!(replay.dispatch_settlement_rows(), persisted_rows);
```

- [ ] **Step 2: Run it and confirm `ledger replay` is unavailable.**

Run: `cargo test -p sea-forge-cli --test conformance_m13 t13_2_replay_matches_persisted_order --offline`

Expected: FAIL with unknown `replay` subcommand.

- [ ] **Step 3: Implement additive ordinal fields and replay.**

```rust
struct ReplayRow {
    ordinal: u64,
    phase: &'static str,
    item_id: String,
    run_id: String,
}
```

Load the case plan and ordered `case-events.jsonl`; invoke the existing `replay_case` reducer over the same event sequence; print only persisted dispatch/settlement rows. Reject missing, duplicate, or non-monotonic ordinals. Do not re-execute any episode.

- [ ] **Step 4: Run replay, server, and CLI focused tests.**

Run: `cargo test -p sea-forge-cli --test conformance_m13 t13_2_replay_matches_persisted_order --offline && cargo test -p sea-forge-server --test conformance_m13 t13_2_mixed_episodes_share_server_cap --offline`

Expected: PASS.

- [ ] **Step 5: Update M13 status and commit.**

```sh
git add crates/sea-forge-case-runner crates/sea-forge-cli crates/sea-forge-server .agents/specs/spec-agent-orchestration.md .agents/CURRENT_STATUS.md
git commit -m "feat(ledger): replay mixed episode dispatch ordering"
```

### Task 5: Full proof and handoff

**Files:**
- Modify: `.agents/CURRENT_STATUS.md`

- [ ] **Step 1: Format and inspect the complete diff.**

Run: `cargo fmt --all -- --check && git diff HEAD~4..HEAD --check`

Expected: PASS.

- [ ] **Step 2: Run the required proof suite.**

Run: `cargo test --workspace --offline && devbox run -- just context-check && devbox run -- just no-async-kernel && devbox run -- just proof`

Expected: PASS.

- [ ] **Step 3: Refresh the knowledge graph after the source commits.**

Run the repository incremental Understand Anything update and validate `.ua/meta.json` against the final source commit.

- [ ] **Step 4: Record evidence and residual scope.**

Document the passing count and commands in `.agents/CURRENT_STATUS.md`; mark T13.2 green only if the mixed-load and replay assertions pass. Keep ACP continuation recovery deferred to M16.

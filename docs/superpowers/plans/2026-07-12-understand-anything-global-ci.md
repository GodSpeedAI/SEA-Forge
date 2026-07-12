# Understand Anything Global and CI Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or the active session's approved inline implementation workflow to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide one automatically updated Understand Anything installation for Claude Code, Codex, Antigravity, Gemini, and OpenCode, and refresh sea-rs's committed knowledge graph after pushes to `main`.

**Architecture:** One Egonex checkout supplies all CLI integrations through upstream-native discovery paths. A locked user systemd service updates and verifies it. sea-rs owns deterministic graph validation and a separate Claude Code Action workflow that updates one graph-refresh pull request; Rust CI stays independent.

**Tech Stack:** Bash 5, Git, pnpm 10, Node.js 22, systemd user units, Claude Code Action v1, GitHub Actions, `jq`, and `just`.

## Global Constraints

- Preserve existing uncommitted sea-rs changes and stage only task-owned files.
- Add no sea-rs dependency and leave the Rust kernel unchanged.
- Use `https://github.com/Egonex-AI/Understand-Anything.git` as the sole upstream.
- Keep credentials and model payloads out of files, logs, fixtures, and commits.
- Pin third-party actions to full commit SHAs.
- Report unavailable CLI and remote tests as skipped with reasons.
- Do not push, open a pull request, deploy, or publish without a separate request.

---

### Task 1: Migrate and verify the shared global installation

**Files:**
- Create: `/home/sprime01/.local/libexec/update-understand-anything`
- Create: `/home/sprime01/.config/systemd/user/understand-anything-update.service`
- Create: `/home/sprime01/.config/systemd/user/understand-anything-update.timer`
- Modify: `/home/sprime01/.understand-anything/repo/.git/config`
- Modify through installers: global skill links and Claude plugin registry

**Interfaces:**
- Consumes: the clean installed checkout, `git`, `flock`, `pnpm`, and `node`.
- Produces: `update-understand-anything [--check]`, shared links, native Claude registration, and a daily timer.

- [ ] **Step 1: Capture the pre-migration state**

Run `git -C ~/.understand-anything/repo status --short --branch`, `remote -v`, `describe --tags --always`, and list all symlinks whose target contains `understand`. Expect a clean v2.7.3-era checkout with the former Lum1104 remote.

- [ ] **Step 2: Create a fail-closed updater**

Write an executable Bash script that accepts only `--check` or no argument. It must acquire `${XDG_RUNTIME_DIR:-/tmp}/understand-anything-update.lock` with `flock -n`; verify a clean checkout; fetch `origin/main`; prove `HEAD` is an ancestor; merge with `--ff-only`; run locked pnpm install, core and skill builds, and focused core and skill tests; then verify:

```text
origin = https://github.com/Egonex-AI/Understand-Anything.git
~/.understand-anything-plugin -> <checkout>/understand-anything-plugin
~/.agents/skills/<each upstream skill> -> <checkout>/understand-anything-plugin/skills/<skill>
~/.gemini/antigravity/skills/understand-anything -> <checkout>/understand-anything-plugin/skills
packages/core/dist/index.js exists
```

`--check` performs only those verifications. Dirty, divergent, missing, or incorrect state exits nonzero with a concrete diagnostic.

- [ ] **Step 3: Prove pre-migration check failure**

Run `~/.local/libexec/update-understand-anything --check`. Expect nonzero because the remote still names Lum1104.

- [ ] **Step 4: Migrate and install all adapters**

Set the remote to Egonex, fast-forward, then run `install.sh` separately for `codex`, `gemini`, `opencode`, and `antigravity`. Run locked install, builds, and focused upstream tests. Expect repeated per-skill linking to be idempotent.

- [ ] **Step 5: Install Claude natively**

Use Claude Code's marketplace commands for `Egonex-AI/Understand-Anything` and install `understand-anything`. Verify its installed plugin registry without printing unrelated configuration or secrets.

- [ ] **Step 6: Create and enable user units**

The oneshot service executes `%h/.local/libexec/update-understand-anything` after `network-online.target`. The timer uses `OnCalendar=daily`, `Persistent=true`, `RandomizedDelaySec=2h`, and `WantedBy=timers.target`. Run daemon-reload, enable `--now`, `systemd-analyze verify`, updater `--check`, and `systemctl --user list-timers`. Expect a valid next-run timestamp.

### Task 2: Add deterministic sea-rs graph validation

**Files:**
- Create: `scripts/validate-understand-anything.sh`
- Create: `scripts/tests/validate-understand-anything.sh`
- Modify: `justfile`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: explicit graph directory or auto-detected `.ua`/legacy directory, `jq`, and Git.
- Produces: `scripts/validate-understand-anything.sh [graph-dir]` and `just understand-validate`.

- [ ] **Step 1: Write the failing fixture test**

Create a shell test that builds a temporary `.ua` with a one-node, zero-edge graph, metadata containing a 40-character commit hash, an empty fingerprint object, and `{"autoUpdate":true,"outputLanguage":"en"}`. Assert the validator accepts it. Replace the graph with an edge from a missing node and assert rejection.

- [ ] **Step 2: Confirm the red state**

Run `bash scripts/tests/validate-understand-anything.sh`. Expect failure because the validator does not exist.

- [ ] **Step 3: Implement minimal validation**

The executable validator must require nonempty `knowledge-graph.json`, `meta.json`, `fingerprints.json`, and `config.json`; parse each with `jq`; require node and edge arrays; reject duplicate node IDs and dangling edge sources or targets; require `autoUpdate=true` and English output. It prints one success line and exposes no graph contents.

- [ ] **Step 4: Confirm the green state**

Run the focused shell test. Expect its valid fixture to pass, its corrupt fixture to fail internally, and the test process to exit zero.

- [ ] **Step 5: Add command and ignore boundaries**

Add `just understand-validate` under the quality group. Ignore only `intermediate/`, `tmp/`, `.trash-*/`, and `diff-overlay.json` beneath both `.ua/` and the legacy directory; durable graph artifacts remain trackable.

- [ ] **Step 6: Verify and commit**

Run the focused test and `git diff --check`, stage only the four task files, and commit `ci: validate understand anything graphs`.

### Task 3: Generate and review the initial sea-rs graph

**Files:**
- Create: `.ua/.understandignore`
- Create: `.ua/knowledge-graph.json`
- Create: `.ua/meta.json`
- Create: `.ua/fingerprints.json`
- Create: `.ua/config.json`
- Create if produced: `.ua/domain-graph.json`

**Interfaces:**
- Consumes: the updated `understand` skill, sea-rs, full Git history, and English configuration.
- Produces: a reviewed baseline for incremental refreshes.

- [ ] **Step 1: Start the required full run**

Invoke the active host's native equivalent of `understand --full --auto-update --language en`. Expect it to create `.ua/.understandignore` and pause at its mandatory review gate.

- [ ] **Step 2: Review and approve `.understandignore`**

Keep crates, manifests, instructions, specs, plans, scripts, and workflows in scope. Exclude build/runtime output, logs, and scratch data. Compare the file with `rg --files -g '!target/**' -g '!.git/**'`, then confirm the upstream gate.

- [ ] **Step 3: Complete all upstream phases**

Require scan, batching, parallel analysis, assembly review, architecture, tour, graph review, and cleanup to finish. Preserve and report warnings. Expect all four required durable files.

- [ ] **Step 4: Validate and inspect**

Run `just understand-validate`, print only node/edge counts with `jq`, check JSON sizes, and inspect `git status` for scratch leakage. Expect nonzero nodes and no scratch artifacts.

- [ ] **Step 5: Commit the baseline**

Stage only reviewed durable `.ua` files, run `git diff --cached --check`, and commit `docs: add sea-rs knowledge graph`.

### Task 4: Refresh the graph after pushes to main

**Files:**
- Create: `.github/workflows/understand-anything.yml`
- Modify: `docs/ci-cd-architecture.md`

**Interfaces:**
- Consumes: `ANTHROPIC_API_KEY`, dedicated `UA_REFRESH_TOKEN`, the baseline graph, and Claude Code Action SHA `e90deca47693f9457b72f2b53c17d7c445a87342` (peeled v1 tag observed 2026-07-12).
- Produces: one `automation/understand-anything` branch and refresh PR after trusted main pushes.

- [ ] **Step 1: Create the workflow**

Define `push` on `main` with graph-directory `paths-ignore`, plus manual dispatch. Use one concurrency group with cancellation. Grant read-only top-level permissions and job-scoped contents/pull-request writes. Checkout full history using the existing pinned checkout SHA and `UA_REFRESH_TOKEN`.

Run pinned Claude Code Action automation with the Egonex marketplace and plugin. Its prompt must request an incremental English auto-update, prohibit edits outside the existing graph directory, and prohibit commit/push. Allow only Read/Write/Edit/Glob/Grep plus narrowly scoped Git-read, Node, pnpm, and Python commands. Cap turns and job time.

After generation, run the deterministic validator and reject every changed path outside `.ua/` or the legacy directory. If changes exist, configure `sea-forge-graph-bot`, reset the stable automation branch, commit only graph files, push with `--force-with-lease`, and use `gh pr view || gh pr create`. Skip graph-bot events to prevent recursion.

- [ ] **Step 2: Document operation and recovery**

Document both secrets, their distinct scopes and rotation, stable branch/PR behavior, the non-required relationship to Rust CI, manual rerun, and the rule that the last merged graph remains usable after failure.

- [ ] **Step 3: Validate locally**

Parse YAML, reject floating action refs, reject `pull_request_target`, scan for secret echo patterns, and run `git diff --check`. Expect all checks to pass.

- [ ] **Step 4: Commit the workflow slice**

Stage only the workflow and CI/CD documentation; commit `ci: refresh knowledge graph after main pushes`.

### Task 5: Review, verify, and hand off

**Files:**
- Modify: `.agents/CURRENT_STATUS.md`
- Modify only if warranted: `.agents/OBSERVED_DEBT.md`
- Modify only if warranted: `.agents/LESSONS.md`

**Interfaces:**
- Consumes: completed installation, validator, graph, workflow, and task commits.
- Produces: accurate handoff and evidence-backed completion report.

- [ ] **Step 1: Review all owned diffs**

Review `main...HEAD` for correctness, security, compatibility, unnecessary complexity, action pinning, secret scope, loop prevention, and separation from Rust CI. Confirm unrelated pre-existing changes remain intact.

- [ ] **Step 2: Run focused and required checks**

Run the updater `--check`, focused validator test, `just understand-validate`, `devbox run -- just context-check`, `devbox run -- just check`, and `devbox run -- just test`. Report missing Gemini or Antigravity executables as skipped runtime discovery tests.

- [ ] **Step 3: Refresh local memory**

Update `CURRENT_STATUS.md` with objective, commits, files, verification, remaining secret setup, remote workflow status, and skipped tests. Add debt or lessons only when they meet repository rules.

- [ ] **Step 4: Commit memory**

Run context-check and whitespace checks, stage only qualifying `.agents` updates, and commit `docs: record understand anything integration`.

- [ ] **Step 5: Report activation requirements**

State that repository administrators must configure `ANTHROPIC_API_KEY` and `UA_REFRESH_TOKEN`, and that the push workflow remains untested remotely until the user authorizes a push or PR.

# Deep Project Security, Correctness, and Reliability Audit

## Scope and Context

| Item | Value |
| --- | --- |
| Repository | `/home/sprime01/projects/sea-rs` (SEA Forge) |
| Revision audited | `df80d73` (working tree; untracked `.jolli/` runtime files and pre-existing `cc*.res`/`cc*.cdtor.c` debris present, dated Aug 3 — not produced by this audit) |
| Date | 2026-08-14 |
| Environment | Linux, rustc 1.92.0, cargo 1.92.0, just; Landlock-capable kernel |
| Audit type | Read-only defensive audit. No production implementation code was modified. |

**Verification harness retained at** `.agents/reports/audit-harness-2026-08-14/` (standalone cargo project with path-deps into the repo; `main.rs` reproduces findings F-01, F-04, F-05, F-06, F-14, F-17; `tests/server_d1.rs` reproduces F-02 and F-03 via `sea_forge_server::handle_request`). Build with `CARGO_TARGET_DIR=<outside-repo> cargo run` / `cargo test`.

### Tools and commands used

- Static: multi-pass source review of all 22 crates (~72k lines Rust), workbench Tauri host + renderer, `justfile`, CI workflows, `Cargo.lock`s, `devbox.json`.
- Dynamic: the verification harness above (compiles `sea-forge-authority`, `sea-forge-sandbox`, `sea-forge-ledger`, `sea-forge-settlement`, `sea-forge-server`, `sea-forge-core` from the working tree).
- `cargo test --workspace --all-features --locked --no-fail-fast` (full suite re-run; result in "Suite Health" below).

### Major components inspected

Authority engine (policy load/validate/matching/grants), sandbox (local + Landlock jail, `safe_join` family), runtime, ledger (hash chain, MMR, checkpoints, ed25519 signing), trace/evidence/capability/settlement, planner + case engine, case-runner, CLI (run/resume/case/ledger/recall/pipeline), server (NDJSON transport, dispatcher, delegation/ACP, approvals, SFWP views, identity gate), agent (ACP session pump), cell/bundle (federation import/export), thoth, self-model, spec-pipeline/domainforge, workbench (Tauri host bridge, supervisor, drafts, CSP, capabilities), CI/CD, supply chain.

### Areas not adequately inspected / verification limitations

- No OS-level probing of Landlock enforcement on this host beyond what the crate's own conformance tests assert (kernel version not validated for ABI v4 network rights).
- `workbench-e2e-real` (packaged Tauri/WebKit) and release packaging were not executed.
- Windows sandbox architecture (`.agents/reports/2026-07-31-...` covers it) was not re-audited.
- Cargo-audit/cargo-deny/gitleaks were not re-run (CI runs them; lockfiles reviewed statically — 0 git deps, SHA-pinned actions).
- Thoth/self-model/spec-pipeline received lighter coverage than the authority/server/ledger core.

---

## Findings

Severity reflects impact × reachability under the project's own threat model (local operator trusted; agent/child output, plan payloads, ledger bytes on disk, and renderer JS context are untrusted; same-uid clients of the server socket are semi-trusted but must not violate governance boundaries).

---

### F-01 — Authority `deny_write` hard-boundary bypass via non-canonical path spellings (`//`, `.` segments); nested `.env*`/`.git/**` built-ins anchored at root only

**Classification:** Security (authorization boundary / spec §10.2 hard boundary, V5, P4b)
**Severity:** High (defense bypass of a specified hard boundary; blast radius contained to per-run workspace paths)
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-authority/src/lib.rs:2888-2901` (`invalid_relative_path` — does not reject `Component::CurDir`; `Path::components()` collapses `//`), `:2906-2914` (`path_denied` — byte-glob over the raw string), `:2748-2762` (`hard_denied` built-ins).
- `crates/sea-forge-sandbox/src/lib.rs:157-190` (`validate_relative_path` — same permissive charset/components check), `:249-297` (`safe_join`/`checked_parent` skip `CurDir`, OS collapses `//` at open).
- `crates/sea-forge-planner/src/case_engine.rs` (`valid_relative_path` on plan WriteFile paths — accepts the same spellings).

**Preconditions**
A plan/operation author (server `submit` client, extension, future agent-authored plan, or any library consumer of `PolicyAuthorityEngine`) supplies a `write_file` path with `//` or `.` segments, or writes a secret/git path nested below the workspace root. An allow rule with `path_prefix: ""` (the shipped demo policy's `allow-model-write`) must exist — it does by default.

**Technical cause**
The deny check globs the **raw string** (`path_denied`), while the write side **normalizes** the same path lexically (Rust `components()` collapses `//` and mid-path `.`; the OS collapses the rest at open). `invalid_relative_path` fails to reject ambiguous spellings, so non-canonical forms reach the matcher and miss every deny pattern. Separately, built-ins `.env*` and `.git/**` are root-anchored (no `**/` fallback is tried because they don't start with `**/`), so `subdir/.env` and `sub/.git/config` match nothing. The codebase already contains the correct primitive — `safe_lexical_join` (sandbox lib.rs:214-247) explicitly rejects `a//b` and `a/./b` "so `a//b` and `a/b` cannot alias" — but authority's deny evaluation does not use it.

**Execution/data path**
Client plan JSON → `validate_proposal` (accepts `src//gen//model.rs`) → authority `evaluate` → `hard_denied` glob misses → `allow-model-write` (`path_prefix: ""`) matches → verdict **Allow** → grant → `sandbox::materialize`/`safe_join` accepts (same permissive validator) → file lands at `<run>/workspace/src/gen/model.rs` — exactly the location the `**/src/gen/**` boundary exists to protect.

**Violated invariant**
Spec §10.2: "Direct generated-zone writes, `.git` writes, secret/env writes … MUST NOT be made allowable by a generic rule." Spec §7.5: path comparisons happen after lexical normalization. Spec V5/P4b proof property.

**Evidence (harness `main.rs` T1/T2, executed against the built engine with the repo's own demo-style policy)**
```
src/gen/model.rs   => Deny    (control — boundary works canonically)
src//gen//model.rs => Allow   << BYPASS of **/src/gen/**
src/./gen/model.rs => Allow   << BYPASS
subdir/.env        => Allow   << BYPASS of .env*
sub/.git/config    => Allow   << BYPASS of .git/**
safe_join(root, "src//gen//model.rs") → Ok; file landed at root/src/gen/model.rs
```

**Counter-evidence considered**
Charset gate runs before globs (no unicode/backslash tricks); `..`/absolute paths are correctly rejected as `unclassified` deny; matching is case-folded; the demo planner's own templates emit canonical paths only; writes are confined to the per-run workspace (a repo-level `src/gen` is not directly reachable). None of these defeat the spelling bypass itself.

**Recommended remediation**
Reject ambiguous spellings (`//`, `.` segments, trailing `/`) in `invalid_relative_path` — reuse `safe_lexical_join`'s normalization rules — and/or normalize the path before glob matching in `path_denied`. Change built-ins to `**/.env*`, `**/.git/**` (and consider `**/src/gen/**`-style fallbacks for all root-anchored patterns). Also fix the allow-side `path_prefix` (lib.rs:2559-2562) to be segment-aware — a prefix `docs` currently also authorizes `docs-private/x`.

**Regression test**
Engine-level: `src//gen//model.rs`, `src/./gen/model.rs`, `subdir/.env`, `sub/.git/config` under an empty-prefix allow rule must yield Deny with hard-boundary reason codes (mirror of harness T1). End-to-end: V5 variation with non-canonical spellings must leave the generated zone unwritten.

---

### F-02 — Server dispatcher: mid-batch error drops the episode `JoinSet` → activated episode never settles; case stranded `Active` with no recovery path

**Classification:** Correctness / reliability / durability (ledger truth invariant)
**Severity:** High
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-server/src/case_dispatch.rs:207-219` (`return Err` on non-executable `Activate` inside the action loop while `active: JoinSet` may hold spawned episodes), plus every `?` early-return in the loop (e.g. `append_event`/`write_json` failures at :107-116, :231, :246-263) and the join-error mapping at the drain sites.
- Contrast with the correct drain-then-return at `TerminateCase` (:157-187) and `ParkHumanTask` (:355-374).

**Preconditions**
A client `submit` plan mixing an executable item ordered before a non-manual non-executable item (e.g. `[SandboxedTask, Stage]`) — accepted by `validate_proposal` (Stage with empty operations passes); or any I/O failure/episode panic mid-loop after an episode spawned.

**Technical cause**
`ItemActivated` (with `run_id`) is durably committed **before** the episode spawns (correct ordering). When a later action in the same batch errors, the function returns through the `JoinSet`'s drop, which **aborts all in-flight episode tasks** (tokio `JoinSet::drop` aborts). Their settlements are never recorded. `recover_cancelled_delegations` (lib.rs:261-504) only recovers `control_request`/`permission_request` records — a plain aborted episode has neither. The case remains `Active` forever; `run.get` shows `Unsettled`; no reopen path exists for `Active` server cases.

**Evidence (harness `tests/server_d1.rs::mid_dispatch_error_aborts_spawned_episode_and_loses_settlement`, executed via `handle_request`)**
```
submit response: {"error":"non_executable item kind cannot be dispatched: Stage"}
case event kinds: ["case_created", "item_activated"]   ← task_a WAS activated first
settlement_recorded=0                                    ← its settlement is lost forever
case.json state: "active"                                ← stranded; no recovery verb applies
```
The existing regression `t13_2_non_executable_activation_returns_typed_error` uses a **single-item** plan (`active` empty at the error), which is why this survived review — the CURRENT_STATUS entry claims the rejection happens "before any side effect", but it is only before *that item's* side effect.

**Violated invariant**
Every activated episode receives a terminal settlement; JSONL truth is complete for allow, denial, escalation, failure, and timeout (AGENTS.md: "Record complete outcomes").

**Impact**
Case-engine availability (case stuck), evidence completeness (activated-but-unsettled episode), and — for sandbox episodes whose `spawn_blocking` closure already started — an executed side effect whose result is discarded and unrecorded.

**Counter-evidence considered**
Permit accounting is correct (no leak); the error is typed and surfaced to the client; `TerminateCase`/`ParkHumanTask` paths drain correctly; the single-item conformance test passes legitimately. The defect is exclusively the early-return paths that bypass draining.

**Recommended remediation**
Every error exit from the dispatch loop after the first spawn must first drain `active` and record terminal settlements (or convert mid-loop errors into per-episode rejected settlements), matching the `TerminateCase` pattern. Alternatively, pre-validate all actions in a batch (including dispatchability of non-executable kinds) before any spawn occurs.

**Regression test**
Mixed plan `[SandboxedTask, Stage-no-manual]` via `submit`: expect typed error AND a terminal settlement for the activated episode AND case state not left `active` (this test, verbatim, is in the retained harness).

---

### F-03 — Escalated required item terminates its case while the approval it opened stays `Pending` forever; escalation traced as `ItemFailed`

**Classification:** Correctness (state machine / governance semantics)
**Severity:** Medium-High
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-server/src/case_dispatch.rs:668-738` (escalation commits a `Pending` `ApprovalRequest`), `:396-437` (`record_completion`) with `crates/sea-forge-case-runner/src/lib.rs:122-134` (`Escalated != Accepted ⇒ ItemFailed`), `crates/sea-forge-planner/src/case_engine.rs:505-510` (`required_item_failed ⇒ TerminateCase`).

**Preconditions**
Plan item with `markers.required: true` (client-controlled field) under a policy verdict `escalate` for its operation.

**Technical cause**
`settle()` returns `Escalated`; the case runner maps any non-Accepted status to `ItemFailed`; the reducer sees a required failed item and terminates the case. The just-opened approval remains `Pending` in `approvals.jsonl`; approving it can never un-terminate the case (no post-approval re-drive exists in the server; the separate CLI `resume` flow is not connected). Additionally, `ItemFailed` misrepresents an intentional escalation as a failure in the trace.

**Evidence (harness `tests/server_d1.rs::escalated_required_item_terminates_case_with_orphaned_approval`, executed)**
```
case event kinds: ["case_created","item_activated","settlement_recorded","item_failed","case_terminated"]
approval requests: 1, approval status: "pending"
case.json state: "terminated" (exit_code 3)
```

**Counter-evidence considered**
The ACP/agent permission-escalation path is different and correct (`mediate` suspends on the broker). Non-required escalated items park correctly. This is specific to the sandbox-task escalation path.

**Recommended remediation**
Give `Escalated` its own terminal-item semantics: park the item (or the case) as `awaiting_approval` instead of `ItemFailed`+`TerminateCase`, so approval resolution can continue the case; or make approval resolution re-drive terminated-because-of-escalation cases.

**Regression test**
Required SandboxedTask + escalate policy → case must be parked/awaiting, not terminated; after `approve`, the case must proceed; trace must not label the escalation `item_failed`.

---

### F-04 — Ledger crash-consistency: MMR persisted before the log record; the desync state is permanently unverifiable and unrepairable

**Classification:** Reliability / data integrity (durability)
**Severity:** Medium-High
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-ledger/src/types.rs:1027-1029` (`append_under_lock`: `save_mmr(&mmr_state)?; self.write_entry(&entry)?;` — wrong order for crash recovery), `:1299-1335` (`quarantine_incomplete_tail` repairs only `entries.jsonl`, never rolls back `mmr.json`, and is only test-invoked today).

**Preconditions**
Process death (kill -9 / power loss) in the window between `save_mmr` and `write_entry` (both fsync'd, so the window is small but real), or any state where `mmr.json` holds more leaves than `entries.jsonl` has records.

**Technical cause**
`save_mmr` is atomic (tmp+rename) and durable; `write_entry` follows. A crash between them leaves `mmr.json` with N+1 leaves and `entries.jsonl` with N records. `verify()` replays the MMR from entries and compares to the stored state → permanent `ledger_integrity_error: mmr root mismatch`. The repair tool only quarantines torn **entries** lines; it cannot roll **mmr.json** back. Because `save_mmr` is durable-before-write, the inverse state (entries ahead of mmr) is the impossible one — the ordering chose the unrecoverable direction.

**Evidence (harness `main.rs` T5, executed)**
```
verify() after desync:      Err("ledger_integrity_error: mmr root mismatch")
quarantine_incomplete_tail: Ok(0)                       ← repairs nothing (entries intact)
verify() after quarantine:  Err("ledger_integrity_error: mmr root mismatch")  ← permanent
```

**Violated invariant**
R3-class recovery: after crash, the persisted prefix must remain valid/verifiable (spec §14.3, §17.2 R3).

**Counter-evidence considered**
`write_entry` itself is flush+fsync+dir-fsync per record (stronger than required); the flock discipline around read-modify-append is correct across threads and processes; both conformance tests for truncation/reordering pass. The defect is solely the cross-file commit ordering plus the missing rollback path.

**Recommended remediation**
Write `entries.jsonl` first (single source of truth), then `mmr.json` (derived cache), and make `verify()`/quarantine treat `mmr.json` as rebuildable derived state: on mismatch with a longer MMR, rebuild MMR from entries (or roll back the extra leaf) rather than failing forever.

**Regression test**
Simulate the crash window (entries with N records + mmr.json from N+1 append): after `quarantine_incomplete_tail` (or a new repair fn), `verify()` must return Ok.

---

### F-05 — `prove_entry` panics (slice out of range) when `entries.jsonl` is shorter than `mmr.json`'s leaf count

**Classification:** Reliability (crash-on-corruption instead of typed error)
**Severity:** Medium
**Confidence:** High (verified by execution)

**Affected components**
`crates/sea-forge-ledger/src/types.rs:916-953` — bounds are checked against `mmr.leaf_count`, but the slice indexes `entries` re-read from disk; `entries[start..start+size]` panics when `entries.len() < mmr.leaf_count` (exactly the F-04 state, or manual truncation with `mmr.json` intact). Reachable from `sea-forge ledger prove` (CLI, disk-derived input).

**Evidence (harness `main.rs` T4)**
```
panicked at crates/sea-forge-ledger/src/types.rs:953:42:
range end index 4 out of range for slice of length 2
```

**Recommended remediation**
Validate `entries.len()` against `mmr.leaf_count` before slicing; return `ForgeError::Internal("ledger_integrity_error: ...")`. Regression: the harness T4 setup must return a typed error, not panic.

---

### F-06 — ed25519 signature verification panics on signature strings containing characters ≥ U+0100

**Classification:** Reliability (crash-on-corruption on the integrity-verification path)
**Severity:** Medium
**Confidence:** High (verified by execution)

**Affected components**
`crates/sea-forge-ledger/src/signing.rs:114` — `let idx = ALPHABET[c as usize];` with `ALPHABET: [i8; 256]`; any code point ≥ 0x100 indexes past the table. Reached from `verify_signature` ← `verify_checkpoints` / `verify_witness_receipts` on tampered/corrupted on-disk ledger data — the exact scenario verification exists to detect. (Note: `é` U+00E9 is in-range and correctly returns a typed error; only ≥ U+0100 panics.)

**Evidence (harness `main.rs` T3)**
```
sig="ed25519:中"     => PANIC index out of bounds: len 256, index 20013
sig="ed25519:AAAAĀ"  => PANIC index 256
```

**Recommended remediation**
`if (c as u32) >= 256 || ALPHABET[c as usize] < 0 { return Err(...) }`. Regression: call `verify_signature` with ≥U+0100 signatures; expect typed error.

---

### F-07 — `fire_notify` can deadlock a tokio worker: piped stdout/stderr are never drained; no timeout; runs synchronously on the async runtime

**Classification:** Reliability / availability
**Severity:** Medium
**Confidence:** High (verified by code reading; not executed — requires a chatty notify hook)

**Affected components**
`crates/sea-forge-server/src/lib.rs:2587-2604`, called from async `commit_plan` (:1993-2001).

**Technical cause**
`stdin(piped).stdout(piped).stderr(piped)`; the event is written to stdin; then `let _ = child.wait();`. If the hook writes ≥ pipe capacity (~64 KiB) to stdout/stderr it blocks on write while the parent blocks in `wait()` — permanent deadlock of a worker thread; the `case.commit` never completes; `request.get_status` stays pending. Even without deadlock, a hung hook parks a worker synchronously (no `spawn_blocking`, no timeout). Stdin write errors are also silently discarded (`let _ =`).

**Preconditions**
Operator-configured `notify_command` that is chatty or hangs. Not directly attacker-triggerable, but a client `case.commit` triggers the hook.

**Recommended remediation**
Drain stdout/stderr concurrently (or use `Stdio::null()` if output is unused), enforce a timeout, run on `spawn_blocking`. Regression: notify hook writing 1 MiB to stdout must not hang commit; a hook sleeping forever must time out.

---

### F-08 — Server discards the verified actor role: every authority evaluation hardcodes `ActorRole::Operator`

**Classification:** Security (role separation ineffective) / correctness
**Severity:** Medium
**Confidence:** High (verified by code reading)

**Affected components**
Identity gate verifies the claimed role (`identity.rs:382-387`) but forwards only `actor_id`; all evaluation sites construct `Actor { actor_id: entity.into(), role: ActorRole::Operator }` — `lib.rs:2481-2484` (record_cancellation), `case_dispatch.rs:613-616` (execute_sandbox), `delegation.rs:335-338`, `agent_probe.rs:186-189`.

**Impact**
An actor bound with only e.g. `security_officer` passes the gate holding that role, but every action they trigger is evaluated **as Operator**. Role-keyed policies (`actor_role: operator`) cannot distinguish principals server-side; SoD role checks gate nothing meaningful downstream. Actor-**id**-keyed identity bindings still bind correctly (limits impact to mis-designed role separation, not full bypass). Renderer-authored roles remain properly rejected (typed enum, fail-closed).

**Recommended remediation**
Propagate the verified `ActorRole` from the identity gate into every `Actor` construction. Regression: bind an actor with a non-operator role and a policy whose only rule targets that role; the action must be allowed for the correct role and denied/escalated when evaluated as Operator.

---

### F-09 — Workbench drafts: unsanitized `draft_id` path traversal (arbitrary `.json` write/delete outside the drafts dir)

**Classification:** Security (defense-in-depth against renderer compromise)
**Severity:** Medium
**Confidence:** High (verified by code reading)

**Affected components**
`workbench/apps/desktop/src-tauri/src/drafts.rs:44-46` (`drafts_dir().join(format!("{draft_id}.json"))` — no charset/canonicalize check), reached from Tauri commands `draft_save`/`draft_load`/`draft_delete` (`bridge.rs:512-546`).

**Preconditions**
A compromised renderer context (malcent npm dependency; strict CSP `script-src 'self'` makes XSS hard). The bridge's own docs treat "a renderer bug or a compromised web context" as in-scope (bridge.rs:380).

**Impact**
`draft_id = "../../../home/<user>/.config/foo"` writes an arbitrary-content `state: Value` inside a `Draft` envelope to any `.json`-suffixed path the user can write (clobber other apps' JSON configs), or deletes arbitrary `.json` files (idempotent remove). Not arbitrary bytes/paths.

**Counter-evidence considered**
Tauri capabilities are minimal (no fs/shell/http plugins — verified); this is the one host command family that interpolates a renderer string into a path, contradicting the repo's own safe-join mandate.

**Recommended remediation**
Validate `draft_id` against `^[A-Za-z0-9_-]{1,64}$` (or canonicalize + prefix-check) inside `drafts.rs`. Regression: traversal-shaped ids must be refused by the host, independent of the caller.

---

### F-10 — CLI `resume` fabricates settlement evidence: synthetic `Completed`/`exit 0` `ExecutionResult` plus empty stdout artifacts for write-only items

**Classification:** Correctness (evidence integrity / settlement-from-evidence thesis)
**Severity:** Medium
**Confidence:** High (verified by code reading)

**Affected components**
`crates/sea-forge-cli/src/commands/resume.rs:503-516`.

**Technical cause**
For items whose operations are all `WriteFile` (or empty — which `validate_proposal` accepts, see F-25), resume synthesizes `ExecutionResult { status: Completed, exit_code: Some(0), ... }` and writes empty `artifacts/stdout.txt`/`stderr.txt`. Settlement then evaluates criteria (e.g. `require_exit_zero`) against evidence **no process produced**. The `run_intent` path handles the same shape honestly (`execution: None` → rejected basis). This is precisely the "false success" shape the spec's core thesis (settlement ≠ exit code; evidence-backed outcomes) exists to preclude — and it is recorded in durable evidence files.

**Recommended remediation**
Represent write-only items without a fabricated process result (e.g. an explicit `write_only` settlement path, or `execution: None` semantics), keeping the two flows symmetric. Regression: a write-only plan item via `resume` must not produce an `ExecutionResult` claiming a completed process.

---

### F-11 — Stranded `Active` cases have no recovery path (CLI mirror of F-02)

**Classification:** Reliability / operability
**Severity:** Medium
**Confidence:** High (verified by code reading)

**Affected components**
`crates/sea-forge-cli/src/commands/resume.rs:187-195` (only `awaiting_approval` can resume; and it flips state to Active *before* the loop at :357-359), `:416-423` (ItemActivated appended before execution), `crates/sea-forge-cli/src/commands/case.rs:72-82` (`reopen` accepts only `Completed | Terminated`).

**Technical cause / impact**
A crash (or kill -9) after activation but before completion leaves the item `Active` in replay; `next_case_actions` returns empty → resume exits 5 forever; resume refuses; reopen refuses. Recovery requires manual ledger surgery. Combined with F-02 (server-side stranded `Active`) this is a systemic "no recovery from the most common crash state" gap. Non-durable `case.json` writes (tmp+rename without fsync, `pipeline.rs:131-136`, `case.rs:29-34`) widen the stale-state window.

**Recommended remediation**
Add a recovery mode: allow `resume` (or a `case repair`) on `Active` cases by replaying the ledger, terminal-settling activated-but-unsettled episodes as `rejected: interrupted`, and continuing. Regression: kill the CLI mid-episode; the documented repair command must return the case to a drivable state.

---

### F-12 — Cell/root convention split (`.sea-forge/.sea-forge/...` double-nesting; mixed conventions in thoth; silently-empty exports)

**Classification:** Correctness (cross-crate contract)
**Severity:** Medium
**Confidence:** Medium-High (verified by code reading; runtime behavior inferred from the two conventions)

**Affected components**
Two conventions under one `--root` (default `.sea-forge`): *state-root* crates join directly (`root/runs`, `root/cases`, `root/ledgers` — pipeline, ledger, case-runner) vs *project-root* crates prepend `.sea-forge` (`crates/sea-forge-cell/src/cell.rs:21`, `crates/sea-forge-cell/src/bundle.rs:81,103`, `crates/sea-forge-self-model/src/store.rs:44`, `crates/sea-forge-thoth/src/service.rs:112,119`, server `transcript_seal.rs`).

**Concrete symptoms (statically verified)**
1. `pipeline.rs:798` `sea_forge_cell::ensure(&root).ok()` — with the default root this creates/reads `.sea-forge/.sea-forge/cell.json`, a **second cell identity** distinct from whatever the cell tooling/server maintains; errors are swallowed into `cell_id: None`.
2. Thoth `service.rs` mixes both conventions in one file (policy at `root/authority` vs capabilities at `root/.sea-forge/capabilities.jsonl`): with `--root .sea-forge`, policy resolves but capabilities are invisible → every `ask_capability` answers "not declared" — fail-closed but wrong.
3. Federation `export` reads run evidence from `root/.sea-forge/runs/...`; invoked with the default state root, `src.exists()` is false for every run → **exports a bundle that silently contains zero run evidence** while reporting success (templates missing produce an error; runs do not).

**Recommended remediation**
Pick one convention (state-root) and thread it through cell/self-model/thoth/bundle; make export fail closed when zero requested run files were found. Regression: `export` with runs present under the default root must include them; `pipeline.rs` cell id must equal the cell tooling's id.

---

### F-13 — Capability promotion matches declarations by substring / `*` → inflated provenance records gating side-effect authority

**Classification:** Correctness (governance metadata)
**Severity:** Medium
**Confidence:** High (verified by code reading)

**Affected components**
`crates/sea-forge-capability/src/promotion.rs:183-191` — `d.plan_item_id.contains(capability_name) || capability_name == "*"`; contrast envelopes matched exactly at :196. `require_proven` (:476-503) gates side-effect authority on the record built from this matching. Same substring pattern in `crates/sea-forge-thoth/src/service.rs:143`.

**Impact**
A capability named `test` aggregates declarations from `itm_contest_7`; `"*"` matches the entire declaration store — inflating `min_declarations`/weights/coverage and capable of flipping a capability to `Proven` on unrelated evidence. Overbroad provenance then authorizes `require_proven`-gated side effects.

**Recommended remediation**
Match declarations the way envelopes are matched (exact `attempted_capability` field or exact plan-item mapping); reject `*` or require explicit confirmation. Regression: declaration set with substring-overlapping item ids must not count toward an unrelated capability.

---

### F-14 — `LedgerStream::open` traverses with unvalidated `ledger_id` and creates directories outside the state root

**Classification:** Security (path handling; local operator surface)
**Severity:** Medium-Low
**Confidence:** High (verified by execution)

**Affected components**
`crates/sea-forge-ledger/src/types.rs:491-506` (`root.join("ledgers").join(&ledger_id)` + unconditional `create_dir_all`), reached from CLI `sea-forge ledger verify|prove <ledger_id>` (`commands/ledger.rs`); same unvalidated-id class in `resume.rs:123`, `case.rs:14`, `ledger.rs:57` (`case_id`), and `sea-forge adopt`'s `parse_template_ref`/`cell_id` (`cell/bundle.rs:424-443`, `cell/template.rs:22-25`).

**Evidence (harness `main.rs` T6)**
```
LedgerStream::open(root, "esc/../../outside_ledger", _) → created <root>/outside_ledger (outside root)
```

**Counter-evidence considered**
`inspect.rs:34` and the server's case/run views *do* validate id grammar — the validators exist and are applied inconsistently; server-minted ids are safe. Threat model is local operator, but this violates the repo's own safe-join mandate and is the exact class F-09/F-15 belong to.

**Recommended remediation**
Apply `ids::valid_*` grammar / `safe_lexical_join` to every id joined into a path at crate boundaries. Regression: traversal-shaped ids must be rejected with `ForgeError::Input` before any fs mutation.

---

### F-15 — `template_ref` traversal in `case.preflight`/`case.commit` (unprotected inspect verb reads YAML outside the cell)

**Classification:** Security (information disclosure, limited)
**Severity:** Medium-Low
**Confidence:** High (verified by code reading)

**Affected components**
`crates/sea-forge-server/src/lib.rs:2040-2046` and `sfwp/case.rs:153-177`: `template_ref.split_once('@')` then `root.join("templates").join(format!("{name}@{version}.yaml"))` — `../` in `name` escapes; `case.preflight` is an unprotected inspect verb (any connected same-uid client).

**Impact**
Arbitrary `*.yaml` file load/parse outside the cell with parse errors echoed back (limited content oracle). Contrast: `case.get_overview`/`run.get` validate ids and have traversal tests; this surface has none.

**Recommended remediation**
Validate `name`/`version` charset (e.g. `[A-Za-z0-9_-]`) before joining. Regression: `../`-shaped template refs must be refused.

---

### F-16 — Requester chooses the authority policy bundle path and plan file path

**Classification:** Security (spec-conformance / trust boundary placement)
**Severity:** Medium-Low (contained by same-uid socket trust model)
**Confidence:** High (verified by code reading)

**Affected components**
`SubmitPayload.policy`/`.plan` and equivalents on `CaseCommit`/`Delegate`/`AgentProbe`/`CancelDelegation` (`lib.rs:779-786` etc.); `resolve_policy_path` honors absolute paths (`agent_probe.rs:507-514`); `case_dispatch.rs:37-42` reads the plan from an arbitrary path.

**Impact**
The bundle the *request* names is what authorizes the request's own actions and, on escalation, what gets materialized as the cell's `active-policy.json`. Same-uid clients can already edit the default policy file, so this is boundary hygiene rather than privilege escalation — but it contradicts the repo's own rule ("Validate workspace-relative paths with the specified safe-join algorithm") and lets requests point outside the cell.

**Recommended remediation**
Constrain policy/plan references to workspace-relative paths under the cell root resolved via safe-join, or pin the cell's active policy at server level.

---

### F-17 — `settle()` quarantine write interpolates `plan_item_id` into a filesystem path (latent traversal)

**Classification:** Security (latent / defense-in-depth)
**Severity:** Low-Medium
**Confidence:** High mechanism, Low current reachability

**Affected components**
`crates/sea-forge-settlement/src/lib.rs:88-100` — `q_dir.join(format!("{}.jsonl", claim.plan_item_id))` with no `safe_join`; also non-atomic `fs::write` as a side effect inside the evaluator.

**Evidence (harness `main.rs` T7)** — `plan_item_id = "../../evil_item"` with declared batch criteria wrote `evil_item.jsonl` outside `run_dir`.

**Counter-evidence considered (why latent)**
Current callers derive `plan_item_id` either from the fixed planner templates or from server plans whose ids are validated by `validate_proposal` (`valid_id` ≤128 `[A-Za-z0-9_-]`). No today-reachable path supplies `..` — this is a library-contract hazard awaiting the first API caller.

**Recommended remediation**
Validate `plan_item_id` grammar inside `settle()` before any write. Regression: harness T7 shape must be rejected.

---

### F-18 — ACP episode: no wall-clock cap; unbounded `tool_calls` map; title-less `tool_call` updates bypass `max_turns`

**Classification:** Availability / resource safety (untrusted agent is the boundary)
**Severity:** Low-Medium
**Confidence:** Medium-High (code reading)

**Affected components**
`crates/sea-forge-agent/src/acp.rs:750-836` — the only time bound is `timeout(per_turn_timeout, incoming.recv())` (bounds waiting, not elapsed); an agent streaming a notification every < timeout keeps the episode alive indefinitely (external `cancel` is the sole backstop); `tool_calls: HashMap` retains every `tool_call` update without size bound; `turns_used` increments only on text-bearing updates (`:836`, `:1188-1195`), so title-less `tool_call` updates never count toward `max_turns`.

**Recommended remediation**
Add an episode wall-clock deadline; bound `tool_calls` (and drop it at cap); count tool-call updates toward turn budget. Regression: agent streaming notifications forever must terminate at the episode deadline; N tool-call updates must respect `max_turns`/memory bounds.

---

### F-19 — Client-controlled `timeout` is unclamped (contained chrono panic; effectively-unbounded episodes pinning permits)

**Classification:** Availability / robustness
**Severity:** Low-Medium
**Confidence:** High (code reading)

**Affected components**
`lib.rs:694-695, 785-786` (bare `u64`, default 60) → `case_dispatch.rs:722` `chrono::Duration::seconds(timeout as i64)` (panics ≳ 9.2e15 s; contained by `spawn_blocking` → join-error → rejected settlement) and `ExecutionRequest.timeout_secs` (:763). Extreme values yield near-unbounded episodes each pinning a semaphore permit (with `max_concurrent_runs` small, four such submits starve the server). Clamp at the protocol boundary.

---

### F-20 — Jail `SandboxViolation` classification is a stderr-substring heuristic (child-controlled)

**Classification:** Correctness of evidence classification
**Severity:** Low
**Confidence:** High (code reading)

`crates/sea-forge-sandbox/src/jail.rs:332-338`: nonzero exit + stderr containing "permission denied" (case-insensitive) reclassifies `Completed` as `SandboxViolation`. A child failing for its own permission reasons (e.g., an HTTP 403 "permission denied" from a remote, a user-facing message) is recorded with basis `jail_violation`. Settlement outcome is Rejected either way — only the durable basis/record is wrong. Prefer exit-code/EACCES-signature detection or annotate as `suspected`.

---

### F-21 — Server `status` verb answers from an in-memory map never rebuilt at startup

**Classification:** Correctness / operability
**Severity:** Low
**Confidence:** High (code reading)

`lib.rs:1534-1540`: after a server restart, every pre-restart case answers `"case not found"` on `status` while disk-backed `case.list` still shows it. Rebuild the map (or answer from disk) at startup/rebind.

---

### F-22 — Signing-key hygiene: verify path mints signing keys; 0644-then-chmod window; self-witnessing possible

**Classification:** Security (key management)
**Severity:** Low
**Confidence:** High (code reading)

`crates/sea-forge-ledger/src/signing.rs:20-46`: `load_verifying_key` calls `load_or_create_signing_key` — a **verify** operation creates and persists a new signing key when the file is missing (side effect; also destroys recoverability of the real key); `read_verifying_key` derives the public key from the private key file (no key separation — any verifier can sign); key file written with default perms then chmod 0600 (world-readable window); `fs::metadata(...).unwrap()` (:26) is a TOCTOU panic if the file vanishes; `create_pre_action_assurance` generates witness keys locally, so "independent" witnesses can be self-minted unless callers provision external key dirs. Fix: create keys with `OpenOptions` mode 0600, split verify (public-key-only) from sign paths, symlink-check key paths.

---

### F-23 — Thoth self-disclosure over-denies: actor **id** passed where grants are keyed by actor **role**

**Classification:** Correctness (fail-closed but wrong answers)
**Severity:** Low
**Confidence:** High (code reading)

`crates/sea-forge-thoth/src/service.rs:241` stores `actor_role: actor_id.to_string()`; `permits` (:212-215) passes the actor id into `SelfDisclosureSurface::permits(actor_role, …)` (`authority lib.rs:759-763`), which matches `grant.actor_role == actor_role`. Grants authored by role (`operator`) never match actor ids (`operator_local`) → permitted disclosures are denied. Fix: map the actor's verified role before matching; regression: a role-authored grant must permit a bound actor holding that role.

---

### F-24 — Blocking-in-async and O(n)-per-append ledger costs (availability under load)

**Classification:** Availability / performance
**Severity:** Low (Medium at scale)
**Confidence:** High (code reading)

- Sync `flock`+`fsync` and full-file `read_last_entry()` per append run directly on tokio workers (`case_dispatch.rs:107-116` et al.; `types.rs:706-778`); `publish_event` holds the async `events_ledger` mutex across the blocking append (`lib.rs:153-156`); `receiver.recv()` blocking-recv on a worker (`delegation.rs:1236`); `git` subprocess + file walks inline in async ACP paths (`delegation.rs:1288-1400`).
- Events ledger grows unbounded and is fully re-read per `events.subscribe`/`get_range`/`publish`; `next_ordinal` is an O(n) scan per event (self-noted). `Precondition.records` unbounded → up to ~15k full ledger scans from one 1 MiB request.
- N concurrent requests on slow/contended disk (flock contention is real: the spawned CLI appends to the same case ledger) can stall all worker threads including accepts.

Fix shapes: `spawn_blocking` for fs/subprocess work; incremental tail-state for appends; cursored/paged event storage; caps on precondition records.

---

### F-25 — Lower-tier hardening items (each real, individually small)

| # | Item | Where |
| --- | --- | --- |
| a | `bunfig.toml` sets `frozenLockfile = false` (recipes pass `--frozen-lockfile`, but bare `bun install` mutates the lock) | `workbench/bunfig.toml:3-4` |
| b | Sidecar inherits full parent env (`Command::env()` adds; no `env_clear` allowlist) | `src-tauri/src/supervisor.rs:513-519` |
| c | Host resolves `sea-forge-server` from `PATH` when no sibling sidecar exists | `supervisor.rs:207-214` |
| d | Dev down-recipes kill whatever holds port 1420/6006 (`fuser -k`) | `justfile:489,546` |
| e | `policy_bundle_hash` computed over bundle including `source_base` absolute path → same policy at different paths hashes differently (audit-comparison instability) | `authority lib.rs:1386-1394` vs `:850-856` |
| f | `AuthorityPolicyBundle` lacks `deny_unknown_fields` (all surfaces have it) — misspelled top-level keys silently dropped | `lib.rs:369-408` |
| g | v0.1 implicit **allow** for `recall_memory`/`inspect_run`/`validate_model` when policy is silent (documented legacy, but note `memory_scope` then defaults to `"any"` → unscoped recall) | `authority lib.rs:2104-2121`; `recall.rs:141` |
| h | argv0 identity TOCTOU: check at decision time, string-bound at spawn (swap window decision→spawn) | `authority lib.rs:2492-2503` |
| i | `safe_join` check-then-use symlink race (TOCTOU) between validation and `File::create`/`fs::write` | `sandbox lib.rs:249-355` |
| j | `hard_denied` covers only `WriteFile`; reserved mutators (`delete_file`, `generated_zone_mutation`, …) have no executor yet but also no boundary — register before the first executor lands | `authority lib.rs:2748-2766` |
| k | `LocalSandbox::collect_artifacts` is a stub returning `Ok(vec![])` — silently ignores requested artifacts | `sandbox local.rs` |
| l | Approval-view journal appended with two unwrapped writes, no lock — concurrent `approval.list` can transiently report the journal `unreadable` (self-heals) | `delegation.rs:1573-1580` |
| m | Empty-`operations` SandboxedTask accepted by `validate_proposal`; recursive `has_cycle` (stack risk on ~10⁵-item adversarial plans) | `case_engine.rs:211-392` |
| n | `status`-verb dedupe gaps: no-`request_id` mutations have no idempotency (replayed `submit` creates a second case) | `correlation.rs`; `lib.rs` |
| o | Unknown events cursor silently replays from the beginning (typo → up to 500 stale events) | `events.rs:127-174` |
| p | Agent-approval waits hold the global semaphore permit → N unresolved approvals starve all submits | `delegation.rs:1783-1793` |
| q | `hash_entry` chains onto the stored tail without verifying it (append does not detect a forged predecessor; `verify()` does later) | `types.rs:1379-1381` |
| r | Evidence/trace/capability writers: flush-per-record but **no fsync** (kill-9-safe, not power-loss-safe; asymmetric with ledger discipline); a single non-UTF8 byte fails a whole recall scan | `evidence lib.rs:63-121`; `trace lib.rs:22-57`; `capability lib.rs:61-94` |
| s | Pre-existing `cc*.res` / `cc*.cdtor.c` rustc temp debris at repo root (Aug 3) — consistent with the unresolved "SIGSEGV during `just test`" note in `CURRENT_STATUS.md` (2026-07-22 audit): a killed/hung rustc leaks linker temps into CWD |

---

## Suite Health (verification run)

`cargo test --workspace --all-features --locked --no-fail-fast` re-run on this revision:

**Result: 859 tests passed, 0 failed across 106 test binaries — suite fully green.** The 2026-07-22 note of a suite-context SIGSEGV before `sea-forge-cli` unit output **did not reproduce** (first run during this audit died from `/tmp` exhaustion — environment artifact, not a repo defect). One observation from the run itself: executing the suite **leaked `rustc*/raw-dylibs`/`symbols.o` temp directories into the repository root** (freshly reproduced; identical class to the pre-existing Aug 3 `cc*.res`/`cc*.cdtor.c` debris — evidence points at the server's `run_cli`/SWE-seed child processes running under `env_clear()` with the repo as CWD, e.g. `conformance_m16.rs:267-270` seeding `SEA_FORGE_SWE_SEED_REPO=<repo root>`). Captured as F-25.s; the debris this audit's run created was removed.

---

## Rejected Significant Hypotheses

Investigated and **disproven** — recorded so future audits don't repeat them:

1. **"The mixed-plan dispatcher error fires before any activation"** (initial harness run suggested `item_activated=0`). Disproven: my instrumentation counted uniform `case_event` record kinds instead of payload kinds. With correct instrumentation the activation is recorded first and the episode is aborted — F-02 stands. Conversely the sub-hypothesis "task_a's run dir proves the side effect executed" was **dropped**: the abort in the reproduction happened before any run-dir creation; the durable-record half of the claim is what's verified.
2. **"Signature verification panics on any non-ASCII char"** — only code points ≥ U+0100 panic (out-of-table); in-table chars like `é` return a typed error. Narrowed before reporting (F-06).
3. **"settle() quarantine traversal is unreachable"** — initially appeared non-reproducible; the path math was mine (`quarantine/../x` stays inside `run_dir`). With `../../` it escapes — latent, reported as F-17 with upstream-validation counter-evidence.
4. **"serde_json `preserve_order` breaks authority hash determinism"** — latent only: today's lockfile has no `indexmap`; `sorted()` does sort. Recorded as fragility, not a finding.
5. **"LocalSandbox forwards parent env to children"** — disproven: `env_clear()` + explicit envs (`local.rs:51-58`); minimal-env tests exist.
6. **"Bundle import is zip-slip-vulnerable"** — comprehensively defended: size-capped reads, per-entry hash+size verification, extra-entry rejection, `safe_lexical_join` before first mutation, staging+atomic rename (agent-verified against `bundle.rs:43-327`).
7. **"CI workflows expose `${{ }}` injection / `pull_request_target` risk"** — disproven: SHA-pinned actions, least-priv permissions, OIDC crates.io publishing, no untrusted interpolation into `run:` blocks.
8. **"Tauri capabilities over-expose shell/fs/http to the renderer"** — disproven: `core:default` only, no plugins, closed serde-tagged command enums, host-inserted identity that can only narrow.
9. **"A rejected run appending a capability envelope is a false-success record"** — by design: `capabilities.jsonl` is attempt memory; envelopes faithfully record `result: rejected`; promotion counts rejections.
10. **"Ledger chain verification has off-by-one/skip or sign-verify byte mismatches"** — none found: ordinal density + predecessor + recomputed entry hash + stored-MMR equality is sound across truncation/reorder/duplicate/restore; checkpoint/witness canonicalizations are symmetric between sign and verify.
11. **"run_id/case_id reach filesystem joins unvalidated in server views"** — disproven for `run.get`/`case.get_overview` (27-char grammar + traversal tests exist); the *unvalidated* surfaces are the ones reported (F-14, F-15, F-09).
12. **"Escalation → `SpawnFailed` fabrication still exists"** — fixed historically (conformance_case_episode.rs documents the fix; verified denial/escalation settle correctly in F-03's run: `settlement_recorded` with escalate basis).

---

## Systemic Observations

1. **Raw-string path matching vs normalized filesystem semantics** is the deepest recurring mechanism: F-01 (authority globs), F-09 (draft ids), F-14 (ledger/case/adopt ids), F-15 (template refs), F-17 (plan item ids). The repo has the right primitives (`safe_lexical_join`, `ids::valid_*`) and applies them inconsistently — always at the *outermost* CLI/server boundary of each crate, never inside the enforcing function. The durable fix is a policy of "the enforcing function validates, not the caller".
2. **Error paths bypass completion invariants.** Happy paths drain, settle, and record; error paths return early through dropped state (F-02), park nothing (F-03), or strand records (F-11). The codebase's own review discipline caught the single-item case but not the batch case — negative tests for *mid-batch* and *post-side-effect* failures are the missing test class.
3. **Crash-consistency is per-file, not cross-file.** Each write is fsync'd and atomic, but multi-file commits (mmr.json/entries.jsonl, case.json vs ledger, active-policy view) lack a defined recovery order and repair tooling (F-04, F-11, F-25.r).
4. **Verified identity attributes stop at the gate.** The identity gate is exemplary (SO_PEERCRED, role verification, SoD re-checks); then the verified role is discarded (F-08) and thoth re-derives its own actor model (F-23) — trust decisions re-constructed downstream from partial data.
5. **Two root conventions coexist** (F-12) — a cross-crate contract that only conventions enforce; symptoms are swallowed (`cell_id: None`, silent zero-run exports).
6. **Async runtime discipline**: correctness is strong, but blocking fs/flock/subprocess calls sit on workers in enough places (F-07, F-24) that load-dependent stalls are structural rather than incidental.
7. **`catch_unwind`-free panic surface on corrupted disk state** is small but nonzero exactly where "detect corruption" is the job (F-05, F-06) — crash-on-corruption in an integrity verifier is the worst place for it.

---

## Final Prioritization

### 1. Immediate
- **F-01** authority deny-write bypass (normalization + built-in anchoring) — the specified hard boundary; small, contained fix, high proof value (P4b/V5 depend on it).
- **F-02** dispatcher error-path drain-and-settle (or batch pre-validation) — case-engine availability + ledger truth; fixes the stranded-case class server-side.
- **F-04/F-05/F-06** ledger crash-window ordering + the two verification-path panics — do these together (all in `sea-forge-ledger`, all "corruption must yield typed errors and repairable state").

### 2. High priority
- **F-03** escalation/approval state machine (park instead of terminate; approval re-drive).
- **F-11** stranded-`Active` recovery verb (pairs with F-02; same invariant).
- **F-08** propagate verified `ActorRole` through server evaluations.
- **F-07** `fire_notify` hardening (drain + timeout + `spawn_blocking`).
- **F-10** fabricated resume evidence (settlement-from-evidence thesis).

### 3. Important but non-urgent
- **F-12** root-convention unification + fail-closed export.
- **F-13** promotion substring matching.
- **F-09** drafts id validation; **F-14/F-15/F-17** id validation at enforcing boundaries (same remediation pattern as F-01 — land together).
- **F-18** ACP wall-clock cap + bounded maps; **F-19** timeout clamp; **F-16** policy/plan path constraints.
- **F-21** status-verb rebuild; **F-23** thoth role mapping.

### 4. Defense-in-depth / hardening
- **F-20** violation-classification heuristic; **F-22** key hygiene; **F-24** blocking-in-async + ledger scan costs; **F-25.a–s** as batched hygiene (bunfig freeze, env allowlist, PATH resolution, dedupe windows, fsync asymmetry, reserved-mutator boundary registration, `cdtor` debris cleanup + close out the SIGSEGV note).

**Ordering dependencies:** F-01's normalization primitive should be chosen first (it also serves F-14/F-15/F-17 and F-09's fix shape). F-04's "mmr as derived state" decision determines F-05's fix. F-02 and F-11 should land before F-03's approval re-drive (the recovery verb is where a re-driven case resumes). F-08 precedes any policy work that leans on role-keyed rules server-side.

---

*End of report. Audit artifacts: `.agents/reports/audit-harness-2026-08-14/` (verification harness; builds against the working tree at `df80d73`; no production code was modified during this audit).*

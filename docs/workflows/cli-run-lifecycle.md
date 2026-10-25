# Workflow: CLI Run Lifecycle (`sea-forge run`)

> **What `sea-forge run --intent` actually does, in execution order, including the control-plane scaffolding created before authority evaluates.**  
> Source: [`crates/sea-forge-cli/src/pipeline.rs`](../../crates/sea-forge-cli/src/pipeline.rs).  
> Interactive companion: [Governed Run Explorer](../explainers/governed-run-explorer.html).

---

## 1. Summary

`sea-forge run --intent "..."` interprets an intent, formulates a one-item case plan, commits criteria and authority records, executes each allowed operation in an OS sandbox, captures evidence, independently settles the declared outcome, and records a capability envelope.

The governing invariant is about **effects**, not directory existence: no command is spawned and no workload file is written until authority returns `Allow` for that operation. The run's control-plane scaffolding (`run_dir`, `workspace/`, `artifacts/`, plan, traces) is created before that evaluation so the decision and its evidence have somewhere to live. Do not read the invariant as "the filesystem is untouched until Allow."

Exit codes are `0` accepted, `3` rejected, `4` escalated. A dispatch error (`Input`, `UnknownIntent`, `Plan`) returns `2`; any other error returns `1`.

---

## 2. Sequence

```mermaid
sequenceDiagram
    autonumber
    actor Op as Operator / script
    participant CLI as sea-forge-cli
    participant Dom as sea-forge-domain
    participant Plan as sea-forge-planner
    participant Led as sea-forge-ledger
    participant Auth as sea-forge-authority
    participant Run as sea-forge-runtime
    participant Sand as sea-forge-sandbox
    participant Set as sea-forge-settlement
    participant Cap as sea-forge-capability

    Op->>CLI: run --intent "..."
    CLI->>CLI: load policy, interpret intent, validate attribution
    CLI->>CLI: create root, runs/, cases/, run_dir, workspace/, artifacts/
    CLI->>Plan: plan(intent, case_id, run_id, current executable)
    Plan-->>CLI: CasePlan
    CLI->>Led: commit settlement criteria, intent, case plan, identity, policy
    CLI->>Auth: evaluate every operation
    Auth->>Led: commit authority evidence, request, decision
    alt every verdict is Allow
        CLI->>Run: execute(grant, request)
        Run->>Run: re-resolve argv[0] and compare to grant identity
        Run->>Sand: select/prepare sandbox
        Sand-->>Run: captured ExecutionResult
        Run-->>CLI: stdout, stderr, exit, timeout status
    else any verdict is not Allow
        CLI->>CLI: trace RunHalted; execute nothing
    end
    CLI->>Set: settle(claim, workspace, run_dir)
    Set-->>CLI: SettlementEvent
    CLI->>Led: commit settlement, capability envelope
    CLI->>Cap: append capabilities.jsonl, extract memory items
    CLI-->>Op: exit 0 / 3 / 4
```

---

## 3. Ordered Execution

### Step 1 — Load policy and validate ingress

The CLI loads the policy bundle first. It then calls `sea_forge_domain::interpret` on the intent. An unrecognized intent raises `UnknownIntent` and dispatch returns `2` before any cell directory is created.

`--entity` and `--process` are validated as bounded identifiers: ASCII lowercase, digits, `_`, or `-`, at most 64 characters.

### Step 2 — Create control-plane scaffolding

Only after policy load, intent interpretation, and attribution validation:

1. create the root if needed and canonicalize it;
2. ensure `<root>/runs` and `<root>/cases`;
3. generate `run_id` and `case_id`;
4. create `<root>/runs/<run_id>`;
5. create `<root>/runs/<run_id>/workspace`;
6. create `<root>/runs/<run_id>/artifacts`.

This ordering is deliberate. A malformed intent does not leave a run. A valid intent gets a stable run directory before authority records are produced, so every subsequent record has a durable home.

### Step 3 — Plan and bind criteria

The planner produces a `CasePlan` using the current executable as the trusted command target. Each item's settlement criteria are derived, committed to the ledger as `settlement_criteria`, and referenced by `settlement_criteria_ref`. The plan is then verified against the criteria map.

The intent, case plan, identity binding, and authority policy snapshot are committed before evaluation. `plan.json` is written exactly once, after `run_started` and before `plan_created`.

### Step 4 — Evaluate authority

For each operation, the authority engine evaluates the action, workspace boundary, actor binding, environment, and any DomainForge candidate (none exists: a bundle that enables `engine: domainforge` is refused at load with `authority_engine_unavailable`). Each evaluation produces:

* an `AuthorityEvaluated` trace event;
* a prepared `AuthorityDecision` evidence record;
* a committed `authority_evidence` entry;
* a committed `authority_request` and `authority_decision` pair.

`authority.json` is materialized from the committed decisions. Authority mirrors under `<root>/authority/` are rebuilt from the ledger.

### Step 5 — Execute only if every verdict allows

If any decision is not `Allow`, the CLI appends `RunHalted` and executes nothing. It still proceeds to settlement with the recorded verdicts, which is how a denial becomes a durable `Rejected` or `Escalated` outcome rather than a missing record.

If every verdict is `Allow`, the CLI appends `WorkspaceCreated` and processes operations in order:

* `WriteFile` is materialized through an authority grant;
* `ExecuteCommand` is handed to `sea-forge-runtime`.

### Step 6 — Re-check executable identity at spawn

The runtime does **not** compare a stored SHA-256 of the executable at spawn. It captures the decision-time canonical executable path in the grant, re-resolves `argv[0]` immediately before spawning, and requires exact path equality. A binary swapped at the same path is a same-instant race; a different argv0 is refused.

The runtime also validates the grant, derives network posture from the grant's boundary map, selects the sandbox class, and prepares the sandbox. Typical read access is the system runtime set plus the bound executable's tree; writes are confined to `workspace/` and `artifacts/`. Ambient environment is not inherited; only `PATH` and `HOME` are forwarded.

### Step 7 — Capture execution evidence

The CLI appends `CommandStarted` and `CommandFinished`, records the `ExecutionResult`, and captures `stdout.txt`, `stderr.txt`, and any `model.sea` artifact. Artifact descriptors carry SHA-256 content identity and pre-mint identity. That content hashing is artifact evidence; it is separate from the executable identity check in Step 6.

### Step 8 — Settle independently

`sea_forge_settlement::settle` evaluates the claim against evidence and workspace state. It does not trust the process exit code as success. A zero exit with a missing required artifact is still rejected.

The settlement event and explicit basis tokens are committed to the ledger, written to `settlement.json`, and traced with `SettlementRecorded`.

### Step 9 — Persist capability and memory

The capability envelope is committed and materialized as `semantic-envelope.json`. The case state is closed as:

* `Completed` for `Accepted`;
* `Terminated` with reason `rejected` for `Rejected`;
* `Terminated` with reason `escalated` for `Escalated`.

The envelope is appended to `<root>/capabilities.jsonl`. Memory items are extracted deterministically into `<root>/memory/items.jsonl`; extraction failure is logged and does not reverse settlement.

---

## 4. State Transitions

| Stage | Before | After |
|---|---|---|
| Ingress | Policy may or may not exist | Intent interpreted or rejected |
| Scaffolding | No run directory | `run_dir`, `workspace/`, `artifacts/` exist |
| Planning | Intent only | Plan and criteria records exist |
| Authority | Uncommitted decisions | Evidence, requests, and decisions in the ledger |
| Execution | Empty workspace | Commands ran, or `RunHalted` was traced |
| Evidence | Execution output on disk | Captured descriptors and execution records |
| Settlement | Unsettled claim | `settlement.json` with status and basis tokens |
| Capability | Unmetabolized run | Envelope appended; memory extraction attempted |
| Case closure | Active | `Completed` or `Terminated` |

---

## 5. Failure Branches

| Failure | Observable result | Exit |
|---|---|---|
| Unknown intent | `UnknownIntent`; no run directory | `2` |
| Invalid plan or input | `Plan`/`Input` dispatch error | `2` |
| Policy load or IO error | Error before or during setup | `1` |
| Authority deny | `RunHalted`, no execution, settlement `Rejected` | `3` |
| Authority escalate | Settlement `Escalated`, case terminated | `4` |
| Nonzero child exit | Execution recorded; settlement `Rejected` with exit basis | `3` |
| Timeout | Execution `TimedOut`; settlement `Rejected` | `3` |
| Missing required artifact | Settlement `Rejected` with `required_artifact_missing:<path>` | `3` |
| Missing executable identity | Runtime refuses before spawn | `1` |
| Memory extraction failure | Run settlement stands; warning logged | Same as settlement |

A denial is therefore both a governance decision and a normal settled outcome. Inspect `settlement.json` and `authority.json`; do not infer the cause from the process exit code alone.

---

## 6. `run --plan` Is Different

`sea-forge run --plan` advances an existing CMMN-style case rather than creating a one-item intent run. It can:

* execute one or more ready items and settle each;
* park on a pending approval;
* park on a human task;
* report that no action is ready;
* complete or terminate the case.

Its exit codes are `0` completed, `3` terminated, `5` awaiting approval or otherwise still active. It does not use `4` as a normal path. Use `sea-forge resume` after the blocking condition is resolved.

---

## 7. Source Trail

* [`crates/sea-forge-cli/src/pipeline.rs`](../../crates/sea-forge-cli/src/pipeline.rs) — Intent run setup, authority loop, execution, settlement.
* [`crates/sea-forge-cli/src/plan_pipeline.rs`](../../crates/sea-forge-cli/src/plan_pipeline.rs) — Case-plan advancement and its `0`/`3`/`5` outcomes.
* [`crates/sea-forge-cli/src/commands/run.rs`](../../crates/sea-forge-cli/src/commands/run.rs) — Command dispatch and settlement-to-exit mapping.
* [`crates/sea-forge-runtime/src/lib.rs`](../../crates/sea-forge-runtime/src/lib.rs) — Grant check, executable re-resolution, sandbox selection.
* [`crates/sea-forge-settlement/src/lib.rs`](../../crates/sea-forge-settlement/src/lib.rs) — Settlement and basis tokens.
* [`crates/sea-forge-authority/src/lib.rs`](../../crates/sea-forge-authority/src/lib.rs) — Authority evaluation and grant minting.

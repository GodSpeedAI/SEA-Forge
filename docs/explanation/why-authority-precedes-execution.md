# Explanation: Why Authority Must Precede Every Side Effect

> **Deep rationale for Invariant AUTH-01: Why directory creation, process spawning, and file touching must wait for an authorized, ledger-committed Allow verdict.**

---

## 1. The Principle of Zero Speculative Side Effects

In conventional operating systems and CI/CD pipelines, tools frequently practice **speculative side effects**:
* A command runner creates a temporary directory (`/tmp/run-12345/workspace`) before evaluating whether the current user is authorized to perform the requested build.
* If authorization fails or the user lacks permissions, the runner aborts, leaving behind an orphaned directory or temporary files.

In SEA Forge, this behavior is strictly forbidden by **Invariant AUTH-01**:

> **AUTH-01:** Every workload side effect, including workspace directory creation, must follow a committed `Allow` decision. Mandatory governance recording (`authority_request`, `authority_decision`, `RunHalted` trace) remains allowed so denied outcomes are fully auditable, but no workload filesystem, network, or process mutation may occur.

---

## 2. Why Creating Directories Before Authority Is Dangerous

Early in the development of SEA Forge, an older server implementation (`case_dispatch.rs`) created the `workspace/` and `artifacts/` directories *before* calling `evaluate()`. A code audit identified this as a critical security and governance flaw:

1. **Information Leakage via Filesystem Presence:**
   If an untrusted actor or unauthenticated client submits an unauthorized case, creating directories on disk proves that the server responded to and processed the intent. On multi-tenant or shared systems, directory presence reveals operational cadence, case names, and timing.
2. **Denial-of-Service via Inode Exhaustion:**
   An attacker submitting millions of unauthorized requests could fill the filesystem with empty `workspace/` directories without ever passing a single policy check.
3. **Audit Inconsistency:**
   A forensic auditor inspecting the filesystem would find directories that were never authorized to exist. Did a command run? Was it deleted? Was it half-executed? By guaranteeing that directories exist *only* if an `Allow` verdict committed to the ledger, directory existence is synonymous with an authorized execution attempt.

---

## 3. The Run Record vs. Workspace Distinction

A nuanced challenge arose during the design of AUTH-01: **Where does a denial get recorded?**

If an authority evaluation denies an operation, SEA Forge must record the denial:
* An `AuthorityRequest` and `AuthorityDecision` must be written.
* A `RunHalted` trace event must be logged.
* An evidence record of the denial must be preserved.

If creating the run directory were forbidden, there would be nowhere on disk to store the audit trail—making an unauthorized intrusion attempt indistinguishable from an event that never occurred.

### The Resolution: Governance Records vs. Workload Space
SEA Forge resolves this by separating **governance recording** from **workload execution space**:
* The run directory (`.sea-forge/runs/<run_id>/`) is created solely to hold governance metadata (`authority.json`, `trace.jsonl`, `evidence.jsonl`).
* The **workload directories** (`workspace/` and `artifacts/`)—the directories that the untrusted child process or file operations would actually touch—remain completely unborn until a committed `Allow` decision is recorded in the integrity ledger.

```text
Upon Intent Ingress:
├── .sea-forge/runs/<run_id>/          <-- Created to hold governance audit trail
│   ├── trace.jsonl                   <-- Logs RunStarted, PlanCreated
│   ├── plan.json
│   └── authority.json                <-- Records AuthorityDecision (Deny)
│
If Denied:
└── Execution HALTS. No workspace/ or artifacts/ directory is EVER created.

If Allowed:
├── workspace/                        <-- Created ONLY AFTER committed Allow
└── artifacts/                        <-- Created ONLY AFTER committed Allow
```

---

## 4. The `ActionGrant` Seam

How does the compiler guarantee that a developer does not accidentally spawn a process without an `Allow` decision?

Through Rust's affine type system:
* `PolicyAuthorityEngine::grant()` produces an `ActionGrant`.
* `ActionGrant` does not implement `Clone`, `Copy`, `Serialize`, or `Deserialize`.
* `sea_forge_runtime::execute()` requires `grant: ActionGrant` by value.

There is no physical way in Rust code to invoke `runtime::execute()` without holding an `ActionGrant`, and there is no way to obtain an `ActionGrant` without the policy engine verifying that the decision is `Verdict::Allow` and committed to the integrity ledger.

---

## 5. Source Evidence

* [`crates/sea-forge-server/src/case_dispatch.rs#L513-L552`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/src/case_dispatch.rs#L513-L552) — Enforcement that `workspace/` is created strictly inside the `Verdict::Allow` branch.
* [`crates/sea-forge-authority/src/lib.rs:69-87`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-authority/src/lib.rs#L69-L87) — Definition of move-only `ActionGrant`.
* [`crates/sea-forge-server/tests/conformance_case_views.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/tests/conformance_case_views.rs) — Conformance test: `a_denied_episode_creates_no_workspace_and_no_artifacts`.

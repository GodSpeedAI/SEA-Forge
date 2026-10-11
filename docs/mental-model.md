# SEA Forge Mental Model

> **The conceptual foundation of governed capability execution.**  
> Why processes cannot self-certify success · Why authority must precede side effects · How truth is separated from views.

---

## 1. The Core Paradigm: Governed Execution

Most software development tooling operates under an implicit assumption: **the developer or process running the command is trusted, and process termination (`exit 0`) equals success**.

In autonomous systems, AI coding agents, and regulated enterprise environments, both assumptions fail:
* An agent can run a test runner, experience a misconfigured glob, execute zero tests, exit 0, and declare that the test suite passed.
* An agent can run an untrusted tool that inspects the parent environment, reads credentials, and contacts an external server.
* A task runner can report that a file was created when in reality it wrote to an unmapped path or hallucinated content.

SEA Forge inverts this paradigm. It treats **execution as untrusted** and **governance as independent**:

```text
Traditional Execution:
Intent (Text) ──> Shell Execution ──> Exit 0 ──> "Success" (Assumed True)

Governed Execution (SEA Forge):
Intent (Data) ──> Typed Plan ──> Pre-Authorization ──> Sandbox Isolation ──> Independent Settlement ──> Capability
                                (Default Deny)        (Landlock Jail)     (Evidence Claims)        (Metabolized)
```

In SEA Forge:
1. **A process is never permitted to declare its own success.**
2. **Execution cannot occur without an explicit, pre-committed grant.**
3. **Outcomes are evaluated from verifiable evidence artifacts, not narrative assertions.**

---

## 2. The Four Pillars of SEA Forge

### Pillar 1: Typed Intent and Deterministic Planning

In SEA Forge, intent is never interpolated into shell scripts (`sh -c "$INPUT"`). Raw intent is represented as a structured Rust struct:

```rust
pub struct Intent {
    pub intent_id: String,
    pub summary: String,
    pub actor_id: String,
    pub process_id: String,
    pub created_at: String,
}
```

The **Deterministic Planner** maps this intent to an immutable `CasePlan` containing concrete `PlanItem`s. Each item specifies:
* The exact operations to be performed (`WriteFile`, `ExecuteCommand`, `AgentTask`).
* The exact `SettlementCriteria` required to satisfy the plan item.
* The required `SandboxClass` (`Local`, `Jail`, `Microvm`).

Because planning is deterministic, identical inputs produce identical plans and identical cryptographic hashes.

---

### Pillar 2: Authority Precedes Every Side Effect

Authority in SEA Forge is **fail-closed** and **unbypassable**:

* **Pre-Execution Check:** Every operation in a plan item is submitted to the `PolicyAuthorityEngine` before execution begins.
* **No Side Effects on Denial:** If any operation in an item is denied or escalated, **no workspace is created**, no child process is spawned, and no file is touched. A `RunHalted` trace event and an `AuthorityDecision` are committed to the ledger so the audit trail is complete, but the system state remains unmutated.
* **The `ActionGrant` Seam:** Upon an `Allow` decision, the authority engine mints an `ActionGrant`. This struct is deliberately **non-cloneable and non-serializable**. It acts as a single-use capability token that must be passed by value into `sea-forge-runtime::execute()`. Without an `ActionGrant`, the runtime compiler forbids process creation.

---

### Pillar 3: Authority and Isolation Are Distinct

A common architectural confusion is conflating **permission** with **sandboxing**. In SEA Forge, they are strictly separated:

* **Authority decides IF an operation may run:** It checks actor identity, roles, organization policies, and candidate domain verdicts. It outputs `Allow`, `Deny`, or `Escalate`.
* **The Sandbox enforces BOUNDARIES during execution:** Once an operation is allowed, the sandbox ensures the process cannot exceed its authorized boundaries. It restricts filesystem access to `workspace/` and `artifacts/`, blocks ambient network connections (`NetworkPosture::Denied`), strips unapproved environment variables, and enforces wall-clock timeouts.

Neither substitutes for the other:
* A sandbox cannot replace policy: running unauthorized work inside a sandbox is still a security violation.
* Policy cannot replace a sandbox: an authorized process can still be exploited or behave erratically.

---

### Pillar 4: Settlement Is Independent of Process Exit

In standard operating systems, a process communicates outcome via its exit code:
* `exit(0)` = Success
* `exit(1..255)` = Failure

In SEA Forge, **`exit 0` is merely one piece of observable evidence**, not acceptance.

When execution completes, `sea-forge-settlement` evaluates a `SettlementClaim` against the recorded evidence:
1. Did the process exit 0? (Only evaluated if `require_exit_zero` was true).
2. Are all required file artifacts present in `artifacts/`?
3. Do the artifact SHA-256 hashes match declared expectations?
4. Did stdout contain the required literal strings?
5. Did domain evaluators award acceptable scores?

Only if **all criteria are satisfied** does settlement return `SettlementStatus::Accepted`. If a command exits 0 but fails to produce a declared output artifact, settlement evaluates to `SettlementStatus::Rejected` with an explicit basis token: `required_artifact_missing:<path>`.

---

## 3. The Lifecycle of an Episode

Every execution in SEA Forge is an **episode** (a single run) that progresses through an invariant state machine:

```mermaid
stateDiagram-v2
    [*] --> Planned: Deterministic Plan Created
    Planned --> EvaluatingAuthority: Submit Operations to Engine
    
    EvaluatingAuthority --> Halted: Verdict == Deny or Escalate
    Halted --> GovernanceCommitted: Write AuthorityDecision & Trace
    GovernanceCommitted --> [*]: Exit with Rejection Code
    
    EvaluatingAuthority --> Authorized: Verdict == Allow
    Authorized --> GrantMinted: Mint Move-Only ActionGrant
    GrantMinted --> WorkspaceMaterialized: Create workspace/ & artifacts/
    WorkspaceMaterialized --> Executing: Spawn Child in OS Jail
    Executing --> EvidenceCaptured: Capture Artifacts, Hashes & Stdout
    EvidenceCaptured --> Settling: sea_forge_settlement::settle()
    
    Settling --> Accepted: Criteria Satisfied
    Settling --> Rejected: Criteria Failed
    
    Accepted --> EnvelopeAppended: Write semantic-envelope.json & capabilities.jsonl
    Rejected --> EnvelopeAppended: Record Rejection Basis
    EnvelopeAppended --> [*]: Exit with Settlement Status
```

---

## 4. Truth vs. Views: The Storage Philosophy

SEA Forge organizes its persistence around the principle of **event sourcing and append-only truth**:

* **Immutable Truth:**
  * Once written, records are never updated in place or deleted.
  * History is appended to `.jsonl` journals and MMR-backed ledger streams.
  * Even failed runs, denied authority requests, and aborted tasks leave permanent, tamper-evident audit records.
* **Rebuildable Views:**
  * SQLite search indexes, active policy snapshots, and SFWP query results are **projections**, not canonical sources of truth.
  * If the SQLite database is corrupted or deleted, running `sea-forge recall --rebuild` replays `capabilities.jsonl` to restore byte-equivalent indexes.
  * A projection can never override or contradict an append-only source record.

---

## 5. Next Steps

* To see how these concepts map to the codebase structure, read the [Architecture Specification](file:///c:/Users/sprim/projects/sea-rs/docs/architecture.md).
* To trace an end-to-end execution path through the source code, read the [CLI Run Lifecycle Trace](file:///c:/Users/sprim/projects/sea-rs/docs/workflows/cli-run-lifecycle.md).
* To explore the policy engine in detail, read the [Authority Fabric Guide](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/authority-fabric.md).

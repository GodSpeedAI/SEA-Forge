# Explanation: Settlement vs. Process Exit

> **Deep rationale for Invariant DOM-01: Why exit code 0 is merely one piece of evidence, and why settlement requires an explicit, machine-readable factual basis.**

---

## 1. The False Success Crisis

In the Unix philosophy and POSIX standard, process status is reported as an 8-bit integer:
* An exit code of `0` indicates success.
* Any non-zero exit code (`1..255`) indicates an error.

For decades, build systems, CI/CD runners, and shell scripts have relied on this binary signal:
```sh
npm test && deploy.sh
```

In automated software engineering and autonomous AI agent operations, **`exit 0` is fundamentally broken as a success metric**. Consider these real-world failure modes:

1. **The Empty Test Suite:** A developer or agent modifies a glob pattern in Jest or Pytest (`test_*.py`). The glob matches zero files. The test runner executes 0 tests and exits `0`. The CI pipeline passes, and broken software deploys to production.
2. **Swallowed Exceptions:** A Python or Node script wraps its entire body in a broad `try ... except Exception: pass` block. An unhandled exception occurs, the error is swallowed, and the process exits `0`.
3. **Missing Work Products:** A code generator runs, encounters an unhandled edge case, prints an error message to stdout, and exits `0` without creating the promised output binary.
4. **Hallucinated File Paths:** An autonomous AI coding agent asserts in natural language that it has created `src/components/Button.tsx`, but inadvertently writes the file to `/tmp/Button.tsx` or leaves it in memory.

In all four cases, the operating system reports **success**, yet the objective task **failed**.

---

## 2. Invariant DOM-01: Settlement Is Not Process Completion

SEA Forge resolves this crisis by enforcing **Invariant DOM-01**:

> **DOM-01:** Settlement, not process completion, determines outcome acceptance. Declared acceptance criteria and verifiable evidence must support the result. Process exit code 0 is merely one piece of evidence; it never constitutes proof of success on its own.

In SEA Forge, execution and settlement are separate lifecycle stages:

```text
1. Execution Phase:
   Process runs inside sandbox -> Produces ExitCode, stdout.txt, stderr.txt, and files in artifacts/

2. Settlement Phase:
   sea_forge_settlement::settle() evaluates the SettlementClaim against physical reality on disk.
```

---

## 3. The Anatomy of a Settlement Claim

When a plan item is formulated, it declares an immutable `SettlementCriteria` contract:

```rust
pub struct SettlementCriteria {
    pub require_exit_zero: bool,           // Did it exit 0?
    pub required_artifacts: Vec<String>,   // Must these files exist in artifacts/?
    pub stdout_must_contain: Option<String>,// Must stdout contain this exact string?
    pub agent_output_must_contain: Option<String>,
    pub evaluator: Option<String>,         // Domain evaluator metric threshold
    pub min_pass_ratio: Option<f64>,       // Batch pass threshold (e.g. >= 0.95)
}
```

When execution completes, `settle()` checks:
1. **Exit Code:** If `require_exit_zero: true`, is `execution.exit_code == Some(0)`?
2. **Artifact Presence:** For every path in `required_artifacts`, does `safe_existing(workspace, path)` find a valid, readable regular file?
3. **Artifact Integrity:** If content hashes were pre-registered, does the SHA-256 hash of the created file match the expected digest?
4. **Stdout Content:** If `stdout_must_contain` is specified, does `stdout.txt` contain the exact substring?
5. **Evaluator Metrics:** If evaluators were executed, did the scores exceed configured minimums?

If **any check fails**, settlement evaluates to `SettlementStatus::Rejected`, even if the child process exited 0.

---

## 4. The Non-Empty Basis Proof

A core design requirement in SEA Forge is that an accepted settlement cannot be a bare boolean. An auditor or downstream system must be able to inspect **why** the settlement was accepted.

Therefore, `SettlementEvent` requires an explicit `basis` array of machine-readable string tokens:

```json
{
  "version": "0.1",
  "settlement_id": "set_20260902T190000Z_abcdef",
  "run_id": "run_20260902T190000Z_123456",
  "status": "accepted",
  "basis": [
    "authority_allow",
    "exit_zero",
    "required_artifact_present:model.sea",
    "stdout_match"
  ],
  "review_required": false,
  "settled_at": "2026-09-02T19:00:01Z"
}
```

If a command exits 0 but fails to generate `model.sea`, the settlement record explicitly reflects:
```json
{
  "status": "rejected",
  "basis": [
    "authority_allow",
    "exit_zero",
    "required_artifact_missing:model.sea"
  ]
}
```

**Mechanical Enforcement:** Conformance proof P1 (`just proof`) asserts that any accepted settlement event must have `status == "accepted"` AND `basis | length > 0`. An accepted event with an empty basis fails the proof gate.

---

## 5. Architectural Consequences & Trade-Offs

### What We Gain
* **Immunity to False Success:** Agents cannot hallucinate task completion; automated workflows cannot deploy empty builds.
* **Deterministic Replay & Non-Repudiation:** Every historical run contains an immutable record of the exact artifacts and assertions that justified its acceptance.
* **Separation of Concerns:** Compilers and test runners can focus on tool execution; SEA Forge independently audits the deliverables.

### Trade-Offs Accepted
* **I/O Overhead:** Capturing stdout, reading output files, and computing SHA-256 digests introduces small I/O and CPU overhead compared to unchecked shell scripts. In practice, for developer and agent workloads, this overhead is measured in single-digit milliseconds and is vastly outweighed by the elimination of undetected silent failures.

---

## 6. Source Evidence

* [`crates/sea-forge-settlement/src/lib.rs:16-100`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-settlement/src/lib.rs#L16-L100) — Core `settle()` evaluation implementation.
* [`crates/sea-forge-core/src/types.rs:610-635`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-core/src/types.rs#L610-L635) — `SettlementCriteria` schema definition.
* [`justfile:335-394`](file:///c:/Users/sprim/projects/sea-rs/justfile#L335-L394) — Normative conformance proofs asserting non-empty basis and false-success rejection.

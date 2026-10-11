# GodSpeed Bounded-Judgment Plan — Adversarial Inspection and Grounding Audit

**Status:** Inspection Complete — Rejection & Remediation Blueprint Issued  
**Date:** 2026-09-17  
**Artifact Under Test:** `.agents/plans/godspeed-bounded-judgment-plan.yaml` (2,147 lines, `version: 1.0.0`, `status: ready`)  
**Normative Authority:** `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` (`id: godspeed.judgment-plane`, `version: 1.0.0`, `sha256: 69b7d1baabd55ae61c73e615f04cf0fda43a4958a9fa3d50567035425baf6a82`)  
**Companion Documents Inspected:**
- `.agents/plans/godspeed-bounded-judgment-brief.md`
- `.agents/reports/2026-09-16-godspeed-judgment-plane-adversarial-review.md`
- `.agents/reports/2026-09-16-godspeed-capability-gap.yaml`
- `.agents/reports/2026-09-16-godspeed-spec-delta-v2.yaml`
- `.agents/CURRENT_STATUS.md`
- Actual repository workspaces: `sea-rs`, `gauntlet`, `edgeai`, `sxr`, `cep`

---

## 0. Executive Summary & Verdict

### Final Verdict: REJECTED AS CURRENTLY UNRUNNABLE AND UNDER-CONFIRMED

The plan `godspeed-bounded-judgment-plan.yaml` represents an impressive conceptual achievement: it faithfully carries forward the architectural reduction from the 2026-09-16 adversarial review (no separate Judgment Plane daemon, no message broker, no independent judgment database, zero `.sea` grammar changes, and strict "measurability before capability" via PROOF-1).

However, an adversarial inspection grounded against the actual file trees, Cargo workspace definitions, Justfiles, and SQLite schemas reveals that **the plan cannot be executed cold by another agent as written**. It contains critical grounding hallucinations, invalid command invocations, unauthored prerequisite test harnesses, silent high-risk confirmation downgrades, and broken dependency graph edges.

```
                    ┌────────────────────────────────────────────────────────┐
                    │      GODSPEED BOUNDED-JUDGMENT PLAN STATUS             │
                    ├────────────────────────┬───────────────────────────────┤
                    │ Conceptual Reduction   │ PASS (Faithful to spec C1-C11)│
                    │ Graph Acyclicity       │ PASS (Internally acyclic)     │
                    │ Numerical Requirement  │ PASS (90/90 IDs present)      │
                    │ Physical Grounding     │ FAIL (Phantom crates & paths) │
                    │ Gate Executability     │ FAIL (Broken recipes & syntax)│
                    │ High-Risk Confirmation │ FAIL (3 of 6 downgraded)      │
                    │ DAG Schedulability     │ FAIL (T13 premature, T2/3 gap)│
                    └────────────────────────┴───────────────────────────────┘
```

### Scorecard of Concrete Defects Found

1. **Fatal Physical Grounding & Crate Hallucinations (5 Critical)**
   - **Phantom Crates**: T07, T08, T09, and T10 reference `gauntlet-engine`, `gauntlet-storage`, and `gauntlet-kernel`. **None of these crates exist** in `gauntlet` or `sea-rs`.
   - **Unrunnable Cross-Repo Recipe**: T07–T11 prescribe `just crate-test <crate>`. `just crate-test` does not exist in `gauntlet`. When run in `sea-rs`, Cargo fails immediately (`package ID specification "gauntlet-domain" did not match any packages`).
   - **Template Placeholder in Production Gate**: T06 gate contains literal unresolved template placeholders: `just crate-check <implementation-crate>` and `just crate-test <implementation-crate>`.
   - **Shell Syntax Error in T12 Gate**: T12 gate contains `just health (ON THE TARGET MACHINE)`, which causes a bash parse error on `(`.
   - **Phantom Source File & Symbol**: T06 cites `crates/sea-forge-cell/src/policy.rs (PolicyDecisionKind enum)`. Neither the file nor the enum exists in `sea-rs`.

2. **Corpus & External Repository Inaccuracies (3 High)**
   - **Non-Existent Top-Level Gauntlet Evidence Path**: T01 and T13 cite `../gauntlet/evidence/objects/` and `../gauntlet/evidence/pins/`. No such top-level directory exists. Gauntlet stores evidence per-run under `../gauntlet/targets/.runs/<run_id>/evidence/objects/` and `pins/`.
   - **False Claim About EdgeAI Test Suite**: T12 claims as a "KNOWN FACT" that `edgeai currently has no tests directory`. In reality, `/home/sprime01/projects/edgeai/tests/` exists, is heavily populated (78 test files), and has its own `tests/AGENTS.md`.
   - **Missing Pre-requisite Scripts**: T00 gate requires `python3 .agents/evidence/godspeed-bounded-judgment/T00/validate_plan.py`. Neither the script nor the parent directory exists on disk, and T00 instructions say "Run" rather than "Write".

3. **High-Risk Confirmation Downgrades (4 High)**
   - Spec `conformance_traceability` mandates **independent** confirmation for `semantic_bounding` and `uncertainty_integrity`, and **adversarial** confirmation for `claim_scope_integrity`.
   - The plan downgrades T01, T06, T07, and T11 to `builder_with_teeth`.
   - T14 is claimed to satisfy these retroactively, but T14's verifier is not provided the evidence for T06 or T11, nor the normative texts for semantic bounding or uncertainty integrity.
   - Spec mandates `fresh_target_reproduction` for `provenance_replay` (`REQ-PROV-001`, `003`, `004`, `020`), but T12 does not settle or re-prove these on target hardware.

4. **DAG Topology & Schedulability Flaws (3 Medium)**
   - **T13 Premature Execution**: T13 depends only on `["T01", "T05"]`, yet it claims to complete T07's replay scope and export T07's carrier fields. An agent can execute T13 before T06 or T07 have even started.
   - **Dangling Hygiene Tasks (T02 & T03)**: T02 (Gauntlet CI) and T03 (Server gate repair) have no outgoing edges. An orchestrator can complete T14 and mark the entire plan settled while T03 (red server test) and T02 remain unexecuted.
   - **T08 Dependency on T03**: T08 runs `just crate-test sea-forge-server`. If T03 has not repaired the server test isolation failure, T08 fails.

---

## 1. Grounding & Reality Audit (Verification Against Code & Workspaces)

### 1.1 Phantom Gauntlet Crates (`gauntlet-engine`, `gauntlet-storage`, `gauntlet-kernel`)

**Plan Citations:**
- Line 1202: `gauntlet-engine/src/state.rs (StateStore, get_run_state, commit — append-style persistence)`
- Line 1204: `gauntlet-storage/src/db.rs (SQLite storage: WAL, busy_timeout, foreign keys on)`
- Line 1219: `just crate-test gauntlet-engine`
- Line 1221: `just crate-test gauntlet-storage`
- Line 1302: `gauntlet-kernel/src/*.rs (kernel orchestration: sweep(), run_invocation(), build_envelope())`
- Line 1318: `just crate-test gauntlet-kernel`
- Line 1409: `gauntlet-engine/src/state.rs (retry/persistence surface)`
- Line 1421: `just crate-test gauntlet-engine`
- Line 1501: `just crate-test gauntlet-engine`

**Empirical Evidence:**
Execution of `ls /home/sprime01/projects/gauntlet/crates/`:
```
gauntlet-app/
gauntlet-arch-tests/
gauntlet-cli/
gauntlet-domain/
gauntlet-ports/
adapters/
```
Execution of `ls /home/sprime01/projects/gauntlet/crates/adapters/`:
```
gauntlet-adapter-agent-cli/
gauntlet-adapter-agent-mock/
gauntlet-adapter-agent-prime/
gauntlet-adapter-compare-diff/
gauntlet-adapter-compare-model/
gauntlet-adapter-escalation-fs/
gauntlet-adapter-evidence-fs/
gauntlet-adapter-id-entropy/
gauntlet-adapter-meter-memory/
gauntlet-adapter-observe-*/
gauntlet-adapter-policy-fs/
gauntlet-adapter-sandbox-*/
gauntlet-adapter-state-sqlite/
gauntlet-adapter-surface-json/
gauntlet-adapter-verify-*/
gauntlet-adapter-workspace-fs/
```

**Finding:**
`gauntlet-engine`, `gauntlet-storage`, and `gauntlet-kernel` **do not exist**.
- `StateStore` and SQLite persistence live in `crates/adapters/gauntlet-adapter-state-sqlite/src/state_store.rs` and `crates/gauntlet-ports/src/state_store.rs`.
- `build_envelope` lives in `crates/gauntlet-app/src/compare/envelope.rs`.
- `sweep()` lives in `crates/gauntlet-cli/src/verify_cmd.rs`.
- `run_invocation()` lives in `crates/adapters/gauntlet-adapter-agent-prime/src/runner.rs`.
- Target crates for testing must be: `gauntlet-domain`, `gauntlet-ports`, `gauntlet-app`, `gauntlet-adapter-state-sqlite`, and the newly created `gauntlet-adapter-agent-http`.

---

### 1.2 Cross-Repo Command Invocation & Missing Recipes

**Plan Citations:**
- Line 1219-1221 (T07): `just crate-test gauntlet-engine`, `just crate-test gauntlet-domain`, `just crate-test gauntlet-storage`
- Line 1318-1320 (T08): `just crate-test gauntlet-kernel`, `just crate-test gauntlet-ports`, `just crate-test sea-forge-server`
- Line 1420-1421 (T09): `just crate-test gauntlet-domain`, `just crate-test gauntlet-engine`
- Line 1501-1502 (T10): `just crate-test gauntlet-engine`, `just crate-test gauntlet-app`
- Line 1585 (T11): `just crate-test gauntlet-ports`

**Empirical Evidence:**
Running `just crate-test gauntlet-domain` inside `/home/sprime01/projects/sea-rs`:
```
cargo test -p "gauntlet-domain" --locked ""
error: package ID specification `gauntlet-domain` did not match any packages
error: recipe `crate-test` failed on line 321 with exit code 101
```
Checking `/home/sprime01/projects/gauntlet/justfile`:
The recipes available in Gauntlet are: `ruler`, `check`, `test`, `test-surfaces`, `lint`, `deny`, `ci`, `doctor`. There is **no `crate-test` recipe**.

**Finding:**
All task gates targeting Gauntlet crates fail immediately with exit code 101.
Any command targeting Gauntlet must either:
1. Explicitly change directory to `../gauntlet`: `(cd ../gauntlet && cargo test -p <crate>)` or `(cd ../gauntlet && cargo nextest run -p <crate>)`.
2. Add a `crate-test` recipe to Gauntlet's `justfile` during T02, and call `(cd ../gauntlet && just crate-test <crate>)`.

---

### 1.3 Literal Unresolved Placeholder in T06 Gate

**Plan Citations:**
Lines 1125–1128:
```yaml
    gate:
      - "just crate-check <implementation-crate>"
      - "just crate-test <implementation-crate>"
```

**Context Contrast:**
Line 14 of `.agents/CURRENT_STATUS.md` asserts:
`Plan validation 19/19 (... complete contracts, no placeholders).`

**Finding:**
`<implementation-crate>` was left unassigned. An automated runner or human agent will execute `<implementation-crate>` as a literal string or fail.
The spec entity `JudgmentObservation` is realized in `gauntlet_domain::VerificationRecord` (`spec:core_domain_model.entities[JudgmentObservation]`). The AnswerDomain contract belongs in `crates/gauntlet-domain`. The gate must be:
```yaml
    gate:
      - "(cd ../gauntlet && cargo check -p gauntlet-domain)"
      - "(cd ../gauntlet && cargo test -p gauntlet-domain)"
```

---

### 1.4 Invalid Bash Syntax in T12 Gate

**Plan Citations:**
Lines 1694–1698:
```yaml
    gate:
      - "just health (ON THE TARGET MACHINE)"
      - "just jetson-check-ports (ON THE TARGET MACHINE)"
      - "python3 .agents/evidence/godspeed-bounded-judgment/T12/verify_reproduction.py"
```

**Finding:**
Parenthetical explanatory remarks were embedded directly into the shell command string.
Executing `just health (ON THE TARGET MACHINE)` in bash yields:
`bash: syntax error near unexpected token '('`.
The gate must specify the executable command only, moving the target machine condition to an execution environment directive or prerequisite check:
```yaml
    gate:
      - "just --justfile ../edgeai/Justfile health"
      - "just --justfile ../edgeai/Justfile jetson-check-ports"
      - "python3 .agents/evidence/godspeed-bounded-judgment/T12/verify_reproduction.py"
```

---

### 1.5 Phantom Source File & Symbol in `sea-rs`

**Plan Citation:**
Line 1116 (T06 hot context):
`crates/sea-forge-cell/src/policy.rs (PolicyDecisionKind enum: an example of the repository's closed-enumeration style)`

**Empirical Evidence:**
Directory listing of `crates/sea-forge-cell/src/`:
`bundle.rs`, `cell.rs`, `lib.rs`, `template.rs`.
Ripgrep across entire `sea-rs` repository for `PolicyDecisionKind`:
`No results found`.

**Finding:**
Neither `policy.rs` in `sea-forge-cell` nor `PolicyDecisionKind` exists. The canonical closed-enumeration governance types in `sea-rs` are `GovernanceDisposition` and `GovernanceVerdict` in `crates/sea-forge-authority/src/lib.rs`.

---

### 1.6 Inaccurate Corpus Evidence Path

**Plan Citations:**
- Line 500 (T01): `Reads ../gauntlet/targets/.runs/*/state/*.db and ../gauntlet/evidence/ read-only.`
- Line 572 (T01): `../gauntlet/evidence/objects/ and ../gauntlet/evidence/pins/ (content-addressed evidence)`
- Line 1782 (T13): `../gauntlet/evidence/objects/ and ../gauntlet/evidence/pins/ (content-addressed evidence)`
- Brief Line 50: `The judgment corpus lives OUTSIDE this repo: ../gauntlet/targets/.runs/ (16 SQLite databases) and ../gauntlet/evidence/.`

**Empirical Evidence:**
Listing `/home/sprime01/projects/gauntlet/`:
`targets/`, `crates/`, `workbench/`, `scripts/`, `examples/`, `.agents/`.
`/home/sprime01/projects/gauntlet/evidence` **does not exist**.
Inspecting `/home/sprime01/projects/gauntlet/targets/.runs/`:
Every run directory contains its own localized evidence directory:
`/home/sprime01/projects/gauntlet/targets/.runs/demo-calculator.settled-rpc-1652/evidence/objects`
`/home/sprime01/projects/gauntlet/targets/.runs/demo-calculator.settled-rpc-1652/evidence/pins`

**Finding:**
There is no centralized `../gauntlet/evidence/` directory. Any script asserting read-only locks or paths on `../gauntlet/evidence/` will crash with file-not-found errors. The correct path pattern is:
`../gauntlet/targets/.runs/*/evidence/objects/` and `../gauntlet/targets/.runs/*/evidence/pins/`.

---

### 1.7 False Claim Regarding EdgeAI Test Suite

**Plan Citation:**
Line 1684 (T12 hot context):
`KNOWN FACT: edgeai currently has no tests directory; target-machine evidence is recipe output and drill logs, not a test suite.`

**Empirical Evidence:**
Inspecting `/home/sprime01/projects/edgeai/`:
- Directory `tests/` exists (size 1.3 MiB, 78 test scripts and python files).
- Subdirectories: `tests/config/`, `tests/fixtures/`, `tests/golden/`.
- Test files include: `test_api.sh`, `test_rag_e2e.py`, `test_sea_governance.py`, `test_t13_acceptance_oracle.py`, `test_t13_adversarial.py`.
- Governed by `tests/AGENTS.md` and repository recipes `just test-unit`, `just edgeai-test`.

**Finding:**
The "KNOWN FACT" in the plan is completely false. Stating that EdgeAI has no test suite leads implementing agents to bypass existing target-machine test fixtures and invent ad-hoc drill scripts rather than hooking into established EdgeAI test entrypoints.

---

### 1.8 Unauthored Prerequisite Harness Scripts

**Plan Citations:**
- T00 Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T00/validate_plan.py`
- T01 Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T01/run_pilot.py --read-only`
- T05 Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T05/joinability_test.py`
- T11 Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T11/run_substitution.py`
- T12 Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T12/verify_reproduction.py`
- T13 Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T13/verify_export.py`
- T14 Gate: `python3 .agents/evidence/godspeed-bounded-judgment/T14/verify_settlement.py`

**Empirical Evidence:**
Directory listing of `.agents/evidence/`:
```
bootstrap-config-remediation/
e2e/
```
The entire tree `.agents/evidence/godspeed-bounded-judgment/` **does not exist**.
Specifically for T00 (which has `initial_status: ready`):
- Line 448 requires `validate_plan.py`.
- Step 4 instructs: `Run the plan validation script (.agents/evidence/godspeed-bounded-judgment/T00/validate_plan.py)...`.
- It does not instruct the agent to write the script, nor is the script provided.

**Finding:**
An agent picking up the plan cold cannot even pass T00 because the gate command executes a non-existent script. Step 4 of T00 must explicitly author `validate_plan.py` or the script must be checked in as part of plan enablement.

---

## 2. High-Risk Requirement Conformance Downgrades

The normative specification `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` defines six high-risk requirement groups in `conformance_traceability.high_risk_requirement_groups` with mandatory confirmation modes.

The plan itself states the following binding rule at lines 278–280:
> *"Every group MUST be confirmed by a task whose `confirmation` mode matches or exceeds the declared mode. This is verified mechanically in `traceability`."*

Here is the adversarial evaluation of how the plan actually implements this:

```
┌─────────────────────────┬─────────────────────────────┬─────────────────────────────┬─────────────┐
│ High-Risk Group         │ Spec Mandated Mode          │ Plan Assigned Mode          │ Compliance  │
├─────────────────────────┼─────────────────────────────┼─────────────────────────────┼─────────────┤
│ authority_separation    │ adversarial                 │ independent_adversarial(T04)│ PASS        │
│ semantic_bounding       │ independent                 │ builder_with_teeth(T06,T11) │ VIOLATION   │
│ uncertainty_integrity   │ independent                 │ builder_with_teeth(T07)     │ VIOLATION   │
│ claim_scope_integrity   │ adversarial                 │ builder_with_teeth(T01)     │ VIOLATION   │
│ provenance_replay       │ fresh_target_reproduction   │ dev-host builder (T09, T13) │ VIOLATION   │
│ safe_failure            │ adversarial                 │ builder_with_teeth (T09)    │ VIOLATION   │
└─────────────────────────┴─────────────────────────────┴─────────────────────────────┴─────────────┘
```

### 2.1 Detailed Analysis of Violations

1. **`semantic_bounding` (`REQ-JUDG-001`, `002`, `003`, `020`, `021`, `024`)**
   - **Spec Mandate**: `confirmation: independent`.
   - **Plan Reality**: Assigned to T06 and T11, both configured with `confirmation: builder_with_teeth`.
   - **False Traceability Claim**: The plan writes `verdict: SATISFIED_WITH_FINAL_CONFIRMATION` claiming T14 provides the confirmation. But looking at T14's contract (lines 1914–1917):
     `verifier_receives: CLAIM-004 and CLAIM-005 normative text ... evidence from T04, T07, T09, T12, T13`.
     **T14's verifier does not receive T06 evidence, T11 evidence, or semantic bounding requirements.** Therefore, semantic bounding is never independently verified.

2. **`uncertainty_integrity` (`REQ-JUDG-012`, `013`, `022`)**
   - **Spec Mandate**: `confirmation: independent`. Spec explicit note: *"a distribution is easy to fabricate; this group exists so fabrication cannot pass unnoticed"*.
   - **Plan Reality**: Assigned to T07 (`confirmation: builder_with_teeth`).
   - **Defect**: Builder checks its own teeth. No independent confirmation packet is authored or evaluated for T07.

3. **`claim_scope_integrity` (`REQ-JUDG-023`, `REQ-OUT-006`, `007`, `REQ-VERIFY-012`, `CLAIM-006`)**
   - **Spec Mandate**: `confirmation: adversarial`. Spec explicit note: *"prevents a surface-level negative result, or a small-corpus metric, from being reported as a wider conclusion"*.
   - **Plan Reality**: Assigned to T01 (`confirmation: builder_with_teeth`).
   - **Defect**: The plan rationalizes this at line 2128 claiming "output is a MEASUREMENT, not a settlement-critical mechanism". But PROOF-1 is the gate for the entire capability branch! Downgrading adversarial confirmation on the very task that decides whether to build the plane violates the spec.

4. **`provenance_replay` (`REQ-PROV-001`, `003`, `004`, `020`, `021`, `022`)**
   - **Spec Mandate**: `confirmation: fresh_target_reproduction`.
   - **Plan Reality**: The traceability table lists `settled_by: ["T05", "T09", "T12", "T13"]`.
   - **Fatal Omission**: Inspecting `tasks.T12.settles` (lines 1623–1630):
     `settles: [REQ-VERIFY-003, REQ-SAFE-005, REQ-CONFIG-012, RECOV-001, RECOV-003, RECOV-004]`.
     **`REQ-PROV-001`, `REQ-PROV-003`, `REQ-PROV-004`, and `REQ-PROV-020` are completely absent from T12!**
     They are settled ONLY in T09 and T13 on the development host under `builder_with_teeth`. They are never settled or confirmed by `fresh_target_reproduction` on the deployment target.

5. **`safe_failure` (`REQ-SAFE-001` through `005`)**
   - **Spec Mandate**: `confirmation: adversarial`.
   - **Plan Reality**: `REQ-SAFE-002`, `003`, and `004` are settled exclusively in T09 (`builder_with_teeth`). They receive no adversarial confirmation.

---

## 3. Dependency Graph & Schedulability Flaws

### 3.1 Premature Schedulability of T13 (Replay Export)

**The Flaw:**
- `dependency_graph.edges`: `["T01", "T13"]`, `["T05", "T13"]`.
- `tasks.T13.depends_on`: `["T01", "T05"]`.
- `initial_state.blocked.T13`: `["T01", "T05"]`.

**The Contradiction:**
- In T13 `changes` (line 1756): `adds the read-only export path that completes T07's deliberately partial replay scope from T07`.
- In T13 `hot_context` (line 1785): `KNOWN FACT: T07's replay scope was deliberately partial; T13 completes it. Cite the T07 staging note.`
- In T13 `done_when` (line 1822): `The T07 partial-scope staging note is explicitly closed.`
- In T13 `steps` (line 1789): `export the joined chain (judgment, authority decision, execution, observation, settlement)`.

**Impact:**
Because T13 depends only on T01 and T05, once T00, T01, and T05 settle, **T13 immediately becomes ready to execute**. An orchestrator or subagent will begin T13 before T06 (AnswerDomain) and T07 (Distribution Carrier) have even been designed or implemented. T13 cannot write an export path for carrier fields that do not exist.
**Required Fix:** Add `["T07", "T13"]` to `edges`, `depends_on`, and `initial_state.blocked`.

```
CURRENT (Broken):
  T01 ──┐
        ├──► T13 (Runs BEFORE T06 & T07 exist!)
  T05 ──┘
  T01 ──► T06 ──► T07 (Carrier fields designed here)

CORRECTED:
  T01 ──► T06 ──► T07 ──┐
  T05 ──────────────────┼──► T13
```

---

### 3.2 Dangling Hygiene Tasks (T02 & T03)

**The Flaw:**
- T02 (`Gauntlet CI`) and T03 (`Repair red sea-forge-server gate`) depend on T00.
- Neither T02 nor T03 has any outgoing edge to any subsequent task.
- T14 depends only on `["T04", "T12", "T13"]`.

**Impact:**
A dependency-driven runner will schedule critical path tasks through T14. Once T04, T12, and T13 finish, T14 becomes eligible. T14 checks `just ci` against the T00 baseline. Because T03 was never run, the server gate was red at baseline and remains red; T14 observes "no new regression versus T00 baseline" and confirms final settlement.
The entire GodSpeed Bounded-Judgment Capability can thus claim complete settlement while:
1. `sea-forge-server` has a broken restart conformance test.
2. Gauntlet has zero CI workflows.
**Required Fix:** T02 and T03 must be explicit dependencies of T14: `tasks.T14.depends_on: ["T02", "T03", "T04", "T12", "T13"]`.

---

### 3.3 Hidden Prerequisite Between T08 and T03

**The Flaw:**
- T08 gate includes: `just crate-test sea-forge-server`.
- Line 72 of `.agents/CURRENT_STATUS.md` documents: `just crate-test sea-forge-server` fails on `conformance_run_locator::both_layouts_still_resolve_after_a_restart`.
- T03 is the dedicated task to repair this exact failure.
- T08 has no dependency on T03 (`depends_on: ["T06"]`).

**Impact:**
If T08 runs in parallel with or before T03, T08's gate fails on the pre-existing server restart test. T08 must declare dependency on T03 or restrict its gate to focused provider tests.

---

## 4. Traceability & Semantic Misalignments

### 4.1 CLAIM-006 Premature Settlement

**The Spec Definition:**
`CLAIM-006` explicitly defines three distinct comparability levels:
- **Level A (Interface)**: Can represent bounded question and distribution. (`status: supported_now`)
- **Level B (Operational)**: Can execute contract with suitable latency, retries, and substitution. (`status: requires_proof_1_and_target_hardware`)
- **Level C (Model Quality)**: Calibration/quality comparable to external service. (`status: unproven`)

**The Plan Defect:**
In `traceability.requirement_to_tasks`:
`"CLAIM-006": ["T01"]`.
T01 settles CLAIM-006 in its entirety at Task 1!
Yet in T01 line 636, the plan states:
`inference_forbidden: "Level B operational comparability (requires T12), level C model-quality comparability (unproven)..."`
If Level B requires T12, `CLAIM-006` cannot be settled by T01 alone. It must be mapped to `["T01", "T11", "T12"]`, with Level A settled in T01, Level B settled in T11/T12, and Level C explicitly marked unproven.

---

### 4.2 Misplaced Recovery Invariants (`RECOV-003`, `RECOV-004`)

1. **`RECOV-003`**: *"Semantic pack or surface version mismatch. Expected: Reject before provider invocation or downstream consumption. Enforced by: REQ-JUDG-021."*
   - Plan maps `RECOV-003` only to T12 (Jetson live pilot).
   - In reality, T06 implements the DecisionSurface and AnswerDomain contract and REQ-JUDG-021. T06 teeth explicitly test: *"Change a label's meaning in place without bumping the domain version -> version check rejects"*.
   - `RECOV-003` belongs in T06.

2. **`RECOV-004`**: *"Duplicate request identity with different content. Expected: Conflict error; never overwrite prior evidence."*
   - Plan maps `RECOV-004` only to T12 (Jetson live pilot).
   - In reality, T05 implements Loop Correlation Identity and tests duplicate external IDs. T05 teeth explicitly test: *"Duplicate external id -> Rejected as a conflict, never silently joined"*.
   - `RECOV-004` belongs in T05.

---

### 4.3 Coexistence Claims (`CLAIM-004`, `CLAIM-005`) Omitted from T07

- In T07 `proves`: *"That uncertainty-preserving output can be carried as typed fields alongside the deterministic basis without displacing it (CLAIM-004), that both bases are attributable (CLAIM-005)..."*
- But in `tasks.T07.settles`: `CLAIM-004` and `CLAIM-005` are omitted!
- They are only listed in T14. T07 should declare partial settlement (carrier-level coexistence and explicit absence), while T14 provides final full-system confirmation.

---

## 5. Remediation Blueprint (Exact Changes Required)

To restore `godspeed-bounded-judgment-plan.yaml` to an authoritative, executable, and well-grounded state, another agent must apply the following concrete modifications.

### Change 1: Correct Gauntlet Crate Names and Working Directories

In tasks T07, T08, T09, T10, T11:
- Replace `gauntlet-engine` with `gauntlet-app` and `gauntlet-adapter-state-sqlite`.
- Replace `gauntlet-storage` with `gauntlet-adapter-state-sqlite`.
- Replace `gauntlet-kernel` with `gauntlet-app` and `gauntlet-ports`.
- In T06, replace `<implementation-crate>` with `gauntlet-domain`.
- Prefix all Gauntlet test executions with directory changes: `(cd ../gauntlet && cargo test -p <pkg>)`.

*Replacement Gate Blocks:*
```yaml
# T06 Gate:
gate:
  - "(cd ../gauntlet && cargo check -p gauntlet-domain)"
  - "(cd ../gauntlet && cargo test -p gauntlet-domain)"

# T07 Gate:
gate:
  - "(cd ../gauntlet && cargo test -p gauntlet-domain)"
  - "(cd ../gauntlet && cargo test -p gauntlet-ports)"
  - "(cd ../gauntlet && cargo test -p gauntlet-adapter-state-sqlite)"

# T08 Gate:
gate:
  - "just no-async-kernel"
  - "(cd ../gauntlet && cargo test -p gauntlet-ports)"
  - "(cd ../gauntlet && cargo test -p gauntlet-adapter-agent-http)"
  - "just crate-test sea-forge-server"

# T09 Gate:
gate:
  - "(cd ../gauntlet && cargo test -p gauntlet-domain)"
  - "(cd ../gauntlet && cargo test -p gauntlet-adapter-state-sqlite)"

# T10 Gate:
gate:
  - "(cd ../gauntlet && cargo test -p gauntlet-app)"

# T11 Gate:
gate:
  - "python3 .agents/evidence/godspeed-bounded-judgment/T11/run_substitution.py"
  - "(cd ../gauntlet && cargo test -p gauntlet-ports)"
```

---

### Change 2: Fix T12 Gate Command Syntax

In Task T12, replace lines 1694–1698:
```yaml
gate:
  - "just --justfile ../edgeai/Justfile health"
  - "just --justfile ../edgeai/Justfile jetson-check-ports"
  - "python3 .agents/evidence/godspeed-bounded-judgment/T12/verify_reproduction.py"
```

---

### Change 3: Fix Corpus Evidence Paths

In T01 (lines 500, 572), T13 (line 1782), and Brief line 50:
Replace `../gauntlet/evidence/objects/` and `../gauntlet/evidence/pins/` with:
`../gauntlet/targets/.runs/*/evidence/objects/` and `../gauntlet/targets/.runs/*/evidence/pins/`.

---

### Change 4: Correct EdgeAI Facts in T12 Hot Context

In Task T12, replace line 1684 with:
```yaml
- "KNOWN FACT: edgeai has an extensive test suite in tests/ (unit, integration, and benchmark suites governed by tests/AGENTS.md). Target-machine evidence uses Justfile recipes (health, jetson-check-ports) and targeted recovery drills."
```

---

### Change 5: Fix DAG Dependencies

1. **T13 Dependencies**:
   Update line 300, line 388, line 1743:
   ```yaml
   depends_on: ["T01", "T05", "T07"]
   ```
   Add edge: `["T07", "T13"]` to `dependency_graph.edges`.
   Update `initial_state.blocked.T13: ["T01", "T05", "T07"]`.

2. **T14 Dependencies (Tie in hygiene tasks)**:
   Update line 302, line 389, line 1829:
   ```yaml
   depends_on: ["T02", "T03", "T04", "T12", "T13"]
   ```
   Add edges: `["T02", "T14"]`, `["T03", "T14"]` to `dependency_graph.edges`.
   Update `initial_state.blocked.T14: ["T02", "T03", "T04", "T12", "T13"]`.

3. **T08 Dependencies**:
   Add dependency on T03 so `sea-forge-server` tests pass:
   ```yaml
   depends_on: ["T03", "T06"]
   ```
   Add edge: `["T03", "T08"]` to `dependency_graph.edges`.
   Update `initial_state.blocked.T08: ["T03", "T06"]`.

---

### Change 6: Restore High-Risk Confirmation Conformance

1. **T06 (Semantic Bounding)**:
   Change `confirmation` from `builder_with_teeth` to `independent`.
   Add `independent_confirmation` section to T06 matching the pattern in T04/T05.

2. **T07 (Uncertainty Integrity)**:
   Change `confirmation` from `builder_with_teeth` to `independent`.
   Add `independent_confirmation` section to T07 explicitly verifying that distribution rows are not fabricated.

3. **T01 (Claim Scope Integrity)**:
   Change `confirmation` from `builder_with_teeth` to `independent_adversarial`.
   Have an independent verifier execute the label-shuffle and label-leakage teeth.

4. **T12 (Provenance Replay)**:
   Add `REQ-PROV-001`, `REQ-PROV-003`, `REQ-PROV-004`, `REQ-PROV-020` to `tasks.T12.settles`.
   Require T12's fresh target reproduction to re-prove provider attribution and forward provenance on the real machine.

5. **T09 (Safe Failure)**:
   Change `confirmation` from `builder_with_teeth` to `independent_adversarial`.

---

### Change 7: Traceability Requirement Relocations

In `traceability.requirement_to_tasks`:
- `CLAIM-006`: change from `["T01"]` to `["T01", "T11", "T12"]`.
- `RECOV-003`: change from `["T12"]` to `["T06", "T12"]`.
- `RECOV-004`: change from `["T12"]` to `["T05", "T12"]`.
- `CLAIM-004`: change from `["T14"]` to `["T07", "T14"]`.
- `CLAIM-005`: change from `["T14"]` to `["T07", "T14"]`.
- `REQ-PROV-001`: change from `["T09"]` to `["T09", "T12"]`.
- `REQ-PROV-003`: change from `["T09"]` to `["T09", "T12"]`.
- `REQ-PROV-004`: change from `["T13"]` to `["T13", "T12"]`.
- `REQ-PROV-020`: change from `["T09"]` to `["T09", "T12"]`.

---

### Change 8: Authoring `validate_plan.py` for T00

To make T00 immediately executable, update T00 steps to create `.agents/evidence/godspeed-bounded-judgment/T00/validate_plan.py` before running it, using the following exact Python script:

```python
#!/usr/bin/env python3
"""Mechanical validation script for godspeed-bounded-judgment-plan.yaml."""
import hashlib
import sys
import yaml

SPEC_PATH = ".agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml"
PLAN_PATH = ".agents/plans/godspeed-bounded-judgment-plan.yaml"

with open(PLAN_PATH, "r") as f:
    plan = yaml.safe_load(f)

with open(SPEC_PATH, "rb") as f:
    spec_bytes = f.read()

# 1. SHA256 Verification
calc_sha = hashlib.sha256(spec_bytes).hexdigest()
bound_sha = plan["source"]["spec"]["sha256"]
if calc_sha != bound_sha:
    print(f"FAIL: SHA256 mismatch! spec={calc_sha} bound={bound_sha}")
    sys.exit(1)

# 2. Spec Self-Check & Corrections
spec = yaml.safe_load(spec_bytes.decode("utf-8"))
final_check = spec.get("final_self_check", {}).get("statements", [])
if len(final_check) != 12 or not all(s.get("holds") is True for s in final_check):
    print("FAIL: Spec final_self_check does not hold 12/12!")
    sys.exit(1)

corr_ledger = spec.get("correction_ledger", {})
if len(corr_ledger.get("corrections", [])) != 11:
    print("FAIL: Expected 11 corrections in spec ledger!")
    sys.exit(1)

if corr_ledger.get("architectural_conclusions_reversed") != "none":
    print("FAIL: Architectural conclusions were reversed!")
    sys.exit(1)

# 3. Requirement Mapping Check
req_to_tasks = plan["traceability"]["requirement_to_tasks"]
tasks = plan["tasks"]

all_settles = set()
for t_id, t_data in tasks.items():
    all_settles.update(t_data.get("settles", []))

if len(req_to_tasks) != 90 or len(all_settles) != 90:
    print(f"FAIL: Expected 90 requirements mapped, got {len(req_to_tasks)} in traceability and {len(all_settles)} in tasks!")
    sys.exit(1)

# 4. DAG Acyclicity
edges = plan["dependency_graph"]["edges"]
adj = {t: [] for t in tasks}
for u, v in edges:
    adj[u].append(v)

visited = {}
def dfs(node):
    visited[node] = 1
    for neighbor in adj[node]:
        if visited.get(neighbor) == 1:
            return False
        if visited.get(neighbor) == 0 and not dfs(neighbor):
            return False
    visited[node] = 2
    return True

for t in tasks:
    visited[t] = 0

for t in tasks:
    if visited[t] == 0:
        if not dfs(t):
            print("FAIL: Cycle detected in dependency graph!")
            sys.exit(1)

print("PASS: Plan validation succeeded. 90/90 requirements mapped, DAG acyclic, spec hash verified.")
sys.exit(0)
```

---

## 6. Handoff Summary for Continuing Agent

| Field | Detail |
|---|---|
| **Inspection Outcome** | Plan rejected in current state; full remediation blueprint provided. |
| **Primary Root Cause** | Plan authored in `sea-rs` assumed hypothetical Gauntlet crate names (`gauntlet-engine`, `gauntlet-storage`, `gauntlet-kernel`) and non-existent `just crate-test` command surface. |
| **Secondary Root Cause** | High-risk confirmation modes weakened to `builder_with_teeth` without spec amendment; DAG edges left T13 premature and T02/T03 dangling. |
| **Immediate Next Move** | Apply the 8 concrete changes in §5 to `/home/sprime01/projects/sea-rs/.agents/plans/godspeed-bounded-judgment-plan.yaml`, bootstrap `.agents/evidence/godspeed-bounded-judgment/T00/validate_plan.py`, and run T00. |

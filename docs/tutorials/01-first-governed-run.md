# Tutorial: Your First Governed Run

> **A guided, step-by-step walkthrough executing your first governed workload using the `sea-forge` CLI.**

---

## Prerequisites

Before starting this tutorial, ensure you have:

* Cloned this repository.
* Installed Rust 1.92.0+ (or [Devbox](https://www.jetify.com/devbox) to activate the pinned toolchain automatically).
* Access to a terminal shell (`bash`, `zsh`, or `pwsh`).

To verify your environment, run from the repository root:

```sh
cargo --version
```

Expected output:

```text
cargo 1.92.0 (or newer)
```

---

## 1. Build the CLI Binary

Build the `sea-forge` command-line tool using Cargo:

```sh
cargo build -p sea-forge-cli
```

Verify the binary works:

```sh
./target/debug/sea-forge --help
```

You will see the top-level help text listing commands like `run`, `case`, `inspect`, and `recall`.

---

## 2. Write a Policy Bundle

SEA Forge is default-deny. Every run needs a policy bundle that explicitly allows the operations it will perform.

Create `/tmp/tutorial-policy.yaml`:

```sh
cat > /tmp/tutorial-policy.yaml <<'YAML'
version: "0.1"
identity:
  source: tutorial-cell
  allow_unresolved: false

rules:
  - name: allow-demo-write
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: ""

  - name: allow-demo-validate
    verdict: allow
    actor_role: operator
    operation_kind: execute_command
    argv0: sea-forge
    sandbox_class: jail
YAML
```

The first rule allows the operator to write the demo model. The second allows the trusted `sea-forge` validator to run inside the `jail` sandbox class.

---

## 3. Execute a Governed Run

Run the CLI with the exact demo intent:

```sh
./target/debug/sea-forge run \
  --intent "generate a .sea model" \
  --policy /tmp/tutorial-policy.yaml \
  --root /tmp/tutorial-cell
```

The command prints:

```text
run_id=run_20260902T190000Z_abcdef
authority=auth_01:allow
authority=auth_02:allow
execution=completed
settlement=accepted
run_dir=/tmp/tutorial-cell/runs/run_20260902T190000Z_abcdef
```

SEA Forge does not simply say “Success”. It reports:

1. The unique **Run ID**.
2. Each **authority verdict**.
3. The **execution result**.
4. The **settlement status**.
5. The **run directory** containing all records.

The command exits with code `0` only when settlement accepts the declared outcome.

---

## 4. Inspect the Created Records

List the run directory:

```sh
RUN_ID=$(ls /tmp/tutorial-cell/runs | head -n 1)
ls -la /tmp/tutorial-cell/runs/$RUN_ID
```

You will see six core governance files and two directories:

```text
├── plan.json                 # The deterministic plan formulated for the intent
├── authority.json            # The authority evaluation and allow decisions
├── trace.jsonl               # Chronological log of lifecycle state transitions
├── evidence.jsonl            # Captured artifact hashes and execution results
├── settlement.json           # Outcome evaluation with basis tokens
├── semantic-envelope.json    # Complete summary envelope of the run
├── workspace/                # The isolated directory where the process executed
└── artifacts/                # Captured work products (stdout.txt, stderr.txt, model.sea)
```

---

## 5. Verify Settlement Truth

View `settlement.json`:

```sh
cat /tmp/tutorial-cell/runs/$RUN_ID/settlement.json
```

Output:

```json
{
  "version": "0.1",
  "settlement_id": "set_...",
  "run_id": "run_...",
  "status": "accepted",
  "basis": [
    "authority_allow",
    "exit_zero",
    "required_artifact_present:model.sea",
    "stdout_match"
  ],
  "review_required": false,
  "settled_at": "...",
  "criteria_ref": "crit_..."
}
```

Inspect the captured work product:

```sh
cat /tmp/tutorial-cell/runs/$RUN_ID/artifacts/model.sea
```

Output:

```json
{"domain": "demo", "entities": [{"name": "Sample"}]}
```

---

## 6. Recall the Capability

Because the run settled as `Accepted`, its capability envelope was appended to `capabilities.jsonl`.

Search for it with `sea-forge recall`:

```sh
./target/debug/sea-forge recall "generate" --root /tmp/tutorial-cell
```

The command prints the full capability envelope as one line of JSON. Line breaks are added here for readability:

```json
{
  "version": "0.1",
  "run_id": "run_...",
  "case_ref": "case_...",
  "intent": { "summary": "generate a .sea model" },
  "plan_ref": "plan_01",
  "authority_decisions": ["auth_01", "auth_02"],
  "evidence_refs": ["evi_0001", "evi_0002", "evi_0003", "..."],
  "settlement_ref": "set_...",
  "capability_delta": {
    "attempted_capability": "generate_and_validate_sea_model",
    "result": "accepted"
  },
  "artifact_refs": [
    { "artifact_id": "art_...", "pre_mint_identity": "ifl:hash:..." }
  ]
}
```

---

## 7. What Just Happened?

Behind the scenes:

1. `sea-forge-domain` parsed your intent into `IntentPattern::Demo`.
2. `sea-forge-planner` constructed a two-operation plan: write `model.sea`, then run the trusted validator on it.
3. `sea-forge-authority` evaluated both operations against the policy bundle and minted an `ActionGrant` for each allowed action.
4. `sea-forge-runtime` executed the validator inside a sandbox and captured `stdout`, `stderr`, and the artifact.
5. `sea-forge-settlement` verified the committed criteria independently of the process exit code.
6. `sea-forge-capability` appended the accepted capability envelope to the cell's append-only journal.

To understand why this lifecycle is designed this way, read the [Mental Model](../mental-model.md) and [Settlement vs. Process Exit](../explanation/settlement-vs-process-exit.md). For an interactive version of the same journey, open the [Governed Run Explorer](../explainers/governed-run-explorer.html).

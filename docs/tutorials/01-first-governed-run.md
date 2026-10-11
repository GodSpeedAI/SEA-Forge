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

## 2. Execute a Governed Run

Execute a basic governed intent that creates and validates a DomainForge model:

```sh
./target/debug/sea-forge run --intent "generate demo model" --root /tmp/tutorial-cell
```

### What You Will See in the Output:

```text
Run run_20260902T190000Z_abcdef completed with exit code 0
Settlement: Accepted
Basis: authority_allow, exit_zero, required_artifact_present:model.sea, stdout_match
```

Notice that the CLI does not simply say "Success". It reports:
1. The unique **Run ID** (`run_<timestamp>_<hex>`).
2. The **Settlement Status** (`Accepted`).
3. The factual **Basis** justifying why the outcome was accepted.

---

## 3. Inspect the Created Records

Let us look at what SEA Forge created inside `/tmp/tutorial-cell`:

List the contents of the run directory:
```sh
ls -la /tmp/tutorial-cell/runs/
```

Navigate into the created run folder (substituting your actual Run ID):
```sh
RUN_ID=$(ls /tmp/tutorial-cell/runs | head -n 1)
ls -la /tmp/tutorial-cell/runs/$RUN_ID
```

You will see six core governance files and two directories:
```text
├── plan.json                 # The deterministic plan formulated for the intent
├── authority.json            # The authority evaluation and allow decision
├── trace.jsonl               # Chronological log of lifecycle state transitions
├── evidence.jsonl            # Captured artifact hashes and execution results
├── settlement.json           # Outcome evaluation with basis tokens
├── semantic-envelope.json    # Complete summary envelope of the run
├── workspace/                # The isolated directory where the process executed
└── artifacts/                # Captured work products (stdout.txt, stderr.txt, model.sea)
```

---

## 4. Verify Settlement Truth

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
  "settled_at": "..."
}
```

Now inspect the captured work product in `artifacts/`:
```sh
cat /tmp/tutorial-cell/runs/$RUN_ID/artifacts/model.sea
```
Output:
```json
{"domain": "demo", "entities": [{"name": "Sample"}]}
```

---

## 5. Recall the Capability from Memory

Because the run settled as `Accepted`, it was appended to the cell's capability memory (`capabilities.jsonl`).

Search for the recorded capability using `sea-forge recall`:
```sh
./target/debug/sea-forge recall "demo" --root /tmp/tutorial-cell
```

Expected output:
```text
Found 1 matching capability:
  Run: run_...
  Item: generate_and_validate_sea_model
  Result: accepted
  Artifacts: model.sea (ifl:hash:...)
```

---

## 6. What Just Happened?

Behind the scenes:
1. `sea-forge-domain` parsed your intent into `IntentPattern::Demo`.
2. `sea-forge-planner` constructed a plan item requiring `model.sea` and stdout confirmation.
3. `sea-forge-authority` verified that writing `model.sea` and running `validate` were permitted by policy, minting an `ActionGrant`.
4. `sea-forge-runtime` executed the task inside an isolated sandbox, writing to `workspace/`.
5. `sea-forge-settlement` verified that the file existed, had content, and matched stdout assertions.
6. The result was appended to the append-only capability journal.

To understand why this lifecycle is designed this way, read the [Mental Model](file:///c:/Users/sprim/projects/sea-rs/docs/mental-model.md) and [Settlement vs. Process Exit](file:///c:/Users/sprim/projects/sea-rs/docs/explanation/settlement-vs-process-exit.md).

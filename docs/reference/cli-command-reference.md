# Reference: CLI Command Manual (`sea-forge`)

> **Exhaustive reference manual for all 24 subcommands, arguments, flags, and exit codes of the `sea-forge` command-line interface.**

---

## Exit Code Taxonomy

The `sea-forge` CLI returns deterministic exit codes communicating the exact outcome of execution:

| Exit Code | Classification | Meaning |
|---|---|---|
| `0` | **Success / Accepted** | Work completed and settlement evaluated as `Accepted`. All criteria met. |
| `1` | **Rejected / Failure** | Work ran or halted, but settlement evaluated as `Rejected` or internal error occurred. |
| `2` | **Unknown Intent** | Intent string did not match any recognized `IntentPattern`. Fails closed before execution. |
| `3` | **Generated Zone Denial** | Operation attempted a forbidden direct write to a protected generated zone (`src/gen/`). |

---

## Master Subcommand Reference

### 1. `sea-forge run`
Executes a one-shot governed workload episode.
```sh
sea-forge run [--intent <STRING> | --plan <PATH>] [OPTIONS]
```
* `--intent <STRING>`: Raw operator intent (e.g. `"generate demo model"`). Conflicts with `--plan`.
* `--plan <PATH>`: Path to pre-formulated `CasePlan` JSON file. Conflicts with `--intent`.
* `--policy <PATH>`: Path to policy bundle YAML (default: `sea-forge-policy.yaml`).
* `--root <PATH>`: Path to cell root (default: `.sea-forge`).
* `--timeout <SECS>`: Execution wall-clock timeout in seconds (default: `60`).
* `--entity <STRING>`: Actor entity identifier (default: `"operator_local"`).
* `--process <STRING>`: Ingress process label (default: `"cli"`).

---

### 2. `sea-forge runs`
Inspects active and historical runs in the cell.
```sh
sea-forge runs [--unsettled] [--root <PATH>]
```
* `--unsettled`: Filters to runs that have not yet reached final settlement.

---

### 3. `sea-forge inspect`
Displays the full governance record of a completed run.
```sh
sea-forge inspect <RUN_ID> [--root <PATH>] [--policy <PATH>] [--entity <STRING>]
```
* `<RUN_ID>`: Run identifier (`run_<timestamp>_<hex>`).

---

### 4. `sea-forge recall`
Searches capability memory (`capabilities.jsonl`) or semantic memory items (`items.jsonl`).
```sh
sea-forge recall <QUERY> [OPTIONS]
```
* `<QUERY>`: Search string.
* `--kind <fact|decision|outcome|preference>`: Filter by memory item kind.
* `--result <accepted|rejected|escalated>`: Filter by settlement status.
* `--limit <N>`: Maximum results returned (default: `10`).
* `--rebuild`: Reconstructs the derived SQLite FTS index from source journals.

---

### 5. `sea-forge case`
Manages CMMN-subset case lifecycles.
```sh
sea-forge case <SUBCOMMAND> [--root <PATH>]
```
* `case create --intent <STR>`: Initializes a new case instance.
* `case reopen <CASE_ID>`: Reopens a terminated or completed case.
* `case add-task <CASE_ID> --plan-item <PATH>`: Adds a discretionary plan item.
* `case status <CASE_ID>`: Prints current case state and item standings.
* `case horizon <CASE_ID>`: Displays dependency graph and ready sentry transitions.
* `case manager-iterate <CASE_ID>`: Triggers one iteration of the Thoth ADLC manager loop.

---

### 6. `sea-forge approve` & `reject`
Resolves pending human approval requests in `approvals.jsonl`.
```sh
sea-forge approve <CASE_ID> <APPROVAL_ID> [--note <STRING>] [--actor <STRING>]
sea-forge reject <CASE_ID> <APPROVAL_ID> [--note <STRING>] [--actor <STRING>]
```
* `<CASE_ID>`: Target case ID.
* `<APPROVAL_ID>`: Sequential approval ID (`apr_0001`).
* `--note <STRING>`: Optional audit explanation recorded in the decision entry.

---

### 7. `sea-forge resume`
Resumes a case episode that was halted awaiting approval or recovery.
```sh
sea-forge resume <CASE_ID> [OPTIONS]
```

---

### 8. `sea-forge ledger`
Inspects and cryptographically verifies integrity ledgers.
```sh
sea-forge ledger verify <STREAM_ID> [--root <PATH>]
sea-forge ledger prove <STREAM_ID> <ENTRY_ULID> [--root <PATH>]
sea-forge ledger replay --case <CASE_ID> [--root <PATH>]
```
* `verify`: Validates entry hash chains and MMR root consistency.
* `prove`: Emits a compact `MerkleProof` demonstrating inclusion of an entry.
* `replay`: Reconstructs chronological dispatch and settlement sequence from `case-events.jsonl`.

---

### 9. `sea-forge project`
Executes an M5 spec-to-code pipeline and emits in-memory projections.
```sh
sea-forge project <ENTRY_PATH> [--projection <calm|rdf>] [--root <PATH>]
```

---

### 10. `sea-forge self-model`
Governs the Genesis self-model (spec-adlc-thoth E11).
```sh
sea-forge self-model validate [--root <PATH>]
sea-forge self-model rebuild [--root <PATH>]
sea-forge self-model show [--root <PATH>]
```

---

### 11. `sea-forge ask`
Queries Thoth about system capability, self-model concepts, and architecture rules.
```sh
sea-forge ask <KIND> <SUBJECT> [--json] [--root <PATH>]
```
* `<KIND>`: Question kind (e.g. `"capability"`, `"concept"`).
* `<SUBJECT>`: Target concept or capability name.

---

### 12. `sea-forge agent`
Governs autonomous agent connections.
```sh
sea-forge agent probe --endpoint <NAME> [--root <PATH>]
sea-forge agent delegate --endpoint <NAME> --instruction <STR> [--max-turns <N>]
sea-forge agent cancel --delegation-id <ID> [--root <PATH>]
```

---

### 13. `sea-forge artifact`
Governs artifact lifecycle transitions (M8).
```sh
sea-forge artifact productize <ARTIFACT_ID> [--root <PATH>]
sea-forge artifact capitalize <ARTIFACT_ID> [--root <PATH>]
sea-forge artifact attest <ARTIFACT_ID> [--root <PATH>]
```

---

### 14. Federation Commands (`export`, `import`, `adopt`)
Manages SeaCell federation bundles.
```sh
sea-forge export --run-ids <ID...> --out <BUNDLE.tar>
sea-forge import --bundle <BUNDLE.tar>
sea-forge adopt <CELL_ID> <TEMPLATE_REF>
```

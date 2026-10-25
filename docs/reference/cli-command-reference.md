# Reference: CLI Command Manual (`sea-forge`)

> **Operator reference for the visible `sea-forge` command surface, its flags, and the exit codes the process actually returns.**  
> Source of truth: `crates/sea-forge-cli/src/main.rs` and `crates/sea-forge-cli/src/commands/`.

The binary exposes **21 visible top-level commands**. Four further commands are hidden test and pipeline seams (`validate`, `stage-check`, `internal-test-sleep`, `internal-test-swe-seed`); they are documented at the end because scripts may encounter them, not because operators should call them.

---

## 1. Exit Codes

Exit codes are not a single success/failure bit. The meaning depends on the command family, because a settled rejection and a parked case are different observable outcomes.

| Code | Meaning | Where it applies |
|---|---|---|
| `0` | Accepted, completed, answered, or ordinary success | `run` accepted, `run --plan` case completed, `resume` completed, `ask` answered/partial, `agent probe`/`delegate` accepted, most read/query commands |
| `1` | Operational error, invalid input/model, or failed verification | Dispatch and internal errors; `validate` invalid; `stage-check` mismatch; `ledger verify` failed; socket/IO failures |
| `2` | CLI rejected the request before execution | `run` dispatch error for `Input`, `UnknownIntent`, or `Plan` |
| `3` | Settled `Rejected`, case terminated, or agent probe/delegation rejected | `run`, `run --plan`, `resume`, `agent probe`, `agent delegate` |
| `4` | Escalated, or a governed denial that is a successful governance outcome | `run`, `resume`; `ask` denied |
| `5` | Work remains: awaiting approval, parked human task, or no ready action | `run --plan`, `resume`, `case manager-iterate` |

Two consequences matter:

* **A governed denial is not necessarily `1`.** `ask` returns `4` for a denied disclosure, and `run` returns `3` or `4` after settlement. Use the printed settlement record, not the shell status alone, to distinguish a policy decision from a crash.
* **`5` is not failure.** It means the case is still alive. The operator must approve, complete a human task, or resume it.

Exit-code sources: [`main.rs`](../../crates/sea-forge-cli/src/main.rs), [`run.rs`](../../crates/sea-forge-cli/src/commands/run.rs), [`plan_pipeline.rs`](../../crates/sea-forge-cli/src/plan_pipeline.rs), [`resume.rs`](../../crates/sea-forge-cli/src/commands/resume.rs), [`ask.rs`](../../crates/sea-forge-cli/src/commands/ask.rs), [`agent.rs`](../../crates/sea-forge-cli/src/commands/agent.rs).

---

## 2. Common Conventions

* `--root <PATH>` defaults to `.sea-forge`. Most commands read or write the cell at that path.
* `--policy <PATH>` defaults to `sea-forge-policy.yaml` where the command requires a policy, or to `<root>/authority/active-policy.json` where the flag is optional. Direct policy flags are not interchangeable between command families.
* `--entity` or `--actor` defaults to `operator_local`. This is an attribution claim, not a capability: the server checks it against the connection's kernel-reported uid when the command crosses SFWP.
* Commands that cross the server socket require `sea-forge-server` to be running on `<root>/server.sock`.
* Commands print human-readable lines by default. `--json` exists only on the commands that declare it (`ask`, `self-model show`).

---

## 3. Execution Commands

### `sea-forge run`

Executes one governed intent, or advances a supplied case plan.

```sh
sea-forge run --intent <STRING> [OPTIONS]
sea-forge run --plan <PATH> [OPTIONS]
```

`--intent` and `--plan` are mutually exclusive and one is required.

| Flag | Default | Meaning |
|---|---|---|
| `--intent <STRING>` | — | Operator intent interpreted by the domain crate |
| `--plan <PATH>` | — | Pre-formulated `CasePlan` JSON |
| `--policy <PATH>` | `sea-forge-policy.yaml` | Authority policy bundle |
| `--root <PATH>` | `.sea-forge` | Cell root |
| `--timeout <SECS>` | `60` | Per-execution wall-clock timeout |
| `--entity <STRING>` | `operator_local` | Actor entity for attribution |
| `--process <STRING>` | `cli` | Ingress process label |

`--intent` creates a run, evaluates each operation's authority, executes allowed operations, settles the result, and prints `run_id`, `authority`, `execution`, `settlement`, and `run_dir`.

`--plan` advances a CMMN-style case. It may complete one or more ready items, park on approval, park on a human task, or report that no action is ready. It prints `case_id` and `case_state`.

The run directory and its `workspace/` and `artifacts/` children are created as part of run setup. Authority gates execution and workload writes, not the existence of those control-plane directories. See [CLI run lifecycle](../workflows/cli-run-lifecycle.md).

### `sea-forge runs`

Lists known runs and their case state, read from `cases/`.

```sh
sea-forge runs [--unsettled] [--root <PATH>]
```

* `--unsettled` omits runs that already have a matching `settlement.json`.
* Output is tab-separated: `run_id`, `case_id`, `case_state`.

### `sea-forge resume`

Continues a case that was parked awaiting approval, a human task, or another resumable state.

```sh
sea-forge resume <CASE_ID> [OPTIONS]
```

| Flag | Default |
|---|---|
| `--policy <PATH>` | `sea-forge-policy.yaml` |
| `--root <PATH>` | `.sea-forge` |
| `--timeout <SECS>` | `60` |
| `--entity <STRING>` | `operator_local` |
| `--process <STRING>` | `cli` |

Returns `0`, `3`, `4`, or `5` with the same settlement/parking meanings as `run`.

### `sea-forge project`

Runs a spec-to-code pipeline as governed case episodes, then settles CALM/RDF projection records from the validated DomainForge model.

```sh
sea-forge project <ENTRY_PATH> [OPTIONS]
```

| Flag | Default | Meaning |
|---|---|---|
| `--root <PATH>` | `.sea-forge` | Cell root |
| `--policy <PATH>` | `sea-forge-policy.yaml` | Authority policy bundle |
| `--entity <STRING>` | `operator_local` | Actor entity |
| `--projection <KIND>` | `calm`, `rdf` | Repeatable projection kind |

---

## 4. Inspection and Memory Commands

### `sea-forge inspect`

Prints the six-record episode view of a run: plan, authority, trace, evidence, settlement, and semantic envelope.

```sh
sea-forge inspect <RUN_ID> [OPTIONS]
```

* `--root <PATH>` defaults to `.sea-forge`.
* `--policy <PATH>` overrides the active policy path.
* `--entity <STRING>` defaults to `operator_local`.

### `sea-forge recall`

Searches capability envelopes, or memory items when `--kind` is supplied.

```sh
sea-forge recall <QUERY> [OPTIONS]
```

| Flag | Default | Meaning |
|---|---|---|
| `--root <PATH>` | `.sea-forge` | Cell root |
| `--entity <STRING>` | — | Optional entity filter |
| `--process <STRING>` | — | Optional process filter |
| `--result <accepted\|rejected\|escalated>` | — | Settlement filter |
| `--kind <fact\|decision\|outcome\|preference>` | — | Search `memory/items.jsonl` instead of `capabilities.jsonl` |
| `--limit <N>` | `10` | Maximum results |
| `--policy <PATH>` | active policy | Authority policy |
| `--actor <STRING>` | `operator_local` | Actor entity |

### `sea-forge memory rebuild`

Rebuilds the derived full-text search index from `memory/items.jsonl`.

```sh
sea-forge memory rebuild [--root <PATH>]
```

The index is derived state; the journal remains authoritative.

### `sea-forge ledger`

Verifies and proves integrity-ledger entries, or replays persisted case order.

```sh
sea-forge ledger --root <PATH> verify <LEDGER_ID>
sea-forge ledger --root <PATH> repair <LEDGER_ID>
sea-forge ledger --root <PATH> prove <LEDGER_ID> <ENTRY_ULID>
sea-forge ledger --root <PATH> replay --case <CASE_ID>
```

* `verify` prints `verified`, or prints a verification failure and returns `1`. It is read-only. When the only problem is crash residue (one torn final record after an interrupted append, or an MMR cache behind complete entries) it also prints the diagnosis and the hint to run `ledger repair`.
* `repair` is offline recovery: stop the server first. Under the same exclusive stream lock appends use, it moves the bytes after the last complete record (at most one torn record) to `ledgers/quarantine/`, prints how many bytes and records were quarantined and where, rebuilds the MMR cache from verified entries if it is behind, then verifies and returns non-zero if verification still fails. It never truncates a corrupt record followed by more data, and never rebuilds over a tampered entry: it refuses, writes nothing, and the ledger must be investigated.
* `prove` prints the entry hash, leaf index, proof hashes, and MMR peaks.
* `replay` reads `cases/<case_id>/case-events.jsonl` only; it never re-executes an episode. It prints persisted dispatch/settlement events in file order and rejects missing, duplicate, or non-monotonic ordinals.

### `sea-forge env`

Lists or shows pinned `EnvironmentSpec` documents.

```sh
sea-forge env list [--root <PATH>]
sea-forge env show <REFERENCE> [--root <PATH>]
```

`show` stores the built-in demo environment when the cell is fresh, then prints the requested spec as YAML.

---

## 5. Case and Approval Commands

### `sea-forge case`

```sh
sea-forge case reopen <CASE_ID> [OPTIONS]
sea-forge case add-task <CASE_ID> --item <PATH> [OPTIONS]
sea-forge case manager-iterate <CASE_ID> --endpoint <ENDPOINT> [OPTIONS]
```

| Subcommand | Meaning | Notable flags |
|---|---|---|
| `reopen` | Reopens a completed or terminated case through the authorized mutation path | `--root`, `--policy`, `--actor` |
| `add-task` | Adds one `PlanItem` from a JSON file | `--item` required |
| `manager-iterate` | Runs one auditable Thoth manager-loop step | `--endpoint` required; `--max-iterations` defaults to `8` |

`manager-iterate` never invents an endpoint. The endpoint must be supplied explicitly and must match the endpoint-id grammar. The command returns `0` when the step settles and `5` when it parks awaiting approval.

### `sea-forge task complete`

Completes a parked human task through the same authorized mutation path used by SFWP.

```sh
sea-forge task complete <CASE_ID> <ITEM_ID> [--note <STRING>] [OPTIONS]
```

### `sea-forge approve` and `reject`

Resolve a pending approval request.

```sh
sea-forge approve <CASE_ID> <APPROVAL_ID> [--note <STRING>] [OPTIONS]
sea-forge reject  <CASE_ID> <APPROVAL_ID> [--note <STRING>] [OPTIONS]
```

Both accept `--root`, `--policy`, and `--actor`. Resolution replays the ledger, checks separation of duty and criteria binding, re-evaluates authority, and writes the approvals journal.

---

## 6. Agent and Thoth Commands

### `sea-forge agent`

Governed HTTP/ACP agent connectivity through the running server.

```sh
sea-forge agent list [--root <PATH>]
sea-forge agent probe <ENDPOINT> <PROMPT> [OPTIONS]
sea-forge agent delegate <ENDPOINT> <INSTRUCTION> [OPTIONS]
sea-forge agent cancel <RUN_ID> [OPTIONS]
```

`agent list` asks the server for configured endpoints. `probe` and `delegate` accept `--model`, `--policy`, `--entity`, and `--process`; `delegate` also accepts `--run-id`, `--max-turns` (default `1`), `--token-budget`, and `--agent-output-must-contain`. `cancel` takes the stable run id and the same policy/entity/process flags.

`probe` and `delegate` return `0` when the settlement is accepted and `3` otherwise. They require a live server socket.

### `sea-forge ask`

Asks Thoth a typed question about the self-model.

```sh
sea-forge ask <KIND> <SUBJECT> [OPTIONS]
```

| Flag | Default |
|---|---|
| `--purpose <STRING>` | `planning` |
| `--case <CASE_ID>` | — |
| `--json` | off |
| `--root <PATH>` | `.sea-forge` |
| `--actor <STRING>` | `operator_local` |

Returns `0` for answered or partial, `4` for denied, and `2` for an unknown question kind.

### `sea-forge self-model`

```sh
sea-forge self-model validate [--root <PATH>]
sea-forge self-model rebuild [--probe] [--actor <STRING>] [--root <PATH>]
sea-forge self-model show [--json] [--root <PATH>]
```

`rebuild --probe` currently records the probe intent; probe execution is not wired into this command. The store consumes evidenced probe results rather than running commands itself.

---

## 7. Artifact and Federation Commands

### `sea-forge artifact`

```sh
sea-forge artifact synthesize <INPUT_JSON> [OPTIONS]
sea-forge artifact productize <INPUT_JSON> [OPTIONS]
sea-forge artifact capitalize <INPUT_JSON> [OPTIONS]
sea-forge artifact attest <ARTIFACT_ID> [OPTIONS]
```

The three transition commands read a `TransitionProposal` from `INPUT_JSON`; the command name must match the transition kind in the file. `attest` accepts:

| Flag | Default |
|---|---|
| `--ledger <ID>` | `ifl` |
| `--degraded-mode <MODE>` | `forbidden` |
| `--requester-role <ROLE>` | `operator` |
| `--approver-role <ROLE>` | `R-SO` |
| `--degraded-control <CONTROL>` | repeatable |

All artifact commands accept `--root`, `--policy`, and `--actor`.

### `sea-forge export`, `import`, `adopt`

```sh
sea-forge export --run-ids <ID...> --out <PATH> [OPTIONS]
sea-forge import <BUNDLE> [OPTIONS]
sea-forge adopt <CELL_ID> <REFERENCE> [OPTIONS]
```

* `export` requires one or more `--run-ids`; `--templates` takes zero or more. Export exposure is computed from the exported runs' case labels, so a classified run falls to default deny unless policy permits the labels.
* `import` takes the bundle path positionally.
* `adopt` binds an imported template reference into a cell.

All three accept `--root`, `--policy`, and `--actor`.

---

## 8. Migration

```sh
sea-forge migrate [OPTIONS]
```

| Flag | Default | Meaning |
|---|---|---|
| `--root <PATH>` | `.sea-forge` | Cell root |
| `--policy <PATH>` | — | Policy bundle supplying signing-key configuration |
| `--key-dir <PATH>` | policy key dir, else `.sea-forge-migration-keys` beside the root | Signing-key directory |
| `--key-id <STRING>` | `migration` | Signing-key id when no policy supplies one |

Migration imports a legacy tree into a signed `legacy-import` stream and writes a migration marker. A migrated root reports `legacy_digest_only` assurance.

---

## 9. Hidden Commands

These commands exist for self-checks and tests. They are hidden from `--help` and are not an operator API.

| Command | Purpose |
|---|---|
| `validate <FILE>` | Config-free `.sea` model shape check; prints valid/invalid and returns `0`/`1` |
| `stage-check <FILE> <SHA256>` | Config-free stage acceptance gate; compares SHA-256 and returns `0`/`1` |
| `internal-test-sleep <SECONDS>` | Deterministic test delay |
| `internal-test-swe-seed` | Test settlement-attestation seam, reading a bounded JSON request from stdin |

`stage-check` exists so a spec-pipeline `ExecuteCommand` can re-invoke this exact binary rather than an arbitrary external command. Do not build workflows on any hidden command.

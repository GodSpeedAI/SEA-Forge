# How-To: Write Custom Authority Policies

> **A practical guide for security officers and operators authoring YAML policy bundles that govern writes, commands, approvals, and the sandbox boundary map.**

---

## Goal

Create a `sea-forge-policy.yaml` file that:

* allows selected file writes and command executions;
* blocks generated zones and other protected paths;
* escalates sensitive operations to human approvers;
* documents why jail-class runs start with a denied network posture.

---

## Prerequisites

* A built `sea-forge` binary (`cargo build -p sea-forge-cli`).
* A cell root directory, such as `/tmp/custom-cell`.
* A text editor for YAML.

---

## 1. Minimal Working Bundle

A policy bundle is versioned YAML with an `identity` block, optional policy surfaces, and a list of typed rules. Rules are evaluated in order and the first matching rule wins. If no rule matches, the default disposition is `deny`.

```yaml
version: "0.1"
identity:
  source: custom-cell
  allow_unresolved: false

policy_surfaces:
  file:
    mode: deny-by-default
    deny_write:
      - "**/src/gen/**"
      - "**/.git/**"
      - "**/.env*"
      - "**/*secret*"

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
```

This bundle allows `operator` to write the demo model inside the run workspace, and to run the trusted `sea-forge` validator inside the `jail` sandbox class. All other operations fall through to default deny.

---

## 2. Rule Anatomy

Each rule under `rules:` is a typed match, not a shell pattern.

| Field | Required | Meaning |
|---|---|---|
| `name` | Yes | Stable rule identifier used in `authority.json`. |
| `verdict` | Yes | `allow`, `deny`, or `escalate`. |
| `actor_role` | Yes | Role that may match the rule, for example `operator`. |
| `operation_kind` | Yes | Typed action family, for example `write_file`, `execute_command`, `agent_task`, or `mcp_tool_call`. |
| `path_prefix` | For writes | Segment-aware path prefix. `docs` matches `docs` and `docs/x`; it does not match `docs-private`. |
| `argv0` | For commands | Exact executable file name compared with the basename of `argv[0]`. |
| `sandbox_class` | Optional | Isolation class: `local`, `jail`, `isolated`, `microvm`, or `openshell`. |
| `requires_approval` | Optional | When `true`, an otherwise `allow` rule becomes an escalation. |
| `boundary_constraints` | Optional | Map of dimension to allowed string values. |
| `compensating_controls` | Optional | Human-readable controls required for a `degraded` disposition. |

A `local` allow rule for `execute_command` must pin `argv0` to the trusted `sea-forge` executable. A `jail` or stronger class may run other declared commands.

---

## 3. Boundaries and Surfaces

### Boundary constraints

Boundary constraints are a map from a dimension to a list of allowed string values:

```yaml
boundary_constraints:
  workspace:
    - "/tmp/custom-cell/runs/<run_id>/workspace"
```

Common accepted dimensions include `workspace`, `artifacts_root`, `timeout_secs`, `env_keys`, `sandbox_class`, `mcp_server`, `mcp_tool`, `db_resource`, `db_scope`, and `permitted_exposure`.

The runtime has a reserved `network` dimension for TCP port grants, but the current `0.1` policy validator rejects it. Do not add `network` under `boundary_constraints` yet; jail-class runs start with `NetworkPosture::Denied`.

### File surface

`policy_surfaces.file.deny_write` is a hard deny list. It runs before rule matching, so an explicit allow rule cannot override it. The default list protects generated zones, Git metadata, `.env*`, and secret-bearing files.

### External API surface

```yaml
policy_surfaces:
  external_api:
    mode: deny-by-default
    allow_hosts: ["api.internal"]
```

An allowed host still escalates because the surface is unsupported for direct execution; an unknown host is denied.

### Escalation

Two equivalent forms force human review:

```yaml
- name: require-approval
  verdict: allow
  actor_role: operator
  operation_kind: execute_command
  argv0: deploy
  sandbox_class: jail
  requires_approval: true
```

or:

```yaml
- name: direct-escalation
  verdict: escalate
  actor_role: operator
  operation_kind: execute_command
```

Both produce `Verdict::Escalate` and append an `ApprovalRequest` to `approvals.jsonl`.

---

## 4. Apply the Policy

### At the CLI

```sh
./target/debug/sea-forge run \
  --intent "generate a .sea model" \
  --policy /tmp/custom-cell/sea-forge-policy.yaml \
  --root /tmp/custom-cell
```

### On the server

Place the bundle at `<cell-root>/sea-forge-policy.yaml`. The server loads it on startup and mirrors it into `<cell-root>/authority/active-policy.json`.

---

## 5. Validate a Decision

### Allowed run

```sh
./target/debug/sea-forge run \
  --intent "generate a .sea model" \
  --policy my-policy.yaml \
  --root /tmp/custom-cell
```

Expected output:

```text
run_id=run_20260902T190000Z_abcdef
authority=auth_01:allow
authority=auth_02:allow
execution=completed
settlement=accepted
run_dir=/tmp/custom-cell/runs/run_20260902T190000Z_abcdef
```

### Escalated run

Submit an intent whose first matching rule is `verdict: escalate`:

```sh
./target/debug/sea-forge run \
  --intent "generate a .sea model" \
  --policy escalation-policy.yaml \
  --root /tmp/custom-cell
```

Expected output ends with:

```text
settlement=escalated
```

The run directory still contains `authority.json`, `trace.jsonl`, and `settlement.json`; the workload did not run.

---

## 6. Troubleshooting

* **Run exits `3`.** The operation was denied or rejected by settlement. Inspect `<run_dir>/authority.json` for `matched_rule`, `reason_codes`, and `policy_refs`, then inspect `<run_dir>/settlement.json` for the exact basis tokens.
* **A rule does not match a relative path.** `path_prefix` is segment-aware, not a glob. Use the normalized spelling of the first path segment.
* **A command rule does not match.** `argv0` compares the file name component of `argv[0]`, not a shell pattern.
* **A generated-zone write is denied despite an allow rule.** `policy_surfaces.file.deny_write` runs before rule matching and cannot be overridden.
* **No rule matches.** The default disposition is `deny`; the run settles as `Rejected` with `authority_deny`.

---

## 7. Source Trail

* [`crates/sea-forge-authority/src/lib.rs`](../../crates/sea-forge-authority/src/lib.rs) — `AuthorityPolicyBundle`, `PolicyRule`, rule matching, and grant minting.
* [`crates/sea-forge-sandbox/src/lib.rs`](../../crates/sea-forge-sandbox/src/lib.rs) — `NetworkPosture` and the fail-closed network posture.
* [`docs/subsystems/authority-fabric.md`](../subsystems/authority-fabric.md) — Authority model, first-match-wins, and candidate verdicts.
* [`docs/explanation/landlock-jail-security-model.md`](../explanation/landlock-jail-security-model.md) — Why jail-class runs start with a denied network posture.

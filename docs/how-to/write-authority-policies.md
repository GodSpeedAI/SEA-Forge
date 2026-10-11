# How-To: Write Custom Authority Policies

> **A practical guide for security officers and operators authoring YAML policy bundles to govern workloads, file access, and network ports.**

---

## Goal

Create and enforce a custom `sea-forge-policy.yaml` file that allows specified commands, restricts file writes to designated patterns, escalates high-risk actions to human approvers, and blocks unauthorized network access.

---

## Prerequisites

* `sea-forge` CLI binary built and accessible.
* Basic familiarity with YAML syntax and glob patterns.

---

## 1. The Structure of a Policy Bundle

An authority policy bundle contains policy metadata, rule definitions, and ledger integrity requirements:

```yaml
version: "0.1"
policy_bundle_id: "sec-eng-standard-v1"
summary: "Standard development and build policy"

integrity_ledger:
  required_for_side_effects: false
  signing_key_id: "dev-key"

rules:
  # 1. High-risk actions escalate to human review
  - rule_id: "rule-escalate-rm"
    description: "Escalate destructive commands to human approver"
    action_pattern: "execute_command:rm*"
    disposition: "escalate"
    require_approval: true

  # 2. Allow standard compiler and test execution
  - rule_id: "rule-allow-cargo-test"
    description: "Allow cargo build and test commands"
    action_pattern: "execute_command:cargo*"
    disposition: "allow"
    boundaries:
      network:
        ports: [] # Denied network

  # 3. Allow writing source and test files
  - rule_id: "rule-allow-source-writes"
    description: "Allow writing Rust source files"
    action_pattern: "write_file:src/**/*.rs"
    disposition: "allow"

  # 4. Forbid writing to generated zones
  - rule_id: "rule-deny-generated-zone"
    description: "Forbid manual writes to generated zones"
    action_pattern: "write_file:src/gen/**"
    disposition: "deny"

default_disposition: "deny"
```

---

## 2. Rule Configuration Options

### Action Patterns
Action patterns match the normalized `AuthorityAction` kind and path:
* `write_file:<glob>`: Matches target file write paths (e.g. `write_file:**/*.sea`).
* `execute_command:<binary>*`: Matches the primary executable binary name in `argv[0]`.
* `agent_task:<endpoint>`: Matches autonomous agent delegation requests.

### Dispositions
* `allow`: Grants an `ActionGrant` enabling execution.
* `deny`: Refuses execution immediately.
* `escalate`: Halts execution and generates a pending `ApprovalRequest` in `approvals.jsonl`.

### Boundaries
* `network.ports`: Array of allowed TCP destination ports (e.g. `[443, 8080]`). An empty array (`[]`) enforces `NetworkPosture::Denied`.

---

## 3. Applying Your Policy

### At the CLI
Pass `--policy` to your run command:
```sh
./target/debug/sea-forge run \
  --intent "generate demo model" \
  --policy my-custom-policy.yaml \
  --root /tmp/custom-cell
```

### On the Server
Place your policy file inside the cell root:
```sh
cp my-custom-policy.yaml /tmp/custom-cell/sea-forge-policy.yaml
```
The server loads this policy on startup and mirrors it into `.sea-forge/authority/active-policy.json`.

---

## 4. Validating Policy Decisions

Test that your policy behaves as expected using targeted runs:

### Verify an Allowed Action
```sh
./target/debug/sea-forge run --intent "generate demo model" --policy my-custom-policy.yaml
```
Output should settle as `Accepted`.

### Verify an Escalation
Submit an intent that triggers an escalation rule:
```sh
./target/debug/sea-forge run --intent "cleanup files" --policy my-custom-policy.yaml
```
Output:
```text
Run halted: authority escalated.
Approval apr_0001 pending in .sea-forge/approvals.jsonl.
```

---

## 5. Common Failure Symptoms & Troubleshooting

* **Symptom:** Command exits with code 1 and error `"no matching rule"`.
  * **Cause:** Default deny is active, and no allow rule matched your command pattern.
  * **Fix:** Inspect `.sea-forge/runs/<id>/authority.json` to view the exact `action_request` string that failed matching, and adjust your rule pattern.
* **Symptom:** `write_file` rule does not match relative paths.
  * **Cause:** Glob pattern is missing recursive wildcards (`**`).
  * **Fix:** Use `write_file:src/**/*.rs` instead of `write_file:src/*.rs`.

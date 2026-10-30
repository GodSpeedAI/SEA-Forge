# Repository Scripts (scripts)

## Overview
Operational, verification, and evidence scripts that glue the SEA Forge
kernel to its environment: agent-context and preregistration gates, E2E
convergence checks, cell lifecycle (seed/backup/restore/reset), doctor
bootstrap evidence, workbench E2E runners, and the security and OpenShell
script subtrees. These are shell and Python entry points, not compiled code;
each documents its own usage and exit contract in its header.

## Interface
- Cell lifecycle:
  - `seed-cell.sh [cell-root]` — seeds a demonstration cell (default
    `$HOME/.sea-forge-demo`) with records produced by really running the real
    kernel; nothing hand-written ([seed-cell.sh:header]).
  - `reset-cell.sh [cell-root] [--yes]` — deletes only a cell this project
    seeded (guards on the `.seeded-demo-cell` stamp and resolved-path
    comparisons) ([reset-cell.sh:header and guards]).
  - `casework-cell-backup.sh <cell-root> <archive.tar.gz>` — consistent
    offline backup: refuses to run while the kernel holds
    `<cell>/.server.lock`, takes the same flock for the copy, writes
    `<archive>`, `<archive>.sha256`, and `<archive>.manifest`; the archive
    is created 0600 and contains signing/continuation keys
    ([casework-cell-backup.sh:header]).
  - `casework-cell-restore.sh <archive.tar.gz> <new-cell-root>` — restore
    that never deletes anything; verifies archive digest and per-file
    manifest before extraction and against the extracted tree; target must
    not exist or must be empty ([casework-cell-restore.sh:header]).
  - `seed_approvals.py <cell-root> <socket-path> <argv0-tool-path>` — seeds
    real escalations and a resolved approval over the cell's Unix socket,
    acting as the gateway principal with `on_behalf_of`
    ([seed_approvals.py:module docstring]).
- Agent-context and convergence gates:
  - `check-agent-context.sh` — the `just status-check` implementation;
    validates `AGENTS.md`, `.agents/AGENTS.md`,
    `.agents/CURRENT_STATUS.yaml`, and `.agents/CURRENT_STATUS.md`
    ([check-agent-context.sh:checks]).
  - `check-e2e-preregistration.sh` — verifies the frozen E2E preregistration
    matches the hash bound in `.agents/plans/e2e-plan.yml`; never updates
    the stored hash ([check-e2e-preregistration.sh:header]).
  - `e2e-prereg-ids.sh` — prints the frozen requirement IDs (edges E*,
    invariants I1–I15, envelope rules ENV-I1–I8) in canonical order
    ([e2e-prereg-ids.sh:header]).
  - `e2e-delta-check.sh` — mechanically verifies preregistration integrity,
    one verdict per frozen requirement with the frozen vocabulary, evidence
    paths resolving under `$REPOS_ROOT`, and byte-identical reproducibility
    of the committed Delta-0 report ([e2e-delta-check.sh:header]).
  - `e2e-delta-report.sh` — deterministically regenerates the Delta-0
    summary from `.agents/status/e2e-current-status.yml`
    ([e2e-delta-report.sh:header]).
  - `e2e-world-loop.sh` — end-to-end world loop across the GodSpeed stack
    (GodSpeed-Agent, Context Kernel, SWE_SEED, SEA-Forge, RealityTrace) in
    one pinned semantic world, including refusal of foreign-world evidence;
    requires `domainforge`, `uv`, `cargo` ([e2e-world-loop.sh:header]).
  - `check-workbench-completion-eval-inputs.sh` — validates the workbench
    completion-evaluation inputs report and exclusions fixture shape
    ([check-workbench-completion-eval-inputs.sh]).
- Bootstrap evidence:
  - `doctor.sh` — Shell-SPEC §7.1 DoctorResult emitter; writes JSONL to
    `target/bootstrap-evidence/doctor.jsonl` and stdout; any `fail` row
    exits nonzero, `skipped` never counts as passed ([doctor.sh:header]).
- Workbench E2E runners:
  - `workbench-e2e-agent-browser.sh` — renderer-only, non-mocked evidence
    via agent-browser; explicitly not real-stack coverage
    ([workbench-e2e-agent-browser.sh:header]).
  - `workbench-e2e-case-authoring.sh` — mocked-IPC case-authoring proof
    journeys (scenarios 6 and 7) driven by agent-browser with a SFWP mock
    shim; not real-stack coverage ([workbench-e2e-case-authoring.sh:header]).
- Script subtrees: [./openshell/README.md] - The only sanctioned launcher of
  a real OpenShell gateway for the live case-runner proof.
  [./security/README.md] - Kernel-security probes, evidence retention,
  round-4 socket gates, and secret-scanning canaries.
- Other (no contract written): `tests/` (self-test harness for
  `check-agent-context.sh`), `windows/` (`identity-e2e.ps1`, the Windows
  control-plane identity E2E run on a real Windows runner).

## Invariants
- Backup/restore lock discipline: the kernel's exclusive
  `<cell>/.server.lock` serializes backups and restores against a running
  server ([casework-cell-backup.sh:header],
  [casework-cell-restore.sh:header]).
- Restore never deletes: it refuses a non-empty target and removes only what
  the failed run itself extracted ([casework-cell-restore.sh:header]).
- Frozen preregistration: `check-e2e-preregistration.sh` and
  `e2e-delta-check.sh` only verify; they never update the stored hash, the
  status file, or the committed report ([check-e2e-preregistration.sh:header],
  [e2e-delta-check.sh:header]).
- Demo data honesty: `seed-cell.sh` and `seed_approvals.py` write only
  records produced by really running the real kernel — hand-written demo
  records would be fabricated evidence ([seed-cell.sh:header],
  [seed_approvals.py:module docstring]).
- Deletion safety: `reset-cell.sh` refuses anything not provably a cell this
  project seeded ([reset-cell.sh:header]).
- Evidence honesty in E2E runners: agent-browser runners label their
  coverage truthfully (renderer-only or mocked-IPC, never real-stack)
  ([workbench-e2e-agent-browser.sh:header],
  [workbench-e2e-case-authoring.sh:header]).

## Side Effects
- Starts and stops real `sea-forge-server` processes and Unix sockets
  ([seed-cell.sh], [reset-cell.sh], [e2e-world-loop.sh]).
- Deletes directory trees, but only guarded demo cells
  ([reset-cell.sh]).
- Writes evidence and reports into `.agents/` and `target/`
  ([doctor.sh:doctor.jsonl], [e2e-delta-check.sh],
  [check-workbench-completion-eval-inputs.sh], [./security/README.md]).
- Creates and removes temp directories, background processes, and browser
  sessions via traps ([workbench-e2e-agent-browser.sh:cleanup],
  [e2e-world-loop.sh], [./openshell/README.md]).
- Backups contain signing/continuation keys and approvals and are written
  0600 for off-host encrypted storage by the operator
  ([casework-cell-backup.sh:header]); no script embeds or prints secret
  values.

## Verification
- Run: `bash -n` on the Bash scripts and `sh -n` on the POSIX `sh` scripts in
  this directory and its subtrees (no manifest or `just` recipe compiles
  these scripts; the substantive gates are the scripts themselves, e.g.
  `just status-check` for `check-agent-context.sh`).

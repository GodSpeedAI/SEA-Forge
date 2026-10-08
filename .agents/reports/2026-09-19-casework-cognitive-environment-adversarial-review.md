# Casework cognitive environment: adversarial document review

Date: 2026-09-19
Depth: deep
Scope: the requested casework spec, implementation plan, current SEA Forge
substrate, and the sibling Gauntlet boundary. This is a document review;
product behavior was not implemented or exercised.

Operational follow-up (2026-09-19): plan v0.2.2 requires isolated SEA Forge
and Gauntlet worktrees for concurrent work. Each worktree uses its own
canonical status files, preserving `just context-check` without a sidecar
exception. A resource preflight may defer Rust baselines while Go/React
foundation work proceeds. Deferred gates remain pending, and Rust-gated
tasks, parity, and removal cannot settle until the actual gates pass.

## Verdict

The original documents were not executable as a settlement contract. The
specification failed YAML parsing near its opening metadata, and the plan
referenced a nonexistent spec path, status path, brief, Go/UI gates, Go
module, and adapters. The revised documents now parse, bind by SHA-256, map
86 requirements to 15 tasks, and make removal of both unwanted UIs a
mandatory post-parity settlement task.

## Findings corrected

1. **Unparseable normative source (blocking).** Initial
   `yaml.safe_load` failed at line 11 of the original 2,090-line spec;
   flattened indentation, Markdown fences, and list markers prevented a
   structured agent from reading it. The revised spec is valid YAML and
   retains the text of the original 84 requirement records. It adds two
   explicit migration requirements.
2. **Phantom substrate and false path binding (blocking).** The initial
   plan named `.agents/specs/godspeed-casework-cognitive-environment.spec.yaml`
   and `.agents/CURRENT_STATUS.yml`, neither of which exists. The source
   checkout has no Go module, Go source, GitHub casework adapter, CopilotKit
   adapter, or donor checkout. The revised spec names the current SFWP
   server, Workbench migration source, and sibling Gauntlet executor,
   with exact proposed new app locations. T00 must inventory actual SFWP
   capabilities and block on missing ones rather than inventing them.
3. **Gates that could appear bound without running (blocking).** The old
   `sha256sum` gate only printed a digest and several gates were unresolved
   sentinels. The new validator compares the exact spec bytes, checks YAML
   duplicate keys, 86/86 bidirectional mapping, the DAG, gate activation,
   high-risk confirmation, and both removal requirements. Existing Rust
   and Gauntlet gates are named; four future recipes have explicit
   activation tasks and cannot count as passed before they exist.
4. **Premature artifact settlement (high).** T08 previously claimed
   authorized backend persistence before the Go/SEA Forge action path in
   T11 existed. T08 now proves rendering, traceability, ephemerality, and
   content safety. T11 settles artifact persistence and provenance under
   the real authority path.
5. **Unnecessary dependency assumptions (high).** The old plan required
   donor extraction, React Three Fiber, and CopilotKit despite absent
   checkouts/adapters and optional spec language. Donors are now use-or-omit
   decisions; graphics and agent frameworks are adapters chosen against
   existing substrate, with prior dependency review.
6. **Missing actual replacement/removal outcome (blocking).** The operator
   requires the SEA Forge Tauri/Rust GUI and Gauntlet TUI to be ripped out.
   REQ-MIG-001/002 and T14 now require parity, source/build removal, a
   post-removal real integration gate, and independent confirmation.
   The SEA Forge Rust authority/server and Gauntlet execution engine remain.
7. **Hidden generated-contract coupling (high).** The current Workbench
   contract generator and Rust
   `generated_schemas_are_committed_and_current` assertion are coupled.
   T14 must migrate their output/check to the replacement client location
   before deleting Workbench-only files; neither gate may be deleted to
   obtain a pass. ADR-006 records this architecture change.

## Verification performed

- `python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py`:
  PASS (spec hash, 86/86 requirements, 15-task DAG, gate activation,
  high-risk groups, and mandatory UI removal).
- `just context-check`: PASS.
- Python AST parse of the validator: PASS.
- `graft build`: exit 0 after the validator change.
- Whole-worktree `git diff --check`: exit 2 on pre-existing modified
  Workbench CRLF/trailing-whitespace lines outside this review. The new
  spec, plan, validator, ADR, and report have no trailing whitespace.

## Remaining execution conditions

No Go or replacement React application exists yet; no legacy UI was removed.
The Go/UI/integrated/removal recipes are planned tasks, not executed proof.
T00 must inspect SFWP methods, historical-state support, identity flow,
Gauntlet TUI workflows, and the real baseline of required Rust/Gauntlet
gates. A red existing gate blocks T00 until repaired without weakening it.
T14 also needs writable access to the sibling Gauntlet repository. These
are explicit plan prerequisites, not claims of current conformance.

# .agents/ — Agent Workbench Guide

Durable operational memory and working context for coding agents in SEA Forge. Task state, handoff ledgers, specifications, and evidence live here. Follow root `AGENTS.md` for repository-wide invariants and verification gates.

## 1. Directory Roles & Authority

* **`specs/` (Normative Source of Truth)**: Authoritative specifications (`Shell-SPEC.md`, `spec-minimum.md`, `spec-full.md`, subsystem specs). Code and tests must conform to these. When changing a public contract, persisted schema, architecture boundary, or proof level, update the spec and ADR in the same change.
* **`plans/` (Execution Scratch)**: Tactical plans and task decompositions. Plans are non-normative execution aids; do not treat plans as specifications.
* **`evidence/` (Immutable Settlement Records)**: Command outputs, test logs, benchmark records, and proof artifacts. Historical evidence is immutable; never rewrite, delete, or fabricate run evidence.
* **`reports/`**: Read-only audits, investigations, and analysis summaries.
* **`skills/`**: Procedural workflows and harness cheatsheets.

## 2. Handoff Contract (`just status` / `just status-check`)

The acting agent updates this workbench when repository reality changes. `CURRENT_STATUS.yaml` is the agent-facing, append-only status history. Read only its last line (`just status` or `tail -n 1 .agents/CURRENT_STATUS.yaml`) for the latest complete snapshot; read earlier records only when history matters. Each update appends exactly two lines: `---` and one JSON-compatible YAML object with a higher `revision`, `recorded_at`, `stage`, `summary`, `verified`, `limits`, `next`, `evidence`, `spec`, and `ledger`. Carry forward facts that remain true; an entry is a complete snapshot, not a delta. Never edit previous entries.

`CURRENT_STATUS.md` is the human-facing view of only the latest snapshot. Update it in place in the same change, with matching status revision and summary; do not append history there.

Handoff validity and coupling are verified by `just status-check` (`scripts/check-agent-context.sh`).

## 3. Workbench Memory Ledgers

* **`OBSERVED_DEBT.md`**: Concrete out-of-scope debt, gaps, risks, or defects noticed while working. Each entry requires evidence, concrete impact, and suggested next move. Update existing entries rather than duplicating.
* **`LESSONS.md`**: Verified, reusable project lessons specific to SEA Forge. Record only lessons backed by executable evidence, commands, or tests.
* **`OPEN_QUESTIONS.md`**: Unresolved choices that genuinely require human judgment and cannot be settled from repository evidence. Always include a recommendation, options, and impact.

## 4. Invariants

* **One home per fact**: Reference canonical sources; do not copy specifications into scratch notes or duplicate ledgers.
* **Never fabricate evidence**: Only record commands that were actually executed and results that were observed.
* **No conversational transcripts**: Keep memory files concise, structured, and free of chat transcripts or machine-specific absolute paths.






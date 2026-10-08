# .agents/ — Agent Workbench Guide

Durable operational memory and working context for coding agents in SEA Forge. Task state, handoff ledgers, specifications, and evidence live here. Follow root `AGENTS.md` for repository-wide invariants and verification gates.

## 1. Directory Roles & Authority

* **`specs/` (Normative Source of Truth)**: Authoritative specifications (`Shell-SPEC.md`, `spec-minimum.md`, `spec-full.md`, subsystem specs). Code and tests must conform to these. When changing a public contract, persisted schema, architecture boundary, or proof level, update the spec and ADR in the same change.
* **`plans/` (Execution Scratch)**: Tactical plans and task decompositions. Plans are non-normative execution aids; do not treat plans as specifications.
* **`evidence/` (Immutable Settlement Records)**: Command outputs, test logs, benchmark records, and proof artifacts. Historical evidence is immutable; never rewrite, delete, or fabricate run evidence.
* **`reports/`**: Read-only audits, investigations, and analysis summaries.
* **`skills/`**: Procedural workflows and harness cheatsheets.

## 2. Handoff Contract (`just context-check`)

Every agent session ending with project changes must update the handoff state before completing. Verified by `just context-check` (`scripts/check-agent-context.sh`):

* **`CURRENT_STATUS.md`** is the primary handoff file. It MUST contain:
  1. An `Updated: YYYY-MM-DD` line near the top.
  2. The 8 required section headers:
     - `## Objective`
     - `## Worktree State`
     - `## Changed Files`
     - `## Completed`
     - `## Verification`
     - `## Remaining`
     - `## Blockers`
     - `## Decisions`
* **`current_status.yml`** is the structured machine-readable companion for cold agent resumption: objective, worktree state, changed files, completed work, verification, remaining steps, blockers, and decisions.

## 3. Workbench Memory Ledgers

* **`OBSERVED_DEBT.md`**: Concrete out-of-scope debt, gaps, risks, or defects noticed while working. Each entry requires evidence, concrete impact, and suggested next move. Update existing entries rather than duplicating.
* **`LESSONS.md`**: Verified, reusable project lessons specific to SEA Forge. Record only lessons backed by executable evidence, commands, or tests.
* **`OPEN_QUESTIONS.md`**: Unresolved choices that genuinely require human judgment and cannot be settled from repository evidence. Always include a recommendation, options, and impact.

## 4. Invariants

* **One home per fact**: Reference canonical sources; do not copy specifications into scratch notes or duplicate ledgers.
* **Never fabricate evidence**: Only record commands that were actually executed and results that were observed.
* **No conversational transcripts**: Keep memory files concise, structured, and free of chat transcripts or machine-specific absolute paths.

**Any debt you encounter while woriking that is out of scope or doesn't block your work must be recorded in .agents/DEBT.md**

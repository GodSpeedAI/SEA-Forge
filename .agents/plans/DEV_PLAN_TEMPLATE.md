<!-- ============================================================= -->
<!-- PLAN TEMPLATE — fill every {{VARIABLE}} and [PLACEHOLDER].     -->
<!-- Conventions:                                                  -->
<!--   {{VAR}}          = a value reused across the doc (define once in the table below) -->
<!--   [PLACEHOLDER: x] = fill-in specific to this plan; delete the brackets when done   -->
<!--   <!-- … -->       = authoring guidance; DELETE these comments in the final plan     -->
<!--   ⟨optional⟩       = include only if relevant; otherwise remove the whole line/block -->
<!-- Golden rule: observable gates derive progress. Checkboxes, prose, file presence, and  -->
<!-- prior claims never establish completion. Every task has a copy-pasteable gate and a   -->
<!-- teeth check; the first failing gate selects the next work.                             -->
<!-- ============================================================= -->

# Implementation Plan — {{PLAN_TITLE}}

**Created:** {{DATE}}
**Source of truth:** {{SOURCE_DOC_PATH}}   <!-- the audit/spec/review this plan executes; quote it, don't paraphrase from memory -->
**Originating context:** [PLACEHOLDER: 1–2 lines — what produced this plan (a review? an audit? a bug?) and on what branch/state]
**Status of the work today:** [PLACEHOLDER: what already exists vs. what this plan closes — be honest about partials]

---

## 0. How to use this plan (agent operating instructions)

<!-- This section is what lets a COLD agent execute without re-deriving context. Keep it. -->

- Execute tasks in the order given. Dependencies: [PLACEHOLDER: e.g. "Task 1→2 dependent (2 reuses X); Tasks 3,4 independent"]. Select the first dependency-ready task whose gate does not currently pass.
- **Every task ends with a verification gate.** Task state is derived by running that gate, never by editing a checkbox or trusting a prior report.
- **{{CORE_PRINCIPLE}}** <!-- the single rule that keeps the work coherent. e.g. "One engine, one producer — never re-implement output in a wrapper/test; extract a shared function." Make it specific to THIS codebase. -->
- Match surrounding code style; locate the relevant idioms here: [PLACEHOLDER: dirs/files that show the pattern to mirror].
- [PLACEHOLDER: any source-of-truth-is-X rule, e.g. "the corpus/fixtures are the spec — change expected files deliberately, in the same commit as the code that justifies them."]

### Completion state model

<!-- Keep these states. They prevent a long-running or Ralph-loop agent from turning a stale plan marker into truth. -->

- `UNVERIFIED`: no current gate result exists for the present tree. This is the default, even when implementation files exist.
- `FAILING`: the gate ran and returned nonzero or its required observable/teeth evidence is absent. This task is eligible work when its dependencies pass.
- `PASSING`: the full gate exits 0 on the current tree and the named teeth check is known to fail under the prohibited mutation. Record the command, revision/tree state, and result in the handoff.
- `BLOCKED`: the gate cannot run because of a specific external input, authority, or environment constraint. Record the exact blocker and the command that will resume verification. Difficulty or incomplete code is `FAILING`, not `BLOCKED`.

Recompute state after relevant upstream changes. A previously passing task returns to `UNVERIFIED` when its evidence is stale. Decorative task lists may summarize these derived states, but they never control them.

### Dependency graph

<!-- Include an ASCII task dependency graph showing task ordering, prerequisites, and parallel/dependent tracks. Example structure below: -->

```text
Task 0 baseline + approvals
  ├─ Task 1 cell import path safety
  ├─ Task 2 canonical transcript/redaction primitive
  ├─ Task 3 jail network isolation
  ├─ Task 4 minimum command/lifecycle compatibility
  ├─ Task 5 DomainForge source-set boundary
  ├─ Task 6 memory authority scope
  ├─ Task 7 SQLite FTS + stale detection
  │    └─ Task 8 recall compatibility + governed evidence (also Task 6)
  └─ Task 9 spec-stage prerequisites
       └─ Task 10A ordinary governed M5 case path (also Tasks 5 and 8)
            └─ Task 10B M5 CLI/project/projection integration
                 └─ Task 11 ledgered self-model + real realization inputs
                 └─ Task 12 ODI provenance production wiring
                      └─ Task 13 Thoth query/disclosure semantics
                           └─ Task 13B Thoth authorship SoD
                                └─ Task 14A real Thoth joins + mediated service
                                     └─ Task 14B CLI/server ask adapters
                                ├─ Task 15 delegation settlement/schema fidelity
Task 2 ─────────────────────────└─ Task 16 transcript retention/storage
Tasks 14B-16 ───────────────────── Task 17 endpoint/topology/manager grants
Tasks 11,15 ────────────────────── Task 18 SWE_SEED reconciliation
Tasks 1-18 (including lettered tasks) ─ Task 19 final conformance and handoff
```

### Global verification gates (must stay green after EVERY task)
<!-- List the cheapest commands that prove the whole system still works. These get re-run constantly. -->
```bash
{{GATE_BUILD_OR_UNIT}}        # [PLACEHOLDER: e.g. cargo test --features cli --workspace]
{{GATE_LANG_A}}               # [PLACEHOLDER: e.g. just ci-test-python]
{{GATE_LANG_B}}               # [PLACEHOLDER: e.g. just ci-test-ts]
{{GATE_LINT}}                 # [PLACEHOLDER: the exact CI lint/format gate]
```

⟨Rebuild steps if you touched {{ARTIFACT}} (bindings/generated code) before running the gates:⟩

```bash
{{REBUILD_CMD_1}}             # [PLACEHOLDER]
{{REBUILD_CMD_2}}             # [PLACEHOLDER]
```

---

## Key facts already discovered (do not re-derive)
<!-- The highest-leverage section. Every file:line you had to hunt for goes here so the next agent doesn't. -->
<!-- Prefer file:line over prose. If a fact isn't pinned to a location, it's probably not actionable. -->

| Thing | Location |
|---|---|
| [PLACEHOLDER: the producer/owner of the behavior in scope] | `{{PATH}}:{{LINE}}` |
| [PLACEHOLDER: the test/oracle that pins current behavior] | `{{PATH}}` |
| [PLACEHOLDER: a non-obvious invariant — e.g. "IDs are content-derived EXCEPT X which is random and must be normalized"] | `{{PATH}}` |
| [PLACEHOLDER: relevant API signatures the tasks will call] | `{{PATH}}` |
| [PLACEHOLDER: where the pre-existing/unrelated issue actually lives] | `{{PATH}}` |

---

<!-- ============================================================= -->
<!-- TASK BLOCK — DUPLICATE THIS WHOLE BLOCK ONCE PER TASK.        -->
<!-- Order tasks by dependency, then by value. Keep each one       -->
<!-- independently verifiable: an agent should be able to stop      -->
<!-- after any task with the tree in a green, coherent state.       -->
<!-- ============================================================= -->

## Task {{N}} — {{TASK_TITLE}}  ⟨(phase/area · priority)⟩

**Goal:** [PLACEHOLDER: one sentence — the observable outcome, not the activity. "X is true and a test proves it."]

**Why this shape:** [PLACEHOLDER: the design constraint that dictates the approach, so the agent doesn't pick a plausible-but-wrong alternative. Omit only if truly self-evident.]

### Steps

1. [PLACEHOLDER: concrete step] — file: `{{PATH}}`. <!-- name the exact file; suggest a signature/name if adding code -->
2. [PLACEHOLDER: concrete step] — file: `{{PATH}}`.
3. [PLACEHOLDER: keep steps small enough that each maps to an edit; reference the "discovered facts" table instead of re-explaining].
4. ⟨Docs/coherence step: update {{DOC}} — but only flip a status claim AFTER its test passes (no claim ahead of proof).⟩

### Gate

```bash
[PLACEHOLDER: exact command(s) that prove THIS task — narrowest scope that's still meaningful]
```

**Done when:** [PLACEHOLDER: observable pass condition + a negative check, e.g. "deliberately breaking X makes the gate fail" — proves the test has teeth].

**Redesign trigger:** [PLACEHOLDER: the discovery that invalidates this approach and what to do instead. Write "none plausible" if genuinely none.]

<!-- END TASK BLOCK — duplicate above as needed -->

---

## Final acceptance gates (whole plan)
<!-- This is a gate set, not a checklist. Progress is the live result of these commands and observations. -->

- **Task 1 gate:** [PLACEHOLDER: command + required observable/teeth result].
- **Task 2 gate:** [PLACEHOLDER: command + required observable/teeth result].
- **Task N gate:** [PLACEHOLDER: command + required observable/teeth result].
- **Cross-cutting gate:** `{{GATE_LINT}}` exits 0.
- **Truthfulness gate:** [PLACEHOLDER: docs/{{DOC}} reflects passing evidence; no status is ahead of a gate].
- **Terminal gate:** all global gates exit 0 on the same tree and `{{ARTIFACT}}` is reproducibly rebuilt.

The plan is complete only when every gate above is `PASSING` on the same relevant tree. A summary marker cannot override a failing, blocked, missing, or stale result.

## Guardrails (do not violate)
<!-- Pulled from the source-of-truth doc. These stop an eager agent from scope-creeping the system wider. -->
- [PLACEHOLDER: e.g. "Do NOT add new {{SURFACE}} to accomplish a task — every item here proves existing behavior, not extends it."]
- [PLACEHOLDER: project-specific invariant that must survive]
- [PLACEHOLDER: commit hygiene — e.g. "keep the unrelated cleanup (Task N) in its own commit titled …"]

<!-- ============================================================= -->
<!-- AUTHORING CHECKLIST (delete before saving the real plan):     -->
<!--  - Every task has a Gate that exits 0 and a teeth-check.        -->
<!--  - Gate results, not task markers, derive progress.             -->
<!--  - Every file reference is file:line, not vibes.                -->
<!--  - Dependencies between tasks stated; independent ones flagged. -->
<!--  - Dependency graph updated to reflect task ordering & prerequisites. -->
<!--  - CORE_PRINCIPLE + Guardrails come from the source doc, quoted. -->
<!--  - Pre-existing/unrelated work is isolated, not smuggled in.     -->
<!--  - A cold agent could start at Task 1 with zero prior context.    -->
<!-- ============================================================= -->

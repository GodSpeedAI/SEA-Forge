# Lessons

Durable, verified, project-specific lessons that improve future agent work or
prevent easy or repeated mistakes belong here. Keep entries concise, dated, and
linked to evidence. Do not use this file as a task log or scratchpad.

## 2026-07-10: Keep executable operations narrower than authority actions

The minimum conformance matrix requires authority to classify reserved and
malformed surfaces that the planner must never execute. A closed `Operation`
cannot serve both roles. `AuthorityAction` is therefore the canonical broad gate
input, while `Operation` remains the two-variant executable planner contract.

## 2026-07-11: Keep settled attempts separate from proven capability

The minimum envelope records a governed attempt and kernel-local verification.
It cannot prove durable capability by itself. Full-spec promotion must consume
independent, reliability-weighted declarations and require evidence of variation,
recovery, and reduced orchestration burden; otherwise identical successes can
manufacture confidence without demonstrating transfer or resilience.

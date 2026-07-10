# Lessons

Durable, verified, project-specific lessons that improve future agent work or
prevent easy or repeated mistakes belong here. Keep entries concise, dated, and
linked to evidence. Do not use this file as a task log or scratchpad.

## 2026-07-10: Keep executable operations narrower than authority actions

The minimum conformance matrix requires authority to classify reserved and
malformed surfaces that the planner must never execute. A closed `Operation`
cannot serve both roles. `AuthorityAction` is therefore the canonical broad gate
input, while `Operation` remains the two-variant executable planner contract.

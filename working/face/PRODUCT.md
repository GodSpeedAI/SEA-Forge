# PRODUCT.md — FACE Workbench

<!--
WHAT THIS FILE IS FOR
This is the one document an agent CANNOT derive from the code, the spec, or git history:
who the product is for, what job it does, what matters more than what, and what "better"
means. SPEC.md says what to build; DEV_PLAN.md says in what order; DESIGN.md says how it
looks and speaks. THIS file says WHY — and it is what an agent reads to break ties when
two implementations are both valid.

HOW TO FILL IT OUT EFFECTIVELY
- Write answers, not aspirations. "Developers running >3 coding agents in parallel" beats
  "power users". If you can't name a real person/team who fits, the segment is a guess — mark it.
- Every {{PLACEHOLDER}} is a question only you can answer. Pre-filled text below a placeholder
  is a DRAFT inferred from dump.md/DESIGN.md — keep it, edit it, or delete it, but decide.
- Anything you leave as [UNDECIDED] is honest and useful: agents will ask instead of assuming.
- Revisit after every design-partner conversation. Stale product docs are worse than none —
  agents will confidently build for last quarter's thesis.
- Keep it under ~2 pages. If a section grows, the overflow belongs in a linked doc, not here.
-->

Status: DRAFT — placeholders unresolved. An agent reading this MUST treat `{{...}}` and [UNDECIDED] as open questions, not requirements.

Owner: sprime01 · Last reviewed: 2026-07-09

---

## 1. One-liner

<!-- The sentence you'd say to a smart friend. Category + who + core promise. No adjectives
     that a competitor couldn't also claim. Test: does it exclude anyone? If it excludes no
     one, it says nothing. -->

{{ONE_LINER}}

> Draft: FACE is the mission-control workbench for governed AI agents — it shows a developer what is preventing settlement, what the cheapest next move is, and what evidence will prove it.

## 2. Who it's for

<!-- 1–3 segments MAX, ranked. For each: who they are, what they're doing today instead,
     and the trigger moment that makes them look for this. The trigger is the most valuable
     line — it's what marketing, onboarding, and the first-run experience get built around. -->

### Primary: {{PRIMARY_SEGMENT}}

- Who: {{e.g. "solo/small-team developers running multiple coding agents daily"}}
- Doing today instead: {{e.g. "tabbing between terminal panes, GitHub PRs, and chat logs"}}
- Trigger moment: {{e.g. "an agent shipped something wrong and they couldn't reconstruct why it was allowed"}}

### Secondary (optional): {{SECONDARY_SEGMENT}}

- [UNDECIDED — delete this section if v1 serves exactly one segment. One is usually right.]

## 3. The job to be done

<!-- Format: When [situation], I want to [motivation], so I can [outcome].
     This should match SPEC.md §1's JTBD in spirit but be about the PRODUCT, not the repo
     foundation. If your JTBD contains the word "manage", dig deeper — nobody wants to manage. -->

When {{SITUATION}}, I want to {{MOTIVATION}}, so I can {{OUTCOME}}.

> Draft: When my agents are working in parallel and one hits a decision that needs my judgment, I want the workbench to surface exactly that settlement — with its evidence, policy verdict, and three-option choice — so I can decide in seconds and trust the outcome later.

## 4. What "good" looks like (success metrics)

<!-- 2–4 metrics. Each needs: the metric, the target, and HOW it's measured (instrumented
     where? which OTel metric / event?). A metric nobody wired into telemetry is a wish.
     Tie these to SPEC.md §12.3 required metrics where possible. -->

| Metric | Target | Measured via | Why it matters |
| --- | --- | --- | --- |
| {{ACTIVATION_METRIC e.g. "time from clone to first settled decision"}} | {{TARGET}} | {{INSTRUMENT}} | proves onboarding works |
| {{CORE_LOOP_METRIC e.g. "median time-to-settle a pending decision"}} | {{TARGET}} | {{INSTRUMENT}} | the product IS this loop |
| {{TRUST_METRIC e.g. "% of settlements with complete evidence at decision time"}} | {{TARGET}} | {{INSTRUMENT}} | the differentiator |

## 5. Scope ladder (what matters more than what)

<!-- THE tie-breaker section. When an agent must choose between speed, completeness,
     polish, and safety, this ordering decides. Rank ALL of these — a tie is a non-answer.
     Example ordering shown; reorder to taste. -->

1. {{e.g. Evidence integrity — never show a state the system can't back with evidence}}
2. {{e.g. Decision speed — the settle loop stays under N seconds end-to-end}}
3. {{e.g. Keyboard-first density — DESIGN.md ergonomics}}
4. {{e.g. Breadth of integrations}}

Explicitly out of scope for v1 (agents: do not build these even if easy):

- {{OUT_OF_SCOPE_1 — e.g. multi-user / team accounts}}
- {{OUT_OF_SCOPE_2 — e.g. light mode (per DESIGN.md §2)}}
- {{OUT_OF_SCOPE_3}}

## 6. The v1 walking path

<!-- The single end-to-end user journey v1 must nail, as numbered steps. One path, not a
     feature list. dump.md §"First settlement target" is your raw material here. Everything
     in DEV_PLAN.md exists to make THIS path buildable. -->

1. {{STEP_1 e.g. "User connects a repo and a running agent"}}
2. {{STEP_2 e.g. "Agent hits a policy gate; a Settlement appears center-screen"}}
3. {{STEP_3 e.g. "User reviews evidence checklist + three affordance options"}}
4. {{STEP_4 e.g. "User settles; decision + evidence land in the trace"}}
5. {{STEP_5 e.g. "CLI shows the identical settlement via `face settle`"}}

## 7. Positioning and non-goals of voice

<!-- Short. What FACE is NOT (competitor shapes to avoid) and the 2–3 brand truths from
     DESIGN.md §8 that product decisions must respect. This prevents an agent from
     "improving" the product into Jira. -->

- FACE is not: {{e.g. "a task tracker, a chat app, an agent vendor console"}} (DESIGN.md: never Jira/Monday/Asana/ClickUp).
- Voice invariants: cybernetic not dopamine; corrective errors; recognition over recall (DESIGN.md §8).
- Category bet: {{CATEGORY_TERM — the noun you want to own, e.g. "settlement workbench"}} [UNDECIDED]

## 8. Riskiest assumptions

<!-- List the 3 assumptions that, if wrong, kill the product — and the cheapest test for
     each. Agents should treat work that de-risks these as higher value than work that
     doesn't. Pull candidates from SPEC.md §5's Assumption rows and your design-partner
     conversations. -->

| Assumption | If wrong... | Cheapest test | Status |
| --- | --- | --- | --- |
| {{ASSUMPTION_1 e.g. "developers will pause for a settlement instead of overriding the agent"}} | {{CONSEQUENCE}} | {{TEST}} | untested |
| {{ASSUMPTION_2}} | {{CONSEQUENCE}} | {{TEST}} | untested |
| {{ASSUMPTION_3}} | {{CONSEQUENCE}} | {{TEST}} | untested |

## 9. Roadmap horizon

<!-- Three buckets only: Now (this plan), Next (after v1 proof), Later (directional).
     Dates are optional; ordering is not. dump.md's digital-twin / projection ideas
     belong in Later until Now is proven. -->

- **Now:** the foundation (DEV_PLAN.md Tasks 1–9) + {{FIRST_FEATURE}}.
- **Next:** {{NEXT_1}}, {{NEXT_2}}
- **Later:** {{LATER — e.g. TUI projection, digital-twin state bus, CopilotKit projection layer (dump.md)}}

---

## How agents should use this file

- Tie-breaks: when two valid implementations differ in what they optimize, follow §5's ladder.
- Unfilled placeholders: ask the user, or pick the draft and flag the choice in your summary — never silently invent product intent.
- Drift check: if a task seems to contradict this file, the file wins over the task; surface the conflict instead of proceeding.

# B. Target system and UX specification

One operating environment for a Field Case, delivered by two existing surfaces with one seam: **Horizon** (the
desk: understand, compare, decide, hand off) and **Cognate Workbench** (the bench: run and steer delegated work).
Everything shown is a projection of a record owned elsewhere (deliverable E §1). This spec extends
`Horizon-gaps/.agents/specs/Horizon_Operator_Workspace_Design_v2.md` and Workbench `src/ui/workbench/UX.md`;
it does not replace their layout, tokens or rules.

## 1. Design rules (inherited, binding)

1. The desk never decides. Decision controls submit a request; the result shown is the owner's record.
2. Four claims are always separate on screen: **accepted** (request taken), **settled** (independent result
   S/F/U/C), **observed** (what records show), **improved** (developmental claim). So are the four status axes
   (`SOP §7.6`): lifecycle, execution, result, disposition. No combined "status" pill.
3. Every value carries a label from the existing set: `recorded`, `derived`, `unknown` (naming the source that is
   silent), `effect`, `untrusted_reference`. Unknown is shown as the word `unknown`, never zero or blank.
4. Blocked possibilities are shown, with the reason and who can unblock (`SOP §1.2`, §7.2).
5. Deliberate friction only on commit, approve, settle, dispose, promote. Everything else is one keystroke.
6. Quiet, dense, legible; WCAG 2.2 AA; colour never the only signal; reduced motion honoured; 360 px usable.
7. State is words. One filled button per surface.

## 2. Navigation model

The desk keeps its six views. The Field Case gives each a definite job, so the operator moves by task, not by
feature:

| View | Operator's question | Field Case content | Representation |
|---|---|---|---|
| **Home** | which cases need me | cases by lifecycle; items awaiting my decision; cases with new contradictory evidence | table |
| **Today** | where was I, what is next | case continuation (`SOP §10.1`): purpose, current decision, pinned world and checkpoint, next affordable move or blocker, who can authorize | document with live fields |
| **Work** | what is running and what did it cost | episodes and assignments: frozen reference, execution state, budget used per currency, link to Workbench session | timeline + table |
| **Explore** | what is the world, and where does practice differ | declared model graph; observed overlay; divergence list; source for any element | graph + inspector + comparison |
| **Decide** | what am I being asked to commit | decision cards: gate assessment, settlement disposition, revision promotion, assignment authorization | sequential cards |
| **Cost** | what have we paid, for what accepted outcome | payment per currency per episode; cost per accepted outcome; unmeasured shown `unknown` | table |

A persistent **case strip** (case, `world_ref` short digest, checkpoint, lifecycle, my role) sits at the top of
every view and in Workbench. Selecting any record anywhere opens the same **inspector** with the seven debugger
operations as tabs: why, impact, diff, timeline, what-if, reach, replay.

## 3. Journeys

Each journey lists the records touched (owner in brackets) and the state change. "→ WB" marks the seam.

**J1 Open or resume a case** (`SOP §4`). Home → select or "New case" → Today.
New: brief form in the `SOP §4.2` groups; every field needs a status (`declared … unknown`) and a source or the
word `unknown`. Submit creates the case from `fde.field_case@1` [SEA-Forge] pinned to the current `world_ref`.
Resume: Today renders `case.continuation`; nothing is read from the journal as fact.
*Done when* a second operator can state the next move and its authorizer from Today alone.

**J2 Explore the world** (Explore). Graph of roles, operations, policies and flows from the pinned model
[DomainForge]. Selecting a node shows its definition, the SOP clause it came from (translation map), status, and
every occurrence observed [RealityTrace]. Search by term; search-around follows declared relations only.
Version switcher shows any earlier `world_ref` read-only with the evidence judged under it.

**J3 Aim: compare candidate moves** (`SOP §7.2`). In Today or Decide, "Next move" lists candidates as rows; columns
are the seven dimensions, each cell `ok | blocked: <reason> | unknown: <missing source>`. `reach` backs the cells.
Choosing an evaluated move opens the **freeze card**: intention, expected change, invariants, irrelevancies,
success and failure regions, falsifiers, evidence source, decision-maker. "Freeze" writes `episode.frozen`
[SEA-Forge] and the question/threshold [RealityTrace], then shows the digest. Exploratory moves are labelled
exploratory at this point and cannot later be shown as tests.

**J4 Delegate, monitor, interrupt, resume** (`SOP §7.3`). From the frozen episode, "Assign" opens the brief
(`SOP §7.4`) and budget, with a dry-run preview of the authority decision. Confirm → delegation [SEA-Forge] →
→ WB session. Work shows live execution state, spend per currency, denials and interruptions. "Interrupt" cancels
the continuation [Cognate]; "Resume" reopens it with the same episode reference. In Workbench the agent session
shows the case strip and cannot alter the frozen reference.

**J5 Claim: settle and dispose** (`SOP §7.5`). When execution ends, the episode appears in Decide for the
**verifier** (not the executor: the card is absent for the executor and the server refuses, I-2). The card shows
expected versus observed side by side, evidence with independence and reliability, and the proposed result.
Settle writes `episode.settled`. A second card asks the disposition authority for one of six dispositions, with
an A3-UV prompt for the reserved ones. The case strip updates lifecycle only if the disposition closes or
suspends it.

**J6 Investigate divergence** (`SOP §9`). Explore → "Differences". List grouped by class (confirmed, contradicted,
observed-undeclared, declared-unobserved, insufficient) with counts and coverage. Selecting one opens the
comparison view: declared element left, observed occurrences right (frequency, order, context, actors), and
`why` (the rule that fired, the occurrences that support it). "Diagnose" records hypotheses against the six
causes and what evidence would discriminate them.

**J7 Structural revision** (`SOP §9.4`). From a diagnosed difference, "Propose revision" links a candidate model.
The card shows the semantic diff, the retained-behaviour regression (pass/fail per prior confirmed element),
the what-if consequences, the migration and recovery path. "Request promotion" goes to an approver who is not
the proposer; approval is A3-UV. After promotion the case strip shows the new `world_ref`; earlier evidence stays
labelled with the world it was judged under.

**J8 Gates** (`SOP §8`). A gate panel in Today lists nine gates with state and the evidence each rests on. Stale
evidence marks the gate "reassess". Setting `not_applicable` requires a rationale and, for hard gates, approval.
No gate advances by itself.

**J9 Handoff and close** (`SOP §10`). "Hand off" previews the continuation and, when chosen, the DE or ADE field
lists; export produces the reusable bundle. Close requires a disposition with the economic account, remaining
liabilities and recoverability filled or marked `unknown`.

**J10 Compound** (`SOP §6`, §9.5). After a settled episode the case lists candidate reusable assets with the
evidence each has: variation, transfer, recovery. An asset is "candidate" until those exist.

## 4. States every surface must render

`empty` (with the first useful action), `loading`, `unknown` (source silent, named), `blocked` (reason and
authorizer), `stale` (source moved; refetch), `refused` (owner's refusal code and text verbatim), `error`
(source unreachable; nothing is shown as if current), `not connected` (system absent; how to connect).

## 5. Cross-product seam

| Moment | Horizon | Workbench |
|---|---|---|
| Assignment created | shows session link and live state | opens session bound to `assignment_id`; shows case strip |
| Agent needs a P3 decision | decision card appears in Decide | shows "awaiting decision at the desk", link |
| Operator steers | — | steering, terminal, browser as today |
| Work ends | episode enters Decide for the verifier | session becomes read-only trajectory |
| Operator signs in | one identity for both | same |

## 6. What is deliberately not built

A drag-and-drop application builder, a pipeline builder, ad-hoc analytics canvases, a second case UI, and any
merging of the Horizon and Workbench codebases. The SEA Forge Workbench and the Casework cognitive UI remain
SEA-Forge administration consoles and are outside the FDE journey.

## 7. Acceptance of the UX

Each journey J1–J10 has a Playwright test against live services (not mocks) and an agent-browser exploration
pass, per repository convention. J2, J5, J6 and J7 additionally have parity tests against the programmatic
result (deliverable G). A fresh operator walk-through, given only the SOP and the running system, must complete
J1, J3, J5 and J6 unaided; where they stall is recorded as a defect.

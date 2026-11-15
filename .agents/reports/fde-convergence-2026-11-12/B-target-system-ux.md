# B. Operator experience specification (draft 2)

Governed by `H-egm-review-record.md` (RD-03, 06, 10, 12, 13, 14, 19) and `_revision-ids.md`. Cites the procedure as `SOP §n.m` (v0.4.1 keeps v0.4 clause numbers).

## 1. What the operator gets

You open one case, see where it stands, pick a next move you can afford and someone may approve, write down what you expect before anyone acts, hand the work to an agent, and later see what the independent record shows next to what was expected. You decide only at the moments that matter; the desk says plainly what is waiting, what is blocked and why, and what nobody has recorded. Every figure is read from the part that owns it.

This **extends** the approved Horizon desk (`Horizon-gaps/.agents/specs/Horizon_Operator_Workspace_Design_v2.md`, "v2") and the Workbench UI rules (`cognate-workbench/src/ui/workbench/UX.md`). It keeps their layout, tokens, attention rules and words. A Field Case adds content to existing places: no new view, no second case screen, no second list of work.

What the code has today (verified, read-only): the left column has **five** views (Today, Conversation, Explore, Decide, Work: `Horizon-gaps/src/ui/desk/model.ts:14-20`, `Nav.tsx:14`, keys 1-5 at `keys.ts:39`); Conversation was added after v2. `views/Home.tsx` is not a view (first-run helpers used by Today); Cost is rendered inside Work (`views/Work.tsx:187`). The inspector offers seven operations (`Answers.tsx:104`): why, impact, diff, timeline, counterfactual, reach, replay.

## 2. Kept / extended / unchanged / not used

| v2 element | Status | What the Field Case adds, or why unchanged |
|---|---|---|
| Layout (left column, open view, inspector, composer) | unchanged | |
| Status bar | extended | adds the open episode and its execution state in words ("Episode 2: running, 3 of 5 checks"), and the Rules edition short code; still only what the parts report |
| Views Today / Explore / Decide / Work | extended | contents in section 5; Cost stays inside Work |
| Conversation view (present in code) | unchanged | agent-question answers and hand-offs appear here as today |
| Lists Days / Cases / Tags | extended | Cases list shows case lifecycle in words; Tags unchanged (a tag is never evidence) |
| Inspector | extended | one inspector; adds the debugger operations and, per record, source mark and owner (section 5) |
| Composer, modes Write / Find / Ask / Hand off | Write, Find, Ask unchanged; **Hand off extended** | Hand off becomes the only way to start an evaluated episode (J3/J4); text is the move's intention |
| Keys | unchanged | no new key completes a decision; A/R/H still only open a preview |
| Nine attention rules | unchanged | inherited by reference (section 3) |
| Decision card | extended | same form; new fields (expected, observed, difference, known, not known, rule) filled from the episode; new types: settle, disposition, promotion, agent question |
| Preview with check code | extended | adds the freeze preview and the promotion preview; "preview out of date" applies to both |
| Comparison view | extended | adds two lists (episode differences, structural differences, never mixed) and the five difference words beside v2's four |
| Notebook (Write, days, links) | unchanged | case timeline and notes meet in Today; notes still never count as evidence |
| Agent panels | unchanged | an answer to an agent's question must be replied to before the step can be accepted |
| One door for work | unchanged | the freeze and assignment ride the same path (J3/J4) |
| Labels (run pass/fail judgement) | unchanged | v2 keeps the word "label" for your pass/fail judgement of a run only |
| Cost per accepted outcome | extended | per currency (time, attention, money, risk, recovery) per episode; unmeasured shows `unknown` |
| Lessons | unchanged | an asset proposal in J10 is a separate action and is not a lesson |
| Presenting switch | unchanged | stand-ins also cover case and episode names in exports |
| Home and Cost as views, "case strip", six views | **not used** | draft 1 invented them; the status bar and Work replace them |

## 3. Design rules

Inherit v2 section 5.2 rules 1 to 9 unchanged (one highlight; agents never move your cursor; Escape; three strengths of interruption; no scroll jumps; state in words; "not recorded"; problems first; pinned stays put), v2 section 5.8 (plain words, one filled button) and Workbench UX hard rules 1 to 13. The Field Case adds:

- **F1 Separate claims.** Four claims never merge: **accepted** (a person said yes), **settled** (independent result), **observed** (what the record shows), **improved** (a developmental claim, only ever offered, never inferred). One primary claim per card.
- **F2 Four axes as one line of words.** Lifecycle, execution, result, disposition: "Open. Running. Result not settled. No disposition." Never a single pill. Detail is in the inspector.
- **F3 Source marks.** Each value shows where it comes from: recorded, derived, effect, untrusted reference. A value with no record says "not recorded, and names the silent source". A payment not measured says `unknown`. Never zero, never blank.
- **F4 Blocked moves are explained**: the blocking dimension in words and who can change it. A greyed control says why.
- **F5 Unknown never zero.** Also for counts, coverage and cost.
- **F6 Reserved decisions are confirmed on the paired phone, never on the desk page.** The desk shows "Confirm on your phone" and waits; it never renders the confirmation itself, even with a passkey prompt (RD-06, NT-24). Before S3 the phone check is not yet enforced: reserved confirmations use the existing approval path, and the desk says the confirmation is session-level, not phone-confirmed (RD-21).

## 4. Glossary and crosswalk (RD-14)

Operator words are the only words on screen. System names appear here and in "records touched" lines.

| Operator word | SOP term | System term / record |
|---|---|---|
| Rules edition | world, `world_ref` (SOP §4.2) | DomainForge model identity `world:<name>@sha256:...`, shown as a short code |
| checkpoint | checkpoint (SOP §4.2): the last agreed point in the case's history that evidence is judged from | case keeper checkpoint id |
| case keeper | SEA-Forge / Casework (SOP §3.4) | SEA-Forge; its request protocol is SFWP |
| record keeper | RealityTrace / SXR | RealityTrace (expectations, observations, residuals) |
| workshop | Cognate Workbench | Workbench |
| episode | episode (SOP §2.1): one bounded inquiry or change with a frozen expectation | `episode.frozen`, `episode.settled`, plan item `(case, step, n)` |
| run | run (SOP §2.2): one runtime execution | the record keeper's `run_id`; an episode has n runs |
| assignment | Case Assignment (SOP §2.1, §2.2): one delegation of work | `assign:<caseId>:<stepId>:<n>:<k>`; `n` is the episode counter, `k` the delegation instance within episode `n` |
| session | session (SOP §2.2): one workshop working period | a sitting of turns in the workshop; an assignment has n sessions |
| step | plan item | case keeper plan item |
| expectation (frozen) | frozen episode reference (SOP §7.2) | frozen reference with check code, anchored outside the executor's reach |
| freeze | freeze (SOP §7.2) | record keeper plan, question and threshold first, then case record, then anchor (two-phase) |
| settle | epistemic result S / F / U / C (SOP §7.5) | `episode.settled` |
| dispose | authorized disposition (SOP §7.5), shown with its level, case or episode | `disposition.recorded {level ∈ case, episode}` |
| close | lifecycle `closed` (SOP §7.6) | derived: terminal state and a close disposition both exist |
| difference | residual (unresolved difference, SOP §2.2) | `residual` record, with a field saying episode or structural |
| carry-over | residue (what survives for reuse, SOP §2.2) | reusable-asset candidate and its evidence |
| source mark | none | provenance tag on a value |
| label | none | v2's pass/fail judgement of a run; never a provenance tag |
| blocked move | affordance blocked (SOP §1.2, §7.2) | a candidate failing one of seven dimensions |
| gate not met | gate state `blocked` (SOP §8.1) | `gate.assessed` with evidence missing or contradicting |
| verifier | independent observer/verifier (SOP §3.1) | principal named on the case's observer-and-authority record |
| approver | approver of a promotion | principal with the promotion authority, never the proposer |
| disposition authority | settlement and disposition authority (SOP §4.2) | principal(s) on the same record; may be the person who is also verifier, subject to I-2 |
| case summary | case continuation (SOP §10.1) | `case.continuation` read model |
| resume point | none | where an interrupted run restarts (runtime continuation) |
| accepted / settled / observed / improved | four claims (SOP §11), see F1 | decision / result S-F-U-C / record keeper / developmental proposal |
| four axes | four status axes (SOP §7.6), see F2 | lifecycle, execution, result, disposition; improved is not an axis |

Terms B uses, defined once:
- **Template** (`fde.field_case@1`): the case form skeleton with the SOP §4.2 groups.
- **Reserved decisions**: settling by a human, a case-level disposition, setting a hard gate not applicable, promoting a Rules edition, recovering the controller. **P0/P1**: read, write, freeze, assign within a grant. **P2**: reserved to a human or an independent verifier. **P3**: also needs an online case-keeper decision bound to the exact operation. **A3**: confirmation on a surface that the agent cannot draw (the paired phone), checked by the case keeper.
- **I-1** no work before the freeze; **I-2** the settler is not in the executor's chain; **I-3** a freeze is never edited, changed criteria make `n+1`.
- **Seven dimensions** (SOP §7.2): visible, reachable, payment, executable, authorized, recoverable, settleable. **Six causes** (SOP §9.2): see the SOP.
- **Translation map**: link from a model element to the SOP clause it came from. **DE / ADE**: delivery engineering / agent engineering receivers (SOP §10.2).
- **Five difference words**: confirmed, contradicted, observed but not declared, declared but not seen, insufficient (v2's four plus insufficient: the record cannot decide).
- **Exploratory**: an observation not tested against a frozen expectation; never shown as a pass.

## 5. Navigation on the v2 layout

| View | Operator's question | Field Case content | Representation |
|---|---|---|---|
| Today | where was I, what is next | case summary at the top (purpose, current decision, Rules edition, checkpoint, next affordable move or what blocks it and who can change it, open episodes, remaining budget per currency, disputed claims); below, the **case timeline** (decisions, freezes, settles, dispositions, differences, interruptions, newest last, with your notes in the same list) | document, then time-ordered list |
| Explore | what is the world, where does practice differ | model graph from the pinned Rules edition; observed overlay; two difference lists (episode, structural); each node shows its definition, source clause and source mark | graph, list, comparison view |
| Decide | what needs me | cards oldest first, one highlighted: settle, disposition, promotion, gate, agent question; count on the view name ("Decide (3)") | decision cards |
| Work | what is running and how did it go | assignments and sessions with the workshop link, episode execution state, comparison view, labels, **Cost** (per currency, per accepted outcome) | list plus comparison, cost table |

Status bar adds: open episode and execution state, "confirm on your phone" when one is pending. **Inspector** operations by real name, and what exists today versus needs WP-40:

| Operation | Answers | Today | Needs WP-40 |
|---|---|---|---|
| why | why this value or difference | object trace | the rule that fired and supporting occurrences |
| impact | what depends on it | edge walk | regression consequences of a candidate edition |
| diff | what changed | object diff | semantic diff of two Rules editions |
| timeline | order of events | existing | case and episode events |
| counterfactual | what if absent | edge walk | regression matrix result |
| reach | can this move be made | capability ladder | move availability across the seven dimensions |
| replay | does it recompute | equality check | re-derivation from raw records |

Until WP-40, an operation that lacks its source says "not recorded: <source>" and shows what it can.

## 6. Journeys

Format: steps; surface and representation; records touched (owner); decisions D and context switches S; what you need not remember and where the system holds it; done when. S counts desk to workshop crossings.

**J1 Open or resume** (SOP §4). 1 Launch opens Today. 2 New case: Write a note "New case: ..." or use the template form (groups of SOP §4.2, one group at a time; each field has an assertion status and a source or "not recorded"; unfilled groups are allowed, flagged). 3 Resume: Today shows the case summary. Surface Today, document. Records: field-case document, pinned Rules edition (case keeper, DomainForge). D 1 new, 0 resume; S 0. You need not remember where you stopped (case summary) or which field is stale (flagged). Done when a second operator states the next move and its approver from Today alone.

**J2 Explore the world** (Explore). 1 Open a node. 2 Inspector: definition, source clause, source mark, observed occurrences. 3 Version switcher compares two Rules editions side by side, read-only. Graph plus comparison. Reads only (DomainForge, record keeper). D 0; S 0. Done when you can name the source clause of any element.

**J3 Aim and freeze** (SOP §7.2). 1 Composer, Hand off: write the intention. 2 Candidate moves list (rows): each with the blocking dimension in words ("not allowed: needs approval"), price per currency (`unknown` where unmeasured), and a mark exploratory or evaluated; row opens the seven-dimension detail in the inspector. 3 Pick one: proposed step preview (check code). 4 Freeze the expectation: expected change, invariants, irrelevancies, success and failure regions, falsifiers, evidence source, decision-maker; prefilled from the template and the step text. 5 Confirm. **Failure behaviour (RD-12):** the record keeper's plan and thresholds are written first, then the case record, then the anchor; if the record keeper is unreachable nothing is frozen and nothing can be assigned, and the card says so. Records: plan item and `episode.frozen` (case keeper), plan, question, threshold (record keeper). D 2 (pick, freeze); S 0. You need not remember the seven dimensions or the code (inspector, card). Done when the frozen expectation shows its check code and the assignment can be created.

**J4 Delegate, monitor, interrupt, resume** (SOP §7.3-7.4). 1 After the freeze, the assignment is created through the same door; the brief is prefilled from the freeze (problem to adopt, evidence after action, settlement criteria) and you add tools, limits and budget per currency; preview shows the authority decision. 2 Confirm. 3 Work shows execution state, spend per currency, denials; the status bar shows the open episode. 4 Interrupt cancels the run; Resume restarts from the resume point under the same episode. Records: assignment, `assign:<caseId>:<stepId>:<n>:<k>` (case keeper), session (workshop). D 2-3; S 1 out to steer, 1 back; the return is announced by a count on Decide. Done when Work shows the episode complete, failed or cancelled with its cost.

**J5 Settle and dispose** (SOP §7.5). 1 A settle card appears in Decide (count on the view name). The verifier is whoever holds the verifier role on the case's observer-and-authority record, assigned when the case is opened or changed, and is not in the executor's chain (I-2); for anyone else the card is absent and the case keeper refuses. A fresh-context agent may be the verifier. 2 Card, v2 form: **Expected** (frozen), **Observed**, **Difference**, **Known**, **Not known**, **Rule**; one line of axes; the proposed result S/F/U/C; reliability detail is in the inspector. 3 Accept, reject, hold open a preview. 4 A human settling confirms on the phone. 5 A separate disposition card goes to the disposition authority: close satisfied, close failed, continue, revise, escalate, suspend; reserved, so "confirm on your phone". Settling never closes the case; only a close disposition does. Records: `episode.settled`, `disposition.recorded` (case keeper). D 2; S 0-1 (reading workshop evidence). Done when the card shows who settled, when, under which rule, and the axes line reads correctly.

**J6 Investigate a difference** (SOP §9). 1 Explore, Differences, two lists: **episode differences** and **structural differences**, never mixed; contradicted first (rule 8). 2 Open one: comparison view (declared or expected left, observed right: frequency, order, actors, coverage). 3 `why` names the rule in words and the occurrences. 4 Diagnose: pick causes among the six, add what evidence would tell them apart. Records: residuals (record keeper). D 1-6; S 0. Done when each difference has a recorded diagnosis or is marked "not yet diagnosed".

**J7 Revise the model** (SOP §9.4). 1 From a diagnosed difference, propose a revision. 2 Tabs on one card: semantic diff, regression matrix result (old class to new class per element; losses fail; weakenings listed), consequences, migration and recovery. 3 Request promotion. 4 The approver, not the proposer, sees a promotion card; confirms on the phone. 5 The status bar shows the new Rules edition; earlier evidence keeps its edition. Records: proposal, pin and promotion (case keeper, DomainForge, record keeper). D 2; S 0. Done when a subsequent episode shows under the new edition.

**J8 Gates** (SOP §8). 1 Today lists nine gates with state in words: met, not met, not assessed, not applicable; "evidence changed since assessed" is a flag, not a fifth state. 2 Select a gate: evidence it rests on. **Gate not met** means evidence is missing or contradicts; **blocked move** means one candidate fails a dimension. Who can change a gate: anyone assesses from evidence; setting not applicable needs the disposition authority, and for hard gates also case-keeper approval on the phone. No gate advances itself. D per gate 1; S 0. Done when each gate shows its evidence or "not recorded".

**J9 Hand off and close** (SOP §10). 1 Preview the case summary for a receiver (DE or ADE field list). 2 Export bundle. 3 Close needs a close disposition with account, remaining liabilities, recoverability (each filled or `unknown`); reserved, phone. Document. D 3; S 0. Done when a different operator resumes from the export.

**J10 Carry over** (SOP §6, §9.5). 1 After a settled episode, Today lists candidate reusable assets with evidence held and lacking (variation, transfer, recovery). 2 Action: **Propose as reusable asset** creates a candidate; it stays "candidate" until the missing evidence exists. 3 A person accepts through Decide. Records: candidate asset (case keeper). D 1; S 0. Done when every candidate shows what is still missing.

Agent-question card (Decide): an agent's question with its context, your answer, and its required reply; the step cannot be accepted until the agent replied.

## 7. States every surface renders

`empty` (first useful action), `loading`, **not recorded** (no data; names the silent source), `blocked` (reason and who can change it), `changed since you looked` (stale), `refused` (owner's refusal code and text verbatim), `error` (source unreachable; nothing is shown as current), `not connected` (how to connect), `waiting for your phone`.

## 8. Desk and workshop seam

| Moment | Desk | Workshop |
|---|---|---|
| Assignment created | Work shows session link and state | opens session bound to assignment and episode; shows case and episode; cannot alter the frozen expectation |
| Agent needs a decision | card in Decide, count on the view name | "awaiting your decision at the desk", link; never moves your cursor |
| Steering | none | steering, terminal, browser as today |
| **Return trip**: work ends | count on Decide changes; settle card queued for the verifier | session becomes read-only; "back to the desk" link |
| Interrupt or resume | Work buttons | session reflects cancel or restart |
| Sign-in | one identity | same |

## 9. Where do I go

| Task | Surface |
|---|---|
| Open, resume, aim, freeze, hand off, decide, compare, revise, hand over | desk (Horizon) |
| Steer or watch an agent, terminal, browser | workshop |
| Administer the case keeper or its consoles (SEA Forge Workbench, Casework cognitive UI) | SEA-Forge administration consoles: administration only, not linked from any operator path |

## 10. Deliberately not built

A builder for applications or pipelines, ad-hoc analytics canvases, a second case screen, a merged Horizon and Workbench codebase, any single success score, any confirmation of a reserved decision on the desk page, any agent-movable cursor or place.

## 11. Acceptance

Per journey J1-J10: Playwright against live services, plus an agent-browser exploration. The five proof-critical journeys (J1, J3, J5, J6, J7) get a fresh-operator walk-through given only the SOP and the system; a stall is a defect. Also a minimal-card check, a 360 px check (stacked move list; no horizontal scroll), parity tests of rendered text against the ledger result, and NT-24.

## 12. Coverage of the mission's nine capabilities

| Capability | Journeys | Surface |
|---|---|---|
| Explore the world and source-grounded relationships | J2 | Explore, inspector |
| Understand case, direction, history | J1, J8 | Today case summary and timeline |
| Discover affordances and blocked possibilities | J3, J8 | Hand off candidate list, inspector reach |
| Investigate evidence and discrepancies | J6, J5 | Explore differences, comparison view |
| Compose, delegate, monitor, interrupt, resume | J3, J4 | composer, Work, workshop |
| Inspect plans, changes, costs, approvals, outcomes | J4, J5, J7, J9 | Work (cost inside), Decide, inspector |
| Decide at consequential boundaries | J5, J7, J8, J9, J10 | Decide, phone |
| Compare candidate changes with current representations | J3, J7 | candidate list, J7 card, version switcher |
| Preserve work and developmental knowledge | J1, J9, J10 | notebook, timeline, export, carry-over |

## 13. Decisions made while revising

- Conversation is listed as a kept view because the code has it; v2 does not.
- "Rules edition" (H) is used for v2's "rulebook edition".
- "Continuation" is split into case summary and resume point; "reassess" is a flag on a gate, not a fifth state.
- Candidate-asset acceptance goes through Decide.

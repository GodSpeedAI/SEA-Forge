# 06 Comprehension and operator experience (Comprehension Goldfish, REVIEW mode)

Read: B, SOP v0.4, C, E, G (full); Horizon PRODUCT.md; Horizon design v2 §2, §5, §7.1-7.5. Not read: A, D, F, Mission.md
(brief limited me to B/C/E/G/SOP + Horizon). Everything below is VERIFIED against those texts unless marked INFER.

## Paraphrase
**What B proposes.** Keep two products: Horizon (desk: understand, compare, decide) and Workbench (bench: run agents).
A "Field Case" is a Casework case plus extra records. The desk gets six views (Home, Today, Work, Explore, Decide,
Cost), a case strip, one inspector with seven tabs, and ten journeys J1-J10 that walk the SOP. All desk controls
only submit requests; results shown are other systems' records.
**How the existing system works, per B/E.** SEA-Forge owns case/episode/settlement truth, Cognate runs agents,
RealityTrace stores expectations/observations/differences, DomainForge owns the model and `world_ref`. Horizon and
Workbench are projections. Horizon-to-Workbench handoff is an `assign` call; P3 approvals stay in Horizon.
I could paraphrase every section of B except Today ("document with live fields") and the seam table row 2 (see F-04).

## Findings (ranked)

**F-01 MAJOR. B contradicts the Horizon design it says it extends.** B §0: "extends ... v2 ... does not replace their
layout". B §2: "The desk keeps its six views" (Home, Today, Work, Explore, Decide, Cost). v2 §5.1: four views (Today,
Explore, Decide, Work) plus lists Days/Cases/Tags, Today opens on launch; cost lives in Work (§5.1 table, §8.3).
B adds Home and Cost, drops the Days/Cases/Tags lists, and replaces the v2 status bar with a "case strip" without
saying which is which. B also drops: composer and Hand-off mode (v2 §5.3, §7.1), preview with check code and
"preview out of date" (§5.6), attention rules (§5.2), pinning, keys (A/R/H open a preview), Presenting, the notebook
(§6, mission item "preserve work and knowledge"). Fix: B must list each v2 element as kept / changed / dropped with a reason, or revert to four views and put Cost inside Work.

**F-02 MAJOR. "One door for work" is bypassed.** v2 §7.1/7.4: work enters only via Hand off (composer) -> proposed
step -> preview -> expectation recorded -> assignment; "Horizon keeps no list of work". B J4: "From the frozen episode,
'Assign' opens the brief and budget, with a dry-run preview". No composer, no proposed-step state, no idempotency
reference ("asking twice changes nothing"). Fix: express J3/J4 as Hand off plus freeze, or state that Assign replaces the door and why.

**F-03 MAJOR. Vocabulary collisions (same word, different thing; different words, same thing).**
- "label": B rule 3 = value provenance tag (`recorded`, `derived`...); v2 §2 "Label" = operator's pass/fail judgement of an agent run (§5.7, §8.2). A reader of both docs cannot tell which. Rename one.
- "blocked": gate state (SOP 8.1), move cell (`blocked: <reason>`, B J3), state-renderer (B §4), and "blocked by" in v2. Never distinguished (see answer iii).
- "residual" (unresolved difference, SOP 2.2) vs "residue" (what survives, SOP 2.2) vs "difference/divergence/Differences" (B J6, C R-47, E "residual.computed" per difference). B's Explore says "divergence list", J6 says "Differences", E calls the record a residual. Same object, three names, and two of the near-homonyms are defined to mean opposite things.
- "episode" / "run" / "assignment" / "session": SOP 2.1 episode = inquiry; E §3 "one run of a plan item"; `run_id` (E §2) vs Cognate "runs" vs v2 "session" (a sitting of turns) vs Workbench "session" bound to `assignment_id`. J4 says "Interrupt cancels the continuation [Cognate]"; "continuation" is also SOP 10.1's case-continuation read model. Two unrelated "continuation"s.
- "settle / dispose / close / settlement": episode.settled (S/F/U/C result), disposition (six choices), close (lifecycle) are three steps; B J5 title "settle and dispose" but J9 "Close requires a disposition" and G "settlement" is also a business-outcome concept (G §2 R-03) and a RealityTrace `settlement_ref`.
- "world / world_ref / model / checkpoint / world registry / Rules edition": v2 calls the pinned model "rulebook edition"/"Rules: payments 1.0.0"; B/E call it `world_ref` ("world:<name>@sha256..."). The case strip shows "short digest". "Checkpoint" is never defined in B, C, E or SOP (only "pinned world and checkpoint").
- "verifier / approver / disposition authority / criterion owner / decision-maker": B J5 verifier settles; J5 "disposition authority"; J7 "approver who is not the proposer"; J3 freeze card "decision-maker"; SOP 4.2 "settlement and disposition authority" (one role, two duties). Whether these are four roles or one is not stated.
- "accept": v2 decision = accept/reject/hold; B "accepted (request taken)" is a different claim (SOP/PRODUCT "accepted" = request accepted). v2 state "accepted" = person said yes. Same word, opposite end of the pipeline.
- "claim": B rule 2 "four claims" (accepted/settled/observed/improved) vs "Claim" step of Golden Key (SOP 7.5) vs four status axes (lifecycle/execution/result/disposition). Two sets of four; their relationship (accepted~?, settled~result) is never mapped.
- "gate": SOP 8 evidence gates (nine, case level) vs PRODUCT "gate state" (per-move authority gate) vs v2 "seven checks".
Fix: one glossary table (B or SOP §2.2) mapping each concept to its one name per layer, and a crosswalk of v2 state words to the four axes.

**F-04 MAJOR. Terms used in B before/without definition.** SFWP, `fde.field_case@1`, P3, A3-UV (B J5 "A3-UV prompt for the reserved ones"; defined only in E §8, as external-report jargon), I-2 (cited in B J5, defined in E), "reserved" dispositions (which of the six?), "seven debugger operations" (named: why, impact, diff, timeline, what-if, reach, replay; but E §9 lists `counterfactual` for what-if and has no `impact`), "seven dimensions" (listed only in SOP 7.2), "six causes" (SOP 9.2), "translation map", "DE or ADE" (SOP 3.1 only), "retained-behaviour regression", "freeze card", "exploratory", "observed overlay", "search-around", "RealityTrace" and "DomainForge" in brackets as owners with no gloss in B. SOP: "IDM", "ODI/JTBD", "Golden Key", "framing" vs "projection" vs "case horizon", "evaluation contract", "A:11" citations (source codes A/B resolve only in header). B never says what a Field Case brief form "status (`declared ... unknown`)" values are; the list is in E §3 (declared, observed, inferred, proposed, unknown) and ends with an ellipsis in B.

**F-05 MAJOR. Sentences read twice / hand-waved.** B §1.2 "four claims ... So are the four status axes" (parse). J4 "Confirm -> delegation [SEA-Forge] -> -> WB session" (doubled arrow, typo). J3 "writes episode.frozen and the question/threshold" (two records, two owners, one click, no failure behaviour if one fails). J1 "Done when a second operator can state the next move and its authorizer from Today alone" is the test, not a step. SOP 7.6: "close-satisfied ... set lifecycle closed" yet 7.5 says "settled episode is not a closed case": a one-person case with one episode closes on one disposition? The relation of episode to case closure is unstated. SOP 2.2 defines "residual" and "residue" one sentence apart.

## Answers from the documents alone

**(i) What the operator sees first when resuming.** Partly answerable. J1: "Resume: Today renders `case.continuation`", and Today = "purpose, current decision, pinned world and checkpoint, next affordable move or blocker, who can authorize" (B §2). But the entry path is "Home -> select ... -> Today", so the first thing seen is Home (a table), not the case; v2 says Today opens on launch. Order inside Today, and what happens with disputed claims, remaining budgets, active assignments, gate evidence (all in SOP 10.1) is NOT stated; "document with live fields" is not a layout.

**(ii) Who may settle and how the system knows.** "the **verifier** (not the executor: the card is absent for the executor and the server refuses, I-2)" (B J5). E §3: refused "when the caller's principal chain intersects the executor's". So the system only knows who is NOT allowed. How a person/agent becomes "the verifier" is not stated in B; SOP 4.2 has a case field "independent verifier", E has no positive binding. SOP 3.2 allows a fresh-context agent as verifier, but E §8 says every `episode.settle` is "P3 human-reserved or independently authorized" and B J5 only mentions A3-UV for the disposition. Contradiction or gap: can an agent settle? NOT answerable.

**(iii) Gate "blocked" vs move "blocked".** NOT answerable. Gate: one of `met | blocked | unassessed | not_applicable` (SOP 8.1; no definition of blocked, B J8 adds "reassess"). Move: cell `blocked: <reason>` among seven dimensions (B J3; SOP 1.2: a move must be affordable, authorized, executable, recoverable ...). B §4: `blocked` = "reason and authorizer". Likely a gate is blocked by missing/contradicting evidence and a move by a failed capability, but no doc says it, and "who can unblock" for a gate is never given.

**(iv) What happens on screen when an agent needs a decision.** Seam table only: Horizon "decision card appears in Decide"; Workbench "shows 'awaiting decision at the desk', link". Not stated: how the operator is notified while in Workbench or on another Horizon view (v2 gives count beside the view, notice in Decide, never moving the cursor); what the card contains; whether the agent pauses; what a "P3 decision" is (not in B); and the four Decide card types (B §2) do not include "agent question" (v2 §7.3 answers to agent panels). INFER: the operator must notice a count themselves.

**(v) Why a difference is "contradicted".** Answerable in outline: J6 comparison view, "`why` (the rule that fired, the occurrences that support it)"; E §6: DomainForge returns false for a policy over the generated instance overlay; E §9 `why` reads the residual record. B does not say the operator sees a policy name, the overlay, or in what words; the rule is a DomainForge policy, the operator never meets DomainForge in B. Also the "five classes" ("confirmed, contradicted ...") are shown without definitions (E §6 has them); v2 has four difference words (matched, contradicted, expected but not seen, seen but not expected) and "insufficient" is new.

## Part 2: operator walk of J1-J10 (decisions D, context switches S = Horizon<->Workbench)

| J | D | S | Holds in memory | Next step unclear | Representation problem |
|---|---|---|---|---|---|
| J1 open/resume | new: ~15-25 fields each needing status+source (SOP 4.2 has about 50 fields, 6 groups); resume 1 | 0 | which field is stale | after submit | Form of ~50 fields x (value, status, source) is a table, not a "brief form"; no progressive entry stated. Resume: fine as document |
| J2 explore | 0 | 0 | node found vs source clause | none | graph+inspector good; version switcher needs a comparison, only "read-only" stated |
| J3 aim/freeze | pick move (1), freeze (1, deliberate) + fill 9 freeze fields | 0 | 7 dims x N moves; 9 freeze fields; digest | what makes a move "evaluated" vs "exploratory" | candidate table 7 cols with 3-way cells is right; freeze card is a long form with no template or defaults. No say who authors criteria (criterion owner) |
| J4 delegate | assign, budget, confirm (3) + dry-run read | 1 out (Assign->WB); to monitor or steer, stay in WB or return | assignment_id, episode ref, budget per currency | where to watch: Work (timeline) or Workbench session? "Interrupt" lives in Work, steering in WB | agent session in WB fine; monitoring split across two apps |
| J5 settle/dispose | verifier: 1 settle + typed A3-UV? ; authority: 1 disposition (6 options) | 1 back from WB when work ends (INFER: no notice) | expected vs observed across sessions | who is verifier; verifier = different person means handoff off-screen | side-by-side card good; sequential cards good; but evidence "independence and reliability" (T/A/I/K/D/X/N) shown on the same card = overload |
| J6 investigate | classify/diagnose: 1-6 hypotheses | 0 | list of 5 classes + counts + coverage | which difference first (no priority rule; v2 rule "problems first" not carried) | comparison + why correct; "Diagnose" records 6 causes with discriminating evidence: a form, not specified |
| J7 revise | propose, request promotion (2) + approver 1 (A3-UV) | 0-1 | candidate vs current diff, regression results per prior element | who approves; waiting state | semantic diff + regression table + what-if + migration path on one card = four tasks on one sequential card; needs tabs or a document |
| J8 gates | 9 gates; each assess or N/A+rationale(+approval) | 0 | which evidence supports which gate | which gate to look at first | table of nine fine; "reassess" is a fifth state not in SOP 8.1 |
| J9 handoff/close | preview, hand off, export, close (3-4); economic account etc. | 0 | remaining liabilities | which receiver (DE/ADE) | document; fine |
| J10 compound | per candidate asset: 0 stated | 0 | what evidence an asset lacks | no action defined: who promotes a candidate | list only; no decision verb |

Totals: only J4 and J5 involve a stated seam crossing (2 per episode). Hidden crossings: reading Workbench trajectory for J5 evidence, and interruptions. The J5 return trip is not announced.

**F-06 MAJOR. Memory load across J3-J5-J6.** The operator must carry `episode_ref`, `assignment_id`, `world_ref` digest and the frozen intent between Today, Work, Decide and Workbench. Only the case strip (case, digest, checkpoint, lifecycle, role) is persistent. Missing: current episode and its status axes in the strip. Fix: add episode ref plus execution state to the strip and carry v2's pinned/"changed since you looked".

**F-07 MAJOR. Rule overload on a real screen.** Rule 2 (four claims) + four axes + rule 3 (label on every value, five label kinds) + the seven dimensions (cells x moves) + seven reliability dimensions on the settle card. J5's card alone has: expected, observed, evidence (+independence, reliability), proposed S/F/U/C, four claims, four axes, labels per value. v2's decision card has 6 lines and one highlight; B has no equivalent budget and no hierarchy (which claim is primary). 360 px usable (rule 6) cannot hold a 7-column move table; no degraded layout. Fix: define the minimum card (v2 5.5 form), with axes as a one-line strip and detail in the inspector.

**F-08 MAJOR. Missing/mismatched representations.** No timeline of case history on Today (J1 resume rests on one document); Work's "timeline + table" has no item specification; no agent-session transcript view in Horizon (only a link); no notebook/journal/preserve-knowledge surface (mission "preserve work and knowledge"; B mentions journal only as "not fact"); no composer; J10 has no decision control; no "agent asks a question" card; Cost as separate view while payment (SOP 5) must be known before choosing a move (J3 cells show only ok/blocked/unknown, no price per currency).

**F-09 MINOR. Dual brief.** J4 brief fields (SOP 7.4, ten items) plus budget plus dry-run in one step; no defaults or reuse from the case/freeze card (intention is entered in freeze, then re-entered as "problem the agent may adopt"). INFER: duplication.

**F-10 MINOR. Acceptance of UX tests only J1, J3, J5, J6 with a fresh operator** (B §7, G AT-19). J2, J4, J7-J10 never walk-tested; J7 (A3-UV approver) is the riskiest step.

## Mission capability check (B and C R-65)
Explore world: yes (J2). Understand case/direction/history: partial (Today; no timeline). Discover affordances and blocked possibilities: yes (J3). Investigate evidence/discrepancies: yes (J6). Compose: NO (no composer). Delegate/monitor/interrupt/resume: yes but monitoring split. Inspect plans/changes/costs/approvals/outcomes: partial (Cost view yes; "plans" has no view). Decide at boundaries: yes (Decide). Compare candidate changes: moves yes, revisions yes (J7). Preserve work and knowledge: weak (journal only, J10 no verb).
R-65 maps all capabilities to J1-J10 under one row with a single test (AT-19); "compose" and "preserve" are not traceable to any journal-free step.

## Right and must be kept
1. Desk never decides; projections only (B rule 1, E §1).
2. Unknown shown as the word `unknown`, never zero (B rule 3, SOP 5.3; same as v2 5.2 rule 7).
3. Journey-level done-when and fresh-operator walk-through as acceptance (B §7).
4. Separate episode comparison from structural comparison (SOP 9.1, R-46).
5. Seam rule: P3 approvals never in Workbench, and one identity for both (E §10).

## Not checked
A, D, F, Mission.md, recon/*; v2 §6, §8-13; Horizon source code or any running UI.

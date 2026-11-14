# A. Current-state architecture and evidence inventory

Date 2026-11-12. Read-only recon; nothing was built or run. Detailed, cited findings are in `recon/01`–`07`
(each claim there is tagged OBSERVED or INFERRED with `path:line`). This file is the consolidated view the
design depends on. "Behind/ahead" uses local `origin/*` refs, which were not fetched.

## 1. Repositories, heads and working state

| System | Git identity | Local checkout(s) | Branch @ HEAD | State | Pinned head (`gsr-heads`) |
|---|---|---|---|---|---|
| SEA-Forge / Casework | GodSpeedAI/SEA-Forge | `sea-rs` (+16 `_wt/*`, `sea-rs-fix`, `sea-rs-horizon`) | `docs/egm-bootstrap-readmes` @ b072ec50; origin/main 595b5270 (0.4.1) | local main 15 behind; 32 local branches | 4e67dfca |
| Horizon | GodSpeedAI/Horizon | `Horizon-gaps` (+9 worktrees) | `horizon-desk-gaps` @ e5b98b8, 150 ahead of main, clean | superset of all branches except `stage0b-containment-core` (3 wip + 8 dirty), `pr1`, `pr1-narrow`; unpushed, no PR | 55f1b42 |
| Cognate | GodSpeedAI/Cognate | `cognate`, `cognate-horizon`, `cognate-identity-wt` | main ≈ bcef663 (0.9.0); `feat/realitytrace-real-snapshot` (+1); `horizon/shared-identity` (+2/−3) | two unmerged feature branches | bcef663 |
| Cognate Workbench | GodSpeedAI/Cognate-Workbench | `cognate-workbench`, `cognate-workbench-horizon` | `feat/desktop-terminal-browser` @ e45e97d (37 dirty PNGs); `feat/horizon-connect` @ c4096b9 (unpushed, 7 unique commits) | desktop branch pins Cognate 0.8.0; main uses 0.9.0 | 1b6fcb4 |
| DomainForge | GodSpeedAI/DomainForge | `domainforge` | main @ f90905a | tooling-only dirt | **none** (readiness expects one) |
| RealityTrace / SXR | GodSpeedAI/RealityTrace | `sxr` | `feat/native-binary-distribution` @ c239d5c (8 ahead / 2 behind) | origin/main has agent-event ingest (#13) this branch lacks | 4d05cc0 |
| CEP | GodSpeedAI/canonical-evaluation-protocol | `cep` | `profiles/context-bundle` @ d7591c1 (PR #3 open) | tree identical to pin | 5111ec3 |
| Context Kernel | GodSpeedAI/Context-Kernel | `Context_Kernel` | main @ 36bd8c8 | pin is 25 behind main | 81a6fdb |
| Gauntlet | GodSpeedAI/Gauntlet | `gauntlet` | `release/native-distribution` @ 5842777 (12 ahead) | PR #7 awaiting merge | 1543cbf |
| SWE_SEED | GodSpeedAI/swe_seed | `SWE_SEED` | `feat/native-distribution` (2 ahead) | — | pinned |
| GodSpeed-Agent | GodSpeedAI/GodSpeed-Agent | `godspeed_agent` | `migration/cep-world-required-evidence` (33 ahead) | implemented, unmerged | not pinned |
| Readiness gate | local only | `godspeed-readiness` | main @ ae90607 | last GO 30/30 on 2026-10-25 heads, profile `local_operator` | — |

Systems outside the mission's list that matter: CEP (envelope and world-transition contracts), GodSpeed-Agent
(developmental layer, E1/E8 endpoints of the world loop), `godspeed-readiness` (the only existing integrated
gate), `gs-term` and `zeroclaw-sea-forge` (Cognate/SEA-Forge consumers; not needed for the baseline).

### Corrections after fetch and independent source review (2026-11-13)

All twelve repos were fetched; remote refs were already current (`evidence/baseline/heads-after-fetch.md`).
The codebase reviewer (`egm-review/03`) re-checked claims in source. Where the table above or the text below
disagrees with this list, this list is right:

- **Horizon:** `horizon-desk-gaps` is *not* a proven superset. Nine other branches carry patch-unique commits;
  each must be diffed for content before it is treated as covered. The branch is unpushed and has no PR.
- **SEA-Forge:** `feat/horizon-desk-support` is PR #91 and a clean fast-forward. `t1/*` and the
  settlement-claim `-main` branch are already on main by patch-id. `fix/server-ambiguous-declare-keeps-lease`
  is the pre-squash of merged #84. PR #71 (hardening harvest) is open.
- **CEP:** origin/main is 5111ec3, which already contains the `godspeed.context_bundle` profile, so Context
  Kernel is not ahead of the protocol. The undefined hash algorithm in CEP-0008 remains.
- **DomainForge:** `domainforge test` now fails loudly rather than exiting 0; the release-path `println!` debt
  item is stale. `validate --format json` emits violations only, cannot distinguish unknown from false, and
  skips operation-bound policies. Release 0.22.0 is pending (PR #165).
- **RealityTrace:** `agent-event-v1` is a closed coding-agent telemetry schema and cannot carry model identity.
  Cognate's outbox emits CEP `execution_trace` envelopes with `extensions`; that is the usable ingest path.
- **SEA-Forge kernel facts:** `PlanItem.markers` is three booleans (no activity tag); repetition needs the
  `repetition` marker; separation of duties is a single-actor `proposed_by` comparison; the world registry is
  built from host configuration and has no promote or pin write path; adding SFWP verbs and record kinds is
  additive (static verb table, free-string `record_kind`).
- **Horizon desk:** the approved v2 design has four views (Today, Explore, Decide, Work) plus Days / Cases /
  Tags lists, a composer and a status bar; the code's navigation has five entries (adds conversation), with
  cost nested in Work. Today's debugger operations are narrower than their names suggest: `reach` is a
  capability ladder, `counterfactual` an edge walk, `replay` a one-field equality check, `diff` an object diff.
  Horizon mints `assign:<caseId>:<stepId>` itself, one per step.
- **Open pull requests:** SEA-Forge #91, #71; Cognate #76; Workbench #28, #27; Gauntlet #7; DomainForge #165,
  #114 (draft).

## 2. Who owns what today (verified in code)

| Consequential state or action | Owner | Evidence |
|---|---|---|
| Semantic meaning: `.sea` grammar, graph, policy evaluation (three-valued), `world_ref = world:<name>@sha256:<digest>`, `ConceptId` (UUIDv5), semantic-pack diff, `ConceptChange` | DomainForge | recon/04 §A |
| Case state and ledger (`cases/<id>/{case,plan,authority,settlement}.json`, `evidence.jsonl`), case verbs, approvals, CEP authority loop, world registry and approval-gated world transitions, settlement (Accepted/Rejected/Escalated), delegation, sandbox | SEA-Forge | recon/01 §2–4 |
| Runs, durable event log, continuations (resume/cancel, `human_approval`), capability policy, shared state, artifacts, remote offers, observation outbox | Cognate | recon/03 §A |
| Agent work cockpit: tasks, contracts, review gate, experts/teams, terminal, managed browser, spend ceilings, mobile remote | Cognate Workbench | recon/03 §B |
| Operator desk: case UI, panels/answers, world debugger (`why/impact/diff/timeline/counterfactual/reach/replay`), handoff, notebook, GitHub connector, mobile remote; "the cockpit never decides" | Horizon | recon/02; `Horizon-gaps/src/domain/debugger.ts:698` |
| Expectation-before-observation ledger: hash-chained records, CAS, `question → threshold → observation → evidence → settlement`, `residual.computed`, `diagnostic.*`, `proposal.raised`, `semantic.snapshot/diff` | RealityTrace | recon/05 §A; `sxr-core/src/record_kinds.csv` |
| Evaluation envelopes, profiles, anti-collapse conformance, reality-gap benchmark, world-transition record | CEP | recon/04 §B |
| Scoped retrieval with citations, session memory, artifacts, `context_bundle` projection | Context Kernel | recon/04 §C |
| Frozen Reference, blind comparison, PASS/FAIL/INCONCLUSIVE ladder, promotion gate | Gauntlet | recon/05 §B |
| Pre-work proof contract, trace chain, `ProofCompleted` | SWE_SEED | recon/05 |

## 3. What works end to end (with recorded evidence)

| Path | Evidence | Limit |
|---|---|---|
| Casework UI → Go gateway → SFWP → kernel, full case lifecycle, three fresh-cell runs | `sea-rs/.agents/evidence/casework-live-wiring/T12/`, verdict CONFIRM WITH CONDITIONS (2026-10-05) | not re-run since |
| Horizon passkey mobile remote (P2) | `Horizon-gaps/test/E2E-remote.test.ts`, 9 Chromium tests; suite 1628 pass / 0 fail | desk browser UI never exercised end to end |
| Horizon ↔ SEA-Forge case keeper (SDK + NDJSON socket, event follower) | `test/desk-*`, `adversarial-case-keeper.test.ts` | live SEA-Forge required |
| Horizon → Workbench assign over Connect RPC with a key file | `horizon-connector/src/adapters/workshop.ts`; Workbench `J-AUTH-CONNECT-HORIZON` | Workbench side only on unpushed `feat/horizon-connect` |
| One-world loop E1→E8 (GodSpeed-Agent → Context Kernel → SWE_SEED → SEA-Forge → SWE_SEED → RealityTrace → GodSpeed-Agent) with four refusal paths | `sea-rs/scripts/e2e-world-loop.sh` | **no recorded run**; execution simulated at E5B |
| Readiness gate over pinned heads | `godspeed-readiness/scripts/readiness.sh` | `local_operator` profile only; no FDE journey |

## 4. Gaps against the mission (the work)

| # | Gap | Where | Consequence |
|---|---|---|---|
| G1 | No observed-interaction reconstruction and no declared-versus-observed structural comparison anywhere | SXR has no process mining; DomainForge accepts no observed state; CEP reality-gap bench is unbridged | Mission §4 and SOP §8 are unbuilt |
| G2 | No Field Case: no episodes, gates, strategy record, dispositions, or the four separate status axes | Casework has cases/plans/approvals only; `CaseState` has no suspended | SOP §4, §6, §7, §9 have no authoritative home |
| G3 | DomainForge cannot declare a lifecycle; the SOP's IDM does not exist | `sea.pest` has operations, policies, roles, flows; no stage construct | Mission §2 unstarted; acceptable because the SOP says activities and gates are not stages |
| G4 | Runtime events do not carry model-element identity end to end | Cognate RealityTrace outbox PARTIAL; SXR anchors by declaration key but verifies `world_ref` syntax only; SEA-Forge emits no `EvidenceRecorded` in production | Mission §3 unmet |
| G5 | Human and agent authority cannot be told apart at the desk; P3 enrolment proven only against an approving test double | Horizon critic item 4; dependencies D1–D6 across SEA-Forge and Cognate identity | SOP §3 role separation and Mission §7 "denied or unknown authority cannot promote" are not enforceable |
| G6 | Horizon has no views for divergence, structural revision, or subsequent-episode capability; RealityTrace adapter is CLI-per-call and unverified | `src/adapters/realitytrace-obligation.ts` | Mission §6 unmet |
| G7 | Contract skew: CEP `context_bundle` schema unmerged while Context Kernel implements it; CEP-0008 hash algorithm undefined; `ProofCompleted` and `world_ref` mirrored, not shared; Cognate 0.8.0 vs 0.9.0; two WebAuthn stacks; identity package under a different npm scope | recon/03 §C4–C5, recon/04 §B8 | integrated gate cannot be trusted until closed |
| G8 | Gauntlet has no link to RealityTrace, SWE_SEED or DomainForge; runners unconfigured | recon/05 §B | SOP's "bounded difficult execution" has no evidence path |
| G9 | Unmerged work everywhere: Horizon gaps (150), SEA-Forge `feat/horizon-desk-support` (5) and ~25 fix/feature branches, Cognate identity and snapshot, Workbench horizon-connect, SXR 8 ahead, GodSpeed-Agent 33 ahead, DomainForge LSP 7 | recon/01 §1, 02 §1, 03, 04, 05 | no single integrated baseline exists |
| G10 | No integrated release gate for the FDE journey; DomainForge unpinned | `godspeed-readiness` | completion cannot be asserted |
| G11 | Known false-green risks: `domainforge test` is a stub that exits 0; SEA-Forge identity falls back to a constant hash when the manifest is absent; live-loop tests skip without prerequisites | recon/04 §A7, recon/01 §8 | must be closed or excluded from proof |

## 5. Reusable UX already built

- Horizon desk: one layout with `Home / Today / Work / Explore / Decide / Cost`, decision card, preview, comparison
  view, notebook, handoff and handoff preview, agent door, panel blocks (`ask, ctx, data, guard, read, viz`),
  sequential UI, world-debugger answers with five honest value labels (`recorded, derived, unknown, effect,
  untrusted_reference`). Design brief: "quiet, dense, legible", WCAG 2.2 AA, deliberate friction on commit,
  approve and settle. Spec: `Horizon-gaps/.agents/specs/Horizon_Operator_Workspace_Design_v2.md`.
- Workbench cockpit: tasks, contracts, review gate, steering, experts/teams, terminal, browser, spending, session
  deep links `#session/<id>[/<eventId>]`; 13 hard UI rules in `src/ui/workbench/UX.md` ("state as words",
  "unknown shows `unknown`, never zero").
- SEA-Forge: SEA Forge Workbench (17 routes: cases, inbox, evidence, delegate, readiness, thoth…) and the
  Casework cognitive UI. These are SEA-Forge's own consoles; the design keeps them as administration surfaces
  and does not route the FDE journey through them.
- Existing IDM artifacts and method: `Horizon-gaps/.sea/interaction/` (model, source-translation map,
  assumptions-and-unknowns, validation output), `sea-rs/.sea/interaction/`, `cognate/.agents/skills/building-with-cognate`.

## 6. Verification status of this inventory

Not run in this phase: any build or test (host memory limits; recon was read-only). Every "tests pass" figure
above is quoted from the owning repository's latest report, not re-measured. The first implementation work
package re-establishes a measured baseline per repository before anything is changed (plan WP-00).

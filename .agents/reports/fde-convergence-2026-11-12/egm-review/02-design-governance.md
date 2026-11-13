# 02 Design critic: architecture, semantic correctness, governance (E primary; B, C, G, SOP)

V = verified in source/doc; I = inferred.

| ID | Sev | Finding |
|---|---|---|
| G-01 | BLOCKER | A3-UV claim does not close the browser-agent hole as written |
| G-02 | MAJOR | Operation classes blurred; "P3 or independently authorized" |
| G-03 | BLOCKER | No emitter authentication; an agent can forge events yielding `confirmed` |
| G-04 | MAJOR | Three-valued mapping and vacuous truth; unclassifiable behaviours |
| G-05 | MAJOR | `declared_unobserved` vs `insufficient` undefined; episode vs structural comparison blurred |
| G-06 | MAJOR | world_ref churn breaks regression, held-out sets, template pinning |
| G-07 | MAJOR | Regression rule blocks legitimate and passes harmful revisions |
| G-08 | MAJOR | Promotion/recovery has no saga; rollback semantics incomplete |
| G-09 | MAJOR | Derived lifecycle axis and Field Case "no new aggregate" do not hold |
| G-10 | MAJOR | I-1/I-2/I-3 not enforceable where stated |
| G-11 | MAJOR | Recovery table omits ledger/RealityTrace split-brain |
| G-12 | MINOR | AD-5 fallback creates a second policy evaluator |

## G-01 BLOCKER: A3-UV does not by itself close "agent drives the paired browser"
Location: E §8 last paragraph; E §10 bullet 3; E §9.
Evidence: E says item 4 "is closed for these operations by A3-UV at the SEA-Forge boundary" and routes P3 to "the Horizon decision card". The owner-selected model (unified doc, Executive recommendation) says the opposite for a page-based ceremony: "A2 on a browser page is not sufficient for a human-reserved P2/P3 decision under S1", and A3 "requires the surface to fetch and render the canonical operation directly from the trusted authority, not accept agent-authored HTML". §5.3: "A separate phone is insufficient if the same hostile desktop can also modify the local authority or the operation sent to that phone". A WebAuthn UP+UV assertion over a digest proves ceremony participation, not that the human saw the true operation. Stage-0B doc §3: the A3-UV proof is "verified by Horizon against the identity service's stored key". V.
Why it matters: if the decision card is a Horizon page in the paired browser, the agent controls the renderer and (on S1) the pending-operation text; the assertion is then A2. Verification inside Horizon also puts authority in the presenter, contradicting E §1.
Fix: state that settlement/disposition/promotion confirmation is performed only on an A3 surface (phone or isolated native) that fetches the canonical operation from SEA-Forge by `pending_id`; Horizon only initiates and shows status. SEA-Forge (or an identity-issued receipt it verifies) validates the assertion; add "human-verification evidence in approval validation" (unified §10 SEA-Forge) to Stage 2 remaining work, it is absent from E's table. State the S1 limit (no hostile same-user process) and that without an A3 channel these actions are refused. Reconcile with 2026-11-08-agent-authority-decision.md (status "open", Options A/B/C) and Stage-0B doc "Owner decision: Option B" (B = "a browser-controlling agent is trusted"): E must cite which decision is binding.

## G-02 MAJOR: operation classes blurred
Location: E §8 "every disposition ... settle ... promotion is a P3 human-reserved or independently authorized".
Evidence: unified §5.1: P2 = "answer an operator question, route a reserved decision"; P3 = external commit with "Online SEA-Forge decision"; "A3 establishes human participation... it does not grant authority". Settlement and disposition are P2-like, promotion and hard-gate waiver P3. "Independently authorized" is undefined for a human-reserved SOP action (SOP §3.2).
Fix: a table: operation, class, required evidence (A3-UV, SEA-Forge decision, both), who may satisfy it, and whether autonomous grant is permitted. Unknown class fails closed (Px).

## G-03 BLOCKER: emitter authentication and forgery of `confirmed`
Location: E §5 (event contract, ingest), E §6.
Evidence: the event carries self-asserted `declaration_key`, `outcome {status}` ("observed, never inferred"), `authority {decision_id}`, `source.record_id`. Ingest rejects only events lacking fields. RealityTrace's hash chain and `allowed_predecessors` detect rewriting order, not origin. In sxr I found no emitter signature/attestation check (grep, `sxr-core/src`; I). Cognate emits events about its own run, so the executing agent's harness is the observer of its own behaviour. Nothing in E or Stage 1-2 identity (unstarted per E §8) authenticates the emitter. Classification uses `declaration_key` as the join to the model, so whoever labels the key controls whether an occurrence confirms a policy.
Fix: (a) ingest requires an authenticated emitter principal (execution identity or service credential) recorded in `source`; (b) RealityTrace resolves `source.record_id` against the owning system and marks `source_verified`; `authority.decision_id` is verified against SEA-Forge, never trusted; (c) `confirmed` for an evaluated episode requires at least one occurrence from an emitter independent of the executor (SEA-Forge ledger, Gauntlet); executor-only evidence caps at `insufficient`; (d) declaration_key assignment by a verified mapping owned outside the executor.

## G-04 MAJOR: AD-5 classification semantics
Location: E §6 bullet "Classification is delegated to DomainForge".
Verified: `domainforge-core/src/policy/core.rs:195-225`: Unknown yields `is_satisfied=false` plus a `Violation` with modality severity; the tri-state is only in `is_satisfied_tristate`. So `domainforge validate` output (violations) cannot distinguish unknown from false unless the tristate field is exposed; with an Obligation, unknown looks like a violation. I.
Gaps: policies are predicates over a graph of entities/instances, not over sequences. Not classifiable by this path: ordering and directly-follows edges, frequency/rate, absence in a window, timing, actor-role distribution, context variation (E lists these as node/edge statistics, but §6 emits classes only via policies, so they get no class). Universal quantifiers over empty instance sets are vacuously true, giving `confirmed` with zero support; one true occurrence also "confirms" a rule that needs N. No minimum-support or quantifier record.
Fix: classify per comparison kind (element predicate, edge/order, frequency, absence); add class `not_evaluable` (never `confirmed`); require `support_count >= k` and record it; expose tristate through a CLI/API contract with a golden test; the WP-10 spike must include an empty set, a single occurrence, and an Obligation returning unknown.

## G-05 MAJOR: coverage and comparison separation
Location: E §6 bullets 2-3; E §5 `expectation_ref?`.
Evidence: "`declared_unobserved` when their emitter reported coverage and `insufficient` when it did not". "Reported coverage" has no definition: per emitter, per key, per window? Reported by the emitter itself (G-03)? A silent emitter and a covering emitter with zero events are indistinguishable.
Also `derive --scope case:<id>` has no episode or `episode.kind` filter: exploratory and evaluated occurrences mix, and structural residuals share the `residual:` prefix with episode settlement residuals. The SOP requires exploratory observation never relabelled and episode comparison against the frozen reference; structural comparison is model vs behaviour.
Fix: a coverage manifest (emitter, declaration_keys, window, heartbeat) frozen with the episode and written before the run; absence needs manifest coverage plus a closed window. Add `--episode` and `--kind evaluated` selectors, separate record kinds (`episode.comparison` vs `structure.residual`), and never feed exploratory occurrences into `confirmed`.

## G-06 MAJOR: world_ref churn
Location: E §7, §4 last bullet, §2 `declaration_key`.
Verified: `application/world.rs:17-19`: "A comment-only source edit or a DomainForge upgrade therefore mints a new `world_ref`". Consequences: (1) every comment edit or compiler upgrade is a "revision" needing promotion; the design needs a no-semantic-change fast path (semantic diff empty) with the same authority record; (2) prior occurrences carry old `world_ref` and old `declaration_key`s; `observed compare --mapping` is optional, but renames/splits (ConceptChange) change keys, so re-running prior occurrences under the candidate is undefined; (3) "held-out" is undefined: the diagnosis was derived from those same records; (4) template `fde.field_case@1` stores a `world_ref`, so every promotion invalidates the template and pinned in-flight cases and episodes with no regeneration/migration rule.
Fix: define re-anchor (old key to new key mapping, mandatory when keys change, produced by the DomainForge semantic diff); define held-out as records excluded from diagnosis by time or case before the proposal; state in-flight cases keep their pin; template regeneration is itself a governed, versioned step.

## G-07 MAJOR: regression rule
Location: E §7 step 3.
Evidence: "Any prior `confirmed` that becomes `contradicted` fails the candidate."
Blocks legitimate: a revision that deliberately changes a distinction (the point of `ConceptChange`) flips some confirmed results; no declared-intent exemption. Passes harmful: `confirmed -> insufficient` (key renamed, evidence lost), `confirmed -> observed_undeclared`, `contradicted -> confirmed` by weakening/removing a Prohibition or Obligation (every behaviour now "confirms"), `not_evaluable` appearing.
Fix: a full transition matrix; allowlisted flips bound to the ConceptChange and approved explicitly; any decrease in distinguishing power or weakened policy is a separate authority-escalated change; require non-decreasing coverage and report `confirmed` count deltas.

## G-08 MAJOR: promotion, version preservation, recovery
Location: E §7 steps 4-5, §11 rows 4-5.
Evidence: the saga (D5) in E §8 and Stage-0B covers device enrolment only. Promotion touches the registry pin, the CEP transition record, RealityTrace `proposal`/`semantic.*` status, template regeneration and Horizon projections, with no step state or idempotency key. Registry is in-memory (debt M-06, E acknowledges). Re-pin leaves residuals classified under the withdrawn world still labelled `confirmed`, and episodes frozen under it stay valid-looking.
Fix: a promotion saga keyed `(proposal_id, candidate_world_ref)` with states, idempotent steps, fail-closed uncertainty; one authoritative pin record in SEA-Forge, RealityTrace status derived from it; classifications carry the world they were computed under and are marked superseded on re-pin; durable registry is a WP prerequisite, not a footnote.

## G-09 MAJOR: AD-2 and the derived lifecycle axis
Location: E §3 table rows "Case content groups", "Case lifecycle axis".
Verified: `sea-forge-core/src/types.rs:14-20` `CaseState` = Active, AwaitingApproval, Completed, Terminated. `ParkHumanTask` (`case_ops/advance.rs:249`) is a plan-item action that appends an item-activation notice; there is no case-level parking mechanism, so "suspended = parked by disposition" requires a new fold over `disposition.recorded`, plus rules for items parked vs case suspended. `closed = Completed | Terminated` contradicts R-69/SOP §10.3 (closure only by authorized disposition): the runner moves a case to Completed when its plan finishes (I; confirm in `case_ops`). No enum change is true for `CaseState` only: new ledger event kinds (`episode.frozen`, `episode.settled`, `gate.assessed`, `disposition.recorded`, `field_case.revised`, `strategy.recorded`) extend the trace-kind set, and readers that reject unknown kinds break (I).
AD-2 coherence: SOP §2.1 "operational projection... not a new ontology primitive" is satisfied at the model level. But `field_case.json` is an authoritative SEA-Forge document with its own `status` enum and 8 SFWP verbs; operationally a new aggregate. The schema is hand-defined, not generated from the `.sea` model, so two definitions of "Field Case" can drift. Fix: say "no new ontology primitive, one new Casework document type"; generate its schema from the model; define lifecycle as a pure function with a golden table including Completed-without-disposition (= "work complete, not closed"); version the trace-kind set.

## G-10 MAJOR: invariants I-1/I-2/I-3
Location: E §3 invariants.
Evidence: I-3 "A second `episode.frozen` for the same `episode_ref` with a different digest is refused", but `episode_ref` already embeds the digest (E §2), so a different digest is a different ref; the enforceable check is on `(case, plan_item, n)`. I-1 is enforced at `agent_run.delegate` and "any governed operation tagged with an evaluated episode": the tag is caller-supplied, so an untagged call bypasses; Cognate `startRun` and Workbench sessions can run without SEA-Forge. I-2 "principal chain intersects the executor's": undefined chain; if it contains the delegating human, the operator who assigned work can never settle it, if not, an agent spawned by the same executor passes. Today separation is a string `proposed_by` check (E: "reuses"; I), and execution identity is Stage 1/2 work not done.
Fix: I-3 keyed on `(case,plan_item,n)`; I-1 enforced by SEA-Forge refusing any governed effect whose execution identity lacks a bound `episode_ref` (not caller tag); define the chain as `{executor execution_id, attempt, delegator chain}` excluding the final human approver; mark I-2 not-enforceable until Stage 2.

## G-11 MAJOR: recovery and split-brain
Location: E §11.
Gaps: (1) freeze is two writes (SEA-Forge `episode.frozen`, RealityTrace `question.declared`): if the second fails after the first, I-1 passes but the ordering proof is missing; §11 only blocks settle, not delegate. (2) `episode.settled` references a RealityTrace `settlement_ref`; orphan records either way, no authoritative side or reconciler. (3) outbox replay duplicates (idempotency key not stated), `recorded_at` skew. (4) RealityTrace chain corruption or CAS loss after settlement. (5) partial promotion (G-08). (6) emitter coverage lost mid-episode. (7) revoked identity mid-run.
Fix: freeze is a two-phase step (RT declared first, then SEA-Forge frozen citing the RT record id and hash); delegate refused unless both exist; one reconciliation job with SEA-Forge authoritative for state and RealityTrace for evidence; event idempotency key `(source.system, record_id)`.

## G-12 MINOR: AD-5 fallback violates ownership
Location: E §6 last paragraph vs E §1 row 1 ("no other repo parses `.sea`" / owns policy semantics).
Fallback makes `sxr-observed` evaluate policies from AST JSON, a second evaluator that will diverge on null semantics (G-04). Fix: time-box the spike; if it fails, block WP-12 on a DomainForge change, or have DomainForge ship a library/CLI `evaluate --instances` rather than copy semantics.

## Right, keep
- Ownership table (E §1): one writer per concern, presenters hold projections only.
- No new ID grammar beyond `episode_ref`; SHA-256 over RFC 8785 JSON named in CEP-0008.
- Deterministic `observed derive` with input-set hash and occurrence ids on every aggregate; classes `observed_undeclared` and `insufficient` rather than forcing a verdict.
- Fail-closed recovery and "skipped gate is NO-GO" (E §11, §12); production refusal until D1/D2.
- Horizon parity rule: answers embed record ids and digests and are byte-compared with CLI output.

## Checked, no defect found
AD-3 (model in existing grammar, translation map with status), AD-8 boundary (Workbench shows read-only strip, no P3 approval), I-1 ordering concept itself (hash chain makes rewriting detectable).

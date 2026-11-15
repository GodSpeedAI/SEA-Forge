# 05 Zoom-out review (scope, simplicity, cost, operational viability)

Reviewer: Zoom Out Goldfish. Read in full: B, E, F, G, STATUS, Mission.md. A, C, D only grepped (C row sample). Spot checks in source: godspeed-readiness/profiles.yaml, stack.yaml, cognate/examples, sea-rs/crates.

## Findings (ranked)

**ZO-01 BLOCKER: proof scenario is circular and non-deterministic.**
Location: G §1-2 vs F WP-22/WP-51.
Evidence: G §1 "FC-001 'Finish the Horizon enrolment authority chain' ... episodes are real engineering assignments (WP-22 parts: D3 route-scope, D5 saga)". F: WP-51 (run the case) has prerequisite "P2-P6", and P6 WP-31 needs WP-22. So the proof's episodes E1-E3 are the work of a package that must already be done before the proof runs. Also E2 ("scenario input directs the executor to begin work before the reference is frozen") and E3 ("executor inserts a fresh-context review") depend on LLM agent behaviour. Mission §1 requires "deterministic, reproducible inputs".
Verified: the Mission asked for "the smallest existing executable workflow", and `cognate/examples/refund-desk` plus `examples/realitytrace` exist. G §1 explicitly supersedes this.
Why it matters: the integrated gate can hang on its own prerequisites, and an agent may simply not produce E3/E4 behaviour (F §4 concedes "NOT TESTED").
Fix: run the first proof on Refund Desk (or a scripted deterministic executor) with all four kinds produced by fixture-driven runs. Keep "GodSpeed runs its own SOP" as a later dogfood case, not the gating proof.

**ZO-02 MAJOR: no thin vertical slice; everything gates on all P2-P6 plus the full identity programme.**
Location: F §1 graph; WP-51 prereq "P2-P6".
Evidence: Mission §1-7 needs: model, frozen expectation, exhaust, derive, compare, revise, authorise, subsequent episode, Horizon read. Mission §6 says "Do not make proof completion depend on finishing unrelated Horizon product features". Yet WP-20/21/22 (identity stages 1-2, A3-UV, enrolment saga, crash injection), WP-23, WP-40, WP-41, WP-60 precede or block WP-70.
Inference: identity A3-UV is needed only for NT-12 and the promotion approval (Mission §5.7 needs "independent authorization", not WebAuthn).
Fix, slice 1 (thin): DomainForge model of an existing small workflow, `interaction_event/1` emitted from one runtime, `sxr-observed` derive+compare (WP-10/11/12), existing Stage 10 approval-gated world transition with proposer != approver (already exists per E §7.4), subsequent episode, Horizon read adapters (WP-30), one script plus bundle. Defer: the 8 new SFWP verbs, identity stages 1-5, Gauntlet and CK bridges, tax transfer, ten journeys, readiness profile.

**ZO-03 MAJOR: eight new SFWP verbs, five ledger events and a `field_case.json` aggregate are heavier than needed.**
Location: E §3.
Evidence: "New SFWP verbs: case.field.get, field.revise, episode.freeze, episode.settle, gate.assess, dispose, strategy.record, continuation. Each ... golden fixtures, mirrored in the Go client and generated TypeScript contracts."
Why: each verb is mirrored in Rust, Go and TS (three copies). Mission needs only freeze (already `question.declared` in RealityTrace), settle (existing settlement) and promotion (existing). The SOP-only items (gates, strategy, disposition, field_case) are not on the Mission path.
Fix: one typed `case.record{kind, payload}` append with a schema registry per kind (`episode.frozen`, `episode.settled` ...) and one read verb. Invariants I-1/I-2/I-3 stay as server-side checks keyed on `kind`. Mission-path kinds first: `episode.frozen`, `episode.settled`. Add the rest when journeys need them.

**ZO-04 MAJOR: operations are not designed.**
Location: absent from E/F/G (grep for retention, observability, launch, compose, existing cases: no hits).
Missing: (a) how the 12-repo stack starts as one system and where each service runs (only `stack.yaml` pins and the proof script); (b) configuration and port map; (c) migration of existing Casework cases with no `field_case`, and of ledgers when `world_ref`-bearing events appear; (d) monitoring of the integration itself (outbox depth, ingest.rejected rate, unanchored share, projection drift) - E §11 lists recovery but nothing surfaces it; (e) retention and backup of new stores (observed representations in CAS, outbox, world registry). Verified: `local_operator` already has O02-O04 backup/restore and U02a-g upgrade gates; the plan does not say the new stores join them.
Fix: add one WP: "operate" with launch command, config reference, backup/restore and upgrade gates for the new stores added to the existing profile, plus integration health panel on Cost or Home.

**ZO-05 MAJOR: new readiness profile `fde_field_case` forks an existing mechanism.**
Location: E §12, WP-70.
Verified: `godspeed-readiness/profiles.yaml` `local_operator` already carries J02, J02R2, SW01, SW02, CK01/02, G01, O01-O04, U02*. `gsr-heads` pinning exists.
Fix: add gates FDE01..n to `local_operator` (or one new profile inheriting it). State explicitly what happens to `local_operator` GO: it must stay GO or the release is blocked. As written, a second profile can diverge from the first.

**ZO-06 MAJOR: Gauntlet bridge, Context Kernel bundle, tax transfer, ten journeys are not Mission-required.**
Location: WP-40, WP-41, WP-60, B §3, G AT-17/AT-19.
Evidence: Mission §1-7 never names Gauntlet or Context Kernel. Existing gates G01, SW02, CK01 already prove those boundaries. Tax transfer (AT-16) answers SOP transfer, not Mission; G §7 itself says one case does not establish transfer.
Fix: defer WP-40/41 (keep existing verification as optional observations). Keep WP-60 only after slice 1 is released. Cut journeys to J1, J3, J5, J6, J7 (the four named in B §7 plus revision); the other five can ship as follow-on. Playwright for ten journeys plus agent-browser plus walkthrough is large.

**ZO-07 MAJOR: new `sxr-observed` crate is probably justified, but its dependency on a DomainForge instance overlay is the highest-uncertainty piece and is not decoupled.**
Location: E §6, WP-10.
Evidence: STATUS: "DomainForge: no observed-vs-declared facility, `domainforge test` is a stub". Overlay via generated `instance ... of ...` facts is a new use of the compiler.
Credit: WP-10 is correctly placed first and has path B. But path B (policy evaluation over AST JSON inside `sxr-observed`) duplicates DomainForge policy semantics, violating E §1 "no other repo parses .sea". Also WP-10 only prereqs WP-02 yet E expects the full SOP model; the spike should use the thin fixture model.
Fix: spike on a 10-element model, decide A/B before WP-02 is finalised; if B, restrict classifier to the minimal policy subset and file the gap.

**ZO-08 MAJOR: durable world registry and identity fixes are scoped as part of feature WPs; integration risk is high and drift-prone.**
Location: E §7 (M-06), WP-04, §2 AD-1.
Cost: touched repos: all 12 (WP-00 alone edits 10, including merging PRs in repos the owner has open). Contracts: `interaction_event/1`, SFWP verbs x8, CEP-0008 amendment, identity package, `episode_ref`, `context_bundle`, `VerificationRecord`. "Contract gates (identical across repos)" mirror rather than share: the schema is vendored into every consumer and compared by digest (WP-01). That fixes drift detection, not multiplication: copies grow with each new contract.
Most likely drift: `interaction_event/1` (Rust emitter in SEA-Forge, TS in Cognate/Workbench, ingest in RealityTrace) and `correlationId = case:<id>/assign:<id>` string convention parsed in several places.
Fix: generate emitters/validators from the CEP schema (one source) and define the correlation id as a structured field, not a parsed string.

**ZO-09 MAJOR: realistic risk the integrated gate never goes green is high; plan front-loads the wrong risks.**
Location: F §3, §4; WP-00.
Evidence: WP-00a-e is a large merge-and-triage phase (landing 5-commit branches, choosing among duplicate branches, rebasing, owner's dirty files). Host OOM constraints force serial gates. Biggest red-gate sources are not in F §4: owner-held PRs (#91, #71, #7, #76) merged into `fde/integration`, WP-70 requiring every repo released from clean artifacts, and a real authenticator for A3-UV (F §4 admits those rows become BLOCKED, i.e. not GO under E §12 "skip = NO-GO").
WP-10 is the right first spike, but there is no equivalent early probe for: DomainForge instance-overlay (yes), A3-UV on the proof host (no, deferred to WP-22), and a clean-cell install (deferred to WP-70).
Fix: add WP-10b "walking skeleton": one thin proof run on a clean cell by week one, even with stubs labelled as such, so install/launch/authority risks surface early. Decide now whether the gate may use Chromium virtual authenticator.

**ZO-10 MINOR: Horizon/Workbench split is acceptable but cognitive-load cost is understated.**
Location: B §5-6, AD-8.
Evidence: two UIs, two design systems, one identity, plus two SEA-Forge consoles stated to be "outside the FDE journey" (B §6).
Judgement: keeping both is defensible because Workbench exists and is where the terminal and spend live. Leaving two SEA-Forge UIs live is tenable only if documented as admin-only and unlinked from the operator path; otherwise operators land on them via approvals and denials. Fix: one "where do I go" table in the operator docs, and the case strip as the only cross-UI affordance; defer "read-only case strip in Workbench" (WP-32) until J4 is proven.

**ZO-11 MINOR: does the plan stop at a demo?**
Verdict: it honours "released, reproducible, operational" on paper (WP-70: release, clean-cell install, GO report), but ops of the running system (ZO-04) and `local_operator` continuity (ZO-05) are the gap. Claims are properly labelled PARTIAL (G §7).

## Things the design gets right (keep)
1. Pre-registration (WP-50) and mutation recording for every negative test (G §4).
2. Ownership table with "sole writer" per concern, and projections rebuildable (E §1, §11).
3. WP-10 spike with a stated fallback, placed in the first batch.
4. Additive, versioned contract changes and fail-closed recovery table (E §11, F §0).
5. Honest separation of what one passing case does and does not establish (G §7); AD-1 closing the undefined hash algorithm.

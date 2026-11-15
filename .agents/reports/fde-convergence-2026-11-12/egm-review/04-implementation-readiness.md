# 04 Implementation readiness (Goldfish review)

Scope: F (plan) with E (contracts) and G (tests). Read F, E, G in full; spot-checked source (branch/worktree state, sfwp fixtures path, scripts). VERIFIED = seen in source; INFER = reasoned.

## Findings (ranked)

**IR-01 BLOCKER. Field Case FC-001 is not executable with the actors that exist.** G §1/§2, F risk table row 6, E §8.
- E §3 I-2 requires settler principal chain to be disjoint from the executor's; E §7 requires proposer != approver; NT-12 requires an A3-UV assertion for dispositions/promotion. G names no principals. The only actors are one human owner plus AI agents.
- The plan never says who the executor, verifier and approver are, how a second agent principal is minted in the Cognate identity contract (WP-20 is not built until P5), or where an authenticator comes from. F §4 admits a missing authenticator makes the human-reserved rows "BLOCKED", but those rows (disposition, settle, promotion) are on the critical path of AT-09, AT-12, AT-13 and AT-15. A Chromium virtual authenticator is "labelled as such" yet the claim ledger has no rule for how that affects PROVEN.
- Fix: add a principal table to G (executor = agent principal X session; verifier = distinct agent principal Y on a different model CLI; approver = the owner's real passkey, with owner presence scheduled at named steps). State which AT/NT rows may be PROVEN under a virtual authenticator and which need the real one. Add a WP that provisions the principals before WP-50.

**IR-02 BLOCKER. E2 ("scenario input directs the executor to begin work before the reference is frozen") has no real execution path.** G §2.
- I-1 is enforced only on `agent_run.delegate` and "governed operation tagged with an evaluated episode". Nothing says how an agent, running in Workbench/Cognate, attempts the move: a scripted delegate call? an instruction in the brief that the agent follows? If an agent is merely told to "begin early", it can decline, so the contradicted class depends on LLM compliance, not on the system. A scripted call proves the refusal, but then the "actual execution" is synthetic.
- Fix: specify E2 as a deterministic harness step (a named SFWP call sequence issued by a test principal) AND record in G that this is a provoked attempt, not natural behaviour. Say which component writes the refusal as an occurrence (SEA-Forge emitter, WP-04) and the declaration_key it anchors to.

**IR-03 BLOCKER. Contracts are named but not concrete; an implementer must invent them.** E §3, §5, §6; F WP-01, WP-03, WP-11.
- `interaction_event/1` (E §5) is a field list with no types, no required/optional marking beyond `?`, no enum for `outcome.status`, no format for `actor.principal`, `recorded_at`, no `profile` identifier/envelope version, no mapping onto the CEP-0008 envelope fields. "covered fields listed" in the CEP amendment is a promise, not a list.
- `field_case.json` (E §3): only "every field carries status and a source ref". No schema, no list of the SOP §4.2 groups as keys, no ref format, no `field_case.revised` payload (full document vs patch), no concurrency/versioning rule for `case.field.revise`.
- Eight new SFWP verbs: names only. No request/response fields, no error codes, no idempotency rule, no which Rust module vs Go adapter. Plan cites `sfwp/testdata/frames.json`; VERIFIED real path is `apps/godspeed-casework-go/internal/adapters/sfwp/testdata/frames.json`, and `conformance_field_case.rs` has no stated crate.
- `episode_ref` (E §2) digest input is "canonical frozen reference" but the frozen reference's field set is never defined (it is only described in G/SOP prose), and `@sha256:` appears inside a string that will be the hash input of nothing else. Circularity risk: reference includes `episode_ref`? Unspecified.
- `observed derive` output: described in prose (nodes, edges, "uncertainty note") with no schema; `representation.recorded` record kind is new but not defined against RealityTrace's record-kind registry and `allowed_predecessors`. `residual.computed` payload fields are listed loosely (rule, occurrences, elements, context) without types or reliability dimension encoding. `--mapping <file>` format undefined. Template format for `fde.field_case@1` undefined (E §3 says "generated from the model" but no generator, input or output format; no `.sea` construct for gates is shown).
- A3-UV evidence at the SEA-Forge boundary: WP-21 says "signed `decision_id`", "A3-UV evidence required" but no format of the proof (WebAuthn assertion fields, challenge = operation digest? digest of what?), no key distribution, no verifier library.
- Fix: add a contracts annex (E2) with JSON Schema or Rust/TS types for each, plus verb request/response/error tables. Make these the output of WP-01 and gate everything else on them.

**IR-04 MAJOR. WP-00b/00c branch landing is not safe or specific enough.** F WP-00b..d.
- VERIFIED: `feat/horizon-desk-support`, `fix/server-ambiguous-declare-keeps-lease` are checked out in the owner's worktrees (`~/projects/sea-rs-horizon`, `~/projects/sea-rs-fix`); in Horizon-gaps and Cognate, several target branches are also worktree-bound (`+`). `fde/integration` creation by merge/rebase is not forbidden, but "rebased" `horizon/shared-identity` rewrites the owner's branch history unless done on a copy, and the owner's global rules forbid rebase/merge without intent. F §0 says "never work in the owner's dirty checkouts" yet the integration branch must read from those branches.
- "pick one of the duplicate branches after content diff": no criteria; "if their content is not already on main": no method (`git cherry`/patch-id) named; "port anything missing" (WP-00c) is open-ended; "commit or drop the 37 PNG re-captures" contradicts "never discard" in the same cell.
- Fix: P0 must say: operate only in a new worktree under `_wt/fde-*` by `git worktree add -b`; cherry-pick or `merge --no-ff` copies, never rebase the original refs; record `git rev-parse` of every source tip first (backup tags `fde-baseline/<branch>`); duplicate-pick rule = patch-id equivalence then newer tip, owner decides on ties; PNGs always go on an evidence branch.

**IR-05 MAJOR. Dependency and ordering defects.**
- WP-03 acceptance lists AT-10/AT-11 whose method needs Horizon J3, J5, J8 (WP-31, P6). Acceptance depends on later work; split into "conformance part" (WP-03) and "journey part" (WP-31).
- WP-03 acceptance AT-04 needs `question.declared`/`threshold.declared` ordering in RealityTrace, but RealityTrace emission of `question.declared` at freeze is not assigned to any package (WP-04 emits events; nobody writes the frozen reference to RealityTrace). Gap: who calls RealityTrace at freeze, and in WP-03 or WP-04?
- WP-13 prereq (12, 04) omits WP-21 (world-transition approval with A3-UV) and WP-03 (strategy/disposition). AT-12 cannot pass without them.
- WP-04 requires "registry durable (M-06)" and "remove constant-hash fallback"; both are SEA-Forge changes shared with WP-21 (see IR-06).
- WP-02 prereq is 00b only but the model needs `domainforge` pinned (WP-00e). WP-10 prereq 02 is minimal but WP-10 is listed in the first batch with WP-02 "in parallel"; contradiction (§3 vs table).
- WP-40/41 prereqs (11, 06) omit WP-03 (episode_ref must exist). WP-50 is declared in P7 yet G §2 says scenario inputs are fixed files and AT-04 order test must exist; pre-registration happens only after P2-P6, but its content is needed by WP-03/04 tests.

**IR-06 MAJOR. The "P5 parallel with P1-P4" claim is false for shared files.**
- WP-21 (SEA-Forge authority and reserved-decision checks for dispositions/settlement/waivers/promotion) and WP-03/WP-04/WP-13 (new verbs, ledger events, world registry) edit the same SEA-Forge crates and conformance suites, and WP-03's verbs `case.dispose` and `case.episode.settle` are the exact operations WP-21 makes reserved. WP-00b also, WP-21 prereq is only 00b. INFER from F: merge conflicts and an unspecified interface between "reserved decision check" and the new verbs.
- WP-22 (Horizon) and WP-30/31 (Horizon) share Horizon; WP-20 (Cognate identity) and WP-05 (Cognate events) share the Cognate integration branch; WP-23 and WP-06 both touch Workbench. Fix: state per repo a single serial lane, or add the interface (reserved-operation list and decision API) to the contracts annex so both lanes build against it.

**IR-07 MAJOR. Several packages lack a runnable acceptance command and a real recovery path.**
- Acceptance columns for WP-02..WP-13 name AT/NT IDs, not commands. G gives methods ("script over translation map", "ledger and RealityTrace order check") but no script path, arguments or location; the only named command is `scripts/fde-field-case-proof.sh` (WP-53, created last, though it is also the AT-18 command). VERIFIED: `scripts/` has no fde scripts yet.
- Evidence paths are inconsistent: `evidence/baseline/` (relative to what repo?), `.agents/evidence/fde-field-case/<run-id>/`, `.agents/preregistrations/`.
- Failure/recovery cells are phrases: "—" (WP-12, WP-23 partial, WP-32, WP-52), "additive, no consumer breaks", "a failing step stops the run; evidence kept" (WP-51: what is the re-run procedure, are ledgers reset, is the pre-registration consumed on first run?). Only WP-10 has a true fallback.
- Fix: one acceptance command per WP in a table (exact `cargo test -p ... --test ...`, `bun test ...`), plus evidence root declared once.

**IR-08 MAJOR. WP-10 spike is mis-sized and mis-timed; AD-5 can fail late.** E §6, F WP-10, F §4.
- The spike validates only overlay-plus-`domainforge validate`. VERIFIED claim to check: `domainforge test` is a stub; E notes it. Whether `instance ... of ...` facts exist in the grammar and three-valued policy results are exposed in JSON is the whole of AD-5, and the fallback (a policy evaluator over AST JSON inside `sxr-observed`) is effectively reimplementing DomainForge policy semantics, violating the ownership rule in E §1. Fallback cost is not estimated and it is not a one-WP fix.
- Fix: run WP-10 first (before WP-02), define pass/fail thresholds (which policy shapes, which result states), and if path B is chosen escalate to the owner as an ownership-boundary decision, not a recovery.

**IR-09 MAJOR. Scope is not realistic as one plan; the critical path is long and serial.**
- 9 repos, ~35 packages, three of which (WP-03, WP-22, WP-51) each hide several packages. Critical path: 00a -> 00b/00e -> 01 -> 02 -> 03 -> 04 -> 11 -> 12 -> 13 (needs 21/22) -> 50 -> 51 -> 52 -> 53 -> 54 -> 60 -> 70, with a serial heavy-build constraint (F §0: one heavy job at a time, 2.5-3.5 GB). Parallelism claims in F §1/§3 contradict that limit.
- Split: WP-03 into (a) schemas/verbs/ledger events, (b) invariants I-1..I-3 and hard-gate rule, (c) template generator, (d) Go/TS regeneration and golden frames. WP-22 into D3, D5 saga, P3Authority client, bootstrap, A3-UV prompt, PR extraction. WP-51 into execution of E1-E5 as separate runs (E5 requires promotion, E2/E3 need design) rather than one all-or-nothing run. Merge WP-52 mutation checks into each owner WP (the plan already requires a recorded mutation in "Done means"). WP-00e is a catch-all of seven repos; split by repo.
- Riskiest three: **WP-03** (new verbs and invariants with no schema, in the ledger every consumer depends on); **WP-11/12 with WP-10** (AD-5 dependency on DomainForge; new crate, new record kind); **WP-22/21 plus FC-001 human approvals** (A3-UV, saga crash injection, no authenticator, single human).

**IR-10 MAJOR. FC-001 is circular: the Field Case implements work (WP-22 D3 and D5) the plan itself needs first.** G §1, F WP-22/WP-51.
- WP-51 needs WP-22 and P6 complete; the case's episodes ARE WP-22 parts (D3 route-scope, D5 saga). Either FC-001's real work happens before the evidence mechanism exists (then episodes cannot be observed) or WP-22 is built twice. Also E2/E3 are "designed to exercise" a behavior: a pre-designed scenario that is also real engineering is both.
- Fix: choose a bounded real task that is not on the plan's own critical path (e.g. a later hardening item), or state that WP-22 is executed under the Field Case itself and drop the P6 dependence for it.

**IR-11 MINOR. Smaller forced decisions.**
- WP-05 `correlationId` `case:<id>/assign:<id>` is stated in E §2 but char-set/escape rules and max length of case_id/assignment_id are not.
- WP-20 package rename to `@godspeedai/cognate-identity`: publish target, registry, version and whether Workbench pins it by path or registry are unstated.
- `undeclared:<sha256 of operation+resource kind>`: input encoding unspecified (separator, case, normalization).
- Episode `n` allocation (who increments) and `episode.kind` immutability (NT-13) path are unspecified.
- WP-70 "install from release artifacts on a clean cell": what the cell is, how provisioned without Docker (owner rule), which artifacts.

**IR-12 MINOR. Plan relies on process-level promises ("Sonnet implementers", "hardening review") with no brief format.** Brief per WP (files, functions, test names) is absent; F says "executable by a fresh implementation agent" but gives only change summaries. Add a per-WP file-level change list for P1-P3.

## Things the design gets right (keep)
1. One branch per WP, integration branch, worktree isolation, "skipped prerequisite is not green", recorded mutation per negative test.
2. Additive, versioned contracts with consumers able to stay on the previous version; digest-compared vendored schemas.
3. Pre-registration before evaluated runs, with git/ledger verifiability.
4. WP-10 spike placed early with a named fallback, WP-00a measuring baselines before any change.
5. NT table: every negative test names the protected property and the mutation that must turn it red.

## Checked, no problem found
- No dependency cycle between packages at the graph level (only acceptance-before-prereq defects in IR-05).
- Host-constraint rules (pg0, `flock -o`, memory floor) are consistent with the owner's recorded rules.

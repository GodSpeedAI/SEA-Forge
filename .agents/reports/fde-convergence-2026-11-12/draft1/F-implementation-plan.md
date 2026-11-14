# F. Dependency-ordered implementation plan

Executable by a fresh implementation agent with deliverables A–E and G. Requirement IDs (`R-`) are in C; tests
(`AT-`, `NT-`) in G; decisions (`AD-`) in E.

## 0. Operating rules for every work package

- **Branching:** one branch per work package per repo, named `fde/<wp>-<slug>`, cut from the repo's integration
  branch `fde/integration` (created in WP-00 from fetched `origin/main` plus landed unfinished work). Use a
  dedicated worktree under `~/projects/_wt/`. Never work in the owner's dirty checkouts.
- **Agents:** Sonnet implementers on tightly briefed packages; Haiku for recon and mechanical edits; the lead
  model writes briefs, reviews diffs and decides. No Opus or Fable subagents.
- **Host limits:** one heavy build or test at a time; check free memory first (≥ 2.5 GB, ≥ 3.5 GB for full-stack
  gates); Postgres through pg0, never Docker; `flock -o` around cargo.
- **Done means:** the package's acceptance command exits 0 on the integration branch, its negative test fails
  when the protected line is reverted (mutation check recorded), the repo's own gates pass, and the evidence
  path exists. A prerequisite that was skipped is not green.
- **Review:** every package touching identity, authority, sandbox, IPC or settlement gets
  `sea-forge-hardening-review` (independent model) before merge. Architectural changes get an EGM review.
- **Merge:** PR per package into `fde/integration`; integration branches merge to `main` per repo only when the
  integrated gate (WP-70) is GO for the set. Unmerged branches are never deleted without a content check.
- **Recovery, generic:** a failed package leaves its branch and a note in `STATUS.md`; nothing downstream starts.
  Contract changes are additive and versioned so a consumer can stay on the previous version.
- **Escalate to the owner:** changes to frozen specifications or identifier grammars, anything touching
  credentials, an irreversible external effect, or a decision the repos record as the owner's.

## 1. Phases and dependency graph

```
P0 baseline ──► P1 contracts+model ──► P2 case+exhaust ──► P3 observe+compare ──► P4 revision
     │                                      │                                         │
     └────► P5 identity+authority ──────────┴──────────────► P6 Horizon+Workbench ◄───┘
                                                                     │
                                         P7 proof (pre-register → run → verify) ──► P8 transfer ──► P9 release
```

P5 runs in parallel with P1–P4 (different files and owners). P6 needs P2–P5 interfaces, and can start against
golden frames once WP-03 and WP-12 publish them.

## 2. Work packages

### P0 Baseline

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-00a | all | — | `git fetch` every repo; record heads, dirty state, toolchain versions; run each repo's gate once on current `origin/main`, serially; write measured baseline | baseline file lists pass/fail/skip per repo with commands and exit codes | `evidence/baseline/` | a red baseline is recorded, not fixed here; it becomes a prerequisite fix in the owning WP |
| WP-00b | SEA-Forge | 00a | create `fde/integration`; land `feat/horizon-desk-support` (5 commits); pick one of the duplicate settlement-claim branches and one of the case-task-approval branches after content diff; land `fix/settlement-orphan-guard`, `test/settlement-class-fail-closed`, `fix/server-ambiguous-declare-keeps-lease` if their content is not already on main; list every other branch as merged / superseded / kept | conformance and kernel suites green on `fde/integration`; branch ledger complete | `evidence/baseline/sea-forge-branches.md` | conflicting branch stays unmerged and is reported |
| WP-00c | Horizon | 00a | create `fde/integration` from `horizon-desk-gaps` (the superset); diff `stage0b-containment-core`'s 3 wip commits and dirty files against it and port anything missing (notably the enforcement-probe test); refresh stale status docs | `bun test`, `tsc`, teeth suite green; no content from other branches lost (diff check recorded) | branch ledger | keep `pr1`, `pr1-narrow`, `core` untouched |
| WP-00d | Cognate, Workbench | 00a | Cognate: `fde/integration` = main + `feat/realitytrace-real-snapshot` + `horizon/shared-identity` (rebased). Workbench: `fde/integration` = origin desktop line + the 7 unique `feat/horizon-connect` commits, on Cognate 0.9.x; commit or drop the 37 PNG evidence re-captures as one batch (owner's files: commit on a separate evidence branch, never discard) | both suites green; Workbench builds against the Cognate integration branch | branch ledger | version skew that cannot be resolved blocks P2 and is escalated |
| WP-00e | RealityTrace, CEP, Context Kernel, Gauntlet, SWE_SEED, GodSpeed-Agent, DomainForge | 00a | RealityTrace: rebase onto main (agent-event ingest #13). CEP: merge PR #3. Context Kernel: pin = main. Gauntlet: merge PR #7 line. GodSpeed-Agent: land `migration/cep-world-required-evidence`. DomainForge: remove the release-path `println!` at `parser/ast.rs:1283`; add to `gsr-heads` | each repo's gate green; `gsr-heads` has ten entries; `scripts/e2e-world-loop.sh` runs to OK and its output is recorded for the first time | `evidence/baseline/world-loop.log` | the world loop failing is a P0 defect to fix before P2 |

### P1 Contracts and declared model

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-01 | CEP (schema), all consumers | 00e | `godspeed.interaction_event/1` profile schema with valid and invalid fixtures; CEP-0008 amended: SHA-256 over RFC 8785 canonical JSON, covered fields listed; conformance gate comparing the vendored copy in each repo to the CEP source by digest | `cep validate all` green; contract gate fails when any vendored copy differs | fixtures; gate log | additive profile; no consumer breaks |
| WP-02 | SEA-Forge (docs, `.sea`) | 00b | promote SOP v0.4 to `docs/sop/`; author `.sea/interaction/fde-sop/` per AD-3 with translation map (one row per numbered SOP clause), assumptions-and-unknowns, validation output; record source hash, `world_ref`, compiler version | AT-01, AT-02 | model, validation output, hashes | a clause that cannot be expressed is recorded `unknown` with the grammar limitation, not approximated |

### P2 Field Case and semantic exhaust

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-03 | SEA-Forge | 01, 02 | AD-2: template `fde.field_case@1` generated from the model; `field_case.json` schema and `field_case.revised`; events `episode.frozen`, `episode.settled`, `gate.assessed`, `disposition.recorded`, `strategy.recorded`; verbs `case.field.get/revise`, `case.episode.freeze/settle`, `case.gate.assess`, `case.dispose`, `case.strategy.record`, `case.continuation`; invariants I-1, I-2, I-3; hard-gate rule; Go client and TypeScript contracts regenerated; golden frames | AT-03, AT-04, AT-10, AT-11, AT-15; NT-01, NT-10, NT-11, NT-13 | conformance tests `conformance_field_case.rs`; frames | new verbs are additive; existing case verbs untouched (regression suite) |
| WP-04 | SEA-Forge | 01, 03 | emit `interaction_event/1` for case ledger events and authority decisions through a durable outbox to RealityTrace; make the world registry durable (debt M-06); remove the constant-hash identity fallback (fail closed) | AT-05 (SEA-Forge part); NT-15; registry survives restart | ledger + RealityTrace records | emitter failure never blocks the case write; outbox retries; settlement waits |
| WP-05 | Cognate | 01, 00d | run and invocation events carry `world_ref`, `declaration_key`, case context, authority decision id; outbox uses the profile; `correlationId` convention `case:<id>/assign:<id>`; cost and interruption events | AT-05 (Cognate part), AT-17 | event log, RealityTrace records | field additions are optional for non-case runs |
| WP-06 | Workbench | 05, 03 | assignment binding (session opened from an assignment carries `assignment_id`, `episode_ref`, brief, budget); spend recorded per currency against the assignment; observations use the profile | AT-17 | J-tests | Workbench without a case behaves as today |

### P3 Observe and compare

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-10 | RealityTrace (+DomainForge read-only) | 02 | spike: ten-occurrence fixture, generated instances overlay, `domainforge validate`, confirm three-valued results per policy are obtainable in JSON | written result: path A (DomainForge evaluates) or path B (classifier over AST JSON) chosen, with the fixture | spike report | path B is the recovery; file the DomainForge gap |
| WP-11 | RealityTrace | 01, 10 | crate `sxr-observed`; `observed derive`; profile ingest on the agent-event path; coverage reporting; deterministic canonical output in CAS | AT-06, AT-08; NT-02, NT-03 | representation records, hashes | derive is read-only over the ledger; no existing record kind changes |
| WP-12 | RealityTrace | 11 | `observed compare`; five classes; `residual.computed` per difference with rule, occurrences, elements, context; reliability dimensions where thresholds exist | AT-07, AT-09; NT-04, NT-06 | residual records | — |

### P4 Governed revision

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-13 | RealityTrace, SEA-Forge, DomainForge (use only) | 12, 04 | diagnosis and proposal flow on existing record kinds; regression command (`observed compare` under candidate over held-out occurrences); SEA-Forge world transition `semantic_change` with proposer ≠ approver, CEP world-transition record, promotion and re-pin | AT-12; NT-05, NT-07, NT-08 | transition record, registry state | re-pin previous world through the same gate |

### P5 Identity and authority (parallel)

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-20 | Cognate | 00d | identity package to `@godspeedai/cognate-identity`; D2: `enrolDevice` / `replaceDevices` verify `{authorization_decision_id, approval_digest, a3uv_proof}` and issue `pending_activation`; session scope claims (D3 identity side); operator recording for bootstrap/recovery (D4/D6 identity side) | identity refuses writes with no evidence, wrong digest, missing decision, replayed proof; pending credential cannot sign in | identity tests | package is unconsumed today, so no rollback risk |
| WP-21 | SEA-Forge | 00b | D1: device-enrolment decision API bound to operation digest and expiry, signed `decision_id`; D4/D6: `bootstrap_controller` and `recovery_controller` policy classes, single-use, expiring, audited; reserved-decision check for dispositions, settlement, hard-gate waivers and world promotion (A3-UV evidence required) | contract test with a real signed decision; refusal on denied, unavailable, expired, digest mismatch | conformance tests | off by default until Horizon wires it |
| WP-22 | Horizon | 00c, 20, 21 | D3 route-scope table enforced before dispatch; D5 saga states and reconciliation job with crash injection; real `P3Authority` client replacing the test double in production wiring; first-controller bootstrap on the local console; A3-UV prompt on reserved desk decisions; extract PR-1 (containment) and PR-2 (enrolment) | NE-73 retired only with a real client; crash after any step leaves no usable credential; NT-12 | Horizon tests; hardening review | production keeps refusing until WP-20 and WP-21 pass |
| WP-23 | Workbench, Gauntlet | 20 | Workbench consumes the identity package (delete duplicate WebAuthn/CBOR); Gauntlet binding record and attempt ledger before launch (stage 3); legacy runs `legacy/unbound` | existing remote and pairing tests green on the shared package; Gauntlet launch refused without binding | tests | one writer cut over at a time |

### P6 Operator experience

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-30 | Horizon | 03, 12 (golden frames suffice to start) | adapters `field-case.ts`, `realitytrace-structure.ts`; `DebuggerSources` fed from them; `reach` over the seven Aim dimensions; answers embed record ids and digests | AT-14; NT-09 | parity test output | adapters are read-only; absent source renders `not connected` |
| WP-31 | Horizon | 30, 22 | journeys J1–J10 in the existing views; case strip; inspector; state set (B §4); Reassign control and request ids for route/reassign/withdraw/remove (gaps A, B) | AT-19 with Playwright against live services, plus agent-browser pass | screenshots, traces | each journey ships independently behind the case template check |
| WP-32 | Workbench | 06 | case strip; "awaiting decision at the desk"; back-link; read-only trajectory after completion | AT-19 (seam) | J-tests | — |

### P6b Verification bridges

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-40 | Gauntlet, SWE_SEED, RealityTrace | 11 | Gauntlet `VerificationRecord` and SWE_SEED `ProofCompleted` exported as profile observations bound to the `episode_ref`; Gauntlet frozen Reference digest equals the episode reference digest | AT-17 (verification part) | records | a missing verifier leaves the episode `U`, never `S` |
| WP-41 | Context Kernel, Workbench | 06 | `context_bundle` (bound to `world_ref`) attached to the assignment brief; cited, labelled `untrusted_reference` in Horizon | AT-17 | bundle in evidence | absent Context Kernel: brief records "no retrieved context" |

### P7 Proof

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-50 | SEA-Forge | 02 | pre-register acceptance tests, falsifiers and scenario inputs (G §2–4) under `.agents/preregistrations/`, hash-pinned and committed before any evaluated run | commit precedes first evaluated episode (verifiable from git and the ledger) | preregistration + hash | changing a criterion later requires a new version and re-run |
| WP-51 | all | P2–P6 | run the first Field Case (G §3): open, strategy, frozen episodes over the four scenario kinds, settle, dispose, derive, compare, diagnose, propose, regress, authorize, promote, subsequent episode | AT-03 to AT-17 green on real services | bundle sections 2–3 | a failing step stops the run; evidence kept |
| WP-52 | all | 51 | negative suite and mutation checks | NT-01 to NT-15, each with a recorded mutation that makes it fail | bundle section 4 | — |
| WP-53 | SEA-Forge | 51, 52 | evidence bundle, claim ledger, single command `scripts/fde-field-case-proof.sh`; update owner proof ledgers (DomainForge `PROOFS.md` and equivalents) | AT-18 | `.agents/evidence/fde-field-case/<run-id>/` | — |
| WP-54 | independent | 53 | fresh verifier (new context, no access to implementer narration; different model CLI where available) reruns the command and audits the ledger against the frozen criteria | verifier report; disagreements become defects | verifier report | unresolved disagreement = claim not PROVEN |

### P8 Transfer and P9 Release

| WP | Repo(s) | Prereq | Change | Acceptance | Evidence | Failure / recovery |
|---|---|---|---|---|---|---|
| WP-60 | SEA-Forge + all (config only) | 53 | second bounded situation: tax pre-filing statutory verification IDM (`.sea/interaction/tax-prefiling/`), template generated from it, one conforming and one divergent episode on synthetic data; **no change to RealityTrace, Casework or Horizon code** | AT-16: diff of the evidence-mechanism code between the two cases is empty | transfer report | a required code change is recorded as a transfer failure and the claim stays PARTIAL |
| WP-70 | godspeed-readiness, all | 54 | profile `fde_field_case`; pin all heads; release each repo per its release process; install from release artifacts on a clean cell; run the proof command there | AT-20; NT-14; GO report | `evidence/REPORT.md`, release notes with exact versions | NO-GO keeps previous releases; failing gate named |

## 3. First implementation batch

Start with WP-00a (measured baseline), then in parallel: WP-00b–e, WP-10 (the spike that can falsify AD-5 at
lowest cost), and WP-02 (the model, which every later package cites). WP-20 and WP-21 can start as soon as their
integration branches exist.

## 4. Known risks carried into implementation

| Risk | Early check | Fallback |
|---|---|---|
| DomainForge cannot evaluate declared policies over generated instances | WP-10 | classifier over AST JSON in `sxr-observed` |
| The SOP's obligations exceed what `policy` can express | WP-02 translation map counts `unknown` clauses | record as model limitation; comparison reports those elements `insufficient` |
| Host memory makes full-stack gates flaky | WP-00a measures | serialize; run gates in the readiness harness with RAM waits |
| Unfetched remotes hide newer work | WP-00a fetches first | re-plan P0 ledger |
| A3-UV unavailable on the proof host (no authenticator) | WP-22 | Chromium virtual authenticator for automated tests, labelled as such; human-reserved decisions in the real case need a real authenticator, otherwise those rows are BLOCKED, not PROVEN |
| The real Field Case does not naturally produce all four scenario kinds | WP-50 designs deterministic inputs through real execution paths | a kind not produced is reported NOT TESTED |

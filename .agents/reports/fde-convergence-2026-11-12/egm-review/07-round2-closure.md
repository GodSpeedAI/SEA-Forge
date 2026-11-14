# 07 Round 2: blocker closure, regressions, new defects (Goldfish, fresh context)

VERIFIED = read in the cited file or source. INFER = reasoned. Files: E, F, G, C, B, H under `$D`; Mission at
`.tmp/reference/Mission.md`.

## Part 1 Blocker closure

| Finding | Verdict | Evidence and residual hole |
|---|---|---|
| F1 anchor | CLOSED | RD-04 carried into E §5 "External anchor", G §4, NT-16/17/18/48/60, AT-04/AT-18. Chain alone never satisfies "before". Residual credential-isolation risk is R2-08. |
| F2 directed behaviour | PARTLY | G §2 drops the directives and adds provenance. But S1 runs only on `program` (G §9), and "Legitimate undeclared ... `program` only where the hash-pinned program contains the step" is the same scripted behaviour F2 attacked. Contradicted depends on E3's real outcome. R-02 is protected only by NT-26, which tests provenance, not "none instructed". |
| G-01 A3 | PARTLY | E §8 and RD-06 now say A3 on a separate surface, SEA-Forge validates, Horizon only shows status (matches source "A2 on a browser page is not sufficient"). Holes: (1) E §8 "With no A3 channel, reserved decisions are refused", yet the S1 gate needs AT-12 promotion approval before WP-31/32 exist; no S1 path is stated (G §3 just says PARTIAL). (2) Source's A3-UV vs A3-T evidence classes dropped. (3) E does not cite which of the two authority decisions binds beyond OD-1 in H. |
| G-03 emitter | PARTLY | RD-05 in E §5, NT-19/20/35/43/49. Not carried: fix (d), verified mapping of `declaration_key` outside the executor. E §2 only has RealityTrace re-derive the key from fields the emitter supplied, so an executor can still label an action with a favourable `semantic_ref`. Signing has no S1 owner (R2-04). |
| C-01 ingest | CLOSED | E §5 uses the CEP `execution_trace` envelope `extensions` (verified: `cognate/packages/realitytrace/src/godspeed.ts:48-60` emits `extensions` with `cep.profile`; sxr has `execution_trace` record kinds). `agent-event-v1` dropped. |
| C-02 evaluate | CLOSED | WP-10 spike then WP-12 `evaluate`; second evaluator withdrawn (E §1, §6, RD-08). Verified `Graph.operation_bound_policy_names` are evaluated in the operation's typed contract context, so the spike case (d) is the right gate. Gap: the `--facts` input schema and its generator are in no contract row (E §13 lists output only). |
| IR-01 principals | CLOSED | G §3 table, virtual-authenticator rule (NT-50), PARTIAL rule. Pre-S3 identity limits are R2-04. |
| IR-02 E2 | CLOSED | Directed E2 removed; I-1 attempt is deterministic NT-32/NT-33. |
| IR-03 contracts | PARTLY | E §13 gives schema paths, owners and minimum fields; honestly deferred to WP-11/WP-20. Holes: (1) frozen-reference minimum fields include `episode_ref`, while `episode_ref` is the digest of that reference (E §2): circular, not resolved. (2) E §13 puts the record-kind registry and SFWP frames in WP-20 (S2), but F WP-11/WP-14 need `case.record.append` for `episode.frozen/settled` in S1. (3) A3 evidence proof format (assertion fields, challenge, key distribution) absent from the `reserved-operation` row. (4) `outcome.status` enum values never listed (E §5). (5) `--mapping` and `--facts` formats unowned. |
| ZO-01 circularity | PARTLY | G §1 claims "no episode builds a package the proof needs", but E4 "Horizon request ids for route/reassign/withdraw (gap A)" is also F WP-32 "request ids and Reassign control (gaps A, B)". S1 executes E-episodes in repos that WP-32 later edits. E3's falsification is a real-world outcome, not controlled. |

## Part 2 Regressions and consistency

(a) VERIFIED: every WP id in B, C, E, F, G exists in `_revision-ids.md`; every WP is used. Only A:120 says "WP-00"
(shorthand for 00a-c). Meaning drift: E §13 assigns `case.record.append` contract to WP-20, F to WP-11 (see IR-03).
NT-20 is owned by both WP-17 (F) and WP-33 (F, G), yet is in the S1 gate.
(b) Ordering and lanes: violated. See R2-01 and R2-07.
(c) Terms. B §4 gives SOP term "none" for run and session, but SOP v0.4.1 §2.2 now defines both. SOP §7.6: a
`close-*` disposition "sets lifecycle `closed`"; E §3/RD-10/B: closed needs terminal state AND close disposition.
B: "one assignment per delegated attempt"; SOP §2.2: episode has n assignments; E §2: id uses the episode `n`
(R2-05). E §2 `declaration_key` rule contradicts RD-11 (R2-06). B J5 names an "observer-and-authority record" that
is none of the six kinds (it is a field_case.json group; say so). G still says "residue" where B/SOP fixed
carry-over (G is not operator text; low).
(d) Identical across E, H, F, G, B: six kinds, three verbs, I-1/I-2/I-3, five classes, `not_evaluable` as basis
(B words it as five difference words), op/class/evidence table (E §8 = H RD-06 verbatim; B paraphrases). Defects:
the regression matrix (E §7 = H RD-11) is not total: `confirmed -> declared_unobserved`, any `insufficient ->`,
`contradicted -> contradicted`, `observed_undeclared -> contradicted` have no verdict or default; G AT-12 repeats
only three of the four rows. `disposition.recorded level in case, episode` exists only in E §14/§3, not in H or
SOP §7.5 (episode-level close-satisfied is undefined).
(e) G S1 gate (G §9) lists the nine Mission §7 proofs (NT-01..09), control, frozen/model/exhaust/derive/compare/
revision/subsequent/Horizon/script/clean cell. It does not truly cover Mission §1 and §4: AT-07 passes with kinds
"reported NOT TESTED", and only conforming and legitimate-undeclared gate S1 (G §2). Achievability with S1 WPs
alone: doubtful (R2-02, R2-04, G-01 hole).
(f) C: 168 distinct rows, 76 critical (VERIFIED by script), each critical row cites an NT, all NT ids exist. 12-row
random sample (seed 7): R-58, R-16c, R-19, R-22b, R-54c, R-71f, R-106, R-12 protect their property; R-43e only the
`insufficient` half (no ambiguous case). Weak: R-32 (NT-02a covers only missing model identity, not actor/operation/
resource/context; NT-15 is unregistered world), R-72a (NT-31 attacks vacuity only; "zero `contradicted`" never
attacked), R-114 (NT-32/33 cover two of three legs; missing anchor -> delegation refused is untested, NT-16 only
tests verify-fail). Also R-02 above. 10 critical rows cite an NT whose owning WP is absent from the row's Change
column (R-02, 21, 31, 32x2, 33, 40, 41e, 43b, 72b).

## Part 3 New defects

**R2-01 MAJOR. Acceptance needs a later-built script.** G header: "Script is created by WP-19 (S1 form)"; every
G command is `scripts/fde-proof.sh --only <ID>` or `scripts/fde-anchor.sh verify`. F WP-09 (prereq 00c)
acceptance is `scripts/fde-anchor.sh verify fde-s1`; WP-13/14/16/17/18 accept AT/NT ids. The anchor also needs
the case-keeper publisher built in WP-14. Fix: WP-09 builds both scripts (skeleton, `--only`, `--mutate`, anchor
verify) and anchor publisher stub; add WP-09 as prereq of 13, 14 and 19.

**R2-02 MAJOR. S1 is no longer thin.** G §9 S1: 12 AT, 45 NT each with a hash-pinned mutation patch (NT-01..09,
03b, 13, 15..18, 20, 21, 25..38, 42..49, 52..55, 60, 61), E1-E5, 11 WPs (09-19) over 8 repos, external GitHub
anchor, two-phase freeze, durable world pin. F §1 calls S1 "one episode loop". ZO-02's thin slice has been
rebuilt as a fat one. Fix: S1 gate = Mission §7's nine NTs + control + NT-16/17/18 + AT-01/04/06/07/12/13/14/18;
the rest become S2 regressions; name the S1 episode set in WP-09.

**R2-03 MAJOR. S1 claim "Covers Mission §1-7" overstated.** See Part 2(e). Fix: state which of the four kinds
gate S1 and downgrade the claim, or make the contradicted case deterministic (a `program` falsifier with a known
result) and require a `live` run for undeclared behaviour before the claim.

**R2-04 MAJOR. S1 signatures and execution identities have no owner.** E §5 `emitter.signature` required; RD-05
"every interaction event is signed"; AT-05 (S1) "check signature". WP-15 does not sign; WP-14 "signed ... emission"
names no key; WP-30/33 and NT-19 are S3. I-1 "on the execution identity" (WP-14) and I-2 "delegation chain"
(WP-21, prereq 20 only) need distinct execution identities; source design line 21: "Cognate runs inherit
`startedBy` as their actor". Round-1 C-05 (I-2 after identity) re-opens. Fix: name an S1 interim emitter key
(SEA-Forge service key, scoped claim) or drop signature from S1 gate; add WP-30 to WP-21's prereqs or define the
interim chain source.

**R2-05 MAJOR. `n` has two meanings.** E §2: `n` "per `(case_id, plan_item_id)`, case keeper, at freeze" (episode
counter); `assignment_id` row "using the case keeper's `n`"; H RD-13/§4 calls it "an instance number"; SOP/B:
several assignments per episode. Re-delegation inside one episode collides on `assign:<case>:<step>:<n>`. Fix:
separate `assignment n`, owned by the case keeper, or include `episode_ref` hash.

**R2-06 MAJOR. `declaration_key` derivation is wrong.** E §2: "derived from `world_ref` + `namespace::name`".
VERIFIED `domainforge-core/src/concept_id.rs:8-11`: `ConceptId::from_concept(namespace, name)` is uuidv5 over
`namespace::name` only. Following E would change keys every version and break RD-11 "stable across versions".
Fix: correct E §2 and the WP-11 schema.

**R2-07 MAJOR. One-lane-per-repo is broken.** Concurrent same-repo pairs with no ordering prereq: WP-09 and WP-13
(+WP-11 SEA-Forge vendoring) after 00c; WP-15 (Cognate) and WP-30 (no prereq; S3 "starts after S0") and WP-23;
WP-32 and WP-40 (Horizon); WP-34 and WP-23/42 (Workbench); WP-33 SEA-Forge part and WP-21/22. WP-31 alone is
sequenced "after WP-21". Fix: add explicit prereqs or a per-repo queue table.

**R2-08 MAJOR (INFER). Anchor independence and clean cell.** RD-04/G §4: tag pushed by the "case-keeper service
identity", "no executor credential has push rights". Executors (Sonnet agents, `program`) run on the same host and
OS user as the owner's git credentials; the S1 adversary model does not resist that. Verification needs GitHub
network and API (server push time) on the "clean cell"; F §0 says credentials escalate, but H §5 has no decision.
Fix: add an OD for a dedicated deploy key and tag-protection ruleset, and state the offline fallback.

Not new but note: F §0 "Test ownership" claims G lists "62 negative tests"; 61 plus NT-03b. Matrix size
(168 rows, 76 critical, 62 mutated NTs, all re-applied by the verifier in S5) is executable only if S1 shrinks.

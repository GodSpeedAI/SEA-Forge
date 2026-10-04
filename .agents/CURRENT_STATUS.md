# Current Status

Updated: 2026-10-01

## Migration Branch Note (migration/cep-world-ref)

This branch is a separate workstream off `casework/live-wiring` @ dc82736: the
CEP-0008 / `world_ref` migration (Stages 0-12). It does not advance T09; the T09 narrative
below is inherited and unchanged. Ledger: `.agents/reports/cep-world-migration/stage0-migration-map.md`.
Stage 0 (recon, migration map): settled 2026-10-04, docs-only. Next: Stage 1 (DomainForge 0.18.2
convergence) spec and plan written (`specs/cep-world-ref-migration.spec.md`, `plans/2026-10-04-stage1-domainforge-convergence.md`).
Stage 1 sea-rs unit: `domainforge-core` 0.16.0 -> =0.18.2, `EXPECTED_DOMAINFORGE_VERSION` and devbox pin updated; dependent crate tests and `just test` pass.
Lint gate: the inherited clippy findings in `sfwp_supervisor.rs` and `sfwp_delegated_identity.rs` are fixed on this branch (behavior-preserving; tests re-run), so `just lint` is green here. `just security` was red on 14 gitleaks hits in historical `.agents/` evidence (commit 44b5a0b3); each was inspected (fixture JSON and benchmark row ids, no credentials) and fingerprinted in `.gitleaksignore` without editing the immutable evidence. `just check` is now green and `just test` has 0 failures.
Release: DOMAINFORGE 0.19.0 IS PUBLISHED (crates.io, PyPI 16 files, npm napi and wasm, GitHub release v0.19.0; merged #132, #133, #134). SEA-Forge `sea-forge-domainforge` is now `=0.19.0` (fail-closed `EXPECTED_DOMAINFORGE_VERSION`, devbox pin, lockfile); `just test` 1110 passed / 0 failed. Earlier note, superseded: DomainForge PRs #132 (world_ref) and #133 (stacked snapshot binding) and cep PR #1 are open with CI green; merging is held for operator review (auto-mode blocked an unreviewed merge). The release (expected 0.19.0) and the consumer bump follow the merges.
Stage 3 (cep `profiles/godspeed-v1` 36b5ca6; DomainForge `feat/cep-snapshot-world-ref` 2bf2df4): see `.agents/reports/cep-world-migration/stage3-settlement.md`; awaiting DomainForge merge and one release.
Stage 2 (DomainForge `feat/world-ref` a3a4157): see `.agents/reports/cep-world-migration/stage2-settlement.md`; blocked on DomainForge merge+release before consumers can use it.
Stage 1 record: `.agents/reports/cep-world-migration/stage1-settlement.md`. Legacy SEA branch unpushed (pre-push hook blocked by baseline generation drift).

## Objective

Complete `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml` through T12,
then split T13 and stop for operator review. Follow `current_status.yml`.

## Worktree State

Branch `casework/live-wiring`; HEAD Ask T09 checkpoint `0b55022`; cap/mirror checkpoint `b4092bd`; canonical checkpoint
`b608771`; T08 checkpoint
`af362f5`; Store checkpoint `48783dc`; T07 checkpoint `aed3494`.
Preserve unrelated Jolli debug log and trusted-daemon target plan/spec plus other working changes;
stage explicit task paths only.
Approved cap, Go/UI mirrors and protected Ask runtime/evidence are checkpointed. Unit4
run-child fixture, scoped-test declarations/new tests and evidence remain uncommitted;
runtime implementation is held for independent scoped expected-RED verification.

## Changed Files

T08 native Go/TS live harness, retention-configurable test stack, owned shared runner,
shared replay assertion and focused local regressions. Store production repair and red/green
evidence already committed. New independent T08 source/gate/proof records and immutable failures.
T09 canonical Ask/observation types, schemas, specs, tests/goldens and independent evidence
are committed at b608771. SFWP client/subscriber cap, Go/UI mirrors, focused tests and
independent RED/GREEN/review evidence are committed at b4092bd. Ask fixture source review
rejected missing numeric/boolean roots; a fresh builder repairs only that test matrix.
All original assignments and rejections remain durable.

## Completed

T00-T08 settled. Full independent T08 approval:
`T08/resume-2026-09-30/independent-final-confirmation.md`, with append-only
`independent-final-confirmation-go-gates-errata.md`. Root verified all13 source/log manifest
entries, both record hashes and byte-identical copied fresh Go logs. Earlier outage debt closed.

## Verification

T09 bounded mirror unit independently APPROVED in mirror-repair-independent-green.md;
root verified16 source/log manifest hashes and36 exact raw/tmp copies. All pre-Ask Go gates
pass; UI263/0/1427 plus production build pass. ROOT compiler token after critic release.

T09 canonical bounded unit independently APPROVED in canonical-final-independent-confirmation.md:
fresh Bun11/0/963 assertions and standalone strict TypeScript exit0. Root matched four source
hashes and both raw logs against original /tmp captures. Runtime enforcement remains pending.

Fresh independent stronger native EventSource race passes35.679s: real authenticated browser,
retention-one resync/retry/disposal without third request, retention-two replay after offline
mutation. Exact full SSE snapshots match stored K/L; current reads require the exact same cursor
and all stable fields, with fresh render timestamps separately validated.
Shared local regressions16/16 and actual shared live port suite pass. Canonical UI passes
258 tests/1271 assertions. Default/local/invalid canonical production builds exclude local data.
Fresh post-Store canonical Go format/vet/tests and full module race pass. Store regression
race/count25 passes. Unchanged Rust594pass/4existing ignored and local ladder11journeys/78steps
are reused with explicit source-scope limits in the independent review.
All raw logs, failures and evidence corrections are immutable under T08/resume-2026-09-30.

## Remaining

Unit5 auth/trace/session source facts and unit6 fixture-source preparation are recorded
under T09/resume-2026-09-30. Session logout/expiry does not currently cancel open SSE;
safe sharing/authorization/lifecycle and terminal predicates need explicit root design
and independent critique before implementation. No UI writes or compiler release occurred.
Additional unit5 cancellation source facts: manual context cancel does not interrupt active
SFWP socket reads; Client.Close only closes idle pool. A cancel-and-drain claim needs actual
blocked-read proof and preservation of mutation/Ask uncertainty semantics.
Cancellation test-first proposal remains HOLD after independent source critique: required
manual Ask/mutation cases and genuine peer-EOF/connection-reuse evidence are added in a new
supplement. Repeat critique before fixture release; unit4 confirmation/checkpoint first.

Current step: bounded unit4 independently APPROVED8b1432ff plus clarifications87e90d10.
Root matched11sourcehashes/17exactrawpairs, exactfixtureinverse/helperequivalence; Graft
buildPASS679cards/7398nodes/14142edges. ROOT token. Explicit localcheckpoint next, then
cancellationfixture proposal (independently sourceapproved with supplement/timeout/peer
requirements) released tests-only to freshLuna builder. FullT09pendingunits5-8/globalgates.
Final staged review: full git whitespace check flags ONLY immutable raw rejected.diff
context markers; allother115 files pass. Exact historical artifact retained, compiler/
format/test gates PASS; no full staged-whitespace pass claimed.

Previous step: independent focusedrace, actualtwo-case kernel proof, canonical Go and full
module race all reportedPASS. Critic finishing immutable archive/review while retaining
SOLEtoken; root mustverify finalsource/rawidentities before unit4approval/checkpoint.
Post-PASSrelaycleanupwarning willbe disclosed. FullT09 remains pending units5-8/globalgates.

Previous step: fresh livefixture externalpackage repair frozenffc979ee; root exact inverse
restores original8e485 and renamedhelper body matches byte-for-byte. First snapshot2c982
was a diagnostic-argument transcription error and remains untouched; NEWcorrected snapshot
matches8e485 exactly. Root source/capture erratum recorded. Primary mirror critic owns SOLE
token to repeat source/focused/live/canonical/fullGo. Unit4/fullT09 remain unapproved.

Previous step: independent Phase2 REJECTf35d91ce: focusedGREEN passes but livefixture import
cycle prevents testbody/kernelproof. Root matched6rawpairs/11sourcehashes. Fresh Luna
run_scope_live_fixture_repair owns ONLY livefixture externalpackage/helperqualification,
preservingoriginal8e485snapshot first; allothercode/tests frozen. ROOT holds compiler token,
builder has none. Repeat independent focused/live/canonical/fullGo after sourcefreeze.

Previous step: fresh Phase2 source frozenlive0f17e374/authoritya9c49702; only2 authorized
files changed and alltests frozen. Primary independent mirror critic owns SOLE compiler
token for source/focusedGREEN, actual two-case kernel proof and fresh canonical/full Go.
Unit4/fullT09 remain pending; no approval is inferred from builder formatting checks.

Previous step: independent Phase1 regressionRED APPROVED2d234225; both focusedracecommands
compiled and reached intended defects. Root matched6rawpairs and10sourceidentities.
Fresh builder Phase2 fixes ONLY live.go/authority.go; tests/Ask/liveproof frozen. ROOT
owns compiler token, builder has no token. Comprehensive independentGREEN, real two-case
kernel proof and fresh canonical/full Go remain required before unit4 approval.

Previous step: semantic repair Phase1 tests frozen57bfc686/a9e0deb0, production unchanged.
Root matched both new test identities, both original rejected snapshots and six-file
rejected diffb0a6b08b. Evidence-only archive13 raw attempts/preflights/exits matches every
original byte/hash; manifest93d8a275 records all failures and invalid innerGREEN honestly.
Primary mirror critic owns SOLE compiler token for independent regressionRED; fresh builder
has no token and sourcePhase2 awaits independent approval/root checks.

Previous step: PhaseB inner race04 passed but independent semantic REJECT8404a0f found a
wrong fixture assumption and two production errors: missing requested case was read through
then classified unavailable based on unrelated cases; all refusal classes were normalized
unavailable. Fresh t09_run_children_fixture_builder repairs2 scoped tests FIRST under
semantic-repair-assignment; production and all otherfixtures frozen, no compiler token.
Independent regression RED precedes source repair. Archive recovery preserved exact sandbox
incorrectcopy; host incorrectcopy and a removed recovery transcription remain unrecovered,
with explicit limitations. Actual six command/preflight originals/correct copies are intact.

Current step: phaseA independent source/compiling scoped assertion RED approved; root matched
6 source identities and6 exact raw copies/full hashes. Critic corrected2 raw archives by
overwriting initial incorrect versions; fresh archive builder restores those oldhash-matching
versions to new labelled paths and an erratum. Actual/tmp outputs and current copies are exact.
PhaseB runtime is released under original assignment/test phases/real-kernel supplement to
t09_ask_fixture_independent_critic, sole compile token for narrow innerGo loop with saved
host preflights. Existing4 new pure/scoped/clone fixtures remain frozen. Then independent
GREEN, actual two-case proof and canonical/full Go gates; full T09 still pending.

Current step: scoped phaseA fresh repair frozen0013b05f.../361fc032...; synchronous dial
counter, exact scoped-case assertion and root-authorized whitespace-only formatting are
complete. Other3 phaseA files and pure fixture unchanged. Primary mirror critic rereviews
original instructions/all actual files, then owns conditional sole token for scoped assertion
RED with saved raw host preflights. Runtime implementation remains held.

Current step: independent scoped phaseA source REJECTc7590c52... found two fixture gaps:
zero request lines does not prove no dial, and fake scoped method ignored requested case.
Fresh t09_run_children_fixture_builder repairs only2 scoped tests; root requires synchronous
Config.Dial counter plus accepted-connection/request counts and exact requested-case assertion.
All declarations/other fixtures frozen. No compiler token or test run; independent re-review
and scoped assertion RED still precede runtime implementation.

Current step: runtime phaseA frozen5files (semantic result/factual unreadable slice declarations
plus3 new scoped tests); repaired pure run-child fixture4ddb2445... unchanged. Primary mirror
critic has original runtime instructions/test-phase supplement and actual freeze hashes for
source review then compiling scoped assertion RED, with conditional sole compiler token.
Runtime implementation remains held; future preflights must have saved raw originals.
Observation source constraints are recorded in observation-runtime-source-review-notes.md;
root must decide authorized sharing/lifecycle/requested-case seam before poller implementation.

Current step: run-child fixture independently approved as compiling assertion RED; root
matched source/review hashes and4 exact raw log/exit copies. Two host preflight archives
have hashes but no saved original files; preserve that limitation. Compiler token released.
Runtime phaseA declarations/scoped tests are released to t09_ask_fixture_independent_critic
under original runtime assignment plus run-children-runtime-test-phases.md, without compiler
or runtime implementation authorization. Freeze/review/scoped assertion RED precedes GREEN.

Current step: fresh run-child fixture repair frozen4ddb2445...; independent source review
approves focused expected assertion RED. Primary mirror critic owns sole compiler token
for that projection race run, with actual host RAM/compiler preflight. Runtime builder
prepares read-only under run-children-runtime-assignment.md; implementation stays held
until independently evidenced fixture RED. Full T09 remains pending.

Current step: independent run-child fixture source review REJECTED missing exact expected-ID
multiplicity (total count plus expected membership permits duplicate/missing children).
Immutable rejection d549a42f... is preserved; fresh t09_run_children_fixture_builder repairs
only the fixture matrix. No compiler token. Repeat independent source review then actual
assertion RED. run-children-runtime-assignment.md is prepared but implementation remains held.

Current step: run-child fixture frozen SHA68511dc5618c9609e27ae501794265d8fd4d5b18d62466e2465dd51b3507269d.
Builder corrected whole-focus equality under root clarification before any critic verdict;
parent focus/narration/order must persist while new real child IDs may enter salience rank.
Primary mirror critic reviews original instructions, clarification and current fixture; no
compiler token until source review is adequate. No tests run yet; full T09 remains pending.

Current step: independently approved Ask source/evidence checkpoint0b55022 complete.
Bounded run-child fixture builder t09_ask_fixture_independent_critic may write only new
projection/run_children_test.go under original assignment plus root fixture clarifications.
No compiler token granted. Independent expected assertion RED precedes runtime implementation.
Full T09 remains pending; compiler serialization and host preflight still apply.

Current step: Graft refresh passed674cards/7349nodes/13995edges; context-check passed.
Foreign Bun PID2055592 runs server.ts with no compiler children, so it is not a compile blocker.
Leave it untouched. Ask bounded approved checkpoint is next; run-child fixture writes remain
held until that checkpoint. Full T09 pending.

Current step: final bounded Ask runtime independent APPROVE is durable in
`ask-runtime-independent-final-confirmation.md`; root matched15 source hashes and13 new raw
copies/full report hashes. Same authorized native archive patch retry succeeded, closing
the temporary usage-limit write blocker. All Go gates complete and compiler token released.
Ask checkpoint awaits Graft refresh and fresh context-check; latest host scan shows foreign
Bun PID2055592 active, so do not overlap compiles. Run-child fixture source preparation is
complete, but writes remain held until Ask checkpoint. Full T09 remains pending.

Current step: Ask runtime GREEN gates complete, compiler token released. Renamed live fixture
`TestLiveAskLedger` changes only its identifier (SHA b97556553bf65a1105e0bfaaccb57e0f421b004141c3a73f782a3cebb0c5478a).
Independent real-kernel proof passed twice under the original temporary root; observed socket
path84bytes fits the kernel95byte limit. Fresh canonical Go and full-module race/count1/parallel1
passed. Final independent archive/review patch was rejected by automatic approval review's usage
limit, not a safety finding. Exact new originals remain under
`/tmp/sea-rs-ask-runtime-gates/phase-b/`. Preserve them and the final independent review before
Ask approval/checkpoint or further Go writes. UI optional fifth typed callback architecture was
independently source-reviewed: preserve canonical counts, separate local frame admission/notices,
clear subscription cache and app sidecar on disposal, and hide current annotations in history.
Full T09 remains pending. Following descriptions are historical.

Current step: PhaseA UTF8 fixture approved with actual three-case RED; root matched all three
raw log/exit/preflight originals. PhaseB guard/documentation repair froze, with original and
new fixtures unchanged. Primary mirror critic owns sole compiler token for complete original
assignment review, GREEN, adapter/transport regressions, real owned-temp-cell kernel ledger
proof and canonical/full Go gates. Run-child Go writes remain held until all those commands finish.
No Ask runtime/T09 approval yet. Following prior descriptions are historical.

Current step: PhaseA fresh repair froze: helper rename, stale comments and three-case raw-byte
fixture. No raw UTF8 guard yet. Primary mirror critic owns compiler token for source review
and focused expected RED only, with saved actual host preflight and raw outputs. PhaseB guard
waits evidence-backed fixture approval. Full Ask/T09 remains open; run-child Go writes held.
Following prior descriptions are historical.

Current step: primary Ask rejection capture complete; root verified all13 frozen source hashes,
three artifact hashes and exact original log/exit/host preflight copies. ROOT owns compiler token.
Fresh t09_ask_fixture_independent_critic performs PhaseA rename/comments/new raw-byte fixture,
without guard or compilation. Primary critic then source-reviews and runs expected UTF8 RED;
only after that approval PhaseB adds the guard before complete GREEN/live verification.
Following prior descriptions are historical; run-child Go writes remain held.

Current step: independent Ask runtime review REJECTED. First Go command failed to compile
because production askBody collides with a frozen fixture helper. Secondary source review
found missing raw UTF8 rejection before JSON normalization and stale correlation comments.
Primary critic preserves failure evidence before token release. Fresh builder
t09_ask_fixture_independent_critic prepares helper rename/comments/new raw-byte fixture,
then independent expected RED before adding the raw UTF8 guard. Edits held until capture completes.
No Ask runtime approval or T09 settlement. Run-child Go writes remain held.
Following prior descriptions are historical.

Current step: Ask runtime source/new adapter and live proof tests froze without compilation.
Independent t09_mirror_independent_critic owns the sole compiler token for original-spec review,
focused/full Go gates and actual owned-temp-cell ledger proof. Existing frozen fixture hashes
are unchanged. Root byte-verified separate original sandbox log/exit copies; contaminated earlier
artifact remains immutable with capture errata. Run-child Go writes remain held until Ask gates finish.
Following prior descriptions are historical.

Current step: Ask fixtures independently APPROVED in ask-fixture-independent-review.md.
Actual expected RED reaches missing route405 and unsafe EOF/overflow double-send; root matched
four source hashes and both raw logs. ROOT owns the compiler token. Fresh runtime builder
t09_run_children_fixture_builder implements the original assignment plus root integration
supplement and ledger projection clarification, without compiling. New adapter/live proof
fixtures and narrow livestack injection are authorized; frozen Ask tests remain untouched.
Independent t09_mirror_independent_critic prepares review and subsequent serialized gates.
No T09 settlement yet. Following earlier orchestration descriptions are historical.

Current step: cap/mirror checkpoint b4092bd complete. Ask fixture critic owns the compiler token.
Independent Ask fixture source review rejected missing numeric/boolean scalar roots.
Fresh Luna t09_ask_scalar_fixture_repair froze only server/ask_test.go, without compiling,
SHA90612d084a10fce6ec4871faae563790c7de5267d2ef60369bc1fc23bd538cbe.
Critic re-reviews and runs expected RED with actual host RAM/process checks.
Ask runtime implementation is held until fixture approval. Run-child and real-kernel Ask
proof preparation is delegated read-only, with no competing compilation.
Following orchestration descriptions are historical and superseded by this paragraph.

Current step: independent mirror critic owns the sole compiler token. Four mirror files,
fresh two-test drift-check repair and three whitespace-only Ask answer fixture repairs are
frozen. Canonical Bun11/0/963, strict TypeScript, focused Go contract race/count1 and
host-escalated just casework-go-check and full module race/count1 pass. UI typecheck passes.
Critic completed UI wire15/0/512 and host justUI263/0/1427 including production build,
independently APPROVED mirror unit and released compiler token toROOT. Unrelated Bun process1167718
exited before UI tests; it was preserved. Earlier runtime-directory/socket sandbox
failures are retained separately. All gates use actual HOST preflights, owned writable temp
paths, real exit statuses and GOLDEN_UPDATE disabled. Preserve immutable raw failures.

All critic Go commands finished; Ask test builder is explicitly released to write only assigned
fixtures and minimal ports/Options declarations per ask-test-first-assignment.md, without compiling.
Ask fixtures/declarations froze exactly four assigned Go files with no behavior/compile.
Independent Ask fixture critic begins source review, awaiting explicit compiler transfer.
Go evidence refers to the pre-Ask source graph; later Ask/global gates must rerun.
Original mirror assignment prohibits authored golden edits; the three
fixture whitespace repairs are a separate root-authorized integration correction, with parsed
JSON unchanged from b608771. Both canonical and mirror verification must cover that correction.
Run-observation read-only preparation is complete; runtime integration decisions are recorded.
Full T09 remains unapproved. Earlier activity descriptions below are historical.

T08 checkpoint complete; Graft refresh and context-check pass. Implement approved
T09 units in `T09/resume-2026-09-30/prepared-implementation-units.md`; exact source inventory
and root DTO/state decisions are in `contract-source-preparation.md`.
Start canonical contract/spec/schema/golden/parity amendment and bounded SFWP response-line cap,
then protected Ask, real run observations, authority-scoped bounded pollers, UI caches and journeys.
Every bounded builder gets an independent critic with original instructions and evidence;
fresh builder after rejection. T09 has not been implemented or independently approved here.
Initial disjoint Luna builders froze canonical and SFWP cap tests. Independent fixture
review REJECTED both sets without compilation. Historical fresh
`t09_contract_fixture_repair` fixes schema inventory, nested shapes and typed count omission;
`t09_cap_config_fixture_repair` adds configured-limit tests plus one authorized Config field
declaration seam. Default/validation/enforcement are intentionally not implemented until RED.
Original assignments, defects and the scope exception are recorded in T09 evidence.
After repair freeze, independent critic reviews original instructions and tests, then runs
expected RED serially; only afterward implement and independently verify focused GREEN.
All eight focused cap cases independently ran race/count1, exit1 with expected enforcement
failures. One oversized-event case falsely passes via100ms timeout; fresh t08_type_builder
repairs only that fixture. Canonical nested/schema fixes are reviewed, but command status
used the wrong run-standing enum; fresh t09_cap_config_fixture_repair fixes actual5 statuses.
Canonical fixtures independently source-approved; fresh Bun8pass/3expectedfail output
captured exactly in canonical-independent-red.log and root byte/hash checked. Fresh Luna
`t09_canonical_contract_builder` implements only authored types/SSE/Ask schemas and specs,
without compiling or editing frozen fixtures. Repaired cap fixtures independently source-approved;
host race/count1 RED reaches all8 cases including actual oversized valid event delivery.
`cap-repaired-independent-red.log` SHA6561396751e2bcb8d9ed138099ee208481c2864b1646587159c435a149c50332.
Fresh cap builder implements only client/subscriber shared reader, without tests/Ask edits or compile.
Cap implementation is frozen and its critic alone compiles. Canonical implementation source
review REJECTED missing Ask root linkage and expressible observation state constraints;
fresh `t09_canonical_schema_repair` froze two schemas plus focused test pins, no compile.
Canonical critic reviews the repair source only. Cap8focused race GREEN passes;
`just casework-go-check` format/vet/SFWP pass but contract mirror omissions keep overall gate red.
Full module race finished, fullSFWP package race passed; cap unit independently APPROVED
with broader contract gate failure explicitly pending mirrors. Mirror builder now edits four
Go/UI mirror/test files, without compiling. Canonical critic owns sole compile token for focused
Bun GREEN and explicit canonical TypeScript pin checking after source-approved schema repair.
Canonical Bun11pass/0fail/952assertions is captured; strict standalone TypeScript REJECTED14
TS2769 helper matcher errors. A fresh builder repairs only test helper typing without dropping
runtime assertions or compile pins. Exact log corrects initial15-error report; canonical
critic explicitly released token toROOT. Canonical unit approval still awaits strict typecheck.
Cap approval includes append-only independently verified Ask recovery errata: no correlation
or status recovery for Ask; uncertain transport sends once and returns unavailable.
Arithmetic count equations require later runtime validation, not nonstandard schema extensions.
Detached narration metadata is a disclosed limitation pending actual UI port/narration wiring.
Independent GREEN and broader gate approval remain pending for both implementation units.
Exact earlier RED and rejections remain retained.
Continue T09-T12, preserve existing T10 preregistration, split T13 then stop.

## Blockers

No operator approval blocker. T09 approval persists; do not reask.
Recovery found no surviving agents. Independent fixture approval is durable in
`fixture-independent-review.md`; cap implementation is independently approved, with Ask
recovery errata. ROOT owns sole compile token after final mirror approval/release.
Canonical strict typecheck previously failed on14 helper errors; fresh repair independently
passes. Four-file Go/UI mirrors and Ask fixtures are in progress. Builders may edit only assigned disjoint files and may
not compile. Canonical strict typecheck and restored broader Go/UI gates remain required.
Check actual host RAM/processes before
EVERY compiler; serialize Go/Bun/Cargo. Go256MiB/GOGC50/GOMAXPROCS2/p1/race/count1;
Cargo jobs1. Preserve foreign processes and historical failed cells; clean up only owned resources.

## Decisions

Only Luna/Terra subagents; Terra unavailable. No publishing/deployment/external writes.
Public canonical contract is authored; no Workbench generation, Rust verb or dependency change.
RunTraceObservation is cohort; nested RunTraceRunObservation is distinct from invocation
ExecutionObservation. Preserve full actual Thoth views and disclosure fields.
Graft refreshed after T08 tracking:665cards/7214nodes/13616edges (exit0).
Context-check passes. Historical notes below are superseded by this primary handoff.

---

# Historical handoff notes

> **2026-09-29 resumed after usage-limit wrap:** recovery source and evidence were externally
> committed as `8741ff4`; only the Jolli debug log was dirty at resumption. The final wrap status
> write was blocked by automatic approval review usage exhaustion. T06 remains last settled.
> Fresh independent Luna critics now review T07/T08. `t07_final_independent` exclusively owns
> compile/test/build work after RAM checks; `t08_final_static` has no compile token.
> Final session-read/provenance and TS stale identity/retry/resync changes await verification.
> Continue from T07 real-session current/history/SSE proof, then T08 gates and the shared
> live conformance suite; prior passing logs do not approve these final source changes.

> **Latest resume findings:** T07 fresh vet passed, but focused/global Go race found four test
> setup failures (absolute URLs doubled by the GET helper; mismatched legacy resync actor).
> A fresh builder repairs the test fixtures while preserving negative authority assertions.
> The rebuilt gateway's loopback launch was rejected by automatic approval review as alleged
> injection drift; explicit approval is pending. T08's F13 trajectory validator repair awaits
> independent Bun gates, now exclusively owned by `t08_final_static`. Live shared conformance
> and actual native EventSource resync/reconnect proof remain required. No task advanced.

> **Verification progress:** independent post-repair T07 focused race and vet pass. Global Go
> race found a probabilistic ineffective mutation in the existing Argon2 hash test; one
> characterization retry passed. A fresh builder now changes decoded digest bytes reliably.
> T07 critic holds the compile token for Rust then final Go gates. Fresh T08 builders repaired
> trajectory endpoints, native retry timer setup and four remaining type errors; final source
> awaits independent gates. Initial failures and rejections are retained as evidence.

> **T07 final source gates:** fresh independent targeted Argon2 race test passed 50 runs;
> full Go race, vet and four-crate Rust gates pass. The critic approved the bounded fixture
> and Argon2 test repairs, while withholding full T07 approval for missing live runtime proof.
> `T07/resume-2026-09-29/final-independent/confirmation.md` is the operative scoped review.
> Compile token transferred to `t08_final_static` for final UI gates and the local ladder.

> **2026-09-30 UI gate progress:** final focused Bun passes 43 tests/144 assertions and
> typecheck passes. Full Bun encountered sandbox EPERM for an existing temporary loopback
> listener; critic reruns through approved local execution. Live gateway approval is pending.
> Immutable T07 critic report contains two intentional Markdown hard breaks flagged by
> `git diff --cached --check`; source whitespace checks pass. No evidence record was rewritten.

> **T07 interim checkpoint:** approved verification repairs and evidence committed as `5e07c62`;
> live confirmation remains pending. T08 round7 rejects production bundles (default/local/invalid
> all contain the local adapter) and full Bun (254 pass/1 EPERM listener failure). Fresh builders
> repair direct DEV import isolation and explicit loopback test binding. Compile token released
> during source repairs; independent final gates and the local ladder follow on stable source.

> **Stable source ready:** both direct DEV import isolation and loopback listener repairs
> are complete. `t08_final_static` exclusively owns their fresh independent gates and local
> agent-browser ladder. Live gateway launch authorization remains pending; no task advanced.

> **Build environment diagnosis:** inherited `NODE_ENV=development` makes Vite's `DEV` flag
> true even during `vite build`; installed Vite source confirms it. A fresh builder prepared
> the one-field package script pin to `NODE_ENV=production`, awaiting controlled verification.
> Local browser ladder is running on stable source (J0 passed). Full Bun passed once 255/0,
> while canonical repeats still hit the listener error; execution context is being diagnosed.

> **Controlled production build confirmed:** explicit `NODE_ENV=production` with a `local`
> source override emits only the HTTP adapter and no local adapter/Northstar/narrator markers.
> The canonical script still needs its planned production environment pin. Local ladder
> J0-J4 passed; package edits remain held until that run finishes.

> **2026-09-29 takeover in progress:** recovered T07 (`eef7459`), T08 (`ce538bf`), and
> T09 (`3e2404f`) implementations; independent confirmation remains outstanding for all three.
> T00-T06 retain their operative independent evidence. Fresh T07 Go vet/race gates and the
> four-crate Rust gate passed (594 tests, four existing ignored). T07/T08 critics rejected;
> fresh builders are addressing origins, log escaping, production selection, artifacts,
> stale refresh, interruption and retained trajectory. Independent re-review is pending.
> A daemon restart interrupted round-2 work. State was checked before resuming; a recovered
> critic is probing cached/SSE reads that retain the relay's default actor perspective.
> T09 inventory found missing live execution frames and
> grounded narration, plus incorrect unavailable journey labels. No new task is settled.
> Compile work is serialized across all agents after a RAM check. Machine state and next move:
> `.agents/current_status.yml`; fresh evidence: `T07/resume-2026-09-29/` and
> `T08/resume-2026-09-29/` under the active evidence root. Preserve unrelated working changes.

> **2026-09-23 COGNITIVE UI FUNCTIONAL-COMPLETION PHASE (contract adapter, not Go):**
> The UI in `apps/godspeed-cognitive-ui` now consumes the interface-contract port
> (`src/ports/contract.ts`), served by a local contract-conformant adapter
> (`src/adapters/local/`: contract data at rest, authority, execution events,
> snapshot-borne settlement). Layout is data-driven and no fixture ids appear in UI code.
> Added:
> - the comparison representation, time marks and compare, and case-design drafts and compare;
> - lazy source renderers (diff, text, markdown, table, chart, JSON, graph, trace, timeline) behind an error boundary;
> - the narration port with an interruptible, resumable streamed player;
> - one intent path shared by human and agent.
> **Affordance-dependency E2E ladder** (`e2e/`, real pointer input through
> agent-browser): J0–J9 + RECOVERY **11/11 PASS, 78/78 steps**. **GATE_UI PASS**
> (`just casework-ui-check`: 185 tests / 633 assertions). VAR-001..007 are exercised UI-side.
> Deferred: RECOV-001/002, and real Go, SEA Forge and Gauntlet integration.
> Report: `.agents/reports/godspeed-cognitive-ui-functional/`.
> Evidence: `.agents/evidence/godspeed-casework-cognitive-environment/ui-journeys/latest/`.
> Nothing committed.

> **2026-09-22 COGNITIVE-UI CORE INTEGRATION (PATH B) HANDOFF:** Continued the
> opencode session that died mid-turn (`ses_f36860523ffenPfOlcyO6sXAdM` looped
> on a repeated bash command, emitted malformed tool JSON, then returned empty
> responses after compaction). Its port had landed as a blind `cp -a` and was
> never validated: nothing imported it and `just casework-ui-check` was RED.
> Repaired the relocation (import depth, deduped `CoreVisualProjection`,
> vitest→`bun:test`, `?raw` golden import → on-disk read, 16 stale provenance
> paths), proved provenance (golden `sha256:9cfdc399…c93cd` identical in both
> copies; `diff -rq` clean against `workbench/.../src/core/`; 22 donor tests
> compare every shader module to the on-disk golden), **settled T06 in the
> governing artifact** (spec v0.2.4 `renderer_boundary` now states layer
> composition explicitly; plan v0.2.5 re-bound; ADR-006 dated addendum;
> decision log D-2026-09-22-T06-01) — no requirement ID added, validator PASS
> 86/86 — then integrated Gargantua as the **canonical persistent CORE**: new
> `src/host/scene/coreVisualIntent.ts`, `CoreLayer.tsx`, `CoreLayer.module.css`;
> `App.tsx` mounts CORE as the deepest layer; `SceneCanvas.tsx` is now
> alpha-composited above it and no longer draws the system-scale
> `CoreObject`/`AccretionSwirl`. **GATE_UI PASS: `just casework-ui-check` —
> frozen install, typecheck, production build, 140 tests / 823 assertions / 23
> files.** Honest gaps: **no GPU here, so there is NO rendered acceptance** (the
> four T06 perceptual states are unrecorded); `CoreObject`/`AccretionSwirl` files
> are not yet retired (acceptance-gated, and `LocalHomeAnchor` still
> uses `CoreObject` as a local marker); donor drag-orbit is unreachable under
> the R3F canvas; reduced-motion for CORE unimplemented; `workbench/` NOT
> deleted (GATE_REMOVAL activates at T14). Nothing committed; all pre-existing
> dirty/untracked work preserved.

> **2026-09-22 GARGANTUA WHITE-LABEL CORE MIGRATION HANDOFF:** The Workbench
> product shell was replaced around the supplied Gargantua single-file WebGL
> implementation by surgical extraction (copy → preserve → separate →
> relocate → wrap → map → integrate), not reimplementation. Golden reference
> `workbench/apps/desktop/public/reference/gargantua.html` (sha256
> `9cfdc399…c93cd`, verified identical at copy). `src/core/` holds the donor
> renderer (verbatim shaders/camera/quality/pipeline + narrow API);
> `src/spatial/`, 9 `src/surfaces/`, `src/projections/`, persistent
> `Composer`, dev-only `CoreTuningPanel`, and `docs/CORE_ARCHITECTURE.md`
> establish the Surface/Object/Relationship/Focus/Zoom/Time grammar with Home
> as canonical CORE state. Legacy SaaS chrome removed (Sidebar, GlobalHeader,
> JourneyRibbon, mockup kit); all pages/hooks/guards/contracts retained and
> composed as surfaces. Gates: workbench renderer frozen-install + typecheck +
> lint + production build + **178 tests** PASS, ui-components 17 PASS,
> `workbench-contracts-gate` PASS, tauri `cargo fmt --check` PASS. Honest
> gaps: GPU side-by-side vs golden pending a GPU host (shader byte-parity
> tests hold); Tauri host build/tests unrunnable here (pre-existing tracked
> self-symlink `target` breaks all root cargo builds + cold compile exceeds
> budget; `src-tauri` untouched); Rust kernel gates not run (same pre-existing
> blocker). Nothing committed. Preserve all unrelated dirty/untracked work.

## Objective

Continue the died-mid-turn opencode session by making Gargantua's raymarched
black hole the **canonical persistent CORE** of the React cognitive
environment, with the required R3F/Three/Drei scene retained for what it is
good at — objects, relationships, spatial projections, interaction geometry —
composited above it, and DOM surfaces above that. Repair the broken relocation
first, prove donor provenance, and settle the renderer composition in the
governing spec/ADR before integrating. Retire the superseded R3F CORE
(`CoreObject`, `AccretionSwirl`) only once integration *and* acceptance hold;
do not maintain them as an alternate or fallback CORE.

## Worktree State

Worktree carries this work uncommitted, on top of a large pre-existing dirty
set that is preserved untouched (staged Workbench removals, unstaged
`workbench/` sources, `.agents/` evidence/plan/spec edits from other efforts,
and two stray opencode session exports
`.agents/.session_export_f38e87{,.clean}.json` — the `.json` is 0 bytes).
New this work: `apps/godspeed-cognitive-ui/src/host/scene/{coreVisualIntent.ts,
coreVisualIntent.test.ts,CoreLayer.tsx,CoreLayer.module.css}`; modified
`src/host/App.tsx` and `src/host/scene/SceneCanvas.tsx`; the relocated donor
tree `src/host/gargantua/**` was repaired in place. Governing artifacts
changed: spec v0.2.3→0.2.4, plan v0.2.4→0.2.5 (re-bound), ADR-006 addendum,
decision log entry. Tracked self-symlink `target` (pre-existing, committed)
still breaks root cargo builds; left as-is. `workbench/` deliberately NOT
deleted.

## Changed Files

- Added: `apps/godspeed-cognitive-ui/src/host/scene/coreVisualIntent.ts`
  (the only application-state → CORE-renderer path; pure and unit-tested).
- Added: `.../scene/CoreLayer.tsx` + `CoreLayer.module.css` (persistent CORE
  mount; deepest layer, `pointer-events: none`).
- Added: `.../scene/coreVisualIntent.test.ts` (12 tests).
- Changed: `.../src/host/App.tsx` (mounts `CoreLayer` before `SceneCanvas`),
  `.../src/host/scene/SceneCanvas.tsx` (alpha canvas, `clearAlpha 0`,
  `scene.background = null`, fog kept; system-scale `CoreObject` +
  `AccretionSwirl` removed from the render path).
- Repaired in place: `apps/godspeed-cognitive-ui/src/host/gargantua/**`
  (import depth, deduped `CoreVisualProjection`, `bun:test` conversions,
  on-disk golden read, 16 provenance-header paths, `PROVENANCE.md`).
- Governing artifacts: the casework spec (v0.2.4), the casework plan (v0.2.5,
  re-bound to the new spec hash), `docs/decisions/ADR-006` (2026-09-22
  addendum), and the casework decision log (`D-2026-09-22-T06-01`).
- Not changed: `workbench/**` and every pre-existing dirty file from other
  efforts; `workbench/` deletion is held for T14/`GATE_REMOVAL`.

## Completed

Phase 1 (relocation repaired: GATE_UI restored from RED); Phase 2 (T06 settled
in the governing artifact — spec `renderer_boundary` now states the CORE/R3F
layer composition explicitly, plan re-bound, ADR addendum, decision logged,
validator PASS 86/86 with no requirement ID added); Phase 3 (Gargantua
`CoreViewport` is the canonical persistent CORE beneath a transparent R3F
layer, driven only by projected visual intent; the system-scale R3F CORE was
removed so exactly one CORE renders).

## Verification

- `python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py`:
  PASS (spec hash, 86/86 requirements, 15-task DAG, gate activation, UI
  removal) — run at baseline, after the spec/plan rebind, and at the end.
- `just casework-ui-check`: **PASS** — frozen install (no changes), `tsc
  --noEmit` clean, `vite build` clean, **140 tests / 823 assertions / 23
  files** (was 128 before this work; 106 before the relocation).
- Donor provenance proven: golden reference
  `sha256:9cfdc399c3fdf5fc73cbaf0545ef2fa7a49b4d6e9f8772a817949b10883c93cd`
  identical in both copies; `diff -rq` between `workbench/.../src/core` and
  the ported tree reports no differing files; the 22 donor tests read the
  golden file from disk and compare every shader module plus the preserved
  numerical contracts, camera orbit, and quality hysteresis.
- Rendered acceptance: **NOT RUN** — no GPU in this environment, so the
  integrated composition has never been drawn.
- Rust/Go gates: not run for this work (GATE_UI is the applicable gate for a
  UI change; Rust compilation is resource-deferred and pre-existing broken by
  the tracked `target` self-symlink).

## Remaining

- Rendered acceptance of the integrated composition on a GPU host: the four
  T06 perceptual states (quiet system/Core world; salient work object among
  quiet objects; focused case with Core reduced to home/orientation; deeper
  semantic zoom), including camera travel and the near-pure-white opening.
- Then retire `src/host/scene/CoreObject.tsx` and `AccretionSwirl.tsx` (and
  decide the `LocalHomeAnchor` marker's replacement) — acceptance-gated, not
  before. Do not keep either as an alternate or fallback CORE.
- Donor drag-to-orbit / wheel-to-CORE interaction routing under the R3F canvas.
- Reduced-motion handling for CORE (would be a new documented donor deviation).
- Formal frame cost for two live WebGL contexts.
- `workbench/` removal remains gated on T14/`GATE_REMOVAL`.

## Blockers

- No GPU in this environment: WebGL paths execute only under failure-surface
  tests, so no rendered evidence can be produced here.
- `target` tracked self-symlink (pre-existing, committed): blocks every root
  cargo recipe. Operator decision required; untouched.
- `workbench/` deletion explicitly held: the plan removes superseded UIs only
  after replacement parity and real integration (`GATE_REMOVAL` @ T14), and
  `resource_deferred_mode` forbids removing old UIs in that mode.

## Decisions

- The persistent CORE renderer is a GodSpeed-owned Three.js renderer with its
  own WebGL context, not an R3F scene graph; R3F keeps objects, relationships,
  projections and interaction geometry above it. Settled in spec v0.2.4
  `renderer_boundary`, ADR-006 (2026-09-22), and
  `D-2026-09-22-T06-01`; no requirement ID added or removed.
- Application state reaches CORE only through `coreVisualIntent.ts`, and CORE
  reports nothing back. Mass/spin stay at donor defaults; activity moves temp
  only within ±0.05 and bloom within ±0.15; semantic zoom is recorded as depth
  and deliberately does not drive camera radius (that would be the
  geometric-only zoom REQ-UI-002 rejects).
- Framing is asserted on focus transitions only, so CORE does not fight the
  user's orbit or the donor's idle drift.
- The relocated donor code was repaired mechanically, never rewritten; the
  move added no dependency (`three@^0.186.0` was already present).

> **2026-09-21 GODSPEED COGNITIVE WORLD INTEGRATION HANDOFF:** The donor-based
> replacement world is integrated and the latest `just casework-ui-check`
> passes frozen install, TypeScript, production build, and **106 tests**.
> Architect-inspected renders now show a recognizable 3D black hole with
> lensing and a slow inclined accretion disk, shaded celestial CognitiveObjects,
> clean desktop/390px/320px openings, camera travel, local Core home anchor,
> causal comparison, historical projection/comparison, role actions, and a
> bounded artifact that expands to readable DOM and restores its exact context.
> The canonical frozen Casework mock adaptation supplies role-scoped actions,
> subscribed execution updates, object-distinct history, and artifact
> provenance; it stays visibly labeled `fixture`, grants no SEA authority, and
> refuses stale/unauthorized/unknown requests. CopilotKit exposes eleven typed
> semantic commands through `CognitiveEnvironment`; default and configured
> builds pass, but no live Copilot endpoint was available. Final supporting
> gates: casework plan validator PASS (86/86, 15 tasks), interface package 9
> conformance tests PASS, `casework-go-check` PASS, and `git diff --check` PASS.
> Remaining honest gaps: current Go demo wire shape is legacy and is not proven
> against the frozen rich contract or real SEA/Gauntlet authority; formal frame
> cost profiling and a live Copilot service demonstration remain unrecorded;
> the governing 15-task replacement/removal plan is not settled by this UI
> implementation. Preserve all unrelated dirty/untracked work. Execution and
> rendered evidence are tracked in
> `.agents/plans/godspeed-cognitive-world-execution.md`.

> **2026-09-20 GODSPEED COGNITIVE WORLD RESUMED AFTER USAGE RESET:**
> The user resumed implementation. The latest visual criterion supersedes the
> recent screenshots: Core must unmistakably read as a **3D black hole in space**
> with visible lensing and a **slow moving accretion disk**; surrounding
> CognitiveObjects must read as **3D celestial bodies orbiting Core**. The
> current `CoreObject` camera-facing shader and `ObjectNode` flat discs fail
> that criterion. A Terra Core rebuild has resumed but has no validated result
> yet. The focused layout pass also
> regression was repaired: `bun run typecheck` and the focused layout tests
> now pass. The UI build bottleneck was narrowed to optional CopilotKit loading;
> the build-time guard makes the default fixture build complete. Integrated
> `just casework-ui-check` passed frozen install, typecheck, production build,
> and 78 tests. It must be rerun after the current adapter/visual/a11y edits.
> Preserve all dirty/untracked work; do not reset or clean. The five-donor
> provenance report is at `.agents/reports/godspeed-casework-cognitive-environment/godspeed-cognitive-world-donor-provenance-2026-09-20.md`.
> A Luna StarDust adaptation from the selected galaxy donor is present, with
> one focused test passing. Terra completed a donor-based 3D Core rebuild;
> Luna completed 3D orbiting ObjectNode bodies. Rendered visual acceptance is
> still open because focused labels and edge bodies remain crowded. The
> frozen rich contract is not yet consumed by the current UI: a Terra agent
> is implementing its typed adapter and role-aware action seam. A Luna agent
> is fixing Recap/Clear contrast (axe observed 3.52:1), and another is
> refining focused-scene framing. The
> user explicitly prefers copying coherent licensed donor source and tailoring
> it, with exact revision/source attribution and retained notices. Racing-game
> mechanic integration remains pending until its new owner verifies it.
> Verified this turn: pinned-Go `just casework-go-check` PASS; casework plan
> validator PASS; frozen interface contract Bun test 9 pass; browser opening
> audit 0 axe violations and no runtime errors before the latest edits. Camera
> journey tests passed 11 focused cases and settled Core return was visually
> observed; new visual criterion keeps its scene acceptance open.

> **2026-09-20 GODSPEED CASEWORK CONTRACT PARITY RESOLVED:** Spec v0.2.3 and
> plan v0.2.4 are bound to SHA-256
> `09d4292b956b09066b84bb03014413cae2b14a2c89f5338b34b6ebf542ab1dae`;
> the ADR addendum remains recorded. The casework validator PASS and focused
> interface contract Bun conformance test (9 pass) are recorded; 86 IDs across
> 15 tasks remain unchanged. The interface package now agrees across prose,
> TypeScript, JSON schemas, examples, and tests on the string-cursor snapshot
> shape and 15-name wire action union. Structural conformance is proven; full
> draft-2020-12 JSON Schema semantic validation remains an explicit limitation.
> The authority correction is complete and no longer blocks UI work. Prior valid
> work is preserved; visual acceptance remains open. Core donor provenance/render
> verification and the camera journey are pending, and T06 visual acceptance is
> not claimed.

> **2026-09-20 GODSPEED CASEWORK NEXT MOVE:** Continue with evidence-led UI
> verification now that the authority gate is clear. Verify retained donor
> revisions/licenses/provenance, inspect the combined first frame, then run the
> rendered Core and camera journeys. Keep Core and camera acceptance open until
> those artifacts and renders are reviewed.

> **2026-09-19 GODSPEED — T28 SETTLED (P2, CONFIRMED WITH QUALIFICATION; FRESH
> VERIFIER: CONFIRM): T27 VALIDLY EXECUTED AND REPRODUCED; REPRESENTATION
> BRANCH CLOSED ON THIS SURFACE; PLAN REACHES EMPTY READINESS.**
> A fresh independent verifier (shielded from all builder narrative and the
> preferred outcome) plus an orchestrator-standalone recomputation importing
> nothing from the T27 harness BOTH reproduce every number exactly: A 9/14
> (0.643) vs B 8/14 (0.571); corrections 0; regressions 1 (accepted-A4
> decided→abstain); c_ctrl 0; McNemar exact p = 1.0; C5(boundary)
> mechanically required (C3 defeated by the single frontier regression;
> under the superseded decided-only rule the label would be C3 — equally
> non-supporting). Integrity proven end-to-end: 14/14 corpus hashes,
> selection-rule replay MATCH, 140/140 store keys bound to the prereg
> file-bytes hash, 70/70 condition-A prompt bytes and 14/14 condition-B case
> hashes reproduced from the frozen specs alone, freeze timing proven
> (prereg 11:38:04 < addendum 11:41:48 < harness 11:47:34 < first call
> 11:49:19), harness = recorded sha, addendum anti-self-serving. Claim
> separation held: behavior shift CONFIRMED (per-call abstentions 18/70 →
> 32/70 — typed structure made pre-terminal record status salient) WITHOUT
> judgment improvement. Stable-wrong `budget-C3`: the provider acknowledges
> supported claims and the truncated trace yet answers from local positive
> valence — the evidence-supported missing distinction is **local positive
> evidence vs sufficiency/completeness of the whole-case record for
> settlement**. Branch pruned: with T18's and T27's independent negatives,
> input-representation provenance typing is closed as a fix on this surface.
> Qualifications recorded append-only, none verdict-changing: frozen
> baseline fraction label 4/14 (actual 6/14 = 0.4286; decimal was correct);
> prereg strict-YAML parse defect in relation_rules (frozen text/hash
> intact, terms independently verified); teeth log is a post-verification
> rewrite artifact. Architecture boundaries held (no provider truth/
> settlement/authority; no `.sea` grammar implication). Next affordable move
> is an OPERATOR DECISION — nothing executed: zero-provider-cost inspection
> of the budget-C3 vs accepted deterministic settlement pattern first; a
> preregistered settlement-sufficiency representation probe (required vs
> observed vs missing evidence coverage) is the one candidate the evidence
> pays for, only if a new developmental experiment is later authorized.
> Evidence: `.agents/evidence/godspeed-bounded-judgment/T28/` (disposition
> report with the full 16-item audit, verifier confirmation record,
> standalone recompute script + output, `verify_disposition.py` gate —
> PASS). Decision D-2026-09-19-T28-01. Nothing committed.

> **2026-09-19 GODSPEED — T27 SETTLED (P2, DISPOSITION C5): CLAIM-VS-OBSERVATION
> TYPING DOES NOT CHANGE CASE-LEVEL BOUNDED JUDGMENT ON THE FRONTIER; TARGET
> LYING-CLAIM CASE STAYS STABLY WRONG; NOTHING PROMOTED.**
> Executed exactly under the frozen prereg (base `41af1921…74343` + append-only
> addendum-1 `a7fb31e2…7609` frozen before any call; the addendum corrected a
> pre-execution analysis defect so discordant pairs score abstain as
> not-correct instead of excluding the frontier's abstention channel).
> Corpus 14/14 hash-verified from the persistent T16 expanded corpus:
> frontier = the 1 stable-wrong case (budget-C3) + all 9 single-call unstable
> cases; controls = accepted-A1/A2, forge-D1, lying-B1 by the frozen
> deterministic stratified rule. 140/140 fresh DeepSeek-v4.1-Flash calls
> (ClinePass seam, 5 reps × 2 conditions, interleaved A-then-B,
> persist-before-score); all four teeth PASS with simulated attacks rejected
> (content-equivalence atom sets, role vocabulary, temporal horizon,
> case-identity binding); store keys verified 140/140; `--verify-only`
> recomputation identical. RESULT: case-level accuracy A 9/14 (0.643) vs B
> 8/14 (0.571); corrections 0; regressions 1 (accepted-A4, an abstention
> shift on an already-unstable case); control regressions 0; exact paired
> McNemar two-sided p = 1.0; practical criterion unmet; budget-C3
> unchanged-wrong under BOTH representations. Mechanically C5 (validity
> passed; C4/C1/C2 unmet; C3's zero-movement condition defeated by the single
> frontier regression → genuinely inconclusive), substantively consistent
> with NO case-level treatment effect. Exploratory: per-call abstentions rose
> 18/70 → 32/70 under B — a behavior shift without a judgment shift.
> CONSEQUENCE: H_REPRESENTATION_T27 not supported on this surface/provider;
> with T18's independent negative, the surface's judgment failure is not an
> input-representation failure; T06 stays blocked; no `.sea` grammar
> implication; T28 (independent confirmation/disposition) is ready and
> receives this frozen evidence — it did NOT participate in producing it.
> Evidence: `.agents/evidence/godspeed-bounded-judgment/T27/` (settlement
> report, paired report, 140 raw observations, teeth log); decisions
> D-2026-09-19-T27-01 … -05 (incl. -04 append-only harness-hash correction).
> Nothing committed.

> **2026-09-19 GODSPEED CASEWORK ENVIRONMENT DOCUMENT REVIEW:** The former
> 2,090-line spec did not parse as YAML, and the plan named absent Go/adapter
> substrate and wrong spec/status paths. The corrected authoritative spec
> preserves the 84 original requirements and adds two mandatory UI-removal
> requirements. Plan v0.2.2 maps 86/86 across T00–T14, binds the spec hash,
> separates existing from future gates, and makes real integration and parity
> precede removal of the SEA Forge Tauri GUI and Gauntlet TUI. ADR-006 records
> the architecture direction; findings are in
> `.agents/reports/2026-09-19-casework-cognitive-environment-adversarial-review.md`.
> Validator PASS; this is document settlement
> only—no Go/React implementation, UI removal, or product conformance is
> claimed. Parallel-work initiation now requires dedicated SEA Forge and
> Gauntlet worktrees, each with its own canonical handoff files. T00 may defer
> Rust baselines for insufficient memory; Go/React work can proceed, while
> Rust-gated tasks remain unsettled until their gates pass. Next move: transfer
> the reviewed artifacts into an isolated worktree and execute T00 inventory.

> **2026-09-19 GODSPEED — T29 BREADTH EXPERIMENT SETTLED (P2, ADVERSARIAL
> VERDICT: CONFIRM): MECHANISM SURVIVES A MATERIALLY DIFFERENT SECOND FAMILY
> IN CHARACTERIZE-AND-STOP MODE; d\*=2 DETECTION BOUNDARY; RM EVIDENCE-
> INTEGRITY CONFIRMED (OPTION A); DOCS UPDATED.**
> RM check: the 3 Muse substitutions were fallback-evidence only — arm1
> 15/15 pure DeepSeek; arm2 Muse rows (nat-02, nat-14) excluded from the
> DeepSeek slice (n=13, reported separately); arm2b Muse row (nat-03)
> excluded (n=14). No pooling; no recomputation needed (D-2026-09-19-RM-04).
> Plan v1.7.0: T29 added (depends_on T26). Family: e2e-forged-report x
> budget ladder (prereg e844fce4… frozen before any probe; rationale: fraud
> refusal vs honest exhaustion, detection-success vs settlement-success,
> passing-claim artifacts, COR-F1 gate; near-duplicate and new-scenario
> options rejected). 6 real probes, deepseek ladders, no substitutions.
> UP rule (bounded-non-settlement ∧ 0 settlements ∧ passing-claim artifact)
> recomputed from raw DBs by builder, orchestrator, and fresh adversarial
> verifier (104 own checks + 3 novel attacks refused): d\*=2 (fires
> {2,3,4,5,6}; d\*-1=1 produces zero dispatches/artifacts), matched negative
> single-factor (budget, target-file diff verified), G3 partition holds
> (UP 6/6 forged; addendum-3 4/4 lying; honest fire neither). Observer
> labels (2 calls) proven inert. Gate 618 checks PASS (orchestrator re-run);
> failed rounds preserved. EARNED: mechanism survival in
> characterize-and-stop mode; d\*=2 boundary + one-factor negative; rule
> complementarity. NOT EARNED: two-settleable-families breadth (no
> honest-completion scenario in the catalog) and all broader claims;
> learned judgment stays observational-only. docs/explanation/native-route-
> discovery.md updated (second-family result + bounded conclusion).
> ready_now=[T27]. Nothing committed. NEXT AFFORDABLE MOVE: operator
> decision — either accept the one-settleable-family scope and route
> breadth investigation to catalog extension (a decision ABOVE this
> branch's authority), or proceed to the claim-vs-observation T27 (ready).

> **2026-09-19 GODSPEED — NATIVE-CORPUS JUDGMENT REMEASUREMENT SETTLED
> (operator-directed, outside the settled plan): LEARNED LAYER ADDS NO
> INFORMATION OVER THE MECHANICAL LAYER; RECOMMENDATION = REMOVE FROM THE
> ROUTE-DISCOVERY DECISION PATH; DOCS INTEGRATED.**
> Stage 1 (documentation): earned route-discovery structure placed as a new
> Layer-5 explanation `docs/explanation/native-route-discovery.md`
> (registered in `docs/documentation-map.md`), cross-referenced from the
> canonical `docs/explanations-and-references/goodspeed-loop.md` at the
> changed-navigation slot; evidence-backed / inference / interpretation /
> open-question distinctions and evidence links preserved; context-check
> PASS. Stage 2 (remeasurement, prereg 8142c0a5… frozen before any scored
> call, corpus manifest frozen RM-02, 15 native rows, mechanical truth):
> arm1 exact-T16 predictive comparability — learned 0.600 CI [0.323, 0.837]
> = face-value baseline 0.600, McNemar p=1.0; arm2 enriched-surface
> discrimination — single 0.692 CI [0.386, 0.909], majority-of-3 identical
> (agreement 0.974: errors are SYSTEMATIC anchoring on terminal_class, not
> sampling noise — aggregation refuted), honest 9/9 vs adversarial 0/4
> (all 4 verification_failed rows collapse to budget_exhausted while the
> surface visibly contains failures:1 and 0 settlements); arm2b exploratory
> graded probe: MAE 0.0 but pure copy-fidelity of a surface field. Payment:
> 78 learned calls / 3217.8s wall vs 0 calls for the mechanical layer.
> ARCHITECTURE INTERPRETATION STANDS (LLM proposes/interprets; governed
> execution produces consequence; deterministic machinery preserves
> evidence; authority external; settlement decides). T21-T26 untouched;
> T18 not reopened; T19/T20 not authorized; T06 stays blocked. Durable
> report: `.agents/evidence/godspeed-bounded-judgment/remeasure-native-
> corpus/remeasure-report.md` (gate exit 0; D-2026-09-19-RM-01/-02/-03).
> NEXT AFFORDABLE MOVE (evidence-following): learned judgment removed from
> the route-discovery decision path (it never gated settlement); keep it
> only as an optional instrumented probe; second-operation-family route
> discovery is the natural next experiment (breadth, mechanical
> discrimination authoritative), plus the recorded 12-call revised-surface
> variant-B idea as a cheap future probe — operator decides.

> **2026-09-19 GODSPEED — NATIVE ROUTE-DISCOVERY BRANCH COMPLETE (T21..T26 ALL
> SETTLED): T26 FRESH ADVERSARIAL CONFIRMATION VERDICT: CONFIRM — CLAIMS A AND
> B EARNED, CLAIM C MEASURED; NO PROMOTION; NOTHING COMMITTED.**
> The developmental branch executed its full frozen arc through real
> consequence: T21 froze the contract (prereg 1fb55d9c… + three append-only
> addenda, each frozen before the round that consumed it); T22 executed one
> real loop advance and repelled authority teeth T1/T2/T5; T23 settled route
> t23-route-001 on 12 REAL sequential gauntlet probes (baseline e2e-happy@4
> settled with 3 settlement_committed; boundary r\*=4 with r\*-1=3 swept on
> the route's own ledger; discriminator separated by the frozen addendum-3
> mechanical rule, 4/4 lying vs 0/8 honest) — independent confirmation
> CONFIRM; T24 inverted the settled route into case t24-case-001 whose
> independent replay genuinely re-settled (freeze-before-replay proven;
> impostor tooth T7 rejected) — CONFIRM; T25 recorded the single-factor
> matched negative (4→3 honestly budget_exhausted), the honest payment table
> (7 branch decision entries / 37 provider calls / 22 executions / 3 failed /
> 406s wall; unobservables n/a), and novelty NOVEL vs the T15/T16 grids —
> CONFIRM; T26 (fresh adversarial verifier, own 892-check gate, 9 attacks
> incl. a novel artifact-laundering forgery) returned VERDICT: CONFIRM with
> claims adjudicated separately and the addendum 1→2→3 history adjudicated
> PRINCIPLED append-only correction (post-hoc construction of addendum-3
> disclosed and recorded as the branch's weakest epistemic link, mitigated —
> settlement also passes under the base wording). FIRST-CLASS PRESERVED
> FINDING: on the enriched surface with neutral prompts, the learned judge
> separated only 1/4 lying probes — consequence-level discrimination is
> mechanically unambiguous but learned-label reliability at this setting is
> weak. Model output never gated settlement (goal evaluation is mechanical);
> judgment stayed evidence for selection. SCOPE: one fixture surface, mock
> replay runner, N=1 route; CLAIM D (promotion) NOT claimed and NOT
> adjudicated; T06 stays blocked; T19 stays not-authorized. Nothing
> committed. NEXT AFFORDABLE MOVE: point the frozen loop's freshly generated
> consequence-backed rows (boundary sweep @5/@6/@7 settled + @1/@3 exhausted,
> lying family at new budgets, accepted case + matched negative) at a
> T16-style remeasurement as fresh judgment-corpus variation, and/or run the
> loop on a second operation family (e2e-forged-report) to test route
> breadth — after the operator decides; also adopt the mechanical-not-
> judgment-mediated goal-evaluation lesson in any future loop prereg.

> **2026-09-18 GODSPEED — T22 SETTLED (P2): ONE REAL ROUTE ADVANCE END-TO-END,
> AUTHORITY TEETH REPELLED; PREREG ADDENDUM 1 FROZEN (FIXTURE-MECHANISM
> CORRECTION); NOTHING COMMITTED.**
> Builder B1 executed the frozen loop for real: deepseek-clinepass generated
> 3 schema-valid candidates from the bounded 24-op neighborhood (raw preserved);
> frozen admission admitted all 3; 3 SEQUENTIAL REAL gauntlet probes returned
> typed terminals from gauntlet's own event log (state DBs pinned
> path+sha256 under $HOME/.local/share/godspeed-route-discovery/T22/);
> bounded judgment over observation records only labeled all 3
> budget_exhausted; selection-policy-v1 retained exactly one move (e2e-happy@7
> boundary probe) with bijective correlation joins; goal honestly NOT met at
> depth 1. Teeth repelled as expected: T1 authority bypass (run_shell /
> prime-agent / shell_plan injections denied at admission, never executed),
> T2 model override (fabricated maximal-confidence settled_accepted changed
> nothing), T5 correlation swap (joinability check fails the swap).
> verify_loop.py PASS (orchestrator re-run). Round-1 failure PRESERVED: the
> base prereg had frozen demo-calculator targeting from T15's preserved
> FAILED round script; T15's clean corpus manifest actually records
> tests/fixtures/target copies + deterministic clock + per-case id seed
> ("the repo's own e2e_full_loop_mock recipe"); on demo-calculator copies
> nothing can settle at any budget (instrument failures=1) — the baseline
> goal was unreachable by construction. Smallest falsified layer =
> probe_execution (fixture mechanism); loop/admission/judgment/selection NOT
> implicated. Prereg ADDENDUM 1 frozen append-only (sha256 c472d1b6…0cea8,
> D-2026-09-18-T21-03) BEFORE any T23 probe: fixture-target recipe, baseline
> = e2e-happy@4, boundary over {1..8}, discriminator unchanged; all other
> frozen terms stand. ready_now = [T18, T23]. Nothing committed.

> **2026-09-18 GODSPEED — T21 SETTLED (P1): ROUTE-DISCOVERY CONTRACT+FIXTURE
> FROZEN AND SELF-PROVEN; GATE 67/67; NOTHING COMMITTED.**
> Builder B1 authored the frozen contract under .agents/evidence/godspeed-
> bounded-judgment/T21/ (route_contracts.py typed records with named refusal
> reasons and lossless round-trips; admission.py classify() with fixed
> precedence; selection_policy.py = selection-policy-v1 implementing the
> prereg eligibility+ordering exactly, no provider capability; operations_
> catalog.yaml 24 run_case operations with derivation provenance; verify_
> contract.py gate). Orchestrator re-verified personally on the final tree:
> gate PASS 67/67 (15 schema-refusal vectors incl. shell-plan and fabricated-
> mechanism refusal; 6 admission vectors; 8 selection vectors incl.
> ineligible-but-judgment-preferred NOT retained, midpoint tie-breaks, r*-1
> priority; fail-closed manifest hash check; prereg sha256 1fb55d9c…8ea8b5
> match), `just ruler` PASS, scope audit clean (only T21/ added). Nine
> interpretive readings recorded and reviewed, all faithful-to-restrictive.
> No route/probe/provider execution occurred (that starts at T22). ready_now
> = [T18, T22]. Nothing committed.

> **2026-09-18 GODSPEED — PLAN v1.5.0: NATIVE ROUTE-DISCOVERY DEVELOPMENTAL
> BRANCH ADDED (T21..T26); T21 PREREGISTRATION FROZEN; VALIDATOR PASS;
> NOTHING COMMITTED.**
> Operator-directed amendment branching from the settled T16 developmental
> result (separate from T06 and from the paused epistemic-role branch
> T18..T20, whose files/corpus/state are untouched). The branch tests whether
> GodSpeed can autonomously construct fresh, genuinely settleable routes over
> an existing governed operation surface and invert a settled route into a
> fresh consequence-backed case at lower payment than manual construction —
> reusing ONLY existing mechanisms: the T15 case-execution mechanism (real
> headless gauntlet runs over disposable demo-calculator copies with the
> deterministic mock replay runner; typed terminal consequences as ground
> truth), the frozen T15/spec answer-domain vocabulary, and the frozen T16
> provider seam (deepseek-clinepass primary, one muse substitution, declared
> deterministic fallback). Frozen bounds: K=3 candidates, 5 retained moves,
> 12 probes, 6 iterations, route goal = baseline settled_accepted +
> discriminator verification_failed_nonsettlement + minimal settling budget
> boundary r*/r*-1 discovered on the route's own ledger. Tasks: T21
> contract+fixture freeze (prereg sha256 1fb55d9c…8ea8b5 recorded in the
> decision log BEFORE any execution) → T22 one real loop advance + authority
> teeth (T1/T2/T5) → T23 multi-step route settlement + teeth (T3/T4/T6) →
> T24 settlement-first inverse case generation + independent replay + tooth
> T7 → T25 matched negative (single factor r*→r*-1) + payment/novelty
> measurement + tooth T8 → T26 fresh independent adversarial confirmation of
> claims A/B/C separately (D promotion explicitly out of scope). Model output
> is evidence, never authority: it may propose/classify/compare/estimate/
> abstain; it can never authorize, settle, or override deterministic
> composition. Probe run state persists under
> $HOME/.local/share/godspeed-route-discovery/ (T18 /tmp-wipe lesson). No
> product/spec/gauntlet-source change; no ToolGrad dependency; validator PASS
> (27 tasks, 42 edges, acyclic, ready_now=[T18, T21]); nothing committed.

> **2026-09-18 GODSPEED — T16 PERMUTATION ORACLE CORRECTED (operator-caught
> inconsistency verified and fixed); DEEPSEEK LEG D-QUALIFIED WITH CORRECTED
> EXACT p = 2.5e-6; PRIME RESUME AUTHORIZED BUT QUOTA-BLOCKED ~4.25h.**
> The operator's caught inconsistency was real: the N=25 exact tooth counted
> settled-truths-on-not_settled-answers as matches, producing the impossible
> 0/5,200,300 with null max 0.48. The faulty artifact is preserved
> (`t16-state.round5-faulty-permutation.yml`; the D-2026-09-18-T16-13
> 'p < 2e-7' claim is superseded). Corrected exhaustive enumeration
> (invariants 1–4 all passing, independent reconstruction): **total
> 5,200,300; observed 20/25 reproduced by the original labeling; exact max
> 20/25 achieved by 13 assignments; count(≥obs) = 13; exact p =
> 13/5,200,300 ≈ 2.5e-6** — matching the operator's analytical cross-check
> exactly. Design wording corrected (the null MAX ≈ the provider's own
> accuracy; what shrinks with N is the upper-tail ratio 13/5.2M vs 4/84 —
> the N=25 choice satisfied its purpose independently of the wording
> defect). **Substantive result survives: DeepSeek D-bar PASS (0.800 ≥ 0.8);
> corrected discrimination tooth PASS (2.5e-6 ≪ 0.05); N=25 sufficient for
> the DeepSeek leg.** Prime resume is AUTHORIZED but its upstream codex-plus
> window is quota-blocked until epoch 1789783016 (~4.25h). T16 remains
> in-flight with the full resume procedure recorded
> (`expanded-round/t16-state.yml`; decisions D-2026-09-18-T16-14/-15).
> No provider responses, corpus, scoring, rule, or prior evidence were
> changed. Validator PASS; context-check PASS; nothing committed.


> **2026-09-18 GODSPEED — T16 EXPANDED MEASUREMENT (N=25): DEEPSEEK LEG
> COMPLETE AND D-QUALIFIED (EXACT PERMUTATION p < 2e-7); PRIME LEG QUOTA-
> BLOCKED; T16 IN-FLIGHT AWAITING THE PRIME WINDOW.**
> (1) **T15/T16 reconciliation:** T15 legitimately settled (projection was
> stale) — `settled=[T00–T05, T15, T17]`. (2) **Exact permutation tooth
> (N=9):** DeepSeek observed 0.667 = the maximum over all 84 class-preserving
> permutations (p = 4/84); Prime p = 34/84 — the corpus could not
> discriminate; recorded as the task-level failure per the tooth's frozen
> declaration. (3) **Expansion (frozen addendum `09169eea…`):** design
> analysis (closed-form hypergeometric null) showed separation at p < 1e-5
> for N ≥ 16; N_total = 25 frozen (12 settled / 13 not_settled); 16 new
> consequence-backed runs generated through real Gauntlet mechanisms
> (e2e-happy ×9; lying ×3; budget ×2; NEW forged-report ×2, smoke-verified);
> expanded manifest `9bc65fb9…`; zero problems. (4) **Expanded measurement:**
> **DeepSeek leg COMPLETE and D-QUALIFIED**: 0 contract failures, 25/25
> answered, accuracy **0.800 = the frozen D-bar exactly** (settled 12/12;
> 4 responsible abstentions on not_settled rows; 1 lying-claim error),
> entropy 0.828 nats, and the **exact permutation tooth over all
> 5,200,300 class-preserving placements: p < 2e-7 (null max 0.48)** — the
> measurement now discriminates absolutely. **Prime leg environmentally
> blocked** (24/25 quota refusals; 5-hour window). The mechanical B letter
> is superseded as environmental per the recorded precedent; the frozen D
> rule requires BOTH providers, so **no outcome letter is issued yet** —
> T06 stays blocked, and the claim-vs-observation representational
> hypothesis is recorded as a SEPARATE future experiment. (5) One incident
> recorded: an edit-assertion failure caused a 9-row re-measure that
> overwrote round-1 files (fresh artifacts preserved as `.9row-rerun.*`;
> the round-4 numbers survive in the NOT_CONFIRM verdict's independent
> recomputation). Evidence: `T16/expanded-round/`; state:
> `t16-state.yml`; decisions D-2026-09-18-T16-08 … -13. Validator PASS;
> context-check PASS; nothing committed.


> **2026-09-18 GODSPEED — T16 RECONFIRMATION AUDIT: SHUFFLE TOOTH FAILS THE
> TASK AT N=9; T16 SETTLEMENT BLOCKED ON CORPUS EXPANSION; CORRECTED BASELINE
> = CONSTANT PRIOR AT 0.667.**
> Post-NOT_CONFIRM corrections verified: baseline now scored against the same
> mapped truth as providers (round-5 harness `289e0fbd…`) — the preregistered
> face-value rule degenerates to the constant not_settled predictor scoring
> **6/9 = 0.667** (DeepSeek ties it; Prime 0.556 is below it); provenance
> closed byte-exact (frozen `94f2d796…` reproduced by substituting the
> round-2 harness hash over the `SET-BELOW` placeholder — sole delta is the
> documented fill-in); entropy relabeled nats. **Shuffle tooth (Case A:
> required for settlement per its own "fails the task" declaration; frozen
> `130db4ef…` before its result): FAILED** — 500-shuffle permutation test:
> DeepSeek's observed 0.667 equals the shuffled maximum; Prime's 0.556 is
> inside the noise band (max 0.778, p95 0.667). At N=9 with a 3/6 split, the
> measured accuracies are indistinguishable from label noise. **T16 cannot
> settle while its own tooth declares the measurement non-discriminating.**
> Outcome C is reinforced a fortiori (no demonstrable signal at all); T06
> stays blocked; the capability-class question stays escalated. **The next
> discriminating move is now tooth-evidenced: expand the varied corpus (the
> frozen generator is reusable) until the shuffle test separates, then
> re-measure under the frozen family.** Provider-substitution record stands;
> original NOT_CONFIRM verdict preserved; rounds 1–4 preserved. Validator
> PASS (`settled=[T00–T05,T15,T17], ready_now=[T16]`); context-check PASS;
> nothing committed.


> **2026-09-18 GODSPEED — T16 MEASURED: OUTCOME C (FROZEN RULE); CLAIM
> NARROWED; T15/T16 STATE RECONCILED; CONFIRMATION OUTSTANDING ON T16.**
> (1) **T15/T16 reconciliation:** T15's contract is satisfied by existing
> evidence — T15 added to `settled_tasks`; the projection was the stale
> artifact. **T16 is now the DAG-ready/in-flight task** (`settled=[T00–T05,
> T15, T17], ready_now=[T16]`). (2) **Provider substitution (append-only):**
> quota-blocked Codex → **DeepSeek 4.1 Flash via ClinePass**
> (`cline-pass/cline-pass/deepseek-v4.1-flash`, discovered from `opencode
> models`; alias-level identity confidence recorded honestly); Prime-Agent
> unchanged (quota reset, smoke green); addendum
> `94f2d796…` records unchanged corpus/surface/domain/baseline/rule/teeth;
> material independence checked-as-far-as-observable with the prime-backend
> caveat; no anonymous "LLM" merging; the result is provider-specific.
> (3) **T16 measurement (round-4, first fully-correct; rounds 1–3 preserved:
> codex quota, then two harness/oracle defects — T01-vocabulary parser and
> unmapped terminal→domain truth labels — corrected per protocol):**
> contract 0 failures both providers; **face-value baseline 0.0 accuracy;
> DeepSeek 0.667; Prime 0.556**; entropy 0.831/0.388; disagreement 3/9;
> DeepSeek's abstentions landed only on budget rows; its single error was
> believing a lying claim; Prime misread two settled runs. **Outcome C as
> frozen** (the 0.8 D-bar unmet — applied without loosening): NO promotion
> for this surface; T06 remains blocked; the capability-class question stays
> escalated. Claim NARROWED per the mixed-evidence rule: careful record
> reading carries consequence-grounded information the trivial baseline
> lacks, but the promotion bar was not earned at N=9. (4) **T16 independent
> adversarial confirmation is OUTSTANDING** (conservative direction: nothing
> unlocks on C); resume procedure in `t16-outcome.yml`. Validator PASS;
> context-check PASS; nothing committed.


> **2026-09-18 GODSPEED CONTINUATION — T15 SETTLED (REAL VARIED CORPUS); T16
> BLOCKED ON PROVIDER QUOTA (transient); T05 CORRECTED; ALL SEA GATES GREEN.**
> (1) **T05 scope audit (operator-directed):** REQ-PROV-021 is universal, so
> T05 was over-settled on the `--plan` path — correction round executed
> (`plan_pipeline.rs` now cites committed decision ULIDs), verified on a real
> completing `--plan` run; agent-item delegations join cross-boundary by the
> deterministic `request_id`. (2) **Gate remediation (T17 executed):**
> SEA_CHECK / SEA_TEST (120 suites) / SEA_CI **all exit 0** — sfwp trio +
> t13_1 root-caused to stale pre-admission callers of the correct
> request-admission gate (deterministic `request_id`s added; server invariant
> untouched); masked clippy drift fixed; RUSTSEC-2026-0285 rustls bump;
> commit-scoped gitleaks allowlist for verified simulated fixtures. No test
> weakened. (3) **Headless affordance classified:** NO session transport was
> missing — the repo's own `e2e_full_loop_mock.rs` recipe (fixture target +
> its own check tool + `GAUNTLET_CLOCK=deterministic` + `GAUNTLET_ID_SEED`)
> drives genuine headless accepted settlements; earlier failures were
> generator targeting (wrong target/tool/wall-clock). No T18 needed; nothing
> bypassed. (4) **T15 SETTLED:** 9 clean consequence-backed runs through the
> real governed path — settled×3 (`settlement_committed` per unit,
> `run_terminal=settled`) + budget_exhausted×6 (3 lying-discrimination runs
> where the verifier ladder recorded FAIL against planted artifacts; 3
> genuine round-constraint cuts). verify_corpus PASS; manifest
> `b8550ad5…`; rounds 1–2 preserved. (5) **T16:** prereg + harnesses frozen
> (`d89bea89…`/`7df3dc8e…`); measurement round-1 attempted — both capable
> providers hit plan quota (429 usage_limit_reached, codex indicates ~11:38 AM
> reset). Classified ENVIRONMENTAL (deliberately NOT outcome B); resume =
> re-run the unchanged frozen harness after quota reset, apply the frozen
> A–E rule, obtain independent confirmation. Validator PASS
> (`settled=[T00–T05,T17], ready_now=[T15 in-flight on quota]`); context-check
> PASS; nothing committed.


> **2026-09-17 GODSPEED BOUNDED-JUDGMENT — FULL EXECUTION PASS COMPLETE: 6/15
> TASKS SETTLED (T00–T05), REMAINDER BLOCKED BY EVIDENCE OR PHYSICS.**
> Operator-directed whole-plan execution. **T01 PROOF-1: outcome C, then the
> preregistered second-surface evaluation ALSO failed authorization** (both
> shapes single-class baseline on this 17-row corpus; capability-class question
> ESCALATED per REQ-JUDG-023 — the corpus supports neither promotion nor a
> class-level rejection). **T02:** Gauntlet CI authored (`just ci` only;
> uncommitted — activation on operator push); lint tooth verified.
> **T03:** the red `conformance_run_locator` gate repaired — the per-cell
> flock invariant proven correct, the test now models a real restart
> (assertions untouched, no production change); three masked sfwp failures
> recorded PREEXISTING_BASELINE_FAILURE. **T04 (P3, CONFIRM):** authority
> non-bypass proven adversarially — strongest allow-shaped provider output
> under a denying policy yields Rejected with ZERO provider contact; all
> failure modes settle Rejected with named typed classes; the independent
> verifier added seven new attacks (incl. engine-level vocabulary minting and
> wall-clock dispatch-ordering proof) and found no widening case.
> **T05 (P3, CONFIRM):** loop correlation identity BOUND — the ledger causal
> chain with the admission intent ULID; the one broken hop (settlement cited
> per-run labels) repaired; joinability 8/8 (collision, duplicate id, restart,
> missing-hop, stale-mapping, mismatched-pair, VAR-008 immutability); the
> verifier's own from-scratch attacks achieved no false join. **Blocked:**
> T06–T12 (judgment-slice escalation — re-open with a larger varied corpus
> re-run through the frozen harness family, or explicit operator override);
> T12 also physically blocked (no Jetson reachable); T13 needs T07; T14 needs
> T12+T13. All evidence under `.agents/evidence/godspeed-bounded-judgment/`,
> decisions D-2026-09-17-T01-01 … T05-05. Nothing committed (authorization not
> given). Readiness is mechanically empty: no task is executable.

> **2026-09-17 GODSPEED BOUNDED-JUDGMENT T01 (PROOF-1) SETTLED — OUTCOME C;
> FRESH INDEPENDENT ADVERSARIAL CONFIRMATION: CONFIRM; READY = T02–T05; T06
> BRANCH-BLOCKED.**
> Executed read-only against the frozen 16-run Gauntlet corpus (manifest
> `6738d5a8…`, 632 evidence files hashed; prereg frozen `c44dcef8…` before the
> first execution; corrections rounds 1–8 preserved in the decision log — six
> mechanical harness defects found and fixed pre-provider, plus one custody
> bug the verifier caught, repaired with the failed-round bytes restored).
> Results (all disclosure-gated, pilot terminology): **history reconstructs**
> — all five chain hops present 17/17, wall-clock temporal integrity ok 17/17;
> **bounded output contract perfect** — codex 17/17 and prime-agent 17/17
> in-domain, 0 schema/out-of-domain/provider failures; **outcome C** — the
> recorded deterministic baseline is single-class `PASS` (outside the frozen
> domain; 0 pairs by the no-coercion rule), so incremental information is NOT
> establishable on this surface; codex abstains 16/17 (mean entropy 0.079
> bits); capable-vs-capable disagreement 4/17 (prime-agent = corpus-producer
> lineage, disclosed confound). Label-shuffle attack honestly recorded
> VACUOUS_NO_BASELINE_VARIANCE. **Consequence (binding): no promotion for
> this surface; no capability-class conclusion; T06 authorized only after the
> preregistered second-surface evaluation (multiclass chain-deficiency shape)
> or an operator re-routing decision.** `../gauntlet` unmodified (648 files
> watched, 0 drift; verifier-confirmed). Evidence:
> `.agents/evidence/godspeed-bounded-judgment/T01/`; confirmation record:
> `t01-independent-confirmation.md`; outcome: decision log D-2026-09-17-T01-08
> … -11. Validator extended (mechanical readiness now honors
> `execution.branch_blockers`). **Next: ready = T02, T03, T04, T05 (heavy
> commands sequential); T06 NOT authorized on DAG readiness alone.**

> **2026-09-17 GODSPEED BOUNDED-JUDGMENT REPRESENTATIONAL REBASE — CASE 2
> (PLAN-ONLY); SPEC BYTE-IDENTICAL; T00 RECONFIRMED; READY = T01–T05.**
> Controlled rebase of spec → plan → T00 settlement → execution state before any
> implementation task. Audit against the ten corrected-logic items (Judgment
> Plane as architectural concern; VerificationRecord as slice-1-only carrier;
> surface/task_family/capability_class falsification scopes; pilot-not-calibration
> with mandatory disclosure; DomainForge sole semantic authority over a dual-basis
> DecisionSurface; answer domain normative with physical placement contingent;
> correlation-as-joinability; provider disagreement as evidence; comparability
> levels A/B/C; authority non-bypass), the PROOF-1 governing question, and branch
> semantics A–E found everything ALREADY_ENCODED in the normative spec (unchanged,
> sha256 `69b7d1ba…baf6a82`) except one PARTIALLY_ENCODED item: T01's
> preregistration freeze contract did not explicitly enumerate semantic basis,
> evaluation basis, provider/config identities, deterministic baseline, and the
> exploratory pilot metric list. **Plan corrected v1.1.0 → v1.2.0**: T01 gained a
> two-stage freeze_contract (F0 before first harness execution; F1 before any
> provider result is observed) — no requirement id, task id, DAG edge, gate, or
> spec binding changed. **Validator updated only as required**: version pin 1.2.0
> and execution-aware readiness derivation (pre-execution checks unchanged;
> post-settlement ready/blocked are derived from depends_on + settled_tasks, so
> readiness is now recomputed mechanically on every run). **T00 status: SETTLED —
> RECONFIRMED (round-2)**: validator + all four teeth rerun green against plan
> v1.2.0; ALL heavy gate baselines explicitly INHERITED from the original T00
> settlement (no governing repository changed; no heavy gate rerun). Evidence:
> `.agents/evidence/godspeed-bounded-judgment/T00/round-2/`; report:
> `.agents/reports/2026-09-17-godspeed-bounded-judgment-rebase-review.md`;
> decision log appended. **Next: a cold agent can simply be told "proceed" —
> ready tasks are T01 (critical path), T02, T03, T04, T05; T06+ remains gated on
> T01's A/B/C/D/E outcome.**

> **2026-09-17 GODSPEED BOUNDED-JUDGMENT T00 SETTLED — AUTHORITY FROZEN, GATES
> BASELINED, READY SET RECOMPUTED (T01–T05).**
> Executed at revision `7234823` (branch `ultracode/sea-forge-completion`),
> builder confirmation, P1. Spec binding verified, never repaired: recomputed
> sha256 equals the bound `69b7d1ba…baf6a82`; final_self_check 12/12;
> correction_ledger 11 corrections, zero reversals. Checked-in validator PASS
> (90/90 bidirectional mappings, 26-edge acyclic DAG, 6/6 high-risk
> confirmation groups). All four declared teeth behaved as declared — teeth-4
> needed a round-2 after a round-1 attack-implementation error (edge added in
> the wrong direction; preserved as evidence); the plan file was restored
> byte-exact after every attack. Global gate baselines, all sequential:
> **RED at pre-existing baseline** — SEA_CHECK/SEA_CI (rustfmt drift reached
> at fmt-check; two of three drifted files are committed unmodified files),
> SEA_TEST (exact ids: `sea-forge-cli::conformance_m13::
> t13_1_agent_task_routed_through_server_and_settles`; and the KNOWN FACT
> re-CONFIRMED by scoped run: `sea-forge-server::conformance_run_locator::
> both_layouts_still_resolve_after_a_restart`); **GREEN** — SEA_NO_ASYNC_KERNEL,
> SEA_PROOF (P1–P4b), GAUNTLET_CHECK/TEST/CI (1787 tests; ci adds 261
> boundary tests + ruler lock PASS). EDGEAI_HEALTH/EDGEAI_JETSON_PORTS
> recorded NOT_RUN_TARGET_ONLY (recipes resolved at `../edgeai/Justfile`
> lines 495/782; development-host output is never target evidence). Masking
> limitation recorded: suites ordered after the first failing binary were
> never executed at T00. Dirty-worktree inventory (73 entries) captured
> before any gate ran. Evidence:
> `.agents/evidence/godspeed-bounded-judgment/T00/`; decision log (created,
> append-only): `.agents/evidence/godspeed-bounded-judgment/decisions.yml`;
> two new OBSERVED_DEBT entries for the pre-existing reds. **Next: per the
> kickoff contract T01 was NOT started in this run — next kickoff recomputes
> ready tasks from `.agents/current_status.yml` (T01–T05 ready; T01 freezes
> its preregistration + corpus manifest before the first harness command).**

> **2026-09-17 GODSPEED BOUNDED-JUDGMENT PLAN v1.1.0 — REMEDIATED, VALIDATED, READY FOR T00 ONLY.**
> The adversarial inspection was vetted against the actual SEA, Gauntlet, and
> EdgeAI workspaces. Its core findings were valid; its raw-Cargo, wildcard-corpus,
> and incomplete confirmation remedies were tightened. The repaired plan now uses
> canonical repository recipes, exact host/cwd gate metadata, an immutable corpus
> manifest, explicit task-harness authoring, a dependency/state-equivalent 26-edge
> DAG, and requirement-complete confirmation for all six high-risk groups.
> Mechanical validation passes: authoritative spec hash unchanged; self-check
> 12/12; 90/90 requirements mapped bidirectionally; 15 complete tasks; acyclic;
> T00 sole ready task. Review:
> `.agents/reports/2026-09-17-godspeed-bounded-judgment-plan-remediation-review.md`.
> Start prompt: `.agents/plans/godspeed-bounded-judgment-kickoff.md`.

> **2026-09-17 GODSPEED BOUNDED-JUDGMENT SPEC — AUTHORITATIVE, UNCHANGED; EXECUTION NOT STARTED.**
> Canonical spec v1.0.0 (`godspeed.judgment-plane`) is authoritative at
> `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml`
> (sha256 `69b7d1baabd55ae61c73e615f04cf0fda43a4958a9fa3d50567035425baf6a82`;
> draft v0.1.0 archived verbatim at `.agents/specs/archive/`). Executable plan
> with 15 tasks (T00-T14), plan revision v1.1.0, at `.agents/plans/godspeed-bounded-judgment-plan.yaml`
> plus cold-agent brief `godspeed-bounded-judgment-brief.md`; mutable state in
> `.agents/current_status.yml` (ready: T00; all others blocked per DAG).
> Plan validation passes (spec binding, 12/12 self-check, 11 corrections,
> 90/90 bidirectional mappings, 6/6 high-risk confirmation groups, acyclic DAG,
> complete contracts, no executable placeholders).
> NO task has run: no implementation, no PROOF-1 execution, no production change.


> **2026-09-09 AGENT INSTRUCTION TOPOLOGY RESTRUCTURING — IMPLEMENTED.**
> Derived repository instruction topology from semantic and architectural boundaries:
> - Root `AGENTS.md` slimmed to universal repository-wide contract and routing (133 lines, 8.0 KiB).
> - Created `crates/AGENTS.md` governing all 22 Rust workspace crates (synchronous kernel vs async edge, fast loop, domain invariants, SFWP contract sync).
> - Updated `workbench/AGENTS.md` absorbing Workbench e2e test evidence rules previously misplaced at root.
> - Created `.agents/AGENTS.md` governing the durable agent workbench (specs vs plans, handoff contract, memory ledgers).
> - All 4 instruction files comply with the <150 line and <32 KiB budget. `just context-check` passes green.

> **2026-09-08 DOMAINFORGE / SXR / SEA-FORGE / CEP INTERFACE UNIFICATION & REMEDIATION — IMPLEMENTED & SETTLED.**
> Settled all recommendations and load-bearing decisions from `.agents/reports/domainforge-sxr-sea-rs-cep-interface-investigation.md`:
> - Upgraded `DomainModelRef` in `sea-forge-domainforge` to `identity_scheme_version: "v2-full-preimage"` (binding full 7-tuple including `d_content_hash`, `semantic_closure_hash`, and real `ADAPTER_DESCRIPTOR_SHA256`).
> - Backward compatibility preserved: missing scheme defaults to `"unknown-pre-versioning"` via `#[serde(default)]`.
> - Fixed CEP-0008 adapter in `sea-forge-extension`: registered `godspeed.event.v1-flat` profile (`sha256:a2b2722008e920d0e74b3970b427b0b2e3e5b323c9321ef9a8f4c017d29162eb`), aligning T05 convergence tests with the full event profile schema.
> - Verification: All 28 `sea-forge-domainforge` tests pass, 42 `sea-forge-extension` tests pass, and 10 `convergence_t05` tests pass green.

> **2026-09-02 REPOSITORY DOCUMENTATION ARCHITECT — IMPLEMENTED (28 DOCUMENTS).**
> Reconstructed and authored the complete self-contained technical knowledge system for SEA Forge
> combining Diátaxis separation, DeepWiki repository modeling, and Google Code Wiki traceability.
> Master deliverables delivered under `docs/`:
> - Master Maps: `documentation-map.md` (Diátaxis sitemap), `source-map.md` (concept-to-source traceability),
>   `architecture.md` (canonical 5-layer spec and invariants), `index.md` (Layer 0 orientation),
>   and `mental-model.md` (Layer 1 conceptual foundation).
> - 11 Subsystem Deep Dives (`docs/subsystems/`): `kernel-pipeline.md`, `authority-fabric.md`,
>   `sandbox-runtime.md`, `settlement-evidence.md`, `integrity-ledger.md`, `capability-memory.md`,
>   `domainforge-boundary.md`, `server-sfwp.md`, `thoth-agent.md`, `workbench-desktop.md`,
>   `spec-pipeline-ip.md`.
> - 4 Execution Workflows (`docs/workflows/`): `cli-run-lifecycle.md`, `case-orchestration-lifecycle.md`,
>   `human-approval-cycle.md`, `agent-delegation-flow.md`.
> - 5 Design Explanations (`docs/explanation/`): `why-authority-precedes-execution.md` (AUTH-01),
>   `settlement-vs-process-exit.md` (DOM-01), `synchronous-kernel-boundary.md` (BUILD-01),
>   `append-only-truth-rebuildable-views.md` (DATA-01), `landlock-jail-security-model.md`.
> - 3 Progressive Tutorials (`docs/tutorials/`): `01-first-governed-run.md`,
>   `02-inspecting-evidence-and-proofs.md`, `03-authoring-first-workbench-case.md`.
> - 5 Practical How-To Guides (`docs/how-to/`): `write-authority-policies.md`,
>   `configure-agent-endpoints.md`, `run-and-verify-gates.md`, `troubleshoot-cell-failures.md`,
>   `regenerate-contracts-and-schemas.md`.
> - 6 Reference Manuals (`docs/reference/` & `docs/operations/`): `terminology.md` (glossary),
>   `cli-command-reference.md` (all 24 subcommands), `sfwp-protocol-reference.md` (SFWP v1 catalog),
>   `configuration-spec.md` (server.yaml & cell layout), `persisted-record-schemas.md` (all 7 record kinds),
>   and `operations/troubleshooting.md` (diagnostic matrix).
> Preserved all pre-existing code, invariants, and unrelated worktree files.

> **2026-08-31 server request admission — IMPLEMENTED, PENDING INDEPENDENT REVIEW.**
> `spec-server-request-admission.md` is approved. Protected socket work now has
> `max_concurrent_runs` active admission permits and an eight-request waiting room.
> Overflow returns `server_busy` before durable state; pre-admission timeouts abort
> with no effect; durable mutations require a caller `request_id`; and pending IDs
> no longer execute concurrent duplicates. Follow-up fixes serialize same-ID
> check-and-bind, use a CAS transition to settle the timeout/admission race, and
> settle interrupted pending requests at restart. Focused Unix-socket overflow/no-effect,
> no-ID, and restart-recovery tests pass, as does `just crate-check sea-forge-server`.
> Three successive independent reviews found and then confirmed fixes for the
> same-ID race, timeout/admission race, restart recovery, and durable probe locator.
> Final independent result: **CONFIRM**.
> Full `just crate-test sea-forge-server` reaches an unrelated
> `conformance_run_locator::both_layouts_still_resolve_after_a_restart` failure:
> its second server cannot start while the first still owns the cell lock.
> See `.agents/current_status.yml` for commands and changed files.

> **2026-08-31 SEA-FORGE JOURNEY SETTLEMENT GAUNTLET — SETTLED (12/12 PASS).**
> Fully automated, canonical journey settlement testing system constructed and executed
> against the live Workbench product using `agent-browser` 0.34.0 and independent backend
> settlement oracles.
> - **Canonical Journeys**: All 12 canonical journeys (CJ01–CJ12) evaluated across all 10
>   required gates (Entry, Visibility, Reachability, Binding, Authority, Execution,
>   Evidence, Settlement, Continuity, Recovery) — all 12 PASS.
> - **4-Dimensional Coverage**: 100% Canonical Journeys (12/12), 100% Reconciled Stories
>   (128/128 from `canonicalization-matrix.csv`), 100% Interface Projections (38/38
>   bindings across Web UI, API, CLI, Agent), 100% Journey Transitions (10/10).
> - **Settlement Integrity**: Browser assertion never substitutes for independent settlement.
>   Oracles independently verify immutable case plans, precondition digests, approval ledgers,
>   semantic envelopes, and execution boundaries.
> - **Artifacts & Evidence**: Machine-readable contracts, schemas, traces, snapshots,
>   consequential screenshots at every boundary, and summary reports committed under
>   `.agents/reports/ux-journey-settlement/`.


> **GODSPEED CANONICAL RUNTIME CONVERGENCE — SETTLED.**
> All 35 frozen requirements CONFIRMED. Delta = 0. Preregistration hash
> `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f`
> intact. T12 fresh independent final verifier returned CONFIRM after
> 13 independent attacks across all 7 variation classes. Evidence:
> `.agents/evidence/e2e/T0{1..12}/`. Infrastructure docs:
> `docs/explanations-and-references/goodspeed-loop.md`.
> **Next executable action: none — plan settled.**

> **2026-08-25 convergence plan T07 settled (subagent-built, subagent-verified).**
> E7: canonical ProofCompleted (SWE_SEED `proof_completed.rs`) binds ONLY to
> real T06-adjudicated settlements via content-addressed refs with mandatory
> causality; sxr native ingestion gate (`sxr-core/src/proof_ingest.rs`)
> preserves expected-vs-observed inputs by reference, names diverged fields
> (`ClaimMismatch`), keeps first-record integrity, and refuses duplicates/
> replay/cross-wire/placeholder identity. Independent adversarial confirmation
> by fresh subagent: **CONFIRM** — 43 empirical attacks incl. content-address
> forgery and both plan teeth; residual D1-D4 debt (out-of-band digest
> pinning, undelivered-bundle markers) recorded for T08. Evidence:
> `.agents/evidence/e2e/T07/`. Delta: **11 open / 24 CONFIRMED; T08 ready**
> (Expected-vs-Observed developmental evidence — E8/I8/I10, P3).

> **2026-08-25 convergence plan T06 settled (subagent-built, subagent-verified).**
> E6/I6/I7: settlement evaluated from ledger-settled observations vs DECLARED
> E4 criteria (exit-zero alone can never accept; non-completion rejected even
> with criteria text present); canonical envelope with all 7 frozen fields,
> mandatory E5A+E5B causality, content-addressed evidence_refs; SWE_SEED
> adjudicator bound to originating work_request_id with restart-safe
> idempotency and conflicting-resettlement refusal; operational-facts output
> type carries zero proof/capability shape. Independent adversarial
> confirmation by fresh subagent: **CONFIRM** — preregistered battery executed
> empirically incl. end-to-end exit-zero coercion cross-repo; T05-A10d/A11
> debt CLOSED here; T04-D1/D2/D3 verified not reproduced. Evidence:
> `.agents/evidence/e2e/T06/`. Delta: **12 open / 23 CONFIRMED; T07 ready**
> (Proof to RealityTrace boundary — P3).

> **2026-08-25 convergence plan T05 settled (subagent-built, subagent-verified).**
> E5A/E5B/I5: canonical AuthorizedInvocation emitted only from a REAL Allow
> AuthorityDecision (deny/escalate ⇒ no envelope, no grant, no governed side
> effect; runtime refuses foreign actions under genuine grants); InvocationLedger
> binds ExecutionObservation to the exact invocation generation (late/cross-wired/
> duplicate/self-asserted-authority refused). Builder: 20 tests, full
> sea-forge-server suite green. Independent adversarial confirmation by fresh
> subagent: **CONFIRM** — 20 empirical attacks incl. all three plan falsifiers;
> trust/durability items (A9-A11) recorded as non-falsifying debt for composing
> transports. Evidence: `.agents/evidence/e2e/T05/`. Delta: **15 open /
> 20 CONFIRMED; T06 ready** (Operational Settlement Return — P3).

> **2026-08-25 convergence plan T04 settled (owner-directed; subagent-built,
> subagent-verified).** E4 GovernedWorkRequest: producer surface
> (`SWE_SEED .../federation/governed_submission.rs` — all 8 frozen fields,
> packet bound to same cycle, causality recorded, opaque submissions refused
> pre-emission) + SEA-Forge ingress gate
> (`sea-rs .../sea-forge-server/src/governed_work_ingress.rs` — producer
> authority, identity vs locally-resolved model, semantic-intent battery,
> distinct proof/settlement obligations, context-packet causal-parent
> binding). Builder: 22 tests both sides, gates green. Independent
> adversarial confirmation by fresh subagent: **CONFIRM** — 22 attacks
> (all three plan falsifiers refused + fresh compositions), D1-D3
> strictness gaps recorded as non-falsifying debt. Evidence:
> `.agents/evidence/e2e/T04/`. Delta: **18 open / 17 CONFIRMED; T05 ready.**

> **2026-08-25 convergence plan T02+T03 settled (owner-directed: proceed).**
> T02 (E2/E3/I4): the SWE_SEED↔Context-Kernel MCP stdio slice now speaks the
> canonical envelope contract on both sides — CK ingress gates (exclusive
> swe_seed producer, identity placeholder rejection, correlation required),
> explicit governed no-context outcomes for required context, canonical
> ContextPacketCreated egress citing the E2 request as causal parent, and
> consumer-side composed adjudication (producer authority + drift + cross-wire
> + causality). Proven by dispatch teeth, unit teeth, AND a live
> cross-binary stdio test. T03 (E0/E1): GSA `canonical_events.py` projects
> DesiredDirection (external-environment authority) and executable affordances
> into WorkRequested (godspeed_agent authority) fail-closed against
> destination-only inputs and pseudo-identities; SWE_SEED `work_ingress.rs`
> enforces the same contract at ingress; a golden fixture generated by GSA's
> real projector is consumed by the Rust gate as cross-language evidence.
> Gates: `just e2e-gate T02`/`T03` bound and green; swe-seed-core 363/0,
> ck-mcp+ck-bin 48/0, GSA new tests 11/11 (5 pre-existing env failures
> verified unrelated by ablation). Evidence:
> `.agents/evidence/e2e/T02/t02-report.md`,
> `.agents/evidence/e2e/T03/t03-report.md`. Delta: **19 open / 16 CONFIRMED;
> next ready task: T04** (blocked until then: nothing — T04's deps are met).

> **2026-08-25 convergence plan T01 implemented (owner-directed: proceed).**
> Canonical semantic envelope + DomainForge identity foundation landed in
> SWE_SEED `crates/swe-seed-core` federation module (the most complete
> existing production binder; no new envelope family, v1 wire shape
> unchanged, no new dependencies): `identity.rs` (VerifiedDomainIdentity
> identity gate — fallback pseudo-hash sha256("agentic_capability_loop"),
> all-zero, malformed, and missing/mismatching artifacts all rejected; strict
> resolution errors instead of falling back), `producers.rs` (exclusive
> event-producer registry per the frozen edge topology; unknown types/agents
> fail closed), `derive_event` + `caused_by:` provenance entries (causal
> parents without breaking v1 `additionalProperties:false`; correlation
> `work_request_id` carried unchanged, mismatches fatal), `IdempotencyLedger`
> (fsync-per-admission durable dedupe surviving restart), `validate_envelope`
> composition gate, and pure `check_conformance` that cannot evaluate claim
> truth. Tests: `tests/convergence_t01_envelope.rs` — 22 tests mapping every
> preregistered T01 falsifier incl. all three plan teeth attacks (missing
> model → rejected; forged EvidenceRecorded by non-RealityTrace → rejected;
> conformant-but-false claim → zero truth consequence). Verification: full
> swe-seed-core suite green (0 failed; 2 pre-existing ignored), touched-file
> fmt clean, clippy clean on touched code. Gate bound: `just e2e-gate T01`
> (runs T01 suite + v1_contract + federation_parity pins).
> **Status: SETTLED 2026-08-25 — fresh independent adversarial confirmation
> returned CONFIRM** (`.agents/evidence/e2e/T01/t01-verifier-independent-confirmation.md`;
> 13 fresh attacks, zero falsifiers; preregistration byte-for-byte intact).
> The 11 requirement verdicts are now CONFIRMED in
> `.agents/status/e2e-current-status.yml`; Delta recomputed (24 open);
> **T02 and T03 are unblocked per dependency_graph and may begin in parallel.**

> **2026-08-25 convergence plan T00 executed (owner-directed: implement
> `.agents/plans/e2e-plan.yml`).** Delta-0 and the verification surface now
> exist. (1) `.agents/status/e2e-current-status.yml`: all 35 frozen
> requirements (12 edges E0–E10 incl. E5A/B, 15 invariants I1–I15, 8 envelope
> rules ENV-I1–I8) classified against fresh cross-repo source evidence
> (sea-rs, SWE_SEED, Context_Kernel, godspeed_agent, sxr, domainforge):
> **0 CONFIRMED / 30 PARTIAL / 5 ABSENT (E4, E6, E7, E8, I2) / 0 CONTRADICTED
> / 0 UNKNOWN — open delta = 35**, independently confirming every round-zero
> hypothesis in the preregistration (E8 has no production producer anywhere;
> GovernedWorkRequest/OperationalSettlement exist in no repo's code). (2)
> Gate aliases bound per the plan's verification model: `just e2e-check`
> →`just check`, `just e2e-test`→`just test`, `just e2e-lint`→fmt+lint,
> `just e2e-delta-check` (new mechanical validator), `just e2e-gate <TID>`
> (T00 bound to prereg+delta checks; other tasks fail closed until they bind),
> plus `just e2e-delta-report`. (3) New scripts: `e2e-prereg-ids.sh` (derives
> the frozen 35 from the preregistration itself — PyYAML + scoped fallback),
> `e2e-delta-report.sh` (deterministic Delta-0 regeneration),
> `e2e-delta-check.sh` (count/uniqueness/frozen-vocabulary/evidence-resolution/
> CONFIRMED⇒production-path validation + committed-report drift detection).
> (4) Evidence: `.agents/evidence/e2e/T00/delta0.md` (committed,
> drift-checked). Teeth (all temp-copy isolated, real files untouched): removed
> requirement→FAIL, illegal verdict→FAIL, CONFIRMED-on-synthetic→FAIL,
> unresolvable evidence path→FAIL, drifted report→FAIL, unbound task
> gate→fail-closed; real gates green after teeth (`just e2e-gate T00` exit 0).
> Frozen preregistration byte-identical (ef571089…a879f); plan file untouched
> (0c57e543…d0e39). Not yet run this session (alias-only, delegates to
> untouched existing gates): `just e2e-check`/`e2e-test` full workspace cost.
> Per plan initial_state, next executable action is **T01** (canonical
> semantic envelope/domain identity foundation) after this handoff.


> **2026-08-25 E2E preregistration freeze gate added (owner-requested).** New
> root recipe `just e2e-prereg-check` →
> `scripts/check-e2e-preregistration.sh`: recomputes the SHA-256 of the frozen
> preregistration `.agents/specs/e2e-preregistration.yml`, reads the expected
> hash strictly from `source.spec.sha256` in `.agents/plans/e2e-plan.yml`
> (PyYAML when available, indentation-scoped fallback otherwise; placeholder
> and malformed values rejected), prints expected/observed plus an explicit
> `VERDICT: PASS|FAIL`, and exits nonzero on mismatch without ever rewriting
> the stored hash — a mismatch stops the convergence/Gauntlet run until the
> change is reviewed and re-frozen. Verified: real preregistration PASSes
> (`ef571089…a879f`); teeth-checked nonzero FAIL on mismatch through `just -f`
> against a tampered temporary mirror, and on unbound-placeholder/malformed/
> missing-file paths; frozen file byte-identical after all checks. The plan's
> final_acceptance item "Verify the preregistration SHA-256 equals
> source.spec.sha256" is now executable as `just e2e-prereg-check`.


> **Deep-project-audit remediation is active on `ultracode/sea-forge-completion`.**
> Preserve the protected stashes (`stash@{0}` and `stash@{1}`); do not pop
> either without reviewing it. The governing plan is
> `.agents/plans/deep-project-audit-remediation-2026-08-14.md`.
>
> **Latest slices: Batch 8 remainder (F-13, F-16) and Batch 9 hardening sweep
> COMPLETE — the deep-project-audit remediation plan is now fully executed.**
> Every finding row in the plan inventory is now R (remediated), AR, P with a
> recorded disposition, or D/NA; remaining follow-ups live in
> `OBSERVED_DEBT.md` (F-25.n idempotency-for-request-less-mutations,
> CEP-0008 inbound causation_id, dead Compatibility/ExtensionInstallRecord
> types, true RFC 8785 JCS profile, ComposedModel overlay precedence) — each
> with its trigger/owner decision.
>
> Batch 8 remainder — **F-13**: promotion matches declarations by *exact*
> `plan_item_id` equality and rejects `"*"` outright (typed error), so a
> capability named `test` can no longer aggregate `itm_contest_7` evidence;
> thoth disclosure uses the same exact rule. Tests: m4a f13_*. **F-16**:
> policy/plan references resolve strictly workspace-relative under the cell
> root (`resolve_policy_path` via core lexical validation; typed UnsafePath on
> absolute/traversal), wired through submit/delegate/probe/cancel/SWE_SEED +
> CLI client spellings; nine server suites + CLI m13 fixtures converted and
> now double as boundary proofs.
>
> Batch 9 — **F-11**: CLI `resume` recovers stranded `Active` cases
> (activated-but-unsettled episodes terminal-settle as rejected/interrupted,
> parked human tasks untouched, loop re-drives lawfully). resume_recovery.rs
> (5 tests). **SUP-06**: probe adapter version derived from config identity
> (`cfg-<12 hex>`); fabricated output digest replaced with a real schema hash;
> failed registration settles Rejected (`agent_endpoint_registration_failed`)
> after authority commits; en-route fix: registry attestation moved from
> per-probe case ledgers to a dedicated cell-scoped `extension-registry`
> ledger (cross-case/restart verification now actually works). Tests: m12
> t12_7_* (4). **F-24**: precondition record cap (16, pre-side-effect typed
> error), resolver opens the ledger once per bundle, jail spawn/harvest/
> continuation-scan/cancellation/events publish/get_range/replay/case minting
> + settlement recording all moved off tokio workers onto blocking threads;
> events ledger mutex-across-fsync eliminated (Arc + ledger flock remains the
> serializer). Tail-cache/paged events deferred with triggers. **F-25
> kernel items**: f pinning test; g narrowed per owner (legacy silent-policy
> recall = own-entity only; cross-entity requires explicit memory_scope);
> h grants bind decision-time canonical argv[0] and runtime re-checks at
> spawn; i shared O_NOFOLLOW `safe_write` through materialize/env/artifact
> paths; j mutation-class reserved mutators hit the hard generated-zone/.git/
> .env/secret boundary before any rule; k collect_artifacts fails closed in
> both backends; l approvals journal single-write append (one core owner);
> m MAX_PLAN_ITEMS=256 + empty-ops SandboxedTask rejection (evaluator-driven
> items exempt) + iterative has_cycle (100k-chain validated); q ledger append
> refuses forged/rewritten tails (predecessor content-hash re-check);
> r sync_data after flush in evidence/trace/capability writers + byte-oriented
> recall scans (non-UTF8 degrades to malformed count). Workbench items a–d
> landed in the separate Tauri workspace (frozen lockfile, sidecar env
> allowlist + no-PATH fallback fail-closed, PID-file-scoped down recipes).
> **SUP-09d**: one canonical primitive (`sea_forge_core::canonical`) behind
> all four former copies with golden vectors — en-route discovery: the copies
> diverged on nested-value NFC; shared primitive recurses at all depths (ASCII
> data ⇒ historical hashes stable; F-25.q would surface any divergence
> loudly). **SUP-09e**: `dual_declared_concepts()` discloses the reviewed
> 30-name overlay-collision set (allowlist-pinned). **SUP-09i**: CEP-0008 ULID/
> sha256/id-segment grammar, descriptor id/version grammar,
> register_built_in hash-mismatch is a loud build bug, import dedupe/terminal-
> standing refusal (registry-brick vector closed).
>
> **Verified after both batches:** `cargo fmt --all -- --check` clean;
> `cargo clippy --workspace --all-targets --all-features --locked -- -D
> warnings` clean; full workspace `cargo test --workspace --all-features
> --locked --no-fail-fast`: **112 suites / 953 passed / 0 failed / 4 ignored**
> (documented real-host release gates); `just context-check` passed.
> Workbench src-tauri gate (after operator installed the Tauri system
> prerequisites): `cargo fmt --check` clean (scoped fmt applied to pre-existing
> drift in bridge.rs/drafts.rs), `cargo clippy --all-targets -- -D warnings`
> clean, `cargo test` green — 27 lib + 4 bridge + 4 packaged_stack; the F-16
> boundary also caught the host bridge test's absolute plan/policy spellings,
> now cell-relative.
> **Latest slices: Batch 5 (identity/role propagation) and Batch 6 (evidence
> integrity) COMPLETE.**
> Batch 5 — F-08: verified `ResolvedActor` (id + role) flows from
> `dispatch_bounded` through `handle_request_as` into all five authority
> evaluation sites; one documented operator fallback for in-process callers.
> F-23: thoth matches disclosure grants against actor id **plus** every role
> the bundle binds to that principal. F-25.e: one `content_hash` primitive
> behind both bundle-hash producers — policy identity no longer path-derived.
> SUP-09f: self-model rebuild idempotency keyed on realization content hash
> (teeth-checked pre-fix). Regression tests:
> `conformance_role_propagation.rs`, thoth `t23_*`,
> `policy_bundle_hash_is_independent_of_source_base_path`,
> `conformance_rebuild_identity.rs`.
> Batch 6 — SUP-04: projection records stamp `Declared`/
> `projection_unvalidated`/`validator_ref "none"`; `verify_projection` now
> hashes materialized outputs against `output_refs` (replaced view file ⇒
> `self_model_error`, teeth-tested in t93). SUP-09h: domainforge authority
> trace honestly describes the stem-heuristic approximation. F-20: jail's
> stderr heuristic classifies as new `SuspectedSandboxViolation`
> (`suspected_jail_violation` basis, still Rejected); definite
> `SandboxViolation` reserved for observed violations.
>
> **Verified after both batches:** fmt clean; clippy `-D warnings` clean on
> all touched crates; full workspace suite 111 suites / 914 passed / 0 failed
> / 4 ignored (documented real-host release gates).
>
> **Latest slice: Batch 7 (root convention + registry trust) COMPLETE.**
> F-12/SUP-07: the state-root convention is now universal — cell.json,
> self-model, thoth capability/declaration reads, transcript-seal keys, bundle
> export/import, template adopt, and the capability promotion readers all join
> directly under the passed root; the CLI default root directory (`.sea-forge`)
> is simply the state root. Export fails closed on zero-resolving requested
> runs; an absent extension registry discloses the snapshot stale
> (`extension_registry_absent`) instead of fabricating a zero-extension cell.
> SUP-08: `ExtensionRegistry::load_verified` proves registry bytes against the
> newest `extension_registry` ledger record; quarantined runtime adapters can
> no longer be replaced/resurrected by registration; duplicate `(id,version)`
> entries rejected at load. Tests: `export_with_unresolvable_runs_fails_closed_not_silently_empty`,
> `legacy_sea_forge_symlink_is_inert_to_import`, extension `load_verified_*` /
> `quarantined_runtime_adapter_*`. Verified: fmt clean, clippy `-D warnings`
> clean on all seven touched crates, full workspace 111 suites / 917 passed /
> 0 failed / 4 ignored.
>
> **Next task:** Batch 8 remainder — governance metadata + operability:
> F-13 (exact capability↔plan-item mapping in promotion matching, reject `*`)
> and F-16 (constrain server policy/plan path resolution to workspace-relative
> under cell root). F-21 already landed cross-batch. Then Batch 9 hardening
> sweep (F-24, remaining F-25.a–d/f–i items, SUP-09d/e/i) plus the still-open
> F-11 (stranded-`Active` recovery verb) and SUP-06.


> **Latest slice (Workbench, owner-directed):** the "Case-authoring proof
> scenarios (6, 7) have no e2e coverage" debt is resolved — per the owner's
> direction the coverage is driven by **agent-browser**, not Playwright. New
> `just workbench-e2e-case-authoring` → `scripts/workbench-e2e-case-authoring.sh`
> + `workbench/apps/desktop/e2e-agent-browser/sfwp-case-authoring-mock.js`
> (mocked-IPC shim; agent-browser counterpart of `e2e/tauriMock.ts`). Both
> journeys pass against the real renderer: stale-precondition repair (commit
> #1 rejected stale carrying digest A, re-preflight pins B, commit #2 carries
> B, zero status recovery) and dropped-commit recovery (one `case_commit`
> total, forced click on the disabled control never reaches the bridge, one
> `request.get_status`, zero axe violations, zero page errors). Evidence and
> method: `.agents/reports/2026-08-15-case-authoring-agent-browser-e2e/`.
> Two new debt entries filed from this work: the authoring pill keeps
> "Preflight passed" in `rejected_as_stale`, and the Playwright
> `tauriMock.ts` readiness fixtures fail the current `ReadinessView`
> contract (`next_lawful_action` required).
>
> **Completed:** Batch 1 (path/generated-zone), Batch 2 (ledger
> crash-consistency/corruption handling), Batch 3 (dispatcher completion
> invariants), and **Batch 4 (resource bounds + panic/overflow) now COMPLETE**:
> SUP-01, SUP-05, F-07, F-18, F-19/SUP-09a, plus SUP-09b (symlink-safe bounded
> migration traversal; rejection precedes key creation and every other side
> effect) and SUP-09c (caps for untrusted whole-file reads: server view
> readers 4 MiB records / 64 MiB journals behind one shared `size_within_cap`;
> trace internal-error append now streams; jail stderr heuristic capped at
> 65,537 bytes with lossy decode; `internal-test-swe-seed` stdin capped at
> 1 MiB). Cross-batch: F-10, F-21, SUP-02 also landed (see the plan's §6).
>
> **Next task:** Batch 5 — identity/role propagation (class D): F-08
> (`ActorRole` into all `Actor` construction sites), F-23 (thoth actor id→role),
> F-25.e, SUP-09f (release-id pinning), per the dependency-ordered plan.
>
> **Latest verified checkpoint (kernel slice):** `cargo fmt --all -- --check`
> clean; `cargo clippy -p sea-forge-cli -p sea-forge-server -p sea-forge-trace
> -p sea-forge-sandbox --all-targets -- -D warnings` clean; new suites green
> (migrate_safety 3, swe_seed_cli 2, conformance_case_views 9,
> conformance_run_views 11, sea-forge-trace 10, sea-forge-sandbox incl.
> conformance_m1 15/15; conformance_m0_migrate 2 regression intact); full
> workspace `cargo test --workspace --all-features --locked --no-fail-fast`
> exit 0 — 109 test binaries, 907 passed, 0 failed, 4 ignored (documented
> real-host release gates). **Workbench slice:** `just
> workbench-e2e-case-authoring` exit 0 (both journeys; recipe wiring
> verified end-to-end); workbench deps installed via
> `bun install --frozen-lockfile` (667 packages).


> **2026-07-27 rebase recovery complete.** `main` now contains the previously
> local full-spec/workbench history rebased onto `origin/main` (110 commits
> ahead, 0 behind). Rebase conflicts preserved the published licensing package,
> added the approved exact `xxhash-rust 0.8.16` / `BSL-1.0` cargo-deny
> exception, and retained a valid `LicenseRef-SEA-Forge` workspace expression
> plus `LICENSE` file reference. `cargo metadata`, `cargo deny check licenses`,
> and `git diff --check` pass. Graph refresh was explicitly deferred. Local
> Jolli state is preserved in `stash@{0}`; the original user stash remains at
> `stash@{1}`. Do not pop either without reviewing its contents.

> **2026-07-27 architectural adjudication Pass 2 complete.** Seven authoritative
> reconciliation and packaging inputs were added under `docs/execution/`:
> `ARCHITECTURAL_TRUTH.md`, `ARCHITECTURAL_INVARIANTS.md`,
> `CONTRADICTIONS_AND_DECISIONS.md`, `PRODUCT_COMPLETION_DEFINITION.md`,
> `EXECUTION_DAG.md`, `DECISION_REGISTER.md`, and `PASS_2_HANDOFF.md`. No product
> feature or source behavior changed. Independent source review corrected Pass
> 1's Copilot, tracked `working/face/`, DomainForge, route-guard, inferred-gate,
> and end-to-end usability claims. The highest completion blockers are the
> server case sandbox path's pre-authority directory creation, incomplete
> lifecycle/exit-code settlement, case-run versus run-view path mismatch,
> unresolved Workbench/server service lifecycle, identity/idempotency gaps, and
> absent packaged real-stack E2E. Distribution packaging is conditionally ready:
> owner decision U-06 (Tauri sidecar versus separately installed local service)
> is required before deployment configuration changes. Verification: fresh
> adversarial review completed and reconciled; `git diff --check` clean;
> `devbox run -- just context-check` passed.

> **2026-07-26 Workbench plan Task 7 (case overview and horizon) complete,
> plus the four `OBSERVED_DEBT.md` entries Task 7's own "review before
> starting" block gates on.**
>
> **New SFWP methods (five, all additive per ADR-003).** `case.list`,
> `case.get_overview`, `case.get_horizon` in new
> `crates/sea-forge-server/src/sfwp/case_views.rs`; `approval.list` and an
> `approval.decide` envelope in new `sfwp/approvals.rs`. `IMPLEMENTED_METHODS`
> is now sixteen, covering six of the epic's journeys. All five are read-only
> projections over records the kernel already committed — `case.json`,
> `plan.json`, per-run `settlement.json`, `case-events.jsonl`, and
> `approvals.jsonl`. None introduces new truth.
>
> **Horizon item standing is folded from trace events, not read off a status
> field.** No per-item status exists anywhere in the kernel; standing *is* the
> `TraceKind` sequence the case runner appended. Folding it is reading kernel
> truth (the same derivation `next_case_actions` performs); caching it anywhere
> would create a second authority that drifts.
>
> **Execution and settlement are structurally separate.** `ExecutionStanding`
> and `SettlementStanding` are disjoint enums with no shared values, neither
> derived from the other, and the UI maps `execution: completed` to a
> *non-success* pill. A zero exit code cannot masquerade as accepted work.
> `execution_and_settlement_are_separate_vocabularies` asserts the vocabularies
> never cross.
>
> **One owner for the approvals fold.** The append-only journal's "latest record
> per `approval_id` wins" rule moved from `sea-forge-cli` into
> `sea_forge_core::approvals`; the CLI module is now a re-export. Two folds
> could disagree about whether an approval is open, and the one saying "open"
> would offer a decision already made.
>
> **Debt resolved.** (1) `scripts/check-agent-context.sh` — CRLF→LF + exec bit;
> `just context-check` is a live gate again. (2) The host's hardcoded
> `GET_RANGE_PAGE_CAP = 256` vs the server's real 500 — deleted rather than
> reconciled: the catch-up drain now terminates on an **empty** page, encoding
> no assumption about the server's page size at all. (3) Coarse readiness
> event-invalidation — `hooks/eventKinds.ts` narrows by kind, deliberately
> asymmetric so an *unrecognized* kind still invalidates (fails open to an extra
> read, never to a stale render). (4) `approval.decide` reachable but not
> discoverable — `approval.list` closes it.
>
> **Two real defects found and fixed en route.** The event-loop listener
> dereferenced `event.payload` unguarded; a throwing listener tears down the
> whole subscription, so it now reads `event?.payload` and degrades to
> "invalidate anyway". And `` `${verdict}d` `` rendered "rejectd" to the operator
> — replaced with an explicit past-tense map.
>
> **Evidence.** `cargo fmt --all -- --check` clean; `cargo clippy --workspace
> --all-targets --all-features -- -D warnings` clean; `./scripts/check-agent-context.sh`
> → "context check passed"; **`cargo test --workspace` — 98 suites, 714 passed,
> 0 failed, 4 ignored** (the ignored are the documented real-host release
> gates); 13 of those are new (`conformance_case_views.rs` 6,
> `conformance_approvals.rs` 7); host `tests/bridge.rs` 4/4 including the new
> `catch_up_drains_until_a_page_is_empty_not_merely_short`, verified to have
> teeth (restoring the short-page rule drops 3 of 6 events); `bun run check`
> clean apart from the pre-existing `router.tsx` fast-refresh warning;
> `bun run test` 73 desktop + 17 component tests green; `bun run build`
> produces a renderer bundle.
>
> On the one failure seen in the *first* workspace run
> (`kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption`): measured
> rather than assumed-flaky, because this change touched `sea-forge-cli`. A
> clean-`HEAD` worktree passed while the working tree failed, which read as a
> regression; re-running both on an idle host resolved it (working tree 8/8
> consecutive, and the second full workspace run green). It tracks host load,
> not the diff. Method recorded in `OBSERVED_DEBT.md`.
>
> **Still open.** Three specimen surfaces remain (Thoth, Assets, Models) —
> `thoth.ask` has no typed response contract, `asset.list`/`domain_model.list`
> do not exist. Plan Tasks 8–14 remain. The Playwright horizon journey named in
> Task 7 step 4 was **not** run: the e2e harness mocks `__TAURI_INTERNALS__`
> entirely, so it cannot prove real event delivery end to end — that limitation
> is its own standing `OBSERVED_DEBT.md` entry and was not closed here.

> **2026-07-26 Workbench plan Task 6 (case authoring: draft, preflight,
> atomic commit) complete.** First protected-command vertical slice: three
> additive SFWP methods in new `crates/sea-forge-server/src/sfwp/case.rs`
> (`Request::CaseEntryOptions`/`CasePreflight`/`CaseCommit`, `lib.rs`). Grounding:
> `case.entry_options` is an honest inspect projection of whatever templates are
> already materialized under `<root>/templates/*.yaml` (empty when none exist —
> `unknown != unavailable`, never a fabricated built-in catalog).
> `case.preflight` instantiates a template (`sea_forge_planner::templates::instantiate`)
> and runs the *same* `case_engine::validate_proposal` `case_dispatch::submit`
> uses — an authoritative dry run, not a forked validator — returning a
> `RecordDigest` pinned to the template's current on-disk bytes
> (`template:<ref>` ref, reusing the existing `sfwp::precondition` mechanism
> rather than inventing a second staleness scheme). `case.commit` writes the
> instantiated plan to a scratch file under `<root>/drafts/` and delegates to
> the exact same `case_dispatch::submit` path `Submit` always used (extracted
> into a shared `commit_plan` helper) after checking the precondition via a new
> `TemplateRecordResolver` — on mismatch, `rejected_as_stale` with zero side
> effects (no case created); on match, the one and only case-minting path runs.
> `request_id` correlation (`record_pending`/`record_outcome`, reused unchanged
> from Task 3) makes a lost commit response recoverable via
> `request.get_status` instead of a resubmit. Draft-storage spike (deferred
> decision in `stack-and-dependencies.md`) resolved as **versioned local JSON
> files under the Tauri host's `app_data_dir`** (new `drafts.rs` + four
> `draft_save`/`draft_load`/`draft_list`/`draft_delete` commands) — zero new
> dependencies, atomic tmp+rename writes, never touches `.sea-forge/`; SQLite/the
> Tauri store plugin deferred until real multi-draft conflict needs appear.
> Frontend: `bridge.rs` gained `SfwpQuery::CaseEntryOptions`/`CasePreflight` and
> `SfwpCommand::CaseCommit` mirrored byte-for-byte; contracts regenerated (6 new
> generated types: `EntryOptionsResult`/`TemplateOption`/`TemplateParameter`/
> `PreflightParams`/`PreflightResult`/`PlanItemSummary` + AJV validators,
> deterministic rerun confirmed, no drift on existing 14). New
> `caseAuthoringMachine.ts` (XState v5): `draft -[PREFLIGHT]-> validating
> -> preflight_ok -[COMMIT]-> committing -> committed | rejected_as_stale |
> ambiguous`. `ambiguous` deliberately has no `COMMIT` handler — only
> `RECOVER -> status_recovery` (calls `request.get_status`) — so a duplicate
> commit is structurally unreachable, not just documented (proof scenario 6,
> asserted by a machine test that sends `COMMIT` while `ambiguous` and checks
> the state didn't move). `rejected_as_stale`'s only transition is
> `RETRY -> validating` (a fresh preflight, never straight back to
> `committing` — proof scenario 7). New `CaseCreationWorkbench` route
> (`/cases/new`, `react-hook-form` for per-template parameter fields, no new
> `@hookform/resolvers` dependency — required-field validation only, since
> real type/shape validation is `case.preflight`'s job, not duplicated
> client-side) and `useCaseEntryOptions` hook (plain TanStack Query, no
> machine, mirroring `useReadiness`). `ReadinessPage`'s previously
> permanently-disabled "Create case" action now navigates to `/cases/new`
> whenever `readiness.get`'s `local_governed_execution` capability and all
> foundations are ready — the first real consumer of that lawful-action slot.
> Tests: 6 new Rust conformance tests (`conformance_case_authoring.rs`:
> entry_options honesty incl. empty-when-absent, preflight ok/error, commit
> success + status roundtrip, stale-precondition-rejects-with-no-case-created,
> commit-outcome-recoverable-via-request.get_status) + 6 XState machine tests
> (including the two proof-scenario tests above) + 1 component test (full
> draft->preflight->commit->navigate flow) + `ReadinessPage.test.tsx` updated
> for the now-enabled action. Gates green: `cargo fmt --all -- --check`,
> `cargo clippy -p sea-forge-server --all-targets -- -D warnings`, `cargo
> clippy` (Tauri host crate, isolated workspace), `cargo test -p
> sea-forge-server` (all suites incl. the 7 new), `cargo test` (Tauri host,
> all suites), `bun run check` (desktop, one pre-existing Fast Refresh
> warning), `bun run test` (desktop 27/27 + ui-components 17/17), `bun run
> generate:contracts` (deterministic), `devbox run -- just fmt-check`/`lint`/
> `test` (workspace-wide, 0 failures), `devbox run -- just proof` (P1–P4b
> green). `just check`'s `context-check` sub-recipe remains blocked by the
> pre-existing `scripts/check-agent-context.sh` issue (`OBSERVED_DEBT.md`);
> no platform test is claimed through that blocked composite gate.
> Limitations/debt filed in `OBSERVED_DEBT.md`: no Playwright e2e was added
> for this slice (the existing mocked-IPC harness cannot honestly prove the
> stale-precondition/duplicate-commit-unreachable scenarios — those are proven
> at the Rust conformance + XState machine level instead, consistent with the
> skill's guidance to decide this deliberately rather than default to the
> mocked pattern); visual fidelity against the wireframe/mockup kit was not
> pixel-checked (no reference mockup for this screen exists in `ui_kits/`, so
> there is nothing to diff against — logged as an open gap, not claimed done).
> Next spendable slice: Task 7 (case overview + horizon) — the first slice
> that needs live per-case event reduction at scale, and the natural home for
> a "view the case I just created" landing page (this slice navigates to the
> still-mockup `/cases` on commit).

> **2026-07-25 Workbench mockup-fidelity repair complete.** The React shell now
> matches the checked-in workbench kit at its responsive evidence breakpoints:
> the Context / Evidence region is a 380px docked grid track above 1420px and a
> transparent, non-modal 400px (maximum 92vw) right overlay below it, beginning
> below the 56px global bar. It remains mounted while closed so the kit's 180ms
> `cubic-bezier(.23,1,.32,1)` slide-out/slide-in completes; Escape, the close
> control, evidence citations, and the header toggle preserve that state.
> Container-responsive Operate layouts stack focus actions and attention rails
> before labels/tables compress, while shell tracks follow the kit's 236/224/64
> navigation widths. The six Operate routes (Thoth, Assets, Domain Models,
> Cases, Inbox, Operations) now render their route-specific focus/panel
> hierarchies and active journey label instead of generic placeholders. Because
> no live route-family read models exist yet, those surfaces are visibly marked
> `Specification preview · not live`; their controls inspect context only and
> do not imply backend mutations. `readiness.get` remains the sole live source
> for the Readiness route.
>
> Durable regression evidence: `mockupFidelity.test.ts` drift-checks the
> reference stylesheet and required regions; `SurfacesPages.test.tsx` covers all
> Operate view structures; component coverage proves the drawer remains mounted
> for exit motion; Playwright asserts 1600px and 1280px shell/drawer geometry,
> exact transition timing/easing, close/reopen motion, all Operate route swaps,
> computed route-grid activation, no horizontal action overflow, and zero axe
> violations or console/page errors for each route plus Readiness. The
> browser-mode Tauri shim now supplies the event plugin's separate
> `unregisterListener` namespace, so listener cleanup is exercised without an
> unhandled rejection. The Workbench skill now requires
> same-viewport browser comparison and computed-style checks in normal and
> reduced-motion media; its source map records that the static kit's Operate
> selectors are accidentally trapped inside an unclosed reduced-motion block,
> so production must project their intent without copying that boundary.
> Frontend gates green: `bun run check` (one pre-existing Fast Refresh warning),
> `bun run test` (20 desktop + 17 UI-component tests), `bun run build`, and
> `bunx playwright test e2e/readiness.spec.ts --workers=1` (3/3).
> `devbox run -- just test` also passed workspace-wide. `devbox run -- just
> context-check` and therefore `just check` remain blocked before execution by
> the pre-existing non-executable `scripts/check-agent-context.sh` (exit 126),
> already tracked in `OBSERVED_DEBT.md`; no platform test is claimed through
> that blocked composite gate.

> **2026-07-25 Workbench plan Task 5 (Readiness vertical slice) complete.**
> First real settlement wired end-to-end: `readiness.get` SFWP inspect method
> (new `crates/sea-forge-server/src/sfwp/readiness.rs`, dispatched as
> `Request::ReadinessGet`) projects self-model validation
> (`sea_forge_self_model::store::validate`) and agent-endpoint config into a
> `ReadinessView { overall, foundations, operational_capabilities,
> recent_invalidations, intended_operation }` — infallible (a validation
> failure renders as a `blocked`/`integrity_halted` item, never propagates an
> `Err`), operation-sensitive (`intended_operation` reorders which capability
> is foregrounded), no new truth introduced. `ReadinessItem`/`Invalidation`
> shapes were defined from scratch (the API spec references but never defines
> them) grounded in the wireframe's condition-table/capability-row fields.
> Frontend: `SfwpQuery::ReadinessGet` added to the closed Tauri bridge
> (`workbench/apps/desktop/src-tauri/src/bridge.rs`); contracts regenerated
> (`ReadinessView`/`ReadinessItem`/`ReadinessGetParams` + AJV validators,
> deterministic rerun confirmed); `workbench/apps/desktop/src/hooks/useReadiness.ts`
> wraps the query in TanStack Query, validates every response against the
> generated AJV validator before trusting it, and invalidates on any
> `sfwp://event` frame (coarse but honest — no readiness-specific event kind
> exists yet); `ReadinessPage.tsx` rewritten from Task 4's hardcoded mock to
> compose real data through `WhyStatePanel`/`GovernedStatusPill`/
> `IntegrityIndicator`/`SourceFreshnessBadge`/`ProtectedActionButton`/
> `EvidenceDrawer`. Case-creation is permanently, honestly disabled (no
> `case.create` verb exists yet — Task 6) with a reason sourced from the live
> readiness view, never implying a working flow. Server disconnect renders
> the last-known view marked `stale` (TanStack Query's default data retention
> across a failed refetch) rather than blanking it. New Playwright + axe-core
> harness bootstrapped from scratch (`playwright.config.ts`, `e2e/tauriMock.ts`,
> `e2e/readiness.spec.ts`) mocking the Tauri IPC bridge via
> `window.__TAURI_INTERNALS__` injection — proves the real `ReadinessPage`/
> `useReadiness` code paths against a fixture, but does not exercise a real
> `sea-forge-server` process (that's covered separately at the Rust level by
> `conformance_sfwp.rs`'s `readiness_get_*` tests, which do boot a real server
> on a temp root). `statusMachine.ts` deliberately NOT replaced — a plain read
> needs no XState machine; it stays as the (unrelated) ProofPage's dependency
> pending a real preflight/commit machine in a later authoring slice. Two
> independent architect-verification passes both returned APPROVED. Gates
> green: `cargo fmt/test -p sea-forge-server` (4/4 readiness tests),
> `devbox run -- just fmt-check`/`lint`/`test` (workspace-wide, 0 failures —
> `just check`'s `context-check` sub-recipe could not run, see
> `OBSERVED_DEBT.md`), `bun run generate` (deterministic), `bun run check`
> (desktop, clean), `bun run test` (desktop 11/11 + ui-components 16/16),
> `bunx playwright test --grep readiness` (1/1, zero axe violations). Four
> new entries filed in `.agents/OBSERVED_DEBT.md`: the broken
> `check-agent-context.sh` (pre-existing, unrelated), the Playwright-mocks-
> vs-real-server e2e gap, coarse event-invalidation scope, and the
> permanently-empty `recent_invalidations`/never-derived `stale` fields.
> Next step: plan Task 6 (case authoring: draft, validation, preflight,
> atomic commit) — the first slice that needs a real mutation/lifecycle
> machine and will also give the disabled "Create case" button real work
> to do.

> **2026-07-25 Workbench plan Task 4 (shell + semantic components + guards) complete.**
> Created `@sea-forge/ui-components` Bun workspace package containing the nine semantic
> components (`GovernedStatusPill`, `DualStateIndicator`, `SourceFreshnessBadge`,
> `IntegrityIndicator`, `ProtectedActionButton`, `WhyStatePanel`, `EvidenceDrawer`,
> `AuthorityBoundaryPanel`, `AvailabilityLadder`) built with CSS Modules and `@astryxdesign/core` primitives.
> Critical invariant enforced: `GovernedStatusPill` with an unknown or invalid variant strictly fallbacks
> to `"unknown"` text & class (`status-pill--unknown`), verified by unit test.
> Storybook v8 configured in `packages/sea-forge-ui-components` with component stories (`*.stories.tsx`) for all 9 components.
> Built governed shell in `apps/desktop` featuring:
> - Sidebar navigation with all 13 top-level surfaces (Readiness, Thoth, Assets, Domain Models, Cases, Inbox, Operations, Evidence, Memory, Capabilities, Artifacts, Federation, Administration).
> - Global Context Bar (Actor, Role, Policy status, Integrity indicator dot, Search shortcut `/`, Inbox count badge).
> - Governed Focus workspace header, journey ribbon, skip-to-content link.
> - Collapsible right EvidenceDrawer skeleton.
> - Route guards G1–G9 with `GovernedDenialSurface` rendering on failed evaluation (never blank screens or crashes).
> - Keyboard traversal & ARIA accessibility support (`Tab`, `Shift+Tab`, navigation hotkeys `R` & `/`), verified by `axe-core`.
> Gates green: `bun run check`, `bun run test` (23 unit tests pass), `bun run build-storybook`, `devbox run -- just check`, `devbox run -- just test`. Added `just` recipes for dev server and Storybook (`dev-up`, `dev-down`, `storybook-up`, `storybook-down`). Next: plan Task 5 (Readiness vertical slice).

> **2026-07-24 Workbench plan Task 3 (SFWP transport) complete.** Additive
> SFWP protocol layer on the existing Unix-socket NDJSON server, added as flat
> `Request` variants behind the ADR-003 seam (`system_*`, `request_get_status`,
> `events_*`, precondition digests) in `crates/sea-forge-server/src/sfwp/`.
> Schema generation via `schemars` (confined to `sea-forge-server`; the
> `gen_sfwp_schema` bin emits JSON Schema to
> `workbench/packages/contracts/schema/`; the `@sea-forge/contracts` Bun
> package generates typed TS + AJV validators via `bun run generate:contracts`)
> — new deps recorded in `docs/decisions/ADR-005-sfwp-schema-generation.md`.
> Tauri host owns the socket via a closed `SfwpQuery`/`SfwpCommand` bridge
> (`workbench/apps/desktop/src-tauri/`, per `workbench/AGENTS.md`).
> Evidence: `crates/sea-forge-server/tests/conformance_sfwp.rs` (the named
> hello → subscribe → mid-flight kill → reconnect → `request_get_status` →
> cursor resume → `events_get_range` gap-recovery scenario) and 3 host
> integration tests in `workbench/apps/desktop/src-tauri/tests/bridge.rs`.
> Full gate green (`devbox run -- just check`/`just test`; workbench
> `bun run check`/`build`/`test`; Tauri `cargo build`/`test`). One watch item
> in `.agents/OBSERVED_DEBT.md` ("Host event-catch-up page cap …"): the host's
> `events.get_range` page-cap guess (256) vs the server's actual cap (500) are
> separately hardcoded — currently safe (conservative) but uncoupled. Next
> step: plan Task 4 (application shell + semantic design foundations: shell
> layout, nine semantic components, route guards G1–G9, Storybook, a11y).
>
> **2026-07-24 Workbench plan Task 2 (workspace + stack proof) complete.**
> New `workbench/` Bun workspace (`package.json`, `bunfig.toml`,
> `packageManager: bun@1.4.0`) with `apps/desktop/` (Vite + React 19.2 +
> TypeScript 6.0 strict, scaffolded via `bun create vite`) and a Tauri 2 host
> crate at `apps/desktop/src-tauri/` — a standalone Cargo workspace (own
> empty `[workspace]` table), deliberately not a member of the root kernel
> workspace. Token/theme packages: `packages/sea-forge-ui-tokens` (byte-exact
> copy-projection of `.agents/specs/frontend/colors_and_type.css`, drift-
> checked) and `packages/sea-forge-astryx-theme` (SEA Forge tokens projected
> onto `@astryxdesign/theme-neutral` via `defineTheme({ name: "neutral",
> extends, tokens })`, keeping Astryx's scoped component CSS wired while every
> token value routes through SEA Forge canonical vars). Locked stack
> exact-pinned: Astryx 0.1.8, StyleX 0.19.0, TanStack Router 1.170.18 / Query
> 5.101.4, XState 5.32.5 / @xstate/react 6.1.0, react-hook-form 7.82.0, ajv
> 8.20.0, @tauri-apps/cli 2.11.4, Vitest 4.1.10 — recorded in
> `docs/decisions/ADR-004-workbench-stack.md`. Proof page (`ProofPage.tsx`)
> renders an Astryx `Button`+`Table` themed by the projected tokens behind one
> typed TanStack Router route with validated search state, one TanStack Query
> call, and one XState machine (`statusMachine`, smoke-tested in
> `statusMachine.test.ts`). `just workbench-check` (new recipe) and the
> plan's exact gate command (`bun install --frozen-lockfile && bun run check
> && bun run build && cargo build --manifest-path
> apps/desktop/src-tauri/Cargo.toml && bun run test`) both pass; Tauri host
> `cargo build` succeeds (`dev` profile, 7 min cold compile). Along the way,
> at the user's explicit request, the machine's mise-managed `bun` was
> upgraded via `bun upgrade --canary` to 1.4.0 (Bun's in-progress Zig→Rust
> rewrite, confirmed via upstream announcement); `workbench/package.json`
> repinned to match, full gate re-verified green under it — reversible via
> `bun upgrade --stable` per the ADR. One watch item filed in
> `.agents/OBSERVED_DEBT.md`: the Tauri host's Cargo-workspace exclusion has
> no automated gate yet. Root kernel untouched (`git status` shows only
> `workbench/` plus this status update, the ADR, and `OBSERVED_DEBT.md`).
> Next step: plan Task 3 (SFWP transport: envelopes, negotiation, request
> recovery, events, generated contracts).

> **2026-07-24 Workbench plan Task 1 (repository grounding and compatibility
> map) complete.** `.agents/reports/2026-07-24-sfwp-grounding.md` grounds all
> 74 target SFWP methods across the 18 catalog families Task 1 names (system,
> request, operation, events, cell, readiness, self_model, case, run,
> agent_run, approval, thoth, settlement, capability, evidence, integrity,
> memory, artifact) against direct repository evidence (four parallel
> research passes, every row `file:line` cited). Verdicts: 14 reuse, 27 adapt,
> 3 merge, 30 add, 0 reject — no target method conflicts with a kernel
> invariant in this pass. Gate (`grep -cE` verdict-row count ≥ 40) passes at
> 75. Notable findings: `case`/`run` families have rich backing logic
> (`case_engine` item states, `TraceEvent` replay, `DelegationResult`) that is
> almost entirely unwired to any server verb; `case.propose_replan`/
> `commit_replan` are wholly missing; the 2026-07-22 audit's "Thoth `ask`
> bypasses governance" finding appears stale (`service::ask` already commits
> ledger records) and is filed in `.agents/OBSERVED_DEBT.md` for
> re-verification, alongside a `cell.migrate`/federation-bundle naming-
> collision caution for later tasks. `repository-integration.md` re-verified
> against the current tree; one line-number drift corrected
> (`prove_entry:915` → `:916`). No production/kernel code touched. Next step:
> plan Task 2 (workspace/stack proof: Tauri 2 + Bun + React 19 + Vite +
> Astryx).

> **2026-07-24 Workbench implementation skill + plan created (no production
> code).** New reusable skill `.agents/skills/building-sea-forge-workbench/`
> (SKILL.md + 8 reference files + deterministic `scripts/validate-skill.py`,
> passing, teeth-checked + 4 evaluations) and repository-grounded plan
> `.agents/plans/2026-07-24-sea-forge-workbench-frontend-api-implementation.md`
> (14 vertical-settlement tasks, capability-delta table, 12 proof scenarios).
> Grounding findings: server = Unix-socket NDJSON `verb`-tagged enum
> (`sea-forge-server/src/lib.rs:396`); no JS workspace/Tauri/schema-gen
> anywhere; SFWP envelopes, request recovery, and `events.subscribe` (cursor
> = ledger `entry_ulid`) are additive work. Frontend spec package staleness
> fixed in place: `css.txt` → `colors_and_type.css` (matches all refs),
> absent `preview/`, `assets/README.md`, `context/provenance.md` references
> corrected in README/SKILL/DESIGN/app README; generated
> `ui_kits/DESIGN-MANIFEST.json` screen-misclassification documented (not
> hand-edited) in the skill's `reference/source-map.md`. Next step: plan
> Task 1 (SFWP method-grounding report).

> **2026-07-24 spec-audit-remediation Task 19 portable closeout complete**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): every focused gate
> from Tasks 1–18 was re-run in dependency order against `ca11dc2`; all
> selected portable tests passed. The Task 9 gate initially selected zero
> tests because its `prerequisite` filter matched no current test name, so
> the plan now uses `predecessor`; the corrected gate selects 10 predecessor
> tests and the complete `sea-forge-spec-pipeline` suite remains green.
> Fresh cumulative results: `devbox run -- just test` passed the workspace
> all-features suite; `devbox run -- just check` passed context, formatting,
> clippy `-D warnings`, typecheck, dependency-policy, and secret-scan gates;
> `devbox run -- just proof` passed minimum P1–P4b; and `devbox run -- just
> no-async-kernel` passed for all 19 kernel crates.
> Linux Landlock connect/bind denial and explicit-grant tests passed. The
> Seatbelt/macOS case was skipped on Linux, and the real ACP and real
> SWE_SEED release tests were not run because their operator-supplied
> environment/host configuration is absent; those three platform/real-host
> claims remain unproved. No matching open entry exists in
> `.agents/OBSERVED_DEBT.md`. The independent correctness/fail-closed/schema/
> dependency/test-teeth review found no remediation blocker; it recorded one
> unrelated historical Markdown-whitespace issue in `OBSERVED_DEBT.md`.

> **2026-07-24 spec-audit-remediation Task 18 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): late SWE_SEED
> declaration reconciliation closes the last documented M16 gap. New
> `sea-forge-server::swe_seed_reconciliation` is a single pure, idempotent
> join: `reconcile_swe_seed_declarations(root, case_id)` reads every
> `agent_task_evidence` record carrying non-empty `harvested_refs` in a case
> ledger, joins it against every ledger-verified `settlement_declaration`
> whose `claim_manifest_sha256` matches an independently-recomputed manifest
> hash over `(case_id, run_id, plan_item_id, settlement_id,
> transcript_sha256, harvested_refs)` (`swe_seed_claim_manifest_sha256` —
> wrong run/verifier/hash therefore never correlates), and commits one
> `swe_seed_correlation` ledger record plus the `runs/<run_id>/swe-seed-
> correlation.json` view per *distinct* resulting declaration set
> (idempotency-keyed on the declaration-id set itself, so an unchanged run
> commits nothing new and matching declarations appear exactly once).
> `delegation.rs`'s inline one-shot correlation construction and its
> snapshot-only `swe_seed_declarations_for_run` helper are replaced by calls
> into this shared reconciler. A new `submit_swe_seed_declaration` in
> `delegation.rs` is the previously-absent production declaration ingress:
> after settlement and harvested evidence are committed, it loads the
> policy's `strong_settlement_authority()` descriptor, builds the
> `SettlementDeclarationRequest` only from already-persisted criteria/
> evidence, and calls `CommandSweSeedTransport`/`SweSeedSettlementAuthority`
> through `tokio::task::spawn_blocking`; an unavailable authority surfaces as
> `settlement_authority_unavailable` and is logged, never locally
> faked — the already-committed settlement is never mutated, and a later
> out-of-process declaration still reconciles. `append_and_reconcile_swe_seed_declaration`
> wraps `sea_forge_settlement::append_declaration_ledgered_once` with an
> immediate reconcile call, used both by the production path and by an
> out-of-process actor appending directly while the server is absent.
> `reconcile_all_cases` runs at `ServerState::new` (alongside the existing
> `recover_cancelled_delegations`), and `verify_swe_seed_completion(root,
> run_id)` reconciles before reporting a run's harvested/declared status —
> the two read-time/startup triggers required by the plan. New tests:
> `sea-forge-server` `swe_seed_reconciliation.rs` unit suite (mismatched
> run/verifier/hash never correlate, declaration-before-evidence correlates
> once evidence lands, repeated reconcile is idempotent, two runs in one case
> correlate independently); `conformance_m16.rs` `t16_8_*` (server-owned
> declaration correlates immediately through a real `CommandSweSeedTransport`
> subprocess — `sea-forge-cli`'s existing hidden `internal-test-swe-seed`
> double, located next to the test binary rather than duplicating a second
> transport fake; a declaration appended directly via
> `append_declaration_ledgered_once` while no server is running reconciles
> on the next `ServerState::new` startup; the same late declaration
> reconciles at read time via `verify_swe_seed_completion` with no restart);
> `sea-forge-settlement` `declaration.rs`
> (`swe_seed_duplicate_declare_for_same_claim_conflicts_on_append`: two
> independent `declare()` calls for the same claim mint different
> `declaration_id`s, and appending both is rejected by
> `append_declaration_ledgered_once`'s idempotency-key conflict check, not
> silently duplicated). Gates green: `cargo test -p sea-forge-server --test
> conformance_m16 swe_seed -- --nocapture`, `cargo test -p sea-forge-settlement
> swe_seed -- --nocapture`, `cargo test -p sea-forge-server
> swe_seed_reconciliation -- --nocapture`. `cargo fmt --all -- --check` and
> `cargo clippy --workspace --all-targets --all-features --locked -- -D
> warnings` are clean; `cargo test --workspace --all-features --locked` (91
> test binaries, 0 failures), `devbox run -- just check`, `devbox run -- just
> proof` (P1-P4b), and `devbox run -- just no-async-kernel` (19 kernel
> crates — `sea-forge-server` is not one) are all green. Real SWE_SEED/ACP
> host release tests remain intentionally ignored pending operator
> configuration.

> **2026-07-24 spec-audit-remediation Task 17 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): built-in topology
> templates and the Thoth manager loop no longer name unregisterable
> endpoints. `sequential_agents_template`/`concurrent_agents_template`
> (`sea-forge-planner::templates`) now take a caller-supplied `endpoint_ref`
> and return `Result<PlanTemplate, ForgeError>`, validating it against the
> same `^[a-z0-9_-]{1,64}$` grammar `AgentEndpointConfig::validate` enforces
> — the old literal `agent:builtin`/`agent:default` values fail this check
> (they contain `:`), so a restored colon ID now fails template construction
> instead of only failing dispatch preflight. `store_builtin` threads a
> `default_endpoint_ref` parameter through to both templates (a new
> `DEFAULT_TOPOLOGY_ENDPOINT_REF` constant covers the CLI's criteria-only
> materialization call site, which never dispatches). Both templates' generated
> `AgentTask` branches/steps and the concurrent rollup milestone are now
> `markers.required = true` — previously `ItemMarkers::default()` left every
> item optional, so `can_auto_complete` completed the case before any branch
> ever dispatched, which is why no test had ever proven real end-to-end
> dispatch through these templates. The Thoth manager loop
> (`sea-forge-cli::commands::manager`) now requires an explicit `--endpoint`
> (never invented/auto-routed) for its synthesized proposal, and binds the
> caller-requested `max_manager_iterations` into the canonical
> `manager_iteration` authority action's parameters so the ledgered decision
> reflects what was actually asked (differing requests now produce differing
> `action_request_hash` values). A new `ActionGrant::max_manager_iterations`
> accessor (`sea-forge-authority`, mirroring the existing
> `network_tcp_ports` fail-narrow pattern) reads an authority-granted
> `max_manager_iterations` boundary constraint (added to the boundary
> dimension allowlist); `manager::iterate` now enforces
> `min(requested, grant_cap)` — a policy-granted cap always wins over a
> larger caller or default-value request, never the reverse. New/extended
> tests: `sea-forge-planner` `conformance_m14.rs` (`t17_0_*`: colon/empty/
> oversized endpoint_ref rejection, caller-supplied endpoint_ref binding);
> `sea-forge-cli` `conformance_m15.rs` (`t17_1`-`t17_5`: caller-above-grant,
> config-default-above-grant, authority decision hash changes with the
> requested cap, exact exhaustion at the grant cap, no further proposal
> after park); a new `sea-forge-server` `conformance_topology.rs`
> (`topology_*`: sequential steps dispatch through the real server in order
> against a stub endpoint and settle accepted; concurrent branches all
> accept and the rollup milestone fires; one required branch's real episode
> settling rejected terminates the case with `blocking_item` and the rollup
> never fires). Gate commands (`cargo test -p sea-forge-planner --test
> conformance_m14`, `-p sea-forge-cli --test conformance_m15`, `-p
> sea-forge-server topology`) all green, plus full workspace
> `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets -- -D
> warnings` / `cargo test --workspace --all-features` (91 green test
> binaries) / `devbox run -- just check` / `just proof` /
> `just no-async-kernel`.

> **2026-07-23 spec-audit-remediation Tasks 11-12 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): self-model rebuild
> is now ledgered end-to-end — `sea-forge-self-model::store::rebuild` opens a
> `self-model` `LedgerStream`, commits validation evidence
> (`self_model_verification_evidence`), a real `AuthorityDecision` for the
> `self_model_rebuild` reserved action, and a `SettlementEvent`, then threads
> those refs into every `ProjectionRecord` (`authority_refs`/`evidence_refs`/
> `settlement_ref` non-empty; a projection missing either evidence or
> settlement is `Quarantined`, never `Accepted`); release/cell realizations
> and the immutable snapshot are committed+materialized through the ledger
> instead of raw file writes, and `store::validate` verifies the ledger's hash
> chain when one exists. The CLI (`sea-forge self-model rebuild`) now reads
> real installation state — the extension registry, host-probed sandbox
> classes, and a real hash over `capabilities.jsonl` — instead of
> caller-supplied empty/placeholder inputs; `--capability-hash` was replaced
> by `--actor` (default `operator_local`), matching other governed commands.
> ODI provenance (M10) no longer ships `sha256:placeholder`/`outcome:primary`:
> `odi_adlc_case_template` takes real `seed_domain_model_ref`/
> `seed_model_sha256` parameters, its origin ref now names the real
> `"Desired Outcome Criterion"` concept, and a new `SeedModelResolver`
> (`sea-forge-planner::criteria`) verifies domain_model_ref/hash/concept
> membership/desired-outcome-class before authority. Both production plan
> ingresses (`pipeline.rs`'s intent path and `plan_pipeline.rs`'s externally
> supplied `run --plan`) now call `verify_plan_criteria_with_resolver` with a
> resolver built from Task 11's real bundled self-model seed instead of the
> fail-closed `NoModelResolver` wrapper; a submitted plan naming a known
> built-in template (`is_built_in_template_ref`) installs it through the
> existing source-owned installer (`store_builtin`, pinned under
> `<root>/templates/`) and derives criteria via `derive_from_template` —
> previously unreachable from production — instead of intent-only
> provenance. New/extended tests: `sea-forge-self-model` conformance_m9 (T9.1,
> T9.3 governance-ref assertions), `sea-forge-cli` `self_model_cli.rs`
> (ledgered-record-kinds + governance-ref assertions),
> `sea-forge-planner` `criteria_provenance.rs` (`m10_seed_resolver_*`: valid,
> missing, wrong-class, unknown-concept, model-drift, unrecognized-model),
> `sea-forge-planner` `conformance_m10.rs` (T10.3 placeholder-absence
> assertions), and a new `sea-forge-cli` `conformance_m10.rs` (T10.6 real
> production plan resolving the real seed hash end-to-end; T10.4 a tampered
> pinned built-in template rejected before authority with no allocated run).
> `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets`
> are clean; full affected-crate test suites
> (`sea-forge-self-model`, `sea-forge-planner`, `sea-forge-cli`,
> `sea-forge-server --test conformance_m12`) are green.

> **2026-07-23 spec-audit-remediation Tasks 13-13B landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): Thoth claim
> derivation (`sea-forge-thoth::engine`) is now bounded and total. A partial
> disclosure grant (e.g. only `DeclaredCapability` permitted) no longer lets
> `derive_claims` emit a stronger natural class/status than granted —
> `capped_capability_claim` picks the natural class when it's permitted, else
> downgrades to the highest permitted class *below* it and caps `status` to
> that class's evidence rung (never elevates). Every `QuestionKind` now has a
> deterministic typed handler (`AskOperationRequirements`,
> `AskAuthorityRequirements`, `AskFailureExplanation`, `AskEvidenceForClaim`
> previously fell through to an empty `_ => {}` arm); each always emits a
> claim (real or a typed `Unsupported` claim) so `Answered` never carries an
> unexplained empty claim set. `ask_why_denied` now requires and resolves a
> verified `RecordedAuthorityDecision` (new minimal `SnapshotView` method,
> default `None`) — unresolvable ⇒ `Denied`; resolved ⇒ discloses only the
> recorded `denied_classes`/`reason_code`, never fresh claims.
> `freshness_of`'s broken placeholder (both branches returned `Stale`
> regardless of `requires_fresh()`) is replaced by an explicit pre-query gate
> in `answer_question`: a required-fresh policy against a stale snapshot
> returns `Denied` before `derive_claims` calls any bounded query method
> (`capability`/`declared_capabilities`/`environment_status` — verified by a
> `CountingSnapshot` test double asserting zero calls). Task 13B adds
> immutable Thoth-claim authorship SoD at both authority-bearing boundaries:
> `GroundedClaim.authored_by` is now stamped `Some("thoth")`
> (`engine::THOTH_ACTOR_ID`) on every claim Thoth constructs; the shared
> predicate moved from `sea-forge-thoth`'s unit-test-only `check_sod` to a new
> public `sea_forge_core::types::validate_claim_authorship_sod` (approved per
> ADR-003); `SettlementDeclarationRequest`/`SettlementDeclaration` gained an
> `authored_by` field (input carried through to the persisted record, part of
> `declaration_hash`) and `sea-forge-settlement`'s `check_integrity` denies
> before acceptance when the declarer's `actor_id` matches the claim's
> `authored_by`; `sea-forge-capability::promotion::declaration_qualifies`
> independently re-checks the same invariant reading straight from the
> persisted `SettlementDeclaration` — so a declaration record copied or
> replayed directly into the promotion pipeline (bypassing `declare()`
> entirely) still can't qualify. New tests: `sea-forge-thoth` `engine.rs`
> (`t13_1_*` capping, `t13_2_*` unsupported-under-each-grant, `t13_3_*`
> per-kind totality, `t13_4_*` ask_why_denied resolve/deny,
> `t13_5_*` pre-query freshness refusal, `t13b_claims_are_stamped_with_thoth_authorship`),
> `sea-forge-settlement` `criteria_provenance.rs` (`thoth_sod_*`: same-author
> deny, different-author allow, copied/relabeled/replayed claim),
> `sea-forge-capability` `conformance_m4a.rs` (`thoth_sod_*`: same at the
> promotion boundary). Gates green:
> `cargo test -p sea-forge-thoth -- --nocapture`,
> `cargo test -p sea-forge-settlement thoth_sod -- --nocapture`,
> `cargo test -p sea-forge-capability thoth_sod -- --nocapture`,
> `cargo test -p sea-forge-thoth t11_7 -- --nocapture`. `cargo fmt --all` and
> `cargo clippy --workspace --all-targets -- -D warnings` are clean; full
> `cargo test --workspace --all-features` is green workspace-wide.

> **2026-07-24 spec-audit-remediation Tasks 14A-14B landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): Thoth now has one
> real, joined, mediated ask service instead of caller-supplied status. New
> `sea-forge-thoth::service` (`LedgerSnapshotView` + `pub fn ask`) implements
> `SnapshotView` over real state: `capability()` returns `None` only when the
> composed self-model (`ComposedModel::concept_exists`) doesn't declare the
> concept at all, otherwise rebuilds a `CapabilityRecord` purely from ledgered
> `capabilities.jsonl`/`settlement/declarations.jsonl` compatibility views
> (`build_capability_record`, tolerant of either file being absent — unlike
> `rebuild_capability`, this never writes a materialized record as a side
> effect of a read); `CapabilityStatus` maps monotonically to Thoth's
> `ClaimStatus` (`Proven`/`Metabolized → Demonstrated`, `Demonstrated →
> Validated`, `Attempted → Declared` — never over-claims). `environment_status`
> reads the real cell realization's evidenced toolchain probes;
> `declared_capabilities` is the real composed model's concept list.
> `recorded_authority_decision` resolves a verified, previously committed
> `self_disclosure_decision` ledger entry (new `Serialize`/`Deserialize` on
> `RecordedAuthorityDecision`) instead of a test double. The `DisclosurePolicy`
> is derived from the authority bundle's `self_disclosure` surface via a new
> `SelfDisclosureSurface::matching_grant` (additive helper, no schema change);
> `requires_fresh()` is true if any grant this actor holds sets
> `require_fresh_snapshot: true` (conservative — never widens disclosure).
> `ask()` validates purpose length (≤500), non-empty typed subject/actor,
> non-empty case_id-when-given, generates ULID-backed question/answer IDs
> (`sea_forge_core::ids::random_id`), and commits the complete evidence chain
> to one `thoth-asks` ledger stream in order: `self_disclosure_question` →
> `self_disclosure_plan` → `self_disclosure_decision` → `self_disclosure_answer`
> (each linked via `authority_refs` to its parent). Task 14B made CLI and
> server thin adapters over this one service: `sea-forge-cli`'s `ask.rs` no
> longer defines `SelfModelSnapshotView`/`SurfacePolicy` or loads policy
> directly — it only parses `QuestionKind` (via new shared
> `sea_forge_thoth::protocol::parse_question_kind`) and formats output.
> `sea-forge-server` gained an additive `Request::Ask` variant (ADR-003
> shape (1)/(3)) dispatching through `tokio::task::spawn_blocking` to the same
> `service::ask`, mirroring the existing sandboxed-task `spawn_blocking`
> pattern; an old server sees an unrecognized `verb` tag and fails
> deserialization cleanly (never panics, never misroutes). New tests:
> `sea-forge-thoth` `conformance_m11_service.rs` (`t14a_*`: demonstrated,
> attempted-only, unavailable-environment, absent-policy-denies,
> partial-grant-caps, stale-required-refuses-before-any-query, replay-stable);
> `sea-forge-cli` `ask_cli.rs` (`ask_with_granted_policy_answers_real_capability`,
> alongside the 3 pre-existing exit-code tests, all still green);
> `sea-forge-server` `conformance_m11_ask.rs` (allowed/denied/unknown-kind,
> full ledgered lineage assertion, wire-tag round-trip, and
> `unknown_request_verb_fails_clean_not_panic` version-skew guard). Gates
> green: `cargo test -p sea-forge-thoth --test conformance_m11_service
> -- --nocapture && cargo test -p sea-forge-thoth`; `cargo test -p
> sea-forge-cli --test ask_cli -- --nocapture && cargo test -p sea-forge-server
> ask -- --nocapture && cargo test -p sea-forge-thoth`. `cargo fmt --all --
> --check` and `cargo clippy --workspace --all-targets -- -D warnings` are
> clean; `cargo test --workspace --all-features` (90 suites), `devbox run --
> just check`, `devbox run -- just proof` (P1-P4b), and `devbox run -- just
> no-async-kernel` (19 kernel crates, still synchronous) are all green.

> **2026-07-24 spec-audit-remediation Tasks 15-16 landed**
> (`.agents/plans/2026-07-22-spec-audit-remediation.md`): delegation
> settlement/schema/termination (Task 15) and retention precedence/sealed
> summarized storage (Task 16). `Operation::AgentTask.response_schema` is now
> carried end to end: `DelegationRequest` gained `response_schema: Option<&
> serde_json::Value>`; `case_dispatch.rs::execute_agent` passes the item's
> field through instead of dropping it via `..`. A minimal, explicitly-scoped
> JSON Schema subset validator
> (`sea_forge_settlement::validate_response_schema` — `type`/`enum`/
> `properties`/`required`/`items`; no full-JSON-Schema dependency is
> approved, so unrecognized keywords are not enforced rather than pretended)
> gates settlement in `delegation.rs`: a schema-invalid final output always
> settles rejected with a typed `schema_invalid` basis; a schema-valid one is
> committed as new named evidence (`response_schema_evidence` — schema hash +
> `valid` flag only, never the raw non-conforming payload). `TurnCapExceeded`
> no longer forces rejection: when the item declares a criterion (schema
> and/or `agent_output_must_contain`) and the available final output
> satisfies it, the episode settles accepted while `turn_cap_exceeded`
> always stays in the basis (no criteria declared ⇒ unchanged prior
> behavior, rejected). `case_dispatch.rs`'s case-level completion record no
> longer constructs a synthetic `basis: ["delegation_completed"]` — it reuses
> `DelegationResult.basis` (new field), the exact basis delegation already
> committed, so cancelled/turn-capped/endpoint-error/criteria-mismatch
> episodes are never mislabeled at the case level. Task 16 added
> `sea_forge_agent::TranscriptRetentionMode` (`Summarized` default/`Full`)
> with `AgentEndpointConfig.transcript_retention: Option<_>` (endpoint level)
> and `AgentConfig.transcript_retention` (global `[agent]` default), plus
> `TranscriptRetentionMode::resolve(item_override, endpoint, agent_config)`
> implementing the full spec §8.1 precedence (plan item → endpoint → global →
> summarized default), typed-erroring on an invalid item override string
> rather than silently falling back. `case_dispatch.rs::execute_agent`
> resolves this once and passes the typed mode into `DelegationRequest`
> (new `transcript_retention` field, replacing the previously-ignored
> destructure). `delegation.rs` now branches on the resolved mode: full mode
> keeps the existing public plaintext `transcript-<hash>.jsonl` artifact;
> summarized mode seals the same canonical redacted bytes with
> `XChaCha20Poly1305` (new `sea-forge-server::transcript_seal` module, ADR-002
> — fresh random key at `.sea-forge/sealed/<run_id>.key` mode 0600, `nonce ||
> ciphertext` at `transcript-<hash>.sealed`) and verifies by decrypting
> immediately; a verification failure is captured and unconditionally forces
> the settlement rejected with a typed `sealed_verification_failed` basis
> (checked before termination/criteria — it never degrades to a
> summary-only success). The redacted `transcript_sha256` is identical
> across both modes since both seal/store the same `produce_transcript`
> output. Random key/nonce bytes use `getrandom` directly (already a
> workspace dependency at the exact pinned version via `sea_forge_core::ids`)
> rather than `chacha20poly1305`'s own `aead`/`rand_core` re-export chain,
> whose `OsRng`/`RngCore` surface has churned incompatibly across versions
> and was not going to be guessed. New tests: `sea-forge-settlement`
> `response_schema_tests` (valid/enum-mismatch/missing-required/malformed-
> json/array-items); `sea-forge-agent` `config::tests` (`retention_*`:
> full precedence chain, invalid-override-typed-error, parse rejects
> unknown/empty, absent-field version-skew defaulting); `sea-forge-server`
> `transcript_seal::tests` (round-trip, ciphertext-never-contains-plaintext-
> marker, tampered/wrong-key/missing-key/missing-ciphertext all fail
> verification, crypto-shred permanently unrecoverable, restart reads durable
> disk state not memory); `sea-forge-server` `conformance_m13.rs` `t15_*`
> (schema valid/invalid with named evidence, turn-cap-with-satisfied-
> criteria accepts and retains basis, case-dispatch reuses real basis not
> synthetic — via a genuine `Request::Submit` end-to-end dispatch, the first
> in this test file) and `t16_*` (redaction digest identical across full/
> summarized modes with mode-specific artifact visibility, case-dispatch
> resolves the full item/endpoint/global/default precedence chain end to
> end, sealed-verification-failure settles rejected never summary-only
> success). Pre-existing M13/M16 tests that read the plaintext transcript
> artifact (`t13_transcript_artifact_hash_verifies` and three ACP `t16_*`
> tests sharing the `execute_fixture` helper) now explicitly request
> `TranscriptRetentionMode::Full`, since the default changed from
> unconditional-full to spec-correct summarized. Gates green: `cargo test -p
> sea-forge-server --test conformance_m13 schema -- --nocapture && cargo
> test -p sea-forge-server --test conformance_m13 turn_cap -- --nocapture &&
> cargo test -p sea-forge-server --test conformance_m13`; `cargo test -p
> sea-forge-server --test conformance_m13 retention -- --nocapture && cargo
> test -p sea-forge-server --test conformance_m13 redaction -- --nocapture`.
> `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets
> -- -D warnings` are clean; `cargo test --workspace --all-features` (90
> suites, 0 failures), `devbox run -- just check`, `devbox run -- just proof`
> (P1-P4b), and `devbox run -- just no-async-kernel` (19 kernel crates, still
> synchronous) are all green.

> **2026-07-23 review remediation landed** (commit 47608a0): addressed 30/32
> full-spec review findings across `sea-forge-cell`, `sea-forge-case-runner`,
> `sea-forge-agent`, `sea-forge-capability`, `sea-forge-domainforge`, and the
> frontend spec/ADR/CURRENT_STATUS docs. **Follow-up resolution:** review
> findings #6 and #16 are now implemented on branch
> `fix/domain-model-validation`: `load_validate` reports the documented
> `ForgeError::Plan { class: "domain_model_error" }`, post-start
> `ForgeError::Run` failures retain their existing `internal_error` trace
> class, and `MAX_IMPORT_DEPTH=16` is enforced
> from DomainForge's existing public canonical semantic-envelope import
> graph (no DomainForge API or release change). The depth traversal derives
> the unique zero-inbound canonical closure root, so normalized entry spellings
> such as `./entry.sea` cannot bypass the limit. `devbox run -- just
> context-check`, `check`, and `test` are green; the two configured real-host
> release tests remain intentionally ignored. fmt + clippy `-D warnings` +
> `test --workspace --all-features --locked` were green for the original
> 30/32 remediation.
> **Gitleaks fix:** added `.gitleaks.toml` (path allowlist for `.entire/`
> and numbered checkpoint transcript dirs) + inline `// gitleaks:allow`
> on test-fixture lines to stop false-positive regeneration under new
> commit hashes; `gitleaks detect` now reports "no leaks found".

## Objective

- **2026-09-09 Objective**: Restructure SEA Forge agent instructions using the repository's actual semantic and architectural topology into a progressive disclosure hierarchy (root `AGENTS.md`, `crates/AGENTS.md`, `workbench/AGENTS.md`, `.agents/AGENTS.md`). (Settled).
- **2026-09-08 Objective**: Implement the recommendations described in `.agents/reports/domainforge-sxr-sea-rs-cep-interface-investigation.md` to unify and remediate the DomainForge, SXR, SEA-Forge, and CEP interfaces across repositories. (Settled).

Implement `.agents/plans/2026-08-02-workbench-product-completion-ralph-loop.md`
in its required order. Tasks 0 and 1 are `PASSING` on the current tree: the
evaluator packet has an exact machine-checked exclusion fixture, and protected
renderer actions fail closed when a server-derived identity is unavailable.
Task 2 is complete at the focused-slice level: readiness and approval standing
resolve only to typed committed records (or explicit unknown/stale/blocked
truth), and bridge refusals preserve their class, no-side-effect standing, and
next lawful action. The generated-contract clean-tree gate is intentionally
pending an eventual commit; it must not be bypassed by staging only generated
files. Task 3 is blocked at its explicit native-driver dependency decision:
the repository has mocked Playwright but no installed Tauri-native automation
driver, and the plan forbids calling the mock integrated proof.

Current documentation task: make `just` the agent-facing command-line interface.
All active agent-facing guides and diagnostics must name only `just` commands;
Devbox, Cargo, and Bun remain recipe implementations. Add generic crate-scoped
check/test and missing Workbench recipes rather than exposing a second CLI.

Close out `.agents/plans/2026-07-22-spec-audit-remediation.md` with fresh,
portable cumulative conformance evidence and claim tables that preserve
platform/real-host skips. M9–M11 + CEP-0008 adapter complete. M12 (Task 4)
COMPLETE and gated. M13 (Task 5)
COMPLETE and gated: all T13.1–T13.7 conformance rows green, including T13.2
mixed sandboxed/agent dispatch with replayable ordinals (`sea-forge-case-runner`
extraction, server-owned per-episode dispatcher, `sea-forge ledger replay`).
M14 (Task 6) COMPLETE and gated (see the M14 entry below): typed deterministic
item expansion, all-of entry-criteria rollup mode, and the built-in
`sequential_agents@0.1.0`/`concurrent_agents@0.1.0` topology templates.
M15 (Task 7) COMPLETE and gated (see the M15 entry below): deterministic
manager-loop judgment (satisfied/blocked/progressing/stalled), `ManagerIteration`
ledger record, discretionary `agent_task` proposal through the existing
add-task path, iteration-cap park+escalate through the existing approval
mechanism, and a structural SoD gate on approval resolution. M16 ACP portable
coverage is green: ACP v1 session driver, durable permission records/approvals,
planned-run cancellation, Landlock jail support, bounded transport/transcript,
continuation recovery, and SWE_SEED proof harvesting. Late SWE_SEED
declaration reconciliation (Task 18, `sea-forge-server::swe_seed_reconciliation`)
is now COMPLETE and gated: a declaration submitted by the server's own
production ingress, or appended out-of-process at any later time, correlates
to its exact run idempotently, at settlement, at server startup, and at read
time (see the Task 18 entry above). Real ACP/SWE_SEED-host release tests
remain intentionally ignored until operator configuration is available.

Historical M13 progress log (kept for context, superseded by "COMPLETE" above):
T13.2 (Task 5) in progress:
slices 5.2, 5.4, server delegation service, CLI delegate command, and
T13.4 token-budget test all landed. 420 tests pass. The M13 conformance
table now records T13.4 green and the remaining tests pending. Next:
slice 5.3's first vertical path is landed: a caller-supplied run ID can be
cancelled through the server only after a `run_cancel` authority decision and
append-only `control_request`; the provider loop makes a cancellation that
races with a response settle rejected. T13.3 remains pending its three-run,
restart, and successor-episode conformance path. Slice 5.5/T13.5 output
criterion landed: an additive `agent_output_must_contain` field on
`SettlementCriteria` rejects an agent's narrated success when the final
response lacks the required literal. T13.6 retention design landed: full
mode persists the redacted canonical transcript artifact and a test
recomputes its SHA-256 to match the recorded `transcript_sha256`. T13.1
plan-acceptance half landed: the case engine accepts `agent_task` items
and a downstream sentry gates on the agent's settlement status; full
execution routing still pending. T13.2 semaphore cap landed: 4 concurrent
agent delegations with `max_concurrent_runs=2` never exceed 2 in-flight
connections. T13.7 hostile tool requests landed: `AgentToolCall` type
models tool/function calls in provider responses; the delegation loop
unconditionally records `tool_request_denied` per call, no side effects,
dialogue settles on criteria. T13.3 cancel-one-of-three landed: three
concurrent delegations, cancelling the middle one mid-flight settles it
rejected/cancelled while siblings settle accepted/completed. 433 tests pass. T13.3 HTTP
restart recovery landed: startup scans verified durable cancellation controls,
writes one rejected/cancelled terminal settlement and evidence, and a second
restart appends nothing. T13.1 full routing landed: an end-to-end CLI integration test starts
the server, routes `agent_task` to it, emits `SettlementRecorded`, unlocks
the downstream sandboxed task, and completes the case. Next: M13 review and
any deferred ACP continuation work belongs to M16.

Repository search and scoped Understand Anything guidance is staged for its
own documentation commit. Next implementation gap: T13.2 mixed sandboxed and
agent load with replayable dispatch/settlement ordering.

T13.2 design approved: replace the whole-case `Request::Submit` subprocess
path with server-owned, per-episode dispatch under the existing semaphore;
persist dispatch and settlement ordinals and add ledger replay. Implementation
 plan: `.agents/plans/2026-07-20-m13-mixed-dispatch.md`.

T13.2 implementation complete through Task 4: case-runner extraction (Task 1),
delegation episode context (Task 2), server-owned per-episode dispatcher under
the sole shared semaphore with dispatched-failure settlement, human-task
drain, and stale-action re-derivation (Task 3), and additive
dispatch/settlement ordinals plus `sea-forge ledger replay --case` (Task 4).
CLI and server conformance pass. Task 4 review nit fixed (replay rustdoc).
Next: full workspace proof, final whole-branch review, and graph refresh.

T13.2 Task 1 extracted synchronous case lifecycle primitives into the approved
`sea-forge-case-runner` workspace crate; the CLI is a compatibility facade and
focused CLI conformance plus runner check pass. Task 2 binds delegation to an
episode context: planned settlements now retain their submitted case/item/run,
while direct `Request::Delegate` continues to create its standalone context.
Focused planned-context and cancellation regressions pass. Next: add the
server-owned episode dispatcher. Task 2 review follow-up preserves planned
case/item/run identity in restart-recovered evidence and settlement, validates
the cancellation control against the persisted plan, and proves one target
settlement despite unrelated prior settlement records. The final Task 2 review
fix also serializes planned case/item/run identity in normal and rejected
delegation evidence; planned success and rejection conformance assertions pass.

Task 3 dispatcher landed: `Request::Submit` now creates the case in-server and
dispatches sandboxed and agent episodes under the existing `ServerState.semaphore`.
Each dispatch is committed before its episode starts; a completed episode is
settled and applied before the reducer derives more ready work. T13.2's five-item
mixed-load conformance path passes with `max_concurrent_runs=2`, alongside all 13
server M13 conformance tests. `devbox run -- just context-check` passed after this
status update. Commit: recorded in git history.

Task 3 review follow-up: the dispatcher now waits on the sole shared permit,
prioritizes recording returned active completions, converts post-dispatch errors
into terminal rejected settlements, and drains active siblings before case
termination. Two focused regressions plus all 15 server M13 conformance tests,
formatting, and server check pass. Commit: pending.

Task 3 human-task review follow-up: ready human tasks now persist their normal
activation event and drain already-dispatched episodes to terminal settlement
before `Submit` returns active. All 16 server M13 conformance tests, formatting,
and server check pass. Commit: pending.

Task 3 final review follow-up: human holds now process every equally-ready
executable action in either plan order before draining settlements, and a
completion during permit wait restarts case reduction rather than processing
stale actions. All 17 server M13 conformance tests, formatting, and server check
pass. Commit: pending.

Task 3 non-executable activation follow-up: the dispatcher now rejects
`CaseAction::Activate` for item kinds it cannot dispatch (Stage,
TimerListener, UserEventListener) with a typed `ForgeError::Input` before
any side effect, instead of panicking in the activation payload. New
`t13_2_non_executable_activation_returns_typed_error` regression; all 18
server M13 conformance tests, formatting, and server check pass.
Commit: pending.

Task 4 review follow-up: dropped the completion-theater reducer invocation
in `commands::ledger::replay_case` that silently swallowed plan-load and
reducer errors (replay now trusts `case-events.jsonl`); added three
focused unit tests for `validate_ordinals` rejection paths (missing
dispatch_ordinal on ItemActivated, duplicate equal dispatch_ordinal,
non-monotonic lower settlement_ordinal) plus an accepts-monotonic
control, each asserting `ForgeError::Input`; spec-agent-orchestration.md
T13.2 status row flipped from "partial" to green, citing
`t13_2_mixed_episodes_share_server_cap`, `t13_2_replay_matches_persisted_order`,
and `sea-forge ledger replay --case`, with a note that replay applies
only to cases created after the additive ordinal change (no migration).
All 3 CLI M13 conformance tests, 18 server M13 conformance tests, the
4 new ledger unit tests, fmt, and `cargo check` on CLI+server+case-runner
pass. Commit: pending.

T13.2 final whole-branch review follow-up (three fixes):

- `sea-forge-case-runner` added to the `no-async-kernel` `kernel_crates`
  array; `just no-async-kernel` now covers 19 kernel crates (was 18).
- `SettlementRecorded` payload in `case_dispatch::record_completion`
  now carries `run_id`, so `ledger replay --case` correlates dispatch
  to settlement by run_id; `t13_2_mixed_episodes_share_server_cap`
  extended to assert each settlement's run_id maps to a same-item
  dispatch.
- `SubmitPayload.intent` documented as ignored by server-owned dispatch
  (intent-only submit is no longer supported via `Submit`); field
  retained for deserialization compatibility.
All 18 server M13 conformance tests, 3 CLI M13 conformance tests, fmt,
and `just no-async-kernel` (19 crates) pass. Commit: pending.

## M14 topology templates (2026-07-21)

Implementation plan: `.agents/plans/2026-07-21-m14-topology-templates.md`.
M14 (Task 6 of the ADLC/Thoth orchestration plan) is COMPLETE and gated:

- Additive `EntryCriteriaMode::{Any,All}` on `PlanItem`/`TemplateItem`
  (`Any` default, byte-compatible with every pre-existing template).
  `case_engine::entry_criteria_satisfied` now supports all-of rollup gating
  alongside the unchanged OR-of-sentries default.
- Typed deterministic item expansion: `TemplatePlan.repeated: Vec<RepeatedItem>`
  expands a shared item body into N items with IDs derived from
  `{id_prefix}_{key}`, bounded by `MAX_REPEATED_ENTRIES` (32), duplicate
  entry keys rejected, forbidden-substitution sites (including the new
  `TemplateOperation::AgentTask.endpoint_ref`) enforced on the shared body
  before expansion.
- `TemplateOperation::AgentTask` closes a real prior gap — templates could
  not express `Operation::AgentTask` at all before this change.
- Built-in `sequential_agents@0.1.0` (steps chained via per-entry
  settlement-accepted sentries referencing the prior step's deterministic
  ID) and `concurrent_agents@0.1.0` (independent branches plus a flat
  `Milestone` rollup with `entry_criteria_mode: All`) registered through
  the existing `store_builtin` installer, no new CLI wiring.
- `conformance_m14.rs`: T14.1 (deterministic ×2 instantiation, chained
  gating), T14.2 (rollup fires only when all N branches settle accepted),
  T14.3 (one branch rejected plus an unrelated rejection never fires the
  rollup — proves source-binding, not leakage), plus mechanism-level unit
  tests for expansion validation and all-of semantics — 9/9 pass.
- Stub agent endpoints in the built-in templates prove scheduler/rollup
  behavior only; they do not upgrade the spec §5 "real agent latencies"
  claim, which stays open for M16 real integration.

456 workspace tests pass (`cargo test --workspace --offline`), fmt clean,
`just no-async-kernel` still covers 19 kernel crates (no new crate added —
`sea-forge-core`/`sea-forge-planner` remain sync/no-HTTP). `.agents/specs/spec-agent-orchestration.md`
§17.3 T14.1–T14.3 rows flipped to green.

## M15 Thoth manager loop (2026-07-21)

Implementation plan: `.agents/plans/2026-07-16-adlc-thoth-agent-orchestration.md`
Task 7. M15 (E16b) is COMPLETE and gated:

- `ManagerJudgment`/`ManagerAction`/`ManagerIteration` additive types on
  `sea-forge-core`; `sea_forge_thoth::manager::judge` is a pure, IO-free
  function classifying `satisfied | blocked | progressing | stalled`
  deterministically from caller-supplied facts (§9.5's rule table exactly —
  completed beats blocked beats ready-work/settlement-progress beats
  stalled), mirroring `engine.rs`'s existing `SnapshotView`-driven pattern
  rather than adding a new crate dependency.
- `crates/sea-forge-cli/src/commands/manager.rs` (`case manager-iterate`
  subcommand): builds the view from `next_case_actions` plus a
  `replay_case` scan for items already `Enabled`/`Active` from a prior
  tick (an item transitions out of `next_case_actions`'s output once
  enabled, but per §9.5 "active or enabled" still counts as progressing —
  a real gap the first test pass caught); tracks settlement progress via
  a `settlement_events_observed` counter compared against the prior
  iteration's recorded count; commits one `ManagerIteration` ledger record
  per invocation (`record_kind: "manager_iteration"`, same `case-<id>`
  ledger stream as everything else about the case).
- Additive `PlanItem.proposed_by: Option<String>` — part of the plan's
  canonical hash, so relabeling/replaying a proposed item cannot strip the
  provenance. `case.rs::add_task` refactored into a thin file-reading
  wrapper around a new `propose_item(root, policy, actor, case_id, item)`,
  reused directly by the manager loop for in-memory synthesized items
  (`item_kind: agent_task`, `proposed_by: Some(actor)`) — no new mutation
  path, no duplicated authority/ledger-commit logic.
- Denied proposals (T15.2) are caught and recorded as the iteration's
  `granted: false` outcome, never a hard error — matches §7.6 "recorded
  to the ledger whether or not the proposal is granted."
- Iteration-cap exhaustion (T15.3) is a guard *before* judging (per §16.2's
  pseudocode) — no `ManagerIteration` record for the cap-exceeded call
  itself, reuses the exact `ApprovalRequest` + `CaseState::AwaitingApproval`
  pattern `plan_pipeline.rs` already uses for settlement escalation.
- SoD (T15.4): `approve.rs::resolve()` gained a `proposed_by`-based check,
  independent of and prior to the existing requester-based check — an
  actor cannot resolve an approval for a plan item it proposed.
- Two real, previously-latent authority-layer gaps found and fixed along
  the way (both are additive allowlist entries, not behavior changes for
  existing kinds): `sea-forge-authority`'s `PolicyRule.operation_kind`
  validation allowlist and its separate `malformed_action` `RESERVED`
  allowlist were both missing `manager_iteration` — every `Reserved`
  resource_type needs an entry in *both* lists or every policy decision
  for it defaults to hard deny regardless of matching rules. This also
  means `discretionary_task_add` (M10, `case add-task`) was reachable via
  CLI for the first time here — nothing else in the workspace exercised
  it end-to-end before this milestone.
- `crates/sea-forge-cli/tests/conformance_m15.rs`: T15.1–T15.5, all
  subprocess-driven through the real `sea-forge` binary (`run --plan`,
  `case manager-iterate`, `approve`) rather than in-process calls, since
  `sea-forge-cli` is bin-only (no `lib.rs`) — 5/5 pass.

465 workspace tests pass (`cargo test --workspace --offline`), fmt clean,
`just no-async-kernel` still covers 19 kernel crates (`sea-forge-cli`/
`sea-forge-authority` are not kernel crates). `.agents/specs/spec-agent-orchestration.md`
§17.4 T15.1–T15.5 rows flipped to green; the "Manager loop bounded,
evidence-grounded, SoD-enforced, escalating on exhaustion" checklist item
closed.

## M16 ACP + SWE_SEED (2026-07-21, partial)

ACP code gate is complete; SWE_SEED proof harvesting is complete; declaration
reconciliation and real-host evidence remain:

- `sea-forge-agent::acp` speaks ACP v1 JSON-RPC/NDJSON with official
  `mcpServers`, `prompt`, `sessionUpdate`, stop-reason, and capability-gated
  `session/load` shapes. Spawn uses tokenized argv, an explicit minimal env,
  shell rejection, process-group cleanup, bounded stderr drain, bounded input
  lines, and bounded cumulative transcript.
- ACP permissions commit exact hashed request data, an authority decision, a
  durable `permission_request`, and (when escalated) an ordinary
  `approval_request`. Broker wake-up only causes a ledger reload; permission
  allow requires an independently committed approved resolution and the
  existing `grant_after_approval` exact-action validation. Deny/timeout stays
  inside the session. Duplicate wake-ups are rejected.
- Server-dispatched agent tasks now register cancellation handles too. Restart
  recovery turns orphaned ACP approval episodes into one rejected
  `acp_disconnect` settlement with a continuation record; successor episodes
  call capability-gated `session/load` under the same key.
- `sandbox_class: jail` ACP children are spawned on a Landlock-restricted
  thread; portable proof denies an outside `/tmp` write while allowing the run
  workspace, and rejects a session-mode escalation.
- SWE_SEED config requires an explicit repo+commit pair. Commit verification,
  safe `.agent-harness` traversal, symlink/type/count/size/hash validation,
  run-bound harvested refs, and a `swe_seed_correlation` ledger record are
  portable-tested. M4a declarations are resolved by immutable run ID when
  present. This is not yet a reconciliation path for declarations arriving
  after episode settlement; real SWE_SEED/transport evidence remains a release
  gate.
- `crates/sea-forge-server/tests/conformance_m16.rs`: 12 portable tests pass;
  two real-host tests are ignored and reported skipped by design.

Verification: isolated full workspace `cargo test --workspace --all-features
--locked` passed (the normal target had corrupted incremental linker objects
after an interrupted build; no source artifact was deleted). Focused ACP,
M12, M13, M16 tests and clippy all passed.

## Spec-audit remediation Tasks 7-8: SQLite FTS memory index + governed recall (2026-07-22)

Implementation plan: `.agents/plans/2026-07-22-spec-audit-remediation.md`.
Task 7 and Task 8 are COMPLETE and gated:

- Task 7 replaces the prior JSON `memory/index.json` projection (a documented
  `ponytail:` compromise) with `memory/index.sqlite`: an FTS5 virtual table
  (`memory_fts`) plus a `meta` table committing a SHA-256 digest of
  `items.jsonl` at rebuild time. `crates/sea-forge-capability/src/memory.rs`
  rebuilds atomically (temp file → `PRAGMA integrity_check` → rename) and
  `query_index` returns `None` (forcing linear-scan fallback) whenever the
  index is missing, corrupt/truncated, or its stored digest no longer matches
  the current `items.jsonl` bytes — freshness is proven by a source
  commitment, never inferred from successful deserialization. Filtering stays
  identical to the linear scan (exact-match SQL pushdown for entity/process/
  kind plus a shared Rust substring filter) rather than relying on FTS5
  `MATCH` token semantics, so indexed and fallback results are provably
  identical, including mid-word substrings that a tokenizer would miss.
  14 conformance tests in `crates/sea-forge-capability/tests/conformance_m4b.rs`.
  **Dependency note:** ADR-002 originally approved `rusqlite = "0.40"`
  (**superseded**), but `libsqlite3-sys 0.38.1`'s `build.rs` unconditionally
  invokes the still-unstable `cfg_select!` macro (rust-lang/rust#115585) and
  fails to compile on this repo's pinned `rustc 1.92.0`. ADR-002 was amended
  in place to pin `rusqlite = "0.32"` (→ `libsqlite3-sys 0.30.1`, still
  FTS5-bundled, same license) — the **final approved dependency version**,
  not a technology substitution.
- Task 8 splits `crates/sea-forge-cli/src/commands/recall.rs` into two
  contracts: the legacy capability-envelope path now prints matched
  `capabilities.jsonl` envelopes completely unchanged (the prior
  `record_assurance`-injected `"assurance"` field is removed — that helper
  is now dead code and was deleted from `mediated.rs`); the `--kind`
  memory-recall path binds exact `entity_id`/`requester_entity`/`process_id`/
  `kinds`/`limit` into the `recall_memory` authority action (an omitted
  `--entity` defaults the target to the requester's own identity, so `own`
  scope authorizes implicit self-recall without an extra flag), re-applies
  the granted `memory_scope` at the executor via `sea_forge_authority::
  scope_allows` independent of the query filter (defense in depth against a
  query-layer bug), and commits a `recall_evidence` ledger record (new
  `ledgers/memory-recalls/` stream) naming the request scope and every
  returned memory ID *before* printing any result. A denial short-circuits
  inside `PolicyAuthorityEngine::grant()` before the callback that would read
  `memory/items.jsonl` ever runs, so cross-entity denial reads nothing and
  commits no evidence. Assurance moved from the removed envelope mutation to
  the governed path: computed once via `mediated::assurance()` and exposed
  through the evidence record plus a structured `tracing::info!` line (the
  pre-existing `required_integrity_checkpoint_precedes_command_start_and_
  witness_outage_halts` lifecycle test was updated to assert it there
  instead of in compat-recall stdout). 1 new lifecycle.rs test (byte-for-
  structure, no injected assurance, capabilities.jsonl untouched) plus 7
  tests in new `crates/sea-forge-cli/tests/conformance_m4b.rs` (own/cross-
  entity/`entity:<id>`/any scope, evidence-ID exactness, limit, SQLite/
  fallback CLI-level equivalence).

**Task 7-8 gate results (historical context):**
- **Pre-fix result** (`just check` snapshot immediately after Tasks 7-8 code
  landed, before the gitleaks `.gitleaksignore` baseline was refreshed):
  `cargo fmt --all -- --check` ✅; `cargo clippy --workspace --all-targets
  --all-features --locked -- -D warnings` ✅; `cargo test --workspace
  --all-features --locked` ✅; `cargo deny check` (advisories/bans/licenses/
  sources) ✅; `devbox run -- just proof` (P1-P4b) ✅; `devbox run -- just
  no-async-kernel` (19 kernel crates, `rusqlite` confined to
  `sea-forge-capability`, synchronous) ✅; but `devbox run -- just check`'s
  `security` sub-recipe FAILED on 3 pre-existing `gitleaks` findings from
  commits `63a746c3`/`aa9d65bb2` (test-fixture fake secrets in
  `sea-forge-agent`/`sea-forge-server` and a `.entire/metadata/**` session
  artifact) that predate this plan and are unrelated to Tasks 7-8; a
  fmt-only whitespace fix was also applied to the already-modified,
  unrelated `sea-forge-domainforge/tests/conformance_m0_domainforge.rs` to
  unblock `fmt-check`, with no semantic change.
- **Post-fix result** (the frozen clean baseline captured under Task 0 at
  line 1223+: `just context-check && just check && just test && just proof
  && just no-async-kernel` — all green, `gitleaks detect` reports "no leaks
  found" after the four fingerprints were re-added to `.gitleaksignore`).
  The Task 7-8 gate is therefore considered PASSED on the post-fix
  baseline; the pre-fix `security` sub-recipe failure is historical and was
  not caused by Tasks 7-8 code.

## Spec-audit remediation Tasks 9, 10A, 10B: M5 governed stage episodes + CLI project (2026-07-23)

Implementation plan: `.agents/plans/2026-07-22-spec-audit-remediation.md`.
Tasks 9, 10A, and 10B are COMPLETE and gated:

- Task 9 replaces order-only stage validation in `sea-forge-spec-pipeline`
  with canonical-chain validation: `validate_stage_prerequisites` checks
  that a stage's declared input resolves either to an earlier stage's
  matching-path output (requiring that predecessor's status to be
  `Accepted`, and its hash/`schema_ref`/`domain_model_ref`/
  `domainforge_version` to match exactly) or, when no local predecessor
  exists, to a fully self-verified externally supplied file (its own
  `schema_ref` present). Three additive `Option<String>` fields
  (`schema_ref`, `domain_model_ref`, `domainforge_version`) were added to
  `StageFile` in `sea-forge-core::types` (ADR-003-approved shape (1)
  addition; `StageFile` gained `Default` to keep every existing struct
  literal compiling). `process_pipeline` now quarantines any stage —
  including one an executor self-reports `Accepted` — whose prerequisite
  chain fails, before classification runs. `compute_proof_classification`
  no longer grants `generated-contract`/`focused-slice` from a sparse stage
  list: it requires the *entire* canonical-order prefix (`Adr..
  GeneratedContract`, then `LastMileAdapter`/`RuntimeWiring`/
  `AcceptanceProof`) to be present and `Accepted`, closing the audited
  defect where four sparse stages could claim focused-slice. 15 new tests in
  `crates/sea-forge-spec-pipeline/tests/conformance_m5.rs` (missing/
  skipped/rejected/quarantined/unhashed/schema-mismatched/model-mismatched/
  version-mismatched predecessors, valid chain, externally-supplied input,
  process_pipeline cascade) plus a reversed internal unit test proving the
  sparse-focused-slice defect is closed.
- Task 10A adds `run_stage_case`/`run_stage_episode` to
  `sea-forge-case-runner`: a synchronous driver that builds the case
  (ledger stream, case/plan JSON, `CaseCreated`), then loops
  `CaseRunner::next_ready_actions` exactly like the existing server
  dispatcher, dispatching `Activate` through the same
  authority-evaluate → ledger-commit → grant → `sea_forge_runtime::execute`
  → settle pattern used elsewhere (mirroring
  `sea-forge-server::case_dispatch::execute_sandbox`). Two structural gates
  run before authority: Task 9's `validate_stage_prerequisites` (a failure
  quarantines the stage with basis `prerequisite_invalid`, no authority
  call), and a generated-zone guard (§10.7): a non-generator-kind stage
  (i.e. not `Ast`/`Ir`/`Manifest`/`GeneratedContract`/`SemanticFixture`)
  declaring an output path under a generated zone is quarantined with basis
  `generated_zone_direct_edit`, also before authority. `sea_forge_planner`
  gained `stage_case_plan`, converting `Vec<SpecPipelineStage>` into a
  `CasePlan` of `SandboxedTask` `PlanItem`s chained by the same
  `reactivation_sentry` helper `sequential_agents_template` already uses
  (settlement-accepted sentry referencing the prior stage), each
  `markers.required: true` (an unrequired item can be silently skipped by
  `can_auto_complete` before ever activating — a real gap the first test
  pass caught) and `sandbox_class: "jail"` (a stage's command is arbitrary
  tooling, not necessarily the trusted `sea-forge` binary, so it cannot
  rely on the `local` class's trusted-argv0 policy exemption). `sea-forge-spec-pipeline`
  stays pure — no scheduling, filesystem writes, or ledger commits live
  there. 4 conformance tests in `crates/sea-forge-case-runner/tests/conformance_m5.rs`
  (accepted stage completes the case; denied generated-zone edit quarantines
  before authority; broken-prerequisite stage quarantines; a rejected
  predecessor's downstream settlement-accepted sentry never fires, leaving
  the successor `Pending` and the case `terminated`) using a self-invocation
  idiom (mirroring `sea-forge-sandbox`'s `net_probe_helper`) since
  `ExecuteCommand`'s hard `untrusted_executable` invariant only trusts the
  exact running process's own executable — no test may shell out to `sh`.
- Task 10B adds `sea-forge project <entry.sea>`: builds the ADR/PRD/SDS/SEA/
  AST/IR/Manifest/GeneratedContract stage chain with real content (authored
  doc text; the actual `.sea` source; `{:#?}` of DomainForge's parsed graph;
  the validated `DomainModelRef`; a manifest JSON; the CALM projection as
  the generated contract), each stage hash-linked to its predecessor's
  output, runs it through Task 10A's `run_stage_case`, then commits a
  `SpecPipelineRun` (via Task 9's `process_pipeline`, proving the achieved
  `proof_classification`) and independently settles a `ProjectionRecord` +
  `SettlementEvent` pair per requested `ProjectionKind` (default `calm`,
  `rdf`) from the one validated model — an unsupported kind (e.g. `sbvr`,
  which `sea_forge_domainforge::project` already rejects) is quarantined
  with a `Rejected`-status `ProjectionRecord` recording the failure reason,
  never silently dropped or accepted. New hidden `sea-forge stage-check
  <file> <sha256>` subcommand (config-free, same shape as the existing
  hidden `validate`) is the only thing a stage's `ExecuteCommand` ever runs,
  since `sea_forge_authority::untrusted_executable` requires argv[0] to be
  this exact running binary. SEA Forge (this command) performs every
  authorized filesystem write; `sea_forge_domainforge::{load_validate,
  project}` remain pure/in-memory. 2 conformance tests in
  `crates/sea-forge-cli/tests/conformance_m5.rs`: the full chain settles
  with `proof_classification=GeneratedContract` and both projections
  accepted, every expected ledger record kind present, and no unexpected
  top-level filesystem entries under root; the negative case shows a
  non-zero exit, `projections_quarantined=1`, and a `Rejected` `sbvr`
  `projection_record` in the ledger.

Full workspace gate passed: `cargo fmt --all -- --check`, `cargo clippy
--workspace --all-targets --all-features --locked -- -D warnings`, `cargo
test --workspace --all-features --locked` (all crates green), `devbox run
-- just proof` (P1-P4b), `devbox run -- just no-async-kernel` (still 19
kernel crates — no new kernel crate added; `sea-forge-spec-pipeline` and
`sea-forge-domainforge` are used only by non-kernel `sea-forge-cli` and by
already-kernel `sea-forge-planner`/`sea-forge-case-runner`, both of which
remain synchronous).

## SodRule transition scope closeout (2026-07-17)

- Added additive `SodRule.transition_kind: Option<String>` with omitted-None
  serialization for policy-hash compatibility. Validation now rejects unscoped
  or unknown `transition_artifact_stage` selectors and selectors on other
  operations.
- A single fail-closed action matcher enforces requester role, canonical action
  operation, and transition selector in both policy evaluation and
  post-approval grants. The v0.2 capitalization SOD rule now targets
  `transition_artifact_stage` / `capitalize`; non-R-SO resolution remains
  rejected.
- Proof passed: `cargo fmt --all -- --check`; `cargo check -p
  sea-forge-authority`; `cargo test -p sea-forge-authority --locked`; M8 CLI
  and artifact-IP tests; `devbox run -- just context-check`, `just check`, and
  `just test`. No tests skipped.

## Worktree State

2026-09-08: Preserved unrelated worktree files (`.jolli/jollimemory/*`, `.agents/plans/*`, `clickhouse`). Modified `crates/sea-forge-domainforge/src/lib.rs`, `crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs`, `crates/sea-forge-extension/src/cep0008.rs`, and updated `docs/subsystems/domainforge-boundary.md`, `.agents/reports/domainforge-sxr-sea-rs-cep-interface-investigation.md`, `.agents/CURRENT_STATUS.md`, and `.agents/current_status.yml`.

2026-08-02 Task 0: preserved unrelated dirty changes to the active completion
plan (Markdown table formatting only) and `.jolli/jollimemory/debug.log`
(one Jolli diagnostic line). Current Task 0 adds only evaluator-input
documentation, its JSON exclusion fixture, and the validator/`just` recipe;
no product behavior, dependency, persisted schema, or CI aggregation changed.

2026-08-02 Task 1: added the shared renderer affordance guard, a session-only
selector for server-advertised actors, bounded `actAs` forwarding, and usable
identity repair links. It covers case creation and commit, approval decisions,
delegation cancellation, and the recorded `thoth.ask` command. The unrelated
completion-plan Markdown formatting and Jolli diagnostic remain preserved.

2026-08-02 Task 2: added additive committed-source projections and regenerated
their TypeScript/AJV contracts. The source and approval changes are intentionally
uncommitted alongside the rest of this implementation. The contracts drift gate
therefore reports the expected uncommitted generated projection; do not stage
only those files merely to make that pre-commit guard pass.

2026-08-03 Task 4 is in progress at the fresh-root entry slice. A missing or
empty configured root now remains `initialization_required` until an operator
confirms initialization through the closed host bridge; opening the application
does not create it. The supervisor rechecks the root under its lifecycle lock
before spawning and refuses if history appeared meanwhile, with no socket or
record write by this path. The Readiness page exposes one initialization action,
does not offer case creation early, and displays a retryable structured refusal
with its next lawful action. The host gate passed all 34 lib, bridge, and
packaged-stack tests, but its final host-wide format check remains blocked by
pre-existing formatting drift in `bridge.rs`; it was left untouched. `just
fmt-check`, `just context-check`, and `git diff --check` passed. `just
workbench-check` remains blocked before renderer tests by the already-uncommitted
Task 2 contract generation drift; selection, recognized-history migration/version negotiation,
and the Task 4 real-cell matrix are still open. The real packaged fresh-cell
proof is now `just workbench-e2e-real initialization`: it starts with no root or
socket, finds and clicks the rendered initialization control through native
WebKit/Tauri, then verifies the bundled sidecar's SFWP hello; it passed on
2026-08-03. The earlier plan filter `cell|readiness|Thoth` matched no native
scenario and is not valid evidence. The supervisor also now fail-closes before
sidecar startup for a malformed or unknown fixed-path self-model manifest;
compatible and legacy history retain the existing startup path, but no migration
is yet claimed. `just workbench-e2e-real "hello|identity|reconnect|request
recovery"` also passed after the fresh-root changes, preserving the seeded
existing-history packaged path; that is regression evidence only, not an
operator-visible selection or migration claim.

The worktree contained unrelated user changes before the `AGENTS.md` refactor;
they remain untouched. The pre-existing uncommitted additions to `AGENTS.md`
were consolidated rather than discarded.

On 2026-07-24, `full-spec` was merged into `main` as `74dc8ab` after a
fast-forward update from `origin/main`. The integrated tree passed
`devbox run -- just ci`; publication is pending the pre-push context gate after
  this final status refresh. The final Workbench grounding report and its two
  out-of-scope debt entries are intentionally included in the pending handoff
  commit, along with the current `prove_entry` source-line reference in the
  Workbench repository map.

On branch `full-spec` at `ca11dc2` before the Task 19 documentation closeout.
The worktree was clean at Task 19 start. Task 19's changes are limited to the
remediation plan's corrected Task 9 test filter and Task 19 status/spec/debt
updates; no product code, dependency, persisted schema, public interface, CI,
or runtime output changed. Unrelated untracked frontend design/API files
appeared while the final gates were running; they were neither inspected nor
modified and remain preserved in the worktree.
Accepted continuation steps 2–4 and 7–8 are implemented: approval-required
authority remains escalated until an exact ledgered resolution is consumed;
artifact transitions park as one canonical pending record; and approved strong
transitions resume through SWE_SEED to exactly one manifest, declaration, and token.

A code-review pass over `.tmp/cr.md` (27 findings) was applied: 14 fixed in
source/tests (plan_item_id propagation, read/no-assurance authorization, per-episode
approval sequence, required-role enforcement, derived_from canonicalization,
resumed-token proposal-hash check, artifact_id path-traversal guard, attestation rebuild
identity check, terminal retry idempotency, attestation degraded_controls binding, governed
capitalize seeding, + 4 conformance-test fixes), 1 partial (policy identity_bindings + R-SO;
transition SOD rule blocked structurally — SodRule can't scope to transition_kind), 10
verified already-fixed/invalid. Pre-existing M8 CI debt was also cleared to green the gate:
settlement_id is now unique per run, the approve-resolution sequence is derived from the
case-ledger decision count, the capitalize double-grant in the artifact-ip test helpers was
removed, and resume-retry approval grants are idempotent via grant_after_approval_idempotent
(strict double-spend rejection preserved). The workspace is CI-green (fmt + clippy +
289 tests). Remaining open debt: SodRule transition_kind scoping (.agents/OBSERVED_DEBT.md).

## Changed Files

- `crates/sea-forge-domainforge/src/lib.rs` — DomainModelRef v2-full-preimage scheme, real ADAPTER_DESCRIPTOR_SHA256, serde default.
- `crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs` — Conformance test updates for v2-full-preimage.
- `crates/sea-forge-extension/src/cep0008.rs` — Registered godspeed.event.v1-flat schema profile.
- `docs/subsystems/domainforge-boundary.md` — DomainForge boundary subsystem doc updated to 0.16.0 and v2-full-preimage.
- `.agents/reports/domainforge-sxr-sea-rs-cep-interface-investigation.md` — Implementation resolution addendum.
- `.agents/current_status.yml` — Active handoff state.
- `.agents/CURRENT_STATUS.md` — Active status ledger.

- `SWE_SEED/crates/swe-seed-core/src/federation/{identity,producers,idempotency}.rs`,
  `{envelope,consume,mod}.rs`, `tests/convergence_t01_envelope.rs` — plan T01
  canonical-envelope/domain-identity enforcement + 22-test falsifier suite
  (builder evidence: `.agents/evidence/e2e/T01/t01-report.md`; independent
  confirmation pending).
- `.agents/evidence/e2e/T01/t01-report.md` — T01 builder evidence,
  falsifier→proof map, honest PENDING confirmation status.
- `.agents/status/e2e-current-status.yml` — new convergence status surface:
  one frozen-vocabulary verdict + resolvable evidence per requirement for all
  35 frozen requirements (Delta-0: 0 CONFIRMED / 30 PARTIAL / 5 ABSENT).
- `scripts/e2e-prereg-ids.sh`, `scripts/e2e-delta-report.sh`,
  `scripts/e2e-delta-check.sh` — plan T00 verification surface: frozen-ID
  derivation, deterministic Delta-0 report, mechanical matrix validation with
  report drift detection.
- `.agents/evidence/e2e/T00/delta0.md` — committed, drift-checked Delta-0
  report regenerated from the status file.
- `justfile` — e2e convergence gate aliases added (`e2e-check`, `e2e-test`,
  `e2e-lint`, `e2e-delta-check`, `e2e-delta-report`, `e2e-gate <TID>`); no
  existing recipe modified.
- `scripts/check-e2e-preregistration.sh` — new frozen-preregistration hash
  gate (SHA-256 of `.agents/specs/e2e-preregistration.yml` vs
  `source.spec.sha256` in `.agents/plans/e2e-plan.yml`; explicit PASS/FAIL
  verdict, never rewrites the stored hash).
- `justfile` — one added `[group('quality')]` recipe `e2e-prereg-check`
  delegating to that script; no existing recipe touched.
- `.agents/reports/workbench-completion-eval-inputs.md` — concrete independent
  evaluator invocation, real temporary-cell/sidecar setup, and identity
  fixture facts.
- `.agents/reports/workbench-completion-eval-exclusions.json` — exact
  owner-approved story and non-story exclusions for the Linux claim.
- `scripts/check-workbench-completion-eval-inputs.sh` and `justfile` — focused
  machine check and `just workbench-completion-eval-inputs-check` recipe;
  intentionally outside CI until Task 12 owns release aggregation.
- `.agents/CURRENT_STATUS.md` — this Task 0 handoff record and current DAG
  standing.
- `workbench/apps/desktop/src/guards/protectedAction.ts` — shared conservative
  renderer affordance guard over validated identity and source-backed readiness.
- `workbench/apps/desktop/src/pages/{ReadinessPage,ReadinessPage.test.tsx}` —
  case creation now blocks on unresolved identity, names unchanged effect, and
  exposes the identity-inspection next action.
- `workbench/apps/desktop/src/{hooks/useIdentity.ts,shell/{AppShell,GlobalHeader}.tsx}` —
  session-only choice among server-advertised actors, with every consumer
  re-deriving the role from the validated current identity view.
- `workbench/apps/desktop/src/{machines/caseAuthoringMachine.ts,hooks/{useApprovals,useDelegations}.ts}` —
  protected host calls carry only the selected actor id as bounded `actAs`.
- `workbench/apps/desktop/src/{hooks/useThoth.ts,pages/{ThothPage,ApprovalInboxPage,DelegationRoster,CaseCreationWorkbench}.tsx}` —
  every remaining recorded/protected action applies the same refusal and repair
  route; `thoth.ask` also receives only the bounded selected actor id.
- `crates/sea-forge-server/src/{identity.rs,sfwp/{readiness,approvals}.rs}` —
  typed committed source references, explicit unknown/stale freshness, resolved
  approval governance context, and structured identity refusals.
- `workbench/apps/desktop/src-tauri/src/bridge.rs` and
  `src/hooks/bridgeError.ts` — structured governed command errors survive host
  transport instead of collapsing to free text.
- `workbench/packages/contracts/{generated,schema}/` — regenerated TypeScript
  interfaces, AJV validators, and JSON schemas for the additive SFWP records.
- `workbench/apps/desktop/e2e/{readiness.spec.ts,tauriMock.ts}` and affected
  component tests — tests prove unresolved/resolved identities, actionable
  repair routes, the drawer-open capability click, and no protected host call
  on identity refusal.

- `AGENTS.md` — command-first root guide using only `just` commands; Workbench
  detail remains delegated to the existing nested guide.
- `justfile` — adds generic `crate-check` and `crate-test` fast-feedback recipes.
- `workbench/AGENTS.md` and `.agents/skills/building-sea-forge-workbench/SKILL.md`
  — route Workbench development and validation through `just`.
- `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs` and its conformance-test
  diagnostics — point schema regeneration to `just workbench-contracts-generate`.
- `.agents/CURRENT_STATUS.md` — records this documentation-only handoff.

- Merge handoff: this status refresh records the `main` integration and the
  current source reference in
  `.agents/skills/building-sea-forge-workbench/reference/repository-integration.md`.
  `.agents/reports/2026-07-24-sfwp-grounding.md` records the completed
  18-family Workbench method-grounding map; `.agents/OBSERVED_DEBT.md` captures
  its two deferred findings without changing production behavior.

- Task 19 closeout: `.agents/plans/2026-07-22-spec-audit-remediation.md`
  corrects the Task 9 zero-match focused filter and records final acceptance;
  `.agents/specs/spec-agent-orchestration.md` aligns the M12–M16 claim table
  with fresh portable evidence and explicit real-host skips;
  `.agents/OBSERVED_DEBT.md` records unrelated historical Markdown whitespace;
  this status file records the final verification and remaining release gates.

- `.agents/reports/2026-07-22-spec-implementation-audit.md` — executable-code and test-evidence audit of all four `spec-*.md` specifications.

- `spec/CEP-0008-semantic-envelope.md` — authoritative CEP-0008 source copied
  from `/home/sprime01/projects/cep/spec/` to support the SemanticEnvelope
  compatibility-debt refactor.
- `.agents/plans/2026-07-16-adlc-thoth-agent-orchestration.md` — revised after
  adversarial review to add approval gates, source-owned template assets, E8
  vocabulary prerequisites, source-bound sentries, item-level scheduling,
  durable cancellation/approval control, exact endpoint authorization,
  credential authority, SoD provenance, and portable/real integration gates.
- `.agents/OPEN_QUESTIONS.md` — records the unresolved contradiction between
  summarized transcript disposal and later hash recomputation.
- `.agents/specs/spec-adlc-thoth-minimum.md` — makes the M9 `self_model.v1`
  additive-record compatibility contract, source-owned templates, lifecycle
  triggers, provenance, and source-bound sentry requirements normative.
- `.agents/specs/spec-agent-orchestration.md` — makes server-owned episode
  scheduling, exact external/secret authorization, durable control/approval,
  source-bound topology semantics, manager SoD, and the M13 transcript-design
  gate normative.
- `crates/sea-forge-sandbox/src/lib.rs` — SandboxClass, ExecutionSandbox trait,
  select_sandbox, SandboxSpec/Handle/Error/RelPath types.
- `crates/sea-forge-sandbox/src/local.rs` — LocalSandbox backend (existing behavior).
- `crates/sea-forge-sandbox/src/jail.rs` — JailSandbox backend (Linux Landlock).
- `crates/sea-forge-sandbox/tests/conformance_m1.rs` — M1 conformance tests.
- `crates/sea-forge-runtime/src/lib.rs` — uses sandbox backend from grant's class.
- `crates/sea-forge-core/src/types.rs` — added ExecutionStatus::SandboxViolation.
- `crates/sea-forge-settlement/src/lib.rs` — settlement basis `jail_violation`.
- `crates/sea-forge-authority/src/lib.rs` — ActionGrant exposes sandbox_class(),
  relaxed hardcoded local-only check to allow any granted class.
- `crates/sea-forge-cli/src/commands/migrate.rs` — new `sea-forge migrate` command.
- `crates/sea-forge-cli/src/commands/inspect.rs` — finds run dirs in both v0.1 flat
  and v0.2 case-nested layouts.
- `crates/sea-forge-cli/src/commands/mediated.rs` — migrated roots report
  `legacy_digest_only` assurance without requiring a signer.
- `crates/sea-forge-ledger/src/types.rs` — `LedgerStream::verify` checks
  `legacy_import` files against recorded sha256/size.
- `crates/sea-forge-cli/tests/conformance_m0_migrate.rs` — M0 migration gate tests.
- `Cargo.toml` — added 10 new kernel crate members to workspace.
- Task 17: `sea-forge-core/tests/version_skew.rs`; CLI `runs --unsettled`,
  parked-run durability/resume reuse, CEP fixture and produced-envelope check;
  final witnessed envelope checkpoint; §18 evidence links and status/debt updates.
- `crates/sea-forge-artifact-ip/src/lib.rs` — M8 registration, transition,
  projection rebuild, strict caller proposal, pending/terminal/claim-manifest
  records, and exact typed authority/approval resolution.
- `crates/sea-forge-artifact-ip/tests/conformance_m8.rs` — M8 conformance and
  hostile rebuild tests for substituted actions, detached approvals, metadata,
  semantic anchors, derivation identity mismatches, and lifecycle view forgery.
- `crates/sea-forge-domainforge/src/lib.rs` — additive typed class references in
  `DomainModelRef`; existing concept membership remains unchanged.
- `justfile` — added `no-async-kernel` recipe; wired into `ci`.
- `Cargo.lock` — refreshed by the workspace expansion.
- `crates/sea-forge-core/src/lib.rs` — reduced to ids/types/errors + `RECORD_VERSION`.
- `crates/sea-forge-cli/src/main.rs` — added `mod pipeline` and `Migrate` command.
- `crates/sea-forge-cli/src/pipeline.rs` — moved from `sea-forge-core`.
- `crates/sea-forge-cli/src/commands/{run,recall}.rs` — updated imports.
- `crates/sea-forge-cli/src/tests/lifecycle.rs` — updated evidence imports.
- `crates/sea-forge-cli/Cargo.toml` — added kernel crate dependencies.
- New crates: `sea-forge-domain`, `sea-forge-authority`, `sea-forge-planner`,
  `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-trace`, `sea-forge-evidence`,
  `sea-forge-settlement`, `sea-forge-capability`, `sea-forge-extension`,
  `sea-forge-ledger` (foundation).
- `Cargo.toml` / `Cargo.lock` — added 11 new kernel crate members, added
  `ed25519-dalek` to workspace dependencies.
- Task 9.5 additions:
  - `crates/sea-forge-core/src/types.rs` — added `OriginRef`, `OriginRefKind`,
    `OriginRole`, `CriteriaDerivation`, `DerivationMethod`, `JobContract`,
    `DirectionKind`, `SettlementCriteriaRecord`, `PlanItem.settlement_criteria_ref`,
    `CasePlan.job_contract_ref`, `SettlementClaim.criteria_ref`,
    `SettlementEvent.criteria_ref`.
  - `crates/sea-forge-planner/src/criteria.rs` — derivation, hashing, and
    verification of settlement-criteria records.
  - `crates/sea-forge-planner/src/lib.rs` — re-exports criteria helpers.
  - `crates/sea-forge-planner/src/templates.rs` — `PlanTemplate` gains
    `origin_refs` and `job_contract`.
  - `crates/sea-forge-planner/tests/criteria_provenance.rs` — M2c planner
    conformance tests.
  - `crates/sea-forge-planner/tests/conformance_m2.rs` and
    `crates/sea-forge-planner/tests/template_conformance.rs` — updated struct
    literals for new fields.
  - `crates/sea-forge-settlement/src/lib.rs` — emits `legacy_unattributed_criteria`
    basis and records `criteria_ref` on settlement events.
  - `crates/sea-forge-settlement/tests/criteria_provenance.rs` — M2c settlement
    conformance tests.
  - `crates/sea-forge-cli/src/pipeline.rs` — derives/commits criteria records
    from intent before authority for built-in `run`.
  - `crates/sea-forge-cli/src/plan_pipeline.rs` — derives/commits criteria
    records for `run --plan` proposals.
  - `crates/sea-forge-cli/tests/conformance_m2.rs` — added committed criteria
    record verification.
  - `Cargo.toml` — added `tempfile` to workspace dependencies; planner and
    settlement crates gained required test dependencies.

## Completed

- **2026-09-08 DomainForge / SXR / SEA-Forge / CEP Interface Remediation**:
  - Upgraded `DomainModelRef` to `identity_scheme_version: "v2-full-preimage"` with `d_content_hash`, `semantic_closure_hash`, and real `ADAPTER_DESCRIPTOR_SHA256` (`sha256:16fcd1519e00727b2d17c68c3ddbefd633dacf3dcadd51a0ef37a900a119460e`).
  - Added `#[serde(default)]` fallback for `identity_scheme_version` returning `"unknown-pre-versioning"` for legacy persisted records.
  - Registered `godspeed.event.v1-flat` profile (`sha256:a2b2722008e920d0e74b3970b427b0b2e3e5b323c9321ef9a8f4c017d29162eb`) in `sea-forge-extension` CEP-0008 adapter.
  - Updated subsystem docs (`docs/subsystems/domainforge-boundary.md`) and investigation report resolution.

- Task 0 evaluator-input vertical slice: created the source-owned input sheet
  and machine-checkable owner exclusion fixture. The focused validator has
  teeth: it failed before either input existed and failed again after removing
  one declared exclusion; it passes with the restored exact fixture.
- Current DAG standing after the fresh Task 0 checks: N00 is `PASSING` once
  this task gate completes; N01 (sidecar/package inventory) is `PASSING`;
  N05 (per-request identity) and N06 (correlation/recovery) are `PARTIAL` at
  product level despite their focused protocol tests; N02–N04 and N07–N13 are
  not yet broadly proven by the new Ralph gate sequence. This is deliberately
  not a completion claim for any downstream journey.
- Task 1 identity-safe affordances: an unresolved socket identity blocks every
  current protected renderer action with `identity_unresolved`, an unchanged
  effect, and an actionable `/admin` repair route. A selected actor is held
  only in session storage, accepted only when it remains in validated
  `identity.get.available`, and passed to the host as bounded `actAs`; no role
  or actor claim is renderer-authored. The host revalidates identity for every
  protected request.
- Task 1 spendable readiness actions: resolved identity reaches `/cases/new`;
  “Inspect all capabilities” focuses the currently validated readiness
  capability projection in the evidence drawer even when that drawer is open.
- Task 2 source-truth slice: replaced readiness code citations with typed
  `SourceRecordRef` values (`ledger_id`, `entry_id`, record kind/id, digest,
  freshness, rebuild standing). The producer now validates the server's actual
  project root, not its parent; an uninitialized cell and an unproven endpoint
  are `unknown`, while a stale snapshot is `stale`/rebuild-required. The
  renderer opens the committed ledger reference rather than synthetic citation
  evidence. Approval rows now optionally project their verified case-ledger
  request/decision chain with reason, policy, boundary, requester, operation,
  evidence, expiry, side-effect standing, and next lawful steps; missing chain
  remains visibly unresolvable rather than invented.
- Task 2 bridge and purpose completion: the approval view now exposes the
  committed request context (including purpose and resource) instead of a UI
  summary. `sfwp_command` returns a structured refusal across the Tauri boundary
  (`error_class`, `no_side_effect`, `next_lawful_action`), and the renderer
  retains those fields as `BridgeGovernedError`.
- Task 3 automation setup: pinned `tauri-driver 2.0.6` is installed locally
  under ignored `workbench/.tools/`; `just workbench-e2e-real [filter]` now
  preflights Linux WebKit plus an isolated Xvfb display before packaging, seeds
  a unique temporary cell through real operations, allocates a fresh native
  driver-port pair per run (and rejects an exited driver before probing), and
  uses a dependency-free W3C client to drive the compiled Workbench, capture
  DOM/window/SFWP artifacts, then clean up only processes it started. `just workbench-e2e-agent-browser`
  is the default browser-only desktop `e2e` command; it starts a real Vite
  renderer without a Tauri IPC injection and proves the no-bridge state fails
  closed. The existing Playwright suite is retained only as `e2e:mocked` speed
  evidence.

- Reconciled the root agent guide with the current `justfile`; removed stale
  implementation-status claims, circular Copilot precedence, repeated guidance,
  obsolete async-boundary wording, and direct Devbox/Cargo commands. Retained
  the user's design, naming, semantic-density, encapsulation, and layer-boundary
  rules in condensed form.
- Added `just crate-check <crate>` and `just crate-test <crate> [filter]` so
  focused Rust feedback stays behind the repository command surface.
- Added `just workbench-tauri-dev`, `just workbench-host-build`, and
  `just workbench-contracts-generate`, plus `just workbench-skill-check`, to
  close the nested Workbench guide's direct-command gaps.

- Audited all four `spec-*.md` documents against source and executable tests; the report identifies conformance blockers in every specification and does not use documentation as evidence.

- Copied the authoritative CEP-0008 Semantic Envelope specification into the
  repository; its text matches the source, apart from adding the conventional
  trailing newline.
- Repaired the `full-spec` pre-push license gate: workspace crates now use the
  valid custom SPDX reference `LicenseRef-SEA-Forge`, cargo-deny explicitly
  allows that reference, and README license links resolve to the checked-in
  `LICENSE` and `COMMERCIAL-LICENSE.md` files.
- Merged `ci-cd` into `main` and pushed to `origin/main`.
- Task 1 — M0a mechanical crate graduation: moved 14 slice modules into 10 new
  kernel crates plus `pipeline.rs` into `sea-forge-cli`, fixed cross-crate imports,
  added required dependencies, added `no-async-kernel` check, and verified the
  gate: `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace`,
  `just proof`, `just no-async-kernel` all pass.
- Noted and fixed one test-path issue: `runtime::tests::timeout_child_helper` became
  `tests::timeout_child_helper` after the move; this is a path reference update,
  not a logic change.
- Task 2 — M0b sea-forge-ledger: complete. All §12 M0 ledger conformance fixtures
  pass: 1000-record multi-stream append with ULID/ordinal/chain/MMR verification;
  one-byte alteration / truncate / reorder / duplicate detection with typed
  `ledger_integrity_error`; Ed25519 signed checkpoints with chain verification;
  MMR inclusion proofs; global checkpoints committing all stream roots;
  independent witness receipts detecting fork substitution (and rejecting
  self-witnessing); secret sentinel redaction rejecting plaintext private keys
  and API keys while accepting approved ciphertext commitments; key rotation
  with old checkpoints verifying under snapshotted key refs; crash recovery
  quarantining incomplete tails. CLI `ledger verify|prove` subcommands added.
- Task 3 — M0c DomainForge semantic adapter: `crates/sea-forge-domainforge`
  created with `domainforge-core = "=0.13.0"`, default features off. Implements
  `load_validate(SeaSourceSet) -> DomainModel` using DomainForge's parser → graph →
  validation pipeline; `DomainModelRef` with `semantic_model_sha256` over canonical
  4-tuple; authority normalization table (Reject/Deny→deny, Escalate→escalate,
  Allow→allow, NotApplicable→deny-if-required); real `.sea` fixture; conformance
  tests covering valid parse → stable ref, invalid syntax → domain_model_error,
  source-hash drift rejection, no-side-effects-on-invalid-input, and normalization.
  pass: 1000-record multi-stream append with ULID/ordinal/chain/MMR verification;
  one-byte alteration / truncate / reorder / duplicate detection with typed
  `ledger_integrity_error`; Ed25519 signed checkpoints with chain verification;
  MMR inclusion proofs; global checkpoints committing all stream roots;
  independent witness receipts detecting fork substitution (and rejecting
  self-witnessing); secret sentinel redaction rejecting plaintext private keys
  and API keys while accepting approved ciphertext commitments; key rotation
  with old checkpoints verifying under snapshotted key refs; crash recovery
  quarantining incomplete tails. CLI `ledger verify|prove` subcommands added.

## Verification

- **2026-09-08 Interface Remediation Verification**:
  - `just crate-check sea-forge-domainforge` — PASS
  - `just crate-test sea-forge-domainforge` — 28/28 tests passed (including `domain_model_ref_v2_full_preimage_matches_specification` and legacy deserialization fallback)
  - `just crate-check sea-forge-extension` — PASS
  - `just crate-test sea-forge-extension cep0008` — 42/42 tests passed (including registered `godspeed.event.v1-flat` profile schema)
  - `cargo test --test convergence_t05` — 10/10 tests passed
  - `just context-check` — PASS

- Task 0 focused evidence: `just workbench-completion-eval-inputs-check`
  passes; it failed with `missing ...eval-inputs.md` before inputs were added
  and with an invalid fixture after removing exclusion `16.3`. `just
  workbench-package-inventory` passes for the current `.deb` (sidecar present,
  no JavaScript runtime, no source maps). `just crate-test sea-forge-server
  identity`, `just workbench-contracts-gate`, and `just workbench-tauri-test`
  were rerun without a command failure; the latter covers U-06 sidecar
  supervision and bridge recovery. After this status update, `just
  context-check`, `just check-fast` (format + workspace typecheck), `just
  workbench-contracts-gate`, `just workbench-completion-eval-inputs-check`,
  and `git diff --check` all pass.
- Task 1 TDD evidence: `ReadinessPage.test.tsx` gained the unresolved-identity
  regression, which failed against the former readiness-only action check.
  After the shared guard landed, `just workbench-check` completed its contracts,
  host, renderer typecheck, build, and test sequence without a reported
  failure.
- Actor-session evidence: `GlobalHeader.test.tsx` proves explicit actor choice;
  `ApprovalInboxPage.test.tsx` proves `operator_b` is forwarded as `actAs` to
  the closed host bridge. `just workbench-check` passed after the selection and
  forwarding slice.
- Task 1 final evidence: focused component tests (34 assertions) pass for
  case commit, approvals, cancellation, Thoth, and repair routes; the complete
  desktop suite passes through the final Workbench gate; Playwright `bun run e2e -- --grep "identity|Inspect
  all capabilities"` passes 3 scenarios. `just crate-test sea-forge-server
  identity`, `just workbench-tauri-test`, `just check`, `just test`, `just
  proof`, and final `just workbench-check` all completed without a reported
  failure. Cargo emitted the pre-existing `license`/`license-file` manifest
  warnings. These browser scenarios use the declared Tauri mock and are only
  Task 1 interaction evidence, never a real-stack claim.
- Task 2 focused evidence: `just workbench-contracts-generate` regenerated 57
  schemas; `just crate-test sea-forge-server readiness` covers uninitialized,
  committed, and stale snapshots; `just crate-test sea-forge-server approval`
  passed 3 unit, 6 approval conformance, 2 identity, and relevant escalation
  tests. Focused desktop tests passed: `ReadinessPage.test.tsx` plus
  `protectedAction.test.ts` (9 assertions), and `ApprovalInboxPage.test.tsx`
  (11 assertions). Desktop typecheck passed with two pre-existing lint warnings
  in `router.tsx` and `useIdentity.ts`; `just fmt-check` and `git diff --check`
  passed after this slice.
- Task 2 approval-chain conformance: the real escalated case episode now proves
  `approval.list` resolves its row to committed `approval_request` and
  `authority_decision` entries with SHA-256 digest and
  `not_executed_pending_approval`; `just crate-test sea-forge-server
  an_escalated_episode_opens_an_approval_and_runs_nothing` passes.
- Task 2 final focused evidence: `just workbench-contracts-generate`, `just
  workbench-tauri-test`, the `identity`, `readiness`, `approval`, denied-episode,
  and `stale_precondition_on_approve_is_rejected_with_no_side_effect` server
  slices, and the focused ApprovalInbox/bridge-error renderer tests all passed.
  `bun run --cwd workbench/apps/desktop check` passed. The prescribed
  `just crate-test sea-forge-server conformance` filter selects no tests (Cargo
  filters function names, which do not contain that word), so the named focused
  conformance tests above are the actual coverage. `just workbench-contracts-gate`
  correctly fails while regenerated contracts remain uncommitted; no index
  manipulation was used to hide that drift.
- Task 3 automation evidence: `agent-browser 0.33.2` with Chrome 151 is
  installed; `just workbench-e2e-agent-browser` passes the no-bridge assertion
  and a `#main-content` accessibility scan. Owner-installed `webkit2gtk-driver`
  and `xvfb`, plus the pinned ignored `tauri-driver 2.0.6`, support the real
  path. A strict Tauri CSP initially left the packaged WebKit renderer blank:
  Astryx's `defineTheme` extension and the generated AJV validators both
  required runtime code generation. The application now consumes the prebuilt
  neutral projection with static scoped SEA Forge tokens, and the contract
  generator emits Ajv standalone validators with unchanged typed `validate`
  exports (no browser-time `ajv.compile()` or `Function(...)`). The focused
  prebuilt-theme and mockup-fidelity tests pass. `just workbench-package`, the
  direct real WebKit smoke, and the complete `just workbench-e2e-real` recipe
  pass: it packages the app, seeds real records, launches the real
  server/socket and compiled app under isolated Xvfb, confirms the mounted
  readiness document, then exercises SFWP hello, identity, reconnect, and
  request recovery. The runner only observes the loaded application; temporary
  diagnostic module reruns were removed. `bash -n` for both runners, Python
  compilation, `just workbench-e2e-agent-browser`, `just context-check`, and
  `git diff --check` pass.

- `git diff --check` for the command-surface paths: passed.
- `just --show` for `crate-check`, `crate-test`, `workbench-tauri-dev`,
  `workbench-host-build`, `workbench-contracts-generate`, and
  `workbench-skill-check`: parsed as expected.
- `just crate-check sea-forge-core`: passed (Cargo emitted pre-existing
  `license`/`license-file` manifest warnings).
- `just crate-test sea-forge-core ids`: passed (1 selected test passed).
- `just check-fast`: passed (same pre-existing manifest warnings).
- `just workbench-skill-check`: passed.
- Contract regeneration and package/desktop-launch recipes were not run because
  regeneration mutates a user-modified generated zone and the latter recipes
  are outside this command-surface change.
- No build or product tests run: only Markdown agent instructions changed.

- 2026-07-22 audit: `devbox run -- just check` passed; focused conformance suites and `just proof` passed as recorded in `.agents/reports/2026-07-22-spec-implementation-audit.md`.
- 2026-07-22 audit: `devbox run -- just test` failed twice with a suite-context `SIGSEGV` before `sea-forge-cli` main-unit test output. Its isolated binary test passed (5 tests); the fault remains unresolved.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test --workspace --all-features --locked`: passed on the current Task 9 worktree.
- `just proof`: P1–P4b passed.
- `just no-async-kernel`: passed.
- `cargo build --workspace --all-targets --locked`: passed.
- `cargo test -p sea-forge-artifact-ip`: 22 passed, 0 failed.
- `cargo test -p sea-forge-authority`: 36 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m8_artifact --locked`: 3 passed,
  0 failed. The evaluator/token hostile test was observed red before implementation
  because no evaluator score was ledgered, then green after the M7 path was wired.
- `cargo test -p sea-forge-sandbox --test conformance_m7 --locked`: 10 passed,
  0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check` plus untracked M8 file checks: passed.
- `cargo test -p sea-forge-cli conformance_m0_migrate --locked`: passed.
- Task 6 migration gate: lossless genesis import, idempotence guard, ledger
  verify corruption detection, and `legacy_digest_only` inspect assurance pass.
- Task 7 — M1 jail sandbox backend: `SandboxClass` enum (`local|jail|microvm`),
  `ExecutionSandbox` trait (§11.2), `LocalSandbox` (existing behavior), and
  `JailSandbox` (Linux Landlock via the `landlock` crate). Landlock ruleset
  allows read-write to workspace+artifacts, read-only to `/`, denies all other
  writes. Thread-based restriction (no `unsafe`/`pre_exec`) keeps the main
  thread unrestricted. Runtime selects backend from `grant.sandbox_class()`;
  unavailable class returns `unsupported_sandbox_class_error`. Settlement adds
  `jail_violation` basis when `ExecutionStatus::SandboxViolation` is detected.
  Conformance tests: jail blocks write outside workspace, schema_error for
  untrusted argv0 on local, class identity, unavailable-platform refusal.
- `cargo test -p sea-forge-planner --test conformance_m2 --test template_conformance`: 13 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m2`: 3 passed, 0 failed.
- `devbox run -- just check`: passed on the current worktree; cargo-deny emitted only non-fatal duplicate/unmatched-allowance warnings.
- Task 5 resolver slice: 12 `sea-forge-authority` tests passed, including typed
  deny/escalate/boundary/degraded/allow resolution and order independence.
- Task 5 execution-boundary slice: authority no longer depends on sandbox;
  move-only, non-serializable grants bind the exact action/run/item/workspace;
  public sandbox materialization and runtime execution consume a matching grant.
- Task 5 canonical-decision slice: `LedgerStream::commit_typed` returns an
  opaque committed-record reference; the CLI commits every authority decision
  before writing `authority.json` or issuing its exact-action grant.
- Task 5 review fixes: boundary dimensions now intersect and incompatible
  boundaries deny; grants require a one-use decision issued by the same engine
  plus a non-deserializable committed ref; authority evidence commits before
  the decision that cites it.
- Task 5 candidate slice: authority decisions now persist candidate verdicts,
  winning source, resolution reason, sandbox grant, boundaries, and controls;
  v0.2 engine declarations reject fail-open modes and unavailable required
  engines deny through the typed resolver.
- Task 5 view/extension slice: authority compatibility output materializes only
  after its committed decision and records current/failed freshness; failed
  materialization preserves verifiable ledger truth. Extension registry saves
  are ledger-first, and imported adoption consumes an exact-action grant plus a
  committed authority reference.
- Task 5 ingress slice: run commits intent, plan, identity, policy, request,
  evidence, and decision before effects; validate, recall, and inspect now pass
  through the same authority engine and ledger-backed exact-action check before
  reading protected data. Minimum v0.1 read behavior remains compatible; v0.2
  policies require explicit read rules.
- Task 5 hardening: v0.2 identity maps fail unresolved identities closed and
  require sponsors for automated agents; complete protected operation names and
  fail-closed engine declarations parse in one schema; DomainForge candidates
  compose through the typed resolver; grants bind timeout, environment keys,
  workspace, artifacts, sandbox class, boundaries, controls, and expiry.
- Required-integrity policies now produce signed stream/global checkpoints and
  independently signed witness receipts before any command-start event. Missing
  or duplicate witnesses fail closed before workspace effects. Authority decision,
  audit, and opaque-constraint mirrors rebuild from ledger records; inspect and
  recall surface ledger assurance.
- Task 5 / M0-G3 complete: v0.2 policy snapshots require all authority surfaces,
  RBAC permissions, SoD rules, source hashes, and canonical bundle hashes;
  configured DomainForge evaluation runs through real CLI ingresses; opaque
  constraints preempt matching work; per-record assurance proves inclusion in
  the exact signed/witnessed checkpoint; authority audit records preserve the
  resolved disposition, canonical resource subject, and case linkage.
- Independent final review: approved with no findings.
- Task 6 — M0f `sea-forge migrate`: lossless v0.1 → v0.2 case layout migration.
  `sea-forge migrate` enumerates legacy source files (excluding views/append-only
  `authority/` and `capabilities.jsonl`), emits `legacy_import` genesis ledger
  entries with byte sha256/path/size/legacy record version, signs an initial
  global checkpoint, and relocates run directories under
  `.sea-forge/cases/<case_id>/runs/<run_id>/` and case files to
  `.sea-forge/cases/<case_id>/case.json` without rewriting record bytes. Migration
  is idempotence-guarded by `.sea-forge/migration.json`. `LedgerStream::verify`
  verifies each `legacy_import` file against its recorded hash and size, so a
  corrupted legacy file fails `ledger verify`. `inspect` finds runs in both v0.1
  flat and v0.2 nested layouts and reports `legacy_digest_only` for migrated
  records. Conformance tests verify byte hashes committed, IDs resolvable, ledger
  verify green, re-migration refused, corruption detected, and inspect assurance
  labeling.
- Task 7 — M1 jail sandbox backend: `SandboxClass` enum (`local|jail|microvm`),
  `ExecutionSandbox` trait (§11.2), `LocalSandbox` (existing behavior), and
  `JailSandbox` (Linux Landlock via the `landlock` crate). Landlock ruleset
  allows read-write to workspace+artifacts, read-only to `/`, denies all other
  writes. Thread-based restriction (no `unsafe`/`pre_exec`) keeps the main
  thread unrestricted. Runtime selects backend from `grant.sandbox_class()`;
  unavailable class returns `unsupported_sandbox_class_error`. Settlement adds
  `jail_violation` basis when `ExecutionStatus::SandboxViolation` is detected.
  Conformance tests: jail blocks write outside workspace, schema_error for
  untrusted argv0 on local, class identity, unavailable-platform refusal.
- Task 8 — M2a CMMN-subset case engine: persisted `PlanItem`/`Sentry`/`Case`/
  `TraceKind`/`SettlementCriteria` types; sentry evaluator as a pure function of
  trace events and workspace file set; static `plan_cycle_error` satisfiability
  check on the entry-criteria dependency graph; `validate_proposal` normalization
  (safe IDs, relative paths, no empty plans); deterministic case reducer with
  enable/activate/complete/park/terminate actions; required-item failure
  terminates the case with `terminated rejected`; `parked` is a normal state, not
  a failure; `sea-forge run --plan` plan-proposal driver; `sea-forge case` and
  `sea-forge task` subcommands (reopen, add-task, complete). Conformance tests:
  A/B(rep×2)/C/M scenario replay reproduces activation order; empty entry
  criteria activate immediately; unsatisfiable sentries rejected; required-item
  failure terminates the case; reducer retries then terminates required items;
  parked case is not failure; proposal validation rejects bad paths and cycles.
- Task 9 — M2b plan templates: `PlanTemplate`/`ParameterDef` types with typed
  parameters (`string`, `int`, `bool`, `path`) stored at
  `.sea-forge/templates/<name>@<version>.yaml`; load-time forbidden-substitution
  checks (`kind`, `plan_item_id`, `name`, `sandbox_class`, `argv[0]`);
  deterministic instantiation yielding byte-identical `CasePlan` for same template
  - params; `template_ref` provenance recorded in `CasePlan` and semantic envelope;
  `load_pinned` with per-version SHA-256 pin that rejects content changes without
  a version bump; built-in `sea_model_demo@0.1.0` template. Conformance tests:
  instantiation byte-identity; forbidden `argv[0]` substitution rejected at load;
  missing required parameter is input error; path parameter rejects
  parent/absolute/prefix escape; pin rejects same-version byte change.

- Task 9.5 — M2c settlement-criteria origin and provenance: `OriginRef`,
  `SettlementCriteriaRecord`, `JobContract`, and `CriteriaDerivation` types in
  `sea-forge-core`; `PlanItem.settlement_criteria_ref` and `CasePlan.job_contract_ref`;
  `sea-forge-planner/src/criteria.rs` with `derive_from_intent`,
  `derive_from_template`, deterministic `criteria_sha256`/`criteria_record_hash`, and
  `verify_item_criteria`/`verify_plan_criteria`; `settlement_criteria` records
  committed to the ledger before authority in both `run_intent` and `run --plan`
  pipelines; embedded criteria snapshot/hash agreement enforced; legacy claims
  without `criteria_ref` marked `legacy_unattributed_criteria` in settlement basis;
  no new crate, database, or independent criteria store added; JobContract not
  synthesized for current paths because existing Intent and PlanTemplate substrate
  already satisfies §7.1a. Conformance tests: every new PlanItem resolves to one
  committed criteria record; origin refs resolve and hash-verify; missing ref and
  hash-mismatch fail with `criteria_provenance_error`; same template+params yields
  identical criteria content, origin refs, and criteria_sha256; changing criteria
  changes the hash; legacy items are skipped by verification; built-in demo does
  not create a JobContract.

- Task 10 — M3 server, approvals, operator loop: `ApprovalRequest`/`ApprovalStatus` types; `approvals.jsonl` append-only store with latest-line-wins resolution; escalate→ApprovalRequest→exit 5 in plan_pipeline; `sea-forge approve|reject` CLI with TTL expiry check and no-re-resolution; `sea-forge resume` re-enters the case loop after approval resolution; `sea-forge-server` crate with Tokio runtime, Unix socket NDJSON protocol (submit/status/approve/reject), `spawn_blocking` dispatch via subprocess, `max_concurrent_runs` semaphore, dynamic config reload (last-known-good on invalid), `notify_command` execution (failure logged and ignored). Conformance tests: escalate→exit 5→approve→resume→completed; double-approve refused; reject→resume→terminated (exit 4); expired approval refuses resolution.

- Task 11 — M4a settlement declarations + capability promotion: `SettlementDeclarationRequest`/`SettlementDeclaration`/`Declarer`/`DeclarationIndependence`/`DeclarationReliability`/`SettlementStrength`/`DeclarationStatus` types in `sea-forge-core`; `CapabilityPromotionPolicy`/`CapabilityRecord`/`CapabilityStatus`/`CapabilityQualifying`/`CapabilityVariation`/`CapabilityRecovery`/`CapabilityOrchestration` types in `sea-forge-core`; `crates/sea-forge-settlement/src/declaration.rs` with `SettlementAuthority` trait, `LocalSettlementAuthority` adapter (strength=local always, qualifies_for_capability=false), `SweSeedSettlementAuthority` adapter with pluggable `SweSeedTransport` trait (test-double in tests), `check_integrity` (post-hoc criteria, self-declaration, missing criteria_ref/origin_refs), `compute_declaration_hash`, `append_declaration`/`load_declarations` JSONL store; `crates/sea-forge-capability/src/promotion.rs` with fixed-point decimal arithmetic (6 places, i64 millionths, clamp, zero-denominator rule), `default_v02_policy`, `compute_policy_hash`, `save_policy`/`load_policy` snapshot store, `declaration_qualifies` predicate (status=accepted, strength=strong, qualifies_for_capability, independent, weight>=min, criteria_ref non-empty, evidence-backed tags), `build_capability_record` pure projection (counts from envelopes, qualifying from declarations+policy, variation coverage dedup, recovery tracking, orchestration burden reduction, confidence = reliability_ratio *coverage_ratio* recovery_ratio * burden_factor, status determination attempted<demonstrated<proven, contraction reasons), `rebuild_capability` (byte-identical modulo rebuilt_at), `require_proven` (denies with citation unless status>=proven). 13 conformance tests: raw counts match 5 mixed runs; local declaration zero qualifying weight; post-hoc criteria integrity failure; self-declaration integrity failure; gameable feedback weight below threshold; low attribution weight below threshold; three qualifying declarations promotion to proven; repeated variation no coverage increase; regression contraction; rebuild byte-identity; require_proven denial with citation; require_proven allows when proven; policy change contraction.

- Task 12 — M4b governed semantic memory: `MemoryKind`/`MemoryItemProvenance`/`MemoryItem` types in `sea-forge-core`; `EvidenceKind::Recall` variant added; `memory_scope: Option<String>` added to `PolicyRule` in `sea-forge-authority`; `crates/sea-forge-capability/src/memory.rs` with `compute_dedup_key` (sha256 of kind + normalized statement + entity_id), `extract_from_envelope` (deterministic: one `outcome` item per envelope, statement capped at 1000 chars, provenance from envelope run_id + evidence_refs), `append_memory_items` (append-only JSONL), `load_memory_items` (dedup-at-read: merge by dedup_key, union run_ids/evidence_refs, earliest created_at, latest last_confirmed_at), `recall_memory` (linear scan, scope filter, kind filter, limit capped at 50), `scope_allows` (own/entity:X/any/default-deny), `rebuild_index`/`query_index`/`recall_with_fallback` (pure JSON projection, identical results to fallback scan); pipeline extraction wired after envelope append in `pipeline.rs` (never fails run, errors logged); CLI `sea-forge memory rebuild` command + `sea-forge recall --kind` flag (memory-item mode, contract preserved without --kind). 8 conformance tests: two-entity dedup + provenance, own-scope isolation, cross-entity denial, index-delete equivalence, extraction safety, dedup-key determinism, limit cap, kind filter. ponytail: JSON index instead of rusqlite/SQLite — achieves same outcome (rebuildable projection, fallback-equivalent) without C compilation dependency; switch to rusqlite if linear scan becomes measured bottleneck.

- Task 13 — M5 spec-to-code pipeline + DomainForge projections: `PipelineRoute`/`ProofClassification`/`StageKind`/`StageStatus`/`StageFile`/`SpecPipelineStage`/`SpecPipelineRun`/`ProjectionRecord`/`ProjectionValidation` types in `sea-forge-core`; `run_spec_pipeline`/`run_projection` added to authority operation_kind list; `ProjectionKind` gained Ord/PartialOrd; `project()` function added to `sea-forge-domainforge` (CALM via `calm::export`, RDF via `KnowledgeGraph::from_graph`→`to_turtle`/`to_rdf_xml`); CALM export's non-deterministic `sea:timestamp` stripped for byte-identical regeneration (§10.7); new crate `sea-forge-spec-pipeline` with `compute_stage_hash`/`compute_chain_hash` (linked SHA-256 chain), `validate_stage_order` (canonical stage ordering), `compute_proof_classification` (authority-only → generated-contract → focused-slice ceiling), `quarantine_stage` (sets status + quarantine_ref + basis), `check_generated_zone_edit`/`is_generated_zone` (src/gen, .ast.json, .ir.json, .manifest.json, semantic fixtures), `project_model` (in-memory CALM+RDF via adapter), `compute_rebuild_hash`/`build_projection_record` (ProjectionRecord with rebuild hash), `process_pipeline` (validate + quarantine + classify), `verify_byte_identity` (regeneration determinism), `compute_input_hash`/`hash_content`. 10 conformance tests + 5 domainforge projection tests. ponytail: no `sea-forge project` CLI command yet (gate is crate-level tests only); no `.sea` synthesis adapter (DomainForge validation test covers the contract); declarative Evaluator deferred to M7 (per Appendix A, command form is Task 15).

- Task 14 — M6 SeaCell federation prep: `cell_id` field added (Option<String>, absent = legacy valid) to `TraceEvent`, `EvidenceRecord`, `SemanticEnvelope` in `sea-forge-core`; `BundleFile`/`BundleManifest` types in `sea-forge-core`; `ids::cell_id()` (`cell_<8hex>`)/`ids::bundle_id()` helpers. New crate `sea-forge-cell`: `cell::ensure`/`read` (load-or-create `.sea-forge/cell.json`, idempotent, schema `cell.v1`); `bundle::export` (tar with `manifest.json`, sha256 per file, deterministic header mode, excludes `workspace/` scratch, includes `artifacts/`); `bundle::import` (atomic-reject per §14.8: stage to `.staging-<bundle_id>`, recompute+verify all sha256/size, reject whole bundle on any mismatch — missing/extra/tampered — clean staging on err, atomic rename into `imported/<exporter_cell_id>/`, never touches `capabilities.jsonl`); `bundle::read_manifest`; `template::adopt` (copy from `imported/<cell_id>/templates/` into active `templates/`, leaves imported provenance trail). `EventSink` trait + `SinkEvent` (8 contract fields: `event_id, trace_id, correlation_id, causation_id, idempotency_key, source_agent, occurred_at, schema_version, subject, payload`) + `JsonlEventSink` (append-only JSONL) + `trace_to_sink` (maps `TraceEvent` → `SinkEvent`) + `subject_for` (maps 24 `TraceKind` variants to `sea.{domain}.{action}.{qualifier}` 4-segment subject per ecosystem map) in `sea-forge-trace`. Authority operation_kind allow-list extended: `import_bundle`, `export_bundle`, `adopt_template`. `pipeline.rs` stamps `cell_id` on envelope. CLI: `sea-forge export`/`import`/`adopt` commands with authority mediation. 11 conformance tests: export/import hash-verify, capability-count-unchanged, tampered-bundle atomic reject (teeth: one flipped byte → whole import fails, no leftover dir), extra-entry rejection, missing-manifest rejection, unknown-schema rejection, re-import replaces prior, read-manifest inspection, cell-id stability, absent-cell legacy valid, imported-template-not-instantiable-pre-adopt. New dep: `tar = "0.4"` (spec mandates tar format; integrity-boundary correctness; MIT/Apache-2.0). ponytail: not wiring EventSink into live pipeline (M6 = seam + roundtrip test only); bundles exclude `workspace/` scratch (only evidence files + artifacts); adopt is copy-not-move (preserves imported provenance trail); no environment bundles (M7/Task 15).

- Task 15 — M7 environment contracts + command evaluators: `PlanItem.environment` plus optional `SettlementCriteria.evaluator`, `records`, `per_record_evaluator`, `min_pass_ratio`; `BatchEvaluationResult`/`BatchFailure` and evaluator scores carried on `SettlementClaim`. `sea-forge-sandbox/environment.rs`: YAML `EnvironmentSpec` (`base`, `provides.commands`, command evaluators), first-use SHA-256 pinning at `environments/.pins/`, missing/tampered specs fail `environment_unavailable`, base materialization, score parsing, and built-in `demo_env@0.1.0`. Authority gains `PolicyRule.environment`, an environment context on `AuthorityEvaluation`, and `command_allowed` intersection enforcement: command basename must be provided by the item environment and satisfy any `argv0` rule. Pipeline loads/materializes the environment before authority, executes every authorized command sequentially, executes declared evaluator commands under their own authority decision/grant, and runs per-record evaluator command pairs through authority with each record materialized as `record.json`; settlement records evaluator scores and writes failing batch records to `quarantine/<plan_item_id>.jsonl`. CLI: `sea-forge env list|show`. 10 `environment_*` conformance tests: evaluator basis, score parsing, 10-record ratio 0.8 with 2 failures accepted + 2 quarantined, 3 failures rejected + 3 quarantined (teeth), three-axis independence, hash pinning, missing/tampered fail-before-materialization, demo fixture, base materialization, YAML round-trip. ponytail: declarative predicate evaluators deferred (M7 proves command form); Endpoint/NetworkFlow remain deny-by-default and credentials remain authority contracts, preserving a future DomainForge Cell projection seam.
- Task 16 M8 authoritative rebuild hardening: transition decisions deserialize as
  `AuthorityDecision` and must allow the reconstructed canonical action with all
  source licenses plus exact run/case/`transition` context and payload hash.
  Capital approvals deserialize as `ApprovalRequest` and bind approved,
  pre-expiry resolution to the token's run, case, criteria, decision, plan item,
  requester, and approver. Hostile substituted-action and unrelated-approval
  rebuilds fail closed.
- Task 16 transition cases now derive their intent/criteria provenance and source
  evidence from the resolved `TransitionInput`, copy the profile's single M7
  evaluator and approval requirement into settlement criteria, derive the item
  environment from `<environment-ref>.<evaluator-name>`, and execute the evaluator
  through the existing environment/authority/runtime path. Settlement events
  ledger evaluator scores; the artifact resolver enforces the profile threshold
  before token append. The detached `transition.ok` marker was removed. Profiles
  requiring approval or strong external declarations park before case creation;
  no approval or declaration is synthesized.
- Task 16 review blockers fixed: `quality` is never qualifying capital value even
  when listed by a gate profile; value-source records resolve external-case
  ledgered run evidence plus an accepted linked settlement before append, with
  aggregate acceptance derived from those records; capitalization approvals bind
  both criteria hashes to the resolved `SettlementCriteriaRecord`. Hostile tests
  cover fabricated value sources and missing/wrong approval hashes, and the
  quality-only path proves accepted quality evidence creates no token/projection.
- Task 16 capital hardening now deserializes every strong reference as a complete
  `SettlementDeclaration`, verifies its embedded hash and exact settlement,
  criteria, run, case, and plan-item links, and reuses the default v0.2 capability
  qualification policy. Strong declarations also require ledger-resolved source
  evidence plus explicit standing, independence, reliability, and adapter
  attestation data. Value evidence requires `canonical_value_evidence_kind` on
  each underlying `EvidenceRecord`, with exact source/wrapper agreement.
- Continuation dependency step 2 removes the caller `approval_ref`/`approver_id`
  authority shortcut. `PolicyAuthorityEngine::grant_after_approval` now requires
  exact committed authority-decision, approval-resolution, and criteria records;
  validates all decision/case/run/item/criteria hashes, action/context, expiry,
  and separation of duties; and yields one exact one-use grant. CLI approval now
  resolves and revalidates ledger truth before committing an idempotent resolution,
  then updates `approvals.jsonl` as a compatibility view.
- Continuation steps 3–4 add strict `TransitionProposal` ingress, canonical
  `PendingArtifactTransition`, typed terminal, and immutable claim-manifest
  records. The artifact-only plan extension commits the exact transition
  escalation and standard approval, then `commit_typed_once` commits pending in
  `case-<case_id>` before exposing `awaiting_approval`/exit 5. Strong-only gates
  park identically; no approval resolution or declaration is fabricated.
- Continuation steps 7–8 add validated `settlement_authorities[]` descriptors and
  a real tokenized-argv SWE_SEED command transport with JSON stdin/stdout,
  bounded output, timeout, minimal environment, and fail-closed behavior. Resume
  detects artifact pending/terminal records before generic M3 handling, trusts
  only ledgered approval resolutions, terminalizes reject/expiry, persists and
  reuses evaluator execution/settlement, commits one immutable manifest and
  qualifying strong declaration, consumes the exact approval grant, and commits
  one token plus terminal. Outage remains `awaiting_approval` with no declaration
  or token; retry/replay returns the existing matching records.
- Final Task 16 domain blockers are closed: required metadata uses an explicit
  `product_contract.*` selector vocabulary over ledgered result evidence;
  semantic anchors are typed as concept/class and resolve against a ledgered
  `DomainModelRef`; derive validates exact source/identity pairs and new result
  identities; append-only lifecycle records carry exact authority bindings and
  rebuild independently from maturity, so retired capital remains capital.
- Final Task 16 review blocker fixed: artifact resume now converts an expired,
  unresolved exact pending approval into one ledgered `expired` resolution before
  terminalizing the matching transition (exit 4, no token). Hostile replay
  coverage confirms terminal/no-token behavior and ledger idempotence.
- Task 17 Definition-of-Done sweep: additive v0.2 fields are deserialized by
  exact v0.1 reader snapshots from actual current record serialization; `just ci`
  retains the no-Tokio kernel check; produced semantic envelopes are checked
  against the copied CEP-0008 fixture with the divergence recorded as debt;
  `runs --unsettled` reports run IDs lacking settlement; approval-parked runs
  persist plan/authority snapshots and resume into the same run; required-integrity
  runs append a final signed/witnessed checkpoint covering the semantic envelope.
- Task 17 crash-recovery drill (2026-07-16): submitted real case
  `case_20260716T175502Z_dcd297` through `sea-forge-server`, reaching approval hold
  for `run_20260716T175502Z_5e7bcc`; terminated the server; verified the run kept
  `plan.json` and `authority.json`; restarted the server and verified status was
  absent from volatile memory (no auto-resume); `sea-forge runs --unsettled`
  discovered the persisted run; explicit security-officer approval plus
  `sea-forge resume` completed the same run with accepted settlement.
- Task 17 substrate reconciliation: inspected and reused existing serde record
  types, case files, ledger records, approval/resume path, integrity checkpoint
  writer, CI recipe, and milestone conformance suites. Extended only the CLI
  read path and parked/final checkpoint persistence; added compatibility and CEP
  checks plus the copied schema. Deliberately added no database, recovery daemon,
  second run store, JSON-schema dependency, or alternate envelope format.
- Task 17 final review hardening: generic resume now derives approval status,
  run identity, and criteria binding from unique case-ledger request/resolution
  records rather than `approvals.jsonl`; `runs --unsettled` requires a parseable
  settlement matching the run; value-source settlement lookup matches both
  `set_01` and run ID; compatibility tests use exact v0.1 minimum-record and
  authority shapes against actual current serialization.
- Generic resume verifies the case ledger hash chain before consuming approval
  truth; a hostile edited approval-resolution payload now fails closed.

## Remaining

- 2026-09-08 Interface Remediation: None. All recommendations from `domainforge-sxr-sea-rs-cep-interface-investigation.md` are implemented, verified, and settled.

- Continue the plan's next bounded Workbench task. Task 3's real-stack native
  automation gate is satisfied; the installed Playwright suite remains only
  mocked speed evidence and is not used for the integrated claim.

- No remaining work for the Just command-surface refactor.

- Run the ignored real ACP release gate when
  `SEA_FORGE_REAL_ACP_ARGV` (and any required `SEA_FORGE_REAL_ACP_ENV`) is
  supplied by an operator.
- Run the ignored real SWE_SEED release gate when the real ACP argv/env plus
  `SEA_FORGE_REAL_SWE_SEED_REPO` and
  `SEA_FORGE_REAL_SWE_SEED_COMMIT` are supplied.
- Run the Seatbelt network conformance case on macOS. None of these skipped
  release/platform checks is claimed by the portable Task 19 closeout.
- `.agents/plans/TODO.md` defines the stronger evidence required before the
  three claims can move from unproved to proven: a jailed real ACP host, a
  real SWE_SEED host plus correlated authority declaration, and an implemented
  macOS Seatbelt backend with non-skipping conformance tests.

## Tasks 1–4 Specification Reconciliation

- Substrate map, reconciliation matrix, M0 gate evidence, and deferrals:
  `.agents/reports/2026-07-12-tasks-1-4-spec-reconciliation.md`.
- The standalone proposed patch, complete patched specification, and patch guide
  were not found in the repository or nearby project tree, so no `git apply` or
  `git apply --check` was possible. The proposals in the user request were
  evaluated manually against the code.
- `spec-full.md` now defines deterministic verdict resolution with `allow` as
  least restrictive, an opaque exact-action/context authorization boundary,
  canonical ledger-before-view failure semantics, M0-G1–G6, completion-claim
  levels, and cumulative release boundaries.
- The implementation plan assigns those implementation and proof obligations to
  Task 5 without prescribing an `AuthorizedAction` type or a parallel authority
  or persistence system.
- Public runtime execution and sandbox materialization now consume opaque,
  one-use, context-bound authority grants; direct ungranted effects do not compile.

- `cargo test -p sea-forge-planner --test criteria_provenance --locked`: 13 passed, 0 failed.
- `cargo test -p sea-forge-settlement --test criteria_provenance --locked`: 4 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m2 --locked`: 4 passed, 0 failed.
- `cargo test --workspace --all-features --locked`: passed; no regressions in P1–P4b or earlier milestones.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo test -p sea-forge-artifact-ip --locked`: 27 passed, 0 failed. Partial
  declaration, metadata-free execution result, and relabeled value-evidence tests
  were observed red before implementation and green afterward.
- `cargo test -p sea-forge-capability --locked`: 22 passed, 0 failed.
- `cargo test -p sea-forge-cli --test conformance_m8_artifact --locked`: 3 passed,
  0 failed.
- `cargo test -p sea-forge-authority --locked`: 36 passed, 0 failed.
- `git diff --check`: passed.
- `just proof`: P1–P4b passed.
- `just no-async-kernel`: passed.
- `just context-check`: passed.
- `devbox run -- just check`: all gates green.
- Continuation dependency step 2: `cargo test -p sea-forge-authority --locked`
  passed (39 tests); `cargo test -p sea-forge-cli --test conformance_m3 --locked`
  passed (5 tests); targeted authority/CLI clippy with all targets/features and
  `-D warnings` passed; `cargo fmt --all -- --check` passed.
- Continuation steps 3–4: `cargo test -p sea-forge-artifact-ip --locked` passed
  (29 tests); CLI M3 and M8 passed (5 tests each); planner passed (28 tests);
  authority passed (39 tests); workspace all-target/all-feature clippy with
  `-D warnings` and `cargo fmt --all -- --check` passed.
- Continuation steps 7–8: settlement passed (9 tests), authority passed (40 tests),
  artifact passed (29 tests), CLI M3 passed (5 tests), and CLI M8 passed (5 tests).
  Strict workspace all-target/all-feature clippy with `-D warnings` and formatting
  check passed. RED was observed first for missing continuation compilation, then
  for a run-workspace grant mismatch; both became green after implementation.
- Final repository gates: `devbox run -- just context-check`, `devbox run -- just
  check`, and `devbox run -- just test` all passed. Cargo-deny reported only the
  existing non-fatal duplicate/unmatched-allowance warnings.
- Final domain-blocker RED/GREEN: `cargo test -p sea-forge-artifact-ip --locked`
  first failed on the missing typed-anchor/lifecycle API, then passed 39 tests.
- `cargo test -p sea-forge-cli --test conformance_m8_artifact --locked`: passed 5
  tests after diagnosing and fixing the fixture's missing declared concept class.
- `cargo test -p sea-forge-authority --locked`: passed 40 tests.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  `cargo fmt --all -- --check`, and `git diff --check`: passed.
- Final requested `just proof` was run once: P1–P4b passed.
- Task 17 full gate through the context check: `cargo fmt --all -- --check`,
  strict workspace clippy, workspace all-feature tests, and `just proof` passed;
  `devbox run -- just check` then stopped only because this status update was due.
- After the status refresh, `devbox run -- just context-check`,
  `devbox run -- just check`, and `devbox run -- just test` all passed.
- After final review hardening, formatting, strict workspace clippy, workspace
  all-feature tests, and `just proof` passed again.
- Final `devbox run -- just context-check`, `devbox run -- just check`, and
  `devbox run -- just test` passed after review hardening.
- After ledger-verification hardening, formatting, strict workspace clippy,
  workspace all-feature tests, and `just proof` passed again.
- Final `devbox run -- just context-check`, `devbox run -- just check`, and
  `devbox run -- just test` passed after ledger-verification hardening.

## Blockers

- 2026-09-08 Interface Remediation: None.

- No external blocker for Task 0. Native evaluator automation remains an
  environment capability to be assessed in Task 3; the evaluator input names
  the required integrated behavior and explicitly forbids a mocked fallback.

- **Task 3 native OS prerequisite:** resolved. `tauri-driver 2.0.6` is installed
  in ignored Workbench tooling; `WebKitWebDriver` and `xvfb-run` are present.
  The runner uses its own software-rendered X display and the compiled renderer
  now mounts successfully without weakening the Tauri CSP. The existing
  Playwright harness remains explicitly mocked and is not a substitute for the
  passing real-stack gate.

- No blockers for the Just command-surface refactor.

- **Task 0.2 (M9 contract delta) — awaiting owner approval.** Seven additive
  changes; the only one with a real compatibility tradeoff is the closed
  `ProjectionKind` enum gaining `Kg` + `SelfModelSnapshot` (old readers reject
  records carrying the new variants; recommended policy: schema-tag the new
  self-model records `self_model.v1`, no global 0.2→0.3 bump). See the approval
  package in chat / this section.
- M10–M16 contract items (OriginRefKind::DesiredOutcome, ItemKind::AgentTask,
  Operation::AgentTask, self_disclosure/external_api/secret_access surfaces,
  ManagerIteration, settlement bases, cancellation records, authorship
  provenance) are inventoried but do NOT block Task 1; approval requested per
  task when each milestone enters scope.
- M12/M16 dependency selection (HTTP client, async strategy, URL, zeroization,
  ACP client) is blocked on explicit approval of exact versions/features; not
  needed for Task 1 (M9 declares no new dependencies).
- M13 transcript evidence uses the Task 0.4 decision (owner-accepted
  2026-07-17): retain a sealed, encrypted canonical transcript for `summarized`
  mode, verify it before crypto-shredding, expose only the deterministic summary
  by default. Canonically recorded in `spec-agent-orchestration.md` "Resolved
  decisions"; `OPEN_QUESTIONS.md` entry retired. Gates M13, not M9.

## Task 0.1 — Baseline re-run (2026-07-16)

Cumulative gate on `b351c95`, branch `full-spec`, fresh worktree:

- `devbox run -- just context-check` — passed.
- `devbox run -- just check` — passed (fmt-check, clippy `-D warnings`
  workspace/all-targets/all-features, typecheck, security). cargo-deny emitted
  only the known non-fatal getrandom 0.2/0.3 duplicate (transitive via
  domainforge-core 0.13.0); no fatal advisories.
- `devbox run -- just test` — passed (full `cargo test --workspace
  --all-features --locked`, exit 0; prior CI-green record = 289 tests; no
  platform skips).
- `devbox run -- just proof` — P1–P4b passed.
- `devbox run -- just no-async-kernel` — "ok: no tokio in kernel crates".
- Tracked `.sea-forge/**`: 0 files (confirmed via `git ls-files`).

## M9 progress (Task 1)

- Slice 1.2a (commit): added `ProjectionKind::{Kg, SelfModelSnapshot}` (appended
  to preserve existing Ord), `ForgeError::SelfModel` (class `self_model_error`),
  and `ids::snapshot_id()` (`smsnap_`). Compatibility boundary proven by 11 new
  tests: `crates/sea-forge-core/tests/projection_kind_boundary.rs` (serde, 6),
  `crates/sea-forge-ledger/tests/projection_boundary.rs` (record_kind skip +
  verify-immune, 3), `crates/sea-forge-cell/tests/projection_bundle_boundary.rs`
  (E6 hash import, 2). No global schema bump; M0–M8 records/readers unchanged.
  `cargo fmt`, clippy (`-D warnings`, 5 affected crates), workspace check, and
  affected-crate tests all green.
- Slice 1.1 (commit): added release-owned model assets `models/seaforge-system@0.1.0.sea`
  (canonical Genesis self-model, namespace `godspeed.seaforge.system`, 90
  concepts) and `models/adlc-odi-case@0.1.0.sea` (from the seed, re-versioned
  to 0.1.0, 131 concepts). Both validate through `load_validate`. Asset sha256
  (pinned release constants for slice 1.3): system=
  `09ead9ac9514009d60de0da18d936b82aa6a94a86db94c00dde7f3da54b77cfa`; adlc=
  `8c891cff33fe7bb012ebb0fda6232bc8e996d5a1af47e9c5a675f2cc7a143613`; original
  seed (spec-cited) =   `2ea06fc9c59d28fac9b9d47f18c4787b2ffb740b0527513a70556cf91dcbffc5`.
- Slice 1.2b+1.3 (commit): added synchronous kernel crate
  `sea-forge-self-model` (workspace member; added to `no-async-kernel`
  inventory). Provides `BundledModels` + `verify_bundled` (byte-check against
  pinned release sha256 before validation), `load_composed` (validates both
  models through DomainForge), `ComposedModel` (typed read-only concept lookup,
  no raw graph), `ReleaseRealization` (deterministic via SOURCE_DATE_EPOCH),
  `CellRealization` (+ `ToolchainProbe`/`ProbeResult`), `SelfModelSnapshot`
  (five digest fields + `snapshot_hash`), and canonical (jcs-nfc-v1-aligned)
  hashing. T9.1 + T9.2 green (lib 4 + conformance 2); clippy/fmt clean;
  no-async-kernel green.
- Slice 1.4 (commit): added `build_cell_realization` — assembles a cell
  realization from registry state, environment contracts, and evidenced probe
  results; a missing/failed/unverifiable probe records `Unavailable` + evidence
  ref and lists the tool under `degraded_components` (caps status, never
  elevates, never crashes snapshot creation). No probe state is cached between
  builds (V5). Probe *execution* stays in the CLI/server layer (slice 1.5); the
  crate only consumes evidenced results. T9.5 + V5 green.
- Slice 1.5a (commit): KG/CALM/JSON self-projections + persistence/lifecycle.
  `domainforge::project` gained a `Kg` arm (Turtle KG). `sea-forge-self-model`
  gained `projections` (project_self → 3 ProjectionRecords with deterministic
  rebuild_hash + verify_projection; byte-identical across rebuilds modulo
  created_at) and `store` (manifest-gated init/upgrade/rebuild, immutable
  snapshot files, rebuildable projections, mark_current_stale without mutating
  snapshots, validate). T9.3 (projection determinism + drift rejection) and
  T9.4 (extension-disable rebuild keeps prior snapshot verifiable) green; init
  idempotency green. CLI wiring (validate/rebuild/show) is the next sub-slice.
- Slice 1.5b (commit): CLI `sea-forge self-model validate|rebuild [--probe]
  [--capability-hash]|show [--json]` wired through `commands::self_model` over
  the store. domainforge Kg output renamed to `model.ttl` (no double nesting).
  CLI integration test (binary spawn) green: rebuild→validate→show--json
  round-trip, distinct snapshot on re-rebuild, and corrupt-snapshot ⇒ exit 1
  `self_model_error`.

## M9 gate (2026-07-17) — GREEN

Cumulative gate on `full-spec` after all M9 slices:

- `devbox run -- just context-check` — passed.
- `devbox run -- just check` — passed (fmt, clippy `-D warnings`
  workspace/all-targets/all-features, typecheck, security).
- `devbox run -- just test` — passed; **313 tests, 0 failed, 0 platform skips**
  (baseline was 289; +24 new M9 tests: 11 ProjectionKind boundary + 4
  self-model lib + 7 conformance_m9 [T9.1–T9.5, V5] + 2 CLI integration).
- `devbox run -- just proof` — P1–P4b passed (unchanged).
- `devbox run -- just no-async-kernel` — "ok: no tokio in kernel crates"
  (sea-forge-self-model included in the inventory).
- Tracked `.sea-forge/**`: still 0 files.
- T9.1–T9.5 + V5 all green. M9 is code-complete and gated. Remaining: M10–M16.

## M12 progress (Task 4 — E14 AgentProvider seam)

**M12 COMPLETE and GATED (408 tests, P1–P4b, no-async-kernel, cargo-deny).**
Base landed in af94ff0; gap closure in c6aebb6; license/spec/status update
in this change.

M12 gate (2026-07-20): `just context-check`, `just check` (fmt + clippy
`-D warnings` workspace/all-targets/all-features + cargo-deny
licenses/bans/sources), `just test` (408 tests, 0 failed, 0 platform
skips), `just proof` (P1–P4b), `just no-async-kernel` (18 kernel crates)
all green. T12.1–T12.6 green. Tracked `.sea-forge/**` still 0 files.

deny.toml: added `CDLA-Permissive-2.0` to the license allow-list —
carried by `webpki-roots` (Mozilla root CA bundle), a transitive dep of
the approved `reqwest` rustls-tls feature. Permissive license, not
copyleft; mechanical consequence of the approved M12 dependency.

spec-agent-orchestration.md: §5 claim table records M12 evidence
(AgentProvider seam, declared-config-not-status, endpoint failure
taxonomy); §17.1 T12 table annotated with status; §17.6 records the
GREEN gate; §18 checklist M12 items checked; summarized-transcript row
updated to the resolved sealed-transcript decision. `sea-forge-agent` adapter crate with
OpenAI-compatible + Anthropic providers (object-safe `AgentProvider` via
`BoxFuture`, no async-trait dep), `Operation::AgentProbe` + exact-action
`AuthorityAction::AgentProbe` (binds endpoint_ref +
descriptor_config_sha256 + normalized scheme/host/port/path/model/limits

- credential_ref + prompt_sha256 so a config reload cannot repoint an
authorized call), `external_api` surface `allow_hosts` enforcement,
DNS-rebinding-safe client pinning (`resolve_to_addrs`), `no_proxy` +
`redirect::Policy::none()` + HTTPS-only (explicit loopback test mode),
private/loopback/link-local/multicast/metadata-address rejection, separate
`secret_access` mediation before credential resolution, `Zeroizing<String>`
credential handling, immutable `runtime_adapter` endpoint registration
(descriptor change requires a new version), governed `agent_probe` service
creating intent→plan→authority→evidence→settlement with typed error
classes, and CLI `agent list|probe` over the server socket. T12.1–T12.3
green (4 server conformance tests); provider-contract tests pin request
shapes, paths, and auth headers for both provider kinds.

Verification (worktree, pre-commit of base): `cargo fmt --all -- --check`
clean; `cargo clippy --workspace --all-targets --all-features --locked
-D warnings` clean; `cargo test --workspace --all-features --locked`
**394 tests, 0 failed, 0 platform skips** (was 372 after CEP-0008; +22:
8 agent lib + 3 provider-contract + 4 server conformance_m12 + 1
authority m12 exact-action + 3 core/extension/case_engine AgentProbe +
3 agent config/network tests).

Dependencies (owner-approved per plan slice 0.3, confined to adapter
crates): `reqwest 0.12` (default-features=false, features
json+rustls-tls+stream), `url 2.5`, `zeroize 1.8`. Cargo.lock refreshed.

Remaining M12 gaps (before cumulative gate):

- T12.5 dependency-boundary gate: rewrite justfile `no-async-kernel` to an
  explicit kernel-crate inventory (add sea-forge-domainforge,
  sea-forge-spec-pipeline, sea-forge-cell, sea-forge-artifact-ip) and a
  forbidden-dependency set covering async runtimes AND HTTP clients
  (tokio, reqwest, hyper, async-std, …), excluding only approved adapter
  crates (sea-forge-agent, sea-forge-server).
- Slice 4.2 finish: `ServerConfig::load` must call `agent.validate()`
  (last-known-good on invalid); endpoint registration must mark the
  self-model snapshot stale when one exists
  (`sea_forge_self_model::store::mark_current_stale`).
- Slice 4.5 finish / T12.6: probe-level error-taxonomy tests for
  unreachable/4xx/5xx/oversize/redirect/schema-invalid → rejected
  settlement + typed `error_class` + no fallback; CLI exit code for
  rejected probe aligned to repo convention (exit 3).
- Streaming (spec §7.4 redaction + split-chunk sweep) is E15/M13 scope;
  M12 probe is non-streaming (hash-only persistence ⇒ sweep trivially
  holds). Recorded in spec §5 claim table.
- Spec §5 claim table update + cumulative gate + this status refresh.

## Decisions

- **2026-09-08 Interface Remediation Decisions**:
  - Upgraded `DomainModelRef` to "v2-full-preimage" with full 7-tuple preimage binding (`identity_scheme_version`, `domainforge_version`, `adapter_descriptor_sha256`, `parse_options_sha256`, `source_refs`, `d_content_hash`, and `semantic_closure_hash`).
  - Used real computed `ADAPTER_DESCRIPTOR_SHA256` (`sha256:16fcd1519e00727b2d17c68c3ddbefd633dacf3dcadd51a0ef37a900a119460e`).
  - Added `#[serde(default)]` fallback for `identity_scheme_version` to preserve deserialization of legacy records as `"unknown-pre-versioning"`.
  - Registered `godspeed.event.v1-flat` schema profile in CEP-0008 adapter.

- Task 0 records the owner-approved exclusions as a JSON evaluator fixture,
  rather than duplicating an informal list in the evaluator prompt. The
  validator requires its exact story-ID set, nonempty owner reasons, Linux
  claim, package/start commands, and no protocol placeholders. It is a local
  Workbench command only; Task 12 is the authorized CI aggregation point.
- Task 1 keeps actor selection renderer-local and session-only, while the host
  resolves and validates the authoritative actor again for each protected
  request. The shared guard is intentionally a conservative affordance check;
  it never decides policy or authority.

- Keep the root guide at approximately 150 lines, expose only `just` commands,
  and route Workbench-specific commands and generated-zone detail through
  `workbench/AGENTS.md`.
- Treat `justfile` as the canonical recipe implementation. Document aggregate
  gate coverage and exclusions instead of copying recipe bodies into the guide.

- Pipeline moved to `sea-forge-cli` (not kept in `sea-forge-core`) because keeping
  it in core would create a circular dependency once authority/runtime/sandbox
  moved to their own crates. This matches the plan’s allowance: “pipeline.rs stays
  in core or moves to cli — keep wherever the diff is smallest.”
- `sea-forge-extension` starts with a minimal lib.rs re-exporting descriptor
  types from `sea-forge-core`; it will gain registry logic in Task 4 (M0d).
- The `timed_out_child_pid_is_no_longer_alive` test’s child-process test-name
  argument was updated to match the new crate-local test path; this is a required
  mechanical reference update, not a logic or proof change.

## Audit Remediation Plan — Task 0 (2026-07-22)

Source: `.agents/plans/2026-07-22-spec-audit-remediation.md`, built from
`.agents/reports/2026-07-22-spec-implementation-audit-independent-validation.md`.
Task 0 freezes the clean baseline and records owner-approved dependency and
contract choices for Tasks 1-18 before any remediation code lands.

- **ADR-002** (`docs/decisions/ADR-002-audit-remediation-dependencies.md`):
  approves `rusqlite 0.32` (`bundled`, for M4b SQLite FTS — confirmed FTS5 is
  always compiled into bundled builds, no separate feature flag exists).
  **Note:** ADR-002 originally recorded `rusqlite 0.40`, but that version is
  **superseded** — `libsqlite3-sys 0.38.1`'s `build.rs` unconditionally
  invokes the still-unstable `cfg_select!` macro (rust-lang/rust#115585) and
  fails to compile on this repo's pinned `rustc 1.92.0`. ADR-002 was amended
  in place to pin `rusqlite 0.32` (→ `libsqlite3-sys 0.30.1`, still
  FTS5-bundled, same license) as the **final approved dependency version**;
  `rusqlite 0.40` must not be reintroduced. Also approves
  `landlock 0.4` (for M1 jail TCP bind/connect denial via `AccessNet`,
  replacing the hardcoded `ABI::V1`; UDP/raw-socket coverage remains outside
  Landlock's scope in every ABI through V6 — this matches `spec-full.md:1434`
  verbatim, which already scopes the requirement to "Landlock v4+ TCP
  restrictions where available, else document the gap"), and
  `chacha20poly1305 0.11` (`zeroize` feature, for M13 sealed summarized-mode
  transcripts — per-run random key at `.sea-forge/sealed/<run_id>.key`,
  XChaCha20Poly1305, crypto-shredding = deleting that key file). All three
  versions were confirmed live against Context7 and the crates.io API on
  2026-07-22, not recalled from training data. All three were presented to
  the owner as `AskUserQuestion` choices and approved as the recommended
  option in each case.
- **ADR-003** (`docs/decisions/ADR-003-audit-remediation-contracts.md`):
  inventories additive contract deltas per task. Notably, several deltas the
  plan's steps describe as "add a field" turned out to already exist in
  source (`Operation::AgentTask.{response_schema,transcript_retention}`,
  `PolicyRule.memory_scope`, the M9 self-model `ProjectionKind` variants,
  `OriginRefKind::DesiredOutcome`) — those tasks (6, 11, 12, 15) are
  behavior/wiring fixes, not contract changes, and are not blocked on this
  ADR. Genuinely new deltas (Tasks 5, 9 conditional, 10A conditional, 13/13B,
  14A/14B, 16, 17) all follow the repository's existing additive-compatibility
  convention (optional defaulted fields; new enum variants gated by
  record_kind or accepted as a clean old-reader `serde` error) — no
  `RECORD_VERSION`/global schema bump is required for this plan.
- **Baseline maintenance**: `just check`'s `gitleaks detect` step was failing
  on two long-known false positives — the `SECRET_SENTINELS` pattern-string
  constant in `sea-forge-ledger/src/types.rs` (contains the literal string
  `"-----begin private key-----"` as a redaction pattern, not a real key) and
  `conformance_m0_ledger.rs`'s test asserting a fake `sk-1234567890abcdef`
  payload is rejected by that same redaction mechanism (both confirmed benign
  by reading the actual lines, not just trusting the prior `.gitleaksignore`).
  Root cause: `gitleaks detect` scans full `git log -p` history, so the same
  line can be re-flagged under a *different* commit hash whenever an
  unrelated nearby edit shifts diff context — the repo's history has two such
  commits (`b4464a91…` original, `effc01c455…` a later shift) for each of the
  two lines. Fix: re-added all four resulting fingerprints to
  `.gitleaksignore` (confirmed via `gitleaks detect --redact -v`, now `no
  leaks found`) and added inline `// gitleaks:allow` comments on both lines
  so future commits never regenerate a fifth fingerprint for the same
  content.
- Global gate (`just context-check && just check && just test && just proof
  && just no-async-kernel`) run clean after the operator's `cargo clean` and
  the gitleaks fix above — **all green**: `context-check` passed; `check`
  (fmt, clippy `-D warnings` workspace/all-targets/all-features, `cargo
  check --locked`, `cargo deny check` — advisories/bans/licenses/sources ok,
  `gitleaks detect` — no leaks found) passed; `test` — **488 passed, 0
  failed, 2 ignored** (the two ignored are the documented real-host release
  gates, `t16_1_real_acp_host_release_gate` and
  `t16_6_real_swe_seed_release_gate`, both requiring operator-configured
  external hosts per their own skip messages — not a portable-gate gap);
  `proof` — P1-P4b passed; `no-async-kernel` — "ok: no async runtime or HTTP
  client in 19 kernel crates". This is the frozen clean baseline Tasks 1-18
  build on.
- `.agents/OPEN_QUESTIONS.md` has no unresolved entries after Task 0 — all
  three dependency choices and the network-isolation scope question were
  resolved by owner confirmation in-session rather than left open.

## Task 8 — the run record (epic 12.1, 12.3, 12.4; unblocks 7.4, 7.5, 9.x, 13.x)

**Slice chosen for blast radius.** Of the epic's open journeys, the run record
was the one whose absence blocked the most downstream work: every other view
already emitted run ids that resolved to nothing. Implementing it turns four
existing surfaces from display-only into navigable, and gives journeys 9
(execution evidence), 11.8 (failure diagnosis), 12 (audit truth), and 13
(capability from settlements) the record they all have to hang off.

**Server** — `crates/sea-forge-server/src/sfwp/run_views.rs`, additive per
ADR-003:
- `run.list` (optional `case_id` scope) over `<root>/runs/*`, cross-indexed
  against `cases/*/case.json` for ownership. Runs no case claims stay listed as
  unclaimed rather than filtered out (epic 11.6 — process death must not hide
  work).
- `run.get` — the linked view: plan item, authority projection, criteria paired
  against the settlement basis, settlement detail, declarations, evidence rows,
  trace rows, and a presence inventory of every canonical run file.
- Both advertised via `IMPLEMENTED_METHODS`; eleven new contract types added to
  `SCHEMA_TYPES` and `gen_sfwp_schema.rs` (which had silently drifted apart —
  eight Task 7 types were in the generator but not in `system.get_schema`; both
  lists are now complete).

**Frontend**:
- `hooks/governedQuery.ts` — one invoke → reject-governed-error → AJV-validate
  path, replacing three copies of `describeAjv` and eight bespoke throw sites.
  The order is load-bearing: a governed error body can never satisfy a view
  schema, so validating first would report every denial as a contract failure.
  `useCases`/`useApprovals` now route through it (which also closed a real gap —
  `approval.list` never checked for the error envelope at all).
- `pages/standing.ts` — the standing→pill maps, shared rather than copied,
  because each encodes a governance rule (`completed` is `degraded`, never
  `ready`) that would eventually be enforced in only one copy.
- `pages/RunRecordPage.tsx` at `/runs/$runId`; `pages/EvidencePage.tsx` replaces
  the `UnbackedSurface` stub with a `run.list`-backed index; the case horizon's
  `Episodes: N` count became one link per attempt.

**Gates**: `cargo fmt --all --check` clean; `devbox run -- just check` — all
gates green (fmt, clippy, cargo-deny, gitleaks); `cargo test --workspace
--all-features --locked` — all suites pass; frontend `bun run check` clean and
`bun run test` — 79 passed / 14 files.

**One flake observed, not caused here**: `sea-forge-cli`'s
`kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption` failed once
under full-workspace parallel load and passes in isolation. It is a `kill -9`
timing test and no CLI code was touched by this task.

**Next spendable slice**: `9.3`/`9.8` (follow non-agent execution, preserve
evidence from every termination) now have their record surface and need only
artifact/stdout access; or `13.4` (capability records), which can read the
declaration rows this task already projects.

**Iteration entry point**: `.agents/WORKBENCH-SLICE-PROMPT.md` holds the
idempotent prompt for advancing the workbench one slice at a time. It derives
state from this file, `IMPLEMENTED_METHODS`, and `router.tsx` on every run
rather than from a remembered plan, so it selects the same next slice against an
unchanged tree and never re-does shipped work.

## Task 9 — the governed asset catalog (epic 4.1, 4.5, 4.7, 4.8; unblocks 4.6, 9.4, 9.5, 6.7)

**Slice chosen for blast radius.** Three surfaces still rendered copied
specification data (Thoth, Assets, Models). Assets was picked over Thoth because
of what sits *downstream*, not because journey 4 is longer:

- Journey 9's delegation stories (9.4 configure an agent task, 9.5 run HTTP/ACP
  agents, 9.7 monitor and intervene) all begin with "choose an eligible
  endpoint". The kernel has spoken `agent_list`, `agent_probe`, `delegate`, and
  `cancel_delegation` since M12 — the capability was real and unreachable from
  the workbench, the same shape as the `approval.decide` gap Task 7 closed.
- The Assets specimen was the *worst-behaved* of the three. Thoth and Models
  fabricate layout; Assets fabricated an availability ladder
  (`Available`/`Installed`/`Declared`) that `AgentEndpointConfig::validate`
  explicitly refuses to let even *configuration* assert, because that ladder is
  evidence-derived. The renderer was claiming what the kernel rejects.

Runner-up was `thoth.ask` (journey 3, nine stories). Passed over because it is
not one slice: it needs a typed response contract, a disclosure-gating decision,
and an AG-UI stream — and nothing downstream is blocked by its absence, since a
Thoth answer is an inspection aid that authorizes nothing.

**Server** — `crates/sea-forge-server/src/sfwp/assets.rs`, additive per ADR-003:
- `asset.list` over three sources the kernel already owns: materialized
  templates (`<root>/templates/*.yaml`), configured agent endpoints
  (`server.yaml`), and the extension registry.
- Endpoint standing folded from committed records, never asserted: `declared`
  (configured only) → `probed` (a registry runtime-adapter entry, which
  `agent_probe::register_endpoint` writes only after an allowed authority
  decision, or a probe run that settled) → `demonstrated` (an *accepted* probe
  settlement). The most recent probe decides, so an endpoint that has started
  failing does not keep an old demonstration.
- Advertised via `IMPLEMENTED_METHODS`; `AssetListResult`/`AssetRow`/`AssetKind`
  added to `SCHEMA_TYPES` and `gen_sfwp_schema.rs`.

What the design had to get right:

- **Three vocabularies stay three.** The obvious simplification is one shared
  availability enum. It would make a quarantined extension and an unprobed
  endpoint render identically — erasing *refused* vs *not yet proven*, which is
  the entire content of epic 4.8. Each row carries its own kind's word verbatim
  (`materialized` / `declared|probed|demonstrated` / `active|disabled|
  quarantined|superseded`) and the UI maps each separately.
- **Standing and blocking are separate fields.** They answer different
  questions. An extension can be `active` and still refused because its trust
  level is quarantined; an endpoint can be `declared` — proven nothing — and be
  perfectly lawful to probe. One field would have to drop one of the two facts.
- **Absence is reported, not dropped.** `case.entry_options` silently skips a
  template it cannot parse and `agent_probe::list` silently drops an endpoint
  that will not snapshot, so a broken file reads as "no such asset" in both.
  `asset.list` lists it with the reason and names the source in `unreadable`.
- **The floor stays the floor.** A configured endpoint with no evidence is
  `declared` with an empty `evidence_refs` — an honest empty list, not a
  missing one, and never upgraded on the strength of being configured.

**Frontend**:
- `hooks/useAssets.ts` over `queryGoverned`. Deliberately *not* event-
  invalidated: no `KNOWN_EVENT_KINDS` entry announces a template, endpoint, or
  extension change, so subscribing to the case/run taxonomy would look like
  liveness without being it. The page says so and offers an explicit re-read.
- `pages/AssetCatalogPage.tsx` replaces the specimen `AssetsPage`. Three
  kind-scoped tables rather than one — a merged table would put the three
  vocabularies in one column and invite reading them as one ladder. `run:<id>`
  evidence refs link to the run record; ledger ids render as ids rather than as
  links that go nowhere.
- `pages/standing.ts` gains `ASSET_STANDING_VARIANT`; nothing in it maps to a
  blocked variant, because blocking is the other column.
- `SURFACED_METHODS` gained `asset.list` — **and `run.list`/`run.get`, which
  Task 8 shipped, wired, and rendered but never listed.** `/admin` had been
  reporting two live methods as "implemented, not surfaced" ever since.

**Neatcode judgment.** Two shared things, both load-bearing; nothing else.
`run_views::read_json` became `pub(crate)` rather than being copied — the two
projections must agree that a half-written record is not a record.
`ASSET_STANDING_VARIANT` lives with the other standing maps for the reason
stated there. Explicitly *not* built: an `asset.get` detail method (the row
carries what the detail panel needs), a server-side `kind` filter (the catalog
is small and a second place to decide "which assets exist" is a liability), and
a shared template-enumeration helper (two readers, not three — logged instead).

**Gates**: `cargo fmt --all -- --check` clean; `devbox run -- just check` — all
gates green (fmt, clippy, cargo-deny, gitleaks, no leaks); `cargo test
--workspace --all-features --locked` — all suites pass, zero failures. Two
tests are **skipped**, both pre-existing and unrelated: `self_invoke_noop_pass`
and `self_invoke_noop_fail` are `#[ignore]`d in the CLI suite. The Tauri host
crate is outside the workspace and was run separately (`cargo test` in
`src-tauri`): 2 lib + 4 integration tests pass. Frontend `bun run check` clean
apart from the pre-existing `router.tsx` fast-refresh warning;
`bun run --cwd apps/desktop test` — 85 passed / 15 files (was 79 / 14).

**Tests added**: 12 in `crates/sea-forge-server/tests/conformance_assets.rs`
(ladder floor, accepted → demonstrated, rejected → probed-and-blocked-with-its-
basis, most-recent-probe-wins, no evidence leakage between endpoints, registry
registration reaching `probed` but not `demonstrated`, no double-listing of a
registered endpoint, quarantined trust blocking an `active` extension,
unparseable template reported not dropped, empty cell staying empty, catalog
advertised as `inspect`); 3 unit tests in `sfwp/assets.rs`; 9 in
`pages/AssetCatalogPage.test.tsx`; 1 in the host bridge; and one closing a
documented debt — `generated_schemas_are_committed_and_current` now asserts
`SCHEMA_TYPES` equals the generator's emitted filenames, so the comment claiming
they "never drift" is finally enforced.

**Next spendable slice**: `9.4` (configure an agent task) is now unblocked and
is the highest-leverage follow-on — endpoints are enumerable with real standing,
`delegate` exists in the kernel, and the missing piece is a job-contract
inspection view plus a `delegation.preview`-shaped inspect method. The
alternative is `thoth.ask`'s response contract (journey 3), which remains the
largest single unclaimed block but is at least two slices wide.

---

## Task 10 — the delegation job contract (epic 9.4; unblocks 9.5, 9.7, 9.2)

**Slice**: `delegation.preview` — an SFWP inspect method that projects the
complete job contract a `delegate` with these exact inputs would run under, plus
a `/delegate` surface that reads it.

**Why this one.** The runner-up was `thoth.ask` (journey 3, nine stories), passed
over for the same reason as last slice: it is a response contract *plus*
disclosure gating *plus* an AG-UI stream, which is at least two slices, and
nothing downstream is blocked by its absence. 9.4 was picked because it is the
step every remaining journey-9 story starts from — 9.5 (provider parity), 9.7
(monitor and intervene), and 9.2 (the granted sandbox) all presuppose a
configured, inspectable agent task — and because the kernel's `delegate` has
been reachable-but-blind since M12: the only way to learn what a delegation
would do was to run one, which is exactly the side effect being decided about.

**Settles** 9.4. **Unblocks** 9.5, 9.7, 9.2.

**Server** (`crates/sea-forge-server/src/sfwp/delegation_preview.rs`):
- `DelegationPreviewParams` takes *exactly* the inputs `Request::Delegate`
  accepts. A preview knob the command cannot take would describe a delegation
  nobody can run.
- The contract carries provider kind, endpoint digest, resolved model and
  transcript retention (each with a `ValueSource`), instruction hash + size
  against the endpoint's own cap, response cap, timeout, turn cap, token budget,
  the authority action kind, and a `contract_digest`.
- `eligible` means only "no precondition known at preview time is unmet". The
  method deliberately does **not** evaluate authority: a verdict with no ledger
  entry behind it would be an unrecorded grant. It names the gate (`agent_task`)
  and stops.
- Standing, evidence refs, and evidence-derived blocking come from
  `sfwp::assets` rather than being re-derived, so `/assets` and `/delegate` can
  never disagree about whether an endpoint is usable.
- An unconfigured endpoint gets **no** standing word rather than being demoted
  to `declared` — absence reported, not inferred onto a ladder it was never on.
- Absent `token_budget` is omitted, never zeroed: a zero budget and no budget are
  opposite instructions to the runner.

**Bug found and fixed.** `delegate_inner` passed `..Default::default()` for
`DelegationRequest::transcript_retention`, whose doc comment says the caller has
already resolved it. Every socket-issued delegation was therefore pinned to
`summarized`, silently ignoring an endpoint that had asked for `full`
transcripts (`case_dispatch` resolved correctly; the standalone verb did not).
Fixed to call `TranscriptRetentionMode::resolve` through the same chain. This
was in scope rather than deferred: a preview reporting `full (from the endpoint
descriptor)` while execution ran `summarized` would be a projection lying about
truth, which is the one thing this method exists not to do.

**Frontend**: `hooks/useDelegationPreview.ts` (parameterised — the request is
the query key, and an uncommitted request issues no query); `pages/
DelegationWorkbench.tsx` at `/delegate`, plus a sidebar entry. Blocked endpoints
stay in the picker: hiding one would report an endpoint that exists and is
refused as one that is absent, and the refusal is what the operator came to
read. Editing after reading marks the contract as describing the earlier
request rather than silently re-attributing it to the current form. There is no
"Run delegation" button — `delegate` is a protected command and belongs behind
`ProtectedActionButton` with its own preconditions, which is the next slice.

**Neatcode judgment.** Two extractions, both load-bearing, no new abstractions.
`delegation::check_preconditions` now holds the four request-shape rules that
were about to exist in two copies — execution takes the first unmet rule, the
preview lists all of them, and neither can drift. `action_for_delegation` became
`pub(crate)` so the preview reports the *same* authority action that will be
submitted, including the descriptor hash, rather than a second spelling of it.
`VALUE_SOURCE_LABEL` joined the existing standing maps and is deliberately not a
pill: provenance is different, not better or worse, and a `ready`/`degraded`
treatment would rank "you chose this" above "the endpoint chose it". Explicitly
*not* built: a read-only authority dry run (see above), a `delegation.commit`
envelope, a debounced live preview (an explicit read plus a staleness marker is
less code and does not manufacture contracts nobody chose), and a new CSS module
(`RunRecordPage.module.css` already carried the layout).

**Gates**: `cargo fmt --all -- --check` clean; `devbox run -- just check` all
green; `cargo test --workspace --all-features --locked` — exit 0, **101 suites,
762 passed, 0 failed, 4 ignored**.
**Those four are the skipped ones**, all `#[ignore]`d, all pre-existing and untouched by
this slice: `self_invoke_noop_pass` and `self_invoke_noop_fail` (self-invocation
fixtures in `sea-forge-case-runner`, deliberately never executed by a normal
run) and `t16_1_real_acp_host_release_gate` / `t16_6_real_swe_seed_release_gate`
(release gates needing operator-supplied real ACP / SWE_SEED hosts). No flakes
observed this run. Correction to the Task 9 note above: it said two skipped
tests in the CLI suite — there are four, and the `self_invoke_*` pair lives in
`sea-forge-case-runner`, not the CLI. The Tauri host crate is outside the
workspace and was run separately in `src-tauri` (3 lib + 4 integration, pass). Frontend
`bun run check` clean apart from the pre-existing `router.tsx` fast-refresh
warning; `bun run --cwd apps/desktop test` — 98 passed / 16 files (was 85 / 15).

**Tests added**: 14 in `crates/sea-forge-server/tests/conformance_delegation_
preview.rs` (contract projection; **nothing written to runs/cases/ledger**;
authority named but never decided and no verdict field present; unconfigured
endpoint has no standing and no contract; every unmet precondition reported not
just the first; over-long instruction blocked against the endpoint's own cap;
instruction reported by hash and size and never echoed; requested vs endpoint
model provenance; retention resolved endpoint-then-cell; a rejected probe
blocking the preview with the *same words* `asset.list` uses; accepted probe
evidence carried through; contract digest moving with request and descriptor;
absent token budget omitted not zeroed; advertised as `inspect`); 4 unit tests
in `sfwp/delegation_preview.rs`; 1 in the host bridge (flattened params, no
nulls); 13 in `pages/DelegationWorkbench.test.tsx`.

**Next spendable slice**: `9.7` (monitor and intervene in agent dialogue) —
`delegate`/`cancel_delegation` both exist in the kernel and `agent_run.delegated`
is already an event kind, so the missing piece is a `delegation.list`-shaped
inspect method over live/finished delegations plus a cancel path behind
`ProtectedActionButton`. That would also give `/delegate` its command half. The
alternative remains `thoth.ask`'s response contract (journey 3): still the
largest unclaimed block, still at least two slices wide.

---

## Task 11 — the delegation roster and per-run cancel (epic 9.7, partially; unblocks 9.8)

**Slice**: `delegation.list` — an SFWP inspect roster joining the server's live
delegation handles with the committed run records, plus a `/delegate` section
that cancels exactly one delegation.

**Why this one.** `cancel_delegation` has been a kernel verb since M12 and the
Tauri host has carried `SfwpCommand::CancelDelegation` since Task 3 — but
nothing could enumerate what there was to cancel. That is precisely the shape of
the `approval.decide` gap Task 7 closed: a capability whose targets are
undiscoverable is not a capability an operator has. It also completes the
command half of `/delegate`, which Task 10 named as its own follow-on.

**Scope honesty — 9.7 is settled in part, not in full.** The story asks for
"bounded turn, token, streaming, permission, and continuation state" *during*
the dialogue. The kernel does not expose that: `DelegationHandle` carries only a
case id and two atomics, and the per-turn loop publishes no event. Turn and tool
counts therefore appear only once `transcript-evidence.json` is written — after
termination. The roster reports them as **absent while running** rather than as
`0`, and the kernel-side gap is logged rather than papered over. What *is*
settled is the control half: see every delegation, and cancel one without
touching its siblings. **Unblocks** 9.8 (terminations are now reachable from a
roster rather than only from a run id someone already had).

**Server** (`crates/sea-forge-server/src/sfwp/delegations.rs`):
- Joins two sources that cannot be merged — the in-memory handle map (live, not
  durable) and `runs/<run_id>/` (durable, only once terminated).
- `DelegationStanding` is a closed four-variant lifecycle vocabulary, distinct
  from the existing execution and settlement ones. The load-bearing variant is
  `unresolved`: a delegation run with no settlement *and* no live handle. That
  is what a server restart mid-episode produces, and the kernel cannot say what
  happened to it — so neither does the projection. `cancelled` would invent a
  request nobody made; `failed` would invent a settlement nobody recorded.
- Standing is not settlement. A delegation can be `settled` and rejected, or
  `settled` after having been cancelled; the verdict is its own field carrying
  the settlement record's own word.
- `cancellable` is per-run by construction — the cancel flag lives on that run's
  own handle — and the UI has one button per row with no bulk affordance.

**Frontend**: `hooks/useDelegations.ts` (roster + cancel, event-invalidated
unlike the asset catalog, because both `agent_run.*` kinds move a row);
`pages/DelegationRoster.tsx` mounted on `/delegate` with per-row
`ProtectedActionButton`. A successful cancel is reported as **requested**, never
as "cancelled" — the kernel records a control request and the episode still
terminates on its own terms. Reporting the outcome in place of the request would
be the execution-equals-settlement conflation the epic forbids everywhere else.

**Two bugs found and fixed en route.**
1. `hooks/eventKinds.ts` listed three event kinds while claiming to be "verified
   against its publish sites"; the server has been emitting five since M12
   (`agent_run.delegated`, `agent_run.cancellation_requested` were missing).
   Nothing rendered stale because the narrowing is asymmetric and unknown kinds
   invalidate — which is exactly why it went unnoticed. Now listed, classified,
   and pinned by a new `eventKinds.test.ts`.
2. **My own Task 10 test was partly vacuous.** `preview_creates_no_run_case_or_
   ledger_entry` checked that a directory named `ledger` stayed empty; the
   kernel's path is `ledgers`, so that third of the assertion could never fail.
   Both that test and the new roster equivalent now diff the whole cell tree
   before and after, which cannot be fooled by a name the author did not think
   of.

**Neatcode judgment.** Two extractions, both at thresholds previously named, and
no new abstractions. (1) `run_views::run_dirs` and `case_index` became
`pub(crate)`: run enumeration had reached its third hand-rolled copy — the exact
trigger recorded in `OBSERVED_DEBT.md` last slice — and "a run belongs to the
case that claims it" is a rule two views must not disagree about. Each caller
keeps its own readability policy, so nothing gained a policy parameter.
(2) `hooks/useGovernedEventInvalidation.ts` replaces **five** hand-copied
`listen("sfwp://event")` blocks that differed only in predicate and query key.
They each carried two rules subtle enough to drift: `event?.payload` (a throwing
listener tears down the subscription) and the `disposed` flag (a late-resolving
`listen()` promise leaks a listener after unmount). `useOperationsStream` was
deliberately left alone — it consumes frames as data, which is a different job.
Explicitly *not* built: a `delegation.get` detail method (the row carries what
the roster needs and `run.get` already resolves the rest), a live turn counter
(the kernel has nothing to report), and a bulk-cancel affordance.

**Gates**: `cargo fmt --all -- --check` clean; `devbox run -- just check` all
green (fmt, clippy, cargo-deny advisories/bans/licenses/sources, gitleaks — 291
commits scanned, no leaks); `cargo test --workspace --all-features --locked` —
exit 0, **102 suites, 776 passed, 0 failed, 4 ignored** (was 762 passed at Task
10). The four ignored are unchanged and pre-existing: `self_invoke_noop_pass`
and `self_invoke_noop_fail` (self-invocation fixtures in
`sea-forge-case-runner`) and `t16_1_real_acp_host_release_gate` /
`t16_6_real_swe_seed_release_gate` (release gates needing operator-supplied real
ACP / SWE_SEED hosts). No flakes observed. The Tauri host crate is outside the
workspace and was run separately in `src-tauri`: 4 lib + 4 integration tests
pass. Frontend `bun run check` clean apart from the pre-existing `router.tsx`
fast-refresh warning; `bun run --cwd apps/desktop test` — 116 passed / 18 files
(was 98 / 16).

**Tests added**: 12 in `crates/sea-forge-server/tests/conformance_delegations.rs`
(empty roster on a fresh cell; **no settlement + no handle → `unresolved`**;
standing and settlement as separate facts; dialogue reported against its bounds;
a cancelled episode settling with `cancelled` as its *termination*; non-agent
runs excluded; case attribution via the shared index; absent token budget
omitted not zeroed; cancelling an inactive delegation refused, agreeing with
`cancellable: false`; unsettled sorting ahead of settled; advertised as
`inspect`; **a full tree diff proving the read writes nothing**); 2 unit tests in
`sfwp/delegations.rs`; 1 in the host bridge (roster verb + cancel command
asserted together, since they are two halves of one control loop); 15 in
`pages/DelegationRoster.test.tsx`; 3 in the new `hooks/eventKinds.test.ts`.

**Next spendable slice**: `9.8` (preserve evidence from every termination) —
the roster now reaches every terminated delegation, `TranscriptEvidence` already
records termination, transcript hash, and harvested refs, and `run.get` resolves
the rest; the gap is a termination-complete evidence view that proves failure
cannot erase the record. The alternative remains `thoth.ask`'s response contract
(journey 3): still the largest unclaimed block, still at least two slices wide.

---

# Casework cognitive environment: product-experience implementation (2026-09-19)

Section owner: the godspeed-casework-cognitive-environment plan (this block is additive; the
bounded-judgment handoff above is untouched).

**What happened**: operator-ordered takeover run that carried the T03 interaction core to the
full product experience. Built: the R3F spatial world (raymarched gravitational-lens Core,
orbital attention physics, semantic zoom via wheel/double-click, contextual relationship lines,
comparison/time-comparison arrangements), the bounded artifact runtime with six lazy renderer
families and boundary-crossing persistence intent, the bottom-center composer with narrated
choreography (pause/resume/interrupt, no transcript), the temporal strip, search palette,
outline (list alternative), a11y announcer, dark mode; the Go boundary server (-serve mode:
projections, SSE with replay, intents with idempotency + fixture authority + leases, artifacts)
embedding the canonical Northstar dataset; operator recipes casework-go-up/-down/-status and
casework-demo-up.

**Gates this run**: GATE_UI PASS (72 tests / 471 assertions, suite run 5x clean), GATE_GO PASS,
GATE_SPEC_TRACE PASS, production build + preview verified, live walkthrough of the reference FDE
journey captured (22 screenshots + report at
`.agents/evidence/godspeed-casework-cognitive-environment/product-experience/`), two full
consequential crossings observed over SSE with the world settling quiet.

**Status of plan tasks**: T04/T06/T07/T08/T09/T10 substance implemented (see implementation
report); NOT settled — the plan's preregistration/teeth/confirmation process was not executed for
them in this run. T05/T11 real SEA-Forge/Gauntlet integration, T12 journey settlement, T13/T14
confirmation and removal remain open. No fixture behavior is claimed as governed integration.

**Game-first reframe (2026-09-20)**: operator reframe applied as a presentation pass: first frame
is Core + composer + white space (peripheral chrome fades, temporal strip only in history mode,
Core whisper "1 thing needs you" inside the event horizon), pointer-driven wake physics (rAF +
`--wake` custom property, no React state), click = camera travel with shift-click select and
double-click artifact open, imperceptible orbit from authored angles, resting relationship
hairlines only where attention is. Verified live (evidence screenshots 24-31); core untouched,
72/72 tests green, typecheck clean. Decisions D-2026-09-19-PE-01..05 and D-2026-09-20-PE-06.

> **2026-09-23 (later) PLAN SWITCH — casework-live-wiring-production (T00 settled):**
> The active plan is now `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml`
> (v0.2.1); machine state lives in `.agents/current_status.yml` (this file remains the narrative).
> T00 settled: live-stack justfile recipes (`casework-cell-init`, `casework-server-up/down`,
> `casework-live-go-up/down`, `casework-stack-down`) + `apps/godspeed-casework-go/configs/live-serve.json`;
> correction C-1 re-planned the gateway recipe name (fixture `casework-go-up` pre-existed);
> baseline captured (cargo/Go/bun GREEN; local ladder deterministically RED at J1 —
> OBSERVED_DEBT, T09 owns the fix; the 11/11 evidence above predates the spatial rework in
> 44ffadd and is stale). Decision log + evidence:
> `.agents/reports/casework-live-wiring/decision-log.yaml`,
> `.agents/evidence/casework-live-wiring/T00/`.

> **2026-09-23 (latest) T01 SETTLED — canonical wire contract:** spec-04 `types.ts` extended
> additively (10 journey intent kinds, 7 typed refusals, payloads, template/preflight DTOs, SSE
> union), 15 golden fixtures under `interface-contracts/golden/`, Go `internal/contract` mirror
> with byte-stable round-trip + exhaustive-kind tests, TS `wireContract` pins, kebab dialect
> `apps/godspeed-cognitive-ui/contracts/` deleted per D-1. Independent confirmation:
> `.agents/evidence/casework-live-wiring/T01/confirmation.md`. New OBSERVED_DEBT: pre-existing
> Go coordinator idempotency flake (~1/5). T04 in flight (unit A: case-ops extraction).

> **2026-09-24 T03 SETTLED — real templates + seeded E2E cell (GAP-D):** fixtures/cells/e2e/
> (sentry-chain + signoff-gate templates, operator policy), casework-cell-init now seeds identity +
> templates + policy (idempotent, hash-proven), case_templates_live 7/7 over the real socket;
> independently confirmed with re-run gates and independently authored cyclic teeth
> (`.agents/evidence/casework-live-wiring/T03/confirmation.md`). Plan-fact note: the SFWP commit
> path emits CaseCreated + case_plan ledger record (never PlanCreated — CLI-pipeline-only); tests
> assert the real truth. T04 unit B implemented, awaiting orchestrator re-verification + independent
> confirmation; unit C (supervisor) next. Two usage-limit windows killed subagents today; inline
> verification continued where honest.

> **2026-09-24 SESSION 1 HANDOFF — read `.agents/current_status.yml` for machine state.**
> Settled with independent confirmation: T00, T01, T03. T04: unit A confirmed (1b73724), unit B
> committed cb28329 (7 SFWP verbs + event publishing; builder died on a usage limit; orchestrator
> re-verified gates/diff/tests; INDEPENDENT CONFIRMATION PENDING — required before T04 settles).
> Unit C (supervisor) not started; T02 preregistered (prereg/T02.yaml) and ready. Branch
> casework/live-wiring, tree clean. Usage limits killed subagents twice today; delegation caveat
> and the full resume recipe are in current_status.yml next_action + agent_protocol.
>
> **2026-09-24 SESSION 2 — T04 SETTLED (unit C: opt-in supervisor, D-3 step 5).**
> Unit C: `SupervisorConfig` (fail-closed `enabled:false`, 1..=3600s poll, 1..=8 cases,
> actor-id validation) + `supervisor.rs` poll loop (slot + shared run-pool permits, per-case
> key locks, missing-policy skip) + `AdvanceCaller::{Verb,Supervisor}` sharing one
> `run_case_mutation` choke point + `AdvanceScope::Supervisor` (SandboxedTask-only, human work
> reads `idle` with zero writes) + 4 `sfwp_supervisor` tests (TDD baseline 1 expected failure →
> 4/4). Fresh re-runs: 3-crate 57 ok suites/0 fail, mutations 10/10, supervisor 4/4, config 15,
> fmt + no-async-kernel EXIT 0. Independent critic t04-critic ran (team run_00001, report via
> mailbox msg_00002+msg_00003): procedural REJECT only (C uncommitted at review, no T04/
> confirmation.md, its live cargo runs hit a 30s env timeout) with ZERO code findings — grounds
> closed in `.agents/evidence/casework-live-wiring/T04/confirmation.md`, which settles T04.
> Unit C committed; T02 in flight (prereg written, ADR-first). Machine state: `current_status.yml`
> (settled [T00,T01,T03,T04], in-flight T02).

> **2026-09-24 (session 3) T02 builder-complete (confirmation pending):** gateway-principal
> delegation implemented (ADR, fail-closed config, raw-line on_behalf_of per ADR-003,
> dual-principal audit record written before handlers, SoD on end users, identity.get effective
> actor); prereg teeth all permanent tests (9/9); gates green incl. three-crate, mutations,
> supervisor; workbench IdentityView contracts regenerated. Independent critic next. ALSO:
> session-2's T04 settlement rests on a builder-written reconciliation of a critic REJECT (the
> critic's report is not in the repo) — session 3 is re-running the independent T04 verification
> (T04/confirmation-independent.md) before treating T04 as confirmed-settled.

> **2026-09-24 (session 3) T02 SETTLED — delegated actor identity (GAP-B):** independent critic
> APPROVE (T02/confirmation.md): prereg falsifier never realized; four mandated refusals
> reproduced as fresh-cell runtime attacks with hash-proven zero writes; dual-principal ledger +
> effective-actor identity.get verified; wire additivity intact. Critic findings handled: ADR
> revocation-timing corrected (commit/restart boundary, F-7), OPEN_QUESTIONS ActorRole::Gateway
> entry (F-8), OBSERVED_DEBT partial-case-dir entry (F-9, T11). T04 also independently confirmed
> this session (confirmation-independent.md). T05 (Go SFWP client) in flight.

> **2026-09-25 T05 SETTLED — Go SFWP client (SEAM-4) + kernel fixes:** independent critic APPROVE
> (T05/critic/confirmation.md): live suite 14 PASS / 0 skip(except env-gated capture) / 0 races;
> kill-mid-commit recovery proven from a kept cell (exactly one CaseCreated); kernel fixes 514fc72
> verified (AUTH-01 ordering, artifact digest re-derived independently, write_only basis, snake_case
> events). T06 (projection + intents) in flight; T07 sequential after it (shared files).

> **2026-09-25 T06 SETTLED — live projection + intents (SEAM-1/2/3/6/7), via a full fix loop:**
> round-1 critic REJECT (F1 empty-actor 403, F2 discretionary-add translation gap) -> fixes
> (b5fd2c3, incl. kernel F7: pending commits bound to a case locator before any ledger write +
> startup reconciliation) -> round-2 critic APPROVE with its own end-to-end drives (perspective
> 200/403, discretionary accept, L5 approval journey, recovery branch live). Live gate posture:
> -p 1 (documented). T07 in flight.

> **2026-09-26 T07+T08 built (confirmation pending — platform outage):** T07 auth/sessions/CSRF/
> static/production posture (eef7459) verified inline by the orchestrator (gates + teeth with the
> real binary; the CSRF cookie, session binding, and production-refusal paths proven live). T08
> HttpCaseworkAdapter (ce538bf) implemented inline: live conformance 15/15 against the real stack
> (login → templates → preflight → PROPOSE_CASE → standing → EXECUTE → SSE → durable ledger →
> stale tooth → second user); prod bundle excludes the local adapter (bundle check). Two gateway
> defects found and fixed: logging middleware hid http.Flusher (SSE broke for logged clients);
> PROPOSE_CASE response now names the new case. Subagent platform killed 6 agents (captcha
> timeouts); independent critics for T07/T08 (+T09/T10 when built) run when it recovers. T10
> preregistration written before any ladder work (prereg/T10.yaml).

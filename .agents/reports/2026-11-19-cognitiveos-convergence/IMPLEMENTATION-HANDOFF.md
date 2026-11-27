# CognitiveOS convergence — implementation handoff

Durable checkpoint for the `/implement-spec` work on parent issue
[#108](https://github.com/GodSpeedAI/SEA-Forge/issues/108) and tickets
[#109–#122](https://github.com/GodSpeedAI/SEA-Forge/issues/109). Git commits, the
issue tracker and executable verification override every claim in this file.

## Session log

| Recorded | Session | Last safe action | Next action |
|---|---|---|---|
| 2026-11-20 | first implementation session (DeepSeek V4.1 Flash / OpenCode) | #109 landed on the integration branches; #110 (`experts.restore`) and #111 (`affordance.field`) integrated and green; #117, #112 (Cognate), #119 in flight | Integrate #117, #112 and #119; prepare the Cognate release plan and present it before any protected-branch merge or publication |
| 2026-11-20 | owner decision on the Cognate publication gate | Owner approved using the existing Release Please + GitHub Actions OIDC trusted-publishing path (no long-lived token; `NPM_TRUSTED_PUBLISHING_READY=true` is set), forbade monorepo imports/vendored tarballs/workspace aliases/manual registry edits, and required a **focused, tested release candidate with its plan shown before any main merge or publish** | Finish and verify the Cognate half of #112, then present exact versions, release scope, approval and verification |

## Parent and specification

- Parent: [#108 — CognitiveOS convergence specification](https://github.com/GodSpeedAI/SEA-Forge/issues/108)
- Published tickets: [#109](https://github.com/GodSpeedAI/SEA-Forge/issues/109) · [#110](https://github.com/GodSpeedAI/SEA-Forge/issues/110) · [#111](https://github.com/GodSpeedAI/SEA-Forge/issues/111) · [#112](https://github.com/GodSpeedAI/SEA-Forge/issues/112) · [#113](https://github.com/GodSpeedAI/SEA-Forge/issues/113) · [#114](https://github.com/GodSpeedAI/SEA-Forge/issues/114) · [#115](https://github.com/GodSpeedAI/SEA-Forge/issues/115) · [#116](https://github.com/GodSpeedAI/SEA-Forge/issues/116) · [#117](https://github.com/GodSpeedAI/SEA-Forge/issues/117) · [#118](https://github.com/GodSpeedAI/SEA-Forge/issues/118) · [#119](https://github.com/GodSpeedAI/SEA-Forge/issues/119) · [#120](https://github.com/GodSpeedAI/SEA-Forge/issues/120) · [#121](https://github.com/GodSpeedAI/SEA-Forge/issues/121) · [#122](https://github.com/GodSpeedAI/SEA-Forge/issues/122)
- Design package: [`A`](A-actual-state.md) · [`B`](B-gap-and-convergence-ledger.md) · [`C`](C-target-architecture.md) · [`D`](D-minimal-contracts.md) · [`E`](E-acceptance-journeys.md) · [`F`](F-implementation-sequence.md) · [`G`](G-residual-uncertainty.md) · [`H`](H-transformation-brief.md) · [`I`](I-closure-register.md) · [`DECISION-LOG`](DECISION-LOG.md)

## Repositories, bases and integration branches

One integration branch per repository. No branch may be merged into a protected
`main` without normal approval.

| Repository | Path | Base | Integration branch | Head |
|---|---|---|---|---|
| SEA-Forge | `~/projects/sea-rs` | `fde/integration` `a5060197` | `integration/cognitiveos-108` | see `git log -1` |
| Cognate | `~/projects/cognate` | `main` `6c42025` (0.10.0 line) | `integration/cognitiveos-108` | `6c42025` |
| Cognate Workbench | `~/projects/cognate-workbench` | `feat/capability-runtime-t4a` `9824735` + `feat/wp23-assignment-binding` `668f686` | `integration/cognitiveos-108` | `1e3bfeb` |
| Horizon | `~/projects/Horizon` | `origin/main` `0e34a34` (0.10.0 line) | `integration/cognitiveos-108` | `18b27d1` |

Worktrees live under `~/projects/_wt/cog-os/`: `sea-int`, `cognate-int`,
`cwb-int`, `horizon-int`. Ticket worktrees are created per implementer and
removed after integration.

### Workbench integration note

The integration head is `feat/capability-runtime-t4a` (which already contains
T3 `efdcc1c` and T4a/T4b) with `feat/wp23-assignment-binding` merged in and
the domain-vocabulary commit. The WP-23 branch predates T3, so the merge
initially dropped T4a-only dependencies; `@modelcontextprotocol/sdk` and
`fast-check` were restored and `bun install` regenerated the lockfile. Every
`@godspeedai/cognate-*` pin is now `0.10.0` and every installed copy resolves
to `0.10.0`.

## Ticket states

States: NOT_STARTED · IN_PROGRESS · IMPLEMENTED_UNVERIFIED · VERIFIED_UNMERGED ·
LANDED · BLOCKED · DEFERRED_BY_APPROVED_SCOPE.

| Ticket | Increment | State | Evidence / blocker |
|---|---|---|---|
| #109 | I0 version alignment + standalone | LANDED (integration) | Workbench `test/cognate-version-gate.test.ts`; 107 + 76 tests; Horizon 28-package gate + 31 workshop/handoff tests. See [#109 comment](https://github.com/GodSpeedAI/SEA-Forge/issues/109#issuecomment-7049556408). |
| #110 | I5 `experts.restore` | LANDED (integration `ae43820`) | Operator-only, in-run refused, verbatim create/reactivate/kept; `test/J-EXPERTS-restore.test.ts` (5 tests) + adjacent suites (13 pass). Merged as Workbench integration `b3a690b`-series. |
| #111 | I5 `affordance.field` | LANDED (integration) | Bounded read over the existing resolver and `CASE_OPERATIONS`; evaluated-move-without-criterion refused; `test/J-COG-AFFORDANCE-field.test.ts` (36 pass with cognitive suites). |
| #112 | I1/I2 deterministic child run | COGNATE HALF LANDED (`63c3e1b` on Cognate `integration/cognitiveos-108`); Workbench half BLOCKED on publication | `packages/runtime-api/src/rlm.ts` (public contract, `RlmProvider`, `StdioSpawnPort`, refusal codes) + `packages/runtime-bun/src/rlm.ts` component + `agent.rlm` + test provider. 168 focused tests pass, typecheck clean. Deviation recorded below. Release candidate `release/cognate-rlm-0.11.0` = `082ea12` on `origin/main`, verification running. |
| #113 | recovery/reconciliation | NOT_STARTED | Depends on #112. |
| #114 | I3b contained streaming | NOT_STARTED | Depends on #112 contract (StdioSpawnPort in runtime-api) and the T4a spawn base. |
| #115 | egress + cumulative payment | NOT_STARTED | Depends on #114. |
| #116 | committed allow ↔ enforced profile | NOT_STARTED (SEA contract note recorded) | Depends on #114. SEA investigation: `sea-forge-core::AuthorityDecision` already carries `sandbox_class_granted: Option<String>`, `boundary_constraints: BTreeMap<String, Vec<String>>` and `compensating_controls`, and `sea-forge-authority` policy rules carry `gate_profile_ref` + free-form `parameters`. The contract can therefore bind one approved execution profile without a new persisted schema; the work is (a) a SEA policy rule that authorizes the exact profile and (b) the host (Workbench) comparing the enforced launch plan/profile to the committed allow before spawn. If either cannot be established, keep admission closed and report the exact gap. |
| #117 | provenance projection | IN_PROGRESS | `ticket/117-settlement-provenance`: `settlementScope`/`receiptRef` + governed settlement family in `OBSERVED_EVENT_TYPES`. |
| #118 | 7B joined proof | NOT_STARTED | Depends on #112–#117. Do not claim until authority, contained execution, cumulative payment, independent evidence and the SEA-Forge receipt are demonstrated together. |
| #119 | Horizon exploratory handoff | LANDED (integration) | Workbench `cfea9d3` (`CaseContext` validated/frozen/forwarded into `startRun`; `assertedBy` recorded); Horizon `de33473` (`deriveCaseHandoff`, `/desk/handoff` forwards context from the boot-verified world). Workbench 5 new tests, Horizon 8 new + 43 combined pass. Not satisfied: no case↔run read projection, no Cognate `caseAssignmentId` (needs evaluated WP-23), evaluated production issuer/verifier path not exercised. |
| #120 | Horizon evaluated WP-23 binding | NOT_STARTED | Depends on #119. |
| #121 | optional Prime adapter | NOT_STARTED | Depends on #114. Optional. |
| #122 | optional Gauntlet escalation | NOT_STARTED | Frontier, optional, no core dependency. |

## Cognate release (owner-approved, executing)

Focused candidate: `release/cognate-rlm-0.11.0` = `082ea12` (cherry-pick of the verified
`63c3e1b` onto `origin/main`). Merged as Cognate PR #89 → `main` `97603d3`.
Release Please version PR #90 (`0.11.0` across the 45-package linked group, feature attributed
to `runtime-bun`/`runtime-api`) merged as `main` `effcf05e`; the `release` workflow is
publishing via OIDC trusted publishing.

| Item | Value |
|---|---|
| Release scope | functional change only in `@godspeedai/cognate-runtime-api` + `@godspeedai/cognate-runtime-bun`; everything else version-only |
| Version | **0.11.0** across the 45-package linked group (Release Please computed) |
| Publication | `release.yml` OIDC trusted publishing; `NPM_TRUSTED_PUBLISHING_READY=true`; `release` environment restricted to `main`; no long-lived npm token |
| Verification | PR #89 CI all green (verify ubuntu incl. PostgreSQL-backed suites, pack/clean-room, CodeQL, secret scans); local 168 focused tests + typecheck + `just check` |
| GitHub-side note | Bot-authored release PRs do not trigger CodeQL, so the `Protect Main` ruleset's `code_scanning` rule blocked the merge until CodeQL was run on the release branch via `workflow_dispatch` (`44817020889`). No credential change and no admin bypass was used. If this recurs, the durable fix is to author release PRs with the existing `RELEASE_PLEASE_TOKEN` so CI runs automatically. |

## Recorded deviations (#112 Cognate half)

- **Parent wait shape.** F-described the parent waiting on `rlm-invocation:<id>` resolved by the RLM
  component. The implementation reuses the existing reconciliation primitive (`ambiguous-invocation`
  on the child) and treats `rlm.invoke` as fire-and-forget addressed by `(parentRunId, invocationId)`.
  It satisfies ticket #112's acceptance and D4, but an integrator should confirm the Workbench Expert
  branch waits for the child before composing its handoff.
- **`rlm.inspect`/`rlm.cancel` input** carries an optional `parentRunId` (defaults to `ctx.runId`) so
  out-of-run callers can be checked against the parent run, per the ticket's parent-scoped standing
  clause. D1 showed `{ invocationId }` only; the extension is additive and optional.
- **Not raised in I1:** `rlm-context-digest-mismatch` (no context resolver yet) and the D8 port codes
  `rlm-containment-unavailable` / `rlm-egress-uncontrolled` / `rlm-port-unavailable` (I3b). The
  vocabulary exists; the refusal paths land with #114.

## Completed verification (exact commands)

On `cwb-int` at `1e3bfeb` with `0.10.0` installed:

- `bun run typecheck` — pass
- `bun test --timeout 15000` over the expert/team/review/plugin suites listed in
  the session log — 107 pass, 0 fail, 1053 assertions
- `bun test --timeout 15000 test/J-B-basic-session.test.ts test/J-C-tool-execution.test.ts test/workbench-contract.test.ts test/workbench-glob.test.ts test/skill-bundle.test.ts` — 76 pass, 0 fail
- SXR absent from `PATH`, so the standalone runs exercised the no-optional-product posture

On `horizon-int` at `18b27d1`:

- `bun run scripts/check-single-cognate.ts` — ok, 28 packages, one copy each
- `bun run check` — ok

Failed or not run:

- Full 246-file Workbench suite, Playwright/browser journeys and Postgres-backed suites were not run in this session.
- Horizon full `verify` (pytest, ledger validators) was not run.
- No cross-process proof, containment proof or settlement proof has been run yet.

## Known blockers and limitations

1. **Cognate publication (owner-approved path, awaiting a focused release plan).**
   Workbench consumes published `@godspeedai/cognate-*` packages and may not import
   monorepo source, vendored tarballs or workspace aliases. The Cognate half of #112
   is implemented and verified locally first; the work then reaches the registry
   through the existing `release.yml` Release Please + OIDC trusted-publishing path
   (`NPM_TRUSTED_PUBLISHING_READY=true`, `release` environment, `id-token: write`,
   no long-lived npm token). Publication requires merging the focused release
   candidate into `main`, which is a protected-branch merge: show the exact
   versions, release scope, approval and passing verification to the owner first.
2. **Host budget.** 8 GB WSL host. Serialize Cargo; use at most two jobs.
3. **pg0.** `cwb-int/.tmp/pg0` is a symlink to the main checkout's pg0. Starts
   use the default instance name; avoid concurrent DB-heavy suites.
4. **No integration branch is pushed yet.** All commits are local-only.

## Next eligible work

1. Integrate #117, the Cognate #112 half and #119 once their tests pass on the
   merged candidate, recomputing the frontier each time.
2. Verify the Cognate release candidate independently, then present the focused
   release plan (exact versions, scope, approval, verification) before any
   protected-branch merge or publication.
3. After publication, update Workbench to the new coherent version, clean-install
   and rerun the package compatibility gate and regressions; then unblock the
   Workbench consumer half of #112.
4. Implement #114, then #115/#116, then #118; keep #122 optional and #121/#120
   gated on their prerequisites.


## Continuation checkpoint — 2026-11-20

- Cognate 0.11.0 release workflow run 44817720906 succeeded, including all package publishing jobs and public npm receipt verification. Do not repeat publication.
- Workbench integration `8a18064` includes the #117 historical replay fix: preserve absent provenance rather than classify a legacy record as local. The focused regression and adjacent settlement suites passed (15 tests); typecheck passed. #117 remains open: real SEA receipt and RealityTrace ingestion are unproved, and the observation interface lacks a settlement-packet projection path. The orchestrator discards the receipt issuer envelope.
- At prior Workbench head `eec0f4a`, version gate (4 tests), typecheck, five focused ticket suites (23 tests), and standalone basic-session/tool/contract/skill suites (76 tests) passed. These are not full acceptance evidence. #119 accepts a caller's verified world claim without registry corroboration; evaluated admission with an episode but without a trusted WP-23 binding still needs proof. #122 uses a mock runner and has no demonstrated real Gauntlet settlement.
- Horizon integration `de33473` has uncommitted `package.json` and `bun.lock` changes, including the two overrides corrected to 0.11.0. Empty-cache frozen install and single-copy gate passed; actual archive typecheck and 222 focused tests passed. `just verify` exited 1 because its default DomainForge binary path was absent: 1,717 Bun tests passed (16 skipped), 150 pytest tests passed, and the two remaining teeth passed separately with a verified cached DomainForge 0.22.0 binary. No single full passing invocation is claimed. The identity archive declares Cognate ^0.9.2 while overrides select 0.11.0; lock metadata still reports ^0.8.0. Resolve compatibility and reproducibility before claiming #109 complete.
- Native tracker dependencies: #112 and #119 depend on #109; #113 and #114 on #112; #115 and #116 on #114; #118 on #113, #115, #116, #117; #120 on #119; #121 on #115. All issues remain open. Preserve the dirty handoff and Horizon files and existing worktrees.

### Continued qualification

- Horizon's lock metadata now matches the identity archive (^0.9.2), generated by Bun remove/add of the same archive in isolation; no unrelated package versions changed. Fresh empty-cache frozen installation passes. The archive's declared range excludes the overridden 0.11.0; exercised identity paths pass but support outside its declared range remains unestablished. Full `DOMAINFORGE_BIN=<verified 0.22.0> just verify` failed with one 5-second connector-reaudit B2 timeout (1,716 pass, 16 skip); the same test passed alone, and `just verify-static` passed including 150 pytest tests.
- At Workbench `8a18064`, isolated empty-cache frozen install resolved all 23 direct and 35 recursively installed Cognate packages to 0.11.0. Typecheck, version gate (4), smoke (73), WP23 (4), cognitive focus (6), and isolated PostgreSQL handoff (5) passed. Full verify stalled in repeated typecheck under host memory pressure and was stopped; it is not a passing full gate.
- #117: CEP settlement_packet already carries the needed fields, but Cognate observationIntent has no settlement-packet transport; its trace field is specifically an execution_trace. Workbench discards the returned settlement envelope. The user was asked for approval to extend the public observation port; do not assume approval. Receipt validation must also refuse missing ledger entry and finality disagreement.
- #122: safe preflight of Workbench's generated reference using the existing Gauntlet 0.1.0 binary failed reference validation because it contains no AK-NNN block. Workbench lock uses bare hex while Gauntlet compares a sha256:-prefixed digest. No real controlled settlement is proven; determine whether faithful typed projection and runner evidence fit the stated one-configuration-plus-reference-projection guard before further integration.

- Follow-up: serialized Horizon `DOMAINFORGE_BIN=<verified 0.22.0> just verify` passed end to end: 1,717 Bun tests passed, 16 skipped; 150 Python tests and ledger, Wayfinder and teeth passed. Log: `/tmp/horizon-109-verify-rerun.log`. This supersedes the earlier failed-run limitation, not the identity archive semver compatibility limitation. The Horizon package files remain uncommitted.

- Workbench full-gate retry at `8a18064` was stopped at typecheck before tests as available RAM fell from 4.3 to 1.5 GiB and swap rose from 8.1 to 9.6 GiB. The temporary PostgreSQL instance was stopped; earlier standalone typecheck and focused suites remain valid. Log: `/tmp/cwb-109-verify-retry.log`. Rerun the full gate when host capacity permits.

### Receipt and optional escalation continuation

- Owner approved the Cognate observation port extension, but raw SEA settlement packets contain evidence and questions without an enforceable non-sensitive guarantee. Unsafe packet persistence and transport remain closed. A source-owned non-sensitive receipt representation with full packet retained in SEA ledger was proposed to the owner; approval is pending. The prior transport drafts remain uncommitted in `/tmp/cognate-receipt-117` and `/tmp/cwb-receipt-117`; do not integrate them.
- Cognate integration `fafc7a3` contains independently reviewed receipt validation: envelope identity and ledger presence, response/packet finality agreement, settled/rejected final, and either finality for unsettled to preserve SEA obligation behavior. Four focused synthetic tests passed; real SEA and full CEP schema validation remain unproven. No Cognate release or push.
- Owner delegated #122 judgment and supplied no case-specific frozen reference. Workbench integration `52ee3fd` drops the optional unqualified Gauntlet integration under the agreed scope guard, preserves historical event identifiers, and passes typecheck, 73 smoke tests and a standalone Case runtime test. Full integrated verify remains pending.
- Owner approved replacing Horizon's existing private identity archive with the reproducible 0.11.0-compatible candidate. Horizon integration has uncommitted `package.json`, `bun.lock` and identity archive changes; archive SHA-256 b245ea22c1eaec83be2a4f0b8097c5ea558ef0395a0d2c364ea2bf471e61e282. Empty-cache frozen install, 28-package single-copy gate and 26 identity tests pass. Full verification has failed under host temp-space/time pressure; nine failures in disk-backed retry remain under diagnosis. No commit or push.

### Receipt continuation and interrupted verification

- The owner approved an allowlisted non-sensitive receipt, retention of full packet/evidence within SEA, and a distinct fail-closed SEA response verb. The uncommitted SEA draft remains at `/tmp/sea-receipt-117` (spec, server, tests and ADR); the uncommitted Cognate consumer draft remains at `/tmp/cognate-receipt-validation-117`. Neither is qualified or integrated.
- Synthetic tests reproduced unauthorised service replay and a forged SEA socket response. Service replay fix `805b644d` remains isolated in `/tmp/sea-117-replay-access`; server same-ID correlation and `request.get_status` owner authorization remain unresolved. The owner approved adding authenticated owner binding for new correlation records and fail-closed recovery for legacy records without verifiable owner.
- The client does not authenticate the configured SEA socket peer; a proposed fail-closed host identity binding awaits explicit owner decision. UID alone does not prove executable identity and a same-UID fake remains possible.
- The draft may report `no_side_effect:true` when projection fails after settlement has committed; do not integrate until corrected and tested. Replay and nonfinal receipt projections are intentionally refused in the current draft pending an integrity-safe winner scan, so acceptance remains incomplete.
- A receipt worker was stopped by an automatic cybersecurity-risk safeguard while performing authorized security work. Its focused Cargo command was already running when the worker stopped; no test result was observed. Preserve the draft and do not claim the test passed.
- Required independent hardening review of `805b644d` is incomplete. Automatic approval review rejected escalation for the external Codex reviewer because it would transmit the internal diff and packet without specific authorization. Review packet is in the primary checkout `.tmp/sea-117-review/packet`; do not claim a clean review.
- Horizon approved identity archive, package and lock changes remain uncommitted. Earlier full log `/tmp/horizon-identity-011-candidate/full-verify-short.log` ends `verify: ok` (1717 Bun pass, 16 skip, 150 pytest pass) but lacks captured shell exit status. Later rerun `/tmp/horizon-identity-011-candidate/full-verify-resumed.log` was interrupted after eight host-load timeouts (exit 130); no fresh full pass is claimed.

- Owner-binding investigation: the approved owner field alone cannot be trusted as an admission binding. A concrete plan at `/tmp/sea-117-owner-binding-plan.md` uses an existing ledger append for a source-owned first-admission claim under the correlation lock. Ledger and correlation writes are separate; an orphaned claim after a crash must remain ambiguous and prevent re-execution. Approval of that additional durable contract and recovery rule is pending. The `same_actor_or_auditor` spec has no implemented auditor grant; a default-deny, exact-request-scoped `request_status_audit` grant was proposed and awaits approval. No binding or audit code was changed.

### Recovery checkpoint — 2026-11-24

Earlier pending-approval statements are superseded: the owner approved the narrow durable owner binding, fail-closed orphan recovery and default-deny exact-request audit grant, and chose a conservative fail-closed host trust design. They declined external disclosure of the hardening review packet.

Temporary worktrees and logs under /tmp have disappeared. SEA branch `ticket/117-replay-access` still contains commits `805b644d` and `46bae538`; the later uncommitted owner-binding draft is not present. Do not describe it as implemented. The hardening packet is retained in the primary checkout `.tmp/sea-117-review/packet`.

Disk-backed unmerged checkpoints: SEA receipt `12f1f28c`, Cognate fail-closed receipt `9dafdf6`, Horizon package candidate `14be523`, Workbench safe-mode regression `c352c0b`, and Workbench affordance draft `7f1a726`. The last passes `just typecheck`, `git diff --check` and staged Gitleaks; its implementer reported 43 focused unit tests and seven isolated PostgreSQL tests passing, but a trusted eligibility preview and full gate remain outstanding. It is deliberately incomplete and must not be integrated as ticket completion.

No external Codex review was run. Automatic approval review rejected external transmission of the internal diff; independent hardening remains blocked. Receipt endpoint provenance, committed-effect reporting, replay authorization, legacy owner handling and real receipt acceptance remain unresolved. Horizon full gate has no available captured successful exit status. The #110 Team restoration versus active-member invariant conflict remains unresolved.

- Recovery: existing branch `ticket/117-replay-access` is checked out at `46bae538` in the primary checkout `.tmp/cog108-sea-replay`; no branch or commits were recreated.

### Continuation checkpoint — committed effect and eligibility

- SEA receipt checkpoint `e3fe1471` returns `no_side_effect: false` when projection fails after a committed settlement and strengthens the existing regression. Status, rustfmt and staged Gitleaks passed; the behavior test remains unrun. The branch remains unqualified and unmerged.
- Owner binding: the recovered replay worktree has a new uncommitted regression for legacy hashless re-execution. Focused crate-test exited 101 before compilation because sccache returned `Operation not permitted (os error 1)`. No binding or audit implementation is complete.
- #111: Cognate `kernel.resolve` identifies a provider but grants no authorization. Governed invocation enforces policy, SEA authority, constraints and escalation, and does not expose a supported read-only eligibility/payment preview. The existing Workbench checkpoint remains fail-closed; a trustworthy preview requires an upstream runtime and authority contract.

- Horizon checkpoint `14be523` passed full `just verify` with captured exit 0: 1,717 Bun pass, 16 skip, 150 Python pass, ledger, Wayfinder and adversarial teeth pass. Evidence: primary checkout `.tmp/horizon-109-verify-14be523-final.log` and `.exit`. The clean checkpoint remains unmerged; integration and cross-repository acceptance remain required. Earlier timeout and sandbox-confounded runs do not constitute a pass.

- SEA replay checkpoint `65bca530` implements isolated durable admission claims and default-deny exact-request audit access. Legacy hashless test passed after confirmed red; claim tests 2/2, correlation 15/15, audit policy 1/1, SFWP socket conformance 24/24 and case-authoring socket conformance 21/21 passed. Simulated orphan and foreign-owner retry were refused. Actual crash-at-write recovery, independent hardening, full CI/proofs and cross-platform checks remain outstanding. No integration.

- SEA crash-test checkpoint `ab015781` proves Unix process termination at the actual boundary: a test-only pause follows durable claim commit and precedes correlation write; SIGKILL and restart lead to replay/status refusal and no case or run. The strengthened test and status/format/secret hooks passed. Power loss and non-Unix behavior remain unproved. Independent hardening and full gates remain outstanding.

- Workbench integration `52ee3fd` full `just verify` in an isolated copy with disposable PostgreSQL passed typecheck but was stopped after 14 observed failures, predominantly timeouts under high swap; captured exit 130. First Team bounds failure passed alone after memory recovery (1/1, exit 0), which does not qualify the full gate. Evidence is in the primary checkout `.tmp/cwb108-verification-52ee3fd/`. Disposable PostgreSQL was stopped, integration remained clean.

- Subsequent diagnosis of the interrupted Workbench gate found PostgreSQL connection exhaustion (SQLSTATE 53300) under the default 100 connection limit; CI uses 300. Four earlier failures preceded exhaustion and are not explained by it. The worker hit its usage limit during preparation, after building DomainForge 0.22.0; the full gate was not rerun or passed. Prepare pinned RealityTrace 0.4.0 and retry with a fresh CI-sized database, preserving actual exit status.

- Workbench `52ee3fd`: completed isolated `just verify` with PostgreSQL 18 max_connections=300, single-writer declaration, RealityTrace 0.4.0 and DomainForge 0.22.0. Captured exit 1: 1873 pass, 13 skip, 1 fail across 238 files. `J-CFG-provider-test` saw an unexpected `POST /cognate.v1.RuntimeService/StreamEvents` before expected `GET /v1/models`. The entire provider test file passed in isolation (4/4, exit 0), which does not qualify the full gate or prove the cause. PostgreSQL was stopped after each run. Logs and exit files: primary checkout `.tmp/cwb108-verification-52ee3fd/verify-ci.*` and `provider-focused.*`. Investigate suite interaction without weakening the negative assertion.

- Diagnostic follow-up: the browser checkout test that immediately preceded the failing provider test and the provider test file ran together in the same observed order, 5/5 passed with exit 0; the unexpected request did not reproduce. The disposable PostgreSQL instance stopped cleanly. Evidence `.tmp/cwb108-verification-52ee3fd/provider-pair.log` and `.exit`. The cause remains unknown and the full gate is still failed.

- Real local-socket diagnostic reproduced a relevant lifecycle defect: after `bootApp` stops its server, its unclosed reconnecting client can send `StreamEvents` to a new listener on the same port (1/1, exit 0). This is a plausible explanation for the full-suite failure but does not identify the particular sender. The existing provider assertion remains intact. Evidence `.tmp/cwb108-verification-52ee3fd/stream-diagnostic.log` and `.exit`. Scoped helper cleanup and regression are in progress in an isolated worktree.

- The interrupted Workbench test-client cleanup is preserved at isolated checkpoint `16b8950`, with staged diff check and Gitleaks clean. The original in-process regression was red before the fix and green after. The final expanded regression passed in-process but timed out on sidecar (1 pass, 1 fail, exit 1); logs and exit statuses are under `.tmp/cwb108-verification-52ee3fd/client-cleanup-{red,green,final}.*`. This checkpoint is not verified or integrated. The implementer hit a worker usage limit; diagnose the sidecar timeout and rerun focused and full gates when available.

- Follow-up checkpoint `3496c65` fixes the sidecar fixture readiness JSON. Focused port-reuse tests passed 2/2, exit 0, and `just typecheck` passed, exit 0. Full `just verify` is running on that isolated commit, with logs under `.tmp/cwb108-verification-3496c65/`; no full-gate result is claimed. Correction: the focused rerun overwrote the prior `client-cleanup-final` timeout log and exit file; that timeout was observed before overwrite and traced to the fixture readiness mismatch. The prior red/green logs remain.

- Workbench full `just verify` at isolated `3496c65` recorded four failure markers (J-TEAM-EVIDENCE, E2E remote workspaces, E2E mission, J-CFG-provider-delete) and was stopped under host pressure. SIGTERM did not stop Bun; SIGKILL yielded captured exit 137. The gate is incomplete and failed, not a pass. Disposable PostgreSQL stopped cleanly. Evidence `.tmp/cwb108-verification-3496c65/verify.log` and `verify.exit`. Diagnose after host recovery; do not merge based on focused tests.

- Each of the four full-run timeout cases passed separately on isolated `3496c65` with the original assertions and captured exit 0 (Builder/Verifier 11 assertions, remote workspace 23, mission 12, provider deletion 7). This does not establish cause or pass the full gate. Evidence `.tmp/cwb108-team-diagnostic-3496c65/` and `.tmp/cwb108-diagnostic-3496c65-{remote,mission,provider}/`. The interrupted full gate modified 17 tracked screenshots in that worktree; preserve them. Its log grew to 8.1 million lines / 369 MiB; investigate before the next full gate.

- Controlled Bun server/happy-dom global `Response` reproduction generated the same type error and >10,000 lines from one dump. Both UI suites passed alone and under Bun per-file isolation (9/9), but the full-run server/client was not identified and isolation is not the canonical gate. Earlier logs already contained some refused connections. The worker is auditing direct clients and preparing scoped lifecycle fixes; they cannot yet be credited with resolving every retrying endpoint. Details and focused logs `.tmp/cwb108-ui-diagnostic-3496c65/`.

- Isolated Workbench checkpoint `fda371a` closes the harness helper and remote-workspaces clients. Regression reproduced five retries before the fix and passed afterward; cleanup suite 3/3, affected integration suites 11/11 (53 assertions), and `just typecheck` passed. Gitleaks scan of the committed diff passed after commit. Evidence `.tmp/cwb108-harness-cleanup-diagnostic/`, `/tmp/cwb108-fda371a-secret-scan.exit`. Additional direct-client owners remain under audit; do not claim all retrying endpoints resolved. The 17 screenshot changes remain unstaged and preserved.

- Default-enabled Workbench test-client lifecycle audit checkpointed through `3336923`. Exact HEAD typecheck passed and affected suites passed 58 tests/644 assertions across 20 files, exit 0; full diff whitespace and redacted Gitleaks checks passed. Evidence `.tmp/cwb108-harness-cleanup-diagnostic/{final_combined,final-typecheck}.*`. Six opt-in sandbox/live-settlement suites still have client ownership gaps and unavailable real prerequisites; unchanged and unverified. Two terminal close-record `CONNECTION_ENDED` warnings recur and predate these changes. Seventeen screenshot changes remain unstaged and preserved. Canonical full gate starting in a disposable copy; result pending.

- Canonical `just verify` on a disposable copy verified against `3336923` reached 1,433 pass markers, 0 failures and 12 skips before stalling under host memory pressure (PSI full avg10 35.55%, swap rising). SIGTERM yielded captured exit 143; disposable PostgreSQL stopped cleanly. No `Response` object errors appeared in sampled log. The gate is incomplete, not passed; the pressure cause remains unproven. Evidence `.tmp/cwb108-full-verify-3336923-copy/`. Investigate resources or use an appropriately resourced approved host and rerun to completion; preserve the 17 unstaged screenshot changes.

- Terra checkpoint `dfd0a58` repairs client ownership in the six opt-in Workbench test files. Typecheck, whitespace, and staged secret scan passed; the first focused lifecycle run had three fixture/bind startup failures and the 12 opt-in journeys skipped. The 17 tracked screenshot edits remain unstaged. Exact-commit local CI typecheck and smoke passed, and the repository’s four `just test-shard N 4` jobs are running sequentially with fresh disk-backed PostgreSQL 18 fixtures. Evidence `.tmp/cwb108-ci.QgQTyM/`.

- Luna deployment recon confirmed that `run_authenticated` is a library path for authenticating workload callers to SEA. The production daemon uses `run(config)` and Cognate connects directly using Bun’s Unix socket client, which does not verify the SEA peer. No acceptable existing endpoint binding or authorized independent reviewer was found. Recommended owner decision: approve a dedicated SEA service principal and a trusted native client that verifies the connected peer before sending settlement data. Until then, refuse receipt acceptance.

- Initial local CI split at `dfd0a58`: typecheck and smoke (73/73) passed; shard 1 exited 1 (36 pass, 1 skip, 5 fail). The provider request checks passed 4/4 in shard 2. A fixture mistake put temporary workspaces beneath the SEA Git root, invalidating a non-repository negative test; stopped the run after discovering it, wrapper exit 143, both PostgreSQL clusters stopped. Four unrelated browser timeouts remain unclassified. Evidence `.tmp/cwb108-ci.QgQTyM/`.

- Two browser directory journeys matched `projects` in both a basename and a parent path. Terra narrowed selectors to the exact basename in `55d8ca5` and `45b2d8d`; staged scans passed. The focused picker passed option and keyboard assertions, then timed out waiting for workspace projection under load (`.tmp/cwb-selector-check/`). Corrected run on exact `45b2d8d` checks both journeys, checkout and lifecycle regressions before sequential CI shards. Its source tree and fixtures are outside Git; PostgreSQL data and evidence remain on disk in `.tmp/cwb108-ci.pEQQwr/`.

- Corrected `45b2d8d` Workbench copy passed typecheck, smoke (73 tests), and 16 targeted tests covering both folder selectors, non-repository checkout, and client lifecycle. Shard 2 passed 325/325, including provider GET-only and checkout, and shard 3 passed 135 with 9 skips, including memory migration. Shard 4 finished with 1,375 pass, 3 skip, 1 fail (Expert capacity 1 submission deadline), exit 1. Shard 1 remains in progress. A fresh install from the frozen lockfile passed, as did all four package version/CaseContext checks with unchanged lockfile. Evidence `.tmp/cwb108-ci.pEQQwr/` and `.tmp/cwb108-clean-install.vHUaeh/`.

- Initial runner cleanup removed a stale Workbench-generated Git worktree registration under `.tmp/cwb108-ci.QgQTyM/run` after verifying its scratch directory was already absent. Existing `workbench/feature-plain` predates the test and remains at `2f2ef65c`. The corrected verifier checks that its tree and workspace fixtures are outside Git.

- The corrected `45b2d8d` first local CI matrix finished across all 239 files: 1,872 pass, 13 skip, 5 fail, aggregate exit 1. Shards 2 and 3 passed. Shard 1 had four browser failures (config, two shell cases, cockpit), and shard 4 had one Expert submission deadline failure. All five later passed scoped runs; the first isolated cockpit run failed on Chromium `ERR_NETWORK_CHANGED`, and the repeat passed (23 assertions) while a count-only monitor observed two network-state changes. That does not identify a causal event. The shard-4 retry is excluded from verification. Live output showed the original capacity case passing and a different runtime idle timeout, but an edit to its active shell wrapper caused wrapper exit 2 and overwrote its test log. The scratch database stopped. A fresh fixed-wrapper run of the existing published-runtime scheduling file passed 4/4, 33 assertions, exit 0 (cwb108-capacity.PJLWFP). The full gate remains unqualified. Evidence `.tmp/cwb108-ci.pEQQwr/` and `.tmp/cwb108-capacity.{rMsDUQ,NJmBb9,DucNmS,iNiecD,PJLWFP}/`.

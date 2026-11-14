# 02 Horizon recon (read-only, 2026-11-12)

Legend: OBSERVED = seen in code/test/doc this session. INFERRED = deduced. UNVERIFIED = not checked.
Repo identity: remote `https://github.com/GodSpeedAI/Horizon.git`. Local `origin/main` = `822e349` (PR #28 merge). Refs were not fetched, so origin/main may be stale (UNVERIFIED).

## 1. Identity

Ahead/behind = `origin/main` (behind, ahead). Containment = "A is ancestor of B" via `merge-base --is-ancestor`.

| Worktree (path) | Branch | HEAD | Behind/ahead main | Dirty | Purpose | Contained in gaps? |
|---|---|---|---|---|---|---|
| /home/sprime01/projects/Horizon-gaps | horizon-desk-gaps | e5b98b8 | 0 / **150** | clean | Stage 0B containment, desk gaps, A3-UV enrolment. **Highest priority** | (is the superset) |
| /home/sprime01/projects/Horizon | horizon-desk-v2 | 73678b2 | 18 / 0 | yes: staged delete `.cognate/vendor/godspeed-identity-0.1.0-ed6c52a.tgz`; untracked `.agents/reports/2026-11-03…`, `2026-11-04…`, `2026-11-05…` dirs | v2 desk, pre-gaps base | yes |
| /home/sprime01/projects/Horizon-core | stage0b-containment-core | 23db068 | 0 / 3 | yes: 8 files (contracts/identity/v1/operations.yaml, src/app.ts, src/ui/desk/client/main.tsx, 5 tests, untracked enforcement-probe test + support/settle-waiting.ts) | WIP checkpoint "containment core … not validated" | no (SHA); 3 commits are `wip` |
| /home/sprime01/projects/Horizon-merge | horizon-desk-v2-merge | b4e1843 | 1 / 0 | clean | v2 merge notes/verify output | yes (HEAD is behind main) |
| /home/sprime01/projects/Horizon-pr1 | stage0b-containment-pr1 | 43e8327 | 0 / 3 | clean | PR-1 catalog, contract loader, checkers, P2 lifecycle | no (SHA); patch-ids not found in gaps |
| /home/sprime01/projects/Horizon-pr1-narrow | stage0b-containment-pr1-narrow | 7115b20 | 0 / 4 | clean | Narrowed PR-1 (containment + classification) + report | no (SHA); patch-ids not found in gaps |
| /home/sprime01/projects/horizon-connector | feat/github-connector | 5369b49 | 66 / 0 | clean | GitHub connector + Stage 4 desk merge | yes |
| /home/sprime01/projects/horizon-seed | feat/seed | f7b411c | 79 / 0 | clean | seed data, `just` recipes | yes (ancestor of connector) |
| /home/sprime01/projects/gsr-heads/horizon | (detached) | 55f1b42 | 155 / 0 | clean | pinned head = `feat(remote): passkey-paired mobile remote (#12)` | yes |
| ~/.local/state/horizon-approval-test/checkout | (detached) | 3a71fd2 | not checked | not checked | approval test checkout (outside projects) | not checked |

Containment chain: gsr < seed < connector < gaps; merge < gaps; Horizon(desk-v2) < gaps. Only core, pr1, pr1-narrow are not ancestors of gaps. Patch-id check: none of the 10 unique commits on pr1/pr1-narrow/core has a patch-equivalent in gaps. Gaps has its own rewritten containment work (OBSERVED in `.agents/reports/2026-11-13-stage-0b-integration-dependencies.md` section 1), so pr1 and core are probably superseded in content (INFERRED). Do not delete them (no-delete-unmerged rule).

Open PR hints: none checked locally. Gaps status report says "not merged, not pushed, no PR" (dated 11-08, stale; OBSERVED).

## 2. Role and authoritative ownership

Source: `PRODUCT.md` (OBSERVED), `AGENTS.md` (OBSERVED), `README.md` (not read; UNVERIFIED).

- Horizon is the operator's cockpit/desk: "observer-relative navigation environment" over case, semantic objects, affordances, governed execution, evidence, and developmental feedback (`AGENTS.md` head).
- Principle: "The cockpit never decides." SEA-Forge decides consequential commits. RealityTrace observes outcomes. Horizon shows frame, gates, outcome, evidence, and keeps accepted / settled / observed / improved as distinct claims (`PRODUCT.md` Design Principles 1 and 5).
- Owns (OBSERVED in src): desk case UI and case-keeper writes (`src/ui/desk`, `src/app.ts` desk routes); operator panels and answers, with one FIFO mutation queue (gaps-status §2); the agent door (keyed, `/agent/panels`); the mobile remote (`src/remote/*`, passkeys); the device-enrolment orchestration and its local CAS state (`src/remote/enrollment.ts`); the GitHub connector (`connectors/github`); settlement orchestration (`src/settlement`); the navigation/obligation adapters (`src/adapters/navigation.ts`, `settlement.ts`); workspace export (`/workspace/export`).
- Does NOT own (OBSERVED): SEA-Forge decisions (`src/adapters/sea-forge-*`, `decide`/approvals via Cognate `SeaForgeAuthority`); RealityTrace observation (`src/adapters/realitytrace-obligation.ts`, CLI-based); identity-service credential issuance (`replaceDevices` is the designated writer, `contracts/identity/v1/operations.yaml:54`); the cognate runtime (`package.json` deps `@godspeedai/cognate-*` 0.9.0).

## 3. Implemented capabilities

Test counts from the latest reports (OBSERVED in `.agents/reports/2026-11-12-remediation-4-stage-0b-a3uv-enrolment.md`): `bun test` 1628 pass, 0 fail, ~16 skip (env-gated). `tsc` clean. Teeth mutation 46/46. Test files: 181 under `test/` (82 `desk-*`).

| Capability | Evidence | Tests | Status |
|---|---|---|---|
| Desk case UI, answers, panels, one mutation queue | gaps-status §2; `src/ui/desk/`, `src/app.ts` | `test/desk-*.test.ts(x)` (82), `desk-answer-writers-queued.test.ts`, `desk-mutation-queue.test.ts` | IMPLEMENTED |
| Durable operator-scoped records, migrations 1–8 | gaps-status §1 table | `desk-migrations.test.ts`, `desk-operator-scope.test.ts`, `desk-conflict-repair.test.ts` | IMPLEMENTED; migrations 7–8 fixture-only (gaps-status §3d) |
| Single-writer lock (`flock`) | gaps-status §2 | `desk-*`, no dedicated lock test seen | IMPLEMENTED (UNVERIFIED test coverage) |
| P2 remote lifecycle (start/refresh/stop/redeem/sign-in) | `src/remote/server.ts`, `src/remote/sessions.ts`; integration-deps §1 | `test/E2E-remote.test.ts` (9 tests), `test/remote-*.test.ts` | IMPLEMENTED, verified |
| Containment: pending descriptors, confirmation codes, single-use | integration-deps §1 | PE-53..59, NE-41..46 | IMPLEMENTED |
| A3-UV controller proof (passkey, verified vs stored key/sign-count) | `src/remote/enrollment.ts`, `src/remote/webauthn.ts` | NE-50..54, NE-63 | IMPLEMENTED against test double only |
| Designated writer: identity service + store CAS; commit requires current-controller approval | enrollment report §"Owner decision B"; `contracts/identity/v1/operations.yaml:54` | NE-68..72 (`test/remote-enrollment.test.ts`) | IMPLEMENTED locally |
| Production enrolment refused (`authority_unavailable`) | `src/remote/enrollment.ts:97,144` | NE-73, NE-74 | IMPLEMENTED (intended refusal) |
| P3 authority decision from SEA-Forge | `sea_forge.human_entitlement` unpublished | NE-75 uses `test/support/p3-authority-double.ts` (approves everything) | SPEC-ONLY in production |
| Bootstrap of first controller | none; `finish` returns 409 `no_controller` | seeded via `test/support/controller.ts` | SPEC-ONLY |
| Agent containment: grants, gauntlet cancel | critic round 2: reserved, no route | — | SPEC-ONLY |
| SEA-Forge case keeper / events | `src/adapters/sea-forge-wire.ts` (NDJSON over Unix socket), `sea-forge-events.ts` (cursor follower) | `test/sea-forge*`? UNVERIFIED | IMPLEMENTED (UNVERIFIED tests) |
| RealityTrace obligation verify | `src/adapters/realitytrace-obligation.ts` (CLI, run per verifier) | `desk-outcomes.test.ts` (references playwright/chromium text; UNVERIFIED) | PARTIAL/UNVERIFIED |
| GitHub connector | `connectors/github/{index,gh,mapping,board,lock,cursor}.ts`; uses `gh` under operator login | `test/adversarial-github.test.ts`, `connector-*.test.ts` | IMPLEMENTED (on connector branch) |
| Settlement (awaiting closure) | `src/settlement/settle-awaiting-closure.ts` | `settle*`/`obligation` tests UNVERIFIED | PARTIAL |

## 4. Interfaces and contracts

Routes (OBSERVED from grep of `src/app.ts`, `src/remote`):
- Desk: `GET /desk`, `GET /desk/state.json`, `GET /desk/work.json`, `GET /desk/next.json`, `GET /desk/first-move.json`, `GET /desk/remote.json`, `GET /desk/lessons.json`, `GET /desk/search`, `GET /desk/session`, `GET /desk/draft`, `GET /desk/conversation`, `POST /desk/decide`, `POST /desk/panels/op`, `GET /desk/panels/catalogue`, `POST /desk/start`, `POST /desk/handoff`, `POST /desk/handoff/preview`, `POST /desk/note`, `POST /desk/lesson`, `POST /desk/conversation/discard`, `GET /desk/setup-guide`, `POST /desk/pair`, `POST /desk/pair/logout`. Static: `/desk/app.js`, `/desk/app.css`.
- Agent door: `POST /agent/panels` (keyed; `x-horizon-source` header, bearer key).
- Mobile: `GET /m`, `GET /m/pair`, `GET /m/health`, `GET /m/api/state`, `GET /m/api/events`, `POST /m/api/command` (`src/app.ts:3226,3237`), `POST /m/api/auth/options`, `POST /m/api/auth/finish`, `POST /m/api/signout`, `POST /m/api/pair/redeem|finish|pending|authorize|await`.
- Workspace: `POST /workspace/export`.
- Identity catalog: `contracts/identity/v1/operations.yaml` (classes P0–P3, PX; rows `device.pair.request` P2, `device.pair.authorize` P3, `device.pair` P3; approver `operator_with_active_controller`; executor `credential_issuer`). Also `params.yaml`, `records.json`, `policy.yaml`, `a3-mechanism.yaml`, `external-interfaces.json`, `platform-claims.yaml`, `issuer-profile.yaml`, and `fixtures/`.
- Config: `config/horizon.toml`; `src/config/{load,setup,types}.ts`.
- Identifiers: request ids (per command, retry-reused); pending ids; operation digest (`enrollment.ts`); SEA-Forge cursors are ULIDs (sea-forge-events header).
- Data model: durable panel/question/answer records, scoped by operator, versioned migrations 1–8 (gaps-status §1). Store engine: UNVERIFIED (not confirmed as SQLite in this pass).
- Event/projection: SEA-Forge event follower (`src/adapters/sea-forge-events.ts`); desk live state via `/m/api/events`; `src/projections/`, `src/ui/surface2/` (reducer/render/grammar). Details UNVERIFIED.
- Agent authority model: P3 (human approval with digest), designated writer = identity service (`identity_service` vocabulary, operations.yaml:54), A3-UV passkey proof, SEA-Forge P3 decision (unavailable).

## 5. Cross-system integrations

| System | Mechanism (OBSERVED) | Direction | End-to-end evidence |
|---|---|---|---|
| Cognate | npm `@godspeedai/cognate-*` 0.9.0 (runtime-bun, client, events, store-sqlite, realitytrace, domainforge, projections, react); `SeaForgeAuthority` SDK client built in `createApplication`; `scripts/check-single-cognate.ts` | Horizon consumes | `bun test`; `check-single-cognate.test.ts`; `src/runtime/required-cognate.ts` |
| Cognate Workbench | 24 files mention "workbench"; `src/ui/workspace/` is Horizon's own compare/export surface | UNCLEAR | not verified |
| SEA-Forge | (a) SDK governance verbs via `SeaForgeAuthority` (decide, approvals, closure, settle); (b) raw NDJSON over Unix socket `src/adapters/sea-forge-wire.ts` for case keeper, human tasks, ledger reads; (c) event follower `sea-forge-events.ts` with cursor | Horizon -> SEA-Forge (client) | Desk case-keeper flows (`test/desk-*`, `test/adversarial-case-keeper.test.ts`); P3 decision path unavailable (D1 in integration-deps §4) |
| RealityTrace (`sxr`) | CLI per verifier run (`realitytrace-obligation.ts`); economics adapter `realitytrace-economics.ts`; enrolment/settlement observe reads through RealityTrace | Horizon -> RealityTrace | Not verified end to end; `settlement.observe` is P3 because it writes to SEA-Forge ledger |
| DomainForge | `@godspeedai/cognate-domainforge`; `src/adapters/semantic.ts`, `src/domain/operational.ts`, `src/app.ts` | Horizon consumes | `desk-*` / semantic tests UNVERIFIED |
| Context Kernel | 13 files reference it (`src/adapters/context-kernel.ts`) | UNVERIFIED | not checked |
| Gauntlet | 1 file | — | SPEC-ONLY (grant/gauntlet cancel no route) |
| Identity service (Cognate-owned, vendored `@godspeed/identity` tgz) | `replaceDevices` called by enrolment commit; API accepts no authorization evidence (gap D2) | Horizon -> identity | NE-71, NE-68 |
| GitHub | `gh` CLI under operator's login | Horizon -> GitHub | connector tests |

Real end to end: P2 remote mobile control (E2E-remote, Chromium virtual authenticator, 9 tests). Not real end to end: P3 enrolment in production (no SEA-Forge decision, no identity evidence check, no bootstrap). RealityTrace-backed settlement: not verified.

## 6. UX surfaces

Routes/screens (OBSERVED in `src/ui/desk/views/`, `src/ui/desk/blocks/`, `src/ui/workspace/`, `src/ui/surface2/`, `src/remote/`):
- Desk views: `Home.tsx`, `Today.tsx`, `Work.tsx`, `Explore.tsx`, `Decide.tsx`, `Cost.tsx`.
- Desk blocks: `ask.tsx`, `ctx.tsx`, `data.tsx`, `guard.tsx`, `read.tsx`, `shared.tsx`, `viz.tsx`, `index.tsx`, `blocks.css`.
- Desk client: `src/ui/desk/client/main.tsx`, `request-ids.ts`.
- Workspace: `compare.tsx`, `export-button.tsx`, `inspect.ts`, `presenting.ts`, `model.ts`, `compare-model.ts`.
- Surface2 (declarative model): `catalog.ts`, `grammar.ts`, `reducer.ts`, `render.ts`, `panel-blocks.ts`, `project.ts`, `binder.ts`. Sequential UI in `src/ui/sequential/` with `vendor/`.
- Mobile remote: `src/remote/phone.ts` (phone page), `panel.ts` (QR panel), `pairing.ts`, `enrollment.ts`.
- Design tokens: `src/ui/desk/desk.css` (35 custom properties, `prefers-color-scheme` present). Brief in `PRODUCT.md`: "Quiet, dense, legible", WCAG 2.2 AA, light and dark, reduced motion, no gradients/glow, colour never sole signal.
- Interaction grammar (from `PRODUCT.md`): world-first case view; gates shown as request accepted / settled / observed; first-run guidance on production screens; deliberate friction on commit, approve, settle.
- Beats, artifacts, free pan/zoom, graphs, timelines, hybrid DOM/WebGL: NOT FOUND in this pass (UNVERIFIED; the memory file names "GodSpeed UI direction" in another repo).
- Known UI gaps (gaps-status §3b, §3c): no browser UI for route, reassign, withdraw, remove, put/publish; no Reassign control for conflicted answers; BACKLOG/BLOCKED states defined but no UI sets them (grep result in gaps report).

## 7. Unfinished work

Gaps branch (`horizon-desk-gaps`, the owner's active work). Source: `.agents/reports/2026-11-08-gaps-status-and-decisions.md` §3 (stale, HEAD moved from 0eca3ed to e5b98b8), `2026-11-08-gaps-review-round4.md`, `2026-11-11-remediation-4-stage-0b-critic-round-2.md`, `2026-11-12-remediation-4-stage-0b-a3uv-enrolment.md`, `2026-11-13-stage-0b-integration-dependencies.md`.

Open items, verbatim-short with path:
1. Browser-agent-as-person (critic item 4). "A browser-controlling agent can omit the page-agent header and use the paired session's human authority." Options A (terminal confirm + OS sandbox), B (document trust), C (leave). Path: `.agents/reports/2026-11-08-gaps-status-and-decisions.md:49-57`. Round-4 review says "the only item every round has kept open" (`2026-11-08-gaps-review-round4.md:51`).
2. Gap A: no request ids for route/reassign/withdraw/remove/put (`gaps-status:59`).
3. Gap B: no Reassign control for conflicted answers (`gaps-status:63`).
4. Not verified: power-loss durability; desk browser UI end to end; round-5 fixes not blind-reviewed; migrations 7–8 only on fixtures (`gaps-status:67-72`).
5. Critic round 2 REJECT (`2026-11-11-…-critic-round-2.md`): F7 device.pair authority (terminal code vs A3-UV, owner decided A3-UV, later decision B); F5 `registered_agent` executor kinds; F6 14 P2 rows with `none_required`; F8 requester=executor; items 5–7 not done; containment for `grant.revoke` and `gauntlet.run.cancel` has no runtime path, SEA-Forge unpublished; same-terminal processes can read/type codes.
6. Enrolment critic round (`2026-11-12-…-a3uv-enrolment.md`): OPEN High: "requirements 4 and 5: `cognate_local_continuation` is catalog text" (resolved by decision B, commit d0ca846). OPEN Medium: approver sees only UA label. OPEN Low: `/m/api/pair/await` unauthenticated but catalog says `caller_own_session`; NE-41/NE-42 reused IDs; `pending-confirmations.ts` auto-confirm seam (test-only).
7. Architectural: no production first controller (409 `no_controller`); loopback WebAuthn registration proposed, trust root is owner's choice.
8. Integration dependencies D1–D6 (`2026-11-13-stage-0b-integration-dependencies.md` §4): D1 SEA-Forge P3 decision API; D2 identity service accepts authorization evidence; D3 controller scope enforcement; D4 bootstrap/recovery authority; D5 saga with reconciliation; D6 first controller.
9. Production stance: `device.pair` refused until D1–D6 close (owner Option B, integration-deps header).
10. Cross-branch: `docs/reports` does not exist; reports are in `.agents/reports/`. The integration-deps report is dated 2026-11-13, after today (2026-11-12). Treat its date as anomalous.

Other branches:
- Horizon-core: 3 `wip` commits (containment core, unvalidated) + 8 dirty files; the `enforcement-probe` test is untracked. Decide: keep or discard after diffing against gaps.
- Horizon-pr1 / pr1-narrow: PR-1 narrowed containment; earlier lineage, not in gaps by SHA. Preserve.
- Horizon (desk-v2): staged deletion of vendored identity tgz; untracked report dirs. Owner-level, not mine to clear.
- connector / seed / merge / gsr: already contained in gaps; safe to treat as historical.

TODO/FIXME: 0 in `src/` and `test/` (OBSERVED).

## 8. Contradictions, duplicated responsibilities, risks

- Stale status report: `gaps-status-and-decisions.md` says 128 commits, HEAD 0eca3ed, "547 tests" in review-round4; current is 150 commits, HEAD e5b98b8, 1628 tests. Do not cite the 11-08 docs as current.
- Test double P3 authority approves everything (`test/support/p3-authority-double.ts`); NE-75 passes only against it. Risk: "verified" claims for enrolment are double-only.
- Catalog vs code: `device.pair` requester/approver/executor split is fixed, but identity service accepts no evidence (D2). The catalog names identity service as designated writer while SEA-Forge decision is missing. Authority is split across three non-atomic writes (ledger, identity, approval), compensated not transactional.
- Browser agent trust (item 1) is unresolved; the agent door still requires its bearer key.
- Duplicate responsibilities: `src/adapters/sea-forge-wire.ts` (raw NDJSON) and Cognate SDK `SeaForgeAuthority` both talk to SEA-Forge. Boundaries are enforced by `scripts/check-boundaries.ts` (only two files may open the socket).
- Fixed-rows risk: 14 P2 rows `none_required` ownership (critic F6) could mask authority gaps.
- Power-loss durability and real legacy-store migration untested.

## 9. Unknowns

- Store engine and on-disk schema: not read (`src/runtime`, `src/projections`). Look there and `.agents/specs/horizon.agent.spec.yaml`.
- Whether `origin/main` is current (needs owner fetch; not done).
- Playwright e2e scope beyond `test/E2E-remote.test.ts`; `test/desk-outcomes.test.ts` mentions chromium: not read.
- Pan/zoom, beats, artifacts, graph/timeline UI: no evidence found.
- Context Kernel integration (13 refs) and Gauntlet (1 ref): not read.
- Horizon README ownership map (`README.md`, 16 KB): not read.
- Cognate Workbench relationship (24 refs): not read.
- `.cognate/vendor` tgz deletion on desk-v2: whether gaps still vendors the same tgz: not checked.
- Whether `~/.local/state/horizon-approval-test/checkout` contains anything relevant: not checked.

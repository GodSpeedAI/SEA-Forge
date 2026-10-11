# Verification Matrix

Maps each completion criterion to its packet, automated verifier, manual
validation, and evidence artifact. Every primary user journey has end-to-end
verification.

Legend: `[auto]` = deterministic command; `[manual]` = human observation on
real stack; `[e2e]` = packaged end-to-end.

---

## Installation and Upgrade

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| Clean Linux host installs versioned artifact set without source checkout | SF-012, SF-013 | artifact-checksum script | `[manual]` install on clean host | clean-host-install-log |
| First-run path selects or initializes a cell | SF-001, SF-008 | `cargo test -p sea-forge-server readiness` | `[manual]` open Workbench on fresh cell | readiness-scenario-tests |
| Desktop/server/CLI/schema/SFWP versions visible and checked | SF-001 | `cargo test -p sea-forge-server --test conformance_sfwp` | `[manual]` system.hello version handshake | conformance_sfwp results |
| Upgrade preserves record IDs and history | SF-004 | `cargo test -p sea-forge-cli migrate` | `[manual]` migrate flat run, verify ID/hash | migration-fixture-test |

## Deterministic Build and Generated Artifacts

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| Pinned inputs produce same contracts on repeat | SF-007 | `just workbench-check` (drift gate) | `[manual]` regenerate, confirm zero diff | drift-gate-recipe output |
| Rust JSON Schema, TS/AJV, tokens regenerated zero-diff | SF-007 | `bun run generate:contracts && git diff --exit-code` | — | zero-diff proof |
| No Bun runtime in bundle | SF-012 | bundle-inventory script | `[manual]` inspect package contents | bundle-inventory |
| Standalone Tauri workspace boundary intact | SF-007 | `just no-async-kernel` + boundary check | — | boundary-check output |

## Required Services and Startup

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| One documented procedure starts Workbench + server | SF-001, SF-012 | startup-matrix test | `[manual]` follow documented procedure | startup-matrix results |
| One absolute cell root and socket path | SF-001 | `cargo test -p sea-forge-server config` | `[manual]` set SEA_FORGE_ROOT, verify both move | config test output |
| Socket is owner-only (0600); second server fails safely | SF-001 | socket-in-use test | `[manual]` start two servers | socket-test output |
| Invalid config blocks startup | SF-002 | `cargo test -p sea-forge-server -- --test-threads=1` | `[manual]` malformed server.yaml | startup-error-tests |
| Valid reload affects future dispatches only | SF-002 | reload-event test | `[manual]` change config, observe | reload-event-test |
| Invalid reload retains last-known-good + typed event | SF-002 | reload-event test | — | reload-event-test |
| 10s request timeout preserves status | SF-002 | timeout integration test | `[manual]` send hanging request | timeout-integration-test |

## Frontend and Real Backend Communication

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| SFWP hello/version through real Tauri host | SF-012 | `[e2e]` packaged E2E handshake | `[manual]` open Workbench, observe connection | e2e-handshake-result |
| Renderer accesses no socket/store/SQL directly | SF-005, SF-007 | static import gate + bridge contract test | — | API-02 gate output |
| Every received payload validated against generated contract | SF-007 | AJV validator tests | — | contract test results |
| Events resume from durable cursor after disconnect/restart | SF-010 | `cargo test -p sea-forge-server --test conformance_sfwp` | `[manual]` kill server, reconnect, observe catch-up | cursor-resume proof |
| 10s timeout does not lose request status | SF-002, SF-006 | timeout + correlation test | `[manual]` timeout during mutation, recover by ID | timeout-integration-test |

## Primary User Journey (12 scenarios)

| Journey step | Packet | Automated verifier | Manual / E2E | Evidence artifact |
|---|---|---|---|---|
| 1. Open or initialize cell | SF-008 | readiness scenario tests | `[e2e]` open Workbench on temp cell | readiness-scenario-tests |
| 2. Display resolved actor, policy, model, integrity, sandbox | SF-005, SF-008 | identity + readiness tests | `[e2e]` observe readiness view | identity-ui-hook proof |
| 3. Ask what installation can do (affordance) | SF-008 | affordance disclosure test | `[e2e]` Thoth ask through UI | affordance-disclosure-test |
| 4. Select domain/template/criteria, inspect provenance | SF-009 | case.entry_options test | `[e2e]` template selection | case-commit-e2e-test |
| 5. Preflight + commit with stable idempotency key | SF-006, SF-009 | idempotency + case commit tests | `[e2e]` commit case | lost-response-test |
| 6. Allow path and deny/escalate path | SF-003, SF-009 | denial-no-side-effect test | `[e2e]` deny policy | denial-no-side-effect-test |
| 7. Command episode + human approval path | SF-009, SF-010 | approval + episode tests | `[e2e]` approve as second actor | sod-approver-test |
| 8. Execution and settlement as separate states (false success rejected) | SF-003, SF-009 | false-success test | `[e2e]` exit-zero + unmet criteria | false-success-test |
| 9. Disconnect during mutation, recover by request ID | SF-006, SF-010 | disconnect-recovery test | `[e2e]` kill connection mid-commit | disconnect-recovery-test |
| 10. Restart during/after work, inspect case/run/recovery | SF-004, SF-010 | restart-resolution + orphan tests | `[e2e]` kill server mid-work | orphan-restart-test |
| 11. Inspect authority, trace, evidence, settlement | SF-003, SF-011 | full-episode-cross-link test | `[e2e]` open run record | evidence-inspection-test |
| 12. Reuse accepted result (memory/capability/artifact) | SF-011 | reuse-path test | `[e2e]` promote capability | reuse-path-test |

## Persistence and Data Integrity

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| Every case-linked run ID resolves to one canonical run | SF-004 | dual-layout-locator test | `[manual]` run.get on case-created run | dual-layout-locator-test |
| Every episode contains authority echo, trace, evidence, criteria, settlement | SF-003 | full-episode-cross-link test | `[manual]` inspect run record | full-episode-cross-link-test |
| Append-only streams survive killed writer as valid prefix | SF-003 | `devbox run -- just proof` (P1-P4b) | `[manual]` kill-9 during append | proof output |
| SQLite/query projections rebuild from canonical records | SF-011 | capability rebuild test | — | rebuild-equivalence proof |
| Flat minimum-run fixtures remain readable | SF-004 | migration-fixture test | `[manual]` run.get on CLI-created run | migration-fixture-test |

## Authority, Security, and Safety

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| Every protected ingress reaches authority before effect (incl. workspace) | SF-003 | denial-no-side-effect test | `[manual]` deny, check no workspace created | denial-no-side-effect-test |
| Exact grant consumed by matching runtime; no reuse with changed context | SF-003 | `cargo test -p sea-forge-authority` | — | authority type-check proof |
| Approver identity and SoD server-resolved | SF-005, SF-010 | sod-approver test | `[e2e]` same actor submit+approve | sod-approver-test |
| Child processes: argv, minimal env, bounded output/time, no network | SF-003 | runtime negative tests | — | sentinel-scan + argv tests |
| Secrets never in source records/logs/errors/views/fixtures/bundles | SF-013 | `gitleaks detect --no-banner --redact` | — | gitleaks output |
| Renderer CSP and Tauri capability scopes permit only required local actions | SF-012 | capability audit | `[manual]` inspect tauri.conf.json capabilities | capability-audit result |

## Errors and Recovery

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| Failures retain machine-readable classes across server/bridge/UI | SF-002, SF-005 | typed-error tests | `[manual]` trigger each failure class | error-class test output |
| Blocked/failed state identifies evidence, capability, next action | SF-008 | readiness scenario tests | `[e2e]` observe blocked state | readiness-scenario-tests |
| Unknown versions/verbs/variants fail cleanly | SF-007 | `cargo test -p sea-forge-server --test conformance_sfwp` | — | conformance test output |
| Orphaned work after restart is explicit and recoverable | SF-010 | orphan-restart test | `[manual]` kill during delegation | orphan-restart-test |

## Packaging, CI, and Release

| Criterion | Packet | Automated verifier | Manual validation | Evidence artifact |
|---|---|---|---|---|
| CI aggregates kernel + workbench + host + drift + package + E2E | SF-013 | aggregate CI workflow | `[manual]` review workflow | aggregate-ci-workflow |
| Linux package + CLI archive built, checksummed, smoke-tested | SF-013 | artifact-checksum script | `[manual]` install artifact | checksums + install-smoke |
| Release notes name migrations, platforms, limitations | SF-013 | — | `[manual]` review notes | release-notes |
| No cargo install claim until closure published | SF-013 | — | `[manual]` review notes | release-notes |

## Test Architecture Coverage

| Layer | Verifier | Packets |
|---|---|---|
| Unit (pure reducers, validation, canonicalization) | `cargo test --workspace` | SF-003 through SF-011 |
| Contract (Rust/schema/TS compatibility, old-reader) | `conformance_sfwp.rs`, `just workbench-check` | SF-007 |
| Integration (real server, socket, authority, sandbox, restart) | `cargo test -p sea-forge-server`, `just proof` | SF-002 through SF-004 |
| Packaged E2E (Tauri → host → server → records) | `bunx playwright test e2e/` | SF-012 |
| Linux Landlock | sandbox tests (already in suite) | SF-003 |
| macOS Seatbelt | `[manual]` on macOS host | SF-012 |
| All skips state reason | test annotation review | all |

## Final Acceptance Commands

```sh
# The complete gate sequence (run from repository root):
devbox run -- just check           # context + fmt + clippy + typecheck + deny + gitleaks
devbox run -- just test            # full Rust test suite
devbox run -- just proof           # P1-P4b minimum proofs
just no-async-kernel               # 19 kernel crates async-free
just workbench-check               # frontend + generated-drift + boundary
cd workbench && bunx playwright test e2e/ --workers=1   # packaged real-stack E2E
```

All must pass. macOS Seatbelt and optional external integrations (ACP,
SWE_SEED, witness, IFL) are skipped with stated reasons unless explicitly in
scope.

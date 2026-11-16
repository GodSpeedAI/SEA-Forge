# EGM Drift Report — 2026-10-30

Scope: all 25 crate contract READMEs under `crates/` (partition file `/tmp/egm-partitions/crates-group-1.txt` was absent; per operator decision all crates were analyzed). Doc-to-code verification of cited symbols, constants, invariants, side effects, and Interface completeness.

## crates/sea-forge-os/README.md
- Overview and Invariants claim the crate is "the only crate in the workspace that may contain `unsafe` code" (README.md:4, README.md:11 "Sole holder of workspace `unsafe` code"). This is false in code: `crates/sea-forge-sandbox/src/socket_policy.rs:47` carries `#![allow(unsafe_code)]` and contains multiple `unsafe` blocks (e.g. lines 81, 137, 159, 172), and `crates/sea-forge-sandbox/src/bin/sea-forge-workload-exec.rs:19` also carries `#![allow(unsafe_code)]`. The crate's own module docs acknowledge this: `crates/sea-forge-os/src/lib.rs:3-8` states "The only other `unsafe` in the workspace is the pre-existing Linux sandbox code (seccomp notify in `sea-forge-sandbox/src/socket_policy.rs`, raw syscalls in `sea-forge-workload-exec`)". (The `Cargo.toml` description repeats the same overclaim at `crates/sea-forge-os/Cargo.toml:6`; the workspace-level lint is `unsafe_code = "deny"` with per-module allows, `Cargo.toml:40`.)

## crates/sea-forge-agent/README.md
No drift detected

## crates/sea-forge-artifact-ip/README.md
No drift detected

## crates/sea-forge-authority/README.md
No drift detected

## crates/sea-forge-capability/README.md
No drift detected

## crates/sea-forge-case-runner/README.md
No drift detected

## crates/sea-forge-cell/README.md
No drift detected

## crates/sea-forge-changeset/README.md
No drift detected

## crates/sea-forge-cli/README.md
No drift detected

## crates/sea-forge-core/README.md
No drift detected

## crates/sea-forge-domain/README.md
No drift detected

## crates/sea-forge-domainforge/README.md
No drift detected

## crates/sea-forge-evidence/README.md
No drift detected

## crates/sea-forge-extension/README.md
No drift detected

## crates/sea-forge-ledger/README.md
No drift detected

## crates/sea-forge-planner/README.md
No drift detected

## crates/sea-forge-realitytrace/README.md
No drift detected

## crates/sea-forge-runtime/README.md
No drift detected

## crates/sea-forge-sandbox/README.md
No drift detected

## crates/sea-forge-self-model/README.md
No drift detected

## crates/sea-forge-server/README.md
No drift detected

## crates/sea-forge-settlement/README.md
No drift detected

## crates/sea-forge-spec-pipeline/README.md
No drift detected

## crates/sea-forge-thoth/README.md
No drift detected

## crates/sea-forge-trace/README.md
No drift detected

# Src-level EGM contract READMEs (crates/*/src/**) — 2026-10-30

Scope: all 32 contract READMEs under `crates/*/src/` (verified via `git ls-files 'crates/**/src/**/README.md'` for tracked files plus `find` for untracked ones). Doc-to-code verification of cited symbols, constant values, invariants, side effects, and Interface completeness.

## crates/sea-forge-agent/src/README.md
No drift detected

## crates/sea-forge-artifact-ip/src/README.md
- Interface lists `LedgerFacts` as part of the module surface (README.md:18 "Views: ... `LedgerFacts`"), but `LedgerFacts` is private: `crates/sea-forge-artifact-ip/src/lib.rs:2114` is `struct LedgerFacts {` (no `pub`), and its constructor `verified_ledger_facts` (`lib.rs:2145`) is likewise private. It is not reachable from outside the crate.
- Invariant "Required attestation must forbid degraded mode" is cited to [lib.rs:authorize_attestation] (README.md:27), but `authorize_attestation` (`crates/sea-forge-artifact-ip/src/lib.rs:3321`) only matches the decision's `degraded_mode` parameter against the carried value; the actual enforcement lives in `attest` (`crates/sea-forge-artifact-ip/src/lib.rs:3410-3414`: `if matches!(policy, AttestationPolicy::Required) && degraded_mode != "forbidden"` returning the "required attestation must forbid degraded mode" error). Correct citation target is [lib.rs:attest].
- Invariant "Each value-evidence source must have at least one accepted parent establishing its `from_stage`" is cited to [lib.rs:validate_ledger_references] (README.md:26), but `validate_ledger_references` (`crates/sea-forge-artifact-ip/src/lib.rs:2562`) validates registration/evidence/token references and does not enforce the from_stage parent rule. The rule is enforced in `validate_before_transform` (`crates/sea-forge-artifact-ip/src/lib.rs:1597-1614`: "non-cognitive source lacks an accepted parent token") and re-checked at `lib.rs:3522-3545`. Correct citation target is [lib.rs:validate_before_transform].

## crates/sea-forge-authority/src/README.md
No drift detected

## crates/sea-forge-authority/src/cep/README.md
No drift detected

## crates/sea-forge-capability/src/README.md
- Interface lists `MAX_STATEMENT_LEN = 1000` as part of the Memory API (README.md:13), but the constant is private: `crates/sea-forge-capability/src/memory.rs:17` is `const MAX_STATEMENT_LEN: usize = 1000;` (no `pub`; contrast `MAX_RECALL_LIMIT` at `memory.rs:18` which is `pub const`). The value matches; only the visibility claim in the Interface section drifts.

## crates/sea-forge-case-runner/src/README.md
No drift detected

## crates/sea-forge-case-runner/src/case_ops/README.md
No drift detected

## crates/sea-forge-cli/src/README.md
No drift detected

## crates/sea-forge-cli/src/commands/README.md
No drift detected

## crates/sea-forge-core/src/README.md
No drift detected

## crates/sea-forge-core/src/yaml/README.md
No drift detected

## crates/sea-forge-domain/src/README.md
No drift detected

## crates/sea-forge-domainforge/src/README.md
No drift detected

## crates/sea-forge-evidence/src/README.md
No drift detected

## crates/sea-forge-extension/src/README.md
No drift detected

## crates/sea-forge-ledger/src/README.md
No drift detected

## crates/sea-forge-os/src/README.md
- Invariants claim "This is the only workspace crate permitted to contain `unsafe` code" (README.md:17, "only workspace crate permitted to contain `unsafe` code (crate description in Cargo.toml)"). False in code: `crates/sea-forge-sandbox/src/socket_policy.rs:47` carries `#![allow(unsafe_code)]` with multiple `unsafe` blocks, and `crates/sea-forge-sandbox/src/bin/sea-forge-workload-exec.rs:19` also carries `#![allow(unsafe_code)]` (SYS_close_range at `sea-forge-workload-exec.rs:35`). The crate's own module docs contradict the README claim: `crates/sea-forge-os/src/lib.rs:2-6` states "The only other `unsafe` in the workspace is the pre-existing Linux sandbox code (seccomp notify in `sea-forge-sandbox/src/socket_policy.rs`, raw syscalls in `sea-forge-workload-exec`)" — the same overclaim already recorded for the crate-level README above. (All other cited symbols/values in this README, e.g. `CLIENT_PIPE_RIGHTS = 0x0012_018B` at `imp.rs:238`, verified correct.)

## crates/sea-forge-planner/src/README.md
No drift detected

## crates/sea-forge-realitytrace/src/README.md
No drift detected

## crates/sea-forge-runtime/src/README.md
No drift detected

## crates/sea-forge-sandbox/src/README.md
No drift detected

## crates/sea-forge-sandbox/src/bin/README.md
No drift detected

## crates/sea-forge-self-model/src/README.md
No drift detected

## crates/sea-forge-server/src/README.md
No drift detected

## crates/sea-forge-server/src/bin/README.md
No drift detected

## crates/sea-forge-server/src/sfwp/README.md
No drift detected

## crates/sea-forge-settlement/src/README.md
No drift detected

## crates/sea-forge-spec-pipeline/src/README.md
No drift detected

## crates/sea-forge-thoth/src/README.md
No drift detected

## crates/sea-forge-trace/src/README.md
No drift detected

## workbench/README.md
No drift detected

## workbench/apps/README.md
No drift detected

## workbench/apps/desktop/README.md
- Stale Vite template boilerplate retained at the top of the contract (lines 1-26: "This template provides a minimal setup...", "two official plugins are available", "React Compiler is not enabled on this template") is generic scaffold text, not generated contract content; the actual app contract begins at "## Overview" (line 28). Cosmetic drift: the contract was not regenerated after the template header was superseded by the real Overview/Interface/Invariants sections.
- No code drift: all 17 routes, scripts, guard set (G1/G2/G9 at src/guards/useGuardContext.ts:25), governedQuery.ts:68 queryGoverned, useIdentity.ts:59 selectedActorId sessionStorage, PROVENANCE.md golden-reference, and package.json deps were verified consistent.

## workbench/apps/desktop/src/README.md
No drift detected

## workbench/apps/desktop/src/composer/README.md
No drift detected

## workbench/apps/desktop/src/core/README.md
No drift detected

## workbench/apps/desktop/src/core/camera/README.md
No drift detected

## workbench/apps/desktop/src/core/chrome/README.md
No drift detected

## workbench/apps/desktop/src/core/pipeline/README.md
No drift detected

## workbench/apps/desktop/src/core/quality/README.md
No drift detected

## workbench/apps/desktop/src/core/shaders/README.md
No drift detected

## workbench/apps/desktop/src/dev/README.md
No drift detected

## workbench/apps/desktop/src/guards/README.md
No drift detected

## workbench/apps/desktop/src/hooks/README.md
- Interface cites a `useCases()` anchor symbol with span [useCases.ts:useCases] ("`useCases()` family: `useCaseList()`, `useCaseOverview(caseId)`, `useCaseHorizon(caseId)`, `CASES_QUERY_KEY`"), but `useCases.ts` exports no `useCases` function — only `useCaseList` (line 54), `useCaseOverview` (line 78), `useCaseHorizon` (line 95), `CASES_QUERY_KEY` (line 28), and a private `useCaseEventInvalidation` (line 51).
- Interface cites a `useRuns(caseId?)` anchor symbol with span [useRuns.ts:useRuns], but `useRuns.ts` exports no `useRuns` function — only `useRunList` (line 42), `useRunRecord` (line 60), and `RUNS_QUERY_KEY` (line 28).

## workbench/apps/desktop/src/machines/README.md
No drift detected

## workbench/apps/desktop/src/pages/README.md
No drift detected

## workbench/apps/desktop/src/projections/README.md
No drift detected

## workbench/apps/desktop/src/shell/README.md
No drift detected

## workbench/apps/desktop/src/spatial/README.md
No drift detected

## workbench/apps/desktop/src/spatial/focus/README.md
No drift detected

## workbench/apps/desktop/src/spatial/time/README.md
No drift detected

## workbench/apps/desktop/src/spatial/zoom/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/artifacts/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/case/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/case-design/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/causal/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/execution/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/home/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/judgment/README.md
No drift detected

## workbench/apps/desktop/src/surfaces/semantic-beat/README.md
- Overview claims "Thoth interaction stays behind `ThothInteractionPort`", but no `ThothInteractionPort` symbol exists anywhere in the codebase (rg over workbench/**/*.ts,*.tsx finds only the comment itself at SemanticBeatSurface.tsx:2). The surface imports `ThothPage` directly (SemanticBeatSurface.tsx:5), and ThothPage calls the host bridge via `useThoth` (ThothPage.tsx:33, hooks/useThoth.ts:71) with no interaction port abstraction.

## workbench/apps/desktop/src/surfaces/temporal/README.md
No drift detected

## workbench/packages/README.md
No drift detected

## workbench/packages/contracts/README.md
No drift detected

## workbench/packages/contracts/generated/README.md
No drift detected

## workbench/packages/contracts/scripts/README.md
No drift detected

## workbench/packages/sea-forge-astryx-theme/README.md
No drift detected

## workbench/packages/sea-forge-astryx-theme/src/README.md
No drift detected

## workbench/packages/sea-forge-ui-components/README.md
No drift detected

## workbench/packages/sea-forge-ui-components/src/README.md
No drift detected

## workbench/packages/sea-forge-ui-components/src/stories/README.md
- Interface claims `AllPillVariants` exercises "all 13 `GovernedStatusVariant` inputs" [components.stories.tsx:AllPillVariants], but the story renders only 12 of the 13 union members: the `unsupported` variant (GovernedStatusPill.tsx:12) is never rendered. The 13th rendered pill is the literal `"unrecognized_variant_xyz"` (components.stories.tsx:38), an out-of-union probe, not a `GovernedStatusVariant` input.

## workbench/packages/sea-forge-ui-tokens/README.md
No drift detected

## workbench/packages/sea-forge-ui-tokens/scripts/README.md
No drift detected

## apps/README.md
No drift detected

## apps/godspeed-casework-go/README.md
No drift detected

## apps/godspeed-casework-go/cmd/README.md
No drift detected

## apps/godspeed-casework-go/cmd/godspeed-casework/README.md
No drift detected

## apps/godspeed-casework-go/internal/README.md
- Subdirectories & Boundaries claims to list the packages "below the command entrypoint" but omits two real packages under `internal/`: `internal/boundary` (enforcement package with `internal/boundary/boundary_test.go`, described in this README's own Overview text yet absent from the subdirectory list) and `internal/loadtest` (`internal/loadtest/load_test.go`, `budgets_test.go`, `rig_test.go`, the `-tags load` suite behind `just casework-load` at justfile:1176). A stray non-Go artifact directory `internal/sfwp/client.go/graft/` (empty, git-ignored) also exists with no corresponding package.

## apps/godspeed-casework-go/internal/adapters/README.md
No drift detected

## apps/godspeed-casework-go/internal/adapters/fakeauthority/README.md
No drift detected

## apps/godspeed-casework-go/internal/adapters/sfwp/README.md
No drift detected

## apps/godspeed-casework-go/internal/apperr/README.md
No drift detected

## apps/godspeed-casework-go/internal/artifactstore/README.md
No drift detected

## apps/godspeed-casework-go/internal/auth/README.md
- `normalizeUsername` is cited as [auth.go:normalizeUsername], but the function is defined in `internal/auth/local.go:84` (`func normalizeUsername(s string) string`); `auth.go` contains no such symbol.

## apps/godspeed-casework-go/internal/config/README.md
No drift detected

## apps/godspeed-casework-go/internal/contract/README.md
No drift detected

## apps/godspeed-casework-go/internal/coordinator/README.md
- The `Intent` type is cited as [coordinator.go:Intents], but no `Intents` symbol exists in `internal/coordinator/coordinator.go`; the type is `type Intent struct` at coordinator.go:30.

## apps/godspeed-casework-go/internal/intents/README.md
No drift detected

## apps/godspeed-casework-go/internal/livestack/README.md
No drift detected

## apps/godspeed-casework-go/internal/livetest/README.md
No drift detected

## apps/godspeed-casework-go/internal/metrics/README.md
No drift detected

## apps/godspeed-casework-go/internal/ports/README.md
No drift detected

## apps/godspeed-casework-go/internal/preflight/README.md
No drift detected

## apps/godspeed-casework-go/internal/projection/README.md
No drift detected

## apps/godspeed-casework-go/internal/server/README.md
No drift detected

## apps/godspeed-cognitive-ui/README.md
No drift detected

## apps/godspeed-cognitive-ui/e2e/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/adapters/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/adapters/conformance/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/adapters/http/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/adapters/local/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/app/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/artifacts/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/artifacts/renderers/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/camera/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/layout/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/model/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/narrative/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/ports/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/scene/README.md
No drift detected

## apps/godspeed-cognitive-ui/src/ui/README.md
No drift detected

## docs/execution/README.md
No drift detected

## docs/execution/journey/README.md
No drift detected

## packaging/deb/README.md
No drift detected

## packaging/macos/README.md
No drift detected

## scripts/openshell/README.md
No drift detected

## scripts/security/README.md
No drift detected

## README.md (root: "Subdirectories & System Boundaries", "Global Invariants", "Global Side Effects", "System Verification")
- Crate-count conflict across contracts: the EGM index says "The 25 workspace crates" (README.md:1037), which matches the actual `[workspace] members` in Cargo.toml (25 entries), but `crates/AGENTS.md:3` still says "the 22 Rust workspace crates" — one of the two governing documents is stale. The kernel-boundary claim itself ("only `sea-forge-server` / `sea-forge-agent` may be async", README.md:1046) matches `crates/AGENTS.md:9` and `just no-async-kernel` (justfile:413). All six child contract links (apps, crates, docs, packaging, scripts, workbench READMEs) exist.

# Settlement Integrity Spec Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or inline execution to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align SEA Forge's minimum and full specifications with the Genesis definitions of settlement and capability without breaking the implemented v0.1 kernel.

**Architecture:** Preserve every v0.1 persisted type and proof. Clarify minimum terminology, then add a v0.2 settlement-declaration layer and promotion model to the full spec. Audit the Rust implementation against the clarified minimum contract and change code only if a focused test exposes a mismatch.

**Tech Stack:** Markdown specifications, stable Rust 2021 if the compatibility audit requires code, existing `just`/Devbox validation.

## Global Constraints

- `.agents/specs/spec-minimum.md` remains the normative implemented kernel contract.
- Full-spec changes are additive and MUST NOT redefine v0.1 IDs, record fields, lifecycle ordering, authority hashes, or P1-P4b.
- One accepted run is an observation, not proof of durable capability.
- Strong settlement requires predeclared proof, feedback-reliability assessment, declarer standing, and independence from the acting agent.
- SEA Forge records governance and settlement facts; CognitiveOS retains payment/path valuation, and GodSpeed-Agent retains horizon/developmental routing.
- No new dependency, network path, async runtime, or persisted v0.1 field is allowed.

---

### Task 1: Clarify the implemented minimum contract

**Files:**
- Modify: `.agents/specs/spec-minimum.md`

**Interfaces:**
- Consumes: implemented v0.1 `SettlementEvent`, `SemanticEnvelope`, and `capabilities.jsonl` contracts.
- Produces: explicit terminology that downstream full-spec promotion rules can extend without re-keying v0.1 history.

- [x] Replace language that implies one envelope proves a capability with “capability-attempt memory” language while retaining persisted names.
- [x] Define kernel-local v0.1 settlement as evidence plus predeclared verification, and list reliability weighting and independent declaration as deferred strong-settlement conditions.
- [x] State that P1-P4b prove the governed kernel, not durable capability.
- [x] Add a compatibility note forbidding v0.1 schema or proof changes for this clarification.
- [x] Run `git diff --check` and verify `rg -n "one accepted|kernel-local|capability-attempt|strong settlement" .agents/specs/spec-minimum.md` finds each normative clarification.

### Task 2: Extend the full settlement contract

**Files:**
- Modify: `.agents/specs/spec-full.md`

**Interfaces:**
- Consumes: v0.1 `SettlementEvent` and the approved design in `.agents/plans/2026-07-11-settlement-integrity-alignment.md`.
- Produces: `SettlementAuthority`, `SettlementDeclaration`, additive policy/config fields, persistence locations, lifecycle rules, and failure behavior.

- [x] Add the SEA Forge → SettlementAuthority → SEA Forge ownership/data-flow boundary and name SWE_SEED as the first external adapter.
- [x] Define `SettlementDeclaration` fields for claim/criteria timing, verifier identity/version, declarer standing, independence, reliability dimensions/weight, evidence, and canonical hash.
- [x] Specify weak/local versus qualifying/strong declarations and fail-closed behavior when integrity inputs are missing.
- [x] Add policy controls for required settlement strength and trusted settlement authorities without letting evaluators self-authorize.
- [x] Add storage, trace/evidence linkage, lifecycle ordering, error classes, and rebuild/source-of-truth rules.

### Task 3: Replace count-only capability proof with promotion

**Files:**
- Modify: `.agents/specs/spec-full.md`

**Interfaces:**
- Consumes: qualifying `SettlementDeclaration` records.
- Produces: additive `CapabilityRecord` promotion fields and deterministic `require_proven` semantics.

- [x] Retain raw accepted/rejected/escalated counts as observations.
- [x] Add `attempted | demonstrated | proven | metabolized` status and promotion-policy reference/hash.
- [x] Add qualifying weight, declared variation dimensions/coverage, disruption recovery, regression rate, orchestration-burden evidence, confidence, and contraction reasons.
- [x] Define default promotion thresholds and deterministic rebuild behavior.
- [x] Change `require_proven` from `accepted >= 1` to the configured promotion status/threshold.
- [x] Keep payment estimation, affordance ranking, and horizon selection outside SEA Forge.

### Task 4: Add threat-model and conformance coverage

**Files:**
- Modify: `.agents/specs/spec-full.md`

**Interfaces:**
- Consumes: Tasks 2–3 contracts.
- Produces: milestone gates and recovery tests that falsify manufactured settlement and false capability promotion.

- [x] Add tests for post-hoc criteria, self-declaration, missing standing, gameable feedback, low attribution confidence, and hidden-debt blindness.
- [x] Add tests showing identical repetition does not satisfy variation and disruption recovery does contribute.
- [x] Add promotion, regression/revocation contraction, and rebuild-purity tests.
- [x] Place the additive implementation in milestone order without weakening minimum P1-P4b.
- [x] Update the full-spec definition of done and claim table.

### Task 5: Audit implementation compatibility and validate context

**Files:**
- Inspect: `crates/sea-forge-core/src/{types,settlement,capability,pipeline}.rs`
- Modify only if required by a failing minimum-contract test: the inspected files and their focused tests.
- Modify: `.agents/CURRENT_STATUS.md`, `.agents/OBSERVED_DEBT.md`, `.agents/LESSONS.md` only where the project memory contract requires it.

**Interfaces:**
- Consumes: revised normative specs and implemented v0.1 code.
- Produces: evidence that no minimum behavior/schema change is required, or a focused TDD correction if a mismatch exists.

- [x] Map every revised minimum MUST to an existing type, test, or explicit full-spec deferral.
- [x] If a mismatch exists, write one focused failing test, run it to confirm failure, implement only the minimum fix, and rerun the focused test. (No mismatch found; no code change.)
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace --all-features --locked`, and `just proof` if Rust changed; otherwise document why runtime gates are unaffected.
- [x] Run `devbox run -- just context-check` and `git diff --check` in all cases.
- [x] Review the final diff for compatibility, source-of-truth coherence, manufactured-settlement gaps, and needless scope.
- [x] Commit the normative spec and memory update as one documentation increment.

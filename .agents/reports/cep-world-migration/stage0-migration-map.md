# CEP / world_ref migration — Stage 0 migration map

Date: 2026-10-04. Branch `migration/cep-world-ref` (worktree off `casework/live-wiring` @ dc82736).
Method: read-only recon (4 Haiku passes) plus direct verification by git grep / rev-list / file reads.
Nothing outside this file was changed in any repository.

## Decisions (operator)
- SEA-Forge work (Stages 1, 4-8) happens on a branch off `casework/live-wiring`, not `full-spec`.
- pg0 migration for SEA-Forge has NOT started and is out of scope. World bindings and governance records must preserve ledger properties (append-only, per-stream ordering, expected ordinal/CAS, previous hash, hash chain, MMR/checkpoints, idempotency) so a later pg0 move does not redo them.

## Participating repositories
| Repo (dir) | Branch | Dirty | DomainForge coupling | Version today |
|---|---|---|---|---|
| domainforge | main | 0 | source of truth | 0.18.2 (core crate, TS pkg; commit 5003b0d) |
| sea-rs | casework/live-wiring | 16 | crate `domainforge-core` (`=` pin, crates/sea-forge-domainforge) | casework 0.16.0; main/full-spec 0.15.0 |
| cognate | cognate/harness | 0 | npm `@godspeedai/domainforge` (18 files) | 0.18.2 |
| sxr (RealityTrace) | main | 0 | CLI `domainforge envelope` via `SXR_DOMAINFORGE_BIN` (sxr-df) | whatever binary is on PATH |
| gauntlet | main | 4 | CLI envelope adapter (`domainforge-adapter`), envelope schema `domainforge-semantic-envelope/v1` | whatever binary is on PATH |
| SWE_SEED | deploy-prep | 1 | type-gated DomainForge identity (`federation/identity.rs`), no dependency | n/a |
| godspeed_agent | audit-corrections/neatcode-2026-07-29 | 0 | identity referenced in convergence test only | n/a |
| Context_Kernel | harden | 0 | none (1 marketing-doc mention of "semantic envelopes") | n/a |
| cep | slice-0a-cep-semantic-envelope | 0 | none (protocol repo) | n/a |
| domainforge-lsp | semantic-adapter | 0 | path dep `../domainforge/sea-core` — target does not exist | BROKEN / stale |
| domainforge-vsc-extension | dev | 0 | grammar only | n/a |
| SEA (legacy) | dev | 45 | npm `domainforge-cli 1.0.9` | separate legacy line |

Installed CLI: `~/.cargo/bin/domainforge` = 0.18.1. RealityTrace and Gauntlet run it, so effective skew exists at runtime even though no manifest records it.

## Facts that shape later stages
1. Branches: `full-spec` has 98 commits `casework/live-wiring` lacks; `casework/live-wiring` has 215 that `full-spec` lacks. They diverged.
2. Canonical CEP source exists: repo `cep` (GodSpeedAI/canonical-evaluation-protocol). `schemas/cep-semantic-envelope.schema.json` (58 KB) is the normative CEP-0008 full profile. `schemas/semantic-envelope.schema.json` (3 KB) is the legacy v1 flat profile. Stage 3 extends `cep`; it does not create another schema.
3. Existing CEP-0008 code that must be reconciled, not replaced: DomainForge `application/envelope.rs` (DomainModelIdentity, canonical_digest, snapshot emission); sea-forge-extension `cep0008.rs` (flat v1 only, self-declared non-conformant to full CEP-0008); sxr-core `envelope.rs`; Cognate `docs/evidence.md` extension binding; hassos-addon-agent-memory-ledger conformance profile test.
4. DomainForge has its own envelope (`domainforge-semantic-envelope/v1`, `self_hash`). Its relationship to CEP `semantic_snapshot` is a Stage 3 question.
5. No `world_ref`, `world_alias`, or `world_label` exists in any repo.
6. SEA-Forge `DomainModelRef` (crates/sea-forge-domainforge/src/lib.rs) holds source_refs, domainforge_version, semantic_model_sha256, concept/class refs, validation evidence refs. Stage 4 evolves it.
7. Cognate: governance today is kernel capability authorization plus runtime `ActionPolicy` (profiles/harness/src/index.ts); continuations exist with idempotency_key and expected_version; transports are Connect/protobuf, MCP, A2A, ag-ui WebSocket; E2E is Playwright (tests/e2e), no agent-browser.
8. SEA-Forge settlement transport to SWE_SEED already exists (`SweSeedTransport`, crates/sea-forge-settlement/src/declaration.rs).
9. RealityTrace is in-process Rust and CLI; no cross-process transport. Stage 8 must add one or use an existing CLI path (`sxr ingest`).
10. Governance for edits: SEA-Forge `AGENTS.md` requires a governing spec under `.agents/specs/` and a plan under `.agents/plans/` before substantial work, `just context-check` before handoff, and asks before dependency upgrades or public-interface changes. Operator pre-authorized the DomainForge upgrade in the migration brief.

## Boundary matrix
Classes: MUST = must become CEP; MAY = may remain native; NOT = not a semantic boundary.

| # | Producer -> Consumer | Native source -> target | Transport today | Class | CEP profile | world_ref | Authority / evidence / settlement | Strategy |
|---|---|---|---|---|---|---|---|---|
| 1 | DomainForge -> SEA-Forge, Cognate, RealityTrace, Gauntlet | Graph/envelope -> native models | crate API, npm, CLI | MUST | semantic_snapshot | defines it | identity only | conform existing emission; add world_ref derivation |
| 2 | Cognate -> SEA-Forge | GovernedOperation (new) -> authority intake | none (internal ActionPolicy / kernel authz) | MUST | authority_request | required | requests authority | Stage 5 port, Stage 6 wire |
| 3 | SEA-Forge -> Cognate | decision -> native decision | none | MUST | authority_decision | required | grants/denies/escalates; keep boundary/degraded/constrained | Stage 6 |
| 4 | Cognate -> SEA-Forge (escalation) | continuation -> authority re-request | Connect continuations (internal) | MUST | authority_request/decision lineage | required | revalidation | Stage 7 |
| 5 | Cognate -> RealityTrace | observation -> sxr ingest | CLI | MUST | execution_trace | required | observation is not settlement | Stage 8 |
| 6 | RealityTrace -> SEA-Forge | evidence -> evidence ingestion | none | MUST | evidence_packet | required | evidence relative to a question/claim | Stage 8 |
| 7 | SEA-Forge -> SWE_SEED / consumers | settlement declaration | SweSeedTransport | MUST | settlement_packet | required | owns settlement | Stage 8 |
| 8 | Context_Kernel -> Agent/Cognate | context bundle | to inspect | MUST (confirm) | context_bundle | required | partial must not read as complete | Stage 9 |
| 9 | GodSpeed Agent <-> SEA-Forge | work request, authority intake | to inspect | MUST (confirm) | work_request + authority_request | required | orchestration grants no authority | Stage 9 |
| 10 | Gauntlet <-> RealityTrace / SEA-Forge | judgment, case/result, evidence | adapters, CLI | MAY / confirm per spec | benchmark/evidence profiles | per spec | keeps its judgment model | Stage 9 |
| 11 | SWE_SEED <-> SEA-Forge / Context Kernel | evaluation/settlement envelope | federation client | MUST where it carries identity or settlement | settlement_packet | required | not an owner | Stage 9 |
| 12 | Cognate kernel internals, cache, KV, UI state, local appends | native | in-process | NOT | none | n/a | none | leave |
| 13 | domainforge-lsp, vsc-extension -> DomainForge | editor tooling | LSP | MAY | none | n/a | none | version convergence only (LSP currently broken) |
| 14 | SEA (legacy) | legacy CLI 1.0.9 | n/a | MAY | none | n/a | none | deprecate or exclude explicitly in Stage 1 |

## Stage 0 settlement gate
- Every DomainForge consumer identified, with mechanism (crate / npm / CLI binary / type gate / path dep / grammar): met.
- Major handoffs identified: met for rows 1-7. Rows 8-11 are classified but their native message types are not yet read; they are Stage 9 work and do not block Stages 1-8.
- Governing documents known: met for domainforge, sea-rs, cognate, sxr, gauntlet, cep, SWE_SEED, Context_Kernel. Not read: godspeed_agent, domainforge-lsp, vsc-extension (Stage 1 and 9 read them before editing).
- Migration order mapped against repo state: met, with the open items below.

## Open items entering Stage 1 (real, not speculative)
- Stage 1 needs a governing spec and plan under `.agents/specs` and `.agents/plans` in sea-rs, then the 0.16.0 -> 0.18.2 upgrade of `sea-forge-domainforge` (API changes unknown until compile).
- Decide the fate of `domainforge-lsp` (broken path dep) and legacy `SEA` (`domainforge-cli 1.0.9`).
- Gauntlet (4 dirty files) and SEA (45 dirty files) have local work; any edits there go through a branch or worktree.
- RealityTrace and Gauntlet pin no CLI version; Stage 1 must add an explicit pin or version check.
- Reconcile the two cep envelope schemas and the DomainForge envelope in Stage 3.

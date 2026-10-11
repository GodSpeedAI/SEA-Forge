# DomainForge, RealityTrace, SEA-Forge, and CEP Interface Investigation

**Status:** Remediated and Verified across all substrates (2026-09-08)  
**Implementation Resolution:** All recommendations (D-01 through D-05) and defect findings (F-01 through F-07) have been implemented, verified, and settled.
- **D-01 / F-01 (DomainForge Lineage):** Slice 0B additive modules ported onto current maintained tip (`chore/release-please-commit-fix`). PR #122 fixes preserved. Unflagged output matches committed golden snapshot byte-for-byte.
- **D-02 / F-02 (Decoupled Validity):** SXR decouples representation existence from model validity. `model_validation_status` preserved on `Resolved`. `authorize_settlement` rejects positive settlement if `semantics_version == "v1-legacy"` or `model_validation_status == "invalid"`.
- **D-03 / F-03 (Failure Synthesis):** DomainForge no-D failure envelopes emit `conformance_status: "conformant"`, `validation_status: "validated"`, omissions record for `representation_unavailable`, and `extensions.domainforge.model_validation_status: "invalid"`.
- **D-04 / F-04 (Preimage Identity):** DomainForge `DomainModelIdentity` and SEA-Forge `DomainModelRef` versioned as `v2-full-preimage`, binding D's `self_hash` and `semantic_closure_hash`. Old records deserialize with `unknown-pre-versioning`.
- **D-05 / F-05 (G1 Registration):** SEA-Forge registers `godspeed.event.v1-flat` profile (digest `sha256:a2b2722008e920d0e74b3970b427b0b2e3e5b323c9321ef9a8f4c017d29162eb`) with explicit condition-based sunset statement.
- **F-06 / F-07 (Subprocess Safety & Composition Proof):** SXR subprocess execution bounded by 16 MiB stdout/stderr buffers and 30s timeout. Live composition verified against real `domainforge` binary via `real_binary_composition_and_settlement_authorization`.

**Date:** 2026-09-07 (Updated: 2026-09-08)  
**Primary context read first:** `sea-rs/docs/explanations-and-references/goodspeed-loop.md`

## Executive verdict

The current repositories do not expose one composed, production-ready semantic-envelope interface.

Three legitimate but insufficiently distinguished artifacts are all called a “semantic envelope”:

1. **D — DomainForge canonical semantic document**, schema `domainforge-semantic-envelope/v1`.
2. **E — CEP-0008 canonical full-profile envelope**, with explicit identity, boundary, completeness, omission, provenance, and lineage fields.
3. **G — Goodspeed cross-component event envelope**, implemented on the older `sea.agent.event.v1` flat shape.

That name collision concealed four material failures:

- SXR targets a CEP-emitting DomainForge **Slice 0B worktree that is not in the current DomainForge main lineage**. Current DomainForge rejects SXR's required flags.
- SXR collapses “D exists” into “model is valid.” It converts an explicitly invalid model into a `Resolved` value whose API reports `valid`, and snapshot creation drops its diagnostics.
- One semantic model has incompatible “model hashes”: authored-source bytes on the Goodspeed wire, D content/closure hashes in SXR, and an adapter-input tuple in sea-rs.
- G is valid against sea-rs's local flat profile but not against the current CEP canonical full-profile schema. sea-rs accurately calls its flat projection only partially conformant; other surfaces do not always name the profile.

The system is therefore **not settled as an integrated DomainForge → SEA-Forge/SXR → Goodspeed semantic contract**, despite green local suites and a green convergence status projection. These are new counterexamples under the frozen preregistration's reopen rule: downstream composition exposes violations and the semantic contract differs across repositories.

## Scope and evidence basis

### Revisions inspected

| Repository/worktree | Revision |
|---|---:|
| DomainForge current checkout | `2fdbe48c4de7` |
| DomainForge Slice 0B worktree | `7d5ba6e1c407` |
| SXR / RealityTrace | `c588968e085c` |
| sea-rs / SEA-Forge | `72348237412d` |
| CEP | `cadc9d7e86a5` |
| SWE_SEED | `3b1a195185a9` |
| SEA semantic-source repository | `552ea0a9f0cd` |

The SXR and sea-rs worktrees already contained unrelated operator changes. This investigation did not alter them.

### Methods

- Read the Goodspeed explanation, frozen preregistration, implementation sources, tests, ADRs, schemas, and repository instructions.
- Compared Git ancestry and history for the DomainForge producer contract.
- Inspected live `domainforge envelope --help` output from both worktrees.
- Executed the exact SXR-style invocation against current DomainForge.
- Emitted Slice 0B envelopes from real `.sea` inputs and validated them with SXR's CEP schema.
- Validated a Goodspeed fixture against both the sea-rs flat profile and CEP full profile.
- Re-ran `just e2e-prereg-check` and `just e2e-delta-check` in sea-rs.

## Contract map

| Boundary | Actual artifact | Consumer | Current state |
|---|---|---|---|
| `.sea` → semantic representation | D: `domainforge-semantic-envelope/v1` | sea-rs in-process adapter | Reachable |
| `.sea` → CEP semantic snapshot | E containing/referring to D | `sxr-df` subprocess adapter | Producer exists only on divergent Slice 0B |
| Goodspeed E4/E6/E7/E8 | G: flat v1 event | next loop component | Locally compatible; not full CEP |
| Goodspeed domain identity | untyped bare hash | equality/shape gates | Not one DomainForge identity algorithm |
| CEP conformance | Draft CEP-0008 + full schema | SXR validates E; sea-rs separately validates flat G | Split profiles; no shared release contract |

## Findings

### F-01 — Critical: SXR invokes a DomainForge interface absent from the current product line

**Evidence**

- `sxr/sxr-df/src/adapter.rs:1-17` explicitly targets “Slice 0B.”
- `adapter.rs:100-135` always constructs:

  `domainforge envelope --emit both [--pack ...] [--default-namespace ...] --scope <json> <entry>`

- It requires exit 0 or 1, JSON on stdout, and either `{representation, cep_envelope}` or a bare failure E (`:124-155`).
- Current DomainForge accepts only one positional target (`domainforge/domainforge-core/src/cli/envelope.rs:9-18`) and emits D only on success (`:20-65`). It has none of SXR's flags or failure-envelope behavior.
- Live current-binary execution returned exit 2: `unexpected argument '--emit' found`.
- Slice 0B exposes exactly the expected flags and shapes (`domainforge-slice0b/domainforge-core/src/cli/envelope.rs:20-60`, `:74-243`, `:246-354`).
- The branches diverge after `c8f7300`; Slice commits `f2e9f59` and `7d5ba6e` are not ancestors of the current checkout.

**Exact cause**

SXR committed a consumer against a worktree-only producer extension without first promoting it into DomainForge's maintained line or pinning a released compatible binary. Fixture and scripted-process tests froze the prototype contract but did not require ordinary current DomainForge.

**Impact**

Normal SXR snapshot creation cannot compose with current DomainForge. It fails before semantic evaluation.

**Recommendation**

Reconcile Slice 0B into maintained DomainForge as a versioned public interface. Preserve `envelope <target>` for compatibility and add an unambiguous surface such as `envelope cep` or `--contract cep-0008-full/0.1`. Make SXR verify a machine-readable capability/version response. Do not make an anonymous worktree the production dependency.

### F-02 — Critical: SXR launders an invalid DomainForge model into `valid`

**Evidence**

- Slice 0B correctly separates an invalid represented model from a valid carrier when D exists: E has top-level `validation_status: validated`, while `extensions.domainforge.model_validation_status` and `representations[0].validation_status` are `invalid` (`domainforge-slice0b/.../application/envelope.rs:1376-1397`, `:1537-1540`).
- A live Goodspeed-model emission had zero full-profile schema errors with exactly that state.
- SXR maps any such E to `SemanticContext::Resolved` (`sxr-df/src/adapter.rs:187-207`), explicitly “whatever the diagnostics say” (`:1-14`).
- `SemanticContext::model_validation_status()` maps every `Resolved` to `valid` (`sxr-core/src/types.rs:173-177`).
- The fixture test preserves the invalid fields but asserts the resulting context is `valid` (`sxr-df/tests/fixture_tests.rs:113-135`).
- `snapshot_create` hard-codes `valid` and an empty diagnostics array for every `Resolved` (`sxr-cli/src/commands.rs:208-255`), then the settlement guard trusts that status (`sxr-core/src/threshold.rs:95-126`).
- Replay projection separately reads the preserved DomainForge extension and derives `invalid` (`sxr-cli/src/projections.rs:415-460`), so immediate and replayed state can disagree.
- SXR architecture documentation says model failure must return `ValidationFailed` (`docs/architecture/logical.md:75-78`), while the subsystem code actually uses that variant to mean D unavailable.

**Exact cause**

The two-variant sum models **representation availability**, but its names and helpers model **semantic validity**. These are independent axes. CEP top-level validation is envelope validation, not represented-model validation.

**Impact**

An invalid model can be stored/exposed as valid, diagnostics can vanish, and positive-settlement gates can consult the wrong state.

**Recommendation**

Use independent typed states:

- `representation: Available { D, content_hash, semantic_closure_hash } | Unavailable { omission, failed_input_hash }`
- `model_validation: Valid | Invalid { diagnostics } | NotEvaluated`
- `envelope_conformance: Conformant { profile, schema_hash } | NonConformant { violations } | NotEvaluated`

All command, ledger, replay, projection, query, and settlement paths must consume the same admitted state. Invalid-with-D must remain **available + invalid**.

### F-03 — High: DomainForge failure E conflates CEP conformance with model validation

**Evidence**

- CEP's schema says top-level `validation_status` is envelope validation and subject/model validation is distinct; `conformance_status` is conformance of E to CEP (`cep/schemas/cep-semantic-envelope.schema.json:576-590`; spec §38 at `:1080-1102`).
- Slice 0B sets `conformance_status: non_conformant` when DomainForge cannot construct D from invalid inputs (`domainforge-slice0b/.../application/envelope.rs:1399-1431`).
- The emitted no-D failure E is nevertheless structurally CEP-valid: the live validator returned zero errors, an explicit omission, and model status invalid.

**Exact cause**

The producer uses envelope conformance as a discriminator for the represented model. But E can be CEP-conformant precisely because it faithfully reports that D is unavailable.

**Recommendation**

DomainForge must validate E against the pinned CEP schema before emission and record that result separately. Put DomainForge model validation in a typed extension or representation-validation record. A complete no-D failure envelope should be conformant even though its represented input is invalid.

### F-04 — Critical: “domain model hash” does not identify one thing

**Evidence**

For the same Goodspeed source model:

- The SEA manifest's `sea_file_hash` is the bare SHA-256 of authored `agentic_capability_loop.sea`: `62f4f0cd...bbcb565`.
- Current DomainForge emits D with a different `self_hash` (`sha256:7e81a8ac...d9694`) and `semantic_closure_hash` (`sha256:7e5bf147...9b920`).
- SXR verifies and persists D content and closure hashes (`sxr-df/src/adapter.rs:210-255`, `:463-493`).
- sea-rs obtains D in-process (`sea-rs/crates/sea-forge-domainforge/src/lib.rs:242-265`) but computes `DomainModelRef.semantic_model_sha256` from DomainForge version, adapter descriptor, parse options, and source refs (`:293-316`), without D's self or closure hash.
- The adapter descriptor is a fixed placeholder-like constant ending `...0001` (`:83-85`).
- Goodspeed boundary APIs accept raw `&str local_model_sha256` (`sea-rs/.../governed_work_ingress.rs:276-295`). SWE_SEED proves digest shape/equality but not DomainForge provenance (`SWE_SEED/.../identity.rs:119-162`; `.../consume.rs:143-172`).
- Whole-loop and SXR convergence tests use hashes of test strings rather than a DomainForge-produced identity (`sea-rs/.../convergence_t11_whole_loop.rs:129-155`; `sxr-core/tests/convergence_t07_proof_ingestion.rs:29-38`).

**Exact cause**

Each repository independently chose a digest for a local concern, then reused the generic name `domain_model_hash`. Hash equality proves equality only for a named algorithm over a named preimage.

**Impact**

All components can agree on an arbitrary 64-hex value without proving a relationship to D. Source formatting can change one identity without semantic change, while interpretation-version changes may not change it. SEA-Forge and SXR records cannot be joined reliably.

**Recommendation**

Define one typed, versioned `DomainModelIdentity`, produced by DomainForge core, containing:

- producer and exact DomainForge version;
- language, interpretation, and canonicalization versions;
- source-set hash;
- D content hash;
- semantic-closure hash;
- pack-set and registry hashes where applicable;
- schema/profile identifiers and hash algorithms.

Use semantic closure as semantic identity only after defining its equivalence contract. Use D content hash to resolve exact bytes. Keep source/config hashes as provenance. Runtime gates must accept a verified type, not raw hex, and migrate existing bare hashes explicitly.

### F-05 — High: Goodspeed G is a partial flat profile, not CEP's full profile

**Evidence**

- The frozen preregistration defines `GodSpeedSemanticEnvelope` `godspeed.semantic-envelope/v1` with explicit `envelope_id`, structured `producer`, `correlation`, and `domain` including model version (`sea-rs/.agents/specs/e2e-preregistration.yml:213-279`).
- Implemented SWE_SEED G uses the closed flat v1 shape and moves namespace/hash into payload (`SWE_SEED/.../federation/envelope.rs:1-9`, `:37-60`, `:200-228`). It has no model version.
- sea-rs E6 emits the same flat shape (`sea-rs/.../governed_settlement_return.rs:735-752`).
- SXR E7/E8 deliberately mirror it (`sxr-core/src/proof_ingest.rs:1-16`, `:35-65`; `evidence_emit.rs:1-20`, `:41-57`, `:357-369`).
- sea-rs's CEP adapter explicitly says this is only partially conformant and cannot claim full CEP-0008 conformance (`sea-rs/crates/sea-forge-extension/src/cep0008.rs:1-11`). CEP concepts live in opaque metadata (`:199-239`).
- A real G fixture has zero errors against the sea-rs flat schema and 15 errors against the CEP/SXR full schema.

Schema digests:

| Schema | SHA-256 |
|---|---|
| CEP/SXR canonical full profile | `4b42bfdd...8b5b17` |
| sea-rs v1 flat profile | `a2b27220...9162eb` |
| DomainForge D schema | `f0577cd0...a0faf3` |

**Exact cause**

Convergence work reused the smallest existing flat wire but never established a normative, loss-detecting mapping that preserves every frozen/CEP field. Conceptual preregistration fields, flat wire fields, and full CEP fields became interchangeable in prose and hand-written validators.

**Recommendation**

Choose one honest design:

1. **Preferred:** define G2 as a registered CEP full-profile family. Preserve explicit boundary/completeness/omission/provenance and use a governed extension or appropriate CEP sections for event-specific content.
2. **Compatibility option:** register G1 as an explicitly partial profile with a unique `$id`, exact schema/profile version and digest, normative mapping, declared omissions, and a loss-detecting G ↔ E projection. Never call it full CEP conformance.

Use a new wire version with dual-read/single-write migration. Do not edit the frozen preregistration in place; evaluate a CandidateRevision.

### F-06 — High: current tests prove fixtures and local gates, not composition

**Evidence**

- SXR process tests use a scripted fake DomainForge and replay fixtures (`sxr-df/tests/process_tests.rs:1-5`, `:43-112`).
- Its real-binary tests require `SXR_TEST_DOMAINFORGE_BIN`, explicitly name Slice 0B, and are ignored by default (`real_binary_tests.rs:1-7`, `:26-58`).
- sea-rs's whole-loop test stamps a constant `DOMAIN_HASH` and sets `domainforge_candidate: None` (`convergence_t11_whole_loop.rs:105-155`).
- SXR E7/E8 tests hash a literal test string rather than DomainForge output (`convergence_t07_proof_ingestion.rs:29-38`; `convergence_t08_evidence_emission.rs:36-43`).
- `just e2e-prereg-check` passes and `just e2e-delta-check` reports 35/35 confirmed because those gates validate the ruler and evidence/status map, not current DomainForge → SXR composition.

**Exact cause**

Repository-local fixture contracts omitted the one mandatory proof: a current producer must feed current consumers through the intended path. This is an existence-versus-composition failure.

**Recommendation**

Create a shared versioned conformance kit and make real-binary compatibility mandatory for DomainForge, SXR, and sea-rs releases. Generate identities/envelopes from source rather than only replaying fixtures. Reopen affected convergence claims through governance; preserve prior evidence and the frozen ruler.

### F-07 — Medium: SXR's process contract is incomplete and unbounded

- `SemanticContextRequest.registry_path` exists but is advisory and never reaches argv (`sxr-df/src/adapter.rs:37-61`, `:100-121`).
- `Command::output()` has no timeout, cancellation, stdout/stderr bound, or producer-version check (`:118-134`).
- Scope JSON rides in argv, exposing it to OS size limits and process inspection.
- Large D carriage uses `--emit both` as a bootstrap substitute for a defined cross-process CAS contract.

**Recommendation**

Implement or remove registry-path support; add capability negotiation, bounded output, timeout/cancellation, and typed exit classes; send structured requests over stdin or a governed descriptor; specify CAS ownership, layout, atomicity, size limits, and missing-object behavior.

### F-08 — Low: documentation and release drift amplify ambiguity

- `sea-rs/docs/subsystems/domainforge-boundary.md:16-43` says DomainForge 0.15.0; source and Cargo pin 0.16.0.
- CEP-0008 and SXR's schema mapping are Draft. DomainForge and CEP carry byte-identical CEP-0008 text; sea-rs differs only by a trailing newline. No approved protocol package is pinned by all runtimes.
- Slice 0B constructs E but does not run the full-profile validator; SXR is the first validator.

**Recommendation**

Pin a released CEP package by version and schema digest, generate validators/bindings from it, and advertise supported profiles through capabilities. Avoid copied normative specs where practical.

## Root-cause chain

1. D, E, and G acquired the same informal name.
2. sea-rs chose an in-process DomainForge boundary, SXR prototyped a richer CLI on Slice 0B, and Goodspeed reused a flat event wire.
3. Each repository invented a local identity algorithm.
4. D availability, model validity, envelope validity, and CEP conformance were collapsed into two variants and string fields.
5. Fixture-centered verification proved each local interpretation without composing current producers and consumers.
6. The status projection settled because the omitted cross-repo path was outside its executable closure.

## Recommended target architecture

```text
.sea source set
    ↓
DomainForge core (semantic authority)
    ├─ D: canonical semantic document
    ├─ DomainModelIdentity: source + D content + closure + versions
    └─ ModelValidation: valid | invalid | not-evaluated
             ↓
CEP projection + pinned validator (representation authority only)
    └─ E: conformant semantic_snapshot; D inline or content-addressed
             ↓
       ┌───────────────┐
       ▼               ▼
SEA-Forge          RealityTrace/SXR
in-process D       E ingestion/evidence binding
authority and      never truth/settlement authority
operational settlement
       └──────┬────────┘
              ▼
Goodspeed G2 event profile references the same typed identity and E/D
```

Authority remains separated:

- **DomainForge:** semantic parsing, resolution, model validation, and model-identity production.
- **CEP:** representation/profile conformance, never truth, authority, proof, or settlement.
- **SEA-Forge:** governed execution authority and operational settlement.
- **RealityTrace/SXR:** expected-versus-observed evidence binding, never developmental settlement.
- **SWE_SEED/Goodspeed federation:** event production and causal transport, not semantic identity invention.

## Implementation sequence

### Phase 0 — Governance and naming

1. Open a cross-repo CandidateRevision; preserve frozen Goodspeed artifacts and prior evidence.
2. Reserve `DomainSemanticDocument` (D), `CepSemanticEnvelope` (E), and `GodspeedEventEnvelope` (G/G2).
3. Select an approved CEP-0008 revision/profile and pin its schema digest.
4. Reopen affected identity/envelope/integration claims pending new evidence.

### Phase 1 — Canonical DomainForge producer

1. Reconcile Slice 0B with current DomainForge main.
2. Expose a pure core API returning D, typed identity, model validation, and CEP projection input.
3. Add a thin versioned CLI plus machine-readable capabilities.
4. Self-validate E before emission.
5. Correct no-D E semantics so faithful model-failure reporting can remain envelope-conformant.

### Phase 2 — Repair SXR semantics

1. Split representation availability, model validation, and envelope conformance.
2. Preserve diagnostics/status through append, replay, projection, query, and settlement.
3. Migrate rows whose status was inferred from `Resolved`.
4. Add subprocess bounds and capability checks.

### Phase 3 — Unify sea-rs identity without weakening ADR-001

1. Keep the side-effect-free in-process integration.
2. Extend `DomainModelRef` with D content/closure hashes and version/provenance inputs.
3. Replace the placeholder adapter descriptor with reproducible identity.
4. Require verified `DomainModelIdentity` at Goodspeed boundaries.
5. Keep SXR and the DomainForge CLI outside SEA-Forge's authority path.

### Phase 4 — CEP-conformant Goodspeed transport

1. Define G2 as full CEP or a registered, explicitly partial profile.
2. Preserve model version, boundary/completeness/omission, causality, provenance, and correlation as typed wire data.
3. Dual-read G1/G2 and single-write G2 during a bounded migration.
4. Make projections loss-detecting and record source profile/schema digest.

### Phase 5 — Shared closure suite

Run common vectors against DomainForge core and released CLI, SXR, sea-rs, SWE_SEED/SEA-Forge/SXR event edges, and the exact CEP validator. Release requires real production composition; fixture/fake-binary tests remain unit evidence only.

## Required acceptance matrix

| Case | Required result |
|---|---|
| Valid source, D available, model valid | E conformant; SXR available+valid; sea-rs/SXR share typed identity |
| D available, model invalid | E conformant; D retained; invalid everywhere; diagnostics retained; positive settlement blocked |
| D unavailable | E conformant failure record with omission and failed-input identity; no fabricated D hash |
| E malformed | producer refuses or consumer rejects as envelope failure, never model failure |
| D/hash tampering | rejected before admission |
| source/pack/registry/version drift | typed identity mismatch; no silent equivalence |
| G1 at G2 consumer | only declared legacy adapter; remains labeled partial |
| current DomainForge with SXR | non-ignored real-binary inline/CAS/valid/invalid-with-D/no-D suite passes |
| whole Goodspeed loop | actual DomainForge identity traverses E1–E10; no test-string substitute |
| timeout/oversize output | bounded typed failure |
| conformant false claim | no automatic truth, evidence, authority, or settlement consequence |

## Reproducible observations

```text
Current DomainForge:
  Usage: domainforge envelope [OPTIONS] <TARGET>
  exact SXR call: unexpected argument '--emit'; exit=2

Slice 0B invalid-with-D:
  full CEP schema errors=0
  envelope validation=validated
  DomainForge model=invalid
  representation=invalid

Slice 0B no-D:
  full CEP schema errors=0
  envelope conformance claim=non_conformant
  DomainForge model=invalid
  representations absent; one omission

Goodspeed E4 fixture:
  sea-rs flat-profile errors=0
  CEP/SXR full-profile errors=15

sea-rs governance:
  just e2e-prereg-check: PASS; frozen hash intact
  just e2e-delta-check: PASS; 35 confirmed, open delta 0
```

The final PASS results prove ruler and status-ledger consistency. They do not rebut the live cross-repo counterexamples.

## Decision summary

The smallest durable repair is not another translation shim. Promote the demonstrated Slice 0B CEP projection into DomainForge's maintained API, correct SXR's independent state axes, bind sea-rs and Goodspeed to DomainForge's typed model identity, and give Goodspeed an explicit CEP profile/version. A mandatory real-producer cross-repo suite must replace fixture agreement as settlement evidence for interface compatibility.

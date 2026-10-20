# Proposal: RealityTrace E8 completeness and observer attribution

Round-4 item (operator decision 2026-10-18: "propose a path"). Two open
limits from ADR-012's obligations, both rooted in the pinned `sxr-core`
dependency (RealityTrace main 42879a8), so neither is changeable from this
repository alone:

## 1. E8 field-level differences bounded at 256 entries

`record_evidence` → `ingest_from` emits the E7-vs-E6 comparison with at most
256 field-level difference entries. A comparison that exceeds the bound is
indistinguishable, in the emitted record, from one that merely has 256
differences: completeness cannot be asserted and a reviewer cannot tell
truncation from exhaustion.

Proposed path, in two steps:

1. **SEA-Forge side (implementable now):** treat `differences.len() == 256`
   as "bounded" rather than "complete". `record_evidence` refuses, before
   anything is emitted, any E8 comparison whose difference set hits the
   bound, with a typed error that names the record and the bound. There is
   no "flag and emit" option for v1: the v1 record has no completeness
   field, so a flagged record would reach verifiers indistinguishable from
   a complete one. This turns silent truncation into a loud, attributable
   refusal and keeps the frozen v1 producer contract intact (no schema
   change, only a refuse-on-bound policy at the emission site).
2. **sxr-core side (CEP-owned change):** extend the comparison object with
   `differences_digest` (a digest over the canonical, ordered full
   difference set) and a `complete` boolean. SEA-Forge pins the new commit
   and then accepts large comparisons again, asserting
   `differences_digest` verifiability instead of entry-count exhaustion.

## 2. Ledger observer attribution is not a signature (ADR-012 obligation 4)

Observation records attribute to an observer identity string only. Obligation
4 of ADR-012 requires attribution that cannot be forged by whoever writes the
ledger entry.

Proposed path: an observer signing key pair per observer identity (issuance
alongside the existing `sea_forge_ledger::signing` key lifecycle — generated
from committed entropy, never persisted outside the owner-only state tree,
rotatable), with each observation record carrying
`observer_signature = sign(observer_id || record_digest)`. A signature only
proves possession of a key, so ingest resolves `observer_id` to its public
key through a trusted registry that the ledger writer cannot alter (keys
pinned in the operator-signed authority bundle, with rotation recorded
there, never read from the ledger being verified). Ingest verifies the
signature against that key before a record participates in any E6/E7
comparison; an unattributable record is refused, not downgraded.

Compatibility with records written before signatures exist is an explicit,
versioned rule, not a silent exception:

- Signed records carry a new record version (v2); the v1 shape is
  unchanged.
- The registry records a cutover ledger position per observer. v1 records
  before the cutover keep verifying under the frozen v1 rules and are
  reported as `unattributed (pre-signature)`; they never satisfy
  obligation 4.
- Any v1 record at or after the cutover is refused, so a ledger writer
  cannot sidestep signing by emitting the old shape.

Rollout is its own task: key lifecycle, rotation semantics, the registry and
the ingestion-side verification each need their own tests and mutation
controls.

Sequencing: step 1.1 is in-scope for round 4's closeout only if the operator
wants the refuse-on-bound policy before the re-attack; the sxr-core changes
and the signing work are follow-ups on the RealityTrace and ledger
roadmaps respectively.

# Ask live ledger projection clarification

Root source finding, 2026-10-01. This supplements the original Ask runtime assignment and
root integration supplement without rewriting them.

Canonical ClaimView has eight required fields and one optional capability_record_ref
(`crates/sea-forge-server/src/sfwp/thoth.rs:17-35`). The persisted GroundedClaim additionally
contains flags, claim-level limitations and optional authored_by
(`crates/sea-forge-thoth/src/protocol.rs:91-114`). Existing kernel conversion intentionally
projects only the ClaimView fields (`sfwp/thoth.rs:79-105`). Do not expand the approved contract.

The live proof's API-to-ledger equality means equality after applying that actual canonical
kernel projection to each committed claim, retaining all eleven top-level answer fields.
It must assert every projected field and optional-reference presence, not compare unlike raw
claim objects literally or erase arbitrary differences until equality passes. Record the three
existing ledger-only claim fields as a material evidence-shape difference. No fabricated answers,
references or defaults are permitted. Ledger links and effective writer attribution are still
verified against the actual persisted entries, independently of this view projection.

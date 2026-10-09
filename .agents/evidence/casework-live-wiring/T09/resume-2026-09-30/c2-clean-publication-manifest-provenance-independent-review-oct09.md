# Independent review: recovered manifest provenance

Date: 2026-10-09

## Scope and checks

This review covers only recovery of the previously reviewed candidate manifest and its CW-42 provenance note. It does not grant publication clearance.

- The frozen `c2-clean-publication-reviewed-manifest-24765dc5-oct09.json` is 134,928 bytes, valid JSON, and has SHA-256 `24765dc564ba0449ea778d5f20ac4eb870b9db0de9ae2b3d4926ccff3d769553`.
- Compared with the unchanged mutable draft (`d39c8fd74a3b275670493dbca186e70287ad6ccbab8655b87ad6647c9cfbbbed`), the frozen artifact has the same top-level structure, 544 unique path entries, and identical non-file metadata. Exactly one entry differs: `.agents/DEBT.md`, whose reviewed metadata is 132,368 bytes / `3fefbd83657a3d363bb1948524636fabc512ba8921a14e739969b1a92429af43` and whose draft metadata is 132,567 bytes / `b174a5cae98b19db985702c0079eaddb7922d8c9929f2d5de89e1a6358d7ad47`. The other 543 entries are identical.
- The provenance note records that only the DEBT metadata substitutions were reversed, provides the full recovered hash, distinguishes hash recovery from runtime evidence and publication clearance, and says later final metadata belongs in a separate artifact.
- The added CW-42 subsection identifies the reconstructed artifact and provenance note, says the hash proof is not a new runtime capture or publication clearance, and leaves CW-42 open with required gates and final disposition pending.

## Verdict

**PASS for exact recovered-manifest identity and bounded provenance wording.** This verifies the reviewer-cited manifest bytes and the declared one-entry metadata difference. It does not independently approve candidate privacy, final manifest freshness, later status/capture additions, gates, or publication. CW-42 and the overall publication hold remain open.

# Independent review: manifest provenance attribution correction

Date: 2026-10-09

## Scope and verification

This review covers only the attribution addendum and its CW-42 insertion. The recovered manifest and original provenance note remain immutable.

- The recovered manifest remains 134,928 bytes with SHA-256 `24765dc564ba0449ea778d5f20ac4eb870b9db0de9ae2b3d4926ccff3d769553`. The mutable draft remains 134,928 bytes with SHA-256 `d39c8fd74a3b275670493dbca186e70287ad6ccbab8655b87ad6647c9cfbbbed`; the earlier review established the exact one-entry DEBT metadata difference.
- The original provenance note remains 1,280 bytes with SHA-256 `9bfee43b3141f97e0682da1e7af7bf5256577bce336dc84e168e9a0b2b6dcd55`. The new 865-byte addendum has SHA-256 `eacd47588dc80df355cb0a2dfade45f01ea53922513a562385b7d878574d275d`.
- The addendum supersedes only the unsupported attribution that the before/after DEBT metadata values were recorded in the earlier review. It attributes those values to the separate bounded DEBT repair/recovery proof, gives the exact recovered manifest hash and length, and explicitly limits that proof to byte identity rather than runtime evidence or publication clearance.
- CW-42 contains one additive attribution subsection that points to the addendum, preserves the no-runtime/no-publication-claim limitation, and leaves CW-42 open. Removing exactly that subsection from current `DEBT.md` reconstructs the previously reviewed file byte-for-byte: 133,378 bytes and SHA-256 `1e84544aad8d7bd608dd6ee35c26e754d4b79748f5db53d056e30535aafd01cb`.

## Verdict

**PASS for the bounded attribution correction.** The earlier attribution defect is corrected in a separate immutable addendum, and the only DEBT change is the matching CW-42 subsection. This does not approve candidate privacy, a final manifest, required gates, or publication; those remain pending.

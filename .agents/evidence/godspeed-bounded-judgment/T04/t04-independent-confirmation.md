# T04 Independent Adversarial Confirmation Record
- Date: 2026-09-17
- Verifier: fresh independent context; builder narrative excluded
- **Verdict: CONFIRM**

Independent verification performed by the verifier (all in a /tmp worktree, since removed):
- Prereg sha256 verified; battery re-run 8/8; sea-forge-authority 62/62.
- Structural ordering proof: authority decision committed before verdict check; provider contact last; single Accepted construction site gated by two committed allows; policy bundles force deny-by-default with closed operation_kind whitelist and deny_unknown_fields.
- Seven NEW independent attacks, 7/7 behaved fail-closed:
  v1 secret gate bypass attempt (rejected, 0 contacts); v2 engine-level vocabulary minting via Unclassified/Reserved (denied by the live engine); v3 prompt-injection under an ALLOWED dispatch (accepted but the directive never re-enters any record; wall-clock ordering proves decision preceded dispatch); v4 malformed/weakened policies (refused at load); v5 host widening outside allow_hosts (rejected); v6 missing schema fields (typed schema failure); v7 traversing policy paths refused (UnsafePath).

Defects found: none reachable from provider output.
Residual notes recorded as debt (not T04 failures):
  1. grant.authorize failure after the allow is committed propagates Err with no settlement (allowed-but-unsettled stranded state; inverse of the claim under test).
  2. GovernanceDisposition::Boundary|Degraded => Allow is policy-declared only (bundle-gated) and unreachable from the probe path; future lens if an executor for a boundary-disposition probe lands.
  3. Builder battery weaknesses closed by the verifier: t04_8 is deserialization-only (engine-level denial proven by verifier v2); t04_6 lacked an ordering proof (added by verifier v3).
  4. Verifier's HEAD-worktree sfwp run showed a 4th failure (generated_schemas_are_committed_and_current) consistent with T02's scratch-worktree environment observation; not present in the real repo's recorded gate.

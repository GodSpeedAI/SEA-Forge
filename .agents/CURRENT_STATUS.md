# Current status

**Status revision:** 392

**Stage:** CognitiveOS #108 hardening review fixes

**Summary:** Harden request admission and status: no existence oracle, blocking I/O off the executor, bounded audit appends, distinct reason codes.

**Verified:**

- A non-owner without the audit grant receives the never-seen reply, including for orphan and existing ids; owners and audit-granted actors keep their replies (conformance_sfwp 26/26).
- Claim lookup/record and audit authorization run in spawn_blocking; repeated denied polls append one audit decision (test red before, green after).
- Legacy hashed records without a claim fail closed with reason_code legacy_unclaimed, orphan claims with orphan_claim; an unwritable correlation record after the claim replies no_side_effect, reason_code request_id_spent and new_request_id_required.
- The crash test now uses a committable case_commit, asserts case, run and claim-ledger counts (claim exactly 1) and proves committability with a control id; lib 205/205 earlier plus admission_crash 3/3 after the final edit; conformance_case_authoring 21/21, conformance_lifecycle 10/10, FDE scenarios 49/49 with FDE_WP21A_FINAL=1, contract digests match, clippy -D warnings and fmt clean.

**Limits:**

- The audit policy is still parsed per non-owner poll because policy edits must take effect without a restart; only denied audit appends are deduplicated, granted polls still append because the grant proves against its committed decision.
- request_get_status still consumes the admission pool and legacy records still reply denied to everyone.
- Power-loss durability, non-Unix behaviour, full CI and proofs have not run.

**Next:** Finish the remaining hardening-review fixes, then run the full gates.

**Evidence:**

- crates/sea-forge-server/src/lib.rs
- crates/sea-forge-server/src/request_admission_claim.rs
- crates/sea-forge-server/src/admission_crash_tests.rs
- crates/sea-forge-server/tests/conformance_sfwp.rs
- docs/subsystems/server-sfwp.md

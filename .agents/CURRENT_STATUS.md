# Current status

**Status revision:** 192

**Stage:** Security: approval, single-use claim, and recoverable resume for settlement pending records

**Summary:** Approval-gated external settlement is now resolved, claimed, and recovered from the ledger. A settlement_pending record is approved, rejected, or expired through the CEP approval path, with self-approval and non-operator resolution refused and the approval bound to pending_id, request_sha256, operation and class. An approval is spent by one create-only claim, so a second use is refused and audited. The state machine (pending, approved, resuming, completed, rejected, expired, failed, released) is derived from ledger records only. Reconcile settles an open claim from the declaration in the case ledger, or fails it, never declaring twice. Identical racing requests return one pending record.

**Verified:**
- cargo test -p sea-forge-authority: 265 pass, 0 fail (cep_settlement: 43 pass, including approve-once resume, reject, expire, self and non-operator refusal, mismatched and forged approval, reuse refused, four-thread resume race, crash recovery, release, racing identical requests)
- cargo test -p sea-forge-server: 550 pass, 0 fail
- cargo test -p sea-forge-cli --no-fail-fast: 87 pass, 2 fail (the two conformance_m8 CLI tests, not rewritten)
- cargo clippy --all-targets -- -D warnings: clean
- cargo fmt --all -- --check: clean

**Limits:**
- conformance_m8_cli_capitalize_commits_one_pending_and_approval_then_exits_5 fails: resume is refused with default_deny for operator_local, because its policy has no rule that reaches an external settlement declaration. An operator escalation rule makes resume exit 5 with a settlement pending record, which the single case approval does not satisfy; the test expects one approval then success. Needs a policy or test decision; not fixed here
- conformance_m8_cli_parks_external_gate_profiles_without_fabricating_records fails for the same reason: resume exits 1 instead of 5
- Reconciliation runs only in the CLI resume path. Startup, read-time, and out-of-process paths are not enforced; the server delegation path records completion but does not reconcile an open claim
- The approver is checked by the approval_resolution policy rule, not by a fixed role; the brief asked for operator role, and the m8 policy uses R-SO
- Adversarial review not done; the new tests were written with the code, not red first

**Next:** Enforce reconciliation at startup, read time, and in the server path; decide how the artifact capitalize path satisfies an external settlement gate (a settlement approval step, or a documented case-approval binding) before the m8 tests are re-scoped.

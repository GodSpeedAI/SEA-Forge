# Current status

**Status revision:** 227

**Stage:** Security: ambiguous SWE_SEED declare keeps the claim; operator resolves uncertain claims (PR 84)

**Summary:** Once the SWE_SEED authority call has started, a failure keeps the claim lease, and the CLI exits 1 with settlement_authority_call_failed. A failure before the call starts settles at once, and an unavailable authority releases the claim. Reconciliation no longer fails a lapsed claim that has no declaration. That claim stays open (Resuming) and blocks resumes. An operator resolves it with `sea-forge resolve-uncertain <case> <approval> --attempt N` plus either `--declared <ref>` or `--not-declared <evidence>`. This is allowed only after the lease lapses, never by the requester (`self_approval_refused`), and only if the `approval_resolution` policy allows it. The close happens first, and its audit record is written once per attempt after the close. Main is merged in: the generated SFWP schemas now live in contracts/schema/ (PR 85); uncertain resolution refuses an empty declaration ref or evidence. Main is merged in again: the case-task inbox row is now CaseTaskApproval, with the approval-standing golden fixture (PR 87).

**Verified:**
- sea-forge-server lib delegation: new test passes
- Mutation M1 (ambiguous_after_call always false) by hand: the new test and the predicate unit test both fail; reverted
- cargo test on authority, settlement, cli, server: 993 passed, 0 failed

**Limits:**
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- --declared <ref> is not checked against a ledger declaration
- A failure inside declare before the transport spawns is held for an operator, which is conservative
- The CLI builds an Operator from --actor, the same local trust model as approve
- The Horizon policy renderer and the sea-rs fixture are not tied by a test (follow-up)
- The server test covers a nonzero exit only; the timeout variant is covered at the CLI
- On Unix a SWE_SEED command can outlive a server crash (no process-group containment tied to the server). An operator NotDeclared attestation after the lease lapses can then race an orphaned command's late declaration. Mitigation needs authority-side dedup on authorization_request_sha256 or containment; tracked as a follow-up.

**Next:** Merge PR 84 after review; follow-up issue for crash-orphan containment.

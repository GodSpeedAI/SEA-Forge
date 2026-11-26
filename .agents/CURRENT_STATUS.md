# Current status

**Status revision:** 398

**Stage:** CognitiveOS #108 receipt contract hardening

**Summary:** Wire the receipt verb to the verified projection with stable codes, fixed error text, a ledger-truthful effect flag and withheld stored receipts.

**Verified:**

- Tests now encode the contract: a first-call receipt on a final outcome succeeds for settled and rejected against the real ledger with exactly the 12 allowlisted keys and no evidence, question or prompt text; replay, nonfinal, integrity and reference failures carry distinct reason codes (red before: 5 of the new socket tests failed on the merged draft; green after, conformance_cep_authority 20/20).
- Corruption tests reach verification with a clean control: edited and edited-and-rehashed evidence and decision rows are refused receipt_integrity_failed; decision-row damage is caught after the settlement commit and reports no_side_effect false.
- Same-id replay is refused and request.get_status withholds the receipt outcome; refusals use fixed public text; an operation named cognate selects only its own records; the longest accepted operation id projects; a JSONL append followed by an MMR save failure reports no_side_effect false.
- Gates: lib 206, conformance_cep_authority 20, conformance_sfwp 26, conformance_case_authoring 21, conformance_lifecycle 10, FDE scenarios 49 (FDE_WP21A_FINAL=1), sea-forge-authority and sea-forge-ledger tests, contract digests, workbench contracts gate, clippy -D warnings on server, authority and ledger, fmt.

**Limits:**

- authority_settle_receipt is a CEP extension verb outside the SFWP method catalog and generated schemas, like every authority_* verb; the catalog and digests are unchanged.
- A receipt for an operation already settled through authority_settle is refused as a replay; clients must use the receipt verb for the first settlement.
- The Workbench Tauri host that builds sfwp_request_status is not in this repository.
- Chain and MMR are unsigned local files, the symlink check is check-then-open, and Windows native paths were not tested.
- Full CI, proofs and cross-platform tests have not run.

**Next:** Independent re-review of the receipt verb, then integrate.

**Evidence:**

- crates/sea-forge-server/src/cep_authority.rs
- crates/sea-forge-server/src/lib.rs
- crates/sea-forge-server/tests/conformance_cep_authority.rs
- docs/decisions/ADR-016-minimal-settlement-receipt.md

# Current status

**Status revision:** 194

**Stage:** Security: out-of-process provenance requirement at the settlement writer and every gate caller

**Summary:** Every gated external declaration carries its authorisation provenance (authorization_ref, the settlement pending id, and authorization_request_sha256), copied from the request and covered by declaration_hash. The settlement writer (append_declaration_ledgered and append_declaration_ledgered_once) refuses an external declaration without both fields, so an honest writer cannot produce an unauthorised record. An out-of-process append that bypasses the writer is still refused at reconciliation and read time by the shared verification. The server path, the CLI resume path, and the artifact transition pass the claimed pending id and request hash. Local declarations are unchanged.

**Verified:**
- cargo test -p sea-forge-server: full suite exit 0 (lib 178 pass; conformance_m16 19 pass, 2 ignored; out-of-process tests use a raw ledger append past the writer and are refused at startup and read time)
- cargo test -p sea-forge-settlement --lib: 18 pass (request authorisation carried into the hashed declaration; duplicate-declare test uses an authorised request)
- cargo test -p sea-forge-authority -p sea-forge-artifact-ip -p sea-forge-capability -p sea-forge-thoth: all pass
- cargo fmt --all -- --check: clean

**Limits:**
- Not yet done: the server path still declares an External Allow verdict that has no pending record; the writer refuses it after the external authority is contacted (checkpoint c)
- Not yet done: an existing matching declaration is reused by the artifact transition without a verification check (checkpoint c)
- Not yet done: conformance_m8 CLI tests (checkpoint d); race test (checkpoint e); adversarial review

**Next:** Checkpoint c: refuse an External Allow at the gate, claim an External pending record only when Approved, release or fail a claim when declare fails, reconcile open claims for the run before a new attempt, and verify a reused declaration.

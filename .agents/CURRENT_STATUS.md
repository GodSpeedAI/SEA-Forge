# Current status

**Status revision:** 228

**Stage:** Tests: pin fail-closed settlement classes for the Horizon gateway policy; log loaded world refs

**Summary:** With the Horizon gateway policy fixture, a settlement declaration whose class is missing or unknown is denied, local is allowed, and external requires operator approval (Horizon#19 is a non-issue; this pins it). cep_authority now logs each loaded world with its registered ref.

**Verified:**
- sea-forge-authority horizon_gateway_policy: a missing or unknown class is Deny, local is Allow, external needs approval
- the world-load log is emitted only after every world and prior identity registers

**Limits:**
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- --declared <ref> is not checked against a ledger declaration
- A failure inside declare before the transport spawns is held for an operator, which is conservative
- The CLI builds an Operator from --actor, the same local trust model as approve
- The Horizon policy renderer and the sea-rs fixture are not tied by a test (follow-up)
- The server test covers a nonzero exit only; the timeout variant is covered at the CLI
- On Unix a SWE_SEED command can outlive a server crash (no process-group containment tied to the server). An operator NotDeclared attestation after the lease lapses can then race an orphaned command's late declaration. Mitigation needs authority-side dedup on authorization_request_sha256 or containment; tracked as a follow-up.
- The world-load log is INFO level; SFWP has no way to list loaded worlds

**Next:** Merge after CI; close Horizon#19.

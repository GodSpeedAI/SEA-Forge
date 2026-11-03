# Current status

**Status revision:** 187

**Stage:** Security: identity check depends on resolution, not bundle version

**Summary:** The identity check in cep/authority evaluation now requires a resolved identity binding (identity_binding_hash present) and no longer also accepts the bundle version as a substitute. An unresolved identity on a 0.2 bundle is never allowed; a regression test pins this.

**Verified:**
- cargo test -p sea-forge-authority: 239 pass, 0 fail, 7 ignored
- cargo clippy -p sea-forge-authority --all-targets -D warnings: clean
- unresolved_identity_is_never_allowed_on_a_v02_bundle: regression pin (the prior code also refused this case; the test guards the gate's form, not a live exploit)

**Limits:**
- Not an exploitable bypass: resolve_identity leaves the hash unset only off RECORD_VERSION, and binding equality forces it to match
- Live SEA-Forge run of the self-approval path not yet performed
- Approval transition for settlement_declaration not yet in the production policy

**Next:** Add the settlement approval gate through a versioned spec amendment, then prove approve and reject against live authority records.

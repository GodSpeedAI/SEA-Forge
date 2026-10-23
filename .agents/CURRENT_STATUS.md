# Current status

**Status revision:** 157

**Stage:** deps-ctutils-advisory

**Summary:** cargo deny failed on main: ctutils 0.4.2 (via chacha20poly1305 in sea-forge-server) is affected by a RustCrypto constant-time advisory, fixed in >=0.4.4. Cargo.lock now pins ctutils 0.4.5. Lockfile-only change.

**Verified:**

- cargo deny check: advisories ok, bans ok, licenses ok, sources ok

**Limits:**

- No source change; the full test suite runs in CI

**Next:** Merge after CI passes.

**Evidence:** `Cargo.lock`

**Spec:** `.agents/specs/security-implementation-v2.spec.md`

**Ledger:** `.agents/OBSERVED_DEBT.md`

# Current status

**Status revision:** 158

**Stage:** deps-ctutils-advisory + workspace

**Summary:** cargo deny failed on main: ctutils 0.4.2 (via chacha20poly1305 in sea-forge-server) is affected by a RustCrypto constant-time advisory, fixed in >=0.4.4. Cargo.lock now pins ctutils 0.4.5. Lockfile-only change. Also adds ../Horizon to godspeedai-stack.code-workspace (the only content of local commit 6e92969 not already on main).

**Verified:**

- cargo deny check: advisories ok, bans ok, licenses ok, sources ok

**Limits:**

- No source change; the full test suite runs in CI
- The DomainForge write-authority engine is still a filename-stem approximation, not model-semantic authority (SUP-09h); readiness B-08 records it as a documented limitation

**Next:** Merge after CI passes. Replace the approximation with domainforge-core's authority evaluator when it exists; until then, do not enable engine: domainforge for production policy.

**Evidence:** `Cargo.lock`, `godspeedai-stack.code-workspace`

**Spec:** `.agents/specs/security-implementation-v2.spec.md`

**Ledger:** `.agents/OBSERVED_DEBT.md`

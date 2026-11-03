# Current status

**Status revision:** 184

**Stage:** Security: permission matrix enforced for every policy version

**Summary:** A declared permissions matrix is always enforced. The legacy grant applies only to a 0.1 policy with no matrix (spec 8.2). Before, a 0.1 policy with a matrix granted every role every operation.

**Verified:**
- cargo test -p sea-forge-server --lib: 174 pass (6 new: sentinel and root refusal, reload change, traversal chain, no traversal outside root or through .., shared-directory refusal, sticky and owner-only acceptance)
- cargo clippy -p sea-forge-server --all-targets -D warnings: clean
- cargo fmt: applied

**Limits:**
- Refusal, not a private staging directory: a socket directory other users can write (without sticky) now fails startup, which a deployment on such a directory will notice
- Traversal directories are 0711 (others may traverse, not list), matching the root's existing mode; the socket itself stays 0660 with the group
- Not yet verified on a live gateway: redeploy is the owner's step
- conformance_m16 t16_8 still fails on origin/main (pre-existing, needs sea-forge CLI binary)

**Next:** Commit, push horizon/socket-group-access, resolve the four threads on PR #73, then merge.

**Recorded at:** 2026-11-03T00:00:00Z

# Current status

**Status revision:** 139

**Stage:** security-round4-webpki-advisories

**Summary:** cargo-deny began failing on two advisories published after the previous push: RUSTSEC-2026-0382 (rustls-webpki 0.103.15 may accept distrusted emSign intermediates) and RUSTSEC-2026-0381 (webpki-roots 1.0.9 bundles the distrusted emSign ECC Root CA - G3). Both are bumped in Cargo.lock to the fixed releases; no advisory is ignored.

**Verified:**

- cargo update -p rustls-webpki -p webpki-roots; cargo deny check advisories via the pre-push hook

**Limits:**

- The 24.04 postgres-lifecycle refusal from revision 138 is still open; see revision 113 for the other round-4 open limits.

**Next move:** Read the pathname/dup/pin_and_verify debug lines from the next 24.04 postgres-lifecycle log.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md

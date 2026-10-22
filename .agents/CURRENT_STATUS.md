# Current status

**Status revision:** 154

**Stage:** corroborated-evidence

**Summary:** authority_evidence can corroborate each evidence item against RealityTrace's own ledger (cep_authority.realitytrace). With it, SEA-Forge reads the evidence.created record named by the item's sxr:<ledger>:<ordinal> provenance, requires the repo's hash chain to verify, and refuses any item whose evidence_id, type, question, representation, relevance, reliability or direction differs from what RealityTrace recorded. With require_corroboration (default), unsourced items are refused and authority_settle counts only packets committed as corroborated. Without the block, behaviour is unchanged and the response says corroboration: none. Also fixes the intermittent authority-opa CI failure (exit 141): `opa version | grep -q` under pipefail.

**Verified:**

- cargo test -p sea-forge-authority --test cep_settlement: 17 passed, including 5 corroboration tests (flipped direction refused and nothing settles; upgraded reliability and invented items refused; unsourced items refused when required; packets committed before the requirement never decide a corroborated settlement; recorded item without direction refused)
- sea-forge-server realitytrace_source unit tests: 3 passed against a stand-in realitytrace binary (recorded item returned by ledger position, broken chain / wrong record kind / unknown position refused, missing binary is an error)

**Limits:**

- RealityTrace's ledger is tamper-evident, not tamper-proof; on a single-user host the anchor is containment (gate S02)
- end-to-end proof with a real realitytrace binary comes with the Workbench settlement orchestrator (readiness SW01)

**Next move:** Workbench settlement orchestrator (readiness P03 WB-1/WB-2) and SW01 against a real realitytrace with this server.

**Evidence:**

- crates/sea-forge-authority/tests/cep_settlement.rs
- docs/how-to/corroborate-evidence-with-realitytrace.md
- ~/projects/godspeed-readiness/docs/proposals/P03-governed-settlement.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md

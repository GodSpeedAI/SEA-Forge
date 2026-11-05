# Current status

**Status revision:** 213

**Stage:** T1 stage D2.1: SFWP schemas moved to contracts/schema

**Summary:** The generated SFWP JSON schemas moved from workbench/packages/contracts/schema/ to top-level contracts/schema/ (git mv, files byte-stable). The generator output dir, system.get_schema paths, the TS generator, the drift test and live docs point at the new home. A conformance test asserts every path system.get_schema returns exists on disk.

**Verified:**
- cargo test -p sea-forge-server --test conformance_sfwp: 20 passed (19 before, plus get_schema_paths_exist_on_disk)
- get_schema path mutation fails the new test as expected
- just workbench-contracts-gate: clean after regeneration (TS banner path updated)

**Limits:**
- Historical logs under .agents (status archive, evidence, session export, run summaries, old plans) still name the old path by design
- crates/sea-forge-server/src/bin/README.md named in the design does not exist, so nothing to edit there

**Next:** Rename the case-task PendingApproval to CaseTaskApproval and add the approval-standing golden fixture (D2.2).

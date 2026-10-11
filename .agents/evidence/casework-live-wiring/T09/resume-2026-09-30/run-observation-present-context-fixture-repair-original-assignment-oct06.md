# Present-context Phase1 fixture repair — fresh original assignment

Date: 2026-10-06. Fresh builder Luna live_cursor_v4_boundary_recon,
different from original guard builder cursor_manager_revision3_doc_builder.

Read complete original guard assignment, frozen stub565ad7a6, test825ab0dc,
independent reviewd715a0b5 and supplementary erratumdbdb11ab. Edit ONLY the
existing new guard test file. Keep source stub hash EXACT, no algorithm/callers.
Preserve every original contract and positive/negative assertion; repair all
findings rather than weakening validation:

- Owned independent before-state oracles, not aliases of passed slices/facts.
  Preserve nil versus nonnil empty slices, including Snapshot.VisibleObjects.
  Copy every populated nested fixture field including Overview.Stages. Fresh
  independently constructed equal fixtures may replace a faulty clone helper.
- Isolate blank revision cursor by keeping compared snapshot/facts/relay cursor
  values consistently blank; add requested nonblank cursor discrepancy coverage.
- Newest-invalid/no-fallback tooth must have relay matching older valid row,
  so an incorrect fallback would actually succeed; newest correct row stays
  invalid and must fail. Assert no input mutation against owned baseline.
- Wrong-case rejection need not query relay. Require history exact requested
  case and every relay call, if any, exact requested case; do not force an
  unnecessary relay read after already invalid history.
- Retain complete empty/populated positive success, copied IDs, repeated-result
  independence, exact opaque equality, later mismatch/matched new horizon and
  removed-parent non-reuse. Report which assertions are expected RED against
  stub and which later checks are unreached until eventual implementation.

No test run, compiler, scanner, Graft build, Git, network, source/docs/status/debt
edits or other tests changed. Native apply_patch only. Read instructions/Graft
and complete file first. Return repaired test hash and all material deviations.
Different independent critic receives ORIGINAL and repair assignment, full file,
prior review/erratum. Actual RED remains held until fixture approval and root's
explicit exclusive compiler grant with fresh RAM/swap. No production release.

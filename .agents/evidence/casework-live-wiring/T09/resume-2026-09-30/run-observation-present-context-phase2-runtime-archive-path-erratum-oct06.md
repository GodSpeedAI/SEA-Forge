# Present-context Phase 2 runtime review — focused archive path erratum

Date: 2026-10-06. This immutable erratum supplements `run-observation-present-context-phase2-independent-runtime-verdict-oct06.md` (SHA-256 `fdf348ca293fd6deed74ea04046c536474798a2f10429cf761b0be32b621cfd1`). It corrects only the focused GREEN archive path stated there; it does not change any gate result or verdict.

The runtime verdict incorrectly named `present-context-red-01-run-root-direct-original-copy.raw` as the root's direct archive for the focused GREEN gate. That file is the earlier Phase 1 focused RED capture against the stub and is not the GREEN output.

The correct direct archive is `guard-green01-focused-go-output-direct-original-copy.raw`. It is 5,953 bytes, SHA-256 `73f5161b8ec3ee23726ff6a51725c48f9efb051310f3af7ddd20dbe108e65b37`, and `cmp -s` against `/tmp/sea-casework-20261006-guard-green01-focused-run.raw` returned exit 0. The erroneous citation did not overwrite either file; both original artifacts remain preserved under their distinct paths.

No tests, compiler, or source changes occurred while correcting this evidence reference.

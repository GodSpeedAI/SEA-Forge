# Hydration cap final-frame metadata fixture repair

Fresh builder: admission_retry_counter_fresh_builder, different from the latest rejected fixture builder admission_wiring_recon. Source only; no compiler.

Read the original cap assignment, first repair assignment, full current fixture bf21b941 and independent rereview7952b429. Repair ONLY the cap test file. Address R1 with explicit assertions in the naturally oversized multi-run case that the emptied run retains its identity, standing and observation metadata, original total, zero retained frames, exactly incremented omitted count and truncated=true. Assert unaffected runs retain correct frame/count metadata. Preserve all existing correct oldest/tie, cap, immutability and nonalias assertions. Production stub unchanged.

Clarification: the same-run EventID tie leaves one of two frames, so that example is not itself an emptied-run example; the run-ID tie and natural multi-run example establish the relevant gap. Do not alter those correct expectations.

Read before edit, native apply_patch, file-only formatting permitted. No other edits, implementation, compiler/test/scanner, Git/status/debt, or generated output. Report final hash and freeze for independent rereview.

# Hydration cap fixture repair — original bounded assignment

Fresh builder: admission_wiring_recon. Source only; no compiler ownership.

Read the original hydration-cap assignment, complete frozen source/test files, and independent Phase 1 rejection. Repair ONLY `apps/godspeed-casework-go/internal/server/run_observation_hydration_cap_test.go`. Preserve the production stub and all existing semantic expectations, including retaining zero-frame run metadata.

Address every rejection:

1. Use valid execution `active` and settlement `unsettled`; keep observation state `validated`, which is valid.
2. Constructors must produce nonnil empty run/frame arrays. Clone helpers must preserve nil versus nonnil empty shape and independently copy nested arrays and optional pointers.
3. Add an oversized successful trimming case that snapshots input, proves it unchanged after the call, mutates returned run/frame arrays, cohort count pointers, and surviving frame execution-status/exit-code pointers, then proves the original unchanged. Ensure at least one frame survives and the independently computed expected oldest frame is removed.
4. Isolate the nine-run bound with valid hydration budget and all unrelated metadata within allowed ranges.

Read before editing; use native apply_patch. No other source, production implementation, dependencies, interfaces, gates, Git, status, debt, or generated output changes. Formatting is allowed for this file only. Report final SHA-256, exact changes and any remaining gaps; freeze for independent review. Do not claim actual assertion RED without a separately authorized verifier run.

# Artifact provenance repair result

The gateway now uses `artifact.get` for the content-addressed bytes and then resolves the returned `run_id` through SFWP `run.get`. It refuses if the run ID differs, if the run has no owned case or plan item, or if the run evidence row and its `artifact_captured` trace event do not match the returned evidence ID, URI, digest, and item. A missing/unreadable owning run is translated to a typed unavailable refusal; authority denials remain authority denials.

The HTTP handler returns the kernel-backed case, plan item, and run IDs. It refuses incomplete ownership instead of emitting blank case/item fields. Content integrity checks, UTF-8 handling, and session verification before any authority lookup remain in place.

`invocation_id` remains the empty string required by the existing payload shape. Neither `artifact.get` nor `run.get` exposes a standalone invocation ID for this artifact, so no value is inferred from the run ID or fabricated. This gateway can establish run/case/item and evidence-to-capture-event linkage; it cannot assert invocation-level provenance from the current kernel views.

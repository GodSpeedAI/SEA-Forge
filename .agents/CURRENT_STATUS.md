# Current status

**Status revision:** 187

**Stage:** capability-harvest: round-1 root causes fixed — delegation containment, unmasked spawn errors, rmdir retry+probe, macOS payload instrumentation; pushing round 2

**Summary:** cgroup Active-path verdict: kernel delegation containment authorizes cgroup.procs writes via the common ancestor, and the runner sits outside the delegated subtree, so the pre-exec join was denied EACCES and the cleanup error masked the primary. Fixes: join_authorized probe converts the situation to typed cgroup_unavailable before any leaf exists; the preflight now also delegates the mount root's cgroup.procs; primary spawn errors lead and cleanup failures append as context with leaked leaves left to the destroy sweep; rmdir retries 3x with diagnostics naming the parent mode/uid. macOS verdict: payload-side (the helper's fallback budget collided with the execute deadline converting helper-never-recorded into a bare TimedOut); helper now writes a started marker with three-way discrimination (never-ran / setsid-denied / recorded) and polls strictly inside the execute budget with evidence-carrying asserts.

**Verified:**

- sandbox lib 26 green; conformance_m5 4 passed locally (Unavailable path); clippy -D warnings green on linux and aarch64-apple-darwin targets; workspace check clean; fmt clean
- CLI suites that failed on CI (m0_migrate, m4b, cli m5) pass locally

**Limits:**

- The cgroup Active proof and the macOS cage-persists verdict are CI round 2's to deliver; the rmdir-EACCES mechanism may not reproduce once the join is authorized
- F17 rest-grant raw-connect semantics await the openshell-live re-dispatch

**Next move:** Commit round-2 fixes, push, re-dispatch openshell-live, collect CI round 2, then freeze for the codex review.

**Evidence:**

- gh run 40369591334 logs
- .tmp/harvest/

# Current status

**Status revision:** 197

**Stage:** capability-harvest: WINDOWS LANE GREEN (round 12 — self-exec breakaway probe worked); macOS cage census race made deterministic; pushing round 13

**Summary:** Round 12: the entire Windows portability lane passes for the first time - the self-exec breakaway probe (test binary re-exec'd into the sandbox, CreateProcessW via the windows-sys dev-dependency) both proved the control create and the kernel's breakaway refusal. The one regression is the macOS cage-persists test flaking back to ProcessGroup: the census samples the workload's ppid tree only while the workload lives, and the sh exited within ~50-100ms of detached.pid appearing, leaving the descendant recording to a single race-prone slice (four green rounds were luck). The workload now sleeps one second after the marker appears, guaranteeing ~10 census slices; the helper parent's 1.5s linger and the 20s execute budget both still fit.

**Verified:**

- gh run 40504750501: verify (windows) success - every windows step green including job-object tests
- apple-darwin check + clippy -D warnings green on the fix; fmt clean

**Limits:**

- Round 13 must confirm the macOS lane green again (windows unaffected)

**Next move:** Push round 13; on green, freeze and run the codex adversarial review.

**Evidence:**

- gh run 40504750501 log

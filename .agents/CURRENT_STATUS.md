# Current status

**Status revision:** 196

**Stage:** capability-harvest: breakaway probe rewritten as a self-exec test binary (donor pattern) after the PowerShell probe proved unfixable; pushing round 12

**Summary:** Donor re-review verdict: ProcessKit exercises Windows workload behavior via interpreter-level Start-Process (which our grandchild test already covers) and has NO in-workload CreateProcessW probe at all - the os-level direct test (passing) is the donor-equivalent breakaway proof. The in-workload demonstration is retained but rebuilt on the repo's proven self-exec pattern: the test binary re-execs itself into the sandbox with --exact breakaway_probe_helper, activated by env vars, calling CreateProcessW via a windows-sys dev-dependency (same locked 0.61 the os crate uses - no new packages; test-only). The breakaway child carries the survivor-sweep token; the control create attributes the refusal to the job.

**Verified:**

- windows-target check + clippy -D warnings + host clippy + fmt green; sandbox lib 26 green

**Limits:**

- Round 12 must confirm the windows lane end-to-end green

**Next move:** Push round 12; on green, freeze and run the codex adversarial review.

**Evidence:**

- gh runs 40459348653 / 40446093616 logs

# Current status

**Status revision:** 191

**Stage:** capability-harvest: round 5 verdicts — CI cgroup interference fixed (registry-scoped sweep); windows rename root cause finally named (pin vs kernel parent open) and resolved as a pinned path-rename

**Summary:** Round 5: ubuntu CI still failed but the instrumentation narrowed it decisively — the by-handle FileRenameInfo rename fails ERROR_SHARING_VIOLATION (32) because the kernel's rename takes a write-class open of the target's parent directory, which the exclusive pin's FILE_SHARE_READ-only share refuses; every prior INVALID_PARAMETER was the same conflict wearing a different error (the anchored NtCreateFile temp failed for the adjacent reason). The publish is now a path-based rename executed while the pin holds: the temp name is not a substitution window under the pin (no concurrent writer can even open the directory), the rename resolves the target by DACL (not share), and the exclusive pin brackets validation, temp creation, write, sync, and publish end-to-end — F05's full-prevention invariant is intact with the only proven-working mechanism. By-handle and anchored-create forms are recorded as platform verdicts in the spike.

**Verified:**

- Windows-target and host clippy -D warnings green; fmt clean; sandbox suite green
- CI run 40410651296 ubuntu jobs beyond the runtime interference green (job-level triage)

**Limits:**

- Round-6 lanes must confirm: windows hard-link replace green, ubuntu cgroup suites green under the registry sweep

**Next move:** Push round 6; on green, freeze and run the codex adversarial review.

**Evidence:**

- gh run 40410651287 log (os error 32 dump)
- .agents/OBSERVED_DEBT.md

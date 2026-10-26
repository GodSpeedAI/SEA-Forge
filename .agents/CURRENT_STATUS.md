# Current status

**Status revision:** 190

**Stage:** capability-harvest: round 4 verdicts in — macOS lane GREEN (SandboxCagePersists verified live), KVM lane GREEN (F17 BLOCKED-with-evidence + deny variant); round 5 fixes the cgroup destroy-sweep interference and the Windows temp creation

**Summary:** Round 4 on real runners: the macOS cage-persists test classifies SandboxCagePersists with a census-recorded setsid survivor (F08-macOS verified); the KVM ladder exits 0 with the honest egress BLOCKED evidence pack and the passing deny variant (F17 disposition final). Two residues named and fixed in round 5: (1) destroy_process_leaves globbed the cgroup root by pid prefix, so one sandbox's destroy() removed a sibling sandbox's live leaf in same-process parallel tests (kill ENOENT, rmdir ENOENT) — the sweep is now registry-scoped (only this process's registered leaves) and double-teardown is idempotent (ENOENT reads as empty/gone); (2) the Windows anchored NtCreateFile FILE_CREATE temp rejects STATUS_INVALID_PARAMETER on Server 2025 (anchored opens of existing entries work; three rename forms and now the anchored create are recorded platform verdicts) — the temp returns to the years-proven std create_new under the exclusive pin (visible name is not a substitution window while the pin holds) with access_mode(GENERIC_WRITE|DELETE) so the handle-identified rename keeps DELETE access.

**Verified:**

- gh run 40406879337: verify(macos), lint, MSRV, workbench, authority-opa all success; KVM run 40406886571 success
- Local: sandbox suite 23 test-targets green, runtime lib 5/5, conformance_m5 4, clippy -D warnings host+windows clean, fmt clean

**Limits:**

- Round-5 lanes must confirm: ubuntu cgroup tests green under parallelism, windows hard-link replace green

**Next move:** Push round 5; when CI+portability are green, freeze and run the codex adversarial review.

**Evidence:**

- gh runs 40406879337 / 40406886571 / 40406874861
- .agents/OBSERVED_DEBT.md

# Current status

**Status revision:** 185

**Stage:** capability-harvest: CI round 1 triaged — F17 fixture rule reshaped to the provable vocabulary; windows pin failures under fix

**Summary:** CI round 1: kernel-security green; portability windows job failed (two pin tests + an access violation in the new anchored FFI — fix subagent working); openshell-live failed because the prover refuses tcp-protocol endpoint rules. Local prober evidence (openshell-prover 0.1.3 AND 0.1.5): tcp rules are unsupported in candidate and boundary policies; IP-literal+rest parses and self-checks within_boundary. DomainForge model vocabulary has no L4-connect operation (only read/write map to access presets), so the F17 control endpoint now projects as a rest (http/read) rule on 10.0.2.2:46244 — the KVM lane will reveal whether enforcement permits a raw connect under a rest grant, which is precisely what the positive control exists to discover.

**Verified:**

- Prober probe: v0.1.3 and v0.1.5 both return 'unsupported' for tcp rules (candidate and boundary); ip+rest and host+rest parse; clean==clean control within_boundary
- Regenerated fixture passes openshell-prover 0.1.3 self-check: within_boundary, full coverage domains
- Fixture-consuming suites green after the regen: cell_episode 24, opa_candidate 29, openshell_stub 35
- domainforge CLI installed at exactly 0.21.2 for regeneration (0.19.0 was stale in ~/.cargo)

**Limits:**

- Whether OpenShell enforcement permits a raw TCP connect under a rest grant is unknown until the KVM lane runs
- Windows pin failures and the access violation await the fix subagent and a lane re-run
- CI (ubuntu test, macOS) and casework lanes were still running at triage time

**Next move:** Land the windows FFI fix, push both fixes, re-dispatch openshell-live, then footprint completion and the codex review.

**Evidence:**

- /tmp/prover-probe (probe outputs)
- .tmp/harvest/
- .agents/OBSERVED_DEBT.md

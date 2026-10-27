# Current status

**Status revision:** 203

**Stage:** ARM64 Jetson packaging: probe workflow registration lag; push trigger added as the reliable trigger

**Summary:** GitHub registered the arm-probe workflow lazily; workflow_dispatch 404s until registration completes. The probe now also fires on push to the branch. Meanwhile the capability-harvest branch's cfg fix (2e83e508) and record reconciliation (5e2a3bb9) are pushed; their lanes re-run.

**Verified:**

- arm-probe.yml present on the remote branch (contents API)

**Limits:**

- Probe verdict pending

**Next move:** Probe verdict -> implement the packaging generalization.

**Evidence:**

- .github/workflows/arm-probe.yml

# Current status

**Status revision:** 152

**Stage:** light-local-hooks

**Summary:** Local git hooks no longer compile: pre-commit runs status-check, fmt-check and the staged secret scan; pre-push runs status-check and fmt-check; `just pr` uses the same light check. The full sweep runs on GitHub (CI / gate), which gates every pull request. `just ci-job <job>` runs locally exactly what one GitHub job runs, so a red check reproduces with one command; `just ci` still runs the full sweep. Also the e2e helper's 'panel closed' wait gets the same 30s budget as the disk wait before it: L-RECOV (c) on a loaded CI runner measured ~16s (4s commit, then two ~6s client-side gaps) against a 15s budget.

**Verified:**

- hook steps time ~2s locally (fmt-check + secrets-staged), previously minutes (workspace cargo check / just ci)
- `just ci-job` with an unknown job exits 2 and lists the jobs
- L-RECOV failure on PR #43 run 39821463499: gateway log timestamps show the commit returned 200 and the history load succeeded; the panel closed after the 15s window; the rerun passed 11/11

**Limits:**

- the ~6s client-side gaps after a commit under CI load are not explained yet (performance, not correctness)
- ci-job mirrors .github/workflows/ci.yml by hand; keep them in step when jobs change

**Next move:** Merge after CI; watch whether L-RECOV stays green over the next ladder runs.

**Evidence:**

- https://github.com/GodSpeedAI/SEA-Forge/actions/runs/39821463499
- docs/explanations-and-references/ci-cd-architecture.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md

# Current status

**Status revision:** 414

**Stage:** CognitiveOS #108 integration: #116 merged

**Summary:** Merged the #116 execution-profile branch (approval standing carries the execution profile digest; hosted CI now runs the CEP validators against a pinned public commit and fails instead of skipping).

**Verified:**

- The merge had no code conflicts; the status files kept the integration history.
- #117 replay and receipt hardening is merged in the next commit; gates run on the merged tree.

**Limits:**

- The merged tree has not yet been built or tested.
- Hosted CI has not run on the integration branch.

**Next:** Merge #117, then run the full gates and hosted CI.

**Evidence:**

- .github/workflows
- .agents/specs

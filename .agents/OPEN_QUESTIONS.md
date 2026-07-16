# Open Questions

Only choices that cannot be resolved from the repository or authoritative
research belong here. Include a recommendation and the trade-off. Remove the
entry when decided.

## Question: How is a summarized transcript hash verified after transcript disposal?

- Context: `spec-agent-orchestration.md` requires summarized mode to omit the
  full transcript artifact, but completion also requires transcript evidence to
  be hash-verifiable. A digest cannot be recomputed after its canonical input
  bytes are discarded.
- Recommendation: retain a sealed/encrypted canonical transcript outside the
  public `full` artifact surface, verify it before any crypto-shredding, and keep
  the summarized view as the default disclosure surface.
- Trade-off: this preserves auditability but weakens strict data minimization.
  If minimization wins, narrow summarized-mode assurance to ledger integrity of
  the recorded digest and reserve transcript recomputation for `full` mode.
- Needed by: Task 0.4 / M13 TranscriptEvidence implementation.
Decision: Retain a sealed, encrypted canonical transcript for summarized mode, verify it before crypto-shredding, and expose only the deterministic summary by default.
This preserves genuine hash verification without making the full transcript broadly visible.

<!-- Entry format:
## Question: Decision needed
- Context: evidence and why research did not resolve it
- Recommendation: preferred choice
- Trade-off: what the recommendation gives up
- Needed by: task or milestone blocked by the decision
-->

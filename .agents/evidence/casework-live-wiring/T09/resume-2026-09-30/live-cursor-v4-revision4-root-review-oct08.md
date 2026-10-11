# C2 revision4 root review: retain HOLD

Date: 2026-10-08. Current candidate SHA-256
3cbf9e42561bbaaf965815709266362460ba76bfdf930bb5549afd4f3ffe7bf2.
Root read the complete current candidate. Proposed shared journal/stamped
capture addresses the architectural writer gap, but exact approval readiness
still requires independent review and corrections below. No source release.

1. The self-contained candidate must explicitly carry revision2's historical
   GET cursor+capture_digest matching and SSE last_capture_digest reconnect
   precondition. Section6 mentions the digest for intents, but the specified
   SSE route carries only last cursor and never defines digest matching for
   historical GET. Without it, same-cursor restart captures remain ambiguous.
2. State the 1MiB maximum individual snapshot/event and at most64 replay
   revisions explicitly. These are root choices; per-case4MiB/queue2MiB caps
   do not replace the per-item limit or replay-count rule.
3. Supply the precise digest bytes/profile/golden exclusions, exact historical
   and stream mismatch status/body/client-reset semantics, and bounded page
   recovery including operation-ID scan continuation. Do not silently permit
   partial recovery scan to declare an operation absent.
4. Verify operation-ULID helper visibility and source inventory against actual
   code. A source-backed table is not proof that every in-tree writer is listed.
   Strict inventory/bootstrap must explicitly state how pending operations and
   legacy flat files interact with complete-empty success, not hide uncertainty.
5. Builder reported an earlier revision4 hash then edited the same artifact
   before reporting the current hash. That first revision4 content was not
   preserved as an immutable artifact. Review the actual current identity only;
   do not claim the earlier hash has a preserved preimage. Revision3 remains.

These are proposal defects/provenance limits, not evidence of runtime failure.
Give a fresh independent critic FULL originals, root decisions/clarification,
candidate, prior rejection and this review. If it rejects, assign a different
builder; preserve all existing candidates/reviews and add a new revision.

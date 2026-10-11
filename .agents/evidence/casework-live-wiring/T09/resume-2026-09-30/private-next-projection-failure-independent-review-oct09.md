# Independent review: private Next projection-failure addendum

## Verdict

**APPROVE the semantic addendum within its stated private scope.** Classifying an
unexpected pure-projector failure as terminal unavailable, with zero aggregate,
no watermark commit, and single-owner teardown, fits the already-approved
terminal unavailable ownership rule. It does not change the separately approved
recoverable read/retention-marker policies. This is a document-level semantic
review only; it is not source readiness, runtime evidence, or approval of the
rejected TDD fixtures.

## Evidence and policy basis

I read the full original assignment, sequencing and notifier addenda, new
projection-failure addendum, and `private-next-tdd-independent-source-review-oct09.md`.
I also checked the referenced revision 6 proposal, its addendum, retention
initializer correction revision 3, operator approval, integration root
decisions, the existing pure projector, and the governing V4 spec's scope and
authority clauses.

The policy draws these distinctions:

* Revision 6 §“Exact ledger, read failures, version capture, and deltas” names
  `ErrPollerReadUnavailable` as transient: zero delta/no watermark advance,
  keep the lease attached, and recover only after a later accepted read.
  Revision 6 §“Typed error set” requires every `Next` error to return a zero
  aggregate and no watermark commit; §“State, lock, and exact ownership” gives
  the drain ownership/join rules.
* Revision 6 retention correction 3 §“Terminal candidate rejection and stop
  ordering” is about terminal **source candidates** rejected by retention; it
  requires generic unavailable, no candidate disclosure, stop before join,
  and counted capacity through actual joins. It does not itself classify a
  pure-projection error.
* The approved private policy (`run-observation-private-policy-operator-approval-oct08.md`)
  explicitly approves nonterminal overflow recovery and terminal overflow
  teardown, but is likewise specific to retention.
* The integration root decision §“Error disposition and drain” explicitly
  says terminal retention failure, authorization/session/cursor/context
  failure, cancellation, and manager Stop enter terminal drain; it separately
  retains read and nonterminal-retention recovery. It also requires releasing
  the Next operation before waiting for teardown. Its “Captures, wake ownership
  and commit” says any run error discards every proposed watermark and the
  entire aggregate, with no recapture/retry.
* The assignment already requires “Any run failure” to return zero aggregate
  and commit no candidate watermarks. Its sequencing addendum establishes
  operation ownership before callbacks and deferred release before teardown
  wait. The approved pure helper itself returns an unavailable error for
  malformed captured state/ledger/window relationships (`run_observation_delta.go`,
  `buildRunObservationRunDelta`).
* The V4 spec is limited to public cursor/frontier/bootstrap/stream contracts;
  it does not grant public error-contract changes or make this private helper a
  public readiness claim (`casework-live-cursor-v4-spec.yaml`, `scope`,
  `authority.no_authority_change`).

The original documents did **not** expressly assign unexpected pure-projector
errors to either the recoverable-marker category or terminal-drain category.
The new root semantic decision closes that gap. It is consistent with the
already-approved generic terminal unavailable path, rather than an assertion
that the narrower retention correction independently covered projector
failures. No policy contradiction remains once this explicit root decision is
included.

## Material additions and limits

The new addendum makes these previously unspecified details explicit:

1. A pure projector failure over an otherwise captured current image is an
   integrity/assembly failure. It returns typed unavailable, zero aggregate,
   and no watermark commit, then schedules the single terminal drain owner.
2. The same captured terminal image is not retried as recoverable; no implicit
   wake or retry is permitted. This preserves the no-recapture/no-retry rule.
3. A multi-run all-or-nothing test may inject a typed pure-projector error only
   after a genuinely advanced candidate was assembled. It must establish
   non-vacuous high-water marks from real worker reads, prove every prior
   watermark remains unchanged, and observe actual teardown. Fault injection
   proves only the error branch, not publication or recovery.
4. Read- and retention-marker recovery remains a separate claim and requires a
   later actual worker publication. The failure test cannot manufacture that
   evidence.

These additions narrow failure semantics and strengthen the proof obligation;
they do not authorize source continuity, retention-policy changes, another
manager hook, a public error contract, or a different lifecycle implementation.
The proposed wrapper still has to call the same production transaction helper
and use the existing pure assembler, as required by the original assignment.

## Review boundary

This semantic approval does not reverse the source review's **REJECT** verdict:
that review found static compile errors and fixture gaps, explicitly including
the vacuous all-or-nothing test and unresolved projector disposition. The new
decision resolves only that semantic question. The fixtures still require a
fresh source-preparation repair and independent source review before root's
expected-RED gate. No compiler, test, formatter, runtime gate, or Git action was
run for this review.

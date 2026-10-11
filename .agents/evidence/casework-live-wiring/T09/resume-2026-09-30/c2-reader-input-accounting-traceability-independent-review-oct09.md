# Independent review: C2 reader input-accounting traceability

Date: 2026-10-09

## Verdict

**APPROVE the changed supplemental specification for normative traceability
only.** The exact reviewed supplement hash is
`805f50bfe39e2c3aded66e0963c23de1d26033c4cd6b4acbbbf0160788b2b561`; the
builder record hash is
`583a47a38a325a10ac45dd7dc10b9738d2ba4bc2222cb8b0eddb76f44f1c29ee`. This
does not authorize reader implementation, tests, runtime work, readiness, or
T09 settlement.

## Authority and reviewed evidence

I read the full original normative traceability grant reproduced in
`c2-reader-input-accounting-traceability-builder-oct09.md`, the full Oct. 9
root clarification
`c2-reader-registration-input-accounting-root-decisions-oct09.md`, and its
independent review
`c2-reader-registration-input-accounting-independent-review-oct09.md`. I also
read the builder record, the Oct. 8 additional-policy operator approval, the
proposal revision 4 independent review, and the preceding normative-package
review. I reviewed the exact current spec diff and the current clauses
`REQ-C2-RANGE-004`, `REQ-C2-RANGE-005`, and `V-C2-RANGE-03`.

The root clarification and its Oct. 9 independent review authorize and accept
the total raw-input accounting interpretation, not source work. The operator's
additional-policy receipt approved one 4 MiB raw-input budget per page,
including non-events and lookahead, while preserving the other established
limits. The revision 4 review is proposal-only. These records support the
normative text below and do not release implementation.

## Findings

The changed `REQ-C2-RANGE-004` correctly charges every actual raw ledger byte
read to the same per-page 4 MiB budget: head/tail pinning, continuation
predecessor revalidation, forward scanning, non-event rows, and lookahead.
It explicitly charges repeated reads again and excludes metadata checks and
seeks. It requires bounding read requests and row-buffer growth by the
remaining budget before reading or allocating. Exhaustion cannot acknowledge
a partial row, undelivered lookahead, or incomplete validation; only prior
acknowledged progress or typed unavailable can be returned.

`V-C2-RANGE-03` requires combined auxiliary-plus-forward exact-cap and
cap-plus-one vectors, explicitly charges repeated reads, and excludes metadata
and seeks. At exact cap it permits acknowledgement only through complete
validated rows. At cap-plus-one it forbids reading beyond the remaining budget
and forbids acknowledging a failed page, partial row, or undelivered lookahead.
This is a direct conformance vector for the total-budget rule, rather than a
forward-scan-only limit.

The exact spec diff changes only
`.agents/specs/casework-live-cursor-v4-spec.yaml`: it updates the revision date,
adds source references for the approved clarification and its review, makes
the accounting requirement explicit, and strengthens the associated test
contract. The parent spec remains byte-identical at SHA-256
`dc9ea678bab7e24d02072beb46d4079ac3181a790c91f1146919e4bb4624601c`; ADR-008
remains byte-identical at
`c66912baee08317e4819ad211a208f5e2abff5d3369053a743a0f384737cc4e5`. Not
editing either is within the narrow grant: the parent delegates C2 detail to
the supplement, and the existing ADR already records the approved 4 MiB cap.

No existing cap or policy is increased or weakened. The 2 MiB row cap
including LF, 500 scanned-row and 500 emitted-frame limits, 1 MiB complete
serialized-response cap, and 50 ms lock-wait limit remain explicit. No public
schema, dependency, interface, authority boundary, or implementation status
changed. The diff does not add an uncharged allowance.

## Lifecycle and limits

The supplement remains `metadata.status: review`, its operator approval records
still say `implementation_status: held`, and the parent still references the
supplement with `status: review`. The parent says the supplement becomes
normative after independent review accepts it. This review accepts the exact
changed supplement, so I recommend root advance the supplement's lifecycle
status to `approved` and mirror that status in the parent's supplemental-spec
entry, while keeping implementation held. The approval receipts are
recommendation-level policy approvals, not exact candidate-hash approval or
implementation authorization. Status metadata should be changed by root as a
separate coupled lifecycle edit; this critic record does not modify either
spec.

No tests, gates, compiler, formatter, or runtime checks were run. No
implementation correctness or runtime readiness is established. The future
bounded-reader builder grant remains outstanding.

## Material deviations

No material deviation from the normative traceability grant or approved Oct. 9
clarification was found. The parent spec and ADR were not edited, as permitted
when not strictly necessary. This approval covers only the reviewed normative
traceability change.

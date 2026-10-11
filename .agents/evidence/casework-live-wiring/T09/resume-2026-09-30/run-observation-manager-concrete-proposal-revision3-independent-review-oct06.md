# Independent review: private run-observation manager proposal revision 3

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-concrete-proposal-revision3-oct06.md`, SHA-256 `568ab32b5617564910c4829e9d216d3e5ab03da25a899fd12414845c66da4ba7`.  
Disposition: **REJECT as an implementation blueprint; preserve public and runtime HOLD.** The review is DOCONLY and does not authorize a contract or source change.

## Revision 2 findings addressed

The revision materially repairs the three manager revision 2 review gaps:

- It specifies an idempotent rollback owner for failed `Prepare`, covering lease reservation, partial run refs, cancellation only after the last attachment, joining owned work/notifier refs, stop races, and retaining draining slots until actual completion.
- It chooses the requested `(InitialCohort, CohortLease, error)` return, separates initial results from later deltas, and specifies initial watermark capture and post-construction watermark advancement.
- It supplies a concrete `attach(ctx, lease, row, guard)` signature, typed dispositions for current/shared/new/capacity/draining/mismatch/invalid outcomes, ownership/ref handling, and initializer token.
- The authorization seams now match existing source signatures: `SessionStore.Current(id) (CurrentSessionState, bool)` (`internal/auth/session.go:147-169`), `PerspectiveVerifier.VerifyPerspective(ctx, ports.ActorClaim) error` (`internal/server/server.go:38-42`), and request identity source fields (`internal/server/session.go:64-70`).

The guard description remains grounded in actual `Revision.Facts`, `CaseFacts`, `CaseHorizon`, Store trajectory, and Relay cursor behavior (`internal/projection/store.go:41-53,122-139`; `internal/projection/builder.go:34-51`; `internal/ports/ports.go:266-274`; `internal/server/relay.go:101-150,170-175`). It appropriately treats the check as as-of-cursor and non-atomic. Session and dev-only bearer rules remain explicit and match auth source. The count equations/status precedence, selected-row failure vs capacity, eight read/selection interpretation, shared physical admission ownership, one-second floor, and unapproved 16-cohort limit remain documented.

## Blocking mismatch: initial payload abandons the accepted hydration DTO and bounding helper

The revision defines a bespoke `InitialCohort` / `InitialRun` result instead of using the approved `contract.RunTraceObservation` hydration payload. It omits contract fields that carry accepted semantics, including top-level `RunListState`, `ObservationState`, and `HydrationReadBudget.Limit/ReadsAttempted`, and per-run `ObservationState`; its local `Status`/`Exhausted` fields do not establish the same DTO encoding or presence rules. The approved DTO has those fields and pointer counts in `internal/contract/contract.go:186-238`.

More materially, the revision says to marshal the whole custom `InitialCohort` and fail with `ErrInitialEnvelopeTooLarge` whenever that value exceeds 1 MiB, with no frame-pruning step. This contradicts the accepted behavior and its available production helper. `boundRunObservationHydration(input contract.RunTraceObservation)` deep-clones the exact DTO, checks run/frame/count invariants, measures the JSON payload, trims globally oldest frames with timestamp then full run/event ID ties, updates counts, and returns typed unavailable **only when required metadata still exceeds the cap after all frames are removed** (`apps/godspeed-casework-go/internal/server/run_observation_hydration_cap.go:22-77,80-149`). The original manager assignment also requires that oldest-frame pruning and metadata-only irreducible failure behavior.

The helper's tests assert payload-at-cap and cap-plus-one behavior, global-oldest determinism, exact preserved counts/metadata, non-aliasing, and unrepresentable metadata rejection (`run_observation_hydration_cap_test.go:143-155,208-263,328-410`). Therefore an ordinary frame-heavy initial payload above 1 MiB must be reduced by this established helper; it must not fail merely because the pre-pruned payload is oversized. The new, separately proposed 1 MiB-per-poller retained-byte policy is not a substitute and remains expressly unapproved.

**Required:** define the initial result as the accepted hydration DTO (with only a private wrapper for internal cursor/lease metadata if needed), pass it through the exact accepted `boundRunObservationHydration` semantics, preserve enum/pointer/read-budget behavior, and fail only when irreducible required metadata cannot fit. Do not make initial payload size alone a whole-prepare failure.

## Remaining concrete-type gap in the claimed exact delta contract

The revision names `RunDelta`, `RunWindowGap`, and `ObservationStatus` but never declares their fields or types. This leaves unreviewable whether deltas preserve approved standing/state fields, identify `(run_id,event_id)` without ambiguity, represent total/retained/omitted/truncated truth, distinguish a coalesced wake from a frame gap, or state omission counts exactly. The assignment specifically requests exact delta types, watermark/omission semantics, and an independently reviewable private producer/consumer contract. The prose gives useful watermark rules, but the types needed to verify those rules are absent.

**Required:** define these types and exact validation/accounting semantics, or reduce the proposal to existing `contract.RunTraceObservation` plus a clearly internal `RunTraceRunObservation` delta envelope. Distinguish `InitialCohort` from subsequent delta by type and prevent loss of read-budget and state fields. Also clarify the two contexts in `CohortRequest.RequestContext` and `Prepare(ctx, req)`: state whether one is removed or which context controls list/read cancellation, request lifetime, and stop joining.

## Existing source / assignment checks

- The safe trace port returns exact run/case/plan IDs, separate standings, safe frames, and total frame count (`internal/ports/run_trace.go:5-30`). Its adapter validates returned identities/standings and caps frames at 1,024 (`internal/adapters/sfwp/run_trace.go:66-176`). The proposal's exact-key and selected-row checks are consistent with that boundary.
- The 16-poller registry is still process-shared by `(case_id,run_id)`, counts draining entries, avoids active eviction, and does not duplicate the accepted shared client admission/retry/cooldown owner. The new 16-cohort cap and per-poller byte budget are marked proposals requiring separate review/operator approval. They are not accepted source facts.
- The 1 MiB cap's existing helper uses exact `RunTraceObservation` JSON bytes rather than SSE framing and preserves required metadata while trimming eligible frames. The revision's replacement failure rule is a material regression against that accepted contract, not merely a private type naming choice.
- Newer `Prepare` rollback text is directionally complete, but the error taxonomy should say which source `ReadRunTrace` failures are selected-row `A` versus which are whole-cohort failures. The prose currently assigns denied/malformed/mismatch/orphaned/source errors to `A`, but also lists generic assembly and initial-envelope errors; the final DTO distinction must remain consistent after the accepted frame-pruning helper is restored.

## Verdict and next step

Revision 3 closes the lifecycle/ownership and initial-versus-delta gaps from the prior review only at the prose/API-outline level. It still replaces required initial hydration behavior with an incompatible bespoke payload and omitted-fields model, and its named delta types remain undefined. **Reject** this blueprint for implementation release. Preserve the revision and this immutable review; request a bounded revision 4 that uses the approved DTO/helper exactly and defines every delta/error field. The new lease/byte limits remain unapproved proposals, and public SSE/V4/frontier wiring remains held.

No source, test, status, debt, or evidence file other than this review was changed. No tests, compiler, scanner, gate, Graft build, Git, or network operation was run.

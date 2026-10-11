# T06 critic findings — fixes (2026-09-25, orchestrator + completion of the fix builder's work)

The independent critic REJECTED T06 (T06/critic/confirmation.md, findings F1-F6). The fix
builder was killed by an infrastructure error partway through (F1-F6 substantively implemented);
the orchestrator completed the remainder and added F7. Per-finding:

- F1 (empty actor claim 403 on every ?actor= world request): live.go VerifyPerspective now sends
  the gateway principal's claim with on_behalf_of = the requested actor, exactly as intents do.
  New live test TestLiveWorldPerspectiveVerifiesAgainstTheKernel: allowlisted actor -> 200 with
  real standing; bound-but-not-allowlisted -> 403 authority_denied.
- F2 (discretionary add could not produce an accepted kernel write): intents.go builds a minimal
  VALID PlanItem (governed write_file op derived from the title, required_artifacts naming it,
  depends_on anchoring to stage_id; kind "sandboxed_task" accepted by kernelItemKind — the
  builder's allowlist had omitted it). New live test TestLiveDiscretionaryAddProducesAnAccepted-
  KernelWrite: add -> plan_mutated (exactly 1) -> execute -> item_completed with settlement
  accepted and the honest write_only basis, from durable state.
- F3 (E2E cell could not open an approval through the served surface): the kernel opens an
  approval_request ONLY via an Escalate verdict at episode dispatch. The E2E policy now escalates
  exactly the signoff-gate's governed draft write (segment-aware review/ prefix), so executing
  task_draft opens the approval the L5 journey resolves. Documented in policy.yaml comments.
- F4 (new_cursor race): the intent guard's post-mutation wait requires cursor ADVANCEMENT
  (postMutationCursor); the SSE drain test skips same-cursor snapshots instead of breaking on
  the first one (late-arriving initial revision under load).
- F5 (plan_schema_error misclassified): now maps to INVALID per the T01 envelope.
- F6 (sfwp pool idle hygiene): IdleTTL (default 8s < the server's 10s line timeout) retires
  stale pooled connections at acquire; OBSERVED_DEBT entry resolved.
- F7 (NEW, kernel, found by the T05 recovery test under the more deterministic -p 1 gates):
  a kill that landed after the case ledger's durable append but before the terminal correlation
  write left a failed record over a landed effect — the correlated outcome was lost and the
  test's completed/failed dichotomy broke. Fix: the admitted commit's pending record is bound to
  the minted case id (locator) BEFORE any case ledger write (correlation_request_id threaded
  through SubmitPayload -> submit's mint step), and the startup settlement now reconciles
  located commits whose case ledger landed (completed with case_id) instead of reporting
  interrupted. Unit tests: landed -> completed+replay; unlanded -> interrupted; locator never
  rewritten and survives settlement. RequestRecord.schema.json regenerated (additive locator).

## Gate discipline change (evidence-backed)
Full live sweeps failed intermittently (restart/resume/relay tests) because each package's
TestMain boots its own kernel cell — a parallel sweep runs ~5 kernel servers concurrently.
With -p 1 (package serialization) two consecutive sweeps are fully green; within a package Go
runs tests sequentially already. The live gate posture is therefore -p 1 (matches the repo's
cargo jobs=1 discipline). Recorded in the decision log (D-3-followups).

## Gates
- go vet 0; go test -race ./... 0; go test -race -tags live -p 1 ./internal/... 0 (x2 sweeps);
  gofmt clean; fixture-tag build 0
- cargo three-crate 58 suites ok / 0 failed (three-crate-after-reconciliation.log); fmt clean;
  correlation unit tests 13/13
- Go recovery tests x3 green against the reconciling kernel

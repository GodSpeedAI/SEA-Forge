# Independent review — private Next notifier proof addendum

**Disposition: APPROVE this proof clarification for the bounded private Next assignment only.** The production helper decomposition plus a direct ownership test using those helpers and real lease/poller state is sufficient when kept distinct from the required actual worker-publication tests. This approves neither implementation nor runtime behavior.

## Reviewed basis and source

- Original assignment: `run-observation-private-next-assignment-oct09.md`, especially no post-drain notifier Add and external waits (lines 30–37), drain joining per-target notifiers before capacity release (54–60), and required notifier coalescing/ownership and shared-survivor proofs (74–81).
- Sequencing clarification: `run-observation-private-next-assignment-addendum-oct09.md`, operation ownership and callback order (6–19), and one exact projector seam with no mutable hook or fake commit path (21–26).
- New root decision: `run-observation-private-next-notifier-proof-addendum-oct09.md` (lines 1–26).
- Current source: `run_observation_poller_worker.go:9–21,381–443`; `run_observation_manager.go:81–97,772–867`.

## Findings

The addendum closes the test-ownership ambiguity without adding a test-only lifecycle. It requires the production publication path to call the same private registration helper under `manager.mu` and send/release helper after unlocking. Registration takes a real lease’s `notifyWG` reference only for an exact eligible attached target; send remains nonblocking and releases every registered reference even when the capacity-one wake coalesces. The helper unit test uses an actual Prepare lease/poller, calls the real helpers at their required lock boundaries, and lets the actual drain remove counted capacity only after the notification reference is released and drain joins. This needs no manual `WaitGroup.Add`, invented lease state, callback, or mutable hook.

The test claims remain properly separated: that unit test proves production ownership bookkeeping, while separate real worker/read/publication tests must exercise actual wake/coalescing and surviving shared work. This distinction matches the assignment’s notifier/coalescing/shared-survivor proof requirement. The TDD-only inert-stub allowance is explicit and cannot be mistaken for completed behavior.

The present source is still only a scaffold: the helper functions return inert values (`poller_worker.go:14–21`), the lease has a `notifyWG` field (`manager.go:88–96`), and current `finishLeaseDrain` joins creator and worker then removes capacity without yet waiting on that notifier group (`manager.go:838–866`). The original assignment already requires joining notifiers (`assignment:54–56`); therefore the upcoming implementation and test must make that join observable and must not claim this addendum as runtime proof. This is a remaining implementation obligation, not a defect in the proof clarification.

No implementation, test, gate, compiler, or Git operation was performed. The addendum changes no authority, public surface, or four-file source boundary.

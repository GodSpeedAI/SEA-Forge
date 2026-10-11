# Independent semantic review: draining reverse membership

**Verdict: APPROVE the clarified private ref-lifecycle semantics for Phase B source design.** This approves the protocol clarification only; it does not approve implementation or runtime behavior.

## Decision

The clarification consistently separates two meanings that the older wording
left conflated:

- `lease.pollers[key] == entry`, with `!lease.draining`, is the eligible
  attachment membership. It is removed atomically when detach wins.
- `entry.refs[lease]` is the exact reverse ownership association. For an entry
  made watcherless by detach, the association remains as the drain owner's
  record until the actual worker has returned and `workerDone` proves JOIN. It
  is not an eligible watcher, must not authorize another read, and must not
  keep shared work alive by itself.

This is a material clarification of revision 6's ambiguous “refs belong to one
lease and are detached exactly once” wording, not a no-op restatement. The
logical lease membership is released once at the detach transition; its
reverse ownership record can remain until its one required final cleanup. The
existing frozen test requires that distinction: after A's detach wait times
out while A reads remain held, each A poller must still have one `entry.refs`
association, a canceled context, an open `workerDone`, and still consume
capacity (`run_observation_manager_test.go:850-927`, especially 881-897 and
915-934). Deleting both maps immediately would fail that assertion and could
release/reuse an entry before JOIN. The final post-JOIN A/B detach assertions
require capacity to become reusable only after the reads return and drains
complete (925-937).

## Ownership and race cases

The required behavior is coherent across the named races, provided Phase B
uses the eligibility predicate everywhere it decides whether work may start,
continue, or be canceled:

1. **Detach and shared survivor.** Under the manager mutex, the first detach
   marks its lease draining and clears that lease's eligible `pollers`
   membership once. For each entry, another lease is a survivor only if it is
   not draining and its own `pollers[key]` is the exact entry. If such a lease
   exists, remove the departing lease's reverse association, do not cancel or
   join the shared worker, and preserve the survivor's association. If none
   exists, retain the departing lease's reverse association for drain
   ownership, then cancel outside the mutex and retain both lease/entry
   capacity through actual worker completion/JOIN.
2. **Eligibility.** `claimPollerReadStart` must check manager admission,
   exact manager-map entry identity, readable phase, and at least one lease
   satisfying both `!draining` and `lease.pollers[key] == entry`. Do not use
   raw `len(entry.refs)` as eligibility. The same predicate must govern
   watcherless/cancel decisions; otherwise the deliberately retained reverse
   association could prevent cancellation or be mistaken for a surviving
   watcher. Stop or final eligible-ref removal winning the mutex bars later
   claims. A claim that won before Stop remains owned through actual port return
   and JOIN even if the call physically enters after Stop was requested.
3. **Selected-A versus detach.** Both contenders check/remove the exact
   forward membership under the same mutex. If detach wins, A cleanup sees
   absent membership and does not release/cancel again; the detach transaction
   owns the retained reverse association and its join. If selected-A cleanup
   wins, it releases only its own membership. If another eligible lease
   survives, it removes the reverse association without canceling that worker.
   If it makes the entry watcherless, it owns cancellation and actual JOIN
   before capacity reuse or completion of that cleanup. The race fixture
   stages both transitions and checks no residual forward/reverse membership
   after return (`run_observation_manager_failure_test.go:471-535`).
4. **Stop and late launch.** Stop closes manager admission under the mutex and
   snapshots work, then cancels/signals/waits outside it. A creator that already
   reserved an immutable launch list still launches every owned worker exactly
   once before finishing its creator operation; Stop waits for that operation
   before joining workers. The worker's read-start claim then fails if Stop
   won. A claim that linearized first remains owned until actual return/JOIN.
   This follows the corrected proposal and worker-launch clarification and
   introduces no scheduler hook.
5. **Timeout and retry.** Timeout is not completion: keep the lease draining,
   retain the watcherless reverse association and entry reservation, and do not
   close drain completion or remove capacity. A later successful wait/finalizer
   may complete the same lease-owned drain only after the real `workerDone`
   closure. Repeated callers join that drain and must recheck under the mutex
   before deleting associations/closing completion, so at most one transition
   removes each exact relation. The existing reverse map is sufficient to find
   retained watcherless entries if the original waiting call returned; no new
   drain-list field or counter is needed. A retry cannot synthesize JOIN.

The same ownership model applies to shared initializer Stop, one Prepare
rollback with another lease surviving, and failed-Prepare cleanup. Each
Prepare has its own lease and operation. Its first drain transition owns only
that lease's exact membership; another lease's work continues. The creator
finishes its own operation before waiting on a drain that includes it, avoiding
self-join. Initial successful A cleanup uses the same exact membership test; if
it wins before detach it alone owns that selected ref, and if detach wins it
cannot remove it a second time. Actual caller-count/A semantics remain as
specified by the initial-failure clarification.

## Protocol reconciliation and deviations

The old revision 6 language described refs as though both maps disappeared at
the same instant, while the frozen draining-capacity test requires
`entry.refs` to remain populated after the detached lease has lost eligibility.
The root clarification resolves this actual test/protocol ambiguity by
defining the forward map as eligibility and the reverse map as an ownership
record that may outlive eligibility until JOIN. It does not add a state field,
counter, token, callback, API, public contract, cap, or architecture boundary.
Manager mutex ownership and the existing phase/context/ready/workerDone
signals remain sufficient. No normative conflict remains once this explicit
root clarification controls the temporal meaning of “ref”; the original Unit
1 requirements still hold: exact-once membership release, shared-survivor
preservation, no lock-held calls/cancel/wait/JOIN, and capacity retained until
actual work and JOIN finish.

The clarification is stricter than its explicit read-claim sentence alone:
Phase B must apply the not-draining/exact-forward-membership predicate also to
survivor detection and cancellation decisions. That is a necessary consequence
of calling the reverse association noneligible, and is stated above as an
implementation requirement. It must not treat a raw reverse-map count as a
live watcher count.

## Evidence, limits, and provenance

Reviewed root clarification SHA-256
`013d9f13d8d1e5155932e26f9f6032ee8c24932fdba7361468ef611d867c4609`; prior
ref-semantics recon result SHA-256
`dc9a57458aa37ad702b92aecec848f499da5a9008edc772aae75a88560d10c0c`; root
exact-ref ownership decision SHA-256
`5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`;
revision 6 SHA-256
`60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b` and
addendum SHA-256
`9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`;
preregistration SHA-256
`f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`;
corrected read-start proposal SHA-256
`979fbdf521377c58f2615a2b3111c089fa9e0869fd350a0e4c973ae3c9166f8b`;
approved supplement SHA-256
`4752e00fa7cc4001d220635cb4aebda4cd63df61b43b2219d60beddff5e114c0` and
review SHA-256
`a3b59162436f5031d5ace6127bf847f00458bc9c14ed55f3cf306522a15e17e7`.

The cited frozen manager fixture is `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`;
the reviewed Phase A failure fixture is
`e63e050a7999b5c239257e57c6642baa346c59a40dfa1871e58d7b64ef80a5cf`. The
earlier recon rejection accurately identified an unresolved meaning before
this root clarification; it is not rewritten. This approval is source-only
protocol approval and gives no lifecycle implementation or runtime claim. No
Go command, test, formatter, scanner, source edit, or Git mutation was run.

# Independent review: manager revision 6 terminal retention lifecycle correction

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-revision6-retention-initializer-correction-revision3-oct06.md`, SHA-256 `b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`.  
Disposition: **APPROVE as a complete DOCONLY repair to the identified revision 2 ambiguities, when composed with frozen revision 6, its addendum, and correction revision 2.** This does not approve the proposed caps or any source, test, public API, or runtime work.

## Reviewed bundle and evidence

I read the original manager repair assignment, frozen revision 6, revision 6 independent review, revision 6 addendum and its independent review, correction revision 2 and its independent review, and the complete frozen correction revision 3. The current correction hash matches the builder's reported value.

I also checked the accepted Go call path that the lifecycle text relies on. `RunTracePort.ReadRunTrace` calls `Client.Do` before decoding/returning the snapshot (`internal/adapters/sfwp/run_trace.go:67-84`). `Client.Do` completes its `roundTrip` call(s) before returning (`internal/adapters/sfwp/client.go:417-435`); `physicalAttempt` defers connection release/discard and then permit release, and both defers run before it returns (`client.go:490-530`; the permit contract states it remains owned through connection cleanup at `run_get_admission.go:19-24`). Therefore a returned `ReadRunTrace` candidate has completed that call's connection cleanup and run-get permit retirement. The correction still correctly requires joining the separate poller worker before releasing the manager entry/slot.

## Assessment

The correction makes the key lifecycle distinction clear: rejected source terminal data does not become current, retained, or disclosed, while a separate safe manager-generated `terminalRetentionFailure`/stop marker may be committed. It makes the stop marker prevent future read starts before worker join, allows the in-progress read/client retirement to finish, then has the worker exit and close its completion channel. The teardown owner joins the worker and drains refs, lease operations, and notifier refs outside the manager mutex. Poller and lease capacity remain counted until actual port/client retirement, worker join, and owned drains complete; timeout does not free capacity. This resolves the prior circular “stop polling after join” sequence and preserves the original join-before-release invariant.

The fixed-width private control image is internally consistent with the proposed exact serialized-size cap: all bounded lifecycle controls are present from the start and encoded at fixed width in both unset and set states. A marker-only state change therefore cannot push a previously exact-limit image over the cap or require evicting an ID/frame. It changes no public or persisted representation and explicitly provides no heap/RSS guarantee. Its accounting and boundary tests remain gated on operator approval.

The attach result contract is now exact for `current`, `sharedInitializer`, `newInitializer`, and the no-reference dispositions. Only `newInitializer` carries the manager-owned generation token; each successful result owns one exact ref released once. The correction retains prior manager-owned initializer publication and surviving-watcher behavior. The nonterminal retention path remains transient, zero/no-advance, and recoverable on a later full-ledger-fitting candidate; terminal retention failure is distinct. `ErrPollerReadUnavailable` remains separately transient. No old version becomes current after rejection, and rejected candidate details are withheld from returned data and client-visible errors/logs.

## Differences and release boundary

Revision 3 supersedes only correction revision 2's terminal-retention lifecycle wording, its terminal test sequence, and private budget-image control sizing. It does not alter the captured-ledger filter, manager-owned initializer token, auth/guard, DTO, count, source-window, physical admission, or read-failure contracts. The 16 cohort slots, 128 attachments, 1 MiB per-poller image, and all overflow policies remain explicitly proposed and require exact operator approval before implementation. Revision 3 authorizes no code or test work; no runtime evidence is claimed.

No unresolved defect remains in this narrow correction. The parent documents continue to govern all other requirements, including one captured immutable version/ledger boundary, actual joins before capacity release, caller-specific authorization, no lock-held waits/cancellation/joins, no public V4/SSE/frontier work, and no T09 settlement claim.

No tests, compiler, scanner, Git, Graft build, or network action was run.

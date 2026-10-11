# Root clarification: claimed cancellation and owned worker launch

This resolves the three findings in the independent read-start proposal review.
It releases no source. The eligibility linearization model is independently
accepted; these missing branches must appear in the fresh corrected proposal.

## Cancellation after eligibility but before actual first invocation

If the worker wins eligibility but its manager-owned context prevents the first
actual RunTracePort invocation, there is still no accepted current value or
actual read attempt. Use initializer code4, phase2-or3, nil current, resolve ready
once outside the mutex and complete the real worker/workerDone path. A stopped
or canceled Prepare returns its typed error, zero wrapper and nil lease with
owned rollback. Do not invent an A outcome for an unperformed read or increment
ReadsAttempted for the eligibility check. Capacity remains through actual JOIN.

## A reserved worker always launches

The Prepare that creates entries owns an immutable launch list. After reserving
all selected entries under the mutex, it unlocks and launches each owned worker
exactly once, including entries whose stop/cancellation already won. Launch all
entries before waiting for any result or returning/rolling back the Prepare.
The stopped worker performs the real code4/readiness/completion path; neither
Stop nor another Prepare launches it again.

Register Prepare operation ownership before reservation. Stop may mark stopping
and cancel owned contexts while a creator is between reservation and launch,
but it must wait outside the mutex for that creator's operation and actual
workers, keeping reservations/capacity occupied. The creator must not wait on its
own unfinished operation registration during rollback.

A genuine private reserve-batch/launch-batch boundary used by production Prepare
can make the test sequence deterministic: reserve through the real method,
order Stop's real transition, then perform the creator's actual launch batch.
Do not expose a callback or scheduler hook. The fresh builder must name the
minimal real private methods/state and their compile-safe fixture seams before
release. All-eight-before-wait remains required.

## Reconcile source identities explicitly

Current encoder fixture is cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256,
after an independently reviewed five-field whitespace correction from8e741.
Actual focused GREEN accepted7 top-level/3 nested, with all8 captures cmp0.
The private key extraction subsequently changed only the untracked manager
declaration from fa1601 to ab9f1c; its independent review is a separate unit.
Future lifecycle proposal must cite current hashes, preserve historical records,
and name its newly permitted manager-field/source exceptions exactly.

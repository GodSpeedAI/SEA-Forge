# Failed Prepare cleanup evidence

Date: 2026-10-08
Applies to source grant bfcfa5cd and frozen fixture4934e9c5.

Groups 3-5 must inspect existing ownership immediately after failed Prepare
returns, before t.Cleanup can remove a leaked registration: cohorts and
pollers both empty. Groups 4-5 additionally assert the captured actual
entry.workerDone is already closed, since the held actual call was released
and owned failure cleanup must have joined it before return.

Both group-6 invalidation scenarios must assert the captured invalid lease
is absent from cohorts and forward/reverse entry membership before teardown,
while the authorized survivor's cohort, exact eligible entry and valid
retained value remain registered. Keep correct owner/shared call counts.

Read-only mutex observations and existing completion channels supply this
evidence. Do not mutate lifecycle state, add production fields/hooks, weaken
assertions, or use global Stop/teardown as proof of the failed Prepare cleanup.
This clarifies the grant's existing exact cleanup requirement. No source edit
or execution is released by this document. Await immutable critic verdict,
then a fresh different builder receives the complete bounded repair grant.

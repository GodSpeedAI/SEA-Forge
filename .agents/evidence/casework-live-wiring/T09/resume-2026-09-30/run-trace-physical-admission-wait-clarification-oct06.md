# Physical admission wait clarification — independent review required

Binding supplement to the root adjudication. The repair proposal's generic
nearest-idle-deadline rule must not wake repeatedly on an already expired,
unrelated idle slot when same-run exclusion still prevents admission.

If the requested run matches a busy record, wait only for the change signal or
context cancellation. If it matches an idle cooling record, wait for that record's
future eligibility, change or cancellation. An eligible matching idle record is
acquired directly. Never use an unrelated record to bypass same-run exclusion.

For an absent run, acquire any eligible slot according to the adjudicated ordering.
If none is eligible, wait for the earliest FUTURE deadline among idle cooling slots,
change or cancellation. If all slots are busy, use change/cancellation only.
Re-evaluate under the mutex after every wake; no zero-duration retry loop is allowed.

This resolves a potential CPU spin in the proposed waiting algorithm. It changes
no source or timer implementation. Independent source review and then an observable
fixture for this same-run-busy/unrelated-expired-slot case remain required.

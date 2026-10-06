# Physical admission repair — root adjudication, review still required

Reviewed the complete repair proposal and the independent rejection. This records
root architecture choices for a new independent source review; it releases no
fixtures or implementation and does not settle T09.

## Decisions and binding clarifications

Accept admission BEFORE connection acquisition, inside the existing roundTrip
timeout context. This differs materially from the initial proposal. It prevents
queue waits from occupying pooled sockets or starting connection cancellation
callbacks prematurely. The permit still marks the actual physical write boundary
inside conn.call, after existing deadline/cancellation setup. Other verbs bypass
admission and preserve existing behavior.

Accept exactly two retained slot records and conservative post-write cooldown.
Unexpired idle records cannot be replaced; busy records cannot be reused even if
their cooldown has elapsed. This intentionally reduces aggregate throughput and
can produce per-call availability failures while queued. The approved amendment
promises upper resource bounds, not completion of all eight reads within a fixed
cohort time. No new cohort deadline or successful-completion guarantee is added.

Clarify the repair algorithm: if the requested run already names a record,
acquisition MUST use that record when eligible, or wait when it is busy/cooling.
It must never fall through to another empty/eligible record for that same run.
For an absent run, prefer the first empty record, otherwise an eligible idle record
with earliest eligibility and slot-index tie-break. No record replacement occurs
before cooldown expiry. Check ctx.Err before granting admission or beginning a
write. This closes the repair's otherwise ambiguous step-one/step-two transition.

Accept the proposed RunGetAdmission and RunGetPermit interfaces and Config field.
The production constructor is proposed as NewRunGetLimiter() *RunGetLimiter;
it fixes two slots and a one-second floor, with no production weakening option.
Test-only package-private construction may shorten intervals for bounded timing
fixtures. Source review must verify this does not expose a production bypass.
The CLI creates one owner and supplies it to EVERY live client. Artifact ownership
run_get participates in the same limit. Nil remains a legacy isolated-test option;
production constructor coverage is a required proof, not assumed enforcement.

One physical helper owns acquisition, connection and permit. Its first registered
defer releases the permit; its later defer retires/releases the connection. It calls
conn.call (which joins its cancellation callback), processes DecodeResponse, and
returns only after connection cleanup then permit release in Go LIFO order.
Every error/early return and safe/busy retry uses that helper. No helper starts a
retry while its previous permit remains owned. Preserve exact existing transport,
refusal, mutation recovery, Ask no-resend and response-cap behavior.

WriteAttemptStarted occurs immediately before the first payload Write. Finish is
the return of the last attempted payload/LF write, including partial/error writes;
it must be recorded before response reading, not delayed until that read finishes.
SetWriteDeadline failure or observed cancellation before any Write has no cooldown.
A deferred release is idempotent and cannot forget a started-but-unfinished attempt:
the implementation must guarantee Finish on every attempted-write return path.

The manager counts a logical attempt when invoking ReadRunTrace, including queued
failure. Retries do not consume additional logical units. Continue through the
already selected candidates after an individual failure while the session remains
valid, never select replacements or exceed eight. Session cancellation stops future
starts and joins outstanding calls. Honest result classification uses existing
unreadable/unavailable counters and observation_state semantics; any failed candidate
prevents a complete claim. The physical limiter neither creates nor publishes those
counts. Exact envelope classification remains a separately bounded manager task.

## Next review

Give an independent critic the original proposal, its rejection, the builder's
original repair instructions, complete repair proposal and this adjudication.
It must verify all material deviations, actual construction/send/defer sites,
slot/cooldown proof, original logical budget and timeout consequences with source
evidence. Missing evidence or an unresolved compatibility issue requires rejection.

No code, compile, runtime, Git or publication claim is made by this document.

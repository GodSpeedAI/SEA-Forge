# Physical run.get admission proposal — source review required

Root retains architecture and semantics. Read the approved T09 observation
requirements and run-trace-read-pacing-source-recon-oct05.md. One logical Do can
send four physical run_get requests; manager-level timing cannot prove physical
spacing. No builder/production change is released by this proposal.

## Root design

Install one process-shared admission owner on every production SFWP client that
can send run_get. Integrate only at the existing physical call's write boundary,
after connection acquisition and cancellation/deadline setup, before writing.
Every initial, safe transport retry, server-busy retry and nested safe retry must
pass that boundary. All other verbs bypass this read-only admission mechanism;
request bytes, IDs, principal/delegation, refusal classes and retry policy stay
unchanged. There is no new kernel verb, public HTTP DTO or identity model.

Use exactly two slot records: run ID, busy flag and next-eligible time. No map
of every previously seen run, unbounded queue or per-session timer registry.
Under a short limiter mutex, an acquisition succeeds only when an eligible slot
exists and no busy/cooling slot names the same exact run ID. Otherwise wait
context-cancellably on a change signal and, when relevant, the nearest cooldown
deadline. No limiter mutex is held during network work or waits. Context expiry
returns the existing typed availability failure; do not invent a public reason.

After any attempted physical write completes, record next eligibility no earlier
than write-completion time plus one second. This conservatively proves a full
second between actual write starts even when a write blocks or fails partially.
Keep the slot busy until response processing and owned connection cancellation,
retirement, callback join and release have actually finished. Register admission
release cleanup so it runs AFTER existing connection cleanup, preserving Unit5A.
An acquired call canceled before any write releases without a cooldown. Release
marks the slot idle but retains its cooldown; no background release goroutine is
needed. Expired idle records may be reused on the next acquisition.

This deliberately holds the global admission slot through its cooldown, bounding
tracking memory to two records and limiting aggregate throughput conservatively
to roughly two completed write admissions per second. It satisfies upper bounds,
not a throughput guarantee. Critic must assess initial-cohort timeout/availability
consequences and the exact approved eight-head budget semantics before approval;
do not silently reinterpret logical versus physical budget accounting.

Production assembly supplies this shared owner before starting any run.get reader;
no client that serves production run.get may silently omit the guard. Test doubles
may keep the old nil-guard behavior. The exact constructor/hook signatures and
all production assembly sites must be source-inventoried before fixture release.
Admission sees only the unchanged run ID, never trace bytes, actor profiles,
cookies, environment or other sensitive data. No diagnostic values are logged.

## Test-first inventory and independent review

After source proposal approval, a bounded builder first adds observable fixtures
and minimal declarations for compiling assertion RED. Test both same-run and
different-run contention; no more than two physical reads; one-second production
configuration; cooldown measured after a blocked/partial write; all four retry
paths; prewrite cancellation; postwrite retirement/join before slot release;
deadline waiting and bounded two-record storage; unchanged Ask/mutation/no-resend
and refusal behavior. Use controllable test timing, not flaky sleeps or changed
production limits. Preserve all cancellation fixtures and Golden_UPDATE policy.

Independent critic receives this original proposal plus source facts/full result.
It must identify material deviations and exact call/defer ordering, verify every
physical send path and production owner, and reject any unproved bound or budget
interpretation. Run scoped tests, full SFWP/module race and canonical Go gates only
under separately assigned single compiler ownership with RAM checks/exact captures.
Source approval cannot claim poller, session drain, SSE/UI or T09 completion.

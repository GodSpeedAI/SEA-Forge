# Protected Ask runtime implementation assignment

Prepared by root for the builder following independently approved expected-RED Ask fixtures.
Do not start implementation before independent fixture review and observed expected RED.
Read ask-test-first-assignment.md, ask-route-source-preparation.md, approved proposal and
runtime-integration-decisions.md plus actual source/tests. Existing identity/governance and
kernel APIs remain authoritative. No dependencies, new Rust verbs, request correlation,
schema/golden weakening or UI runtime changes. Native apply_patch only; builder does not compile.

Own the new protected Ask adapter/server implementation and narrow integration edits required
in ports declarations, server/http.go/server.go/ratelimit.go, SFWP frame.go/client.go and live
main.go. Preserve the independently frozen tests. Prefer new ask.go files over unrelated edits.
Root must bound exact file ownership against concurrent work before authorizing implementation.

Application AskPort receives effective ActorClaim separately from semantic AskQuestion.
Configured adapter supplies the service gateway principal and on_behalf_of actor, plus the
wire actor_id equal to effective ActorID. Use actual verb ask, with no request_id. Preserve
existing authority constructors and HTTP constructor signatures through Options.Ask injection.
Do not use Options.Perspective as service identity. Nil Ask capability fails honestly unavailable.

Register POST /api/ask with requireSession, requireCSRF, then separate Ask rate limiter.
Use existing actual session/bearer and remote-IP conventions, separate session6/min burst2
and IP20/min burst4 buckets, without changing intent accounting. Refused auth/CSRF/rate or
invalid input never dispatches Ask. Verify current session perspective/effective actor using
the existing governance boundary; the kernel still authorizes every protected Ask.

Strict raw body maximum8192 bytes; one nonnull JSON object with kind/subject/purpose?/case?
only. Reject unknown fields (including browser identity), null/wrong types, arrays/scalars,
malformed/trailing nonwhitespace JSON and excess bytes. Nine exact kinds; reject trimmed-empty
subject/provided case while preserving valid supplied values. Purpose omitted planning,
supplied empty allowed, UTF8 byte length maximum500. Do not reuse the shared decoder that
accepts certain trailing errors or larger bodies. No kernel call on invalid input.

Return the complete actual AskAnswer mapped to the canonical ThothAnswerView without dropping
claim disclosures or optional capability_record_ref. Preserve answered/partial/denied as200
answer bodies; typed transport/identity failures remain errors. Keep every reference, omitted
class, freshness, assurance, limitation, authority notice and timestamp. Do not fabricate
claims, citations, authority, directives or response defaults. Unexpected malformed/unsupported
upstream disclosure views must fail honestly rather than invent missing fields or enum values.

Separate transport retry safety from the existing correlated mutation map. Ask EOF/timeout/
overflow after uncertain send is typed unavailable with exactly one Ask and zero status
recovery calls. Existing inspect fresh retry and correlated mutation recovery are unchanged.
The explicit pre-admission server_busy refusal retains its separate single bounded retry;
it is not an ambiguous transport failure. Correct comments equating protected operations with
durable correlation without adding Ask to request_get_status recovery.

Freeze with exact diff/source facts/material differences. Independent critic gets this original
assignment and actual result; re-runs focused fixtures, full Go required gates and delegated
real kernel Ask proof with ledger attribution/disclosure evidence. Actual host RAM/process
preflight before every compiler; single token, Go256MiB/GOGC50/GOMAXPROCS2/p1/race/count1.
Retain original complete stdout/stderr and immutable failures. Approval requires evidence;
fresh builder after rejection. Full T09 settlement still requires the remaining runtime units.

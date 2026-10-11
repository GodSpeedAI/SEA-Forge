# Protected Ask source preparation

Read-only Luna research traced existing routes, middleware and actual kernel admission.
This is source preparation, not implementation or runtime proof.

Register POST /api/ask with session, then CSRF, then independent Ask limiter, then handler.
Use session IDs and existing bearer fallback/IP extraction; do not trust forwarded IP headers
or change intent limiter accounting. Approved Ask rates are session6/min burst2, IP20/min burst4.
Strict8KiB object decoding must reject unknown fields, null/wrong types and any trailing bytes
other than whitespace before contacting the kernel. Existing decodeStrict has a1MiB cap and
accepts some trailing decode errors, so it is insufficient for this new route.

The request has kind/subject/optional purpose/optional case only. Validate nine approved kinds,
trim-nonempty subject/provided case, omitted purpose planning, supplied empty purpose allowed,
500UTF8-byte ceiling. Use verified session actor and existing service governance block plus
on_behalf_of. Preserve the entire actual ThothAnswerView, including all returned disclosures,
claim references and optional capability_record_ref; do not invent citations or directives.

Important recovery boundary: kernel Request::Ask has no request_id (server lib.rs727-745).
request_id/ requires_durable_locator matches exclude it (lib.rs1429-1469); identity::is_protected
includes it. Handler lib.rs2267 writes through Thoth service, but no correlated status record
exists. Therefore ambiguous Ask EOF/overflow must return typed unavailable after one send,
with zero request_get_status calls. Do not fabricate Ask correlation or extend the kernel API.
Keep existing correlated mutation recovery and inspect retry intact; add the minimal explicit
transport retry-safety distinction for uncorrelated Ask. This is a material limit on recovery,
not permission to resend. An explicit server_busy refusal is pre-admission/no_side_effect by
dispatch_bounded lib.rs1724-1732, so its existing bounded retry remains separately safe.

Affected owners: Go server.go/http.go/ratelimit.go; ports and SFWP adapter frame.go/client.go;
focused server Ask tests and adapter single-send tests. Nearby production_test.go and
intent_authorization_test.go provide actual session/CSRF/limiter/no-dispatch test patterns.
Real delegated kernel Ask proof is required; in-process handler tests alone miss the socket gate.
No dependency, identity, Rust verb or request-correlation API change is needed.

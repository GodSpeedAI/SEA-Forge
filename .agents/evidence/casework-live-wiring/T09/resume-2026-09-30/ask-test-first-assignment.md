# Protected Ask test-first assignment

Builder: Luna t09_ask_test_builder. Read-only preparation completed before authorization.
Root owns architecture and integration; this assignment authorizes fixtures and minimal
declarations only. No compilation by builder; the independent critic receives this original
assignment and actual result, reviews completeness/deviations, and runs expected RED.

Own new internal/server/ask_test.go, internal/adapters/sfwp/ask_transport_test.go,
new internal/ports/ask.go and only the Options.Ask declaration in server/http.go.
Read files and nearby tests before native apply_patch edits. No dependency, Rust/identity,
canonical contract, mirror, golden, cap implementation, route or transport behavior changes.

Architecture: separate application-owned ports.AskPort, AskQuestion and complete AskAnswer
and claim disclosures; no SFWP types or untyped payload holes in the application core.
Inject through Options.Ask without reshaping existing constructor signatures. Runtime
production injection belongs to the following implementation unit. Subject/purpose/case
values preserve the approved request semantics; optional case retains absence. Same-package
transport tests can use newRequest("ask") plus exact fields without implementing NewAsk.
AskPort.Ask receives context, effective ActorClaim and AskQuestion as separate parameters.
The question contains semantic fields only. The configured adapter owns the service gateway
claim and constructs the existing Governance actor/on_behalf_of block; Options.Perspective
is never repurposed as gateway identity. Server tests pin the effective actor; adapter tests
pin the full wire principal pair. Writes are held until mirror Go gates finish, so those gates
observe a stable source graph rather than partially introduced Ask declarations or fixtures.

Server fixtures prove session then CSRF then distinct Ask session6/min burst2 and IP20/min
burst4 limits. Authentication/CSRF/rate/validation failures make zero Ask calls. Verify
separate intent accounting, real remote address rather than forwarded headers, refill and
session/IP isolation. Verify effective session actor, perspective refusal and kernel dispatch.

Strict single-object body limit8192 bytes: reject unknown actor/role/other fields, null or
wrong field types, array/scalar roots and malformed or trailing nonwhitespace JSON. Allow
trailing whitespace within the raw limit. All nine kinds; trimmed nonempty subject/provided
case; omitted purpose planning, supplied empty allowed; purpose500UTF8-byte limit including
multibyte boundaries. Test exact8192 and over8192 raw bodies and zero dispatch on rejection.

Preserve every answered/partial/denied Thoth answer/claim disclosure and reference, including
optional capability_record_ref, omitted classes, freshness, assurance, limitations, authority
notice and timestamp. Governed partial/denied are successful answer bodies, never authority.

Socket fixtures prove actual wire verb ask with no request_id.
The protected wire actor_id equals the effective session ActorID, alongside the service
actor and on_behalf_of pair; no browser actor_id nomination is accepted. Reject trimmed-empty
subject/case while preserving otherwise valid supplied content, as the approved proposal states.
Uncertain EOF/timeout/overflow
returns typed unavailable after exactly one Ask send, zero request_get_status, and no resend.
Explicit pre-admission server_busy refusal permits only the separate bounded safe retry.
Existing inspect retry and correlated mutation recovery remain unchanged and separately tested.
Use owned socket/net.Pipe fixtures and meaningful delivery/call counters, avoiding idle-timeout
false positives. Actual delegated kernel proof is required later; unit tests cannot replace it.

Independent critic must explain every material difference and cite source/diff/commands/logs.
Before each compile/test inspect actual HOST RAM/processes; one compiler token, Go256MiB,
GOGC50/GOMAXPROCS2/p1/race/count1. Retain original full logs immutably, no reconstructed logs.
Fresh builder after rejection; no task settlement by self-verification.

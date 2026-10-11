# T09 runtime integration decisions

Root architecture preparation, not runtime approval or observed passing evidence.

## Ask authority boundary

The application Ask port receives the verified effective ActorClaim separately from the
semantic question. The configured adapter owns the gateway service principal and supplies
the existing Governance actor/on_behalf_of pair. The projection Perspective option is an
end-user perspective and must not become a service identity. Keep constructor compatibility
through Options.Ask; configure the adapter explicitly in the existing live stack assembly.
Kernel Ask is protected and can write durable records without request correlation. Ambiguous
transport outcomes therefore remain unavailable after one send; no status lookup or resend.
Application ports AskClaim/AskAnswer remain semantic value structs without JSON wire tags,
following neighboring ports models. The adapter translates the kernel view into those values;
the HTTP handler explicitly maps them into canonical ThothAnswerView. This avoids treating
domain structs as an accidental HTTP codec while preserving every disclosure field.

## Exact numeric observation metadata

Canonical exit_code is a JavaScript number, while the actual kernel uses optional i64.
Projection must validate exact representability using the JavaScript safe integer interval
before exposing an exit code. An out-of-range recorded value makes that read unavailable;
never round, stringify into an incompatible field, silently omit a present value or fabricate
a replacement. Preserve any earlier validated run observation with explicit unavailable state
on a failed subsequent read. This representation limit needs focused boundary tests later.

## Run children and retained facts

The approved proposal requires one execution_trace child per case-owned run, preserving the
real run ID and actual horizon item parent, with captured execution/settlement standing.
World child count is distinct from the eight-run live hydration selection limit. CaseFacts
may retain the factual run summaries needed for those captured children, but must never
retain RunTraceObservation buffers, frame identities or current poller state. Live trace
metadata belongs exclusively to the separately authorized informational SSE side channel.
Unknown run ownership or a missing actual horizon parent must not fabricate a child link.
Existing run-artifact provenance validation remains a narrow separate concern; safe observation
decoding must not quietly relax its ownership or artifact authorization checks.

## Verification sequencing

Canonical bounded unit independently passes focused Bun and explicit strict TypeScript.
The mirror critic receives the original assignment and actual frozen source. Ask fixtures
and minimal declarations are held while mirror broader Go checks run, preserving one stable
source graph. Source rejection requires a fresh builder before repeated independent review.
No final T09 settlement until protected Ask, observations, UI wiring and required global gates
have independent evidence. Preserve all earlier raw failures and append corrections separately.

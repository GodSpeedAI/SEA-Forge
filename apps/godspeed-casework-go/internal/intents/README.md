# intents

## Overview
Translation of the canonical journey intents (the consequential set) onto exactly one governed SFWP
verb call each, through the `ports.CaseAuthorityPort`, with the effective actor carried as
`on_behalf_of`: the gateway principal is the request's actor, the end user rides beside it, and the
kernel — never the gateway — verifies both and ledger-pronounces the pair. Refusals are OUTCOMES
(HTTP 200), never transport errors, carrying a typed kind plus a plain-language message.

## Interface
- `Handler` — the intent translator: `NewHandler(auth ports.CaseAuthorityPort, cursors CursorSource, lister CaseLister, opts Options)`, `Handle(ctx, contract.InteractionIntent) contract.IntentResponse` [intents.go:Handler]
- `CursorSource`, `CaseLister` — injected cursor and listing dependencies [intents.go:CursorSource]
- `Options` — tunable timeouts and the post-mutation wait (`postMutationWait` 5 s) [intents.go:Options] [intents.go:postMutationWait]
- Typed refusal kinds — `RefUnavailable`, `RefInvalid` plus the T01 vocabulary pinned by `contract.AllRefusalKinds` (e.g. `UNAUTHORIZED_ROLE`, `SOD_VIOLATION`, `AUTHORITY_DENIED`, `STALE_PROJECTION`, `JUSTIFICATION_REQUIRED`) [intents.go:RefUnavailable] [intents.go:package doc]
- `mapKernelRefusal` — maps kernel verdicts (apperr kind + verbatim class) onto the refusal vocabulary [intents.go:mapKernelRefusal]
- Role gateway guards — `roleWouldOffer`, `isConsequentialAction` (gateway-level guard BEFORE any kernel call) [intents.go:roleWouldOffer]
- Idempotency — `bodyHash`, `replay`, `record` keyed on intent id + body hash [intents.go:bodyHash]
- `kernelItemKind`, `discretionaryWritePath`, `proposedObject` — intent payload mapping helpers [intents.go:kernelItemKind]

## Invariants
- Only `CONSEQUENTIAL_CASE` intents cross this gateway; anything else is refused `INVALID` [intents.go:validate]
- The intent id is the SFWP request_id, the log correlation id, and the replay-cache key; it is bounded and kept free of control bytes [intents.go:Handle]
- Refusals are HTTP-200 outcomes carrying the typed kind and message; kernel refusal classes pass through verbatim for `AUTHORITY_DENIED` [intents.go:package doc]
- The gateway-level role guard runs BEFORE any kernel call [intents.go:roleWouldOffer]
- Mandatory justification for discretionary work, human-task completion, and approval decisions (`RefJustificationNeeded`) [intents.go:validate]
- Same id + identical body replays the recorded response; same id + different body is refused `INVALID` [intents.go:replay]
- Client-asserted actor is overwritten with the session's kernel standing (session layer); the kernel re-verifies each delegation [intents.go:package doc]

## Side Effects
- Calls governed kernel verbs through `ports.CaseAuthorityPort` (commit case, decide approval, add item, advance, execute item, complete human task) [intents.go:execute]
- Waits for the post-mutation kernel cursor after mutating verbs before answering [intents.go:postMutationCursor]
- Records idempotency entries in process memory [intents.go:record]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)

# Root adjudication of first production runtime rejection

The independent focused race run rejects runtime approval. Timestamp and bounded
ring source approval remain scoped; no full trace/Go/T09 success is claimed.

## Empty frame fixture contradiction

The root empty-slice erratum correctly permits nil for semantic empty success,
with later wire mapping responsible for canonical arrays. Its assertion that
existing empty fixtures consistently expect nil was incomplete: the retention
count table builds a nonnil zero-length expected slice for count0, whereas the
presence table explicitly expects nil. Preserve the erratum and historical
approvals; this new record corrects that missed fixture inconsistency.

A fresh builder must change ONLY the count0 expected-frame initialization to
nil, leaving counts1024/1027, order, total count, zero-error snapshots, allocation
capacity and all other assertions unchanged. This aligns the existing semantic
nil policy rather than broadening the accepted runtime behavior. Do not remove
the empty-input case or change runtime output merely to fit inconsistent tests.

## Response decode classification

The original production assignment requires decode/projection failures to be
typed unavailable with a zero snapshot and no decoder payload in its message.
frame.go DecodeResponse:547-557 wraps malformed raw responses as an internal
error with Op=decode before the adapter's projection decoder is called. The
port currently passes every Client.Do error through, violating that instruction.

Normalize ONLY non-refusal response-decoder errors at this port boundary to the
existing fixed runTraceUnavailable error, without wrapping decoder causes. Use
typed errors.As inspection of apperr.Error.Op and preserve any Refusal in the
error chain first. Do not classify by error prose, normalize all internal errors,
alter Client.Do/DecodeResponse, change retries or suppress legitimate authority
refusal classes. Preserve nondecoder transport/setup failures as before. Existing
raw-invalid-JSON tests retain their unavailable/zero-snapshot expectations.

The fresh repair owns only run_trace.go and its count0 fixture initialization.
Read the original assignment, V3, erratum, every actual builder/review and latest
runtime rejection. No compiler/scanner/Git/status/debt; independent trace critic
must recheck complete source and run the focused race gate under a new sole token.
Capture exact original instructions, full diff, hashes and deviations in a NEW
immutable builder record. No test weakening or runtime approval by the builder.

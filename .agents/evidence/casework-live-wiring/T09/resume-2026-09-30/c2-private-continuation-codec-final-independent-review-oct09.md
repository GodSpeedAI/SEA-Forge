# Private continuation codec: independent bounded review

**Verdict:** Approved for the bounded private codec source unit and the checks
listed here. This is not approval of T09 integration, public exposure, or the
shared ledger decoder.

## Scope and source

Reviewed against the private codec grant and cap/input addendum. The code remains
private and unwired. Final source SHA-256 is
`b23cedb3e86681d06750f0012479af4c3ef124b8fdf04f81ae008c73d1de1d26`; the
module declaration SHA-256 is
`11302429bfdb29634c8d780546c42de2bd10e0fdc7d49ce6bc37980a7e12f876`.
The bounded source validates fixed framing and size limits, canonical values,
context digests, position/filter constraints, and strict signature suffix
shape before JSON decoding. The shared ledger helper was not changed. Encoder
checks establish payload structure only; callers must still establish the
context and ACK/frontier authority represented by the payload.

## Independent evidence

- GREEN04: all six archived captures matched their actual `source_capture`
  bytes, hashes, and lengths; actual and preflight exits were 0; the targeted
  18 tests passed with 0 failures.
- `just check` check02: all six archived captures matched actual source bytes,
  hashes, and lengths; actual and preflight exits were 0; output reported all
  gates green.
- Graft02: all six archived captures matched actual source bytes, hashes, and
  lengths; actual and preflight exits were 0; output reported 8,059 nodes,
  16,073 edges, and 713 cards.
- Earlier full-server capture: 449 passed, 0 failed, 2 ignored, but it tested
  pre-repair source `b69338947dca0a35f4a08f96c57d37163ac282d0902cfc2454e4f776901b6bbb`.
  It is historical evidence, not a post-repair full-server result.

## Material corrections and limits

The absent-To digest fixture was corrected after independent recomputation of
the domain-framed SHA-256 vectors. A frontier test was corrected because an
ACK ending exactly at the pinned start is valid non-overlap; the adjacent case
remains covered and the invalid case uses a later start. The shared decoder
accepts trailing text after Base64 padding, so the private codec gained a
canonical signature-shape check and regression vectors for padding tampering,
signature-byte tampering, and noncanonical pad bits. These changes did not
weaken assertions. Formatting was applied mechanically. The final Clippy fix
changed only the test expression at line 497 from slicing before `as_bytes()`
to slicing the byte view; reversing that expression reproduces the prior source
hash. The earlier full test-source preimage was not available for a whole-file
byte comparison; the corrections and final tests are recorded above.

The docs60 builder used an escalated subprocess write after its native patch
attempt encountered a read-only context error, contrary to the required native
`apply_patch` workflow. The resulting status and debt text were independently
read and checked; this workflow deviation is disclosed, not represented as
compliant.

CW-43 remains open for shared-helper compatibility and caller review. Startup
signer lifecycle, ledger reader, filter/dispatch integration, settlement, and
the remainder of T09 remain outside this approval. Keep T09 partial and stop
before T13; no publication approval is granted.

# Private continuation codec cap and input clarification

**Status: clarification addendum for fresh independent review.** This new
record clarifies the cap-vector classification and input type in the held
grant. It does not edit or supersede either reviewed record; the grant remains
held for independent review and explicit root implementation release.

## Records clarified

- Grant: `c2-private-continuation-codec-root-grant-oct09.md`, SHA-256
  `19be806076544b1f06323c6186bf8aa1484eef4831d907b6827a213c62f16906`.
- Independent review: `c2-private-continuation-codec-independent-review-oct09.md`,
  SHA-256
  `31e58f246fb7639a1b67926efeaf416d0e48cde8508eeb4f31b267194475b37e`.

## Input and safe-layout clarification

The private decoder accepts `&str`. Its input is already valid UTF-8, and
`str::len()` measures UTF-8 bytes. The token's existing outer cap remains
4,096 bytes, while the payload cap remains 2,048 bytes and the separator plus
fixed signature suffix remains 97 bytes. A structurally valid token can
therefore be at most 2,145 bytes; no valid token can be 4,096 bytes.

Before making borrowed slices, derive the separator and suffix offsets with
checked arithmetic and enforce the outer and payload caps. Use checked string
access for the candidate payload and suffix so a boundary inside a UTF-8 code
point is rejected safely. Require a nonempty payload and the exact fixed
suffix shape: one separator followed by 96 ASCII signature bytes, including
the required `ed25519:` prefix. A malformed layout or non-ASCII suffix is
rejected before signature verification. After layout admission, verify the
signature over the exact payload bytes before parsing JSON.

Because the decoder accepts `&str`, invalid UTF-8 is unrepresentable at this
private interface. The wire JSON string is UTF-8 validated before it reaches
the decoder; no raw-byte decoder API or invalid-UTF-8 unit vector is required.
UTF-8 boundary and non-ASCII-suffix vectors remain applicable.

## Required boundary-vector classification

- A 4,096-byte input passes only the outer encoded-length check. It then fails
  payload-size/layout admission before signature verification because the
  fixed suffix leaves more than 2,048 bytes for its payload. This is not a
  valid exact-cap token vector.
- A 4,097-byte input fails the outer length check before any offset
  calculation or slicing.
- A 2,048-byte payload followed by the separator and fixed 96-byte ASCII
  signature representation has a 2,145-byte token. It passes payload
  admission and reaches signature verification; this does not assert that the
  signature verifies or that the payload parses.
- A 2,049-byte payload with that fixed suffix is rejected at payload
  admission, before signature verification.

These vectors clarify admission stages only. The existing payload and outer
caps, signature-before-JSON ordering, and authority boundary are unchanged.
This addendum approves no source implementation, tests, public API, dependency,
policy, startup, ledger, or runtime change. Keep the grant and this addendum
held for fresh independent exact-document review and explicit root release.

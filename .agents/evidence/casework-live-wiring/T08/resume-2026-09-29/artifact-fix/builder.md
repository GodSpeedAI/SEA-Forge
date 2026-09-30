# T08 artifact resolution builder note (2026-09-29)

## Implementation

- Added a protected `GET /api/artifacts/{digest}` route in the live server. The
  handler takes the authenticated session actor, runs the existing
  `PerspectiveVerifier` delegation check, and reads through the injected
  `ArtifactGetter` implemented by the live SFWP authority.
- The handler accepts a raw lower-case SHA-256 or `sha256:` plus 64 lower-case
  hex characters, normalizes the prefix for the kernel, and independently
  verifies the authority digest, returned size, and SHA-256 of the actual bytes
  before responding. Invalid UTF-8 is refused instead of replacement-decoded.
- The JSON response uses the existing `ArtifactPayload` field names and
  supported content types. It preserves the authority's evidence ID and run ID;
  unknown provenance fields remain empty. Extension mapping selects Markdown,
  JSON when the body is valid JSON, diff, or plain text.
- `HttpCaseworkAdapter.resolveArtifact` now uses the authenticated GET route,
  validates the response shape and digest against the requested digest, and
  surfaces the gateway's typed error envelope. The existing `OPEN_ARTIFACT`
  intent path remains unchanged.
- Production and live-test server assembly inject the existing authority via
  `NewWithArtifacts`. Shared T07 wiring is retained: `server.go` consumes
  `TrustedOrigins`, and the production command passes the configured allowlist.

## Added regression coverage

Added focused Go cases for a Unicode Markdown artifact with actual session
delegation, empty text, typed not-found, byte tampering despite matching digest
metadata, invalid UTF-8, type selection, rejected delegation before kernel
lookup, unauthenticated access, and invalid digest input. Added adapter cases
for Unicode payloads, empty content, typed not-found, and response digest
mismatch.

## Verification status

No Go formatting, tests, build, TypeScript tests, or UI build were run by this
builder. A read-only `gofmt -d` inspection of the Go files listed above and
`git diff --check` reported no formatting/whitespace differences. The T07
builder later reported its focused, vet, and global race gates passed after
these artifact tests were added; the exact command transcript is held by that
builder. That run preceded the subsequent trajectory route registration, so a
relevant Go gate must be rerun after the trajectory handler is integrated.
The real-kernel durable artifact proof remains pending a granted slot.

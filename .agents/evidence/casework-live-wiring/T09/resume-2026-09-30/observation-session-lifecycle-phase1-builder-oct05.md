# Observation session lifecycle Phase 1 builder record

Date: 2026-10-05. Source-only fixture/stub phase; implementation remains
unapproved pending independent review and intended assertion RED.

## Original bounded assignment

Read `observation-session-lifecycle-root-proposal-oct05.md`, its independent
review, and the existing auth session/store tests and identity type. Own only
`apps/godspeed-casework-go/internal/auth/session.go`, a new
`apps/godspeed-casework-go/internal/auth/session_observation_test.go`, and this
new immutable record. Add the detached internal `CurrentSessionState` shape
(`Claim ports.ActorClaim`, `ValidUntil time.Time`, `Revoked <-chan struct{}`)
and temporary `Current(id string)` stub returning zero state and false. Do not
implement revocation or alter Start, Resolve, Destroy, Sweep, or eviction.
Provide focused fixture coverage for non-sliding detached reads; strict idle
and absolute boundaries; Resolve sliding idle but fixed absolute; every removal
path and stable channel closure; repeated deletion, unrelated-session safety,
capacity bounds, and synchronized lookup/delete races. The stub must fail at a
live-Current behavioral assertion, not setup, compilation, or a blocked peer.
No test, compile, scanner, Git, hook, status, or debt commands.

## Source identities and exact changes

`session.go` before SHA-256:
`5d43071abc65760a78ccefa2d3e473d5b65815319ab02aab696fce862667a4ca`

`session.go` after SHA-256:
`f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`

`session_observation_test.go` was absent before this assignment. Its added
source SHA-256 is:
`78cbe26fffc70837eb27ce597872ad9a1dfe7fe70cce745441ae10a721c2c6aa`.

Exact `session.go` delta:

```diff
 import (
 	"crypto/rand"
 	"encoding/hex"
 	"sync"
 	"time"
+
+	"github.com/GodspeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
 )

+// CurrentSessionState is a detached view for observation work; it contains no bearer or CSRF
+// material and does not expose the mutable Session stored by SessionStore.
+type CurrentSessionState struct {
+	Claim      ports.ActorClaim
+	ValidUntil time.Time
+	Revoked    <-chan struct{}
+}

 func (st *SessionStore) Resolve(id string) (*Session, bool) {
     ... unchanged ...
 }

+// Current is a test-first stub. Production observation lookup is added in the next phase.
+func (st *SessionStore) Current(id string) (CurrentSessionState, bool) {
+	return CurrentSessionState{}, false
+}
```

The new test file adds these complete focused test groups: detached/non-sliding
Current state; strict idle and absolute equality/just-after expiry; Resolve idle
extension with fixed absolute deadline; closure for Destroy, Resolve expiry,
Current expiry, Sweep expiry, and capacity eviction; stable idempotent removal,
unrelated session preservation, configured capacity bounds; and synchronized
Current/Destroy race outcomes. Its synchronized clock protects concurrent reads
and changes. Every test that needs a live session first asserts Current returns
one, so the temporary stub's failure is the intended behavioral RED. Test
failure messages do not print bearer/session or CSRF values.

## Deviations and verification boundary

No material deviation. The existing lifecycle methods remain untouched; this
phase contains declarations, the intentionally unavailable stub, and tests
only. No test, formatting, compiler, scanner, Git, hook, status, or debt command
was run. Both source files are frozen for fresh independent review; the
assertion RED remains unrun until the root transfers the exclusive compiler
token.

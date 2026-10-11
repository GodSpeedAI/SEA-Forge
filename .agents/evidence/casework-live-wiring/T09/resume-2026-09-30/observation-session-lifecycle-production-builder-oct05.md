# Observation session lifecycle production builder record

Source-only implementation handoff. This record makes no compilation, test,
scanner, runtime, or independent-review claim.

## Original instructions and frozen inputs

Production scope and requirements are from
`observation-session-lifecycle-production-root-assignment-oct05.md`; the
approved design is `observation-session-lifecycle-root-proposal-oct05.md`,
approved by `observation-session-lifecycle-proposal-independent-review-oct05.md`.
The declaration/stub and fixture history is in
`observation-session-lifecycle-phase1-builder-oct05.md`,
`observation-session-lifecycle-phase1-fixture-builder-repair-oct05.md`, and
`observation-session-lifecycle-phase1-timeout-repair-builder-oct05.md`.
The final fixture readiness approval, actual focused RED analysis and root
acceptance are `observation-session-lifecycle-phase1-timeout-independent-review-oct05.md`,
`observation-session-lifecycle-phase1-runtime-red-review-oct05.md`, and
`observation-session-lifecycle-phase1-root-acceptance-oct05.md`.

Input identities (SHA-256):

| Input | SHA-256 |
|---|---|
| Production assignment | `0bbc1c4b6ef06a0d5c55cd811281959d5a77b09101368f7882c9491b4888b6bb` |
| Approved proposal | `57d29e3efe01acf4c045594442159235bc93a62ce92c0ca5d5d737e470d5f022` |
| Proposal independent approval | `e61604ccb243df7bd9588b57b2c3bda0ca08ff6b613f4b765e726b3943b77e7f` |
| Original Phase 1 assignment/builder | `c9cad7b48a9997bffe6999c3c172512052cccb4a5db45797965ab2809d6e0124` |
| Fixture repair record | `c69742298ade24039e9b6dfc3fb668cefd35d053c2daa7b27542aabc1e9df731` |
| Shared timeout repair record | `d0fed2cfa3bac9fac6488255d1b80c4636b84238705291e857d4b58d076eea6e` |
| Fixture source-readiness approval | `a02820348f0364075d34ba76b5b8629393bf95124e0c741ec05a3c86ab1be0b6` |
| Independent focused RED review | `dfca6495f7c52532c2ccf2b4b3544aa72cd2c83ee73e05668a8d1186f5ed0ea2` |
| Root RED acceptance | `fae38703680cd038002d04818560ea68a4b38def91887129f8c28954554b78a9` |

The original production source was SHA-256
`f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`.
The accepted fixture remains SHA-256
`8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f`.

## Resulting source identity

| Source | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/auth/session.go` | `d1c9daabfb758de94ea20ea0b7db7fbce1fad3a4ec43f6dee461d428b9972aa5` |
| Frozen `apps/godspeed-casework-go/internal/auth/session_observation_test.go` | `8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f` |

## Full exact source diff

Only `internal/auth/session.go` was modified. The private channel is stored on
each session created by `Start`; the single locked helper closes it before map
deletion. `Current` snapshots copied claim, earliest deadline, and signal
without sliding `LastSeen`. Existing strict expiry comparisons and
identity-to-claim mapping are preserved.

```diff
--- a/apps/godspeed-casework-go/internal/auth/session.go
+++ b/apps/godspeed-casework-go/internal/auth/session.go
@@ -13,6 +13,7 @@ import (
 	"crypto/rand"
 	"encoding/hex"
+	"strings"
 	"sync"
 	"time"
@@ -31,6 +32,7 @@ type Session struct {
 	CreatedAt time.Time
 	LastSeen  time.Time
 	ExpiresAt time.Time // the FIXED absolute deadline (never extended; idle expiry rides on LastSeen)
+	revoked   chan struct{}
 }
@@ -106,6 +108,7 @@ func (st *SessionStore) Start(identity Identity) *Session {
 		CreatedAt: now,
 		LastSeen:  now,
 		ExpiresAt: now.Add(st.absoluteTTL),
+		revoked:   make(chan struct{}),
 	}
@@ -133,7 +136,7 @@ func (st *SessionStore) Resolve(id string) (*Session, bool) {
 	now := st.now()
 	if now.After(sess.ExpiresAt) || sess.Idle(now, st.idleTTL) {
-		delete(st.sessions, id)
+		st.removeLocked(id)
 		return nil, false
 	}
@@ -142,9 +145,29 @@ func (st *SessionStore) Resolve(id string) (*Session, bool) {
-// Current is a test-first stub. Production observation lookup is added in the next phase.
+// Current returns a detached observation state without extending the session's idle lifetime.
 func (st *SessionStore) Current(id string) (CurrentSessionState, bool) {
-	return CurrentSessionState{}, false
+	if strings.TrimSpace(id) == "" {
+		return CurrentSessionState{}, false
+	}
+	st.mu.Lock()
+	defer st.mu.Unlock()
+	sess, ok := st.sessions[id]
+	if !ok {
+		return CurrentSessionState{}, false
+	}
+	now := st.now()
+	if now.After(sess.ExpiresAt) || sess.Idle(now, st.idleTTL) {
+		st.removeLocked(id)
+		return CurrentSessionState{}, false
+	}
+	validUntil := sess.LastSeen.Add(st.idleTTL)
+	if sess.ExpiresAt.Before(validUntil) {
+		validUntil = sess.ExpiresAt
+	}
+	return CurrentSessionState{
+		Claim: sess.Identity.Claim(), ValidUntil: validUntil, Revoked: sess.revoked,
+	}, true
 }
@@ -149,7 +172,7 @@ func (st *SessionStore) Destroy(id string) {
 	st.mu.Lock()
 	defer st.mu.Unlock()
-	delete(st.sessions, id)
+	st.removeLocked(id)
 }
@@ -168,8 +191,9 @@ func (st *SessionStore) Sweep() int {
 	for id, sess := range st.sessions {
 		if now.After(sess.ExpiresAt) || sess.Idle(now, st.idleTTL) {
-			delete(st.sessions, id)
-			swept++
+			if st.removeLocked(id) {
+				swept++
+			}
 		}
 	}
@@ -185,9 +209,22 @@ func (st *SessionStore) evictOldestLocked() {
 	}
 	if oldestID != "" {
-		delete(st.sessions, oldestID)
+		st.removeLocked(oldestID)
 	}
 }
 
+// removeLocked closes the session's stable revocation signal before removing it. The caller
+// must hold st.mu; deleting through this helper makes repeated removal harmless.
+func (st *SessionStore) removeLocked(id string) bool {
+	sess, ok := st.sessions[id]
+	if !ok {
+		return false
+	}
+	close(sess.revoked)
+	delete(st.sessions, id)
+	return true
+}
+
 // randomToken returns 256 bits of CSPRNG output as hex.
 func randomToken() (string, error) {
```

## Deviations and verification boundary

No material scope or behavior deviation. `Start`, `Resolve` return types,
identity/CSRF mapping, CSPRNG, expiry equality, TTL defaults, capacity policy,
and eviction ordering are preserved. All five removal paths use the same
helper: Resolve expiry, Current expiry, Destroy, Sweep, and capacity eviction.
No callback, channel receive, network read, cancellation, or join was added
under the store lock. The frozen fixture was not edited. No compile, test,
scanner, Git, status, or debt operation was run. Runtime remains unverified;
independent source review is the next step.

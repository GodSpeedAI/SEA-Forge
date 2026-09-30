# T07 cached intent authorization repair: original instructions and handoff

## Independent finding

The independent live critic rejected full T07 after an exact previously successful intent
POST returned its cached receipt following revocation, while the same session's world/SSE
reads returned `403 authority_denied`. The successful replay assertion passed; its exact
response body was not captured. Do not infer undocumented response bytes from that assertion.
The separately verified cursor publication repair remains approved.

Source: `internal/server/server.go` dispatches intents without current perspective verification;
`internal/intents/intents.go` checks its replay cache before role, payload, stale-cursor and
kernel execution checks. The existing session and CSRF checks alone do not revalidate the
kernel delegation. No documented ADR exception permits revoked sessions to read cached receipts.

## Bounded original builder instructions

Fresh builder `production_build_environment_builder`, Luna, tests first:

- Prime a successful receipt under an authorized session perspective, then revoke it.
- The exact idempotent POST with valid session/CSRF must return typed `403 authority_denied`
  before dispatcher/cache access and without another case or approval mutation.
- Restore authorization: the same receipt must replay without another logical mutation.
- Missing verifier, unmapped actor and unavailable kernel fail closed before dispatch.
- Body actor spoofing must be overwritten by the authenticated session identity.
- Missing CSRF, untrusted Origin and malformed JSON stop before kernel verification/dispatch.
- Preserve legacy exact-perspective fallback and existing intent memo semantics.
- A cache-like dispatcher spy is acceptable for focused HTTP tests; independently repeat
  revocation/restoration with the real handler/kernel before approving full T07.
- Tests only initially; production edits wait for independent red evidence. No dependencies,
  public DTO changes, identity model changes, status edits or commits by the builder.

After independent red, the intended minimal production change is to verify the session's
current kernel perspective in `handleIntent`, after existing request protections and before
`s.intents.Handle` can read the cache. Use the existing verification/error path.

## Builder result, recorded by root from delivered handoff

New `internal/server/intent_authorization_test.go` contains cached replay/restoration,
fail-closed verifier and request-guard tests. The builder reports only `gofmt` and
`git diff --check`; no test, build or runtime command. `server.go` is untouched at this phase.
The independent critic has the original instructions and sole compile token for the red
baseline. This handoff is neither a passing gate record nor approval of the implementation.

## Independent red and authorized production edit

Independent critic ran:

```sh
GOCACHE=/tmp/t07-final.uUPtaO/go-cache GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./internal/server -run 'TestIntentAuthorizationPrecedesCachedReplay|TestIntentAuthorizationFailsClosedBeforeDispatcher|TestIntentRequestGuardsPrecedePerspectiveVerification'
```

Approved loopback execution exited 1: revoked replay 200 instead of 403; absent verifier
200 instead of 503; unavailable kernel 200 instead of 502; unmapped actor 200 instead of 403.
Earlier attempts failed at a read-only default cache and sandbox IPv6 listener binding;
all three exact logs are copied alongside this note. Critic reported available RAM respectively
2.4, 3.1 and 2.9 GiB with no active compiler before attempts. Test file SHA256 at red:
`b398367594a4458cafcfd392627ca56f0d65d7a4de69e48299552fb53dfb877b`.
Root verified all three copied logs byte-identical to their originals. The actual approved
red log SHA256 is `b7aaa0e6185009a834710d468a79c1f927db363cb29eff01e4816aec3adf5aa3`;
the critic's delivered hash contained a transcription error. The file bytes are authoritative.

Root authorized phase two after red. Builder now calls existing `verifySessionPerspective`
with `sessionIdentityOf(r).Claim()` after strict decoding, session actor overwrite and
correlation setup, before dispatcher access. Only `server.go` production source changes;
builder reports gofmt/diff-check only. Independent green and real-kernel cached replay
revocation/restoration proof are pending. This is not full T07 approval.

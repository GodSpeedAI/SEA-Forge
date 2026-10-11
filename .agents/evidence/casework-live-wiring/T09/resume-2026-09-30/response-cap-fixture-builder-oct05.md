# Response-cap retry fixture repair — builder record — 2026-10-05

## Scope

Implemented the binding assignment in
`response-cap-fixture-repair-root-assignment-oct05.md`. Only
`apps/godspeed-casework-go/internal/adapters/sfwp/response_limit_test.go`
changed. Its SHA-256 before the edit was
`7ec880bc2a7cdf15eb8c4dfa26daca7d4ea021b7c50940a20e16839cec7b6299`; after
the edit it is
`dfb98f494880faf946ceaf3d0d10a139a20ed3260647112dd08dfc6372ccaa88`.

## Exact source change

In `TestInspectOverLimitResponseRetriesOnceOnFreshConnection` and
`TestOverLimitMutationAndRecoveryResponsesNeverResendMutation`, respectively:

```diff
 func TestInspectOverLimitResponseRetriesOnceOnFreshConnection(t *testing.T) {
+    const responseLineLimit = 1024
     var caseLists atomic.Int32
-    oversized := strings.Repeat("x", approvedResponseLineLimit+1)
+    oversized := strings.Repeat("x", responseLineLimit+1)
     ...
-    client, err := New(testConfig(fs.socket))
+    config := testConfig(fs.socket)
+    config.MaxResponseLineBytes = responseLineLimit
+    client, err := New(config)
 }
 
 func TestOverLimitMutationAndRecoveryResponsesNeverResendMutation(t *testing.T) {
+    const responseLineLimit = 1024
     var commits, statuses atomic.Int32
-    oversized := strings.Repeat("x", approvedResponseLineLimit+1)
+    oversized := strings.Repeat("x", responseLineLimit+1)
     ...
-    client, err := New(testConfig(fs.socket))
+    config := testConfig(fs.socket)
+    config.MaxResponseLineBytes = responseLineLimit
+    client, err := New(config)
 }
```

Each fixture now sends 1,025 `x` bytes and the fake server appends LF, making a
1,026-byte invalid-JSON line against the explicit 1,024-byte configured cap.
This keeps overflow ahead of JSON decoding. The valid recorded recovery JSON is
153 bytes, or 154 including LF, so it fits the cap.

## Preserved behavior and verification boundary

No request/connection counts, mutation send/status retry counts, recovered
outcome assertions, deadlines, recovery budgets, production code, configuration
defaults, or other tests changed. The dedicated fragmented fixture still
checks exactly 32 MiB acceptance and one-byte overflow poisoning through the
default client boundary. `gofmt -d` produced no output and `git diff --check`
passed. No Go compiler, tests, or other compiler command ran; the root retains
the sole compiler token for independent gates.

There were no material deviations from the assignment. This is a frozen,
source-only builder result awaiting independent review.

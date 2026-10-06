# Safe-trace fixture Close-call repair — builder record

Date: 2026-10-05

## Original bounded assignment

The root assignment was to repair only
`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` by replacing
the erroneous `_ = f.client.Close()` with the void call `f.client.Close()`.
Every test, assertion, and cleanup behavior was to remain unchanged. The
assignment required reading the original test-first instructions, accepted v3
proposal, latest fixture shape/source reviews, the compile-blocker review, and
the nearby `Client.Close` declaration/pattern. It prohibited compilation,
tests, scanners, Git operations, hooks, and status/debt edits.

## Source identities and exact change

Before SHA-256:
`e8859aa2e88632f03301cb4f4d97b9039e9105a96a2eaa2c97092f2d104a1155`

After SHA-256:
`fc9bbb748766dd2590b73c2a0ad1b0309e9dc62c2bd7d163e862d310a7b391cd`

```diff
@@
 		if f.client != nil {
-			_ = f.client.Close()
+			f.client.Close()
 		}
```

`Client.Close` is declared `func (c *Client) Close()` and returns no value.
Nearby adapter tests likewise invoke `client.Close()` without assignment.

## Scope and deviations

Only the assigned fixture line changed in source. This new immutable builder
record is the sole additional file. No test, assertion, cleanup ordering, or
other source behavior changed. No deviation from the assignment. No formatting,
compile, test, scanner, Git, hook, status, or debt command was run. The fixture
is frozen for fresh independent source review and the separately authorized
assertion RED attempt.

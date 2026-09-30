# Focused test coverage added (not run)

Source-only pass; no test, build, or compile command was run because the shared compile token is held by another builder.

- `internal/projection/store_test.go`: retained `CaseFacts` are deep-copied on append and read, including nested slices.
- `internal/server/session_read_test.go`: relay uses one facts capture to produce snapshot plus retained facts; session-specific historical and SSE rendering preserves the cursor and role-specific actions; actor/role query overrides are refused before cached lookup; cross-case cursor requests and revoked perspectives fail closed; legacy snapshots cannot be guessed into another actor's perspective.
- Existing `TestWorldPerspectiveComesFromTheSession` remains the current-read perspective check.

Fresh-cell runtime probes and the focused/global Go gates remain pending the compile-token owner.

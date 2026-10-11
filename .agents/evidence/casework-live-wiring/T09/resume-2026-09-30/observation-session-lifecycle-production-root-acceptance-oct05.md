# Root acceptance: bounded SessionStore observation prerequisite

Accept the independent source and focused/full auth race reviews for session.go
d1c9daab and unchanged fixture8de408be (full hashes in the reviews). Root rehashed
both files, inspected the complete source diff and all five centralized removal
paths, and checked the detached/no-slide/deadline/locked notification design.

Both actual race commands joined exit0. Root byte-compared all six archive
`auth-session-{focused,full}-{preflight,stdout,exit}-oct05.txt` files with
`/tmp/auth-prod-{focused,full}-{preflight,stdout,exit}-oct05.txt`; every copy is
exact. Full auth uses authorized loopback access without weakening OIDC tests.
The namespace-limited preflights are not host-wide process absence proof;
explicit token ownership and actual exits establish managed serialization.

This accepts only the auth-store prerequisite. Channel closure is notification;
server timer rechecking, session/perspective admission, cancellation/join of
pending reads, SSE writes and shared poller lifecycle still need implementation
and independent proof. Full module/T09 completion and publication are unclaimed.
Compiler token transferred to trace critic only after both auth sessions joined.

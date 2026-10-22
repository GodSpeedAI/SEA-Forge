# Current status

**Status revision:** 148

**Stage:** one-uid-one-actor-committed-local

**Summary:** One operating-system uid now maps to exactly one actor (SF-005 follow-up). IdentityBinding.uid is Option<u32> (omitted = no direct OS principal, reachable only via gateway on_behalf_of or an authenticated workload claim). IdentityBindings::validate runs from ServerConfig::validate before any cell record or socket: two actors on one uid and one actor bound twice are refused with actionable messages. resolve_direct and gateway lookup compare Some(uid); local_operator binds Some(uid). Every multi-actor-per-uid fixture, config, script and test migrated to gateway (uid-less human actors) or to one direct principal per uid; premises that depended on a second same-uid direct claim were rewritten and say so in the commit. Local commit only; not pushed.

**Verified:**

- cargo test -j 2 -p sea-forge-server --no-fail-fast (RUSTC_WRAPPER empty): 519 passed, 0 failed, 2 ignored across 46 suites
- cargo test -j 2 -p sea-forge-authority --no-fail-fast: 190 passed, 0 failed, 7 ignored across 14 suites (includes yaml_corpus with the t08c server.yaml copy)
- cargo test -j 2 -p sea-forge-cli --no-fail-fast: 87 passed, 0 failed across 19 suites (sea-forge-cli binary built first)
- cargo test -j 2 -p sea-forge-runtime --test isolated_execute_command: 2 passed, 0 failed
- cargo clippy -j 2 -p sea-forge-server --all-targets -- -D warnings: exit 0
- Go in apps/godspeed-casework-go: go build ./... exit 0, go vet ./... exit 0, go test ./... exit 0 (12 ok packages)
- scripts/seed-cell.sh bash -n syntax check ok; docs/execution/journey/drive_identity.py py_compile ok; just --dry-run casework-cell-init parses

**Limits:**

- Live SFWP ladder, Postgres broker and live e2e (apps/godspeed-cognitive-ui/e2e/live, live-conformance.ts) were not run
- TypeScript typecheck for apps/godspeed-cognitive-ui could not run: no node_modules in this worktree (tsc not found); touched TS changes are YAML template strings only
- drive_identity.py was not executed against a live cell; its gateway-based approval path is unverified
- Schema/contract: IdentityView exposes only the peer uid (already nullable); no contract exposes the binding uid, so no generated contract changed
- Human-versus-service classification of uid-less bindings is a judgement call for casework and t08c configs

**Next move:** Review the commit, then push and open a PR; run the live casework ladder and drive_identity.py against a real cell before merge

**Evidence:**

- crates/sea-forge-server/src/identity.rs
- crates/sea-forge-server/src/config.rs
- crates/sea-forge-server/tests/conformance_identity.rs
- crates/sea-forge-server/tests/security_acceptance_v1.rs
- crates/sea-forge-server/tests/sfwp_delegated_identity.rs
- justfile
- scripts/seed-cell.sh
- .agents/reports/casework-live-wiring/config-reference.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md

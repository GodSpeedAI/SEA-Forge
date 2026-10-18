# Round 3 — live authorization gaps, state DAC, syscall surface

Builder evidence, not independent verification. Nothing here has been
re-attacked. Branch `security/enforcement-gaps-20261017`, base `544593b`,
uncommitted at the time of these runs.

Environment: WSL2 kernel 6.18.54 (Landlock ABI 7, x32 ABI enabled,
`io_uring_disabled=0`), PostgreSQL 18.1 (pg0), x86_64. Runs used
`SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1` and `SEA_FORGE_REQUIRE_POSTGRES=1`.

## Gates of record: `gates/` (run 7, final tree incl. the RealityTrace pin)

| Gate | Result |
| --- | --- |
| fmt-check, lint, typecheck, security (deny + gitleaks) | exit 0 |
| no-async-kernel, proof | exit 0 |
| `cargo test -p <crate>` for all 24 crates, `-j 1` | 1516 passed, 0 failed, 24 ignored (the extra ignore is the open pathname-socket test) |
| v1 baseline (`tests/security-v1/run.py all`, MCP provisioned) | 6 PASS / 10 NOT-YET-EXECUTABLE |
| frozen v1 diff against `sea-forge-security-v1-2026-10-16` | empty |
| `just check` | all gates green, again after the commit-independent gitleaks change (`gates/check-after-option1.log`) |

## Invalid runs, kept for the record

- `gates-run1-shared-target/`: `target/debug` was a symlink into the main
  checkout's target, so artifacts were shared across worktrees. The CLI
  lifecycle test failed on `current_exe()` resolving there. Its t16_4d
  failure was a real defect (below). Its v1 count (4 PASS / 2 blocked) came
  from MCP ingress not being provisioned, not from a regression.
- `gates-run5-before-final/`, `gates-run6-interim/`: superseded by later edits; run 6 failed three realitytrace tests on a schema construct jsonschema 0.18 does not apply as written (fixed).
- `gates-run2-before-filter/`: green apart from status-check; superseded by
  the syscall filter. `test-sea-forge-server-after-fix.log` there failed
  t16_4d because of a stale mutant (next item).
- `gates-run4-stale-mutant/`: the mutation harness restored sources from a
  backup whose mtime was older than the mutant build, so cargo reused mutant
  artifacts (M15 in the sandbox helper; M10 earlier, which caused the
  "intermittent" t16_4d failure). The harness now updates mtimes on restore.

## Defects found and fixed this round

- `harden_dir`: `fchmodat(AT_SYMLINK_NOFOLLOW)` is unsupported on Linux, so
  files stayed 0644. Files are now changed through an `O_NOFOLLOW` descriptor
  after an inode/owner re-check.
- Per-role PostgreSQL timeouts were removed: a workload can `SET` them away
  on its own connection (`lifecycle-with-indb-timeouts.txt`).
- `scope_admits` treated a scope's own exposure label as foreign, so the
  grant-time transition revoked a just-issued read session (t16_4d). The
  scope's own label is now admitted; every other label still needs an
  explicit permission (`postgres::tests::scope_admits_only_…`).

## Syscall surface (`syscall-before-filter.txt`, `syscall-after-filter.txt`)

Before the filter, inside full isolation: `io_uring_setup` and unprivileged
user, net and mount namespaces all succeeded, also at x32 numbers. Landlock
alone already refused remounting `/` from a nested user+mount namespace, so
the namespace refusal removes kernel attack surface; it does not close a
demonstrated Landlock bypass. After the filter: io_uring, `setns`, and
`unshare`/`clone` with any namespace flag return `EPERM` (normal and x32
numbers); `clone3` returns `ENOSYS`; fork, threads, subprocesses and file
writes still work. Calls from any other audit architecture kill the process.

## Staged-file secret scanning (`staged-canaries*/`, gitleaks 8.30.1)

Before: the pre-commit hook ran `check-fast` only, so no secret scan saw a
commit before it existed; `just security` scans committed history. The
canaries (`scripts/security/staged-secret-canaries.sh`, synthetic tokens
generated per run, never stored) show `gitleaks git --staged` scans the index
itself: staged new and modified files are caught, a staged secret reverted in
the worktree is still caught, unstaged-only changes are not (they never enter
the commit), and every report is redacted.

`staged-canaries-before/` (the mutation control, original config) also
showed four allowlists admitting NEW secrets:

| Allowlist | Defect | History coverage measured | Change |
| --- | --- | --- | --- |
| `.entire/metadata/`, `^[0-9]+/…jsonl` | path-only, unanchored | 0 findings | removed |
| "commit-scoped" fixtures | no `condition = "AND"`: commit OR path | 12 (4 in unlisted `bd661c9`) | AND + rules + `bd661c9`; current-tree identifiers by exact value |
| casework T10/T09 | path-only, every rule | 75, all `generic-api-key` | `targetRules = ["generic-api-key"]` |

After: history scan clean, all 13 canary expectations met, and
`pre-commit` now runs `secrets-staged`; `security` runs the canaries.
Residual: a new `generic-api-key` match under the casework evidence paths is
still admitted (credential-specific rules are not).

Open for integration: a tree-mode scan (what a depth-1 CI checkout sees)
still reports 22 `generic-api-key` matches in committed round-1/2 evidence
(`security-v1/continuation-20261017/all/{results.json,rust-A.log}`,
`security-v2/realitytrace-sec12-20261017/bundle/sec12-trace.json`). History
mode passes them only through commit-pinned exceptions, which do not match a
rescanned merge commit. Not changed here: widening evidence allowlists is
outside this round's authorization.

## Output channels from an exposed context

| Channel | Status | Evidence |
| --- | --- | --- |
| Brokered PostgreSQL sessions | revoked before classified plaintext moves; verified termination | `postgres_lifecycle.rs`, M1–M3, M8–M11 |
| MCP tool calls | destination rules see the context's labels; gate withholds on failure | `exposure_flow.rs`, M7 |
| Network (TCP, abstract sockets) | contained: isolated agents always run `NetworkPosture::Denied` in their own net namespace | round 2 `workload_boundary` |
| Pathname UNIX sockets | **open**: reachable outside declared trees (`unix-socket-reach-before.txt`) | `unix_socket_reach.rs` (ignored until contained) |
| Workspace / artifacts → federation bundle | export states the union of its runs' labels; currently every export is denied as unclassified because `export_bundle` is not a reserved surface | `export_exposure.rs` |
| Transcripts, run records | stay under the owner-only control-plane root | `state_tree.rs` |
| Changeset promotion | separate OpenShell pipeline; no exposure source feeds it; human-approved CAS | by code reading |
| RealityTrace envelopes (E5A/E6) | identities, constraints and settlement outcomes only; no in-tree caller | by code reading |

## RealityTrace `evidence.recorded`

Accepted by the owner and registered on RealityTrace `main` 42879a8 (pushed);
SEA-Forge pins `sxr-core` there and ingests proofs under
`ProofProducer::SecurityAcceptance`. `sxr-gates/` holds RealityTrace's own
fmt, clippy and 209-test run; `sea-forge-against-local-sxr.log` the
pre-push check of this crate against it. Mutation controls:
`mutation-evidence-recorded-schema.txt`, `mutation-proof-producer.txt`.
See `docs/security/realitytrace-evidence-recorded-proposal.md`.

## Retention outside Git

`scripts/security/retain-evidence.py` copies evidence (including every
failed and invalid run) into an owner-only archive and writes a hash
manifest with git provenance and v1 run identifiers. The v1 runner outputs
(`.agents/evidence/security-v1/round2-20261017`, `round3-20261018-run*`)
carry synthetic idempotency identifiers that match `generic-api-key`, so
they are retained in the archive and indexed by
`.agents/evidence/security-v2/retention/round3-20261018.json` instead of
being committed. `verify` detected an altered file, an extra file and a
widened mode in its negative controls.

## Mutation controls

`mutations-round3-run2.txt` (M1–M11) and `mutations-syscall-filter.txt`
(M12–M15): all KILLED, each by its targeted test, with restored sources
verified by SHA-256. `mutations-round3.txt` is the first M1–M9 run, built
into the shared target directory; it is superseded.

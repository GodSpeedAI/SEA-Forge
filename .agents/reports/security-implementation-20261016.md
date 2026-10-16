# Security implementation checkpoint — incomplete

The mandatory security implementation is **not complete**. The unchanged v1 runner reports **6 PASS / 10 NOT-YET-EXECUTABLE**, exit 1. Four legacy regression assertions fail under the stronger identity restriction. Stage A remains open; Stages B and C and the complete independently frozen supplementary contract are outstanding. This report makes no adversarial-validation claim.

## Frozen baseline and integrity

- External protected tag: `sea-forge-security-v1-2026-10-16`.
- Independently retained full commit: `af41ea2465e38c87dfe86471c33c6b54ea8564e1`.
- Production baseline recorded by v1: `5664912f085f5abbc82895570de8dca19b598a6f`.
- Frozen manifest SHA-256: `986454070b7e7f3acfbfe89f4b747fa4e22477cfe300653cd337c50bd70a1e66`.
- Frozen Rust test SHA-256: `8022c7850719980cf626e380797d3974503ed195c727ec7482f4a86fde3404e8`.
- CONTRACT SHA-256: `ebb7b5a60f769961d47d6da3170a8538b46e8535dd529eb4fb0d0011648b65a4`.
- Inventory SHA-256: `781aa12f17d3fc354944b645c859f0a2a4a706d74dea0f919ba18944a4b913ae`.

All 44 protected files, modes and Git objects matched before implementation and again at this checkpoint. The verifier was loaded from the pinned Git object rather than the mutable checkout:

```sh
set -o pipefail
git --no-replace-objects show af41ea2465e38c87dfe86471c33c6b54ea8564e1:tests/security-v1/verify.py |
  python3 - --commit af41ea2465e38c87dfe86471c33c6b54ea8564e1
```

The last captured verification exited 0 in [frozen-integrity-final](../evidence/security-v2/verification-20261016/frozen-integrity-final/result.json). The immutable manifest retains the individual hashes of every original fixture and expected result. No v1 file, gap row or expected outcome changed.

## Contract status

These are full-contract statuses, distinct from narrower executable rows.

| Contract | Status | Evidence and remaining work |
|---|---|---|
| SEC-01 identity | BLOCKED | Same-connection substitution is refused. Real private-capability API authenticates exact principal/actor/request and passes the migrated approval control. Production launcher delivery and isolated workload identity remain missing. |
| SEC-02 credentials | BLOCKED | Workload-bound issuance, renewal, credential secrecy and durable revocation are unimplemented. |
| SEC-03 MCP | PASS | Frozen stdio scope: denied call and fresh-session retry reach no prohibited upstream handler; allowed read completes. |
| SEC-04 canonical MCP | PASS | Frozen malformed-frame corpus and valid mixed batches preserve one parsed request through authorization and forwarding. |
| SEC-05 PostgreSQL | BLOCKED | Native least privilege passes against real PostgreSQL. Product workload-to-database credential/session mediation is unimplemented. |
| SEC-06 alternate paths | BLOCKED | Child environment control passes. Credential/filesystem/network/alternate database bypass boundaries lack complete executable coverage. |
| SEC-07 elevation | BLOCKED | Independently authenticated approval is demonstrated at the private API; resource privilege scope, expiry and revocation lifecycle remain unimplemented. |
| SEC-08 active revocation | BLOCKED | No product barrier revokes already-active PostgreSQL/MCP sessions. |
| SEC-09 exposure | BLOCKED | Trusted sensitive-read exposure propagation and destination controls remain unimplemented. |
| SEC-10 injection | BLOCKED | Scripted v1 attempts are frozen; required integrated private-context and alternate-output execution is missing. |
| SEC-11 delegation | BLOCKED | Parent/child exposure inheritance, bounded resource effects and active parent revocation remain unimplemented. |
| SEC-12 evidence/settlement | BLOCKED | Real denied file write now settles rejected; full end-to-end authenticated attribution and resource observation across all execution routes remain missing. |

The ten NOT-YET-EXECUTABLE rows remain in the original inventory. PostgreSQL availability is now established locally; its missing product mediation is not an infrastructure excuse. The workload-launch boundary requires an operator deployment decision or an authorized, verified Linux isolation implementation. Other missing product mechanisms remain product work.

## Actual resource observations

Latest unchanged v1 run: `51bbdae0-8915-4763-b089-fe34ae59f920`, [results.json](../evidence/security-v1/implementation-checkpoint-20261016/all/results.json), SHA-256 `c463f174a829246bed367976ab012154d0f2c6c705319f1abc4c3bd094bf6cd0`.
Its rust-A log hash is `122a23bf92bbd53ab922d48563a90781a15bc6236edec1ed8e9a22e7301802c1`; rust-B is `0fa668f45f7fa410a8374615a70ef45be8606dce5347437ce751c5eb5be54289`.

- **MCP:** the actual unrestricted upstream handler records exactly one allowed `metadata.read` and zero `audit.delete` invocations in the SEC-03 probe. The prohibited call and new-session retry were both sent and returned -32001. Discovery advertises only the read. Protected audit digest remains `9642fb1a5cfafce59185718d6b969707a28ae854e637f1d4fba1aeda5abfcd25`.
- **Canonical MCP:** 18 malformed/ambiguous byte frames return -32600 without upstream effects. Three valid batches actually attempt `audit.delete`; all deny it. Two mixed batches also complete their allowed reads, preserve reply IDs, and have independently checked handler deltas and unchanged protected audit data. All effects/reply checks pass.
- **PostgreSQL 18.1:** SELECT returns two rows; the catalog control succeeds. The reader's five privileged role flags are false. Eight attempted destructive/escalating statements return native SQLSTATE 42501 and psql exit 3: DELETE, TRUNCATE, DROP CASCADE, SET ROLE, GRANT, disabling read-only then DELETE, alias DELETE, and search-path alias DELETE. The protected table still exists with two rows; before/after digest is `3d74f767a718a7e53cfcc44cb95c5d5a889d8ac8c9874c6e0848ade47caca20d`. The fixture's owner-write positive control succeeds. No agent receives owner credentials.
- **Filesystem and settlement:** the destructive write is actually attempted and receives EACCES, OS error 13. Protected digest stays `06ebb27e5df9512149363c4be7accb7b424d746db456dbe9987601129712e146`. A supplied “completed” observation falsely claims the positive-control digest; settlement rejects with `resource_observation_rejected:file_digest_mismatch`. The genuine allowed write succeeds, yields `29229c6162f8edabc1db0aeaca7fc7b924b2b83278818a7c62620b365054df26`, and settles accepted. Ledger integrity checks pass.
- **Private authentication/approval:** [authentication-evidence-06](../evidence/security-v2/verification-20261016/authentication-evidence-06/command.log) records actual approved journal attribution to operator_b, actor-switch/public impersonation/self-approval refusals, changed-request/duplicate-key/replay refusals, and verified issuer plus authentication ledgers. Three successfully authenticated attempts carry the actual committed issuer decision reference, actor, principal and request hash. Authentication alone never claims execution or settlement.
- **Isolation limitation:** [the regular-file descriptor probe](../evidence/security-v2/isolation-20261016/README.md) observed a same-host-UID observer read a synthetic sentinel from a rootless bwrap workload's open file descriptor. Namespace UID 0 mapped to host UID 1000. It did not test socket duplication, socket theft, ptrace or memory access. `unshare --map-auto` fails because newuidmap is absent. Namespace-only isolation therefore does not establish the required credential boundary.

Earlier prechange and intermediate failed runs are retained, including the genuine universal MCP denial caused by an omitted Reserved-action allowlist entry. That was repaired in production and positive controls then succeeded; the earlier RED evidence was not rewritten.

## Architectural additions and migration

[ADR-011](../../docs/decisions/ADR-011-security-enforcement-boundaries.md) and [the implementation spec](../specs/security-implementation-v2.spec.md) describe the interfaces.

1. Transport-principal continuity rejects same-connection substitution. Host-local direct mode restricts one UID to its primary configured actor; configured gateway delegation retains its distinct existing rules.
2. The trusted-launcher API consumes existing authority grants binding a private channel to principal, actor and exact request. One request per capability, expiry checks before/after admission queue, and durable issuer-decision provenance are enforced by production code. Public protected requests fail in authenticated mode.
3. The server edge crate adds a session-scoped stdio MCP broker. Existing authority handles declarative `mcp_tool_call` rules with explicit server/tool boundaries and canonical request hashes. No second authority engine is added. Existing DomainForge and OPA Reserved-action representations carry these facts.
4. One recursive duplicate-aware JSON parser is reused for client/upstream MCP and private request hashing. Entire batches validate before dispatch; exact grants are consumed at the forwarding boundary.
5. File adjudication opens absolute workspace components from the trusted host root without following symlinks, pins directory descriptors, and hashes bounded regular-file contents. It replaces any submitted witness and rejects missing/mismatched observations, including nested claims. Settlement preserves the rejection basis.
6. JSONL ledgers remain authoritative. MCP attempt/decision/forward/response/settlement records and private authentication evidence use existing ledger primitives. Capability memory and SQLite FTS5 ownership are unchanged.

The operator authorized migration of only the strong legacy `conformance_identity.rs` approval control. Its old expectation let actor A change to B on one socket. The replacement preserves all positive approval/journal assertions and uses independently authority-issued private capabilities, with negative controls. It exercises a real API, but does not prove launcher isolation.

The full regression found four additional assertions in `conformance_cep_authority.rs` and `sfwp_case_mutations.rs` that select secondary same-UID actor claims. They remain unchanged and failing. An explicit extension of migration authorization is pending. No compatibility relaxation has been introduced to make them pass.

## Dependencies, unsupported boundaries and remaining bypass surfaces

No Cargo, lockfile, production JavaScript, runtime service or daemon dependency was added. Existing Tokio/serde/sha2/libc/ledger components are reused. Teleport source and dependencies were not used. aimock was not needed: frozen scripted protocol inputs reproduce the attacks deterministically.

Optional test-only pg0 is v0.15.2, executable SHA-256 `06c4678ff7b461dba8ed1f95c5e202ead3b57b70f549a22c17a94985121c0bf5`; official release metadata hash is `e9b7727e6f1d01fafcb8a8afed5fd3a5fb639c8729e330f37ed688e2133d970e`. [Provisioner evidence](../evidence/security-v2/pg0-20261016/README.md) documents library wrappers, cleanup and native observations. Temporary credentials are not retained.

The normal server entrypoint still uses host-local mode; no production launcher calls the private API. Holder authentication is not process attestation, and private FD secrecy is not demonstrated. Trusted MCP configuration/upstream ownership is assumed. MCP progress/streaming/other transports are unsupported. MCP response evidence alone is not proof of upstream native effects. Local file witnesses prove current bytes, not write causality; hardlinks, directory substitution before acquisition and concurrent writers remain outside this witness proof. Linux procfs is required; other platforms fail closed for file observations. Existing ordinary execution routes are not all wired to this observer. OpenShell/Landlock integrations were retained, but no live OpenShell isolated environment was available.

## Commands and results

Captured commands preserve exit codes and complete logs under [verification-20261016](../evidence/security-v2/verification-20261016).

| Command | Result |
|---|---|
| Unchanged pinned v1 verifier, before and after | PASS; 44 protected files match |
| Latest unchanged v1 `run.py all`, actual broker + PostgreSQL | Exit 1; 6 PASS / 10 NOT-YET-EXECUTABLE |
| Authenticated identity and delegated identity regressions | 24 passed |
| File/invocation/settlement/delegation regressions after root fix | 34 passed |
| Scoped root/file sandbox regressions | 2 passed |
| Authority MCP scope/exact-grant regression | 1 passed |
| `just test` full workspace/all-features | Exit 101; 1,462 passed / 4 failed / 23 existing ignored |
| `just check` after root fix | Exit 0 |
| `just proof` P1–P4b | Exit 0 |
| `just no-async-kernel` | Exit 0 |
| Directory secret scan of retained v1 evidence | Exit 0 after reviewed exact nonsecret exclusions |
| Post-commit \`just security\` | Exit 0; dependency checks pass, 469 commits scanned with no leaks |
| Candidate SHA-256/mode consistency | Exit 0; all 10 entries match; no independent freeze |
| Final status check and index refresh | Both exit 0 |

The full-test-01 log hash is `a6d2fd38f37c6be35444866d773fdefb8e29476d401621bd8ad5393266cf1aad`. It precedes the root acquisition correction; the affected file/invocation suites were rerun after that correction. No full-regression success is claimed. `just ci`, other-platform execution, Workbench and live OpenShell tests were not run for this backend checkpoint.

Initial directory scans found 72 repeated generic-api-key matches on public fixture idempotency digests. [The analysis](../evidence/security-v2/idempotency-scan-analysis.json) records every path/value/rule tuple. The scanner configuration adds one conjunctive exact-file/exact-value/single-rule block per evidence file. Raw evidence is preserved; neither directory-wide nor generic-hex exemptions were added. Original security assertions were not suppressed.

Reproduce the full resource run using the build and environment commands in [tests/security-v2/README.md](../../tests/security-v2/README.md). Use fresh output directories; the provisioner is optional and local paths must match the installed binaries.

## Supplementary hashes and freeze status

The supplementary coverage is **candidate, incomplete and not independently frozen**. There is no external supplementary Git pin. It does not discharge the ten v1 gaps. [candidate-manifest.json](../../tests/security-v2/candidate-manifest.json) SHA-256: `60b95a5942b46511cfecd9831438af33cb7bc7f33756de7cf9047ad7a134c971`.

| Candidate test/source/fixture | SHA-256 |
|---|---|
| `crates/sea-forge-authority/tests/mcp_boundary.rs` | `b9c93885e2f1f431e57cfa78344d5f4ee96abd30cf704c2ab7851cb9bbfdab4c` |
| `crates/sea-forge-sandbox/tests/observation_file.rs` | `cbaf71742a3928bf10a3ee7edba2bcb7a48cdb280e9d9f0ede46c8a1ed19a6f6` |
| `crates/sea-forge-server/tests/file_effect_observation.rs` | `5d40ab2ab37dbec6014ea47a2eafcfc58bd6e293d908027a00cb690b3c3aaea8` |
| `crates/sea-forge-server/tests/conformance_identity.rs` | `0b68cc5058f8d612b7a3183414e83204cf7e65aaf1bc3793b8bc4aa55a104113` |
| `crates/sea-forge-server/src/mcp_protocol.rs` | `1af23c70cd5592094be818adb3e50824be549ca094860aeb593551efcec8df9a` |
| `crates/sea-forge-server/src/workload_connection.rs` | `ef5c200c1db7fbcade33722b9870ba49558e466695bfca867c7f133b4b1b783a` |
| `tests/security-v2/mcp-policy.yaml` | `9a22879156589f5c0ffc8a9493bfa74456e4b742cc08f1c55708c6d57a91f9da` |
| `tests/security-v2/provision_mcp.py` | `d009ccc4cf4c0b75dce2ee13c4f8e02da32357009be650c9ae774302cbb6b9e0` |
| `tests/security-v2/capture.py` | `7a211f1278fc40a5bca10ea9f7897a2b31e0ad54b91bd99a9ad2f39878d382a4` |
| `tests/security-v2/README.md` | `5ac8c0df146ddbaff0d75ad4b7cc75a83aff849185b04eb82f3681632c5dd846` |

## Commit and next action

Implementation and initial evidence commit:
`e96c45ca2e39f05ea3f15e83a7554d2745ebd9a5`
(`fix(security): mediate MCP and reject unverified effects`).
The first ordinary path-limited commit attempt exited 1 before checks: Devbox
could not contact its Nix cache before its deadline. The successful retry used
the hook's existing local-tool fallback with the already installed pinned Nix
profile: Just 1.55.1, Rustup 1.28.2, Rust/Cargo 1.99.0. It ran the unchanged
status, formatting and locked workspace checks; no hook or gate was disabled.
The [commit receipt](../evidence/security-v2/verification-20261016/commit-02/result.json)
and full hook log retain the result. The four operator-staged
paths are preserved separately. Required next decisions are the production
isolation target and authorization for the additional legacy controls. After
those decisions: wire and prove the launcher; implement native PostgreSQL
mediation and lifecycle; complete exposure/destination/delegation enforcement;
add, review and independently freeze complete supplementary contracts; rerun
the resulting full suite. No unresolved mandatory blocker may be relabeled as
a successful settlement.

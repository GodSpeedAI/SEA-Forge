# Security continuation — clean regressions, Stage A blocked

The four authorized legacy controls are migrated and independently reviewed.
The full Rust suite is clean: **1,467 passed / 0 failed / 23 existing ignored**.
The unchanged frozen v1 resource run still exits 1 with **6 PASS / 10
NOT-YET-EXECUTABLE**. No mandatory gap has been relabeled as success.

Stage A remains incomplete. The actual WSL host lacks the trusted UID/GID
mapping helpers needed by the selected rootless launch approach. No alternate
configured runtime was found at the checked standard paths. Production
isolated workload launch, MCP execution integration and workload-bound
PostgreSQL sessions remain unimplemented; installing helpers alone would not
discharge those product obligations. Stages B/C remain gated on Stage A.

## Increment: authenticated legacy workflows

The original multi-actor workflows were legitimate; selecting an actor label
on a shared-UID socket was insufficient identity evidence. The replacement
trusted test driver uses production authority evaluation, committed decisions,
exact grants, private connections and the real authenticated server. Clients
hold only one endpoint; the driver does not intercept attacks or derive their
identity from incoming claims. No production identity check was relaxed.

| Original control | Preserved behavior | Decisive resource observations |
|---|---|---|
| CEP escalation approval | Independent operator approves; identical request replays; approved action revalidates once; later reuse refuses | Actual authority journal unchanged on impersonation and replay; approved response attributed to op-alice; ledger verifies |
| CEP resolution validation | Missing actor retains identity_required; independently authenticated malformed resolution retains cep_request_invalid | Public unauthenticated attempts refuse before the authority journal is created; real missing-file error is checked |
| CEP world transition | Source-only edit allows; semantic change escalates; requester cannot approve; operator approves; revalidation allows; exact two transitions list; unknown world refuses | Authority journal unchanged on impersonation; committed transitions and approval state are read through real projections; ledger verifies |
| SFWP discretionary item | A proposes; B executes and produces escalated settlement; A cannot resolve its own proposal | Case ledger unchanged on impersonation; approvals.jsonl byte-identical after sod_violation; case ledger verifies |

Each relevant multi-actor control sends the operator/executor claim on the
previous requester connection and a fresh public socket. Both return
identity_authentication_refused. These are actual transmitted requests, not
fixture-level refusal logic. The original positive assertions remain present.

Native resources checked by these tests are
`<cell>/ledgers/cognate-authority/entries.jsonl`,
`<cell>/ledgers/case-<id>/entries.jsonl`, and
`<cell>/approvals.jsonl`. Authentication records correlate each successful
authentication to its actual issuer decision, principal, actor and request hash.
A later business refusal is not presented as successful execution.

The [fresh reviewer receipt](../evidence/security-v2/continuation-20261016/migration-review.json)
records inspection of the original tests at
`dd494c1ae865487fa928647703be275087a943b0` before the migration diff. Its verdict
accepts the narrow regression scope, not isolated OS identities or SEC-01–12
qualification. [Migration purposes and reproduction](../../tests/security-v2/migrations/README.md)
are documented separately.

## Actual host prerequisites and corrected scope

Two probe contexts are retained. The initial probe ran inside the tool sandbox.
Its PID1=codex, NoNewPrivs=1, one-entry parent UID map and read-only cgroup facts
must not be represented as host constraints. A [scope caveat](../evidence/security-v2/continuation-20261016/host-preflight/scope-caveat.json)
preserves that correction without rewriting the original raw observations.

The [outside-sandbox host probe](../evidence/security-v2/continuation-20261016/host-preflight-outside/README.md)
observed WSL2 kernel `6.18.54.1-microsoft-standard-WSL2`, PID1 systemd,
UID/GID 1000, a full current UID map, NoNewPrivs=0, Seccomp=2, no effective
capabilities, and Landlock ABI 7. Transient user/PID/network namespace bwrap
probes succeed. A single-ID map still maps namespace root to host UID1000;
it does not separate agents.

Subordinate UID/GID ranges exist. `newuidmap` and `newgidmap` are absent;
`unshare --map-auto` actually fails with exit 127 on the missing helper.
No distinct host UID workload was launched or qualified.

The [outside-sandbox runtime audit](../evidence/security-v2/continuation-20261016/host-preflight-outside/runtime-audit.json)
found no Docker/Podman/containerd sockets at the checked standard paths or
matching runtime daemon processes. OpenShell reports not_configured.
Arbitrary custom runtime endpoints are outside that search.

Required operator action: provision the distribution's `uidmap` package or
equivalent audited mapping helpers on the actual host, or supply a controlled
runner with working distinct host UID/GID mappings. No daemon or global kernel
security-setting change is proposed. No installation, sudo/root-changing
command, container/service startup or host security-setting mutation was
performed. Writable cgroups are not assumed necessary for UID separation;
aggregate resource constraints need their own enforcement proof.

After provisioning, the production launcher must still establish and verify
distinct host IDs, drop groups/privileges, constrain filesystem/process/network
access, deliver only the bound private capability, and pass the recorded
cross-process regular-file descriptor negative target. The earlier probe did
not demonstrate socket theft, and no socket-protection claim is made.

## Frozen v1 and actual resource results

External protected pin:
`af41ea2465e38c87dfe86471c33c6b54ea8564e1`,
tag `sea-forge-security-v1-2026-10-16`.
Frozen manifest SHA256:
`986454070b7e7f3acfbfe89f4b747fa4e22477cfe300653cd337c50bd70a1e66`.
All 44 protected files/modes/Git objects match before and after this increment.
The original test and gap inventory are untouched.

Latest [v1 result](../evidence/security-v1/implementation-continuation-20261016/all/results.json):
run `7ae96e02-6dd3-4cb6-b5d4-9e3e69689156`,
recorded `2026-10-17T01:43:30.025623+00:00`.
SHA256 `e974c8c81409e33d19e99a3a06c0fb134cec05ec7edc14db1d02592dd60e96c4`.
Rust A/B log hashes:
`45cb2c1794cd072e03285a93ce0a38884233e623bee9b440f9da6e767a9896bb`,
`aad5e29fbc03f21b1edee054a286872f994831ae7bfc2afa8515f08f0f4668c3`.

- MCP: actual upstream log has exactly one metadata.read and no prohibited
  handler invocation in the direct/retry probe. Both audit.delete attempts
  return -32001; protected audit digest remains
  `9642fb1a5cfafce59185718d6b969707a28ae854e637f1d4fba1aeda5abfcd25`.
  Canonical corpus: 18 invalid frames refuse; three valid attack batches deny
  the prohibited call while retaining their reply/effect assertions.
- PostgreSQL 18.1: eight actual destructive/escalating attempts receive native
  SQLSTATE42501, psql exit3. Same-session read controls succeed; five privileged
  reader flags are false. Table remains present with two rows and unchanged
  digest `3d74f767a718a7e53cfcc44cb95c5d5a889d8ac8c9874c6e0848ade47caca20d`.
  This is native role evidence, not product session mediation.
- Filesystem: prohibited write is actually attempted and receives EACCES13.
  Protected digest stays
  `06ebb27e5df9512149363c4be7accb7b424d746db456dbe9987601129712e146`;
  false completed/write claim settles rejected with file_digest_mismatch.
  Genuine allowed write succeeds with digest
  `29229c6162f8edabc1db0aeaca7fc7b924b2b83278818a7c62620b365054df26`
  and settles accepted.
- Child environment: the actual helper runs without the synthetic inherited
  credential. Alternate filesystem/network/credential paths remain unqualified.

| Full contract | Status | Outstanding qualification |
|---|---|---|
| SEC-01 | BLOCKED | Isolated kernel workload identity and production delivery |
| SEC-02 | BLOCKED | Credential isolation, issuance, renewal, expiry and revocation |
| SEC-03 | BLOCKED | Frozen stdio denial scope passes; production MCP execution integration remains required |
| SEC-04 | PASS | Frozen canonical stdio scope only |
| SEC-05 | BLOCKED | Production workload-bound database credentials and sessions |
| SEC-06 | BLOCKED | All alternate access paths |
| SEC-07 | BLOCKED | Native resource privilege scope and lifecycle |
| SEC-08 | BLOCKED | Active-session revocation |
| SEC-09 | BLOCKED | Trusted exposure and destination enforcement |
| SEC-10 | BLOCKED | Integrated private-context/alternate-output injection attempts |
| SEC-11 | BLOCKED | Bounded delegation, inherited exposure and revocation |
| SEC-12 | BLOCKED | Full authenticated resource/evidence/settlement correlation |

## Commands, review hashes and dependency changes

[Captured commands](../evidence/security-v2/continuation-20261016) retain complete
logs and actual exit statuses.

| Command | Observed result |
|---|---|
| Both full affected regression targets, --nocapture | 23 pass, 0 fail |
| just test (workspace/all-features/locked) | Exit0; 1467 pass, 0 fail, 23 existing ignored |
| Rebuild actual MCP broker, then unchanged v1 run.py all | Build exit0; runner exit1, 6 PASS / 10 NOT-YET-EXECUTABLE |
| just check | Exit0 |
| just proof / just no-async-kernel | Both exit0 |
| graft build | Exit0 |
| Pinned-object v1 verifier before/after | Exit0; protected files match |
| Direct redacted scans of new v1/v2 evidence | Exit0 after exact independently classified public-hash exclusions |
| Post-commit just security | Exit0; dependency checks pass, 471 commits scanned with no leaks |
| Final just status-check | Exit0; status revision108, operator-staged paths preserved |

Full regression log SHA256:
`6ce81828c6627e05112cd8fed9172b650a434f60a9c9cd6171f1b40dc0b13b0f`.

[Reviewed migration manifest](../../tests/security-v2/migrations/manifest.json)
SHA256 `8cec41162e571d10cc66299083a2cbf41d9a8a782dc09fe35eeb191662e3ce78`.

| Reviewed candidate | SHA256 |
|---|---|
| conformance_cep_authority.rs | 876aa3d9340df6c59338b8cc018de29dfd9e630da7c07c26b423c6003c87a775 |
| sfwp_case_mutations.rs | 95b61318f00c75f09f3fe6f4779d3c36233e814340462d707a84dfc1f88c6fd2 |
| support/authenticated.rs | ef51962a89490c2ad1f44633550db621124e5e3ecbb6bb22f24d7719e960b396 |

These are reviewed legacy candidates, not an independently frozen supplementary
qualification contract. The previous candidate manifest and its coverage remain
unchanged. No mandatory v1 gap is discharged by these hashes or by a policy
verdict alone.

No Rust/JavaScript/runtime dependency was added. Optional test-only pg0 remains
the previously pinned provisioner. Landlock, OpenShell, authoritative JSONL,
capability memory and SQLite FTS5 ownership are unchanged. Teleport code and
custom cryptography were not used.

[Scanner analysis](../evidence/security-v2/continuation-20261016/idempotency-scan-analysis.json)
classifies 18 generated public idempotency-digest matches and one recomputed
reviewed-source SHA256. New exceptions require exact path, exact public value
and one rule; raw evidence is retained. No directory-wide or generic-hex
exemption was introduced.

## Commit and next action

Migration and initial continuation evidence commit:
`c8e2a57915a67889bb9cbd0d32ea2b85eda41631`
(`test(security): authenticate legacy multi-actor workflows`).
The unchanged pre-commit hook passed with the exact cached pinned tool profile
through its supported local-tool fallback. No hook or gate was disabled;
the [commit receipt](../evidence/security-v2/continuation-20261016/commit-01/result.json)
retains the result. This increment changes regression coverage, not production
enforcement; the production boundary introduced in the prior increment remains
unchanged.
Prior implementation/evidence: `e96c45ca2e39f05ea3f15e83a7554d2745ebd9a5`;
prior checkpoint report: `dd494c1ae865487fa928647703be275087a943b0`.
Operator-staged status/Explore/workspace changes remain separate.

Proceed after operator provisioning or a suitable runner is available. First
prove distinct host UID/GID separation and the descriptor negative target,
wire the launcher/MCP/PG paths into real execution, and rerun Stage A resource
controls. Then implement B/C and obtain independent supplementary test review
and freeze before qualification. This is not completion or adversarial
validation.

Graft estimates from Luna recon/review total approximately 550,561 tokens
avoided (~$0.43); these are tool estimates, not measured implementation savings.

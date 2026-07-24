# Release Evidence TODO: Real ACP, SWE_SEED, and macOS Seatbelt

## Status

The Task 19 portable closeout is green. Three release/platform claims remain
unproved:

| Claim | What is proven | What is missing |
| --- | --- | --- |
| Real ACP | Scripted ACP v1 fixture | A compatible external host under the required sandbox posture |
| Real SWE_SEED | Portable harvest and declaration reconciliation, separately | One real projected host and one real authority in the same run |
| macOS Seatbelt | Unsupported platforms fail closed | A working macOS jail backend and enforced conformance tests |

Do not change the claim tables or `CURRENT_STATUS.md` until the corresponding
evidence exists. Skips remain skips; they are never passes.

## 1. Real ACP release gate

**Current gate:**

```sh
SEA_FORGE_REAL_ACP_ARGV='["/absolute/path/to/acp-host","..."]' \
SEA_FORGE_REAL_ACP_ENV='["PATH=...","HOME=...","KEY=value"]' \
cargo test -p sea-forge-server --test conformance_m16 \
  t16_1_real_acp_host_release_gate \
  -- --ignored --exact --nocapture
```

The current test proves only that a configured process settles `accepted` and
produces `agent_task_evidence`. It does not yet prove T16.1's required jail
grant, read-only task, or real-host permission mapping.

### Required work

1. Select and pin an ACP v1-compatible host. Record its executable version,
   source revision, and the command argv used for the run. Supply every child
   environment value explicitly; jail execution clears the ambient environment.
2. Change the real-host fixture policy to grant `sandbox_class: jail`. Make the
   task deterministic and read-only.
3. Add assertions for the ACP session record, transcript artifact, transcript
   SHA-256, accepted settlement, and no write outside the run workspace.
4. Run a real permission-request scenario. Assert that SEA records the request
   and its allow or deny decision.
5. Choose the host's network model before running under jail:
   - use an offline/local host, or
   - implement an authority-granted scoped egress projection for ACP jail
     sessions.

   The current jail ACP path has `NetworkPosture::Denied` and no scoped egress
   projection, so a hosted-model agent cannot work under a jail grant without
   that implementation.

### Acceptance evidence

Keep the test output, host identity/version, policy hash, command argv with
secrets omitted, ledger verification output, settlement, ACP session record,
and transcript artifact/digest. The evidence must identify the tested commit.

## 2. Real SWE_SEED release gate

**Current gate:**

```sh
SEA_FORGE_REAL_ACP_ARGV='[...]' \
SEA_FORGE_REAL_ACP_ENV='[...]' \
SEA_FORGE_REAL_SWE_SEED_REPO='/absolute/path/to/exact/checkout' \
SEA_FORGE_REAL_SWE_SEED_COMMIT='<40-character-commit>' \
cargo test -p sea-forge-server --test conformance_m16 \
  t16_6_real_swe_seed_release_gate \
  -- --ignored --exact --nocapture
```

The present test checks only that `harvested_refs` is nonempty. Its policy has
no SWE_SEED settlement authority, so it cannot prove the complete T16.6 claim:
a real `SweSeedTransport` declaration correlated to the same run.

### Required work

1. Provide a real SWE_SEED-projected ACP host that writes route/proof artifacts
   under the run workspace's `.agent-harness/` directory.
2. Bind the run to a repository at the supplied immutable commit. The test must
   reject a changed HEAD or a mismatch between the configured repository and
   commit.
3. Extend the release fixture with an operator-supplied, real SWE_SEED
   authority command and policy descriptor. Do not use the
   `internal-test-swe-seed` helper for this proof.
4. Assert that every harvested reference resolves under `.agent-harness/` and
   has its recorded SHA-256.
5. Assert that the authority's `settlement_declaration` names the same case,
   run, plan item, settlement, transcript hash, and harvested-reference
   manifest.
6. Assert that `swe-seed-correlation.json` includes that declaration ID and
   `verify_swe_seed_completion` reports the expected declaration.
7. Re-run reconciliation and prove it is idempotent.

### Acceptance evidence

Keep the real host and authority identities, repository URL/path and commit,
proof files and hashes, declaration, correlation view, ledger verification
output, and transcript. Do not put credentials or secret environment values in
the repository, test output, or retained evidence.

## 3. macOS Seatbelt

This is an implementation gap, not an unrun release command. `JailSandbox`
currently rejects non-Linux hosts, and the existing macOS test proves that
fail-closed unsupported behavior only.

### Required work

1. Implement a `cfg(target_os = "macos")` Seatbelt backend for ordinary
   execution and interactive ACP children.
2. Generate profiles from canonical workspace and artifact roots. Preserve
   argv-based execution, explicit child environment, timeouts, and process
   cleanup. Fail with `jail_unavailable` if profile generation or enforcement
   cannot be established; never fall back to `local`.
3. Add macOS-only conformance tests that run on the supported macOS runner:
   - writing inside the workspace succeeds;
   - writing outside it fails and creates no outside file;
   - default-denied connect accepts zero fixture-server connections;
   - default-denied bind/listen fails;
   - each supported explicit network grant succeeds;
   - interactive ACP children inherit the same confinement.
4. Establish the exact network scope Seatbelt can enforce. If it cannot enforce
   the current exact-port contract, resolve that contract deliberately and
   document the supported macOS limitation before claiming network confinement.
5. Run the macOS-specific tests and `devbox run -- just ci` on the existing
   `macos-latest` job. On that designated supported runner, unavailable
   Seatbelt is a failure of the release gate, not a passing skip.

### Acceptance evidence

Keep the macOS version, Seatbelt backend/profile identity, focused test output,
and full CI result. The tests must demonstrate denied side effects, not merely
return an error from the child process.

## Closeout order

1. Define the real ACP host and SWE_SEED authority interfaces, including their
   secret-handling path.
2. Strengthen the real ACP and SWE_SEED release tests before treating their
   current ignored tests as sufficient proof.
3. Implement Seatbelt and its macOS-only conformance suite.
4. Provision the real host, real authority, exact repository checkout, and
   macOS runner.
5. Run the three gates, retain the evidence bundles, then update the status and
   claim tables.

## Source context

- `crates/sea-forge-server/tests/conformance_m16.rs` — current real-host gates.
- `crates/sea-forge-server/src/delegation.rs` — ACP jail spawning and SWE_SEED
  harvest/declaration flow.
- `crates/sea-forge-sandbox/src/jail.rs` — Linux-only jail implementation.
- `crates/sea-forge-sandbox/tests/conformance_m1.rs` — Landlock and current
  macOS skip-not-pass behavior.
- `.agents/specs/spec-agent-orchestration.md` §17.5 — T16.1 and T16.6 evidence.
- `.agents/specs/spec-full.md` §§10.1 and 17.5 — jail and real-integration
  requirements.

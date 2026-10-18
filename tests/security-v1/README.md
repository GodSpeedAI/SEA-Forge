# Frozen security acceptance v1

Read [CONTRACT.md](CONTRACT.md), [inventory.json](inventory.json),
[BASELINE.md](BASELINE.md) and [HANDOFF.md](HANDOFF.md). The normative repository
entry is `.agents/specs/security-acceptance-v1.spec.md`.

Run from the repository root using Python 3.10+ and the repository's pinned Rust
toolchain. No dependency was added; no JavaScript runtime or live model is needed.
Scripted JSON attacks play the attacker role. Provisioning and missing interfaces
never change assertions or produce a green skip.

```bash
python3 tests/security-v1/selftest.py
python3 tests/security-v1/run.py A --output .agents/evidence/security-v1/<unique-run>/A
python3 tests/security-v1/run.py B --output .agents/evidence/security-v1/<unique-run>/B
python3 tests/security-v1/run.py C --output .agents/evidence/security-v1/<unique-run>/C
# Or all affordable gates together:
python3 tests/security-v1/run.py all --output .agents/evidence/security-v1/<unique-run>/all
python3 tests/security-v1/regression.py --output .agents/evidence/security-v1/<unique-run>/regressions
```

Replace `<unique-run>` with a new directory name; existing output directories are
refused. Every status except PASS exits nonzero, including NOT-YET-EXECUTABLE.
Rust failures without a complete case record are BLOCKED, never evidence of a
security violation. The runner captures logs and correlated `results.json`.
`RUSTC_WRAPPER=` may be supplied when workstation sccache is unavailable; it does
not change product isolation. Resource tests require normal local process/socket
permissions. A host restriction that prevents execution is an infrastructure block.

The existing `just crate-test` recipe cannot select a test binary and pass
`--nocapture`; the runner uses locked Cargo for that missing recipe. Existing
`just check` and `just test` remain required and are not replaced by the runner.
`just test` intentionally includes the RED Rust acceptance assertions. Python
PostgreSQL/MCP tests additionally require the gate runner. `just proof` remains
the independent P1–P4b minimum proof gate.

## PostgreSQL: provision once, assert identically

Default: `postgres_fixture.py` locates `psql`, `initdb`, and `pg_ctl` on PATH,
under `/usr/lib/postgresql/*/bin`, or in `SEA_FORGE_SECURITY_PG_BINDIR`. It starts
a private socket-only SCRAM cluster, then stops and removes it. It never uses
Docker. A non-root local fixture owner is required by initdb.

External instance: provide `SEA_FORGE_SECURITY_PG_ADMIN_DSN` in the trusted
test driver's environment as a `postgresql://user:password@host:port/database`
URI. The target must be a disposable real PostgreSQL database. The fixture owner
needs CREATE SCHEMA and CREATE ROLE privileges. A provider such as pg0 may supply
that URI; pg0 is optional and is not installed or invoked by this suite. External
instances need only `psql`, not local server binaries. URI options supported by
the provisioner are host, port, sslmode, sslrootcert, sslcert, sslkey and
connect_timeout; duplicate/unknown options fail explicitly.

The fixture creates unique schemas and a fresh metadata-only login with a
separate generated password. Its native acceptance assertions live exclusively
in `postgres_probe.py`, shared by every provider. An optional
`SEA_FORGE_SECURITY_PG_AGENT_DSN` supplies a different agent-facing connection
route; its user/password are replaced by the generated restricted role's values.
Administrative credentials, ambient PG settings, `.pgpass`, and owner TLS client
keys are not forwarded to agent queries. Credentials never appear in command
arguments or evidence. Supply only disposable/synthetic credentials and TLS material.

Each SQL attack and its metadata positive control execute on the same PostgreSQL
connection and record the backend PID. Native privilege SQLSTATE, unchanged row
count and ordered data digest are mandatory. A separate owner write control
proves mutation is supported. Cleanup drops only generated schemas/roles; it
never stops an externally supplied instance. A native fixture PASS does not
prove SEA Forge credential issuance, runtime identity, network isolation or
active-session revocation. Those remain separately inventoried obligations.

```bash
# Dedicated PostgreSQL subcase, with the same environment as Gate A:
python3 tests/security-v1/postgres_probe.py
```

## MCP: real ingress, unrestricted observed upstream

Set `SEA_FORGE_SECURITY_MCP_COMMAND` to JSON argv for the real SEA Forge stdio
MCP ingress, or a trusted deployment wrapper that configures that ingress through
its actual configuration interface. Configure its restricted workload externally:
`metadata.read` allowed, `audit.delete` denied. Do not implement policy in the
wrapper or upstream fixture. There is currently no such production MCP interface
in this repository, so the default observed result is BLOCKED.

The trusted provisioner receives `SEA_FORGE_SECURITY_UPSTREAM_COMMAND` (JSON argv
launching `mcp_upstream.py`) and `SEA_FORGE_SECURITY_OBSERVER_DIR`. These variables
are fixture inputs for deployment configuration, not a proposed production API.
Only the trusted gateway/provisioner/upstream receives the observer path. The
upstream advertises both tools, enforces no authorization and performs a real
synthetic audit-file deletion when the prohibited handler runs. Its fsynced log
records handler names and argument digests independently of gateway responses.

Discovery filtering, direct denied calls and reconnect/retry are tested separately.
Every malformed raw frame gets fresh protected data and a new initialized session.
The frozen corpus contains duplicate/escaped keys, conflicting envelopes, wrong
types, invalid UTF-8, invalid batches and trailing JSON. Valid mixed batches in
both attack orders must preserve allowed calls and individually deny prohibited
calls, with responses correlated by ID. This follows
[MCP 2024-11-05](https://modelcontextprotocol.io/specification/2024-11-05/basic/messages)
and [JSON-RPC 2.0 batching](https://www.jsonrpc.org/specification#batch). A valid
Unicode/nested-argument call must execute upstream with exactly the expected digest.
The self-check points at the unrestricted upstream directly and requires both
probes to return RED; that harness test is never reported as product security PASS.

## Evidence boundary and coverage limits

The trusted driver confines evidence to `.agents/evidence/security-v1`, rejects
symlinked/traversing output paths, and creates private, new directories. Disposable
agent workspaces are separate and do not receive the final evidence directory.
Mode bits alone do not isolate arbitrary malicious host code sharing the owner's
UID. Live same-UID evidence-store tamper resistance, hostile gateway binaries,
and production observer authentication are explicit unproven harness boundaries.
The attacker simulation controls inputs/actions, not the observer executable.

SEC-01's executable probe covers actor substitution on one existing connection;
its safe submit and missing/unbound/role/self-approval controls are meaningful.
It cannot establish independently authenticated B or full separation on fresh
same-UID sessions. SEC-12 covers the current public operational settlement API
with actual filesystem refusal and a real successful write control; it does not
prove every production settlement ingress authenticates observers. The inventory
keeps these wider obligations non-green. Gate B/C retain full seven-part
specifications and frozen attack fixtures, with explicitly incomplete integration.

## Verify the freeze from an external pin

Retain the full frozen commit outside the implementation agent's write access.
Obtain it from the separately protected remote tag or an operator-held record.
Do not trust a working-tree manifest, locally movable tag, or working-tree verifier.
Execute the verifier **from the pinned Git object**, with replacement refs disabled:

```bash
PIN=<full-40-character-frozen-commit-from-independent-record>
set -o pipefail
git --no-replace-objects show "${PIN}:tests/security-v1/verify.py" |
  python3 - --commit "$PIN"
```

The verifier authenticates the manifest against that commit, verifies its baseline
ancestry, compares every protected Git blob/hash/mode with the working file,
checks the manifest copy itself and refuses symlinks. It ignores local Git replace
refs. Supplementary tests may be added under a separately versioned path; v1's
contract, expected outcomes, fixtures and assertions must remain byte-identical.

Actual commit, tag, remote publication and protection readback belong in
`.agents/evidence/security-v1/freeze-receipt.json` after freezing. Local tag creation
alone is not independent integrity protection. A receipt in this editable tree
alone is also insufficient; the externally protected ref or independently retained
commit is the trust anchor.

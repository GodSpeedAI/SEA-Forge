# Bootstrap configuration remediation evidence

Date: 2026-08-31

## Applied

- Replaced the mismatched local SOPS identity at the requested path after preserving the prior identity outside the repository. Updated the public recipient rule and regenerated `secrets/dev.enc.env` from the ignored `secrets/dev.env`. No secret value is recorded here.
- Added ignored runtime `.sea-forge/server.yaml` with the local OpenAI-compatible endpoint and explicit `allow_loopback_test: true`; it contains no credential or asserted endpoint status.
- Terminated only audited PID 306180 after exact command-line verification.
- Restaged the ignored Tauri sidecar using `just workbench-sidecar debug`.
- Installed the exact declared Bun 1.4.0 host runtime. The configured npm registry does not publish requested 1.4.1, so the declared pin remains truthful at 1.4.0.
- Normalized `scripts/doctor.sh` to LF; its prior CRLF form failed before diagnostics in the Devbox Bash environment.
- Hardened `sea-forge-server`: extra argv now exits 2 before root/config/socket effects; strict server config rejects unknown fields and unsafe bounds; one root lock protects all socket overrides; non-socket and symlink socket destinations are preserved; slow partial requests time out; accepted connection tasks are bounded to 64.

## Verification

| Command | Result |
| --- | --- |
| `just secrets-check dev` with `SOPS_AGE_KEY_FILE=~/.config/sops/age/keys.txt` | pass; profile decrypts, declared variable redacted |
| `just crate-test sea-forge-server unknown_argument_exits_two_without_creating_cell_state` | pass |
| `cargo test -p sea-forge-server --locked --lib load_` | pass |
| `cargo test -p sea-forge-server --locked --test conformance_transport_hardening` | pass: 9 tests, including adversarial root override, file/symlink, stale socket, slow-client, and oversized-line cases |
| `just workbench-sidecar debug` | pass |
| staged sidecar `--version` with a temporary `SEA_FORGE_ROOT` | exit 2; root not created |
| `bun --version` | `1.4.0` |
| `just workbench-contracts-gate` | pass |
| `cd workbench && bun run check` | pass, 2 pre-existing warnings and 0 errors |
| `devbox run -- just doctor` with Nix profile on PATH and `SOPS_AGE_KEY_FILE` | pass; all checks OK, `mcp_api` correctly skipped |


## Follow-up owner decisions applied

- Removed the six tracked, empty `cc*.cdtor.o` compiler artifacts.
- Retained the gauntlet bootstrap example as versioned source; `cargo check -p sea-forge-server --locked --example journey_gauntlet_bootstrap` passes.
- Documented the retained local-draft IPC contract and future renderer test requirements in `docs/execution/WORKBENCH_DRAFTS.md`; the UI product story remains a scoped future decision.
- Added `.agents/specs/spec-server-request-admission.md`, a draft normative specification for bounded request-work admission and timeout containment. It is intentionally not an implementation claim.

## Deliberately not changed

- `.cargo/config.toml` stays serial/non-incremental by repository policy.
- Draft IPC commands remain registered by the documented product decision; `sfwp_cell` is already used and is not part of that decision.
- Endpoint configuration alone does not authorize live delegation: identity, policy, templates, and an approved endpoint remain separate governed prerequisites.
- Deeper bounded admission before work detached by the existing 10-second request timeout remains an architecture hardening item; this remediation did not alter its durable-work semantics.

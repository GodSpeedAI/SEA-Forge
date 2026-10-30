# Security Verification Scripts (scripts/security)

## Overview
Standalone probes and gates for the kernel-security campaign (SEC rounds;
SEC-06 / ADR-014). They separate kernel behavior from SEA-Forge code, retain
security evidence outside Git with a committable digest manifest, exercise
the round-4 pathname-socket containment gates, and canary-test the staged-file
secret scanner against synthetic tokens.

## Interface
- `notif_trap_probe.py` — kernel probe: installs a seccomp user-notify filter
  (same BPF shape as `crates/sea-forge-sandbox/src/socket_policy.rs`)
  trapping `connect(2)`-family syscalls (42, 44, 46, 307 and their x32
  variants) and answers the notification with errno 13 from a supervisor
  thread. Prints one verdict line distinguishing "trapped + denied by broker"
  from "untrapped" or a hung worker. Used by the kernel-security preflight
  ([notif_trap_probe.py:module docstring, BPF program construction,
  supervisor/worker threads]).
- `retain-evidence.py` — evidence retention tool. Subcommands:
  `archive --out MANIFEST [--archive DIR] EVIDENCE_DIR...` and
  `verify MANIFEST [--archive DIR]` ([retain-evidence.py:argparse setup]).
  `archive` copies evidence directories unfiltered into an owner-only store
  (default `$SEA_FORGE_EVIDENCE_ARCHIVE` or
  `~/.local/share/sea-forge/evidence-archive`; directories 0700, files 0600)
  and writes a manifest with per-file SHA-256 and size, git provenance of the
  producing tree (HEAD, head tree, branch, worktree diff/status digests), and
  observer/run identifiers from any `results.json`
  ([retain-evidence.py:archive, provenance, run_refs]).
  `verify` re-hashes every archived file and exits non-zero on any missing,
  changed, extra, or not-owner-only file ([retain-evidence.py:verify]).
- `round4-socket-gates.sh` — SEC-06 / ADR-014 pathname-socket containment
  gates. Must be run from a process tree without an inherited seccomp
  user-notify listener (`Seccomp: 0` or `1` in `/proc/self/status`)
  ([round4-socket-gates.sh:header]). Probes mapped user namespaces from
  `/etc/subuid` plus `unshare` to decide whether to set
  `SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION=1` or record the suite as
  NOT-EXECUTABLE ([round4-socket-gates.sh:isolation probe]); runs
  `cargo test` for `sea-forge-sandbox`, `sea-forge-server --test
  postgres_broker`, and (when a PostgreSQL install root is available via
  `SEA_FORGE_SECURITY_PG_INSTALL` or the newest `/usr/lib/postgresql`)
  `--test postgres_lifecycle` ([round4-socket-gates.sh:run() and suite
  selection]); then runs mutation controls M-S2 (supervisor allows every
  pathname target) and M-S4 (helper drops socket grants), requiring each
  mutant to be killed by
  `cargo test -p sea-forge-sandbox --test unix_socket_reach`
  ([round4-socket-gates.sh:mutation()]).
- `staged-secret-canaries.sh` — staged-file secret-scanning canaries (SEC
  round 3). Runs `gitleaks` with the repository's `.gitleaks.toml` against a
  throwaway repository holding synthetic GitHub-PAT-shaped tokens generated
  at run time, and asserts which states each scan mode catches: staged new
  file, staged modification, unstaged-only change, staged-then-reverted,
  committed (history scan), allowlisted/commit-scoped/casework/value-scoped
  paths, path adjacent to an allowlist, and a clean negative control
  ([staged-secret-canaries.sh:token(), scan(), case list]). Usage:
  `scripts/security/staged-secret-canaries.sh [OUT_DIR]`; exits non-zero when
  an expectation fails ([staged-secret-canaries.sh:usage header and expect()]).

## Invariants
- `retain-evidence.py` retains passing, failed, invalid, and inconclusive
  runs alike — nothing is filtered — and the manifest carries digests and
  identifiers only, never file content, so it can be committed without
  widening any secret-scan allowlist ([retain-evidence.py:module docstring]).
- `retain-evidence.py` refuses to overwrite an already-archived directory
  and applies owner-only permissions (0700 dirs / 0600 files, umask 077)
  ([retain-evidence.py:archive() refusal, restrict(), umask]).
- `round4-socket-gates.sh` treats an unavailable workload-isolation pool as
  NOT-YET-EXECUTABLE (honestly recorded) rather than as a PASS, and skips
  mutation controls unless the baseline suite passed — a skipped or failing
  baseline cannot kill a mutant ([round4-socket-gates.sh:isolation probe and
  mutation gating]).
- Directory modes changed for helper reach (`o+x` on target-directory
  ancestors) are restored on exit, failure included
  ([round4-socket-gates.sh:RESTORE_MODES and restore_modes trap]).
- Mutation tests restore the mutated source file with
  `git checkout -- <file>` before re-verifying
  ([round4-socket-gates.sh:mutation()]).
- Canary tokens are synthetic, generated at run time, never stored in
  source, and no synthetic token may appear in any report or log (redaction
  assertion) ([staged-secret-canaries.sh:token(), redaction check]).
- `staged-secret-canaries.sh` unsets git `--local-env-vars` so every git call
  targets the throwaway repository, never the real one
  ([staged-secret-canaries.sh:env var stripping]).
- `check-e2e`-style honesty applies here too: `round4-socket-gates.sh` exits
  non-zero on any gate failure; the watchdog converts a hung test into a
  FAIL with its log preserved ([round4-socket-gates.sh:run() watchdog]).

## Side Effects
- `retain-evidence.py` writes an owner-only evidence archive and manifest
  files outside Git (under the archive dir and `--out`), runs `git`
  read/provenance commands ([retain-evidence.py:archive(), provenance()]).
- `round4-socket-gates.sh` writes evidence logs and `summary.txt` under
  `.agents/evidence/security-v2/round4-20261018/gates`, compiles and runs a
  Landlock ABI probe at `/tmp/llabi-gate.c`, temporarily changes directory
  modes on cargo target-directory ancestors, and temporarily rewrites and
  restores two source files for the mutation controls
  ([round4-socket-gates.sh:EV dir, llabi probe, chmod grants, mutation()]).
- `staged-secret-canaries.sh` creates and destroys a throwaway git
  repository and per-case JSON reports and logs under the output directory
  ([staged-secret-canaries.sh:fresh(), scan()]).
- `notif_trap_probe.py` installs a seccomp user-notify filter on its own
  worker thread and connects a Unix socket to a nonexistent path; no
  persistent effect ([notif_trap_probe.py:worker()]).

## Verification
- Run: `bash -n scripts/security/round4-socket-gates.sh scripts/security/staged-secret-canaries.sh && python3 -m py_compile scripts/security/notif_trap_probe.py scripts/security/retain-evidence.py`
- The substantive gates are the scripts themselves: `round4-socket-gates.sh`
  is the SEC-06 gate (requires an isolation-capable host) and
  `staged-secret-canaries.sh` is the SEC round 3 canary gate (requires
  `gitleaks` on PATH).

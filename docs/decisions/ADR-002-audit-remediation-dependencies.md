# ADR-002: Dependency choices for the four-spec audit remediation plan

## Status

Accepted

## Date

2026-07-22

## Context

`.agents/plans/2026-07-22-spec-audit-remediation.md` Task 0 requires exact,
research-backed dependency versions/features before any downstream task adds
code, per the repository rule that no task may guess a dependency. Three gaps
required new or upgraded dependencies:

1. **Task 7 (M4b)** — the memory index must become a rebuildable SQLite FTS
   projection (`spec-adlc-thoth-minimum.md` explicitly selects SQLite FTS; no
   crate is in the workspace today).
2. **Task 3 (M1)** — the jail backend must enforce default network denial;
   `crates/sea-forge-sandbox/Cargo.toml` currently pins
   `landlock = "0.3"` (resolved `0.3.1`) and `jail.rs` hardcodes `ABI::V1`,
   which predates Landlock's network-restriction ABI entirely.
3. **Task 16 (M13)** — `spec-agent-orchestration.md` §0 "Resolved decisions"
   (owner-accepted 2026-07-17) requires a sealed, encrypted canonical
   transcript for summarized-mode retention, verified before crypto-shredding.
   §6.3 states explicitly: *"This spec selects neither package nor version.
   Adding a dependency... requires a later owner approval of the exact
   versions and features."* No crypto/AEAD crate exists in the workspace
   today; `sha2` (hashing only) and `ed25519-dalek` (signing only) do not
   cover symmetric sealing.

Every version below was confirmed against live sources on 2026-07-22 (Context7
docs mirrors and the crates.io API — not training-data recall, per operator
instruction), because prior knowledge of Rust crate versions is not
trustworthy for a repository rule that requires exact current versions.

## Decision

### 1. `rusqlite` for the M4b memory-index FTS projection

Add to `crates/sea-forge-capability/Cargo.toml`:

```toml
rusqlite = { version = "0.32", features = ["bundled"] }
```

**Post-approval build regression, discovered during Task 7 implementation
(2026-07-22):** the originally approved `0.40` (→ `libsqlite3-sys 0.38.1`)
fails to compile on this repository's pinned `rustc 1.92.0` (`rust-toolchain.toml`).
`libsqlite3-sys-0.38.1/build.rs:110` unconditionally invokes the still-unstable
`cfg_select!` macro (tracking issue
[rust-lang/rust#115585](https://github.com/rust-lang/rust/issues/115585)),
which `rustc 1.92.0` stable rejects with `E0658` before any workspace code
compiles — confirmed independent of this plan's changes via a clean
`cargo check -p sea-forge-capability` on the unmodified `0.40` pin. No
`libsqlite3-sys` patch fixes this (`0.38.1` is current; `cargo info` confirms
no newer release exists as of this check). `rusqlite = "0.32.1"` resolves to
`libsqlite3-sys = "0.30.1"`, which does **not** use `cfg_select!` and builds
cleanly; its `build.rs` still passes `-DSQLITE_ENABLE_FTS5` unconditionally
under `bundled` (confirmed by reading `libsqlite3-sys-0.30.1/build.rs`), so
FTS5 availability is unaffected. This is a version-pin downgrade to the same
approved crate/mechanism (`rusqlite`, `bundled`), not a technology
substitution — `deny.toml` license allowances (`MIT`) are unaffected. Revisit
`0.40` once `cfg_select` stabilizes or a `libsqlite3-sys` patch avoids it.

- Confirmed current release: `0.40.1` (crates.io `max_version`/`newest_version`
  as of 2026-07-22; wrapper crate license `MIT`).
- `bundled` compiles `libsqlite3-sys`'s vendored SQLite 3.x from source via
  the `cc` crate rather than linking the host's system SQLite, giving a
  deterministic build across dev machines and CI regardless of host OS
  SQLite version.
- **FTS5 requires no separate rusqlite-level feature.** Confirmed directly
  from `libsqlite3-sys`'s `build.rs` (`build_bundled` function): the bundled
  compile always passes `-DSQLITE_ENABLE_FTS5` (alongside `FTS3`, `RTREE`,
  `JSON1`) unconditionally when `bundled` is enabled. Rusqlite itself has no
  `fts5` cargo feature — FTS5 tables are created and queried with plain SQL
  (`CREATE VIRTUAL TABLE ... USING fts5(...)`) once the underlying library
  supports it.
- Vendored SQLite C source is public domain and carries no separate SPDX
  license entry (it is not a distinct crates.io dependency); no `deny.toml`
  license-allow-list change is required.
- Alternative considered and rejected: system-linked SQLite (default
  features, no `bundled`). Rejected because FTS5 availability would then
  depend on how the host distribution compiled its SQLite package — not
  guaranteed on every dev machine or CI image, and it would reintroduce the
  "silent capability drift across environments" problem Task 7 exists to
  eliminate.

### 2. `landlock` upgrade for M1 jail network denial

Change `crates/sea-forge-sandbox/Cargo.toml`:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
landlock = "0.4"
```

- Confirmed current release: `0.4.5` (crates.io `max_version` as of
  2026-07-22; license `MIT OR Apache-2.0`, already allowed).
- `landlock` `0.3.x` predates `AccessNet` entirely (added in the ABI V4
  network-restriction release). `0.4` adds
  `AccessNet::{BindTcp, ConnectTcp}`, `NetPort` rules, and the
  `ABI::V4`/`V5`/`V6` variants, confirmed from the crate's own `compat.rs`
  source (V4 = Linux 6.7 network, V5 = Linux 6.10 ioctl, V6 = Linux 6.12
  scoped signals/abstract-unix-sockets).
- `jail.rs` currently hardcodes `let abi = ABI::V1;` twice (lines 42, 275).
  Task 3 replaces this with negotiated compatibility (the crate's
  `Compatible`/`RulesetAttr` pattern, e.g. `ABI::V4` requested with graceful
  degradation on older kernels), not a second hardcoded constant.
- **Network scope is TCP-only, and this matches the spec as written.**
  Landlock has never had UDP or raw-socket coverage in any ABI version
  through V6 (confirmed: `AccessNet` only defines `BindTcp`/`ConnectTcp`).
  `spec-full.md:1434` already scopes the requirement to exactly this: *"no
  network unless the rule grants `network: true` (default false; Landlock
  v4+ TCP restrictions where available, else document the gap on macOS)."*
  Task 3 therefore enforces TCP bind/connect denial by default and documents
  the UDP/raw-socket gap explicitly (in code comments and the M1 conformance
  suite) rather than inventing a broader mechanism the spec does not ask for.
- **No new dependency for network-namespace isolation.** A `nix`-based
  `unshare(CLONE_NEWNET)` approach was considered for full-protocol coverage
  but rejected for this plan: it is not what the spec requires, it adds a new
  direct dependency plus a privileged-operation portability question
  (unprivileged user/network namespaces are not universally available), and
  it would need its own explicit-grant proxy design that is out of scope.
  Revisit only if a future spec revision requires UDP/raw coverage.
- **macOS**: `crates/sea-forge-sandbox/src/jail.rs` has no Seatbelt branch
  today — `JailSandbox::new()`, `execute()`, and the interactive path all
  return `unsupported_sandbox_class_error` unconditionally off Linux. This
  matches `spec-full.md`'s own acceptance criteria verbatim ("Jail probe on a
  host without Landlock → fails preflight with `unsupported_sandbox_class_error`;
  nothing runs" and "Seatbelt cases on Linux CI MUST report as skipped, not
  passed"). Task 3 does not need to implement Seatbelt; it must not regress
  this already-correct fail-closed behavior.

### 3. `chacha20poly1305` for the M13 sealed summarized-mode transcript

Add to `crates/sea-forge-server/Cargo.toml` (the crate that owns
`case_dispatch.rs`/retention resolution per Task 16):

```toml
chacha20poly1305 = { version = "0.11", features = ["zeroize"] }
```

- Confirmed current release: `0.11.0` (crates.io `max_version` as of
  2026-07-22; license `Apache-2.0 OR MIT`, already allowed). Published by the
  RustCrypto `AEADs` project; pure Rust, `no_std`-capable, with optional
  architecture-specific acceleration — no `unsafe` in the consuming crate
  (workspace-wide `unsafe_code = "deny"` lint is unaffected).
- The crate ships **XChaCha20Poly1305** in the same package (confirmed via
  crates.io keywords `xchacha20`, `xchacha20poly1305`), which uses a 192-bit
  random nonce — safe to generate per-seal with the existing `rand`/
  `getrandom` workspace dependencies without a nonce-reuse counter to manage.
- `features = ["zeroize"]` wires the crate's key material to the workspace's
  existing `zeroize` dependency (already used in `sea-forge-agent` for
  credential handling), zeroizing the in-memory key after use.
- **Design**: generate one random 256-bit key per delegation run at seal
  time; store it at `.sea-forge/sealed/<run_id>.key` (mode `0600`) — a path
  outside the public `full`-mode artifact surface (`artifacts/` and
  `runs/<run_id>/` evidence). Seal the Task 2 canonical redacted transcript
  bytes with `XChaCha20Poly1305`; store the ciphertext next to the run's
  other evidence (its own confidentiality comes from the key file, not from
  its storage location, so it does not need special placement). Verify the
  AEAD tag by decrypting immediately after sealing, before settlement
  completes (Task 16 step 4's "verify before completion"). Crypto-shredding
  is deleting `.sea-forge/sealed/<run_id>.key`; without it, the ciphertext is
  permanently unrecoverable while `transcript_sha256` and `summary` (per
  `spec-agent-orchestration.md:190-194`) remain independently verifiable.
- Alternative considered: `aes-gcm` (same RustCrypto family, same license,
  hardware-accelerated via AES-NI on capable CPUs but slower without it — a
  pure-portability tradeoff, not a security one; `chacha20poly1305` was
  preferred for uniform performance across dev/CI/production without an
  AES-NI dependency).
- Alternative considered and rejected: the `age` crate / existing
  SOPS+age operator-secrets infrastructure (`justfile` `secrets-*` recipes,
  `.sops.yaml`). Rejected because `age` is designed around long-lived
  recipient keypairs for human-operated file encryption; reusing the
  operator's single age identity as the transcript-sealing key would mean
  crypto-shredding one run's transcript requires destroying the key for
  *every* sealed transcript across the whole installation, which defeats the
  per-run crypto-shredding semantics the spec requires.
- Alternative considered and rejected: `ring` (already present transitively
  via `reqwest`'s `rustls-tls` feature). Rejected in favor of the
  RustCrypto family for a less C/BoringSSL-derived dependency footprint and
  a simpler license story under `deny.toml`.

## Alternatives Considered

See the per-dependency "Alternative considered" notes above; each dependency
had one narrower/rejected option, not a separate section, because the
tradeoffs are local to that dependency rather than to the plan as a whole.

## Consequences

- Three new/upgraded direct dependencies: `rusqlite` (new, `sea-forge-capability`
  only), `landlock` (upgraded in place, `sea-forge-sandbox` only, Linux-only
  target dependency), `chacha20poly1305` (new, `sea-forge-server` only).
  None touch a kernel crate covered by `just no-async-kernel`— `rusqlite` and
  `chacha20poly1305` are synchronous libraries, `landlock` was already a
  sandbox-only dependency.
- `cargo deny check licenses` requires no `deny.toml` changes: `MIT`,
  `Apache-2.0`, and `MIT OR Apache-2.0`/`Apache-2.0 OR MIT` are already
  allowed.
- `landlock` 0.3→0.4 is a breaking API change for `jail.rs` (new `Compatible`
  trait shape, `AccessNet` addition); Task 3 owns that rewrite. This ADR only
  approves the version/feature choice.
- The sealed-transcript key-file path (`.sea-forge/sealed/<run_id>.key`) is a
  new persisted artifact location; Task 16 must add it to whatever inventory
  enumerates `.sea-forge/**` runtime output and must never commit it (per the
  plan's guardrail against committing `.sea-forge/**`).
- None of these three choices requires a `Cargo.lock` change beyond the
  ordinary `cargo fetch`/`cargo build` refresh each owning task performs when
  it lands its dependency addition; Task 0 does not modify `Cargo.toml`
  itself, only records the approved exact choice.

## References

- `.agents/plans/2026-07-22-spec-audit-remediation.md` Task 0, Task 3, Task 7,
  Task 16
- `.agents/specs/spec-agent-orchestration.md` §0 "Resolved decisions", §6.3,
  §7.4 (TranscriptEvidence)
- `.agents/specs/spec-full.md:1434` (Landlock v4+ TCP scope), `:302`,
  `:1774`, `:1784`, `:1901`, `:1905`
- `.agents/specs/spec-adlc-thoth-minimum.md` (SQLite FTS selection, M4b)
- crates.io API: `rusqlite` (`0.40.1`), `landlock` (`0.4.5`),
  `chacha20poly1305` (`0.11.0`) — fetched 2026-07-22
- `github.com/rusqlite/rusqlite` `libsqlite3-sys/build.rs` (`build_bundled`,
  `-DSQLITE_ENABLE_FTS5`) — fetched 2026-07-22
- `docs.rs/landlock/0.4.3` via Context7 (`AccessNet`, `ABI` enum, `compat.rs`)
- `crates/sea-forge-sandbox/src/jail.rs:42,275` (current `ABI::V1` pin),
  `crates/sea-forge-sandbox/Cargo.toml` (current `landlock = "0.3"`)
- Owner approval recorded via in-session confirmation, 2026-07-22 (three
  `AskUserQuestion` choices, all "Recommended" options accepted verbatim).

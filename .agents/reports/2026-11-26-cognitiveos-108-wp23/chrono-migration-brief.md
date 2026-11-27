# Migrating off chrono (RUSTSEC-2026-0543)

## Why

RUSTSEC-2026-0543 marks chrono as unmaintained in every version. cargo-deny therefore fails SEA's lint gate and its weekly Security job, on main and on the integration branch alike. We will not add a deny.toml waiver for it: that would weaken a production policy just to get a pass, and the owner ruled that out.

## Scope

chrono reaches SEA through three of our own repos. They have to migrate in this order:

1. **DomainForge** (`domainforge-core` 0.22.0). The migration changes its public API, so it ships as a new breaking minor, 0.23.0. Publishing is authorized.
2. **RealityTrace** (`sxr-*`, pinned by git rev in SEA).
3. **SEA-Forge**, which then moves to the new DomainForge version and the new RealityTrace rev.

The replacement is the `time` crate, which SEA's lockfile already contains at 0.3.55, so no new crate enters the tree. Record the choice in each repo's dependency notes.

## The rule that makes this safe

**Every serialized timestamp must keep its exact bytes.** Signed, digested and wire timestamps sit in signatures, digests, ledger entries, CEP envelopes and SFWP frames. Changing any of their bytes breaks verification and replay of existing records.

The recon (Haiku, read-only) found four formats in use:

| Format | Example | Where |
|---|---|---|
| chrono `to_rfc3339()` | `+00:00`, with an AutoSi fraction of 0, 3, 6 or 9 digits | DomainForge signing payload (`authority/signing.rs:20`); SEA ledger `created_at`, `committed_at`, `signed_at` (`sea-forge-ledger/src/types.rs`); CEP `expires_at` and obligation windows |
| chrono serde | `Z`, with an AutoSi fraction | DomainForge `FactEnvelope`; SXR `RecordContext` |
| `to_rfc3339_opts(Micros, true)` | `Z`, always 6 digits | SXR ledger, outbox and CEP envelope; DomainForge `cli/envelope.rs:152` |
| `Millis` + `Z` | always 3 digits | SFWP `case_records.rs:500` |
| `Secs` + `Z` | no fraction | `postgres.rs` `expires_at` |

The `time` crate's defaults differ from every one of these. It writes `Z` instead of `+00:00`, it trims trailing zeros from the fraction, and its default serde form is a tuple. Matching chrono needs explicit formats.

## Method (each repo, test-first)

1. **Pin the current bytes.** Before touching any types, add golden tests that pin today's exact output for every hotspot, with chrono still in place. Use fixed instants with fractions of 0, 1 ms, 1 µs, 1 ns and 999999999 ns. Commit these tests first, and they must pass on chrono.
2. **Add one helper module per repo** (e.g. `timefmt`), with one function per format above, plus `#[serde(with = ...)]` adapters that reproduce chrono's serde output and parse what chrono accepted. Inputs that used to be accepted must still be accepted, including `+00:00` and non-UTC offsets where `parse_from_rfc3339` allowed them.
3. **Swap the types.** Use `time::OffsetDateTime` everywhere, or a `UtcTimestamp` newtype if the repo wants a narrower type. Use `time::Duration` (`num_seconds` becomes `whole_seconds`). Route every format and parse through the helper.
4. **Prove the bytes held.** The golden tests from step 1 must pass byte-for-byte without being edited. Existing fixture and golden files must not change; a fixture diff is a failure.
5. **Prove chrono is gone.** `cargo tree -i chrono` must come back empty in the repo, and `cargo deny check advisories` must be clean of 0543.
6. **Run the gates.** Run the repo's full gates under the memory and flock rules, open a PR, get CI green, and resolve every CodeRabbit thread.

## Publish and pin

1. DomainForge: bump to 0.23.0 and publish through its release pipeline. Then pin SEA to 0.23.0.
2. RealityTrace: merge, then pin SEA to the merged rev, verified against origin.
3. SEA last. It needs a status revision and its own PR.

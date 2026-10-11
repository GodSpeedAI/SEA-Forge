# Retained-version helper scaffold — source receipt

Date: 2026-10-07. Append-only companion to
`run_observation_retained_version_helper_assignment_oct07.md`.

## Frozen new files

| File | SHA-256 | Bytes |
|---|---|---:|
| `run_observation_retained_version.go` | `37a9ab2a0341f1f3236968e7a5fd85f6a0a870b5010c918352c9d6ccd5f2f38a` | 5,829 |
| `run_observation_retained_version_test.go` | `ad27b93aa93dc7fadd9eb621d8986d64c34cee5bbac81d8b8c0ef193845898f4` | 12,502 |
| `run_observation_retained_version_helper_assignment_oct07.md` | `cc1fed59eb1f440e53756d3f2adee79369d0c57f17f800481b9c92a1093c446a` | 9,829 |

The manager source and test, contract, ports, adapters, hydration helper, and
selector were read-only. Only the two named Go files were added as source. The
original assignment record and this receipt are package-local evidence files;
no existing record was overwritten.

## Scope and behavior

The source file defines the private retained state, safe status outcome codes,
canonical ordered image structs, fixed-width decimal control encoding, and
deterministic image marshal helper. Its candidate publisher is an explicit
unavailable test-first stub that does not mutate or accept input state. It has
no lifecycle, manager, locking, I/O, auth, `Next`, or HTTP/SSE behavior.

The test file contains direct behavioral fixtures for accepted version/pointer
copy and timestamp/high-water; exact-ID first ordinal across reorder, payload
rewrite, and shorter source windows; nonterminal over-budget atomicity and
full-ledger recovery; terminal safe-state refusal; exact/one-over irreducible
metadata image bounds; fixed-width marker codes with maximum uint64 generation
and high-water; 1,024/1,025 frame boundaries; and ordinal overflow. All test
calls are synchronous and bounded; there are no sleeps, channels, or setup
waits. The stubs are intended to produce behavioral failures when a separately
authorized compiler owner runs them.

## Root-ratified canonical image field set

Root confirmed the minimal helper image: exact case/run/plan key; current safe
trace execution, settlement, declared observation status/counts, and accepted
timestamp; source-order current frame window with exact event IDs, first
ordinals and deep optional pointers; full exact-ID ledger sorted by opaque ID;
fixed-width availability code covering current, transient read failure,
retention failure, terminal retention failure, and stop scheduling; fixed
20-digit generation and high-water counters; and a schema-version string.
There is no duplicate next-ordinal counter and no additional speculative
control. Any integration-retained lifecycle controls outside this pure helper
must still be included in the full poller image accounting.

The current encoder serializes every listed field once, sorts only the
canonical ledger image, preserves frame order, and clones optional pointers
while building the image. It accepts the `1<<20` limit as a later publisher
responsibility; the stub does not make acceptance decisions. These private
encoding choices change no persisted/public schema.

## Deviation and verification limits

No Go test, compiler, typecheck, formatter, scanner, runtime, or Git command was
run. No RED/GREEN claim is made. The current candidate publisher always
returns rejected, and the focused fixtures are expected to fail behaviorally;
that expected failure has not been executed. An independent source critic must
review this receipt and both frozen Go files before any test run. Root retains
implementation release and architecture decisions.

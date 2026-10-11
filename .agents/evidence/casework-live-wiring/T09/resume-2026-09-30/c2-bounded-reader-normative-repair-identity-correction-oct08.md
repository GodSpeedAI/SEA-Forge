# Additive identity and traceability correction

Date: 2026-10-08. This supplements the immutable
`c2-bounded-reader-normative-repair-result-oct08.md` (SHA-256
`c0bd6ed9e6856c5e6869198f5780d04227e0644e034194e22fc6006134a6e6e8`). That
result's recorded post-edit hashes identify the immediately preceding source
state and are superseded by the final identities below. The original result was
not modified after it was written.

## Final identities

| Path | Initial pre-repair SHA-256 | Final SHA-256 |
|---|---|---|
| `.agents/specs/casework-live-cursor-v4-spec.yaml` | `c5a46ef01b677049cfb9a7e822c956990d403605b364a869f25a3dc9f07ddfca` | `4bb9730c25e582be159a35c5febe0f09f8af8ef4c17b59683599d147ff5e1b67` |
| `docs/decisions/ADR-008-casework-live-cursor-v4.md` | `67ab46ce7bc56a137ac99a025c3d6579d8837b533a7ea7d8027c17aa11d31eb5` | `c66912baee08317e4819ad211a208f5e2abff5d3369053a743a0f384737cc4e5` |

The final spec adds `REQ-C2-RANGE-005` to `V-C2-RANGE-01`'s requirement list.
That vector now pins the distinct stream/from/to digest domains, exact
presence/checked-length/UTF-8 encoding, and mismatch rejection before seek. The
ADR acceptance vector records the same digest checks. This directly traces the
approved exact-byte binding correction; no requirement or verification ID was
added. It does not alter the registered-stream predicate, seed zeroization,
startup entropy behavior, or existing limits.

This correction records identities only; it changes no source after the
hashes above. No tests, compiler, formatter, scanner, build, gate, or Git
operation was run. The normative package remains subject to independent review
and root acceptance; no implementation or runtime readiness is claimed.

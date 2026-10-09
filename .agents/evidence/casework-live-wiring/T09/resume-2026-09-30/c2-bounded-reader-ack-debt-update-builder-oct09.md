# C2 bounded-reader ACK debt update

Date: 2026-10-09. Read-only source inspection; no tests, gates, compiler, or
Git commands were run.

Updated only CW-37 in `.agents/DEBT.md` to record the current projection gap:
`crates/sea-forge-server/src/sfwp/events.rs:92-107`
(`frame_from_entry`) defaults missing/non-string `kind` and missing `detail`,
and maps non-string `case_id`/`run_id` to absence. Future C2 server integration
must validate every `sfwp_event` against `REQ-C2-RANGE-003` before filtering,
ACK/frontier advancement, or frame emission, including filtered-out rows; a
malformed event fails the candidate page without ACK. This records a future
integration requirement; it does not claim a current reader implementation or
close the gap.

Authority evidence: root design grant SHA-256
`f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`, ACK
addendum SHA-256
`1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`, and
independent ACK re-review SHA-256
`8fdc7dfea99cf9e16985e33ec25db894b524cfa5fdbbcddd2a76df42d3e9753e`.
The review approves the design grant only; implementation and server
integration remain pending. Normative source: `.agents/specs/casework-live-cursor-v4-spec.yaml`
`REQ-C2-RANGE-003`, lines 253–270.

Resulting SHA-256: `.agents/DEBT.md`
`6604564cc682e0277cedd1a6e13c015007acd3e2d4ec157fd9b76ef2ff536c71`;
`events.rs`
`2e69b1983bb76c4d8b79b5a5907788321f4af9cbf81d690494048ecaf5b90d1a`.
No historic debt was removed or reclassified. Material change is confined to
CW-37 status, evidence, impact, and next step; no source or approval boundary
changed.

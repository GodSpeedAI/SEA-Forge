# CW-37 observed event-validation debt: independent review

Date: 2026-10-09

## Verdict

**Approve the CW-37 update.** Its new observed-gap entry accurately records
the current event projection behavior, the C2 validation requirement, and the
approved-design/runtime-pending boundary. The reviewed `.agents/DEBT.md` diff
is limited to CW-37. No source, tests, gates, compiler, Git, status, or debt
state changed for this review.

## Evidence and identity checks

- Builder receipt: `c2-bounded-reader-ack-debt-update-builder-oct09.md`,
  SHA-256 `ce89f9877c32a7dcb433ca2ba2874881564215187b7282ae43368c556292c77a`.
- The reviewed diff changes only `### CW-37` in `.agents/DEBT.md` (30
  insertions, 7 deletions); the resulting file SHA-256 is
  `6604564cc682e0277cedd1a6e13c015007acd3e2d4ec157fd9b76ef2ff536c71`.
- The cited source is accurate: `frame_from_entry` maps a missing or
  non-string `kind` to `""`, maps non-string `case_id`/`run_id` to `None`, and
  maps missing `detail` to JSON null (`crates/sea-forge-server/src/sfwp/events.rs:92-107`).
  `get_range` filters by record kind and cursor ordinal before calling
  `frame_from_entry` (`events.rs:169-207`), so this projection currently does
  not validate those known event fields before filtering.
- The normative anchor is correct: `REQ-C2-RANGE-003` requires known field
  validation, including string `kind`, present `detail` that may be null, and
  absent/null/string `case_id` and `run_id` (`.agents/specs/casework-live-cursor-v4-spec.yaml:253-270`).
- The recorded addendum identity is correct: SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`. The
  accepted independent re-review identity is correct:
  `8fdc7dfea99cf9e16985e33ec25db894b524cfa5fdbbcddd2a76df42d3e9753e`; its
  root-grant identity is correctly recorded as
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
  Those records approve the design grant only and leave implementation and
  runtime proof pending.

## Scope and material-difference review

The CW-37 addition says future C2 integration must validate every event row
before filtering, ACK/global-frontier advancement, or frame emission, including
rows later filtered out; a malformed candidate page fails without ACK. This
matches the ACK-boundary addendum, which assigns the checks to the
server/caller, prevents filtered-out rows from hiding invalid payloads, and
keeps lookahead outside the acknowledged prefix. It does not claim that current
`get_range` already satisfies C2 or close the debt.

The new status correctly records independently approved bounded ledger-reader
design and retains the earlier policy approvals and repaired normative
amendments, while implementation, server integration, and runtime proof remain
pending. The next step reflects the grant: implement only the approved ledger
slice, with server validation held for separately authorized integration. The
historical proposal/recon evidence and prior resource-limit facts remain, and
no other debt entry is included in the diff. The heading still says “need
decisions,” but the updated status unambiguously records that the design
decisions are approved; this wording does not alter the entry's substantive
status or create a blocking discrepancy.

The builder receipt correctly describes the observed gap, its authority links,
and the resulting `.agents/DEBT.md` digest; the recorded file digest matches
the reviewed worktree content. It also states that no historic debt was
removed or reclassified. The source digest in the receipt is consistent with
the current `events.rs` content.

This is a documentation-only review. No builder implementation or pending
ledger tests were reviewed, and no source or runtime claim is made. Graft was
queried first for the projection/validation path and ACK/frontier requirements;
it saved approximately 8,690 tokens in one retrieval call.

# T09 proposal repair: per-line limits and recorded approval

## Original rejection and scope

The fresh independent `t08_final_static` critic rejected the latest proposal because it claimed
an eight-read cohort had a 256 MiB cumulative response bound while omitting the separate
`run.list` selection request and the Go client's existing inspect retry. A per-line ceiling
does not establish a cumulative cohort wire-byte ceiling. The verdict was relayed in the
2026-09-30 builder handoff; no separate durable verdict file was present under the T08/T09
evidence paths at the time of this repair.

The specific stale numeric claim is preserved in the earlier proposal-review record at
`t09-reconnect-budget-proposal-review.md` (section “Revised proposal limits,” bullet “Client
buffer bound”). This repair edits only
`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md` and adds this note.
No source, client, contract, normative spec, status file, dependency, or implementation was
changed.

## Material proposal changes

- Removed the claim that eight `run.get` responses imply at most 256 MiB cumulative response
  bytes, and removed the derived 64 MiB raw-line total. The proposal now distinguishes one
  `run.list` selection from up to eight logical `run.get` calls per connection cohort.
- Stated that each logical inspect call retains the current one fresh-connection transport
  retry. The proposed 32 MiB ceiling is per response line and applies to each attempt; it is
  not multiplied into a cohort-wide or total-wire-byte claim. The independent critic's concern
  about omitted retry traffic is addressed explicitly.
- Preserved the proposed limit of two concurrent `run.get` reads and the separate 1 MiB cap
  on projected initial hydration metadata. The proposal says this does not bound `run.list`
  enumeration, aggregate kernel file reads, cumulative retries, or later SSE poll/maintenance
  traffic. Rust's per-journal 64 MiB cap is not described as an aggregate `run.get` bound.
- Clarified that the 32 MiB line cap is approved as a design proposal but is not implemented;
  the existing client still uses `ReadBytes('\n')` without a response-line cap. The proposal
  does not claim bounded client allocation until implementation.
- Updated the proposal status and operator-decision section to record the operator's explicit
  approval of the amendment and scoped proposal. Independent proposal review remains pending;
  implementation and normative updates remain out of scope for this repair. Removed stale
  “approval pending” wording without reopening the operator decision.

## Source evidence checked

- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go:175-213` currently reads a
  response with `bufio.Reader.ReadBytes('\n')`; no incremental response-line cap is implemented.
- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go:315-380` documents and implements
  a single fresh-connection retry for inspect transport failures; correlated mutations use
  request-status recovery and are not re-sent.
- The proposal's existing Rust source anchors describe `run.list` as unpaginated and globally
  enumerating directories before applying its optional case filter; `run.get` can assemble data
  from multiple individually bounded journals. These facts prevent a client line cap from
  being treated as an upstream work bound.
- The existing proposal-review note `t09-reconnect-budget-proposal-review.md:21-25` records
  the cohort read budget, hydration envelope, and earlier cumulative-byte arithmetic. The
  fresh rejection called out the omitted list/retry operations and rejects that arithmetic.

## Approval and remaining work

The operator approved the contract amendment, response cap, bounded hydration/cache design, and
Ask proposal semantics, as recorded in `.agents/CURRENT_STATUS.md` and
`.agents/current_status.yml`. This note does not claim independent approval or implementation.
The fresh independent proposal review must still confirm the corrected scope; only then can the
proposal move toward implementation after T07/T08 settlement. The 32 MiB line cap, two-read
concurrency, eight logical hydration reads, 1 MiB projected metadata envelope, and Ask settings
remain design limits until implemented and verified.

No tests, build, compiler, vet, Bun, browser, or runtime commands were run. No operator approval
was requested again.

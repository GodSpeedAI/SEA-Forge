# Manager revision 6 response-line cap source recon

Date: 2026-10-06. Read-only source/document recon; no source, test, proposal,
status, or Git file was changed. No compiler or test command was run.

## Original bounded assignment

> Read-only narrow recon no implementation/heavy/Git: manager6 references
> 'separate approved32MiB source-line cap unimplemented prerequisite'. Find
> original safe-trace-port assignment/rootproposal/approval anchors exactcap32
> wording, actual SFWPclient inboundresponse reader, existing linecap/maxframe
> behavior/tests. Determine precisely missing behavior and smallest bounded
> test/source change usingexistingerrorclasses no publicschema/interfaces/
> physicalgate/cooldown/cancel changes. Critically reconcile kernel64MiB journal
> read cap vs32MiB transportline/safeDTO cap; don't conflate. Graftfirst,
> directsourceverify. Write NEW source recon originaltask/existing behavior/
> approvedcontractdiff and actionable builderbounded spec/testmatrix. No source
> changes/testcompile, rootpush5909soleheavy.

## Finding

Revision 6's statement that the “separate approved 32 MiB source-line cap
remains an unimplemented prerequisite” (`run-observation-manager-concrete-
proposal-revision6-oct06.md:55-56, 593-594`) is incorrect if “source-line” means
the approved SFWP inbound response-line limit. That exact cap is implemented in
the Go SFWP client and covered by frozen boundary, overflow, retry/recovery, and
subscription tests. The accepted cap assignment required it; the independent
cap review approved that bounded unit. There is no remaining Go response-line
implementation task established by this record.

Do not infer a 32 MiB cap on each server-side `trace.jsonl` record from that
contract. The Go cap is one serialized SFWP response line, including its LF.
The kernel journal cap is a different limit over the complete file read before
JSONL parsing. The approved run-observation contract explicitly leaves
aggregate kernel reads/serialization outside the response-line guarantee.
Therefore the 64 MiB journal cap does not satisfy or violate the 32 MiB
transport-line cap; the two bound different byte sequences and different
layers. No source change to the kernel is justified by the cited approval.

## Original contract and approval anchors

- `cap-implementation-assignment.md:14-26` is the source implementation
  assignment: incremental cap before decode for RPC/retry/status recovery and
  subscription; default 32 MiB including LF; positive lower limits; invalid
  limits rejected as config before dial; overflow typed unavailable and
  connection poisoned; no changes to retry/recovery semantics; no aggregate
  kernel-work or cumulative-allocation claim.
- `cap-implementation-independent-review.md:1-19` approves the frozen cap
  implementation for that bounded client unit. It records exact/over boundary,
  poison, partial EOF, retry, correlated mutation, subscription and coalesced
  line behavior. Its wider package/module gate limitations do not retract the
  cap-unit approval.
- `canonical-implementation-assignment.md:25-35` records the already approved
  32 MiB incremental response-line prerequisite and expressly excludes
  aggregate wire bytes, upstream run-directory enumeration and kernel journal
  work.
- `observation-cohort-runlist-recon-correction-oct05.md:1-34` retracts the
  earlier stale claim that the response cap was unimplemented and points to
  current Go source and tests. `observation-cohort-runlist-response-cap-
  correction-oct05.md` is an additional append-only correction with the same
  source distinction.
- The earlier `t09-reconnect-budget-proposal-review.md:8-31` is historical
  proposal-stage evidence: it correctly described a cap as proposed before
  implementation/approval. Later cap assignment and independent approval
  supersede that pending status. It must not be used as the current status.

## Direct source evidence

- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go:32,35-40,71-80`
  defines the 32 MiB default, documents that the limit includes LF, and
  validates the configurable ceiling before dialing. `client.go:252-276`
  reads the response with `readBoundedLine` before response decoding; overflow
  is typed unavailable and the caller closes the connection. `client.go:279-299`
  incrementally checks each `ReadSlice('\n')` fragment against remaining
  logical capacity. Its comment accurately discloses bounded `bufio.Reader`
  read-ahead; it does not claim a strict socket-byte or heap-allocation cap.
  `subscribe.go:121-140` uses this reader for subscription lines.
- `response_limit_test.go:20-21,45-155,185-227,249-338` includes the 32 MiB
  constant, configuration and lower-limit tests, an exact 32 MiB fragmented
  response acceptance case, one-byte-over rejection and connection poison,
  truncated EOF, retry on a fresh connection, and correlated mutation/status
  recovery without resending the mutation. The subscription suite checks
  over-limit event rejection before delivery; its large fixture is valid JSON.
  The independent review records the remaining focused cap assertions and
  current limitations.
- `crates/sea-forge-server/src/sfwp/mod.rs:50-60` defines 4 MiB per record and
  64 MiB per append-only journal. `run_views.rs:422-435` applies
  `MAX_JOURNAL_BYTES` to the whole file with a metadata check, reads that file,
  then iterates `text.lines()` and parses a well-formed prefix. There is no
  32 MiB per-JSONL-row check in this helper. Trace reads use it at
  `run_views.rs:793` and `:842`; evidence and declarations use the same helper.
  `get` projects the trace into its response at `:842-857,947-963`.
- The safe trace projection's 1,024-frame retention is separate again:
  `run_trace.go:20,67-177` retains at most 1,024 selected safe DTO frames only
  after `Client.Do` returned and decoding/validation traversed the response.
  It does not cap the inbound raw response or server journal. The response
  reader's 32 MiB cap runs before this projection; accepted responses can still
  incur decode expansion and frame validation work.

## Boundary and discrepancy table

| Limit | Layer and measured object | Current evidence | Status |
|---|---|---|---|
| 32 MiB including LF | Go SFWP inbound response line | `readBoundedLine`; exact/over tests | Implemented and independently approved |
| 64 MiB | Whole kernel JSONL journal file, pre-read metadata size | `MAX_JOURNAL_BYTES`; `read_jsonl` | Existing kernel limit; not a transport limit |
| 1,024 frames | Safe Go DTO retained selected frames | `run_trace.go` post-decode selection | Projection retention only; not input-size bound |

An individual 64 MiB `trace.jsonl` file may be read and projected by Rust, while
the resulting serialized run.get response may exceed the Go client's 32 MiB
line ceiling and then be rejected as unavailable. Conversely a journal under
64 MiB can serialize to a response under or over 32 MiB depending on record
content, JSON escaping, wrapper/projection fields, and other source files.
Neither cap implies the other. Existing transport behavior safely fails closed
for an oversized inbound line; this is not evidence that kernel work or
response construction stayed below 32 MiB.

## Smallest correction / bounded builder disposition

The bounded correction is documentary: revise the current manager proposal's
two stale sentences to state that the already approved Go 32 MiB per-response
line ceiling is implemented and remains an input/output boundary, not a bound
on Rust reads, serialization, decoded heap, or aggregate work. Preserve the
proposal's own “no aggregate … kernel work” limit. Do not create a Rust line
cap, change the 64 MiB journal cap, or change Go client source/tests under this
assignment: no approved requirement makes JSONL source rows individually 32
MiB, and the client contract is already implemented and tested. This is a
finding/recommendation only; this recon did not edit revision 6 or any source.

No implementation builder or new runtime test is warranted for the approved
32 MiB transport-line contract because the exact boundary/overflow behavior
already has focused coverage. If a later operator explicitly approves a
separate kernel per-record byte ceiling, that is a distinct policy/source task
requiring its own owner decision on affected readers and failure semantics.
Only then should a focused kernel fixture cover: exact row boundary, one-byte
over row, multi-row journal under the existing whole-file ceiling, whole file
above 64 MiB, malformed/truncated tail behavior, and `run.get` disclosure of
the existing `Unreadable`/unavailable result without inventing a public error
class. That future work must prove selected trace behavior without silently
changing `case_views`/evidence/declaration semantics. It is not authorized or
assigned here.

There are no deviations from the read-only assignment. No code, tests,
configuration, public schema/interface, retry, physical admission,
cooldown/cancellation, journal cap, compiler, heavy job, or Git state was
changed or exercised.

## Source hashes at inspection

```text
client.go                                      ad3d599224527beda603a320b1b86faa82b80c75480d914cd805fa1e4de3a857
response_limit_test.go                         dfb98f494880faf946ceaf3d0d10a139a20ed3260647112dd08dfc6372ccaa88
run_views.rs                                   52dd5abf1da9a2a6bc65c7d0a19ef3c16c28ab13b738240b9e63c932c505a1ca
sfwp/mod.rs                                    cb6780d752f931d9d25dbfb0438ce4f3a2c11e7ecc3ad6d4384f1755377e037d
cap-implementation-assignment.md              19481f4666463e4f7c3bdd7b0d9cf9d5c32d32c79ed957f8d42c92f88f41e591
run-observation-manager-concrete-proposal-revision6-oct06.md
                                               60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b
```

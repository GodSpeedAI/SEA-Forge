# Root causes behind repeated review findings (2026-11-17)

**Why this exists:** Codex blind review ran 6 rounds on Cognate Workbench T3 (PR #39) and 4 on SEA-Forge WP-21B. Every round
reproduced new defects, and each fix closed one instance while the next round found a sibling. The owner directed:
"consider root causes and solutions that solve whole classes of problems (such that warrant minimal design changes are acceptable)."

Below, each recurring finding is mapped to the root cause that produced it, and each root cause gets a class-level fix. The same
analysis applies to future tranches.

## Cross-cutting root cause: verification by example, not by invariant

Both workstreams were verified by listing cases: a reviewer found an input, a test was written for that input, and the next
reviewer found the next input. Nothing stated the invariants, and nothing generated inputs against them.

**Class fix:** write each safety property as an invariant, then test it with generated inputs: property-based tests and stateful
model tests.
- Workbench: `fast-check`, a dev dependency.
- SEA-Forge: `proptest`, a dev dependency.

Blind reviews then target invariants ("break invariant I3"), not features. A generated counterexample becomes a permanent
regression seed.

## Cognate Workbench T3 (MCP client, governed spawn)

| Recurring findings | Root cause | Class fix |
|---|---|---|
| stderr, then handshake error text, then error digits, then tool names, then oversized names persisted | Records accept arbitrary strings, and each code path decides what is safe | **Typed records.** In the MCP/spawn paths, record fields are a closed type: enum codes, numbers, a branded `Sha256`, and branded ids we mint. There is no `string` field, so server-originated text cannot be persisted, by construction (the compiler enforces it). |
| ReDoS, then `$ref` blow-up, then `uniqueItems`, then the size check skipped by result shape, then lost envelope validation, then deep nesting throwing | Work happens on untrusted bytes in several places before bounding: the SDK, ajv, the error path, stringify | **One ingress boundary.** Raw bytes are bounded at the stream, then a depth- and size-limited parse, then a strict envelope, giving a typed, bounded value. Nothing downstream, the SDK included, sees raw server data. Validation uses only allowlisted keywords. |
| Errors told apart by text, then forged markers, then timeouts mislabelled by elapsed time, then id attribution by heuristic | Facts inferred from untrusted content | **Facts come only from our own state** (per-call state machine, our own timers, connection state). An oversize or malformed message is a connection-level protocol violation; we never attribute it to a request. |
| Failure paths throwing, records missing | Error handling can itself fail | **Total functions on the failure path.** Every call ends in exactly one finished record, written through a writer that cannot throw. |
| Slots freed while processes survived, then reused pgids, then unverifiable groups | "Is it ours, is it dead?" can't be answered reliably without containment | **Fail closed.** A slot is released only on positive proof that nothing survives; unknown means reserved. Residual (documented): an uncontained child that leaves its session and clears its marker escapes tracking. Optional, stronger and Linux-only: a transient user cgroup per spawn (`systemd-run --user --scope`, which works on this host), killed with `cgroup.kill`. That is a follow-up, not T3 scope. |

**Invariants for property tests** (a generated adversarial MCP server sends random bytes, split chunks, no newlines, nested JSON,
huge messages, forged errors, delays and early exits):
- **I1:** no record or log contains any substring (16 bytes or more) of server-originated bytes.
- **I2:** every call ends in exactly one finished record within deadline + ε, and the client never throws.
- **I3:** buffered bytes per connection stay at or under the cap.
- **I4:** pending state is empty after every call.
- **I5:** the slot is released iff no member of the spawn group is alive.

## SEA-Forge WP-21B (Field Case records)

| Recurring findings | Root cause | Class fix |
|---|---|---|
| Unlocked verbs, then CLI paths, then pre-admission writes of case.json/plan.json, then delegation released the lock | Case state spans many files, written by many code paths, with checks placed per verb | **One transaction API.** Every write under a case dir requires a `WriteToken` from `admit_write`. Low-level writers take `&WriteToken`, so an unadmitted write fails to compile. |
| Gate bypass by automatic milestones, then by human completion, then R11 on terminate, then write-then-refuse on completion, then repeat completion | Rules are written per event type and checked event by event | **Decide on the post-state.** A pure `decide(verified_state, proposed_batch)` applies the whole batch (verb plus cascades) to the folded state, then checks invariants on the RESULT: terminal implies a disposition; was closed implies now finished; a completed gate item implies an assessment; and so on. A new event type is covered automatically. The batch is written all or nothing. |
| Corrupt plan made the case look non-Field, then records checked only for parsing, then shallow document checks | State loading tolerates partial failure | **All-or-nothing load.** The verified state is built from every store. Any parse, chain or schema failure is STORE_INTEGRITY, and Field-ness is decided by any marker, never by a parseable plan. |

**Invariants for stateful model tests** (proptest generates random sequences of every verb, CLI and server, including corruption
injections and suspension):
- **C1:** no terminal state without a case-level close disposition.
- **C2:** after a close disposition, the only accepted write finishes the plan.
- **C3:** no gate item or milestone completes without a passing assessment.
- **C4:** a corrupt store accepts no write.
- **C5:** a refused verb changes no byte under the case dir.
- **C6:** a suspended case accepts only the contract's permitted writes.

## Process change
0. **Three-round rule** (owner, 2026-11-17): a review/fix loop gets at most three rounds of instance fixes. After that, read what the
   findings so far have in common, name the problem classes they belong to, and fix those classes before anything else. Here that
   would have meant stopping after T3 round 3 and WP-21B round 3, not round 6 and round 4.
1. Reviews ask the reviewer to break the named invariants, with generated or hand inputs. Feature checklists are retired.
2. A fix is accepted only when the invariant's property test would have caught the finding (red first), not only a single example.
3. Merge when the property suites pass at a meaningful case count (≥ 1,000 per invariant in CI; more locally) and one invariant-targeted review finds no violation.

## WP-21B round 5: the remaining classes (independent review of 9f9dbe85)

Holds: C1 and C3. Violated: C2 (bookkeeping writes and a non-terminal case.json on closed cases), C4 (corrupt event lines silently
dropped, unverified stores, no schema check in the CLI, a case not detected when markers are deleted), C5 (a refusal writes a deny
decision and the lock file), and C7 (the grep test is evadable, and a token permits any write in the dir). One product defect:
after `close-failed` on an unfinished plan, terminate is refused forever, so the operator cannot stop the case.

| Root cause | Class fix (minimal design change) |
|---|---|
| The guard judges a write by its path or kind NAME, separately from the bytes written; the token proves only that some admission happened | **Admit-and-apply.** `commit(case_dir, Batch)` takes the lock, runs `load_verified`, runs `decide` on the post-state, writes the exact batch, and unlocks. The guard performs all I/O; callers build a typed `Batch` and never write. No standalone token exists. |
| Lifecycle rules are hand-coded per category, with exempt "bookkeeping" | **Rules come from the contract's registry.** The per-kind table (README section on record kinds; `fixtures/case-record-kinds/valid/registry.json`: what suspended and closed cases accept) is loaded as data. Every file a batch touches belongs to a registry kind or to a declared projection, which is rebuilt from the ledger inside the same commit. Anything else is refused. No exemptions. |
| Ending actions are conflated with progress | **C2 restated from the contract:** after a close disposition, only plan-ending actions are admitted, either finishing the plan or `CaseTerminated`, plus the kinds the registry marks as accepted when closed (e.g. late `episode.settled` evidence). |
| The loader tolerates partial failure; the schema validator lives only in the server | **Strict load.** Every line of every store parses and schema-validates, else STORE_INTEGRITY. The case-runner crate owns the validator (schemas compiled from `contracts/fde` via `include_str!`), shared by the CLI and the server. A case is a Field Case if ANY `field*` artifact exists. |
| Writer coverage is verified by grepping source | **Observe writes at runtime.** The model test runs every verb under a filesystem observer (inotify, or a test-only I/O shim in `commit`) and asserts that observed writes under the case dir equal the committed batch writes. C5 compares tree hashes. Lock files live outside the case dir. If the contract requires a refusal record, the refusal is its own committed batch of a declared kind. |

## T3 round 7: proxies treated as guarantees (Codex invariant review of 4c11d46)

I5 holds. I1–I4 are violated. Codex named the shared root cause: **local surrogates (brands, counters, wrapper maps, attempted
writes) are treated as end-to-end guarantees.** The general principle for every invariant here and in later work: **measure the
guarantee at the place it is consumed, and make the code commit on that same ground truth.**

| Invariant | Surrogate that failed | Ground truth to enforce and test against |
|---|---|---|
| I1 no server text in records | the `Trusted` brand, cast (`as Trusted`) from server-derived keyword names | Brands are constructible only inside `typedRecord.ts`, from compile-time literal unions (our own keyword and code tables). A test fails on any `as Trusted`, `as Sha256` or `as MintedId` cast outside that module. |
| I2 exactly one finished record | an attempted record write | **The durable finished record is the commit point.** A call's result is delivered only after its finished record is confirmed in the store. If recording throws, rejects or stalls past the deadline, the outcome is `record_failed` and the result is withheld (fail closed). Each record has a minted id, and the store deduplicates by that id, so a retry or fallback can never produce two. The test counts records in the STORE. |
| I3 bounded buffering | a counter over the partial line | **Retained bytes = partial line + parsed queue + anything else held.** At the cap, stop reading the pipe (`stream.pause()`, i.e. backpressure) until the consumer drains. The test measures the queue's actual size. |
| I4 no pending state | the wrapper's map | **Any transport fault (EPIPE, write error, close of either pipe) closes the whole connection,** which rejects every SDK handler. The test asserts on the SDK's own pending-handler count. |
| Ingress | server request ids serialized outside ingress | Ingress validates and types request ids (a number, or a string of ≤ 64 restricted chars). Server-initiated requests we do not support are answered by ingress with a fixed error. |

Generator gaps to close: batching, EPIPE and half-closed pipes, recorder throw, rejection and stall, real process-group
membership, plus malformed results and protocol violations. Oracles read ground truth (the store, SDK state, queue size, process
table), never the code's own counters.

## WP-21B round 6 (Codex review of 81a99fd0): the remaining classes

Every invariant was violated again, but the findings fall into five classes:

| Findings | Root cause | Class fix |
|---|---|---|
| C1, C2, C3: `RunFile` with `../plan.json` or `../case-events.jsonl` writes an arbitrary path and arbitrary bytes; `decide` judges an assumed effect, not the real one | `Batch` has an untyped raw-file op, and `decide` predicts the post-state instead of reading it | **Verify the real post-state.** Stage the batch in a copy (or overlay) of the case tree, run the strict `load_verified` and every invariant on the STAGED tree, then swap it in atomically; a failure leaves the live tree untouched. Raw-file ops are path-contained (no `..`, no absolute paths, no symlinks) and cannot target store files; store files change only through typed record ops. |
| C4: a torn trailing fragment is silently repaired, then the write proceeds | The loader tolerates a malformed tail | **Strict load, tail included.** Any byte after the last complete newline-terminated record is STORE_INTEGRITY; repair is an explicit operator verb, never implicit. |
| C5: the self-settlement refusal writes a deny decision and an outbox entry | The contract (AT-05, interaction map 3.2b) REQUIRES the deny decision and the interaction event | **C5 restated from the contract:** a refused verb changes no bytes except its declared refusal record, committed as its own batch of a declared refusal kind through `commit`. This is not a weakening: the contract requires that record. |
| C6, and the server's `is_field_case` returning false when plan.json is missing | Field-ness is decided in several places, by different rules | **One classifier, with durable authority.** `.case-index/<id>.json` `field: true` is authoritative once written; otherwise any `field*` artifact or a Field plan template counts. The server, gate, CLI and case-runner all call this one function. A Field case with plan.json missing is STORE_INTEGRITY. |
| C7: sandbox materialization writes into `cases/<id>/runs/<run>/workspace` | Scratch output lives inside the case store | **Move run workspaces outside the case dir** (e.g. `<root>/.run-workspaces/<case>/<run>`), so the case dir holds only committed state. |

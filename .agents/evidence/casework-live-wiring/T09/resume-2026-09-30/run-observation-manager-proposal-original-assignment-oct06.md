# Private run observation manager — original proposal repair assignment

Documentation ONLY. This does not release implementation or public changes.
Read the operator-approved T09 extension proposal in full, governing spec and
DTO/schema, source inventory plus root inventory correction, final accepted
trace/session/physical-admission/selector prerequisites, and actual server,
projection Store/Relay, authority and trace-port source cited in the inventory.
Root owns semantic decisions and integration acceptance; Luna turns this bounded
architecture into a concrete reviewable private proposal.

Write ONE NEW immutable run-observation-manager-concrete-proposal-oct06.md in
this directory using native apply_patch. No source/test/status/debt/Git/gates/
compiler/scanner/Graft build changes. No public HTTP/SSE/cursor/kernel/interface
or identity change is approved by this assignment.

## Root decisions for the private proposal

- Preserve process-shared (case_id,run_id) keys; at most16 entries including
  initialization/draining, two physical run_get attempts and post-write cooldown
  remain owned by the accepted shared client admission hook. Manager does not
  bypass it or implement a competing physical limiter. Tick no faster than1s
  per run; no evicting an active entry to attach another run.
- Use accepted deterministic selector over complete decoded readable summaries,
  selecting at most8. One scoped run.list and at most8 initial logical trace
  calls; physical retries do not increment logical counts. Reuse only a verified
  shared current buffer for the exact identities, preserving its observation
  time. No automatic reads for omitted rows during this cohort.
- listed_run_count means readable summary rows. unreadable_run_count separately
  counts scoped case-claimed UnreadableIDs. selected is min(8,readable). validated
  counts selected candidates whose exact case/run/plan-item ownership and known
  parent are verified, including reused buffers. unavailable counts selected
  denied/malformed/mismatched/orphaned candidates. omitted counts readable rows
  outside selection plus selected rows blocked by shared capacity. Distinguish
  these categories; failed validation is unavailable, not capacity omission.
- Proposed exhaustion interpretation for review: eight actual logical initial
  calls AND additional known readable candidates not validated because the
  eight-read limit is reached sets exhausted true, including rows also excluded
  by the eight-selection bound. Cached reuse does not count as a read. Fewer
  than eight calls never sets exhausted. Cite proposal31/35/39/41/43 and schema
  causal wording; explicitly discuss the simultaneous selection/read-limit
  ambiguity. Critic/root must resolve it before any fixture implementation.
- All count pointers remain absent on failed/refused/undecodable list. Successful
  decoded zeroes are present. unavailable takes precedence when list/unreadable/
  selected validation fails; otherwise omitted>0 means capacity_limited; no
  readable or unreadable entries means no_runs; otherwise complete. Only owned
  validated candidates contribute run frames. Never infer settlement.
- Supply parent checks from a trusted latest-known PRESENT case revision and its
  captured Horizon; it adds no hydration source read. Describe this honestly as
  as-of-cursor knowledge, not continuously current kernel truth. Require a
  private caller guard to reject cold/evicted/gapped/history contexts. Do not
  implement or claim a public readiness frontier here; it belongs to held V4.
  Exact run/case/plan-item checks still apply to every fresh or cached trace.
- Each watcher retains its own verified session/perspective. Current(id), stored
  role/actor/deadline/revocation and current kernel perspective must be checked
  before disclosure and every later polling/fanout boundary. Bearer-mode behavior
  must be explicit, honest and within existing development-only constraints.
  Sharing a buffer never grants access. Do not let one watcher's departure cancel
  a read still needed by another authorized watcher.
- No callbacks/network waits under a manager lock. Last watcher, terminal run,
  revocation/expiry of all watchers, or shutdown cancels and JOINS any pending
  trace call before deleting entry/releasing slot. Entry remains counted while
  draining. Adopt actual accepted transport callback/pool-retirement guarantee;
  do not claim SessionStore.Current alone drains reads. Define race-safe attach,
  concurrent initialization sharing, delivery, detach and shutdown results.
- Preserve1024 retained safe frames/run and accurate totals/truncation. Initial
  payload JSON (not SSE framing) <=1MiB; omit globally oldest parsed timestamp
  frames, ties by complete RunID then EventID bytes, while retaining per-run
  order and updating retained/omitted/truncated fields exactly. If metadata alone
  exceeds cap, fail with typed unavailable and emit no oversized payload; no
  fabricated counts or silently dropped required run metadata. The irreducible
  failure treatment is a private fail-closed proposal, requiring review.
- Later polls deliver only new frame IDs while those IDs remain in a bounded
 1024-frame retained window; do not promise unbounded exactly-once behavior.

## Required proposal result

Give exact private constructor/attach/lease/stop interfaces and types, state
machine, lock/context ownership, lifecycle and caller preconditions, explicit
count equations and table of outcomes. Show reuse/authorization races, current
parent guard, initialization single-flight, capacity and terminal cleanup. Give
bounded test-first decomposition into small disjoint files, fixtures distinguishing
correct behavior from each failure, actual RED before implementation, independent
critics and serial required gates. Explain every material difference from this
assignment or approved proposal with source citations. Identify assumptions or
unsatisfied prerequisites rather than papering them over. Public SSE/frontier
wiring stays held, regardless of private proposal approval.

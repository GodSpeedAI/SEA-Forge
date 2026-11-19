# Implementation readiness (EGM Goldfish) — revision 2

Reviewer mode: REVIEW. Scope: revision 2 of `design.md`, `D`, `F`, `E`, `B`, `G`, read against
`egm-review/design-critic.md` (rev 1) and against the cited code. Baselines inspected:
cognate `29dcd1e`, cognate-workbench `f60acb7` (+ `feat/capability-runtime-t4a` `bbe6c73`,
`feat/wp23-assignment-binding` `668f686`), Horizon `0e34a34`, gauntlet `6a7fe6a`. No build or
test was run; every claim below is a reading of the named file:line.

## Verdict

**Not ready to implement.** Revision 2 genuinely fixed the rev-1 blockers that mattered: there is
no `native` provider, authority/depth/budget are honestly caller-attested, `StdioSpawnPort` is a
declared port rather than an extraction, `assertedBy` is recorded not enforced, J13 no longer
pretends the refusal is macOS-only, and the Gauntlet vocabulary is real. Twelve MUST-FIX items
remain, and they cluster in one place: **the seams between the documents are still unbuilt.**
`E` was not regenerated with `D` (it still asserts the abolished `native` provider and still
carries the truncated J6 that rev 1 flagged); the rev-2 `unknown`/`reconcile` story duplicates a
reconciliation gate Cognate already has; the parent's resumption path after `rlm.invoke` is
nowhere; the six-value `terminalState` cannot be derived from the exit code the contract calls
authoritative; and no increment owns the Cognate *release* that I2/I3/I7 need, because both
consumers read `@godspeedai/cognate-*` from the registry, not the workspace.

Individually each is small. Together they mean an implementing agent would make six design
decisions this package claims to have made (resume mechanism, invocation store, provider
registration, refusal-code mapping, Gauntlet parse rule, `caseContext` shape) — the exact failure
mode this reviewer exists to catch.

---

## MUST-FIX

### 1. `E` was not updated with `D`: it asserts the abolished provider and is still truncated

*Severity: Blocker. Affected: `E:13-14` (J1), `E:75-81` (J6), `E:186` (orphan), J6/J14 order.*

* `J1` **Pass** reads "`rlm.providers` returns exactly `native`" (`E:14`). Revision 2's central
  correction is that no `native` provider exists (`D0:11`, `D1:51-52`, `F:149`), and `J4` in the
  same file says it "returns an empty array" (`E:48`). Two journeys assert opposite pass criteria
  for the same call.
* `J6` still stops mid-sentence — "no Gauntlet process was spawned (assert via the spawn"
  (`E:80-81`) — and its Negative/Trace survive only as an orphan line after `J14`
  ("ledger being empty); the completion path is J1's path.", `E:186`). `J14` sits between them.
  This is verbatim rev-1 minor 4, unfixed.
* Consequence: a separate agent cannot execute J6 at all, and J1 fails for the correct
  implementation. `design.md:71` calls J1 one of the four journeys that "decide whether this
  design is real".

*Fix:* regenerate `E` from `D` (drop `native`; complete J6; restore order) and add a consistency
gate: every capability id and refusal code named in `E` must exist in `D5`, and no journey may
name a provider `D1` says cannot exist.

### 2. The parent has no way to resume after `rlm.invoke`

*Severity: Blocker. Affected: `D1:79`, `D4:180-183`, `F:52-61` (I2), `E:45-61` (J4), `E:128-140`.*

`rlm.invoke` returns `{ status: "admitted" }` and creates a child run. Nothing in `D`, `F` or `E`
says how the invoked Expert learns that the child finished. What exists at `f60acb7`:

* `agentCoding.ts:806-812` — the **only** blocking wait, `ctx.waitFor("expert-assignment:<id>")`,
  resolved by `bridge.ts:93-104`, which scans `state.assignments` for a finished *Workbench
  Assignment*.
* `subagent` is explicitly fire-and-forget, "There is no `ctx.waitFor` here"
  (`agentCoding.ts:822-825`); `B:18-19` says `subagent.start` is only fire-and-forget startup.
* No component opens or resolves a plain run-completion continuation
  (`grep 'resolveContinuation('` returns only `approve:*`, `plan-review:*`,
  `ambiguous-invocation` and the bridge).

So I2 needs a **new continuation type plus resolver**, and the design forbids the only alternative
("no new polling loop", `design.md:141`). That is a design decision, not an implementation detail.
*Fix:* name it in `D4` (type, key — e.g. `rlm-invocation:<invocationId>` — opening site, resolver
trigger, idempotency key, resume payload) and put the resolver in `F I2`'s boundary.
*Test:* J4's "both runs produce a handoff with evidence" and J11's "after `reconcile(completed)`
the assignment resumes" both hang if this is left to the implementer.

### 3. Invocation state has no home, contradicting "the contract never owns state"

*Severity: Blocker. Affected: `D1:87-111`, `D4:198-201`, `B:25-26` (G1 risk), `B:334-349`, J11.*

`rlm.inspect`, `rlm.cancel` and `rlm.reconcile` are keyed by `invocationId`; J11 requires
`rlm.inspect` to answer `unknown` **after a restart**, i.e. across a process boundary with no
in-memory state. Yet `B:25-26` says "the contract returns run ids and never owns state". Either
that sentence is wrong or restart-survivable inspect is impossible as specified. Nothing names a
collection, a record shape, or a lookup. Both options have precedent:
`runtime-bun/src/service.ts:143-148` writes a per-assignment record with `expectedVersion: 0`
(CAS), and `startRun` already records `idempotencyKey`, so `invocationId → runId` could resolve
through the run record.

*Fix:* choose one in `D4`: (a) a durable invocation record (name the collection, CAS guard and
writer), or (b) resolution via the recorded `idempotencyKey = "rlm:" + invocationId` with no
second store — then qualify or delete `B:25-26`. *Test:* J11's restart leg.

### 4. `unknown` + `rlm.reconcile` duplicates a reconciliation gate Cognate already has

*Severity: Blocker. Affected: `B:334-349` (G18 "Current"), `D1:107-111`, `D4:200-201`, J11.*

`B:336-337` states "Cognate has no generic 'external invocation outcome unknown' state". At
`29dcd1e` it does:

* `runtime-bun/src/executor.ts:186-205` — on recovery with an unrecorded invocation it appends
  `run.invocation.ambiguous`, then `context.waitFor("ambiguous:<stepId>", { type:
  "ambiguous-invocation" })` instead of re-performing the effect.
* `runtime-bun/src/index.ts:87-96` + `runtime.ts:184` — `execution.runOwnership.ambiguousSteps:
  "reexecute" | "wait"`, documented as "record an ambiguity and require explicit reconciliation
  before the external effect may repeat".
* Workbench already exposes the operator door: `src/capabilities/ambiguousStep.ts:1-40`, capability
  `run.invocation.reconcile`, operator-only, resolving that continuation.

So rev 2 adds a *second* reconciliation vocabulary (`unknown`, `rlm.reconcile`,
`rlm-reconcile-required`) beside a working one — the "fourth mechanism" the design refuses — and
misses a load-bearing detail: `ambiguousSteps` defaults to **`"reexecute"`** (`runtime.ts:184`),
i.e. the blind retry J11 forbids. No increment changes it in `src/server.ts`'s `createRuntime`
options (`server.ts:720`).

*Fix:* either EXTEND `run.invocation.ambiguous`/`run.invocation.reconcile` to carry
`invocationId`/`providerId`/digest (and drop `rlm.reconcile`), or state why the existing gate
cannot carry an RLM invocation and label the duplicate in `B4`. Either way add
`ambiguousSteps: "wait"` to the `F I2` boundary and file list, and correct G18's "Current" line.


### 5. No increment owns the Cognate **release** that I2/I3/I7 depend on

*Severity: Blocker. Affected: `F:19-29` (I0), `F:49-61` (I2), `F:63-77` (I3), `F:126-136` (I7),
`D6:239-240`.*

Both consumers depend on published packages, not the workspace:
`cognate-workbench/package.json:19-38` pins `@godspeedai/cognate-*` at `0.9.2`,
`Horizon/package.json:20-29` at `0.10.0`, `bun.lock` carries registry tarball entries, and the
installed trees resolve `0.8.0`. I1 adds `runtime-api/src/rlm.ts` and `runtime-bun/src/rlm.ts`
**in the cognate repo**; I3 adds `StdioSpawnPort` to the same published package; I7 needs
`worldCorroboration` on `CaseContext`.

I0 aligns both apps to *an existing* release and explicitly forbids the only workaround ("do not
paper over it with path aliases", `F:28-29`). That release cannot contain I1's code, which does
not exist yet. So I2's stated prior state ("I0, I1", `F:51`) is unreachable: Workbench would
import `rlm` types the installed `0.9.2` does not export. `D6:239-240` ("one version line for
Workbench and Horizon (I0)") cannot hold across two increments that each add to that line.

*Fix:* insert an explicit cut-and-publish step (Cognate version bump + both lockfiles) between I1
and I2, and again after I3 and after any `CaseContext` change; or state that I1/I3 are developed
against a `bun link`ed workspace removed before handoff. Name the version and the harness that
proves the resolved version contains `rlm`.

### 6. `rlm-budget-unverifiable` is unreachable, and `budget` has no reservation to consult

*Severity: Blocker. Affected: `D1:71`, `D4:194-197`, `D5:221`, `F:43-44`.*

Three statements, mutually exclusive:

* `D5:221` — `rlm-budget-unverifiable`: "a numeric limit was supplied without a reservation the
  seam can consult" (a **refusal**).
* `D4:194-196` — "A numeric limit is enforced only when the caller supplies a reservation the seam
  can consult; otherwise the invocation is admitted with `payment.basis:"unreserved"`" (an
  **admission**).
* `F:43-44` — the I1 test asserts the admission.

And the input cannot express it: `budget` is `{ currency, limit }` only (`D1:71`). The code can
never fire, which breaks the "every refusal code reachable" property an implementer is entitled to
rely on. *Fix:* keep the admission (delete the code from `D5`) or add the reservation reference and
the consult rule (delete the `unreserved` branch and change I1's test).

### 7. The authority rule is written twice, in two incompatible forms

*Severity: Blocker. Affected: `D1:81-85`, `D4:187-191`, `D5:219`, `E:41-42` (J3), `E:63-72` (J5).*

* `D1:82-83`: "this seam re-checks only that the request is a **subset**" → a non-subset is
  **refused**, which is what makes `rlm-authority-exceeds-parent` (`D5:219`) reachable.
* `D4:187-189`: "`tools(child) = parent.authorityTools ∩ authority.tools`; an empty intersection
  with `toolPolicy !== "bridge-only"` refuses" → a non-subset is **silently narrowed**; the code
  never fires and a request for `bash` from a parent without `bash` is admitted with `bash`
  dropped.
* `E:41-42` asserts the opposite of D4: "a child requesting `bash` when the PM lacks it is refused
  (**not silently narrowed**)".

An implementer cannot satisfy both. `D1:70`'s `authority.mode` is a third unused clause: no rule
consumes `"read-only" | "workspace-write"`.
*Fix:* state one predicate (requested ⊄ parent → `rlm-authority-exceeds-parent`; the intersection
is what the child *gets*, not a compromise the seam computes), and mark `authority.mode`
recorded-only or give it an enforcement rule.

### 8. The Gauntlet invocation cannot be executed as specified

*Severity: Blocker. Affected: `D3:168-173`, `F:120-123` (I6 verify), `E:191-205` (J7), `D6:237-238`.*

Read `gauntlet` `6a7fe6a`, `crates/gauntlet-cli/src/run_cmd.rs`:

* The argv is `gauntlet run [--dry-run] [--config <path>] "<intent>"` (`:28`, `:35-63`). The intent
  positional is **required**: with none it prints "the operator intent is required (a quoted
  phrase)" and returns exit **2** (`:64-70`). `D3`, `F I6` and J7 say only
  `gauntlet run --config <path>`. An implementer following the document gets exit 2 and no run.
* `exit_of` (`:466-471`) returns 0 **only** for `TerminalState::Settled` and 1 for every other
  terminal state; 2 is reserved for usage errors (`:36,45,52,58,69`). `D3:169` says "`exitCode`
  0/1/2 is authoritative", and J7 requires a `gauntletTerminal` "one of the six Gauntlet states".
  The exit code is binary; it cannot yield six values.
* Where the six states actually appear: `print_outcome` writes `run: run_id=…`,
  `run: terminal=<state>`, optional `run: reference_breach=…` / `run: escalated — …` (`:428-442`),
  then `print_run_report` writes `human_summary()` **and** `to_json()` to stdout (`:464-476`). So
  stdout is prefixed lines, then prose, then JSON. `D3:171` says only "`reportRef` is the archived
  stdout artifact" and gives no split rule.
* Refusals the design wants to "surface" (tampered reference, unclean preflight, unresolvable
  config) go to **stderr** (`:170-181`, `:412-421`, `:72-78`), which a stdout-only `reportRef`
  does not capture — so J7's "Workbench surfaces that refusal" has no artifact.
* Terminal spellings are snake_case wire tokens (`wire.rs:80-113`; `values.rs:262-268`:
  `settled | blocked | budget_exhausted | escalated | aborted | inconclusive`); `D3`/`E` write
  PascalCase. Pick one for the recorded field.
* Unstated constraint: "every composed path takes the state dir's WRITER role exclusively for its
  lifetime" (`run_cmd.rs:16-20`), so a second concurrent escalation against one state dir refuses
  typed. Nothing in `D3`/J7 mentions the state dir or that refusal.

*Fix:* write the exact argv including the intent (Workbench authors it from the frozen criterion —
say so), the parse rule (last `run: terminal=` line, or the report JSON's `terminal_state`), what
is archived (stdout+stderr, or the JSON only), the casing, and the writer-lock refusal.
### 9. `caseContext` is not shippable as specified — and the stated refusal is unreachable

*Severity: Blocker. Affected: `D1:72-77`, `D3:155`, `E:82-97` (J8), `F:133-135` (I7), `B:181-191`.*

Read `cognate/packages/runtime-api/src/types.ts:14-38` and `case-context.ts:11-33`, the validator
that actually runs at the service boundary:

* `role: string` is **required** and non-empty (`role` is in the `["caseId","planItemId","role"]`
  loop). `D1`'s `caseContext`, `D3`'s field list and J8's "Then the run carries …" list all omit
  it, and nothing says who supplies it.
* `worldRef` is mandatory for **both** `kind` values and must match
  `^world:[a-z][a-z0-9]*([._-][a-z0-9]+)*@sha256:[0-9a-f]{64}$`. `D3:155`, `E:93` and `F:133-134`
  all state the refusal as "`kind === "evaluated"` without `worldRef`". That branch cannot be
  reached — a context missing `worldRef` is refused for `exploratory` too, by the vendored check.
  The evaluated-only requirement is `episodeRef` (`case-context.ts:24`).
* `worldCorroboration` is **not** a `CaseContext` field. Adding it changes a published
  `runtime-api` type; no increment lists that file, `D6`'s compatibility section does not cover
  it, and `design.md:48` is silent on it. Passing the extra key from Workbench is a TypeScript
  excess-property error against the pinned package.

*Fix:* add `role` to `D1`/`D3`/J8 and say where it comes from (the SOP role of the run's
principal); restate the refusal as "evaluated without `episodeRef`, or any kind without a valid
`worldRef`"; and either add `worldCorroboration` to `CaseContext` with its file and version line
named, or record it outside the frozen context (e.g. Assignment evidence, where `assertedBy`
already lives).
*Test:* J8's negative, written as specified today, passes against a *correct* implementation for
the wrong reason and never exercises `episodeRef`.

### 10. Public capabilities and dispositions with no owning increment

*Severity: Blocker (scope integrity). Affected: `D3:159`, `B:236-284` (G12–G14), `B:299-314` (G16),
`B:93-104` (G5), `B:351-369` (B4), `F` (whole).*

`D3` declares `affordance.field` as a new public read capability with refusal conditions. No
increment in `F` implements it; `B:279-284` (G14) puts it in `src/cognitive/operations.ts` with a
new data file, and no increment touches that file. G14's cited verification — "journey 1 and the
'investigate instead of implement' negative test" — does not exist in `E`: J1 has no such negative.
Same pattern for G16's `pluginRegistry.ts` deletion and the unused `cognate-profile-harness`
dependency, and for G5's `caseAssignmentId` rename (`E:91` asserts it as a projection field; no
increment adds it).

`B4` also contradicts `F` on two rows: it lists "app-local `execSpawn`" under **DELETE**
(`B:361`) while `F I3` keeps `execSpawn.ts` as the port implementation and says "No store moves
into Cognate" (`F:67-69`); and it claims four duplicated mechanisms removed while three of the
remaining removals are unowned.

*Fix:* assign each disposition an increment (or delete it from `B4`/`D3`), add the missing negative
test to J1, and reconcile the `execSpawn` row with I3. Until then `design.md:43` ("removes at
least four duplicated mechanisms") is not traceable to any work item.

### 11. Retry semantics contradict themselves

*Severity: Blocker. Affected: `D1:59`, `D4:180-183`, `E:134-138` (J11).*

* `D1:59` — `invocationId` is "stable across retries".
* `D4:181` — `idempotencyKey = "rlm:" + invocationId`.
* `E:135-136` — "after `reconcile(failed)` a retry uses a **new** idempotency key".
* `E:138` — "replaying a completed `invocationId` returns the same run id".

With a stable `invocationId`, a retry produces the same `idempotencyKey`, so Cognate replays the
recorded (failed) run instead of starting a new one. Either retries take a new `invocationId` (and
the old one is recorded as lineage) or J11 is wrong. Also unspecified: what `parent.attempt` is for
here, given the Assignment already carries `attempt` (`experts/types.ts:69-83`).
*Test:* J11's positive retry leg — it either replays a failed run or banks a second run under one
idempotency key, depending on which rule the implementer picked.

### 12. Provider registration is unspecified and self-contradictory

*Severity: Blocker. Affected: `D2:113-149`, `D7:242-246`, `F:63-95` (I3, I4), `E:142-153` (J12),
`E:63-72` (J5), `F:74-76` (I3 verify), `D1:26,41`.*

Four gaps in one seam:

1. `D7:244` declares `RlmProvider` **explicitly not public**, yet I4 puts an implementation of it
   in a separate package (`cognate/packages/rlm-prime/*`, `F:85-87`). A port implemented outside
   the declaring package is public by definition. Decide and say so.
2. Nothing says how a provider reaches the registry. `RuntimeOptions.components?:
   readonly Component[]` already exists (`runtime-bun/src/index.ts:57`), but `D1:26` calls provider
   ids "operator-configured" and `D7:245` makes the settings file format explicitly non-public, so
   the *operator action* that installs a provider has no home. J12's "revocation is durable across
   restart" needs one.
3. Two refusal-code sets with no mapping. `D5` has `rlm-containment-unavailable`; the wrapper the
   port delegates to returns `posture-contained`, `stream-port-unavailable`, `no-process-groups`,
   `unenforceable:<restriction>`, `network-scope-unsatisfiable`, `world-unavailable`,
   `posture-unknown` (`execSpawn.ts:188-197` and `decideContainment` at `bbe6c73`). `F:74-76` and
   `E:159-163` assert the wrapper codes; `D5` does not contain them; `D1:41` says
   `rlm.providers[].reason` comes "from a closed set" that is never enumerated. State whether
   wrapper codes pass through verbatim (then add them to `D5`) or are mapped, and enumerate
   `reason`.
4. The `unattested` refusal (`E:71-72`) appears **only** in `E`. `D4`'s ten rules do not contain
   it, so `F I4`'s "`unattested` refused" has no norm to test against.

---

## SHOULD-FIX

### S1. `D2` still names four undefined types
`D2:116-123` uses `ProviderFacts`, `InvokeRequest`, `InvocationView` and `InvocationOutcome`, and
none is defined anywhere in the package — while `D:3` says "TypeScript signatures are normative
shapes". `ProviderFacts` is load-bearing: it is the only bridge between `probe()` and the
`rlm.providers` entry. Define it as exactly that entry shape (one source), and the others as
one-liners, or drop the sketch and let `D1` be the whole contract.

### S2. Two different identities are both called `runId`
`RlmProvider.invoke` returns `{ runId: string }` (the provider's own handle, since Cognate has
already created the child run) and `rlm.invoke` returns `runId` (the Cognate child run). Same field
name, two identity spaces, in adjacent sections. Rename one (e.g. `providerHandle`) and say where
the handle is stored — this is the input to MUST-FIX 3.

### S3. `rlm.providers` has an unbounded hashing cost
`D1:39` requires `executable: {version, sha256}` per listing, `D4:198` records the digest per
outcome, and `design.md:140-141` promises "provider probing is lazy and bounded". Hashing provider
binaries on every UI render has neither bound nor cache (rev-1 minor 5, unfixed). Add a cache TTL
and say who invalidates it, or move `executable` behind an explicit probe.

### S4. The admission table is missing the axis that actually refuses
`admission` (`D1:43-48`) carries governance, posture and externalProvider. The wrapper also
requires a **per-world `PostureRecord`** authorizing weaker `network` and `subprocess` with a
non-empty note (`execSpawn.ts:76-84`); `network` is `not-enforced` on `local` and unsatisfiable in
`linux-bubblewrap` (`:53,64-66`). So `uncontained-local` alone does not admit an external
provider, while J13's negative reads as if it does. Add the fourth axis, or state that I3 must
create the posture record.

### S5. `rlm.cancel` has two targets and one is unspecified
Cancelling an invocation must stop the provider process *and* the Cognate child run. `D1:101-105`
describes neither, and `cancelled` vs `already-terminal` depends on which terminal state wins the
race. Say the order and what happens when the child completes during the cancel.

### S6. Declared inputs with no rule
`authority.mode`, `parent.assignmentId`, `parent.attempt` (`D1:63-70`) appear in no `D4` rule. Mark
each "recorded only" (honest and cheap) or delete it. Every input should be either used or
explicitly unused; three are neither.

### S7. The upgrade rule for the new payload fields lives in `B`, not `D`
`B:229-231` says old events without `settlementScope` must be read as `local`. That is a
compatibility rule and belongs in `D6`, where the reader implementer will look for it.

### S8. Persisted Workbench state changes are unaddressed
`ExpertExecution.runner`, `Assignment.caseContext`/`assertedBy` and `Assignment.caseAssignmentId`
all land in `ExpertState` (`schema: 1`, `experts/types.ts:96-104`). Say the new fields are optional
and that state written before them still reads (precedent: `reviews: string | null`, documented
"for data saved before this field"), and whether `schema` must bump. Repo policy requires asking
before a persisted-schema change, so the design must state its position.

### S9. I3 carries a refactor it does not need
"the ad-hoc stdio handling inside `src/mcp/client.ts` becomes a caller of the same port"
(`F:72-73`, `B:48`) rewrites the T3-hardened MCP path in an increment whose verification is "the
containment matrix tests still pass unchanged". Scope it out, or name the exact functions and the
tests that must change.

### S10. `experts.restore` and O5 still do not meet
`D3:161-166` is now coherent, but "verbatim baseline" (G O5 default (a)) is undefined for a deleted
baseline id whose customized text is gone, and J12 does not test delete-then-restore. Add the case
or narrow the promise.

### S11. J8's "refused with a typed code" has no code
`D5` has no case-context code and the vendored refusal is a plain string from `caseContextIssues`.
Name the code (or say the vendored message is the contract) so the negative test can assert
something.

### S12. `G` still speaks revision 1
`G:35-37` says "`requestedBy` improves attribution" (renamed `assertedBy` in rev 2) and `G:57-58`
says "the native runner is not RLM". `D0` claims the vocabulary changed; `G` did not follow.
Cheap to fix, but `G` is where an implementer reads the unverified assumptions.

---

## NOTE

1. **Baseline drift.** Local `main` is `f60acb7`; `origin/main` is `efdcc1c` (T3). Neither
   `bbe6c73` (t4a) nor `668f686` (wp23) is an ancestor of either, so "main" in this package names
   one specific local checkout. Say `origin/main` explicitly, and re-pin before I3/I7 start.
2. **One unqualified branch citation.** `D4:206-207` cites `execSpawn.ts:196` with no branch
   qualifier; that file does not exist at `f60acb7` (only `harness.ts`). `D2:142`, `B:33` and
   `E:170` qualify it correctly — do the same in `D4`.
3. **I1's "depth refusal" test is not a safety property.** Because depth is caller-attested, the
   test proves the seam's arithmetic, not a bound: nothing stops a caller passing `maxDepth: 99`.
   Keep the test; do not present it as enforcement in the increment summary.
4. **`handoffItem` already carries a `settlement_criteria` object**
   (`{required_artifacts: [], required_declarations: []}`, Horizon `handoff.ts:81`), not an empty
   field. `F:132` ("Removed: the always-empty `settlement_criteria`") risks deleting a field the
   case keeper requires — run the `case_preflight` that `G:207-208` promises
   (`sea-forge-case.ts:181`) before removing it.
5. **`design.md:82-83` still counts wrong**: "four capabilities (`rlm.providers`, `rlm.invoke`,
   `rlm.inspect`, `rlm.cancel`, plus operator-only `rlm.reconcile`)" names five (rev-1 minor 1).
6. **`rlm.inspect.payment.currency` is `string`** (`D1:94`) where `budget` uses `Currency`. Same
   package; use the union.
7. **O3 is answered but still listed open.** "May `case.escalate` be used from a solo Expert?" is
   settled by `D3:175` ("Refused from inside any run and from any non-operator actor"). Remove it
   or mark it assumed, or the I6 scoping conversation restarts for nothing.
8. **The one artifact nobody in this chain has opened** is the 0.9.2/0.10.0 tarball (G #3), and I0
   is the only increment whose verification depends on it. Consider making I0's first task "fetch
   and grep the tarball" rather than "install and hope".

---

## Spot-checks performed

Read directly, at the named commits. **Verified as claimed:**

* Branches `feat/capability-runtime-t4a` (`bbe6c73`) and `feat/wp23-assignment-binding` (`668f686`)
  exist; Horizon `0e34a34`, gauntlet `6a7fe6a`, cognate `29dcd1e` match the package's baselines.
* `experts/service.ts:119-124` refuses a second binding ("Assignment already bound"); `:190` is the
  Team `maxDepth` check; `:195-199` is the authority/membership intersection.
  `experts/types.ts:11` has `reconciliation-required`; `:58-68` is `ExpertExecution` (no `runner`).
* `observations.ts:28-37` contains `world.settled` and **not** `workbench.settlement.recorded` —
  rev-1 M1's correction holds in rev 2.
* `capabilities/subagentStart.ts:43-57` derives the child principal from the parent's starter
  exactly as `D4.2` describes.
* `capabilities/connectHorizon.ts:41-49` mints one session for the operator identity, so the
  `assertedBy`-not-refused correction is right.
* `gauntlet` `crates/gauntlet-ports/src/agent_runner.rs:150-229` — `InvocationStatus`,
  `is_terminal`, `RunnerCapabilities{isolation, result_delivery, max_depth}`: the adopted
  vocabulary is real. `TerminalState` is `gauntlet-domain/src/values.rs:262-268`. `gauntlet run`
  accepts exactly `--dry-run` and `--config`; no `--project`.
* `cognate/packages/runtime-api/src/types.ts:14-38` `CaseContext`; `:264` `startRun` with optional
  `caseContext`; `index.ts:1` re-exports every type; `assignment.ts:19-20` `Currency` and
  `number | "unknown"`.
* `runtime-bun/src/service.ts:143-148` — one run per assignment id, `expectedVersion: 0`.
* `runtime-bun/src/index.ts:57` `components?`, `:87-96` `ambiguousSteps`; `runtime.ts:184` default
  `"reexecute"`; `executor.ts:186-205` `run.invocation.ambiguous` + `ambiguous-invocation`.
* `execSpawn.ts` at `bbe6c73`: `worldMatrix`/`decideContainment` and the refusal codes cited in
  MUST-FIX 8 and 12, including `stream-port-unavailable`, `posture-contained`, `no-process-groups`
  at the lines `D4.9` and J13 cite.
* `Horizon/src/capabilities/handoff.ts:77-86` `handoffItem` (five keys);
  `Horizon/src/adapters/workshop.ts:174` (`expert.converse` payload `{id, requestId, expertId,
  workspaceId, message}`).
* Dependency reality: `cognate-workbench/package.json:19-38` = `0.9.2`, `Horizon/package.json:20-29`
  = `0.10.0`, both `node_modules/@godspeedai/cognate-runtime-api` = `0.8.0`, `bun.lock` entries are
  registry tarballs with integrity hashes. `src/server.ts:13,720` shows Workbench calls
  `createRuntime` in-process (so `ctx.runId` is available and the top-level refusal is
  implementable).

**Contradicted (basis of the MUST-FIX items):** `E:14` vs `D0:11` (native); `E:80-81`,`E:186`
(truncated J6); `case-context.ts:11-33` vs `D1:72-77` and `D3:155` (`role`, `worldRef`,
`episodeRef`, `worldCorroboration`); `B:25-26` vs J11 (stateless contract, restart inspect);
`B:336-337` vs `executor.ts:186-205` (G18 "Current"); `run_cmd.rs:64-70,466-471,464-476` vs
`D3:169-171`; `D1:82-83` vs `D4:187-189` vs `E:41-42`; `D5:221` vs `D4:194-196`/`F:43-44`;
`D1:59`/`D4:181` vs `E:135-136`; `D7:244` vs `F:85-87`; `B:361` vs `F:67-69`.

**Not checked** (needs network or a run): whether the `0.9.2`/`0.10.0` tarballs contain
`CaseContext`; whether any consumer of `pluginRegistry` remains; whether Prime can be launched
bridge-only (G #1); the `sea-rs` `fde/integration` diff (G #4, #5). None blocks a MUST-FIX; all are
already listed in `G §1`.

*Drafted with EGM*



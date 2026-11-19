# G — Residual uncertainty and design risks (revision 3)

Revision 3 applies `egm-resolve` to the previous register. Every item now has a disposition, and
the ones that could be settled by evidence were settled rather than carried:
[`egm-review/goldfish_review_report.md`](egm-review/goldfish_review_report.md).

## G1. Questions settled in this pass

| # | Question | Verdict | Evidence | Effect on the design |
|---|---|---|---|---|
| 1 | Can Prime be admitted without a tool allowlist? | **Yes — through the port; not through its tool list** | **Tool route unavailable:** `prime-agent` 0.10.2 is the Rust build; no `--tools`/`--no-tools`/`--no-builtin-tools`/`--exclude-tools` in help, zero binary string hits for `defaultTools`/`no-builtin-tools`/`exclude-tools`, no `defaultTools` in `~/.prime/agent/settings.json`; the allowlist docs belong to the **TypeScript donor** in `gauntlet/.tmp/donor/pi/`. **Port route verified on this host (bwrap 0.13.1):** live stdio under `--unshare-all` (`PORT_OK`/`GOT:hello`), network unreachable, bind-mounted unix socket reachable (`PROXY_OK`) | Adapter declares `toolPolicy:"unattested"` **and** `portGovernance`; admission uses Route B (D8). Until I3b lands, only an ungoverned `local` port exists, so the old refusal still applies in practice |
| 2 | Is a contained provider viable? | **Yes, with a contained streaming world and proxied egress** | Governance requires `contained` (`src/server.ts:510-511`); today's wrapper refuses host spawns under `contained` (`execSpawn.ts:196`) and its bubblewrap provider is buffered, not streaming. The mechanism itself is capable: verified live stdio, network denial and socket egress above. What is missing is a contained **streaming** world plus a declared egress channel | I3b is **in scope**. Port route: contained streaming world + one proxied egress socket + fs-diff evidence; ACP events are claims, not approvals |
| 3 | Does `pluginRegistry` still have a consumer? | **Yes — four plus a test** | `src/server.ts:111,638,806` (constructed and injected), `src/capabilities/pluginInstall.ts:14`, `src/workbench/bundleSources.ts:22`, `test/workbench-plugins.test.ts:5` (six `verifyImplementation` assertions) | `B` G16 and `B4` change from DELETE to **KEEP**; deletion deferred behind a port to the bundle store |
| 4 | Is `sea-rs` `fde/integration` safe as baseline? | **Only after review** | 45 files / 19,131 insertions versus `main`, including `crates/sea-forge-settlement/src/lib.rs` (190 lines) and new server tests | Baseline stays `main`; the settlement diff is a named pre-read for the implementer |
| 5 | Does Gauntlet `fde/integration` change the CLI/reference? | **No** | 7 files, 3 insertions, 391 deletions, all in `doctor_cmd.rs`, `fabric_compose.rs`, `lib.rs`, `report_manifest.rs`; no `run_cmd.rs`, no `reference/*` | I6 keeps its pin; only doctor output differs |
| 6 | DomainForge → SEA-Forge envelope accepted without translation? | **Still unverified** | Both CLIs are installed (`domainforge`, `realitytrace` on `PATH`), but acceptance needs a running SEA-Forge cell | Remains a fixture test in J14; not a design assumption |
| 7 | Seven CEP profiles or eight files? | **Seven profiles + one record schema** | `cep/schemas/profiles/godspeed/` holds 7 `*.v1.schema.json` profiles and `world-transition.v1.schema.json` | `A4`/`D6` say so explicitly; the spec's "Six" is stale |
| 8 | Can the TS Prime host be extracted as three files? | **No — the closure is larger** | Import closure reaches `./focus`, `./horizon`, `./affordance`, `./operation-gateway`, `./prime-topology`, `./terminal-session` | I4 extracts protocol + driver only; Gauntlet keeps its policy/topology modules |
| 9 | Are Expert effects confined to `tool.*`? | **Yes, with two operator-only exceptions** | `src/capabilities/tools.ts:172-173` is the only Expert path; the other `process.exec` callers are `src/capabilities/cognitive.ts:111` (cases) and `src/workbench/gitOps.ts:38` (UI git) | The claim stands with the exceptions named |
| 10 | Does Cognate 0.9.2 contain `CaseContext`? | **Unverified; only `bun install` can settle it** | Installed tree is 0.8.0 and has no `CaseContext`; the manifest pins 0.9.2 | I0 is the settling action; not a design assumption |

## G2. Owner decisions remaining

**None.** Both were decided in revision 5 with the evidence that settles them:

* **O1 — adapter form:** a package, `@godspeedai/cognate-rlm-prime`, speaking **ACP**. Prime 0.10.2
  answered an ACP `initialize` and `session/new`, so the protocol is real and current; a sidecar would
  add a wire format and a supervisor for no gain. Guard: if ACP disappears, fall back to `--mode rpc`
  using Gauntlet's fixture.
* **O2 — I3b in scope:** **yes.** It is the only route to a governed provider and the mechanics are
  verified (I1 rows 1–9). Guard: if the proxy proves unworkable, ship the port without network and
  keep governed providers refused.

O3–O8 are closed in [`I-closure-register.md`](I-closure-register.md) §I4.

### The two governed-provider routes (route 1 remains future work; route 2 is I3b)

1. **Host-mediated model calls.** The provider asks the bridge for `model.generate` instead of
   holding credentials and a network path. It needs no network, so `contained` becomes possible and
   the provider never sees a key. Cost: a larger adapter contract, and a Prime that can be told to
   use it — which G1 row 1 shows it cannot today.
2. **Controlled egress.** Keep the network namespace but route it through a host proxy with a
   declared destination set. Cost: a containment project with its own threat model, and it removes
   the `--unshare-all` guarantee the current posture relies on.



## G3. Residual risks (accepted, with guards)

1. **Two execution doors inside one Expert run.** One `runner` switch decides; a test asserts a
   single start-work site. Accepted.
2. **Adapter honesty.** `toolPolicy` is a declaration. Guard: operator authorisation is per provider
   id *and* executable digest, recorded in evidence. Attribution, not proof — sharper now that Prime
   defaults to `unattested`.
3. **Bearer-key attribution.** Horizon holds one session for one operator; `assertedBy` is a claim.
   Accepted; unchanged from current behaviour.
4. **CEP without a world.** A local case cannot be settled externally until it acquires a world.
   Accepted and stated.
5. **Gauntlet reference projection drift.** Guard: version pin, refuse on mismatch, fixtures copied
   from Gauntlet. Accepted.
6. ~~`pluginRegistry` deletion may break a consumer~~ — **closed** by G1 row 3. No deletion now.
7. **Version alignment is a hard prerequisite.** If I0 is skipped, cross-repo journeys pass in
   source and fail at runtime. Accepted as a sequencing rule.

## G4. Assumptions, now validated or rejected

| Assumption | Status |
|---|---|
| Provider-internal subagents are opaque and attributed to the Specialist's Assignment | **Accepted**; the only truthful option given no tool-level identity inside the provider |
| A provider without an interceptable tool path is unusable under governance | **Validated** for Prime 0.10.2 by G1 row 1 |
| The ordinary Expert loop is not RLM and must be described that way | **Accepted**; revision 2+ has no native provider at all |
| `uncontained-local` self-approval is a pre-existing hazard, not one this design creates | **Validated** (`src/server.ts:508-511`) |
| Cognate's settlement path is the same on `main` and `fde/integration` | **Rejected** for the branch; see G1 row 4. Design pins `main` |
| A contained provider is a policy choice the owner can simply allow | **Rejected**; it is blocked by two independent facts (G1 rows 1–2) |

## G5. Readiness review (revision 2) — must-fix dispositions

A fresh-context implementation-readiness reviewer produced 12 must-fix items
([`egm-review/readiness.md`](egm-review/readiness.md)). All twelve were fixed in place:

| # | Must-fix | Disposition |
|---|---|---|
| 1 | `E` not regenerated (J1 said "exactly native"; J6 truncated with an orphan tail) | **Fixed**: J1 expects an empty provider array; J6/J7 rebuilt; orphan removed |
| 2 | No increment owns the Cognate *release* | **Fixed**: I0 owns release and pin; I2's prior state says "released and pinned" |
| 3 | `caseContext` not shippable (`role` required, `worldCorroboration` not a field) | **Fixed**: `role` optional; corroboration moved to run evidence; refusal is the type's `episodeRef` rule |
| 4 | Parent cannot resume after `rlm.invoke` | **Fixed**: `rlm-invocation:<id>` continuation resolved by the RLM component |
| 5 | Invocation state has no home | **Fixed**: mapping in parent run state via `updateState`; `inspect` reads mapping + child run |
| 6 | `unknown`/`rlm.reconcile` duplicates existing ambiguity machinery | **Fixed**: `rlm.reconcile` deleted; reuses `run.invocation.ambiguous` / `ambiguous-invocation` / `run.invocation.reconcile` |
| 7 | `rlm-budget-unverifiable` unreachable | **Fixed**: `reservationRef` required for a numeric limit |
| 8 | Authority specified two incompatible ways | **Fixed**: non-subset refused; recorded set is the intersection |
| 9 | Gauntlet invoked wrong (`<intent>` positional, binary exit code, stdout/stderr, snake_case states) | **Fixed** in `D3`, `C4`, `E` J7 |
| 10 | Unowned work (`affordance.field`, catalogue, `pluginRegistry`, `caseAssignmentId`) | **Fixed**: I5 (first three; `pluginRegistry` now KEEP-only), I7 (last) |
| 11 | `B4` DELETE of `execSpawn` contradicts I3 | **Fixed**: `B4` keeps `execSpawn`, deletes only the ad-hoc stdio plumbing |
| 12 | Retry keys contradictory | **Fixed**: reuse returns the existing run; a retry is a new invocation derived with the attempt number |

## G6. What still needs execution — now a decision tree, not an open question

Five checks cannot be settled by reading. Each is converted to a pass/fail decision with a
pre-agreed response, so none of them can stall the work:

| Check | If it passes | If it fails |
|---|---|---|
| `CaseContext` in 0.9.2 | proceed with I0 | pin 0.10.0; if that breaks Workbench, define the field locally and open the Cognate PR |
| DomainForge → SEA-Forge `evidence_packet` | J14 passes | carry profile fields only; if a required datum has no slot, refuse the handoff and file a CEP change proposal |
| A real Prime invocation under ACP | J4-with-provider passes | correct the adapter from a recorded transcript; if Prime cannot run non-interactively, ship contract + test provider and claim no RLM |
| Gauntlet escalation | J7 passes | drop I6 from this change set; it is optional |
| Runtime behaviour of any journey | TDD loop | report it failing; no design change unless a guard is disproved |

Full register: [`I-closure-register.md`](I-closure-register.md) §I5.

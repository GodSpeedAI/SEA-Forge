# Goldfish Annotated Review Report

**Target:** `G-residual-uncertainty.md` (full file) and the three open questions carried into `H`
**Procedure:** `egm-resolve` — every finding gets exactly one disposition
(`CONTEST | ESCALATE | PROPOSE FIX | DECLINE`), and a contest must be checkable.
**New evidence gathered for this pass:** the installed `prime-agent` binary and settings, the donor
pi sources vendored in Gauntlet, live import graphs, git diffs between `main` and the two
`fde/integration` branches, and the CEP profile directory.

> **Superseded in part by revision 4 (port governance).** The tool-list finding in F1 stands: `prime-agent` 0.10.2 has no allowlist. The *feasibility* verdicts in F1/F2 were too strong, because they assumed the port could not be governed. Smoking-tested bubblewrap 0.13.1 shows it can: live stdio under `--unshare-all`, network denied, bind-mounted unix socket reachable. Admission is now two-route (`D8`), with `I3b` delivering the contained streaming world. This report is left as the audit trail of what was believed at that point.

## Prioritized findings

### F1 — "Prime can run bridge-only" is not supported by the installed Prime, and the donor docs do not apply
* **Severity:** High (blocked I4's governed claim)
* **Affected scope:** `G` §G1 #1, `F` I4, `D` `toolPolicy`
* **Discrepancy:** `G` said "no evidence on either side". There is now evidence on one side.
* **Rationale and evidence:**
  * Installed binary is `prime-agent` **0.10.2**, a Rust build (`prime-agent --help`: "update to the
    latest Rust build (uninstalls the TypeScript version)").
  * Its help has **no** `--tools`, `--no-tools`, `--no-builtin-tools`, `--exclude-tools`.
  * Binary string search: `defaultTools` 0 hits, `no-builtin-tools` 0 hits, `exclude-tools` 0 hits.
  * `~/.prime/agent/settings.json` top-level keys: `defaultModel, defaultProvider,
    defaultThinkingLevel, mcpServers, npmCommand, onboardingShown, packages, recentModels, skills,
    telemetry` — no `defaultTools`.
  * The tool-allowlist documentation exists only in `gauntlet/.tmp/donor/pi/packages/coding-agent/docs/`
    — the **TypeScript donor**, not this binary.
* **AI suggestion:** **PROPOSE FIX**
* **Justification:** a capability the adapter cannot attest must not be declared. `bridge-only` is
  unattainable for Prime 0.10.2; declaring it would be the exact "prompt-only restriction" the
  owner forbids. The adapter must declare `unattested`, and every path that refuses `unattested`
  now refuses Prime.
* **Proposed edit:** `G` #1 becomes resolved with this evidence; `F` I4 loses any governed claim and
  accepts only an explicit operator opt-in on an ungoverned, uncontained host, recorded as
  `toolPolicy:"unattested"`; `D` adds one sentence that `bridge-only` requires CLI or settings
  evidence and shows none for Prime 0.10.2.

### F2 — A `contained` provider cannot be authorised today, for two independent reasons
* **Severity:** High
* **Affected scope:** `G` §G2 O2, `D4.9`, `E` J13
* **Discrepancy:** O2 was framed as a policy preference. It is a feasibility question with a
  negative answer.
* **Rationale and evidence:**
  1. Governance requires `COGNATE_EXECUTION_POSTURE=contained`
     (`cognate-workbench/src/server.ts:510-511`), and the governed wrapper refuses host spawns under
     that posture (`cognate-workbench@feat/capability-runtime-t4a:src/capabilities/execSpawn.ts:196`).
  2. The contained world unshares the network (`--unshare-all`), so a provider that must reach a
     model API cannot work there (the same file, `:64-66`).
  3. `bridge-only` would be the escape, and F1 shows Prime cannot attest it.
* **AI suggestion:** **PROPOSE FIX**
* **Justification:** "operator may authorise contained" is not a decision that unlocks anything; it
  would only let a run proceed and fail later. Keep the refusal, and name the two real future routes
  so the question is not reopened from memory.
* **Proposed edit:** O2 is closed as "no today". `G` records the two routes: (a) host-mediated
  model calls so a provider needs no network (feasible, larger contract), (b) a controlled egress
  proxy inside a shared network namespace (a containment project). Neither is in this design.

### F3 — `pluginRegistry` is live, and deleting it would break four consumers
* **Severity:** Medium (a wrong DELETE in the ledger)
* **Affected scope:** `B` G16 and `B4`, `G` §G3 risk 6
* **Discrepancy:** the ledger proposes DELETE gated on an inventory; the inventory now exists and is
  non-empty.
* **Rationale and evidence:**
  * `src/server.ts:111,638,806` — imported, constructed, and injected into `pluginInstallComponent`.
  * `src/capabilities/pluginInstall.ts:14` — imports the type.
  * `src/workbench/bundleSources.ts:22` — imports `PluginSource`.
  * `test/workbench-plugins.test.ts:5` exercises `PluginRegistry`, `verifyImplementation` and
    `PluginSource` (six assertions on `verifyImplementation` alone).
* **AI suggestion:** **CONTEST** the DELETE.
* **Justification:** a live registry with a capability mounted on it is not legacy by assertion. The
  consumer count, not its self-description, decides.
* **Proposed edit:** `B` G16 and `B4` change from DELETE to **KEEP** with a named successor plan
  (fold `PluginSource`/`verifyImplementation` into the bundle store when `pluginInstall` is ported;
  delete only then, with the test moved). `G` §G3 risk 6 becomes "closed: no deletion now".

### F4 — `CaseContext` availability in 0.9.2 remains unverified
* **Severity:** Medium
* **Affected scope:** `G` §G1 #3, `F` I0
* **Discrepancy:** unchanged; no 0.9.2 tarball was fetched.
* **Rationale:** the installed tree is 0.8.0 and lacks `CaseContext`; the manifest pins 0.9.2. The
  only way to settle it is an install.
* **AI suggestion:** **ESCALATE** (cannot be settled without a dependency install, which this phase
  forbids).
* **Justification:** I0 exists precisely to install and assert. Leave it as a pre-flight, not a
  design assumption.
* **Proposed edit:** none to the design; keep `G` #3 and add "settling action: `bun install`, then
  `grep CaseContext node_modules/@godspeedai/cognate-runtime-api/src`".

### F5 — `sea-rs` `fde/integration` differs materially in settlement code
* **Severity:** Medium (baseline choice)
* **Affected scope:** `G` §G1 #4, `A` baseline table
* **Discrepancy:** "no diff review was performed" — now performed at file level.
* **Rationale:** `git diff --stat main..fde/integration` over the authority/settlement/server crates
  is 45 files, 19,131 insertions, including `crates/sea-forge-settlement/src/lib.rs` (190 lines) and
  new field-case/server tests. The branch is not only docs.
* **AI suggestion:** **PROPOSE FIX**
* **Justification:** `A4`'s settlement description is accurate for `main`; claiming it for the branch
  would be unsupported. Name the diff and make the choice explicit.
* **Proposed edit:** `A` baseline note and `G` #4 record: design baseline is `main`; the
  implementation agent must review `crates/sea-forge-settlement/src/lib.rs` and
  `crates/sea-forge-server/tests/` before touching settlement-adjacent work.

### F6 — Gauntlet `fde/integration` does not change the CLI or reference format
* **Severity:** Low
* **Affected scope:** `G` §G1 #5
* **Rationale:** `git diff --stat main..fde/integration` shows 7 files, 3 insertions, 391 deletions,
  all in `doctor_cmd.rs`, `fabric_compose.rs`, `lib.rs`, `report_manifest.rs`. No `run_cmd.rs`, no
  `gauntlet-app/src/reference/*`.
* **AI suggestion:** **PROPOSE FIX**
* **Justification:** the risk to I6 is nil for the reference and CLI; only doctor output moved.
* **Proposed edit:** `G` #5 becomes resolved with this diff; I6 keeps its pin.

### F7 — The CEP profile count is seven profiles plus one record schema
* **Severity:** Low
* **Affected scope:** `A4`, `D6`, `DECISION 13`
* **Rationale:** `cep/schemas/profiles/godspeed/` contains 8 files: the seven `*.v1.schema.json`
  profiles and `world-transition.v1.schema.json`, which is a record schema, not a profile.
* **AI suggestion:** **PROPOSE FIX**
* **Justification:** the documents already say seven profiles; the directory count invites a wrong
  "eight". State it once.
* **Proposed edit:** `A4` and `D6` add "(+ one `world-transition` record schema, not a profile)".

### F8 — The TypeScript Prime host cannot be extracted as "three files"
* **Severity:** Medium (scope of I4)
* **Affected scope:** `G` §G1 #8, `B` G3, `F` I4
* **Rationale:** import closure from the four candidate files reaches `./focus`, `./horizon`,
  `./affordance`, `./operation-gateway`, `./prime-topology`, `./terminal-session` — Gauntlet policy
  and topology modules, not protocol.
* **AI suggestion:** **PROPOSE FIX**
* **Justification:** a Cognate package that drags Gauntlet's affordance and topology model across the
  boundary is worse than duplication: it makes Gauntlet's UX the RLM adapter's dependency. Extract
  the protocol and process driver only.
* **Proposed edit:** I4 extracts `prime-host` (port + bookkeeping) and `real-prime-host` (process
  driver) plus the `terminal-session` *validation functions* it needs; `prime-recursion`,
  `prime-context-envelope`, `prime-topology`, `affordance`, `operation-gateway` stay in Gauntlet.
  `B` G3 records this boundary.

### F9 — Expert effects are confined to `tool.*`, with two operator-only exceptions
* **Severity:** Low
* **Affected scope:** `G` §G1 #9
* **Rationale:** every Expert-facing effect goes through `src/capabilities/tools.ts:172-173`
  (`filesystem.writeText`, `process.exec`). The other `process.exec` callers are
  `src/cognitive/cognitive.ts:111` (case operations, operator-invoked) and
  `src/workbench/gitOps.ts:38` (UI git actions); neither is reachable from an Expert tool call.
* **AI suggestion:** **PROPOSE FIX**
* **Justification:** the claim holds, with the two exceptions named so a later reader does not
  over-generalise it.
* **Proposed edit:** `G` #9 resolved with this qualification.

### F10 — O1 (package vs sidecar) is still an owner call, but the evidence now favours the package
* **Severity:** Medium
* **Affected scope:** `G` §G2 O1
* **Rationale:** F8 reduces the extraction to protocol + driver; a sidecar would additionally need a
  new wire format, a process supervisor and per-platform packaging for the same behaviour Gauntlet's
  TS host already has.
* **AI suggestion:** **ESCALATE** (owner), with a recommendation.
* **Justification:** it is a distribution decision, not a technical one.
* **Proposed edit:** O1 restated as a one-line owner choice with the recommendation attached.

### F11 — O3, O4, O5 can be decided now
* **Severity:** Low
* **Affected scope:** `G` §G2 O3–O5
* **Rationale:** O3 (`case.escalate` callers) — escalate is a spend-and-control decision; operator
  only is consistent with `world.settle` and `method.accept` being human acts
  (`src/capabilities/cognitive.ts:256,369`). O4 (local `worldHash` as CEP `world_ref`) — refused,
  because the `world_ref` grammar is `world:<name>@sha256:<64hex>` minted by the world registry and
  a local manifest hash is not that. O5 (restore verbatim vs latest seed) — verbatim, because the
  purpose is recovery, not upgrade.
* **AI suggestion:** **PROPOSE FIX** (decide and record; no owner knowledge needed).
* **Proposed edit:** O3–O5 become decisions, leaving O1 as the only owner call.

### F12 — Residual risks 1–5 and 7 are accepted with guards; risk 6 closes
* **Severity:** Low
* **Affected scope:** `G` §G3
* **Rationale:** risks 1–5 and 7 already name a mitigation or an acceptance; risk 6 (`pluginRegistry`
  deletion) is closed by F3.
* **AI suggestion:** **DECLINE** (recorded, not fixed). Risk 6 is removed.


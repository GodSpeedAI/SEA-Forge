# WP-19 evidence: Horizon read path (Horizon half)

Repo Horizon, worktree `~/projects/_wt/fde-horizon-integration`, branch `fde/integration`, commits `fac2f84` (adapters, schemas, fixtures) and `fd2e19b` (panel, debugger feed, parity gate), pushed. No PR, merge, publish or version bump.

## Where

| Item | Pointer |
|---|---|
| Source states, shared types, `NOT_CONNECTED = "not connected"` | `src/adapters/fde-types.ts` |
| Case records adapter (`field-records.jsonl`, `field-anchor-outbox.jsonl`) | `src/adapters/fde-read.ts` `readCaseRecords` |
| RealityTrace adapter (compare result + derive wrapper) | `fde-read.ts` `readRealityTrace` |
| World pin adapter | `fde-read.ts` `readWorldPin` |
| JSON Schema subset validator (throws on an unimplemented keyword) | `src/adapters/fde-schema.ts` |
| Vendored contracts by digest | `contracts/fde/{case-record-append,frozen-reference,residual,observed-representation}.v1.json`, `DIGESTS` |
| Display table `CLASS_DISPLAY` (the NT-09 attack surface), debugger rows | `src/adapters/fde-view.ts` |
| Paths/env (`HORIZON_FDE_CASE_DIR/_COMPARE/_DERIVE/_PIN`) | `src/adapters/fde-sources.ts`; `createApplication` option `fdeSources` (`src/app.ts`) |
| Desk panel (ids and digests as text, `data-field` per value) | `src/ui/desk/EpisodeEvidence.tsx` |
| Debugger feed: `DebuggerSources.fde`, `why` about an episode ref | `src/domain/debugger.ts` (`why`), `src/app.ts` `inspectFocus` |
| Independent oracle (no adapter imports) | `test/support/fde-parity.ts` `expectedFromRaw`, `parity`, `renderedFields` |
| Browser harness (the desk component in Chromium) | `test/support/fde-harness.tsx` |
| Tests | `test/fde-parity.test.ts` (30), `test/fde-read.test.ts` (19) |
| Command | `scripts/fde-parity.sh`, `bun run parity:fde`; docs `docs/fde-read-path.md` |

Reused pattern: Playwright harness bundle (`Bun.build`) as in `test/desk-outcomes.test.ts`; adapter outcome unions as in `src/adapters/EGM.md`; `DebuggerSources` / `why` / `DebuggerValue` classes. No parallel source pattern was created.

## Fixture provenance (details in `test/fixtures/fde/README.md`)

- RealityTrace (`rt/*`): REAL `realitytrace observed ingest|derive|compare` binary from `~/projects/_wt/fde-sxr-integration` at 4930b7d, via `test/fixtures/fde/generate/gen-rt.py`; the only non-CLI step is `rt-seed`, a 25-line crate appending the `evidence.recorded` ledger rows exactly as `sxr-cli/tests/compare.rs::setup` does. Scenarios: confirmed+observed_undeclared (tail unverified), same with `--expected-head` (tail `verified_against_expected_head`), contradicted, insufficient (`evidence_source_silent`).
- SEA-Forge case records (`sf/*`): REAL `case_records::append`, the unmodified `case_records.rs` and `interaction_events.rs` compiled through `#[path]` (`generate/sf-seed`), driven over the contract's own valid request fixtures (`contracts/fde/fixtures/case-record-append/valid`). Source commit 69357a5d plus uncommitted edits to `case_records.rs` in that worktree (another WP in progress); the exact file hashes are in `generate/PROVENANCE.txt`.
- World pin (`pin/*`): HAND-WRITTEN (labelled), shaped by `PinState::to_json` / `PinRecord`. No producer output was generated: it needs the full authority crate and an approved promotion.
- DomainForge `evaluate` (binary 3960326) was not exercised; no `policy_*` basis fixtures.

## Results (verbatim)

- `bun test test/fde-parity.test.ts`: `30 pass`, `0 fail`, `181 expect() calls`
- `bun test test/fde-read.test.ts`: `19 pass`, `0 fail`, `51 expect() calls`
- `scripts/fde-parity.sh AT-14`: `fde-parity: PASS (AT-14, 28 tests)`; `scripts/fde-parity.sh NT-09`: `fde-parity: PASS (NT-09, 2 tests)`
- Teeth of the command itself: with `insufficient: { label: "ok"` temporarily in `fde-view.ts`, `scripts/fde-parity.sh AT-14` printed 6 `PARITY_MISMATCH` and exited 1; file restored.
- `bun run check`: `check: ok (local_operator, 127.0.0.1:4319, horizon)`; `bun run boundaries`: `boundaries: ok (.)`; `tsc --noEmit` clean.
- Full suite `bun test`: `1688 pass`, `16 skip`, `0 fail`, `12745 expect() calls`, `Ran 1704 tests across 185 files` (baseline 1639 / 16 / 0; +49 are the new tests).

## AT-14

Each of 3 case ledgers x 4 RealityTrace results x 2 pins (24 mounts) is rendered by the desk component in headless Chromium; every expected field (case id, plan item, episode n, episode ref, frozen digest, anchor state/tag, world, residual id/class/basis/rule/occurrences/world, comparison id, representation id/hash, input set, tail status, settlement result/actor/ref/record, pinned world and history) equals the independently computed canonical text; ids and digests match their formats. Also: all sources absent renders exactly "not connected" in four places and no `affirmed` element; one absent source leaves the others rendered; a residual missing `world_ref` renders "invalid source" with the violation and no `rt.*` or `residual.*` field; `insufficient` renders tone `undecided` plus "Not decided. This is not a pass."; `contradicted` tone `denied`; an unverified tail is tone `undecided` with "tail NOT verified", a verified one `affirmed`; settlement `U` is `undecided`, `S` `decided`.

## NT-09 (the mutation)

The attack is a text patch of the adapter mapping applied to the bundle by a Bun plugin (the file on disk is never changed, so restore is by construction and asserted): `contradicted: { label: "contradicted"` becomes `label: "confirmed"` in `fde-view.ts`. Result, verbatim from the test's assertion:
`[{ code: "PARITY_MISMATCH", field: "residual.0.class", expected: "contradicted", rendered: "confirmed" }]`
The unpatched bundle gives no mismatch before and after; the patch is caught on every fixture holding a contradicted residual and only those.

## Deviations and limits

1. No JSON Schema validator existed in the repo and no dependency was added: a subset validator was written (`fde-schema.ts`). It agrees with the producers' own valid/invalid fixtures except the cross-field rules the contracts say need a contract test (digest equals episode_ref digest, ordering, count equalities); those are not checked by Horizon.
2. The compare result document and the `authority_pin` response have no JSON Schema; their top-level shape is checked by hand-written code, residuals and the representation by the real schemas.
3. The case ledger and the RealityTrace fixtures are different episodes (different producers' fixtures); the panel is given the case's episode ref explicitly. `selectEpisode` otherwise uses the comparison's `episode_ref`. No cross-system join is asserted.
4. The case ledger hash chain is not verified by Horizon (it renders what the file holds); `field-records.jsonl` is read as a file, not through `case.field.get` (WP-20, not present).
5. Settlement world is the world of the freeze its `episode_ref` names; the settled record carries no world of its own.
6. The parity test fails (never skips) when no Chromium build exists.
7. `README.md`-level docs: `docs/fde-read-path.md`, `src/adapters/EGM.md` line; `docs/documentation-map.md` not updated.

## NOT done

Any write path, Workbench change, WP-40 full debugger (diff/counterfactual/reach/replay over these sources), settlement UI actions, a real `authority_pin` fixture, DomainForge-policy fixtures, `scripts/fde-proof.sh` wiring and the S1 claim-ledger builder (SEA-Forge half of WP-19), `just verify` (not run in full; check, boundaries, tsc and the whole bun suite were).

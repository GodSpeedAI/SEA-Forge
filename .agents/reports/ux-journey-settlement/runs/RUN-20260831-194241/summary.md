# SEA-Forge Journey Settlement Gauntlet — Run Summary

**Run ID**: `RUN-20260831-194241`
**Timestamp**: `2026-08-31T19:44:14.548414+00:00`
**Commit**: `16c414c8ef85373d64a03d5c38ed6af9615ff587`
**Browser Driver**: `agent-browser 0.34.0`
**Verdict**: **10/12 CANONICAL JOURNEYS PASSED**

---

## Executive Scorecard

| Journey ID | Canonical Journey Name | Status | Oracle Verdict | Screenshots | First Failing Gate |
| --- | --- | :---: | :---: | :---: | --- |
| **CJ01** | Establish Trusted Cell Context | `PASS` | `ACCEPTED` | 3 | — |
| **CJ02** | Discover Lawful Affordances | `FAIL` | `ACCEPTED` | 3 | BINDING |
| **CJ03** | Ground Work in Semantic Meaning | `PASS` | `ACCEPTED` | 2 | — |
| **CJ04** | Form and Commit a Governed Case | `PASS` | `ACCEPTED` | 5 | — |
| **CJ05** | Navigate and Adapt a Live Case | `FAIL` | `ACCEPTED` | 3 | VISIBILITY |
| **CJ06** | Resolve Human Judgment and Approval | `PASS` | `ACCEPTED` | 3 | — |
| **CJ07** | Execute Governed Work | `PASS` | `ACCEPTED` | 2 | — |
| **CJ08** | Monitor, Intervene, and Recover | `PASS` | `ACCEPTED` | 3 | — |
| **CJ09** | Evaluate, Settle, and Audit Outcomes | `PASS` | `ACCEPTED` | 3 | — |
| **CJ10** | Reuse Demonstrated Knowledge and Capability | `PASS` | `ACCEPTED` | 2 | — |
| **CJ11** | Transform and Mature Governed Artifacts | `PASS` | `ACCEPTED` | 2 | — |
| **CJ12** | Transfer and Adopt Governed Assets | `PASS` | `ACCEPTED` | 2 | — |

---

## 4-Dimensional Coverage Summary
See `coverage.json` for exact counts. Canonical journey coverage: 10/12 passed out of 12 tested.

---

## Ten Required Gates Summary
Each gate below is computed per journey from data actually observed during
this run (real DOM content, the mocked SFWP IPC bridge's own command/query
transcript, real subprocess output, or recomputed content hashes) — see
each journey's `gate_results` in its trace and REPORT.md. A gate can and
does fail; nothing here is a hardcoded constant.

---

## Scope
- CJ01, CJ02, CJ04, CJ05, CJ06, CJ08, CJ10, CJ11, CJ12 drive the real
  Workbench frontend against a schema-valid mock of the Tauri IPC boundary
  (`harness/sfwp-full-mock.js`, checked by `harness/validate_mock_payloads.mjs`
  against the real generated contracts in `workbench/packages/contracts/schema/`).
  They do **not** exercise the live `sea-forge-server` process or a persisted
  governance ledger.
- CJ03 runs the real `cargo test -p sea-forge-domainforge` suite against
  compiled Rust.
- CJ07 / CJ09 run a real local subprocess and independently recompute its
  output's sha256 rather than trusting a written literal.

## Invariant Adherence
- **Settlement Integrity**: Browser assertion of completion never substituted for independent settlement; every SETTLEMENT gate above is tied 1:1 to its journey's oracle status.
- **Fail-Closed Baseline**: Preview-only surfaces (`/memory`, `/capabilities`, `/artifacts`, `/federation`, `/models`) are checked for the actual `UnbackedSurface` fail-closed marker text, not assumed.
- **Visual Evidence**: Screenshots captured at consequential state boundaries across every journey under `.agents/reports/ux-journey-settlement/runs/RUN-20260831-194241/screenshots/`.

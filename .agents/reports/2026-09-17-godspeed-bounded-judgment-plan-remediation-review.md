# GodSpeed Bounded-Judgment Plan — Finding Vetting and Remediation Review

**Date:** 2026-09-17

**Reviewed report:** `2026-09-17-godspeed-bounded-judgment-plan-adversarial-inspection.md`

**Remediated plan:** `godspeed-bounded-judgment-plan.yaml` v1.1.0

**Normative spec:** `GODSPEED_JUDGMENT_PLANE_SPEC.yaml` v1.0.0, unchanged

**Verdict:** findings substantially valid; remediation blueprint required corrections

## Vetted findings

| Finding | Disposition | Repository evidence / correction |
|---|---|---|
| Phantom `gauntlet-engine`, `gauntlet-storage`, and `gauntlet-kernel` packages | Confirmed | `cargo metadata` lists `gauntlet-domain`, `gauntlet-ports`, `gauntlet-app`, and `gauntlet-adapter-state-sqlite`; none of the three named packages exists. |
| `just crate-test` used against Gauntlet | Confirmed | Gauntlet exposes canonical `just check-package` and `just test-package`; the report's raw-`cargo test` replacement was rejected because Gauntlet policy requires repository recipes. |
| T06 unresolved implementation placeholder | Confirmed | T06 now binds its first implementation to `gauntlet-domain` and uses exact Gauntlet package recipes. |
| T12 parenthetical shell syntax | Confirmed | T12 now uses structured gate entries with explicit repository, host, working directory, and executable command. |
| Phantom SEA source/symbol | Confirmed | `PolicyDecisionKind` and `sea-forge-cell/src/policy.rs` do not exist; the plan now cites `GovernanceDisposition` and the mandated Gauntlet verification carrier. |
| Incorrect top-level Gauntlet evidence paths | Confirmed | Evidence is per run under `targets/.runs/<run-id>/evidence/{objects,pins}`. The report's wildcard replacement was insufficient; T01 now freezes exact paths and hashes in a corpus manifest and refuses runtime globs. |
| False statement that EdgeAI has no tests | Confirmed | EdgeAI has a governed `tests/` tree and canonical recipes. The plan now states that fact without replacing targeted target-machine drills with a full suite. |
| Missing task-local gate scripts | Confirmed | T00 now has a checked-in validator. T01/T05/T11/T12/T13/T14 explicitly author and hash their task-local scripts before first execution; correction rounds cannot overwrite them. |
| High-risk confirmation downgrade | Confirmed and understated | Five of six groups were nonconformant, not three: semantic bounding, uncertainty integrity, claim scope, provenance replay, and safe failure. v1.1.0 gives every requirement in each group a conformant confirmation task. |
| T13, T14, and T08 dependency defects | Confirmed | T13 now depends on T07; T14 depends on T02 and T03; T08 depends on T03. All task dependencies, graph edges, and initial blocked state now match exactly. |
| Traceability misplacements | Confirmed | CLAIM-004/005/006, RECOV-003/004, and provenance requirements now appear in both the relevant task `settles` lists and the bidirectional traceability map. |

## Additional defects found during vetting

1. `explicit_non_requirements` was indented inside the `target_settlement` block scalar, so YAML parsers did not see it as a key. It is now a top-level field and the validator checks it.
2. The report proposed `gauntlet-adapter-agent-http` as if it existed. It does not. The accepted spec requires one HTTP transport adapter behind `AgentRunner`; v1.1.0 names the future crate explicitly and requires operator approval before changing workspace membership or dependencies.
3. T08 called SEA's `just no-async-kernel` the binding gate for a Gauntlet dependency boundary. That gate cannot prove Gauntlet layering. T08 now binds Gauntlet `just boundaries` and retains the SEA gate separately.
4. Changing only T06 to independent confirmation would still leave `REQ-JUDG-003` at T11 without independent confirmation. T11 is now independently confirmed too.
5. Fresh target reproduction must cover every provenance-replay requirement, including correlation requirements 021/022. T12 now settles and re-proves all six provenance-replay requirements.
6. The original status said both “ready” and “rejected pending remediation.” Status now has one executable truth: v1.1.0 is ready for T00 only.

## Spec disposition

The normative spec was not edited. Its sha256 remains
`69b7d1baabd55ae61c73e615f04cf0fda43a4958a9fa3d50567035425baf6a82`,
its final self-check remains 12/12, and its correction ledger remains 11 entries
with no architectural conclusion reversed. The defects were projection and
execution-contract defects, not normative contradictions.

## Mechanical closure

`.agents/plans/validate-godspeed-bounded-judgment-plan.py` now rejects:

- spec hash/version/status drift;
- failed spec self-check or correction-ledger drift;
- missing/swallowed plan keys;
- incomplete task contracts or missing independent-confirmation packets;
- absent preregistration declarations;
- missing gate scripts without explicit authoring steps;
- unresolved gate placeholders and stale phantom literals;
- traceability that differs from task `settles` in either direction;
- stale, cyclic, or state-inconsistent DAG declarations; and
- any high-risk requirement without its mandated confirmation mode.

Current mechanical result: 90/90 normative requirements mapped, 15 tasks,
26 dependency edges, acyclic graph, T00 sole ready task, and 6/6 high-risk
groups confirmation-conformant.

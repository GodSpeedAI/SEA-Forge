# GodSpeed Bounded-Judgment — Implementation Brief

> NON-NORMATIVE execution context. The normative specification governs:
> `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` (v1.0.0, authoritative).
> The executable plan is `.agents/plans/godspeed-bounded-judgment-plan.yaml`.
> Plan revision: v1.2.0 (2026-09-17 remediation + controlled representational
> rebase; v1.1.0 = remediation).
> Mutable execution state lives in `.agents/current_status.yml`.
> Any conflict between this brief and the spec MUST be surfaced as a blocked
> decision, never silently resolved.

## Read order for a cold agent

1. `AGENTS.md`, then `.agents/AGENTS.md`; read `crates/AGENTS.md` before
   touching Rust. When a task touches a sibling repository, read that
   repository's root and nearest scoped `AGENTS.md` files before editing it.
2. This brief.
3. `.agents/current_status.yml` — authoritative ready/blocked/settled state.
4. The plan YAML — your task's `changes / proves / does_not_prove / steps /
   gate / teeth / redesign_trigger / evidence / done_when`, plus its
   `hot_context` file list.
5. The spec — only the requirement ids your task settles, plus the
   guardrails you must not violate.
6. `.agents/evidence/godspeed-bounded-judgment/decisions.yml` — decisions
   and discoveries that changed the execution model.

## The one-paragraph picture

The spec defines bounded, typed, provenance-preserving judgment that may
INFORM but NEVER REPLACE deterministic authority, evidence, execution,
observation, or settlement. There is no Judgment Plane service, port,
database, or deployment artifact — it is an architectural concern realized
through existing machinery. The plan first MEASURES (PROOF-1, read-only,
T01) whether uncertainty-preserving output carries information on one frozen
surface, then settles the minimal slice only if measurement authorizes it.

## Execution order (also computable from the plan DAG)

- **T00 first, alone.** Freeze authority (spec sha256), bind real gates,
  validate the plan. Nothing else starts before T00 settles.
- **After T00:** T01 (PROOF-1), T02 (Gauntlet CI), T03 (server gate repair),
  T04 (authority non-bypass), T05 (correlation) all become ready. T02–T05
  are independent of T01's outcome — do not serialize them behind it.
- **T06** needs T01's settled outcome recorded in the decision log.
  If T01 returns A or B, or C without the second-surface evaluation,
  T06 stays blocked and the branch consequences in T01 apply.
- **T07** needs T06. **T08** needs T03+T06. **T09** needs T07+T08. **T10** needs T07.
  **T11** needs T08+T09. **T12** needs T05+T09+T10+T11.
  **T13** needs T01+T05+T07. **T14** needs T02+T03+T04+T12+T13;
  it changes nothing and cannot settle while any earlier task remains open.
- Critical path: T00 → T01 → T06 → T07 → T09 → T11 → T12 → T14.

## Repository facts that are expensive to rediscover

- The judgment corpus lives OUTSIDE this repo under per-run directories in
  `../gauntlet/targets/.runs/`. T01 discovers candidates, then freezes exact
  run ids, database/evidence paths, and sha256 values in its corpus manifest.
  The harness refuses runtime globs and writable sources; any mutation is a
  plan violation.
- Historical rows carry `model: null` — never fabricate model identity.
- The corpus is ~17 settled judgments dominated by one run: every metric
  is unstable and must be labelled as such.
- T08 uses the existing synchronous `gauntlet_ports::AgentRunner` port and,
  after explicit operator approval, adds one edge adapter crate named
  `gauntlet-adapter-agent-http`. Gauntlet `just boundaries` is its binding
  dependency-direction gate; SEA `just no-async-kernel` remains independently
  required and does not prove the Gauntlet boundary.
- Target-machine recipes (`just health`, `just jetson-check-ports`) resolve
  ONLY on the Jetson-class target. Development-host output is never target
  evidence.
- Sea-rs gates: `just check | test | ci | proof | no-async-kernel`,
  narrow: `just crate-check <crate>`, `just crate-test <crate> [filter]`.
- Gauntlet gates: `just check | test | ci` (`ci` = check test lint deny
  boundaries ruler); scoped packages use `just check-package <package>` and
  `just test-package <package>`. Gauntlet currently has NO hosted CI workflow —
  that is T02.
- The workstation is RAM-constrained. Heavy gates, builds, tests, and daemons
  run sequentially. Never run the full daemon suite or parallel heavy jobs.

## Language rules (violations fail gates)

- T01 produces a PILOT MEASUREMENT, never a calibration report. The terms
  calibrated probability, calibration curve, production threshold,
  population-level reliability, and model generalization are FORBIDDEN.
- A negative result falsifies a SURFACE, never the capability class.
  Surface-scoped language is mandatory.
- Substitution comparability has levels: A interface (T11), B operational
  (T12), C model-quality (UNPROVEN in this plan — claiming it is a violation).

## Evidence and correction discipline

- Evidence root: `.agents/evidence/godspeed-bounded-judgment/T<num>/`.
- Preregistrations: `.agents/preregistrations/godspeed-bounded-judgment-T<num>.prereg.yaml`
  (T01, T04, T05, T12, T14). Each is frozen and hashed before the first
  evidence-producing command for that task—not globally before T00.
- Task-local gate scripts under the evidence root are authored before first
  execution, hashed into that task's evidence, and never overwritten. A fix
  uses a new round directory. T00 alone uses the checked-in plan validator.
- Failed rounds are preserved as `round-2`, `round-3`, … — never overwritten.
- Baseline-red gates are recorded as PREEXISTING_BASELINE_FAILURE with exact
  test ids — never weakened, skipped, or hidden to obtain green.
- T01/T04/T05/T09/T14 require independent adversarial confirmation;
  T06/T07/T11 require independent confirmation; T12 requires fresh target
  reproduction. The builder's narrative conclusion is excluded from every
  confirmation packet.

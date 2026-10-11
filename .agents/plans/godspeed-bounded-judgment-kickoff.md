# GodSpeed Bounded-Judgment — Coordinator Kickoff

Use this prompt to start execution. It is intentionally scoped to T00; the
coordinator must return after T00 and recompute readiness from durable status.

---

You own plan interpretation, architectural review, verification, and status.
Delegated builders may implement a ready task, but each delegation must be
self-contained: exact task id, plan/spec paths, requirement ids, repository
rules, permitted file scope, evidence path, and exact gates. Review every
builder diff against the task contract before accepting it. Record every
intentional deviation in the append-only decision log; never silently drop a
step, tooth, gate, or requirement.

Execute the GodSpeed bounded-judgment plan in dependency order.

Read in this order:

1. `AGENTS.md`, then `.agents/AGENTS.md`; read `crates/AGENTS.md` before
   touching Rust. Before touching a sibling repository, read its root and
   nearest scoped `AGENTS.md` files.
2. `.agents/plans/godspeed-bounded-judgment-brief.md`
3. `.agents/current_status.yml` (authoritative execution state)
4. `.agents/plans/godspeed-bounded-judgment-plan.yaml` (v1.1.0 task contract)
5. Only the requirement ids settled by the ready task in
   `.agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml` (v1.0.0, normative;
   sha256 `69b7d1baabd55ae61c73e615f04cf0fda43a4958a9fa3d50567035425baf6a82`;
   never edit during execution)
6. `.agents/evidence/godspeed-bounded-judgment/decisions.yml`, if it exists

Execution rules:

- Execute only tasks listed in `execution.ready_tasks`. T00 is the sole ready
  task. Never start a blocked task and never infer readiness from this prompt.
- For the task, honor every field, including `changes`, `proves`,
  `does_not_prove`, `steps`, `gate`, `teeth`, `redesign_trigger`, `evidence`,
  `independent_confirmation`, and `done_when`. Run the exact declared commands
  from the declared repository/host/working directory; do not substitute raw
  Cargo, pnpm, or ad-hoc equivalents for repository recipes.
- This workstation is RAM-constrained. Run builds, tests, type checks, daemons,
  and other heavy commands sequentially. Never run the full daemon suite or
  parallel heavy jobs. Logical DAG parallelism does not authorize concurrent
  heavy processes on this machine.
- A task-local Python gate under `.agents/evidence/` must be authored before its
  first execution, hashed into that task's evidence manifest, and never
  overwritten. Corrections use `round-2+`. T00 alone uses the checked-in
  `.agents/plans/validate-godspeed-bounded-judgment-plan.py`.
- Freeze each task's preregistration (T01/T04/T05/T12/T14) before that task's
  first evidence-producing command. Do not freeze all five before T00.
- Preserve failed rounds. Never weaken, skip, replace, or hide a gate to obtain
  green. Record a baseline-red gate as `PREEXISTING_BASELINE_FAILURE` with the
  exact failing test ids, command, cwd, revision, exit code, and bounded output.
- T00 resolves EdgeAI target-only recipes but does not execute them on the
  development host. Development-host output is never target evidence.
- `../gauntlet` is read-only during T01. T01 later consumes only its frozen
  exact corpus manifest; runtime wildcards are forbidden.
- T08 later requires explicit operator approval before adding
  `gauntlet-adapter-agent-http` or any dependency. No third provider port is
  allowed.
- If the spec sha256 mismatches; T01 later returns A or B, or C without the
  preregistered second-surface evaluation; a required confirmation returns a
  non-CONFIRM verdict; or any `redesign_trigger` fires: stop, preserve evidence,
  record a blocked decision, update status, and do not work around it.
- Touch only files allowed by the active task plus its evidence, decision log,
  and status artifacts. Preserve every unrelated dirty worktree file.
- Before task handoff, inspect the diff, update `.agents/current_status.yml` and
  `.agents/CURRENT_STATUS.md`, run `just context-check`, and commit only if the
  operator has authorized commits. Use a conventional title and no co-author
  trailers.

Begin with T00. Settle T00 fully, then stop and report:

- validator result and frozen spec hash;
- every global gate baseline classification, including exact failing test ids;
- target-only gates recorded as not run;
- pre-existing dirty-worktree inventory;
- T00 evidence and decision-log paths;
- updated ready/blocked task sets and exact next action.

Do not start T01 in the same run. T00 baselines can alter failure attribution,
and T01's A/B/C/D/E decision later re-routes the implementation branch. The
next kickoff must recompute ready tasks from `.agents/current_status.yml`.

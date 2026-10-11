# T08 production import isolation repair

Date: 2026-09-30

## Original bounded builder assignment

The fresh `production_import_guard_builder` was assigned only
`apps/godspeed-cognitive-ui/src/main.tsx`, with no tests/build/compile permission.
The independent round-7 critic had directly reproduced local-adapter/Northstar data in
default, explicit `local`, and invalid production Vite builds. The builder was instructed to:

- Put the local adapter and Northstar import branch directly behind `import.meta.env.DEV`,
  preserving local development as default and allowing development to select live.
- Force production HTTP regardless of source environment values; preserve session/login,
  source labels, local/default case selection and authenticated actor boot.
- Remove the static scripted local narrator import and import it dynamically only for
  development/local mode, retaining `?agent=off` and `agentFailAfter` fixture controls.
- Leave live narration null through the existing nullable port until T09 supplies grounded
  kernel Ask narration; never substitute a script or invent a Thoth answer.
- Add no dependencies, build aliases/plugins, config or public contract changes; do not
  refactor UI flows, edit other source files, commit, update status, or spawn agents.
- Report source changes, material differences and pending independent default/local/invalid
  builds, unit/typecheck and local browser gates. Source whitespace checks were allowed.

## Builder result and material differences

The builder reported completion and a passing `git diff --check`, with no compile, tests or
typecheck run. Root inspected the resulting diff: the local import branch uses the direct
DEV guard; the alternate branch imports `HttpCaseworkAdapter`; the static narrator import
is replaced by a type-only `NarrationPort` and a dynamic DEV/local import.

Live narration becomes explicitly null rather than running local scripts. This is the
declared fixture-isolation change; T09 grounded live narration remains outstanding. Local
fixture controls and session boot retain their original behavior by source inspection.
No runtime pass or bundle isolation claim is made from this inspection.

The builder could not write this note because its `.agents/` mount was read-only. Root
records the original assignment and received result here; the independent critic must
review the original instructions plus the source diff and execute the remaining gates.

## Pending evidence

- Fresh typecheck, focused and full Bun suite, canonical UI gate.
- Production default, `VITE_CASEWORK_SOURCE=local`, and invalid source builds: inspect every
  emitted asset for local adapter, fixture and scripted narrator leakage.
- Existing local agent-browser ladder on stable source.
- Actual live shared conformance/native reconnect proof and full T08 approval are still
  blocked on the separately requested live gateway launch authorization.

The failed pre-fix build and test results remain immutable in round-7 critic evidence.

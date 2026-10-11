# T08 independent recovery review — round 9 production build pin

**The package build-pin unit is APPROVED.** The only package change matches its bounded
assignment, and the canonical UI gate and all three production-source variants pass with local
fixture/narrator assets absent. **T08 remains unconfirmed** pending operator-approved F14 native
EventSource retention-one resync/reconnect proof and shared real-kernel conformance.

## Assignment and final diff

Reviewed `production-build-environment-fix.md` and the final package diff. Assignment allowed
changing only `apps/godspeed-cognitive-ui/package.json`'s `build` field from `vite build` to
`NODE_ENV=production vite build`; no dependencies, lockfiles, other scripts, source, or Vite
configuration were allowed to change.

The final package diff contains exactly that one field change. `dev`, `preview`, `typecheck`,
`test`, and `e2e` are unchanged. No material deviation found. The assignment builder ran no
build or tests; all results below are from this independent critic.

## Canonical UI gate

Before the compile gate, `NODE_ENV=development` was confirmed in the inherited environment.
`free -m` reported approximately 3.1 GiB available; processes were inspected with `ps`. Runs
were sequential. No Go/Rust build or gateway was run.

The normal sandbox cannot bind the explicit local HTTP listener, as established in round 8. I
therefore ran the canonical gate directly with the approved escalated command context:

```text
XDG_RUNTIME_DIR=/tmp just casework-ui-check
```

**PASS — exit 0.** Frozen install reported no changes; `tsc --noEmit` passed; the build log
showed `$ NODE_ENV=production vite build` and 109 transformed modules; Bun reported 255 tests,
0 failures, and 1268 assertions. The real loopback HTTP 503 tooth passed. The recipe ended with
`casework-ui-check: frozen install, typecheck, build and tests green`.

The production environment was scoped to Vite by the package script. The caller retained its
inherited development environment for subsequent commands.

## Default/local/invalid canonical build variants

Each build had a fresh `free -m` and process check (approximately 3.2 GiB available); builds
were serialized. All commands ran from `apps/godspeed-cognitive-ui` with inherited
`NODE_ENV=development`, through the canonical `bun run build` script, and used unique output
directories under `/tmp`:

```text
env -u VITE_CASEWORK_SOURCE NODE_ENV=development bun run build --outDir /tmp/T08-prod-default-20260930-round9
NODE_ENV=development VITE_CASEWORK_SOURCE=local bun run build --outDir /tmp/T08-prod-local-20260930-round9
NODE_ENV=development VITE_CASEWORK_SOURCE=invalid bun run build --outDir /tmp/T08-prod-invalid-20260930-round9
```

**PASS — all three exit 0.** Each output showed `$ NODE_ENV=production vite build`, transformed
109 modules, and emitted the HTTP adapter chunk
`httpCaseworkAdapter-BfSkWyjN.js`. The canonical `just` gate's `dist/` and each isolated output
contain 95 files. Scanning every emitted JS asset in those four production outputs for
`LocalContractAdapter`, `NORTHSTAR_CASE_ID`, `northstarData`, `createLocalAgent`, `localAdapter`,
`localAgent`, `evi-case-timeline`, and `case-northstar` found **no matches**. All outputs include
the HTTP adapter asset. Thus an unset source, explicit local override, or invalid override cannot
put the local fixture adapter or scripted narrator into production assets.

## Disposition and remaining scope

The bounded production build environment fix is approved with direct canonical and variant
evidence. The earlier F13, stale-refresh, retry, source-isolation, loopback tooth, and local
browser ladder evidence remains as recorded in round 8. No gateway was launched and the
pre-existing local UI listener was left untouched.

Full T08 is still **UNCONFIRMED**. The operator has not approved gateway launch, so no F14
real-native EventSource retention-one resync/reconnect proof or shared real-kernel conformance
run exists. Do not infer those proofs from local tests, the UI ladder, or bundle results.

No source, status, or commit was changed by this critic.

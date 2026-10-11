# T08 production build environment fix

Date: 2026-09-30

## Original bounded builder instructions

Change only the `build` field in `apps/godspeed-cognitive-ui/package.json` from
`vite build` to `NODE_ENV=production vite build`. Do not change dependencies,
versions, lockfiles, other scripts, source, or Vite configuration. Do not run a
build, tests, typecheck, or other compile command; the independent critic owns
the compile token. Record the original instructions, reason, exact result, and
limitations here, then return the change for fresh independent verification.

## Reason

The host environment inherits `NODE_ENV=development`. Vite defines
`import.meta.env.DEV` as true in that environment, even when invoking `vite build`,
so the canonical `bun run build` could compile the development-only local adapter
and narrator branch into its output. The direct DEV guard in `main.tsx` excludes
those imports when Vite actually runs in production mode, as confirmed by the
independent round-8 controlled build.

## Result and material difference

Changed only the package `build` script:

```diff
-    "build": "vite build",
+    "build": "NODE_ENV=production vite build",
```

The script now scopes production mode to the Vite process it starts. It does not
modify the caller's environment or alter `dev`, `preview`, `typecheck`, `test`, or
`e2e`. No dependencies, versions, lockfiles, source files, or Vite configuration
were changed.

## Existing causal evidence and limitations

The independent round-8 critic measured inherited `NODE_ENV=development` and
confirmed that a controlled
`NODE_ENV=production VITE_CASEWORK_SOURCE=local vite build` completed and emitted
HTTP adapter assets without local adapter/narrator markers. The same critic found
that uncontrolled diagnostic builds inherited development mode and included
local assets; those runs were correctly not counted as production checks.

This builder did not run a build or tests. The canonical `bun run build` after
this script change, plus default/local/invalid source bundle scans, still require
fresh independent verification. Full T08 remains unconfirmed pending those checks
and the live shared-conformance and native reconnect proof described in the
round-8 critic report.

# Renderer chunk serialized verification — 2026-10-07

## Result

The independent source review approved the checker and build hook. The
focused fixture test, strict typecheck, standalone production build, and
canonical `just casework-ui-check` all passed for the exact source identities
below. The canonical run completed frozen install, typecheck, build, and all
299 tests (0 failures; 1,674 expectations). This accepts the renderer
verification gates; it does not prove browser downloads, runtime interaction,
ladder behavior, live integration, or overall T09 settlement.

## Source identities

Each actual preflight recorded all three hashes. They remained unchanged
through the final canonical gate:

| Source | SHA-256 |
|---|---|
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.ts` | `52cdf00b9650f0413c91e4375c0ce5b6527b63ad9f09219f2102383dca4dec64` |
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.test.ts` | `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08` |
| `apps/godspeed-cognitive-ui/vite.config.ts` | `95c287317aa4028a1976c145124f918d0d856a3bb0ca60802f31abd909e901ea` |

All six gate preflights exceeded 1,258,291,200 bytes available RAM and
536,870,912 bytes free swap. Tool versions were Bun 1.4.0, Node v26.8.2, and
Just 1.58.0. Exact preflight captures preserve resource byte counts and
available `/tmp` space.

## Gates and dispositions

1. Focused `bun test src/build/rendererChunkContract.test.ts` from the UI app:
   exit 0; **13 pass, 0 fail, 35 expectations**.
2. `bun run typecheck` from the UI app: exit 0 (`tsc --noEmit`).
3. `bun run build` from the UI app: exit 0. The output contains
   `[renderer-chunk-contract] verified nine distinct renderer chunks` and
   emits nine separate renderer JS files. Vite also reported a chunk-size
   warning for an output over 500 kB; the build completed successfully.
4. Initial root `just casework-ui-check`: joined exit 1 before recipe start;
   `/run/user/1000/just` was read-only. Preserved as-is.
5. `JUST_TEMPDIR=/tmp just casework-ui-check` in the ordinary sandbox: joined
   exit 1. Install, typecheck, and build completed, but the suite reported
   **298 pass, 1 fail, 1,668 expectations**. The failing TOOTH SSE-drop
   journey received `EPERM` while creating its local `Bun.serve` listener.
   Preserved as-is.
6. The same canonical command with root-approved local-only execution
   escalation and a fresh preflight: joined exit 0. Frozen install,
   typecheck, build, and the full suite completed; **299 pass, 0 fail, 1,674
   expectations**. The build marker appeared. The two prior failed attempts
   remain preserved separately.

The Just runtime-directory retry used the previously approved
`JUST_TEMPDIR=/tmp` workaround. The listener retry used separate captures and
the root-approved local bind escalation. No source, fixture, package,
lockfile, or Git changes were made; builds produced generated `dist/` files.

## Emitted artifact evidence

The successful build marker confirms the emitted Rollup chunk graph passed the
assertion. Read-only inspection of `dist/assets/index-CdN8EEXV.js` found
dynamic `import()` references to all nine distinct renderer JS files. Their
actual paths, byte sizes, and SHA-256 values are in
[`renderer-chunks-root-actual-capture-manifest-oct07.json`](renderer-chunks-root-actual-capture-manifest-oct07.json).
That root-created immutable manifest also records all 18 actual
preflight/stdout/exit originals and their evidence copies with `cmp: 0` for
each. Exit captures were preserved byte-for-byte without a final LF.

No browser was started and no renderer download was observed. This result
remains limited to the assigned build assertion and its named gates.

# T09 canonical type-check first rejection

## Verdict

**REJECT test fixture pending a fresh repair.** Focused Bun is green, but the required
explicit strict TypeScript check exits 2 on errors in the canonical conformance test. The
errors are not missing compiler/type infrastructure and are not imported UI implementation
diagnostics under the minimal standalone invocation. I did not edit source or tests.

## Exact verification

Immediately before each compiler invocation, `free -h` reported at least 2.0 GiB available and
`ps -eo pid=,comm=,args=` found no Bun, Node, TypeScript, Go, Rust, or Cargo compiler process.
The Bun focused run passed 11 tests, 0 failed, 952 assertions; its raw output is
`canonical-final-green.log`, SHA-256
`98b7517f41be28b0ffb569972ab082f6090cb546bd728d8e3a350a951f964bed`.

The authoritative explicit type-check ran from
`apps/godspeed-cognitive-ui` with the existing `node_modules/.bin/tsc`:

```text
./node_modules/.bin/tsc --noEmit --strict --target ES2023 --lib ES2023,DOM,DOM.Iterable --module ESNext --moduleResolution bundler --skipLibCheck --types bun-types --typeRoots ./node_modules ../../.agents/reports/interface-contracts/tests/contract-conformance.test.ts
```

Exit code: **2**. The exact complete TypeScript stdout/stderr is preserved byte-for-byte in
`canonical-explicit-typescript-pins.log`, SHA-256
`c3c4c2fe9fe2328d4aabd7a2812c0b58bd4d1a73be263564ae2ecf68dc8b3161`. It was copied from
`/tmp/sea-rs-canonical-typescript-pins-minimal.log`; `cmp` returned 0. `rg -c 'error TS'`
reports 14 primary TS2769 diagnostic headers (with both overload explanations per header).

The source hashes at review time were:

- `contract-conformance.test.ts`: `89befcb93b26ee7d37d0ceb368b180a347148d4370cb18bb1bcf8f90a15ba0db`
- `typescript/types.ts`: `d272a25a7e799b839ac1997a4c153fc0a8697ca11f365708b100030c1b092b7b`

## Finding

All 14 compiler diagnostics are TS2769 overload errors in newly added runtime test helpers
in `tests/contract-conformance.test.ts` at lines 198, 203, 213, 214, 236, 237, 260, 261,
262, 266, 276, 283, 332, and 599. Each passes a value of type `unknown` to a typed Bun
`expect(...).toContain` or `toBe` overload. The finite-array membership checks should retain
their runtime semantics and can widen the expected array type safely (for example, to
`readonly unknown[]`). Do not use `any`, `@ts-ignore`, or remove the checks/type equality pins.

An initial stricter invocation copied every UI tsconfig flag and produced additional existing
errors in imported `client.ts`/`mock-adapter.ts` from `verbatimModuleSyntax` and unused-symbol
settings. That output is not the basis of this rejection: the command above omits those
project-only flags and still reproduces only the fixture's unknown-argument errors. The
`Expect<Equal<...>>` parity declarations themselves produced no errors, but the file as a whole
does not pass explicit strict type-checking, so they are not yet approved as a green typed
conformance test.

The source-schema repairs remain independently source-approved in
`canonical-second-implementation-source-review.md`; this rejection concerns the conformance
fixture's required TypeScript verification only. A fresh builder must repair only the
authorized canonical test helpers while preserving their membership and shape checks. The
next independent critic must rerun focused Bun and this explicit strict TypeScript command
after separate RAM/process preflights. No source implementation or schema edit is requested.

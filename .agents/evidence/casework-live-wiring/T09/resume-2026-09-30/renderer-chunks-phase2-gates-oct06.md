# Renderer chunk Phase 2 verification record

Date: 2026-10-06. The focused fixture gate and strict typecheck passed. The
canonical UI gate completed frozen install, typecheck and production build,
but the full test suite ended with two failures; therefore the canonical gate
is **not green**. This record does not claim browser downloads, current ladder,
live behavior, or T09 acceptance.

## Preflight identities and resources

All four preflights met the assigned minimums of 1,258,291,200 available RAM
bytes, 536,870,912 free swap bytes and sufficient `/tmp` capacity. Every
preflight captured all three accepted source identities below, package/lock
identities, and the applicable tool versions. No source, fixture, package, or
lock identity changed.

| Input | SHA-256 (all four preflights) |
|---|---|
| `src/build/rendererChunkContract.ts` | `52cdf00b9650f0413c91e4375c0ce5b6527b63ad9f09219f2102383dca4dec64` |
| `vite.config.ts` | `95c287317aa4028a1976c145124f918d0d856a3bb0ca60802f31abd909e901ea` |
| `src/build/rendererChunkContract.test.ts` | `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08` |
| `package.json` | `85044b7b0619f390791f87d089a84463610814cc199f705662207f00337e2f54` |
| `bun.lock` | `9e3725c9ebd21566fb6499fe1df6f2fecf7230307b489dc6700ea768d7e39744` |

Bun was `1.4.0`, Node `v26.8.2`, and Just `1.58.0` where applicable.

| Gate preflight | Available RAM | Free swap | `/tmp` available |
|---|---:|---:|---:|
| Focused test | 3,578,413,056 B | 2,818,125,824 B | 1,360,941,056 B |
| Strict typecheck | 3,270,176,768 B | 3,069,808,640 B | 1,360,928,768 B |
| Canonical initial attempt | 3,057,664,000 B | 3,245,916,160 B | 1,359,552,512 B |
| Canonical retry | 3,022,532,608 B | 3,259,895,808 B | 1,359,540,224 B |

## Serialized gates and joined outcomes

1. From `apps/godspeed-cognitive-ui`, `bun test
   src/build/rendererChunkContract.test.ts`: **13 pass, 0 fail, 35
   expectations, joined exit 0**.
2. From the UI app, `bun run typecheck`: **`tsc --noEmit`, joined exit 0**.
3. Initial root `just casework-ui-check` attempt: **joined exit 1 before the
   recipe started**. Just could not create
   `/run/user/1000/just/just-KROsWR` because that path was read-only. The exact
   failed attempt was preserved. Under root's bounded authorization, the
   canonical retry changed only Just's scratch directory:
   `JUST_TEMPDIR=/tmp just casework-ui-check`.

The retry completed `bun install --frozen-lockfile` (`Checked 83 installs
across 133 packages (no changes)`), `bun run typecheck`, and the production
Vite build. The build printed
`[renderer-chunk-contract] verified nine distinct renderer chunks` and emitted
separate renderer JS files. The subsequent full Bun suite ran 276 tests across
22 files: **274 pass, 2 fail, 1456 expectations, joined recipe exit 1**.
Failures were:

- `journey: execute + execution pill + sentry > TOOTH: SSE drop flips the
  store to reconnecting via the live adapter error path, and an event resumes
  it`: Bun reported `EPERM: operation not permitted, listen` at the local
  `Bun.serve` setup in `src/ui/journeys.test.tsx:352`.
- `LocalContractAdapter > shared CaseworkPort behavioral conformance`: the
  future-only subscription check observed repeated cursor
  `1.0000000009` after head `1.0000000008`.

These failures were not rerun or diagnosed within this bounded task. The
canonical gate remains red; no claim is made that either is a baseline issue.

## Emitted artifact inspection

The build log contains the success marker above and lists the nine separate
files below. The emitted main bundle is
`apps/godspeed-cognitive-ui/dist/assets/index-CdN8EEXV.js`; its SHA-256 is
`7a322ddffd541ed0746973c123c74a654aa90b3de9d085a356166aba5b3d3ee9`.
Read-only inspection of that artifact found these nine `import()` references:

```text
import("./DiffRenderer-C7B9rAkF.js")
import("./TextRenderer-Zvs8B1j7.js")
import("./MarkdownRenderer-OzOvJShT.js")
import("./TableRenderer-DOQo_-4h.js")
import("./ChartRenderer-B_UWivW0.js")
import("./JsonRenderer-DL4mVUJ5.js")
import("./GraphRenderer-BEtn_yge.js")
import("./TraceRenderer-IX8LTwXi.js")
import("./TimelineRenderer-C71VGLCP.js")
```

A targeted static-import search for these renderer filenames in the main
bundle returned no matches. The successful build hook also means the actual
Rollup module map passed the checker: each required renderer source module
was found once, in a distinct dynamic-entry chunk linked by the registry.
Together with the dynamic `import()` references and separate artifacts, this
shows the renderer modules were not statically placed in the main chunk for
this build. It does not show that a browser fetched any of them.

| Emitted artifact | SHA-256 |
|---|---|
| `dist/assets/DiffRenderer-C7B9rAkF.js` | `ce98065c44dfab1cdbb2b1f1ad9e4fa2e7d58f9affee3d7ac5492268c9b8445b` |
| `dist/assets/TextRenderer-Zvs8B1j7.js` | `a697a5f37a8460b6fea3e2c9f69bb5116248ba8332612f9614018a7916b4eb6c` |
| `dist/assets/MarkdownRenderer-OzOvJShT.js` | `5ef55f3f49cdcfe871d9f1111037d043ca8aeafee89d10e1e64101615261ea8f` |
| `dist/assets/TableRenderer-DOQo_-4h.js` | `ff85b7f2cef0f1373f9232ccc008dbf83d4cb01d9a0c66713f880fecd91ee026` |
| `dist/assets/ChartRenderer-B_UWivW0.js` | `e68ed628a315927d447c87628319690a27fe35086a9fcd49188d6cf537c7ec35` |
| `dist/assets/JsonRenderer-DL4mVUJ5.js` | `0bbc0739642ceba258296195923eb81f0cb178784b2b423265cb26e65f99a76d` |
| `dist/assets/GraphRenderer-BEtn_yge.js` | `f75009e8056bc7fb1f04020232a76efaeed308bc0da979c92c12e0c53f02a5a4` |
| `dist/assets/TraceRenderer-IX8LTwXi.js` | `622bdb32f09bfa16823f45c5de82cd14c899c4d16d44fadb91db0284fbd0f54c` |
| `dist/assets/TimelineRenderer-C71VGLCP.js` | `49df77263eb0f3e2efcf12820495c379c4b7d525f8c7e58eefec33597abc84af` |

## Immutable captures

Every preflight, raw command output, and joined exit was captured under a
unique `/tmp/renderer-chunks-phase2-*.oct06.*` directory and archived under
this evidence directory. Each archived file was byte-compared with its
original using `cmp -s`; all comparisons returned exit 0.

| Gate attempt | Preflight SHA-256 | Raw SHA-256 | Exit SHA-256 |
|---|---|---|---|
| Focused | `2ea590757d2edd3e61d5dcc2db9ac4db9136a924645bc398982d03859f7214bd` | `05bc99cfddf5d3bd1cfa3b0ac26dd26db2d87475872be11868b2e03d3f79ab15` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| Typecheck | `729748eb23c2ccc35d4f6ed2e5ce94d43c01d05e77e12aeb94b56d403972fad3` | `8366207267355d3e3d5bf3bf6e8c94c5f93f6078c34f08973fa2b38cdda6cc92` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| Canonical initial attempt | `f887c3e5ee990b249fcfabc17ef632515b17171d9eced7b78410f7e445582e5c` | `9ae1a980548a3a9052a00f3db51c4460c950089a238c7ba7ca057bc2d026b5b4` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |
| Canonical retry | `a6613867397764bfef8f6597f013b146e33f6eafb7ff1b856b677a385ad090a0` | `85ba0b92ccb4a69fda5cdb0d1750e3df5c43d656a5cf4fe4a49c9eb7ce6edaed` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

No source, test, package, lock, Git, status, or debt file was edited. The
canonical test failures and the lack of browser/runtime evidence remain open.

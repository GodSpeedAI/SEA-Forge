# Renderer chunk serialized verification assignment

Date: 2026-10-07. Root has released the sole heavy/compiler token for this
bounded verification. This assignment authorizes exactly these sequential
gates against the already source-approved three-file renderer unit:

1. `bun test src/build/rendererChunkContract.test.ts`
2. `bun run typecheck`
3. `bun run build`
4. Repository-root `just casework-ui-check`

For the first three, run in `apps/godspeed-cognitive-ui`. The canonical recipe
is defined in root `justfile`; it performs frozen install, typecheck, build,
and the full Bun test suite. It runs from repository root. Do not edit source,
tests, package/lock files, Git state, or other product files. Build outputs are
permitted. Do not overwrite prior evidence.

Before **each** gate, capture one actual preflight to a new unique `/tmp`
original containing current SHA-256 values for all three files below, actual
available RAM and free swap, `/tmp` free space, cwd, and relevant tool versions.
Proceed only if available RAM is at least 1,258,291,200 bytes and free swap is
at least 536,870,912 bytes. The required source identities are:

* `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.ts` —
  `52cdf00b9650f0413c91e4375c0ce5b6527b63ad9f09219f2102383dca4dec64`
* `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.test.ts` —
  `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08`
* `apps/godspeed-cognitive-ui/vite.config.ts` —
  `95c287317aa4028a1976c145124f918d0d856a3bb0ca60802f31abd909e901ea`

Capture combined stdout/stderr and the joined process exit as separate actual
files in that same unique `/tmp` directory. Immediately after each gate joins,
copy its preflight, raw output and exit into new, gate-specific names under
this evidence directory and byte-compare all three archived files with their
`/tmp` originals before starting the next gate. Preserve failed attempts and
never transcribe, normalize, or repair output bytes; if a wrapper has no final
LF, preserve that exact byte sequence.

For the canonical recipe, preserve the first sandbox result. If its local
listener is denied with `EPERM`, a retry may use the authorized local-only
execution escalation described by root, but it must have its own fresh
preflight with all three identities/resources and its own raw/exit pair. Do
not discard or replace the first attempt. Report each attempt and its actual
joined exit.

The review may accept source approval independently from gate outcomes. Report
the focused case count, typecheck result, build marker and nine distinct
emitted renderer artifacts when present, and canonical full-suite totals and
failures. A passing renderer build does not establish browser downloads,
ladder behavior, live integration, or T09 settlement. This verification does
not amend the earlier renderer checkpoint scope: the three renderer files
remain separate from cursor checkpoint `f549bf0`.

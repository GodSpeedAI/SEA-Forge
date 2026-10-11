# Append-only clarification to the unit 4 approval

Date: 2026-10-01. This supplements, and does not modify, `run-children-phase2-independent-approval.md` (SHA-256 `8b1432ff9d846512d5269bd0e87cada9256b0d84d50004893d5cfbeaa7fbc198`). The bounded unit 4 approval remains in force; full T09 remains pending.

## Clarified behavioral descriptions

The approval's sentence “It makes no action request” was imprecise. The live proof DOES call `Authority.ExecuteItem` to execute the two governed `task_prepare` items and confirm settlement. The projected run-child objects expose no actions; the proof makes no action request on a projected child.

“Bounded child collection” describes the source-level observer hydration cap only. The pure fixtures check all 24 execution/settlement pairs in their matrix, and the live proof checks the actual children returned for the two committed cases. The assertions do not imply that every run list is capped at eight children; the eight-item cap applies to observer hydration, a separate path.

## Original and archived gate artifact paths

The following complete path pairs identify each run's original output, exit, and immediately preceding host preflight, and its separately archived counterparts under `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/`. Hashes refer to exact bytes; output hashes are also in the original approval manifest.

| Gate | Original output → archived output | Original exit → archived exit | Original preflight → archived preflight |
|---|---|---|---|
| Focused | `/tmp/t09-unit4-fix-focused.raw` → `unit4-fix-focused.raw` | `/tmp/t09-unit4-fix-focused.exit` → `unit4-fix-focused.exit` (SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`) | `/tmp/t09-unit4-fix-focused-preflight.raw` → `unit4-fix-focused-preflight.raw` (SHA-256 `60de62b32583346c8e2914778fb3c72f8b42268e5ce0434ee9a260d4034157fc`) |
| Live kernel | `/tmp/t09-unit4-fix-live.raw` → `unit4-fix-live.raw` | `/tmp/t09-unit4-fix-live.exit` → `unit4-fix-live.exit` (SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`) | `/tmp/t09-unit4-fix-live-preflight.raw` → `unit4-fix-live-preflight.raw` (SHA-256 `3c69c3245ba2e110580b600d30178e2566fead2d615a536439dbd1074bb50f66`) |
| Canonical | `/tmp/t09-unit4-fix-canonical.raw` → `unit4-fix-canonical.exact.raw` | `/tmp/t09-unit4-fix-canonical.exit` → `unit4-fix-canonical.exit` (SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`) | `/tmp/t09-unit4-fix-canonical-preflight.raw` → `unit4-fix-canonical-preflight.raw` (SHA-256 `57017d5a2114c9504a89201250c1a05b190c9c380be8e92ce3044db3cbe271ee`) |
| Full Go race | `/tmp/t09-unit4-fix-fullrace.raw` → `unit4-fix-fullrace.exact.raw` | `/tmp/t09-unit4-fix-fullrace.exit` → `unit4-fix-fullrace.exit` (SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`) | `/tmp/t09-unit4-fix-fullrace-preflight.raw` → `unit4-fix-fullrace-preflight.raw` (SHA-256 `5c02178547eaf3fe6ddf47ed572bf00fa252609009b4f83c5f435d1a2fb293c5`) |
| Supplemental live capture | `/tmp/t09-unit4-fix-live-capture.raw` → `unit4-fix-live-capture.raw` | `/tmp/t09-unit4-fix-live-capture.exit` → `unit4-fix-live-capture.exit` (SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`) | `/tmp/t09-unit4-fix-live-capture-preflight.raw` → `unit4-fix-live-capture-preflight.raw` (SHA-256 `41058c1e39b1e8b1ae986a4f73a54344b928f739d31978038786c3482b690b15`) |

For completeness, source-byte identities are the 11 full SHA-256 values listed in the frozen-source table of the approval. They were obtained from `sha256sum` on the named files, not inferred from Git state or test output. The supplemental run's path sidecar is `/tmp/t09-unit4-fix-live-capture-path.raw` → `unit4-fix-live-capture-path.raw`; the watcher capture is `/tmp/t09-unit4-fix-live-server.raw` → `unit4-fix-live-server-watcher.raw`. Exact hashes and the `cmp` results are recorded in the approval.

## Historical evidence-preservation limitations

The unit 4 approval does not claim an unbroken immutable archive history for earlier Phase A artifacts. `run-children-phase-a-initial-archive-recovery-erratum.md` and `run-children-phase-a-root-recovery-check.md` document that an earlier archive step overwrote two initial log copies with incorrect transcriptions. The sandbox incorrect copy was recovered and retained as `run-children-phase-a-sandbox.initial-archive-mismatch.log`; the host incorrect copy with reported SHA-256 `697abd30fc2344adb383a4e4eb230fc3851ea7d8f762b82fb9b278996015e002` could not be recovered. During recovery, a newly transcribed incorrect recovery artifact was also removed before a matching copy was created; its intermediate bytes are unavailable. The exact original command outputs remain present and were independently matched to their `/tmp` originals, so these limitations do not change the executed gate results. They remain archive-history limitations.

There is also an earlier live fixture snapshot transcription error. The first snapshot `run-children-live-import-cycle-original.go.snapshot`, SHA-256 `2c982cb81ebefab5be1877310bc76bf12f88ed8243fc550181ccb728dceb3c1f`, remains retained as incorrect; `run-children-live-package-root-source-check.md` records the one diagnostic-argument difference. The separate corrected snapshot `run-children-live-import-cycle-original.corrected.go.snapshot`, SHA-256 `8e485c64cdc0776d46a8fda6678d22f3bfe98ec837bd8eaaeb94fecced8a9134`, matches the original. This is a snapshot transcription repair, not a production or test assertion change.

The canonical and full-race one-space archive transcription errors for the final gates are likewise preserved as their initial bad artifacts, with corrected exact copies and their erratum in `unit4-fix-archive-transcription-erratum.md`. The actual original command outputs are retained and independently byte-compared against the exact new-path copies. These known capture-process failures are disclosed; they are not being treated as passing evidence or as waived verification failures.

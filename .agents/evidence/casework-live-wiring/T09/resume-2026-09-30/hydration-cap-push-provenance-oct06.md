# Hydration-cap push evidence provenance

Date: 2026-10-06  
Scope: archive index and provenance summary only. These records preserve captured command output; this note does not certify a source change, implementation, runtime behavior, or approval.

## Preserved raw captures

The five files below were added from their named `/tmp` originals without altering bytes. Each archive was compared with `cmp`; every comparison returned 0. The SHA-256 values match for each source/archive pair.

| Capture | Archived file | Bytes | SHA-256 |
|---|---|---:|---|
| Checkpoint command output | `hydration-cap-checkpoint-run-oct06.raw` | 10,196 | `4c9ee6713ab8aba5689d5c9a5fdb6b7f62b02d2d5ca520d9997e3adb2d2125b7` |
| Checkpoint exit output | `hydration-cap-checkpoint-exit-oct06.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| Push preflight output | `hydration-cap-push-preflight-oct06.raw` | 228 | `7769e4a5f68a4899256b058f0891893e1c5342deeb38a65abbee56bc59e55ee3` |
| Push command output | `hydration-cap-push-run-oct06.raw` | 237,577 | `a3d3703687aaabf38a298968c285a5c06506cf649b576ce57f875635c6dd32c7` |
| Push exit output | `hydration-cap-push-exit-oct06.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |

No checkpoint-preflight original was supplied. No such capture is inferred or fabricated.

## Captured run metadata

- Checkpoint command/session: `78326`.
- Commit/push command owner session: `37401`; recorded join count: `0`.
- Captured CI summary: 125 suites, 1,110 passed, 0 failed, 4 ignored.
- Captured history/leak summary: 511 commits checked, no leaks reported.
- Remote observation: root's separate tool observed exact remote value `10067`. This is recorded as a **root observation**, not as a value observed by this archival worker or an execution performed by this worker.

The raw captures are authoritative evidence for the output they contain. The summary above records supplied provenance metadata and makes no new runtime or source claim. No feature/source approval, production approval, or broader verification approval follows from these captures.

## Verification boundary

The archive checks performed here were byte comparison and SHA-256 only. No tests, compiler, scanner, gate, Git, network, source, status, or debt operation was run by this worker.

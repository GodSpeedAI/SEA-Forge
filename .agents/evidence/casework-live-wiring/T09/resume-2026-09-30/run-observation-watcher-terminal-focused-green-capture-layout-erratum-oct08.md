# Focused verification capture-layout erratum

Date: 2026-10-08

This additive note clarifies the capture layout for
`run-observation-watcher-terminal-focused-green-result-oct08.md` (SHA-256
`dab62acbba5d783b4051dd45dd691c8e5a2fdff95f7bb32f7b320bbbb3e24404`). The
execution instruction requested a combined stdout/stderr capture, alongside
command, preflight, and test exit. The actual shell invocation redirected
stdout and stderr into separate original files, and also captured the
preflight exit separately. Therefore six original/archive pairs were created:
command, preflight, preflight exit, stdout, stderr, and test exit.

The stdout original is `/tmp/sea-observation-watcher-terminal-focused-green-etaBTw/stdout.raw`
(14,570 bytes, SHA-256
`29b3faee979d7922009ed73c1d4a682030690e01fbb52e1d827c32a7e978b8b6`). The
stderr original is `/tmp/sea-observation-watcher-terminal-focused-green-etaBTw/stderr.raw`
(0 bytes, SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`). Their
separate archives are
`run-observation-watcher-terminal-focused-green-etaBTw-stdout-oct08.raw.json`
and
`run-observation-watcher-terminal-focused-green-etaBTw-stderr-oct08.raw.json`.
Both were decoded and byte-compared with their original files; both
comparisons were true. The test process exit and observed results in the
result receipt are unchanged. This records an archive-layout deviation; it
does not combine or rewrite the preserved output streams.

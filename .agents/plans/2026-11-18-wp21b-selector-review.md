# WP-21B native selector review

Both passes used a fresh Codex Terra session with
`codex -a never exec -s read-only -m gpt-6.1-terra` against a detached
checkout. Each packet contained the full reviewed diff, the Field Case
storage specification, and native test evidence. Both reviewers exited zero.

| Pass | Frozen commit | Critical | High | Medium | Low |
| --- | --- | --- | --- | --- | --- |
| 1 | `1e1761ee92d61aec1a810a92b3a99ca68d7860e7` | 0 | 0 | 2 | 0 |
| 2 | `fd843472230b7edba4ad900abb8a244113202b9b` | 0 | 0 | 0 | 0 |

Pass 1 found that scheduling could let every reader miss the selector
rename. The follow-up pauses the real selector reader with the prior handle
open, waits for publication, then verifies the complete old generation. A
fresh reader verifies the new generation.

Pass 1 also found that prior error injection occurred after the Windows helper
had already finished. The follow-up injects at its pre-rename, post-rename
identity, and post-rename parent-flush boundaries. The fixed-NTFS test checks
visible selector bytes and error classifications. The staged publisher test
checks the conversion to publication uncertainty.

The native Windows 10.0.26300 fixed-NTFS run passed the helper test and all
14 staged library tests. Code review and these native tests cover the
trusted namespace and staged publication contracts. OS-reset evidence
currently covers Linux/ext4. [Portability evidence](https://github.com/GodSpeedAI/SEA-Forge/actions/runs/44423954384)

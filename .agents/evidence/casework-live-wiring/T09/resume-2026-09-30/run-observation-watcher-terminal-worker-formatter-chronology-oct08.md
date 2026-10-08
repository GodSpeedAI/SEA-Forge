# Worker formatter chronology clarification

Date: 2026-10-08

The retained worker pre-format source JSON contains SHA-256 `5deebf4b629e24356b2a79dfd95388d9b2d69864d608fc3efdf8c6e2fb239bbe` (13,243 bytes). The worker formatter witness instead declares its input as SHA-256 `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` (13,217 bytes). Those are not the same pre-format source.

Between that source capture and the formatter witness, the source cleanup removed the redundant `|| entry.ctx.Err() != nil` from the `!validWatcher` condition in `runPollerFromClaim`; the existing separate context check immediately before the trace-port call remains. The gofmt witness therefore represents the post-cleanup source, rather than a direct transformation of the retained pre-format JSON. The post-cleanup source SHA matches the captured gofmt stdout and final source exactly. No original capture is reconstructed, and no source is changed by this clarification.

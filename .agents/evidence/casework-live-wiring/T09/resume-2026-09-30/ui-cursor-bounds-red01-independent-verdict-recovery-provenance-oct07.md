# UI cursor bounds RED01 verdict recovery and provenance

Date: 2026-10-07.

The first verdict was written at `ui-cursor-bounds-red01-independent-verdict-oct07.md` and reported as SHA-256 `53a6a5df77473a6d305656b4a3826a5848330366edb4ed01e00b528afeaac048`. A later edit to that same path added the fresh preflight resource sample; that current file now has SHA-256 `36ef5054883ccff001c8df92b9b3b232a5382bf13f2f516fb0bef2017bebb3f1`.

The first native AddFile payload was recoverable from this agent's tool-call history. It has been preserved verbatim at `ui-cursor-bounds-red01-independent-verdict-prior-written-recovery-oct07.md`; its verified SHA-256 is `53a6a5df77473a6d305656b4a3826a5848330366edb4ed01e00b528afeaac048`, matching the originally reported hash. A byte diff against the current verdict shows exactly one paragraph change: the current verdict adds the preflight resource figures (4,051 MiB available memory and 9,203 MiB free swap). All other bytes are identical.

The recovered artifact preserves the original verdict version. The current verdict remains as written. This provenance note does not revise either verdict or add runtime/source claims.

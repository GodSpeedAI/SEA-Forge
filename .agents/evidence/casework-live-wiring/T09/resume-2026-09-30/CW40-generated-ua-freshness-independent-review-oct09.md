# Independent review — CW-40 generated graph freshness debt

**Disposition: REJECT CW-40’s next-step wording pending one narrow correction.** The evidence and distinction between `.ua` and Graft are accurate. The next step should not imply that routine `.ua` refresh requires separate operator authorization.

## Direct evidence checked

- CW-40 in `.agents/DEBT.md` records commit `9efd039e47ee095c1e6ac4a7b9d0f99fcb89beaa`, the stale `.ua` hook message, and the successful Graft refresh.
- Decoding `initial-lease-checkpoint01-stderr-oct08.raw.json` yields the final two hook lines: the commit changed source files and `.ua` is stale; the hook requests the Understand incremental refresh.
- `initial-lease-graft01-command-oct08.raw.json` decodes to `graft build`; its stdout records 7,901 nodes, 15,655 edges, and 710 cards; its captured exit is `0`. This proves Graft refreshed successfully and does not refresh `.ua`.
- The root instruction prohibits hand-editing `.ua` or treating it as source. It does not make a routine supported Understand refresh an operator-approval gate. The grant prohibited this reviewer/builder from running Understand; it said `.ua` refresh need not expand this milestone.

## Finding

CW-40’s status, evidence, and impact correctly say `.ua` remains stale, distinguish it from current Graft output, and avoid a hand-edit. Its next line says “refresh `.ua` through the understand workflow in a separately authorized task.” “Separately authorized” may create an unsupported approval prerequisite. The originating restriction is scoped to this debt-recording task and does not establish that future routine refresh requires operator authorization. Change the next step to “refresh through the supported Understand workflow when in scope; do not hand-edit generated graph files or infer `.ua` freshness from Graft.”

No debt text was edited by this critic. No Understand/Graft command, compiler, test, gate, or Git operation was run.

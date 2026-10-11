# Independent review — CW-40 wording repair

**Disposition: APPROVE CW-40.** The repair removes the unsupported future-authorization implication and replaces the temporary absolute path with the canonical archived capture name. The entry still distinguishes the stale `.ua` graph from the successful Graft refresh and does not claim `.ua` was refreshed.

## Evidence checked

- `.agents/DEBT.md:1497–1508` is the reviewed CW-40 text.
- `initial-lease-checkpoint01-stderr-oct08.raw.json` is the canonical archived capture. Its decoded final lines report that commit `9efd039e47ee095c1e6ac4a7b9d0f99fcb89beaa` left `.ua` stale and request the incremental Understand refresh.
- `initial-lease-graft01-command-oct08.raw.json` decodes to `graft build`; the archived stdout reports 7,901 nodes, 15,655 edges, and 710 cards, and the archived exit is `0`. This is Graft evidence only, not `.ua` refresh evidence.
- The prior rejection `CW40-generated-ua-freshness-independent-review-oct09.md` asked for the exact two repairs now present: use the relative archive name and say refresh when in scope/needed without requiring a separately authorized task.

## Disposition details

CW-40 now says the supported Understand workflow may refresh `.ua` “when in scope or needed.” It still prohibits hand-editing generated graph files and warns that Graft freshness does not establish `.ua` freshness. The evidence path is relative to the repository’s evidence directory, and the entry accurately leaves `.ua` refresh deferred. No unsupported operator-approval prerequisite or runtime/proof claim remains.

No DEBT, `.ua`, source, status, or other evidence file was edited by this critic. No Understand/Graft command, test, compiler, gate, or Git operation was run.

# Current status

**Status revision:** 160

**Stage:** casework-live-l5-design-click

**Summary:** The live ladder's design helper retries the Design case click until the template picker opens. Main's scheduled Casework live run on 17253b0 failed L5 with 'Timed out waiting for templates listed', and SEA-Forge #48 failed the same step on all three attempts. The gateway log of the main run shows no GET /api/templates during L5 (L1 made two), so the openProposals handler never ran: the center chips are re-created on every scene update while the L4 case is still settling, and the click landed on a replaced chip. The templates assertion itself is unchanged.

**Verified:**

- Main run 39921072860 (job 120306162746) evidence: L5 step 2 failed after 25.5 s; gateway.log between 08:01:18 and 08:01:48 has only GET /api/session; L1 at 07:58:48 shows two GET /api/templates.
- Store reducer: openProposals always resets proposal to status loading, which triggers getTemplates; a request-free L5 therefore means the click handler never ran.
- The edited helper parses (bun build --no-bundle); the retry waits on window.__gs.store.getState().proposal and keeps the >= 2 proposal-template assertion.

**Limits:**

- Not reproduced locally: the full live ladder needs more memory than this host has spare; proof is the Casework live check on this PR.
- The diagnosis rests on the absence of the template request; no L5 failure screenshot exists in the evidence bundle.

**Next move:** Merge after the docs PR (status revision 159); then SEA-Forge #48 merges main and renumbers its status revision.

**Evidence:**

- `apps/godspeed-cognitive-ui/e2e/journeys-live/helpers.ts`
- `apps/godspeed-cognitive-ui/src/model/store.ts`

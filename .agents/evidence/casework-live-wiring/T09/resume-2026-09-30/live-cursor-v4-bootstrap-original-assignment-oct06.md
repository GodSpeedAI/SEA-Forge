# V4 bootstrap component proposal — original builder assignment

Date: 2026-10-06. Builder: Luna live_cursor_v4_boundary_recon.
DOCONLY; public implementation and full V4 approval remain HOLD.

Write one NEW `live-cursor-v4-bootstrap-concrete-proposal-oct06.md`.
Root architecture direction:

- Preserve ready GET `/api/world` response `{snapshot: ordinary
  CognitiveWorldSnapshot}`. Author a mutually exclusive `{bootstrap: {...}}`
  arm with exact fields: `status: "no_cases"`, RFC3339 actual capture timestamp,
  existing `ActorPerspective`, and `templates: TemplateEntryOption[]` mapped
  from existing `case.entry_options`. No new kernel verb; no ordinary snapshot,
  history, world_id, case_id or cursor in bootstrap.
- Specify authored JSON Schema oneOf response arms with additionalProperties
  false, concrete Go/TS variants and generated-source mapping. Edit no generators
  or generated files for this proposal.
- Bootstrap UI state is outside WorldHistory/cursor caches. Preserve usable
  template, preflight and PROPOSE_CASE flow. Explicit create-only empty case_id
  and client_cursor exception retains authentication, CSRF, idempotency,
  preflight and governance. After creation, use returned real case ID, await a
  validated ready ordinary snapshot, then history/focus/scoped streaming.
  Never manufacture a cursor or revision.
- Explicit case query never redirects. True no-case success requires corrected
  fail-closed inventory and complete-empty proof, currently absent and held.
  Distinguish loading/unavailable from true empty. One outstanding authenticated
  refresh, no faster than one second, abort/dispose correctly; no empty SSE.
- Inspect exact templates types, entry UI, intent validation and dispatcher.
  Enumerate compatibility changes, races, failure cases and test-first matrix.
  No unexplained gaps or claimed kernel frontier.

This component feeds full V4. IDs/cursors, inventory/frontier and Store changes
remain separate HOLD. Preserve old documents. Read applicable instructions and
use Graft first. No source, tests, status or debt edits; no tests, compiler,
scanner, Graft build, Git or network commands. Root owns sole heavy token for
push37401. Return path/hash and every material divergence from this assignment.

An independent critic must receive this original assignment plus the complete
result and cite actual source/schema evidence. Approval of a component document
does not approve the public V4 contract or release implementation.

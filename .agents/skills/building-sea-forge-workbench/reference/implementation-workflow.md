# Implementation workflow — vertical slices

The unit of work is a **vertical slice**: one user-visible settlement proven
end-to-end (server → bridge → view model → route → tests), never a horizontal
layer built "for later".

## The loop (execute in order, every slice)

1. **Identify the user-visible settlement.** One sentence: what can the
   operator now see/do/verify that they could not before?
2. **Inspect current repository implementation.** Start from
   `repository-integration.md`; verify the rows the slice touches are still
   true (`file:line`).
3. **Load only relevant source contracts** via the source-map — the screen's
   wireframe section, the route's view-flow section, the methods' catalog
   entries. Not the whole package.
4. **Map target concepts to actual types and modules.** Every noun in the
   spec gets a repository symbol or an explicit gap.
5. **Classify each capability**: `EXISTS` / `PARTIAL` / `MISSING` /
   `CONFLICTS` / `UNKNOWN` / `NOT REQUIRED` — with evidence.
6. **Select the smallest coherent vertical slice.** If the slice needs three
   new subsystems, it is too big; shrink the settlement.
7. **Implement missing server/query/command/event support only where
   genuinely absent** — additive verbs per ADR-003, thin adapters over shared
   services (mirror `crates/sea-forge-cli/src/commands/` + the `Ask` verb
   pattern in `sea-forge-server/src/lib.rs`). Never fork governance logic
   into the server handler.
8. **Generate or adapt typed frontend contracts** from the Rust types. No
   handwritten duplicate enums when generation is feasible.
9. **Implement the Tauri bridge** — typed command or channel per the bridge
   policy in `stack-and-dependencies.md`.
10. **Implement view model and XState machine where needed** (lifecycle /
    recovery flows only; plain queries need no machine).
11. **Implement route and Astryx-based components** per
    `design-and-ux-contract.md`.
12. **Wire source-backed queries and events** — TanStack Query for reads,
    event stream reduction for liveness, authoritative refetch on doubt.
13. **Add failure and recovery handling**: denial, escalation, stale
    rejection, transport ambiguity → `request.get_status`, event gap →
    `events.get_range` + refetch, disconnect → reconnect machine.
14. **Add tests** per `testing-and-settlement.md` (bridge, contract, machine,
    component, route, a11y as applicable).
15. **Validate visual fidelity and accessibility** against the wireframe
    section and `DESIGN.md`; run axe checks; keyboard-only pass.
16. **Run repository checks**: `devbox run -- just check && devbox run --
    just test` (Rust touched) and the Bun workspace checks (frontend
    touched). Keep spec-minimum proofs green after server changes.
17. **Report evidence, limitations, and next spendable path** per the SKILL.md
    completion contract; update `.agents/CURRENT_STATUS.md` at handoff.

## Order within a slice

Route (URL + guards + search state) → controller/view model (queries,
machine, event reduction) → components (organism down to atoms) → tests
alongside each layer, not after all layers.

## Local draft vs canonical state

| Frontend may own | Frontend must never own |
|---|---|
| Navigation, reversible local drafts, presentation prefs, filtering/sorting, keyboard interaction, formatting, non-authoritative completeness hints, cached views with explicit freshness, pending-request presentation | Authority evaluation, identity/sponsor resolution, sentry activation, canonical case reduction, execution truth, settlement evaluation, declaration standing, capability promotion, integrity verification, agent permission grants, artifact maturity, federation adoption, canonical record mutation |

The normal frontend path never edits `.sea-forge/` files. Drafts are
reversible and clearly non-authoritative; a draft becomes real only through
preflight → commit.

## Dependency direction

```text
renderer components → view models → typed bridge (generated) → Tauri host →
socket client → sea-forge-server → kernel crates
```

Nothing points the other way. Kernel crates never import frontend concepts;
the server never shapes responses for one component's convenience (it shapes
**views** that any consumer may read); generated contracts flow Rust → TS,
never TS → Rust.

## Invariants (verbatim; violating any one fails the slice)

```text
navigation ≠ mutation            draft ≠ proposal
proposal ≠ committed plan        preflight allow ≠ execution authority
approval ≠ successful execution  execution state ≠ settlement state
agent dialogue ≠ termination     agent termination ≠ settlement
settlement acceptance ≠ capability promotion
derived view ≠ source truth      event delivery ≠ source truth
imported record ≠ local authority
unknown ≠ unavailable            restricted ≠ empty
denial ≠ application failure     retry ≠ replay
```

## Prohibited shortcuts (hard fails in review)

Renderer mutation of canonical files; frontend-owned lifecycle truth;
frontend authority/settlement logic; duplicate handwritten canonical enums
when generation is feasible; broad retrieval followed by client redaction;
optimistic approval/cancellation/settlement/integrity/capability; silent
provider fallback; silent event loss; retry after transport ambiguity without
status recovery; generic status or success contracts; speculative backend
endpoints; direct renderer Unix-socket access or SQL; GraphQL as primary API;
treating the static Readiness kit as a complete application; flattening
domain modules into generic cards; changing SEA Forge semantics to simplify
UI implementation.

## Definition of done (per slice)

- The named settlement is demonstrable with a copy-pasteable command or
  scripted interaction, and its test fails when the behavior is broken.
- All capability classifications recorded in the task report with evidence.
- Rust gates green (`devbox run -- just check`, `just test`); frontend gates
  green (`bun run check`, tests); spec-minimum proofs unaffected.
- No invariant weakened; no prohibited shortcut present; no new dependency
  without its ADR.
- Storybook fixture exists for each new component state that reviews care
  about; a11y checks pass for new surfaces.

## Debt prevention

- Extend shared services, never fork per-client variants of governance logic.
- One producer per contract: schema generation is the only source of TS
  domain types.
- Every `PARTIAL` classification either becomes a follow-up task in the plan
  or an entry in `.agents/OBSERVED_DEBT.md` — never silence.
- Prototype code (`ui_kits/app/*.js`) is never imported, only read.

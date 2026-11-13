# D. Reference capability mapping

Inputs: `.tmp/reference/{palantir-foundry,palantir-ai-fde,nvidia-auto-ontology}/{recon.md,features.csv}`; digest
with line citations in `recon/07-reference-digest.md` (IDs `PF-`, `AF-`, `NV-` below are defined there).

## 1. Source confidence (applies to every row)

The reference material is a secondhand reconstruction, not an observation of the products:

- No feature row carries an observed-versus-inferred marker; all three `screens/` folders are empty.
- Data-model entities sit under a heading "Inferred" while marked `confidence: high`.
- Stated feature counts disagree with the CSVs in all three; AI FDE click counts do not match their own steps;
  the NVIDIA benchmark deltas cite no run.

Therefore every reference behaviour below is rated **R** (reported by the reconnaissance, unverified against the
product). Nothing in the design depends on a reference claim being true: each row is adopted because the *job*
is required by the SOP or Mission, and the reference only suggests an affordance pattern. No reference feature is
a requirement.

Decision column: **ADOPT** (in the baseline, mapped to a requirement), **HAVE** (GodSpeed already does it),
**LATER** (useful, outside the completion baseline), **NO** (not our job).

## 2. What makes the references feel complete (mechanisms worth copying)

| Mechanism | Seen in | GodSpeed-native form |
|---|---|---|
| Everything is an addressable object with one dossier view | Foundry object 360 (PF-24) | the inspector over any record id (B §2) |
| Pivot along declared relations only | search-around (PF-23) | Explore graph follows `.sea` relations; `impact` refuses undeclared relations (already built) |
| Governed mutation as a first-class object with audit | Action Types (PF-18, PF-20) | SEA-Forge governed operation + ledger; decision card |
| Unusable actions hidden or explained by permission | PF-30 | *explained, not hidden*: blocked move shows reason and authorizer (`SOP §1.2`) |
| Every consequential step is a card in the stream with a fixed state set | AI FDE plan, approval, diff, clarification cards (AF-11, AF-12, AF-25, AF-26) | Decide's sequential cards; freeze card; panel blocks `ask`, `guard`, `read` |
| Approval scope is explicit and stored | once / session / branch (AF-21, AF-23, AF-24) | SEA-Forge approvals and bounded grants; P0–P3 classes |
| Context is pinned, visible and stateful | context ribbon (AF-15, AF-16) | case strip + assignment brief "context sources allowed" |
| Cost is visible per turn with thresholds | AF-05, AF-30 | Cost view, per-currency, `unknown` when unmeasured |
| Draft versus certified status on every term; answers link to provenance | NVIDIA (NV-06, NV-15) | assertion status `declared … unknown`; value labels; `why` |
| Readiness is a visible, queryable state | NV-19 | SEA-Forge `readiness.get`; "not connected" state |
| Every screen enumerates empty, loading, error, permission states | all three | B §4 state set |

## 3. Capability mapping

### Semantic world

| Ref | Job | GodSpeed capability today | Remaining work | Decision | Test |
|---|---|---|---|---|---|
| PF-13–16 object, property, key, link types | define entities and relations with stable identity | `.sea` entity, resource, relation, `ConceptId` | none | HAVE | AT-01 |
| PF-17 interfaces | polymorphic traits | not expressible | — | LATER | — |
| NV-12, NV-13 term discovery, definitions, synonyms | draft a glossary from sources | IDM derivation method; Horizon source-translation map | SOP vocabulary capture fields (`SOP §4.2`) | ADOPT (as fields, human-authored with agent assist) | AT-02 |
| NV-15 human certification of a term | steward gate before a meaning is relied on | assertion status; custodian role | status transitions `proposed → declared` require custodian approval | ADOPT | AT-02 |
| NV-16 acronym dictionary | resolve jargon | semantic packs preserve contextual meanings | none | HAVE | AT-01 |
| NV-28 YAML import/export | interchange | DomainForge projections (CALM, KG, SBVR, protobuf) | none | HAVE | — |
| PF-09 dataset branches | isolate change | git branch + `world_ref` per version; gated promotion | AD-6 | ADOPT | AT-12 |
| PF-12 lineage graph | impact analysis | `impact` operation, bounded traversal | feed with case and RealityTrace sources | ADOPT | AT-14 |
| PF-22–24 explorer, search-around, object view | find and inspect | Horizon Explore, `/desk/search`, inspector | declared graph + observed overlay | ADOPT | AT-19 (J2) |

### Data connection and pipelines

| Ref | Job | GodSpeed today | Decision |
|---|---|---|---|
| PF-05–07, NV-08–11 connectors, schema introspection, profiling | ingest external sources | Context Kernel adapters (filesystem, GitHub, web); Horizon GitHub connector | HAVE for the first case; warehouse connectors LATER (needed by tax transfer only as file intake) |
| PF-08, PF-10, PF-11 pipeline builder, schedules, code workspaces | transform data | Workbench (code work), Cognate schedules | NO pipeline builder; code work stays in Workbench |
| PF-25–29 Workshop app builder | build operational apps | Horizon agent-made panels (`surface2` grammar, blocks) | HAVE in narrow form; builder is NO |
| PF-31–33 analytics canvases | ad-hoc analysis | — | NO |

### Intelligent interaction and agent work

| Ref | Job | GodSpeed today | Remaining | Decision | Test |
|---|---|---|---|---|---|
| AF-01–03 chat, history, streaming | direct an agent | Workbench conversation; Cognate event stream | none | HAVE | — |
| AF-04–06 outline, tokens, summarize | manage context | Workbench session; Cognate recall | none | HAVE / LATER | — |
| AF-07–10 modes, self-management | scope tools to the task | Workbench experts/teams, capability eligibility | brief's "tools allowed" enforced per assignment | ADOPT | AT-17 |
| AF-11 plan before writes | approve steps | Casework plan items, `delegation.preview`, `add_item dry_run` | preview in Assign (J4) | ADOPT | AT-17 |
| AF-12 clarification cards | resolve ambiguity | Horizon `ask` blocks, agent door | none | HAVE | — |
| AF-15–16 pinned context | attach resources | Context Kernel `context_bundle` bound to `world_ref` | bundle attached to assignment brief | ADOPT | AT-17 |
| AF-17–20 skills | reusable team procedure | repository skills; SOP templates | reusable-asset candidates (J10) | ADOPT (as candidates with evidence) | AT-13 |
| AF-21–24, AF-26 tiered approval, identity, branch rules | gate writes under the user's identity | SEA-Forge approvals; unified identity design | AD-7 | ADOPT | NT-05, NT-12 |
| AF-25 diff inspection | review before commit | Workbench review gate; Horizon comparison view | semantic diff in J7 | ADOPT | AT-12 |
| AF-27 tool-call audit | trace | Cognate events; RealityTrace | interaction-event profile | ADOPT | AT-05 |
| AF-28–29 deep links | start from context | Workbench `#session/` link; Horizon assign | seam (B §5) | ADOPT | AT-19 |
| AF-30 spend telemetry | admin spend view | Workbench economics | per-currency Cost view | ADOPT | AT-17 |
| AF-13, AF-14 solution designer, notepad | diagrams, reports | Horizon notebook | none | HAVE (notebook) / NO (designer) |

### Grounded querying and evaluation

| Ref | Job | GodSpeed today | Remaining | Decision | Test |
|---|---|---|---|---|---|
| NV-01–05 question to SQL, charts, export | ask business questions of data | — | not a Field Case need | NO (baseline) |
| NV-06 provenance drawer | verify an answer | value labels, `why`, record ids | parity rule | ADOPT | AT-14 |
| NV-18–24 MCP tools incl. `check_answerable`, `check_readiness` | let outside agents query, and know when it cannot answer | Cognate MCP, Context Kernel MCP, SEA-Forge readiness | "answerable" maps to `insufficient` class and `unknown` labels | ADOPT (concept) | AT-07 |
| NV-25–26 model role endpoints | choose a model per role | provider-agnostic capability contracts (`SOP §3.5`) | none | HAVE | — |
| NV-29 evaluation toolkit | measure accuracy against benchmarks | CEP benchmarks, Gauntlet ladder, RealityTrace | bridge Gauntlet records to RealityTrace | ADOPT | AT-17 |
| NV-07 feedback | improve terms | residue capture, lessons | J10 | ADOPT | AT-13 |

### Governance and platform

| Ref | Job | GodSpeed today | Decision |
|---|---|---|---|
| PF-01–04 spaces, RBAC, markings, tenancy | organize and restrict | SEA-Forge tenants, roles, policy; Horizon operator scope | HAVE; markings LATER |
| PF-35–37 developer console, SDK generation, subscriptions | programmatic access | SFWP contracts with generated TypeScript; Cognate Connect, MCP, A2A; event subscribe | HAVE |

## 4. Where GodSpeed goes beyond the references (and must prove it)

None of the references, as reported, has: an expectation frozen before the act and protected from rewriting; a
separation between request accepted, result settled and capability improved; observed behaviour reconstructed
and compared against declared meaning; or a model revision that must pass retained-behaviour regression and
independent authorization. These are the Mission's core and are the parts with no reference pattern to borrow,
so their UX (J3, J5, J6, J7) is original and carries the highest design risk; it gets the fresh-operator
walk-through in B §7.

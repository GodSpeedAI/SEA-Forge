# 07 Reference product digest (read-only, 2026-11-12)

Source: `.tmp/reference/{palantir-foundry,palantir-ai-fde,nvidia-auto-ontology}/{recon.md,features.csv}`, all six read in full. No outside knowledge added.

**Confidence, all rows:** no features.csv row has a confidence marker. The only markers are `confidence: high` on "Inferred data model" entities (Foundry recon.md:156-209; AI FDE :148-197; NV :141-197). Every entity is marked high under an "Inferred" heading. All three `screens/` dirs hold 0 files. Source tables list doc URLs (Foundry :13-20; AI FDE :13-20; NV :13-18), but the files do not say the pages were fetched. Confidence cells read `unmarked` (no marker) plus the csv line.

Key: `csv:N` = features.csv line N. `S##` = screen row in the same recon. `F##` = flow.

## A. Palantir Foundry

| ID | Capability | User job | User sees / controls | Prereqs | Confidence |
|---|---|---|---|---|---|
| PF-01 | Spaces and project hierarchy | organize assets | S02 breadcrumbs, asset grid | org>space>project | unmarked csv:2 |
| PF-02 | RBAC Viewer/Editor/Owner | limit actors | S02 permission-denied state | roles | unmarked csv:3 |
| PF-03 | Markings MAC, propagated | restrict derived data | marking badges; S20 overlay | markings | unmarked csv:4 |
| PF-04 | Organizations tenancy | isolate tenants | S19 org selector | org admin | unmarked csv:5 |
| PF-05 | Connector framework | ingest sources | S03 catalog, S04 config/test | credentials | unmarked csv:6 |
| PF-06 | On-prem sync agent (should) | reach private nets | S03 agent grid, offline state | installed agent | unmarked csv:7 |
| PF-07 | Schema inference on ingest | make dataset | S04 sync list; F01 | source | unmarked csv:8 |
| PF-08 | Visual pipeline builder | transform w/o code | S05 DAG, preview, build | datasets | unmarked csv:9 |
| PF-09 | Dataset branches, transactions | isolate change | S07 transaction timeline | dataset store | unmarked csv:10 |
| PF-10 | Health checks, scheduling (should) | keep fresh | S03 schedule overview | schedules | unmarked csv:11 |
| PF-11 | Code Workspaces (git IDE) | code-first transforms | S06 IDE, PR, CI | git repo | unmarked csv:12 |
| PF-12 | Lineage graph | impact analysis | S20 expanders, cycle warning | dependency graph | unmarked csv:13 |
| PF-13 | Object Type from dataset | define entity | S08, S09 editor; F02 | backing dataset | unmarked csv:14 |
| PF-14 | Property types, validation | typed attributes | S09 mapping grid | columns | unmarked csv:15 |
| PF-15 | Primary key uniqueness | stable identity | S09 pk selector, duplicate state | unique column | unmarked csv:16 |
| PF-16 | Link Types 1:1, 1:N, M:N | model relations | S10 cardinality, key warning | join keys | unmarked csv:17 |
| PF-17 | Interfaces (should) | polymorphic traits | no screen or flow cites it | object types | unmarked csv:18 |
| PF-18 | Action Type | governed mutation | S11 params, rules, writeback | permissions | unmarked csv:19 |
| PF-19 | Webhook writeback | push effect out | S11 "call webhook"; F03 | endpoint | unmarked csv:20 |
| PF-20 | Action audit, before/after | trace actions | S13 history log | exec records | unmarked csv:21 |
| PF-21 | Ontology Functions (should) | logic, scoring | none cited | TS/Python runtime | unmarked csv:22 |
| PF-22 | Object Explorer search, facets | find objects | S12 query bar, facets | indexed objects | unmarked csv:23 |
| PF-23 | Search-around | pivot to linked objects | S12 flyout, S13 carousel | link types | unmarked csv:24 |
| PF-24 | Object 360 view | single-object dossier | S13 props, links, actions | instance, markings | unmarked csv:25 |
| PF-25 | Workshop drag-drop studio | build ops app | S14 palette, canvas, preview | object sets | unmarked csv:26 |
| PF-26 | Reactive variables | wire widgets | S14 inspector, circular error | variable schema | unmarked csv:27 |
| PF-27 | Table and metric widgets | show data | S15 | object set | unmarked csv:28 |
| PF-28 | Workshop Action button | act from app | S15 modal, toast; F05 | action permission | unmarked csv:29 |
| PF-29 | Form widgets | collect input | S15 filters | variables | unmarked csv:30 |
| PF-30 | Permission-aware rendering | hide unusable actions | F04 edge case | roles, markings | unmarked csv:31 |
| PF-31 | Contour analysis (should) | ad-hoc metrics | S16 query board | dataset | unmarked csv:32 |
| PF-32 | Quiver canvas (should) | chained analytics | S17 card canvas | object sets | unmarked csv:33 |
| PF-33 | Slate custom apps (could) | code-first dashboards | none cited | HTML/CSS/JS | unmarked csv:34 |
| PF-34 | AIP Logic (should) | LLM actions on ontology | none in Foundry screens | model access | unmarked csv:35 |
| PF-35 | Developer Console, OAuth | self-serve credentials | S18 OAuth, scopes, expiry | dev app | unmarked csv:36 |
| PF-36 | OSDK generation (TS, Py) | typed client | S18 "Generate SDK"; F06 | ontology scope | unmarked csv:37 |
| PF-37 | WebSocket subscriptions (could) | live updates | none cited | OSDK | unmarked csv:38 |

PF-38..41 (csv:39-42: Apollo, FedRAMP/IL6, ERP binaries, industry ontologies) are skip rows (recon:239-242).

**Complete and navigable (mechanisms):** left nav; omni-search on S01/S08/S12; breadcrumbs; recent and favorites on the hub. Each screen lists its states (empty, loading, error, permission, not-found, redacted). Links connect objects, actions and lineage (search-around, relations, action buttons). Six flows give click counts and edge cases. Action results appear in object history (F05 step 6).

**Journeys (stated clicks):** F01 ingest 11; F02 object+link 14; F03 action+webhook 9; F04 Workshop 16; F05 triage 7; F06 OSDK 8.

**Unsupported or contradictory:**
- Feature matrix (recon:230-233) says 22/8/5/4. csv has 41 rows: must 27, should 8, could 6 (4 skip).
- Entities labelled "Inferred" are marked `confidence: high` (recon:156-209).
- Per-screen "States seen" has no artifact (screens/ empty).
- Apollo and IL6 claims (recon:239-240) are outside the sources.
- Pricing source (recon:21) has no csv row or screen.

## B. Palantir AI FDE

| ID | Capability | User job | User sees / controls | Prereqs | Confidence |
|---|---|---|---|---|---|
| AF-01 | Conversational platform chat | direct the agent | S02 stream | session | unmarked csv:2 |
| AF-02 | Turn history | review turns | S02 thread, S08 tree | message store | unmarked csv:3 |
| AF-03 | Streaming markdown | watch progress | S02 streaming state | model stream | unmarked csv:4 |
| AF-04 | Chat Outline (should) | navigate session | S08 sidebar | turn index | unmarked csv:5 |
| AF-05 | Per-message tokens (should) | cost awareness | S08 meter, warn 60-85%, critical >85% | token counts | unmarked csv:6 |
| AF-06 | Summarize or delete turns | avoid overflow | S08 buttons; F05 | turn content | unmarked csv:7 |
| AF-07 | Specialized modes | scoped tools | S04 grid of 9 modes | mode defs | unmarked csv:8 |
| AF-08 | Auto mode detection (should) | no manual routing | S04 auto toggle | classifier | unmarked csv:9 |
| AF-09 | Manual mode override | force a mode | S04 "manual locked" | none | unmarked csv:10 |
| AF-10 | Agent self-management tools | agent changes mode, loads docs | F03 "Load skills" | tool registry | unmarked csv:11 |
| AF-11 | Plan before writes | approve steps | S05 approve/adjust | plan generation | unmarked csv:12 |
| AF-12 | Clarification cards (should) | resolve ambiguity | S15 options, free text | none | unmarked csv:13 |
| AF-13 | Solution designer (could) | architecture diagrams | S11 canvas | design id | unmarked csv:14 |
| AF-14 | Notepad with live widgets (could) | report authoring | S12 document | doc store | unmarked csv:15 |
| AF-15 | Context ribbon pinning | attach resources | S03 search, chips | RIDs | unmarked csv:16 |
| AF-16 | Dynamic hydration of RIDs | auto context | chip states loading, invalid | catalog read | unmarked csv:17 |
| AF-17 | AIP Skills engine | team rules as Markdown | S09 authoring | skill store | unmarked csv:18 |
| AF-18 | Semantic skill discovery | auto-load a skill | F03 step 5, "Skill applied" badge | description field | unmarked csv:19 |
| AF-19 | Skill editor | author skills | S09 fields, preview | none | unmarked csv:20 |
| AF-20 | Org vs project skill scope (should) | share skills | S10 toggles, promoted badge | scope field | unmarked csv:21 |
| AF-21 | Tiered tool approval | gate writes | S06 Once / Session / Reject | approval tier | unmarked csv:22 |
| AF-22 | Execution under user identity | no escalation | F01 edge case; core loop | verified identity | unmarked csv:23 |
| AF-23 | Branch-aware approvals | protect prod branch | F04 master flagged, redirect | branch rules | unmarked csv:24 |
| AF-24 | Session approval grants (should) | fewer prompts | S06 "for this session" | grant store | unmarked csv:25 |
| AF-25 | Diff inspection | review before commit | S07 split diff, Accept | file diff | unmarked csv:26 |
| AF-26 | Approval card in chat | human-in-loop | S06 inline card | none | unmarked csv:27 |
| AF-27 | Tool-call audit | trace | FdeToolExecution (recon:166-171) | exec store | unmarked csv:28 |
| AF-28 | Pre-fill deep link | start session from URL | S13 parser; F06 | URL schema | unmarked csv:29 |
| AF-29 | Zero-click autoStart (should) | instant start | S13 3-second cancel | autoStart param | unmarked csv:30 |
| AF-30 | Token telemetry, quotas (should) | admin spend view | S14 dashboard | telemetry | unmarked csv:31 |

AF-31..33 (csv:32-34: LLM routing, Apollo/telemetry, pre-packaged skills) are skip rows (recon:213-215).

**Complete and navigable (mechanisms):** each consequential step is a stream card (plan, approval, diff, clarification) with a fixed state set (recon:40-45, 136-137). Approval scope is explicit (once, session, branch) and stored as its own entity (recon:179-183). Pinned chips show RID state; the outline shows tokens per turn. Other apps enter via URL schema with a cancel window (recon:144).

**Journeys (stated clicks):** F01 pipeline on feature branch 9; F02 object 8; F03 skill 10; F04 protected branch 7; F05 summarize 5; F06 deep link 3.

**Unsupported or contradictory:**
- Stated clicks do not match the steps. Summed: F01 = 7 (recon:59-66), F02 = 6 (:72-79), F03 = 8 (:84-91), F04 = 5 (:96-103), F05 = 4 (:109-115), F06 = 2 (:120-124).
- Feature matrix (recon:204-207) says 22/8/3/3. csv has 33 rows: must 20, should 8, could 5 (3 skip).
- Token warnings conflict: S08 warns at >80% (recon:43); gauge warns at 60-85% (recon:139). F05 uses 82%.
- F05 figures (18,500 tokens, 250-token summary, 44%) are unsourced examples.
- Entities under "Inferred" are marked `confidence: high` (recon:166-197).

## C. NVIDIA Auto-Ontology

| ID | Capability | User job | User sees / controls | Prereqs | Confidence |
|---|---|---|---|---|---|
| NV-01 | Text-to-SQL chat | ask business questions | S02 chat, starters | sources, terms | unmarked csv:2 |
| NV-02 | SQL grounded in certified terms | trustworthy SQL | S02 SQL block; F03 | certified terms | unmarked csv:3 |
| NV-03 | Virtualized result table | inspect rows | S02 grid | result | unmarked csv:4 |
| NV-04 | Auto charts (should) | visualize | S02 Bar/Line/Pie tabs | typed result | unmarked csv:5 |
| NV-05 | CSV export | take rows out | csv only, no screen | result | unmarked csv:6 |
| NV-06 | Provenance drawer | verify answer | S08 terms, tables, join path | QueryTrace (recon:189-192, "high") | unmarked csv:7 |
| NV-07 | Thumbs feedback (should) | improve terms | S02 buttons; F03 | telemetry | unmarked csv:8 |
| NV-08 | Schema introspection | crawl metadata | S05 "Introspect Schema Now" | DB credentials | unmarked csv:9 |
| NV-09 | PostgreSQL connector | connect | S05 card; F01 | URI | unmarked csv:10 |
| NV-10 | Snowflake, Databricks, DuckDB (should) | warehouses | S05 cards | warehouse access | unmarked csv:11 |
| NV-11 | Column profiling (should) | sample values | S06 dictionary | introspection | unmarked csv:12 |
| NV-12 | Term discovery | draft glossary | S05 "Run Semantic Discovery" | schema, samples | unmarked csv:13 |
| NV-13 | Term definition, synonyms | canonical meaning | S04 editor, tag ribbon | term record | unmarked csv:14 |
| NV-14 | SQL attribute mapping | bind term to columns | S04 SQL editor | physical tables | unmarked csv:15 |
| NV-15 | Human certification | steward gate | S04 "Certify Term"; F02 | steward role | unmarked csv:16 |
| NV-16 | Acronym dictionary | resolve jargon | S07; F04 | acronym store | unmarked csv:17 |
| NV-17 | Schema viewer, FK badges | inspect joins | S06 | FK metadata | unmarked csv:18 |
| NV-18 | FastMCP server | external agents query | S11 status, copy-config; F05 | port 3003 | unmarked csv:19 |
| NV-19 | MCP check_readiness | verify deployment | S11 tool health | server | unmarked csv:20 |
| NV-20 | MCP search_terms, get_term | discover terms | F05 step 5 | certified terms | unmarked csv:21 |
| NV-21 | MCP get_term_columns, get_term_sql_attributes | inspect bindings | SQL attributes (recon:177) | SQL attributes | unmarked csv:22 |
| NV-22 | MCP describe_table | table structure | F05 step 5 | introspection | unmarked csv:23 |
| NV-23 | MCP check_answerable | is question covered | scope list only (recon:3) | coverage | unmarked csv:24 |
| NV-24 | MCP ask_question | answer with SQL, rows | scope list only (recon:3) | model, DB | unmarked csv:25 |
| NV-25 | Model role endpoints | pick LLM per role | S09 role cards | keys | unmarked csv:26 |
| NV-26 | NIM and OpenAI-compatible APIs | provider routing | S09 defaults | API access | unmarked csv:27 |
| NV-27 | PostgreSQL with pgvector | store embeddings | SemanticTerm vector (recon:168) | Postgres | unmarked csv:28 |
| NV-28 | Ossie YAML import/export (should) | interchange | S10 dropzone, validation | file | unmarked csv:29 |
| NV-29 | Eval toolkit (should) | measure SQL accuracy | S12 dashboard; F06 | benchmark sets | unmarked csv:30 |
| NV-30 | Better Auth sign-in | identify user | S01 form, SSO | auth provider | unmarked csv:31 |
| NV-31 | Forward-auth user header | pass identity | csv only, no screen | gateway | unmarked csv:32 |

NV-32..34 (csv:33-35: NIM credits, warehouse accounts, proprietary dictionaries) are skip rows (recon:225-227).

**Complete and navigable (mechanisms):** left nav over 12 screens. Status badges on terms (certified, draft) and sources (connected, offline, syncing) (recon:126-133). Each answer links to a provenance drawer with exact terms and join path (S08). Certification is one visible state change, and the catalog filters by status (F02). MCP page shows copyable client config and live tool health (S11).

**Journeys (stated clicks, sums match):** F01 ingest to draft terms 7; F02 certify 6; F03 question to answer 4; F04 acronym 5; F05 MCP 4; F06 benchmark 4.

**Unsupported or contradictory:**
- Feature matrix (recon:216-219) says 22/8/2/3. csv has 34 rows: must 25, should 6, could 3 (all skip).
- Benchmark deltas (Exact Match +34%, Execution +42%, Hallucinated Columns -88%, recon:117) appear as output. No run or file is cited.
- Model roles are called a "triplet" (recon:42) but listed as four, including Judge (recon:136). csv:26 says three.
- Entities under "Inferred" are marked `confidence: high` (recon:144-197).
- "@jane" example (recon:83) and Next.js 16+ / Prisma stack (recon:15) have no source file.

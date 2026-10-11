# C2 bounded-reader normative update result

**Disposition:** normative amendments prepared under the targeted policy approval; independent review remains pending. Implementation, source, schema generation, migration, runtime readiness, and T09 settlement remain held.

## Full source grant

> Fresh tightly scoped normative DOCUMENT builder grant; your earlier reader proposal review is accepted and operator just APPROVED ALL four new policies. Receipt BASE/c2-bounded-reader-additional-policy-operator-approval-oct08.md. Read updated .agents instructions/lateststatus. Only amend .agents/specs/casework-live-cursor-v4-spec.yaml and docs/decisions/ADR-008-casework-live-cursor-v4.md, plus new immutable builder-result in BASE. Read full prior normative package/review and full reader proposal revision2+rev3+rev4/rootclarifications/all independent holds & latestapproved review; preservespec/ADR approved six C2 areas. Incorporate approved process-memory fresh Ed25519 existingcodec/getrandom key lifecycle failstartup entropy/rotateinvalidtoken boundedrecovery, token4096/payload2048 bytes preallocation checks/signaturevalidation, rawrow2MiB inclLF/page4MiB inclnon-event/lookahead vs unchanged500rows500frames1MiB FULL envelope50mslockwait caps, cooperativepinnedhead/fulloriginvalidation and no-lookaheadack; exactbinding+signednullable resolved filterords≤ackfrontier nilvsunresolved/ordinal0real; verifiedempty nullhead/frontier complete=true no synthetic and presentfilterunknownerror; unknownfilterdeferreduntilhead v2 and legacyallornothingerror unchanged wrapper. Preserve EXISTING typed entry_hash and completeValue payload_hash, unknown top-level toleratedignored/unknown payload retainedhashed, known eventfield validation detailnullvalid/no newkindgrammar. Separatev2DTOs/newoptional range_version existingverb not reshape oldframe. Budgetsguarantees no RSS/I/Olatencycap; noelapsedpagecap. Trustedglobalowner recovery vs arbitraryrequest no destructiveownerchange. Actual scopes to approved proposal; newrequirements acceptancevectors where necessary, no duplicative parallel spec. No Rust/Go/UI/schema/source/test/gates/Git edits, rootstatusowned. Native apply_patch ONLY; read everyeditedfile+nearby pattern. New concise artifact reproduces FULL thisgrant/originalproposal references, exact finalhashes and everymaterialdifference. Independent critic afteryou (different builder) required; no readiness/settlementclaim. Root owns architecture; escalate contradictions promptly rather than wrestle.

## Inputs and provenance

| Input | SHA-256 |
|---|---|
| Original bounded-reader options | `a4d2aa1c7c7a023b2bf08e75b4bd9d59e3e35d87088014a31cc7463644ed7d03` |
| Bounded-reader source recon | `168bbd5a11d8d427a7fe84c9fb0a3667c29a296e9ac79f9aed1a05d3c662ff97` |
| Original options independent HOLD | `b3a832dcc49ed5fc578934064db466a30e4f08fbe5b626293b03db9287c1f94a` |
| Revision 2 proposal | `4ff005dd073c186f159cf1e1f47432fb0e119106d105cd0b8b547ca04aa71227` |
| Revision 2 independent HOLD | `1641b2ca1a26db550d86cd6c63b2897f4f0323d9ed6aa12ffc24c4e2484956d7` |
| Root reader clarification | `c6ffb613f5eeffe2a80183e9d1de73634884c9ebd4ff73cf565852c2af7b5696` |
| Root frontier-boundary addendum | `7055688af770af4de8c003901d908ddc7e7f9f2435757a4fbb2946346910e1d9` |
| Root v2 filter clarification | `7b9af2ff4a7662d56b83ca057a6150627d649c2ec90d5146285868d07173f72d` |
| Known-field compatibility clarification | `e61df326baab37703e179e4c24d6a4a2d2a1535d0505d3f25aa54cb071e0f0f0` |
| Revision 3 overlay | `bfe581e9f5eb372b372d2a515c90d92ac5c8a932ccee2d39c77e295ef6f76f09` |
| Revision 3 independent HOLD | `615326781b2b3002ba5358da1b1de65dd54eaae04d8c89146748dfaa9bde9103` |
| Resolved-filter state addendum | `79f59d2bdd77fc24f73725ff382cce9c8922ecaad94f9c3dd07b3ada1c1b37e2` |
| Revision 4 overlay | `83f769c00083a514283fa990d8b32ea5fefdfe43d85df9abaf35e0fc9b7e489a` |
| Revision 4 independent proposal approval | `7e4a9649e2412f33bdb3cc1c6c06669b60f921daa4abf444babd35c6e84cf07e` |
| Additional policy operator approval | `d6433029755d907aa730c518402f6889933b9528c9c0d5646e7cd25ca3c381bf` |
| Existing six-area operator approval | `7651f333fcf27b741db557d0dcbd4aba140e06b670edb49d70968f267be6be4b` |
| Existing C2 spec preimage | `d509ea469408e03c1cb62cf10ab463036e161ad13bf99a7b177d7ccf3406be31` |
| Existing ADR-008 preimage | `30556f78e37ac3f6f33b2ea7e5f976b87645ca67e1cc51e330a54deb5dad17b3` |

The four added policy choices are recorded in the operator receipt above. The operator approved their normative incorporation after independent proposal review; this did not approve an exact candidate hash or implementation. The earlier six recommendation-level areas and all unrelated C2 requirements remain unchanged.

## Files and exact result identities

| File | Before SHA-256 | After SHA-256 |
|---|---|---|
| `.agents/specs/casework-live-cursor-v4-spec.yaml` | `d509ea469408e03c1cb62cf10ab463036e161ad13bf99a7b177d7ccf3406be31` | `c5a46ef01b677049cfb9a7e822c956990d403605b364a869f25a3dc9f07ddfca` |
| `docs/decisions/ADR-008-casework-live-cursor-v4.md` | `30556f78e37ac3f6f33b2ea7e5f976b87645ca67e1cc51e330a54deb5dad17b3` | `67ab46ce7bc56a137ac99a025c3d6579d8837b533a7ea7d8027c17aa11d31eb5` |

## Coverage and material differences

- The spec records the new policy approval separately from the six-area approval, retains `exact_candidate_hash_read_or_approved: false`, and keeps implementation status held. Its approval rule and verification vector now account for both approval records without approving any further design expansion.
- `limits.event_page` now includes 4,096 encoded-token bytes, 2,048 payload bytes, 2 MiB raw row including LF, and 4 MiB raw input per page including non-events and lookahead. The prior 500 rows, 500 frames, 1 MiB complete serialized response, 50 ms lock wait, and separate two-page/500 ms reconciliation turn are unchanged.
- New requirements `REQ-C2-RANGE-004` and `REQ-C2-RANGE-005` specify the process-memory Ed25519 key, startup entropy failure, restart invalidation, bounded token/signature validation, trusted-owner no-ack and bounded origin recovery, exact filter binding, signed nullable resolved ordinals, empty-stream behavior, and deferred v2 unknown-cursor refusal. Existing `REQ-C2-RANGE-001..003` now preserve the exact stream/head/row boundaries, lock-and-pin boundary, legacy error/shape contract, no elapsed page guarantee, and established row/event validation.
- The existing Rust hash semantics are explicit: `payload_hash` covers the complete `Value`; `entry_hash` uses the typed `LedgerEntry` helper; unknown top-level fields remain tolerated and ignored after typed deserialization; unknown payload fields remain included. Known event fields are checked, null `detail` remains valid, extra fields remain accepted, and no new kind grammar is introduced. The revision 3 raw-row hash proposal is not restored.
- Verification cases now cover token/input boundaries, empty-stream and requested-bound outcomes, filter resolutions across pages, the zero-versus-unresolved distinction, lookahead non-acknowledgement, and page-limit resumption. The legacy vector remains all-or-nothing. No per-page elapsed-time, RSS, heap, or blocking-I/O-latency guarantee is added; the outer scheduling deadline cannot turn an unfinished scan into success.
- ADR-008 adds the bounded-reader decision, points to the policy approval and revision 4 proposal, records the same exact compatibility and resource rules, and keeps all implementation/readiness gates. No other C2 area, authority rule, route, kernel verb, dependency, ID grammar, generated schema, or runtime behavior is changed.

## Scope and verification limits

Only the normative C2 spec and ADR-008 were amended, besides this new immutable result. No source, public schema, generator, test, status, debt, plan, or Git state was changed. No tests, compiler, formatter, scanner, build, or runtime gates were run; root retains status and gate ownership. The final normative package requires independent review before it becomes accepted, and separately requires the existing writer-participation, schema, TDD, canonical, and runtime prerequisites before any readiness or settlement claim.

# Specification workflow handoff

Read the [glossary map](GLOSSARY-MAP.md), [concordance](semantic-concordance.md) and the owning glossary before putting terms in a specification. Use the [convergence ledger](semantic-convergence-ledger.md) to map each term to current identifiers. The [source baseline](source-evidence.md) distinguishes mainline and integration behavior. Consult [PR #107’s closure register](../2026-11-19-cognitiveos-convergence/I-closure-register.md) when the separate execution design enters scope.

## Ready for specification

- Pin **World Reference** for semantic scope, **Execution State Digest** for Workbench manifest identity, and separately typed correlation or lineage when relating them. Describe the Horizon Closure Pin as its own subject.
- Keep **Case Assignment**, **Expert Assignment**, **Task**, and **Run** distinct. If work uses a binding, require the issued episode and preserve both assignment references. Treat Horizon’s Case-Step Correlation as correlation.
- Use issuer and subject to name the judgment. Give **Assignment Acceptance**, **Candidate Promotion**, **Controlled Run Settlement**, and **Operational Settlement** independent criteria. A **Settlement Receipt** records the issuer’s decision and finality.
- Treat **Tool**, **Capability Contract**, **Permission Rule**, and **Action Grant** as preconditions or declarations with distinct effects. Specify CognitiveOS Capability evidence independently. Context Access and Reference Citations provide retrieval and provenance.

## Dependencies before implementation tickets

1. Resolve Horizon’s frozen field and pin migration with the semantic and protocol owners. Begin with identity vectors and negative cases.
2. Specify the Case Assignment link using the SEA-Forge contract and Cognate runtime. Recheck Workbench WP-23 and the Expert acceptance lifecycle at the selected integration ref.
3. Specify Workbench’s observation and receipt mapping from actual producer provenance and historical payloads. Coordinate any new public record through the owning release workflow.
4. Resolve owner decisions on the formal Workbench role name, GodSpeed-Agent event naming, E6 and CEP scope, and deployment-tier theory claims as they enter scope.
5. Follow the [dependency and integrity plan](rename-impact.md) for migration and acceptance checks. Use the scenario table as negative-test input.

This package supplies language and evidence for the later Matt Pocock workflow. Behavior changes require an accepted specification and implementation plan.

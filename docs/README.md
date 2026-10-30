# Documentation Tree (docs)

## Overview
The SEA Forge technical knowledge system: a Diátaxis-organized documentation
tree indexed by `documentation-map.md` (orientation `index.md`, mental model,
architecture overview, tutorials, how-to guides, explanations, reference,
workflows, and interactive explainers under `explainers/`). Normative-adjacent
material lives here as prose: architecture decision records under
`decisions/` (ADR-001 through ADR-014), technical reference pages under
`reference/` (CLI command reference, configuration spec, persisted record
schemas, SFWP protocol reference, terminology), operations guidance under
`operations/`, subsystem documentation under `subsystems/`, security
documentation under `security/`, and the execution campaign subtree. All
direct content is Markdown; the only executable code in this tree is the
journey driver scripts.

## Subdirectories & Boundaries
- execution: [./execution/README.md] - Execution campaign documents
  (mission, plan/DAG, handoffs, verification artifacts) plus the live
  journey driver scripts and SF-0xx work packets.
- decisions, explainers, how-to, operations, reference, security, skills,
  subsystems, superpowers, tutorials, workflows, explanations-and-references,
  source documentation: (no contracts written) - Markdown documentation
  groupings only; no executable code and no contracts authored for them in
  this pass.

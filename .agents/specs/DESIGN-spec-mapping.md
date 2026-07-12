# DESIGN.md ↔ spec-full.md Mapping Review

Verdict: the visual system (tokens-by-cognitive-function, one primary focus,
CLI equivalence, quarantine color, density rules) fits spec-full well. The
**object model** did not: several design primitives belong to CognitiveOS /
GodSpeed-Agent (excluded by spec §2.4), and the design was missing the states
spec-full actually emits. Corrections applied to DESIGN.md are marked ✎.

Ground rule from the spec: the UI is a **non-goal for the substrate** (§2.4)
and arrives later as a client of E3's contracts — the Unix-socket JSONL event
stream, the CLI, and the rebuildable JSONL/ledger records. So the frontend is
a read-only projection plus exactly the governed write actions the spec
defines: approve/reject, human-task complete, manual-activation start,
discretionary add-task, case reopen/terminate, resume. Every one is an
authority-checked operation that leaves evidence — nothing else is clickable.

## 1. Primitive mapping

| DESIGN.md primitive | spec-full.md backing | Status |
|---|---|---|
| Settlement (panel, states) | `SettlementEvent` + `SettlementDeclaration` (§7.2.1) | ✅ but needs `strength: local\|strong`, `qualifies_for_capability`, independence — added ✎ |
| Evidence checklist (complete/missing/failed/stale/quarantined) | EvidenceRecord, quarantine files, verification refs | ✅ direct fit |
| Policy gate (allowed/denied/escalated) | `AuthorityDecision`, `GovernanceVerdict`, precedence `deny>boundary>allow>degraded>escalate` (§7.0) | ✅ add `boundary`/`degraded` and `OpaqueConstraint` halt as display states |
| Trace | trace.jsonl / ledger entries; `sea-forge watch` | ✅ |
| Capability | `CapabilityRecord` (§7.3) | ⚠️ design had no ladder; `attempted<demonstrated<proven<metabolized`, confidence, coverage, recovery, contraction reasons — added ✎ |
| Actor / Resource / Action | `CanonicalActionRequest` fields | ✅ |
| Notification | server events: `awaiting_approval`, `run_finished`, `run_failed` (§10.3) | ⚠️ design bans mechanic notifications — map `run_finished` → "Settlement declared", `run_failed` → "Settlement rejected" |
| Decision / choice architecture (3 options) | **No backing.** SEA Forge never ranks affordances | ❌ owner is GodSpeed-Agent (routing) — external feed, noted ✎ |
| Payment pill | **Non-goal** §2.4: "CognitiveOS owns pricing and path valuation"; SEA Forge MAY record observed cost/burden evidence only | ❌ external feed or fall back to `orchestration_burden` — noted ✎ |
| Affordance / Situation | Not spec entities. Nearest: enabled/available PlanItems (affordance-ish), case state + sentry status (situation-ish) | ⚠️ mapped in nav note ✎ |
| Mission | Not a spec entity. Nearest: workspace/cell (`cell_id`) | ⚠️ mapped ✎ |
| Agent status pill | Agents are callers/payloads, never kernel components (§1) | ✅ design already renders capabilities, not vendors — keep |
| CLI equivalence per panel | CLI-first spec; but the binary is `sea-forge`, not `godspeed` | ❌ example fixed ✎ |

## 2. Semantic states the design was missing (all added ✎)

1. **Approvals** — the single most important interactive object. `ApprovalRequest`
   `pending|approved|rejected|expired` with TTL (default 24h; expiry ⇒ rejected).
   The UI's "decision bar" belongs here, **not** on settlement: settlement is
   computed from predeclared criteria (manufactured settlement is the spec's
   primary threat, §15) — an "accept settlement" button would violate the model.
2. **Human tasks & manual activation** — operator work items (`sea-forge tasks`),
   `enabled` items awaiting operator start.
3. **Parked / awaiting_approval case state** — exit 5; "a waiting case is a
   normal state, not a failure" (§9.2). Needs a calm hold state distinct from
   blocked and from pending.
4. **Assurance level** — every inspect surface MUST display
   `legacy_digest_only | local_tamper_evident | checkpoint_signed |
   externally_verified`, plus `integrity_pending` and `ledger_integrity_error`
   (§10.0a). A GUI is an inspect surface; this is a spec MUST, not a nicety.
5. **Capability promotion ladder** + contraction reasons (§7.3).
6. **Settlement strength & independence** (local vs strong, qualifying weight).
7. **Sandbox class per run** (`local | jail | microvm`) — safety-relevant;
   jail-unavailable is a rejection, never a downgrade.
8. **Artifact stages** `cognitive→intellectual→product→capital` +
   TransitionTokens, capitalization-requires-approval (E10).
9. **Spec-pipeline proof classification** ladder `authority-only →
   generated-contract → focused-slice → live-dev-proof → release-gate-proof →
   enterprise-shippable`; generated zones read-only (E5).
10. **Milestones** (`milestone_achieved`) and sentry state — "why is this item
    not active yet" is the case-engine question a Case Detail view must answer.

## 3. Views SEA Forge needs (spec-derived)

| View | Backing records / commands |
|---|---|
| Mission Control (workspace) | open cases, pending approvals count, unsettled runs (`runs --unsettled`), assurance banner |
| Case Detail | case.json, case-events.jsonl, stage/plan-item tree with sentry satisfaction, milestones, discretionary add-task |
| Settlement Queue | pending approvals + enabled human tasks + awaiting_approval cases — the actual "decisions pending" set |
| Settlement Detail | SettlementEvent + declarations, criteria (declared-before-execution timestamp!), evidence manifest, strength |
| Approvals & Tasks | approvals.jsonl (TTL countdown), `approve|reject`, `task complete` |
| Evidence | per-run evidence + quarantine browsers |
| Capabilities | capability list/show, ladder, confidence, variation coverage, `require_proven` denials |
| Policies | policy bundle snapshots (hash-addressed), surfaces, SoD rules, opaque constraints |
| Traces | watch stream projection, live tail |
| Domain Models | DomainModelRef, source hashes, concept refs, validation evidence |
| Spec Pipelines | pipeline.json stages, proof classification, quarantine |
| Artifacts & IP | catalog.jsonl, transitions.jsonl, stage gates |
| Templates & Environments | template/env list/show, immutability (hash-pinned) |
| Memory | items.jsonl, recall evidence ("which memories informed this run") |
| Ledger Integrity | verify/prove/consistency results, checkpoints, witness receipts, quarantine |

Read model: JSONL files + event stream; all views are rebuildable projections
(same invariant the spec applies to `capability list`). The design's "semantic
state bus" is concretely: E3 event stream + the ledger.

## 4. Items deliberately left alone

- Dark-only, density tokens, motion rules, Lucide, tone — no spec conflict.
- Three-option choice architecture kept as design intent but marked
  external-feed; with no CognitiveOS feed the panel degrades to the spec-native
  set: pending approvals + enabled items (recognition over recall still holds —
  the queue is ordered, the top item is the recommended next settlement).

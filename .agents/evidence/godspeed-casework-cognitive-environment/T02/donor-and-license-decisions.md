# T02 — donor and dependency/license decisions

- **Task:** T02 (P2, confirmation: peer). Gate: `GATE_SPEC_TRACE`.
- **Preregistration:** `.agents/preregistrations/godspeed-casework-cognitive-environment/T02.prereg.yaml`
  (frozen 2026-09-19T22:16:36Z, **before** any donor search, license inspection, or dependency inventory).
- **Settles:** REQ-DONOR-001, REQ-DONOR-002, REQ-DONOR-003.
- **Teeth:** `teeth/run-teeth.sh` → exit 0 (`teeth/teeth-run.log`), both teeth behaved as specified.

## 1. Decision summary

| Donor | Spec permission | Decision | Basis |
|---|---|---|---|
| **Open MCT** | REQ-DONOR-001: MAY be used as an implementation donor for minimum temporal, provider, or time-aware-view mechanics that materially reduce engineering cost | **OMIT** | No licensed checkout is reachable from the searched roots, so no material reduction can be *demonstrated*; the spec permits native implementations and the required mechanics are specified behind GodSpeed-owned ports (brief §3 contracts, §4 consequences 2–4, §11 rules). |
| **OpenMontage** | REQ-DONOR-002: MAY inform semantic-beat, narration, choreography, renderer routing, and evidence-insertion mechanics; source reuse MUST follow explicit review | **OMIT** | Same: not reachable, nothing to compare or freeze. Narration/choreography stays GodSpeed-owned per REQ-NAR-001…005 and is implemented by T09 under the interaction grammar, not by importing a donor product's model. |
| any third-party dependency | not required by the spec | **OMIT for T02** | This task admits no dependency at all (see §3). A dependency would be an architectural/dependency change requiring the repository's prior review. |

**Absence is not the sole argument.** The decision is not "the donors are missing, therefore omit" — it is: the spec makes donor use optional and conditional on materially reducing cost; without a licensed checkout, no cost reduction and no license/provenance review can be performed, so the only honest decision available in this task is omit, with native implementations specified behind GodSpeed-owned ports. If a licensed checkout later appears, the decision must be revisited as a **new** decision record — not by quietly copying code into a later task.

## 2. Provenance and license obligations (vacuous, and recorded as such)

- **Files copied: none.** No file in this repository was copied from either donor by this task, so
  there is no provenance record and no notice obligation to discharge. This is asserted, not assumed:
  `teeth/run-teeth.sh` searches this plan's implementation paths and the whole `apps/` tree for
  third-party licence/notice headers and finds none (the tree does not yet exist; the tooth says so
  explicitly rather than passing silently).
- **Every future copied file would require:** exact donor revision, licence identification and
  compatibility check against this project's distribution, retention of required notices, a recorded
  source path, and the repository's prior review **before** copying — per the spec's `change_rule` and
  REQ-DONOR-002's "source-code reuse MUST follow explicit review".
- **Donor nouns/types must stay out of application contracts** (REQ-DONOR-003). Tooth 1 distinguishes
  the two things that are easy to conflate: a *decision* that names a donor (expected, and present in
  this record and in the brief's capability matrix) versus a *contract* that depends on one. It inspects
  the brief's contract definitions, any `go.mod`/`package.json`, and any Go/TS source under `apps/` for
  donor markers on declaration/import lines or in namespaced references, and finds none.

## 3. Dependency boundary

- **Dependencies added by T02: none.** No Go module requirement, no npm package, and no Cargo
  dependency is introduced. The tooth checks that no dependency manifest changed.
- **One toolchain declaration already exists** from T00's approved remediation: `mise.toml` pins
  `go = "1.27.1"`. That is a toolchain declaration in the mechanism this repository already uses for
  its other tools — **not** an application dependency — and it is recorded in
  `T00/approvals-and-remediation.md` with the operator's approval.
- **Boundary rule for later tasks:** the Go module may depend on the standard library and on reviewed,
  licence-compatible packages only; the React app likewise. Any addition is an architectural/dependency
  change requiring the repository's prior review and must not be smuggled in as part of a feature task.

## 4. What T02 does not prove

The plan is explicit: T02 "does not prove the resulting temporal or narration behavior; later tasks
must integrate and verify those mechanics". So this task settles the *decision and boundary* only.
The native temporal mechanics remain unbuilt until T07, and narration until T09; both remain bound by
REQ-DONOR-003 (no donor ontology in GodSpeed contracts) and must re-run tooth 1 against the real
interfaces when those exist.

## 5. Evidence index

| Artifact | Content |
|---|---|
| `teeth/run-teeth.sh`, `teeth/teeth-run.log` | Both teeth, executed |
| `../T00/approvals-and-remediation.md` | The approved Go toolchain declaration referenced in §3 |
| `.agents/plans/godspeed-casework-cognitive-environment.implementation.md` §4, §5 | The capability matrix rows and the GodSpeed-owned interface consequences this decision relies on |

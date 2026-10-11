# T02 settlement report — donor mechanics and dependency/license boundaries

- **Task:** T02 (P2, confirmation: peer). **Gate:** `GATE_SPEC_TRACE` — PASS.
- **Date:** 2026-09-19. **Settles:** REQ-DONOR-001, REQ-DONOR-002, REQ-DONOR-003.
- **Preregistration:** frozen at 2026-09-19T22:16:36Z, before any donor search or dependency inventory.

## Result

**OMIT both donors; admit no dependency.** Open MCT (REQ-DONOR-001) and OpenMontage
(REQ-DONOR-002) are not reachable from the searched roots, so no material cost reduction can be
demonstrated and no licence/provenance review can be performed; the spec makes donor use optional
and permits native implementations. Every mechanic the spec allows a donor to supply is specified
behind GodSpeed-owned ports and is owned by a later task (temporal by T07, narration by T09).
**No file was copied from any donor, so the notice/provenance obligation is vacuous — asserted by a
tooth rather than assumed.** Detailed reasoning, per-mechanic mapping, and the dependency-boundary
rule are in `donor-and-license-decisions.md`.

## Gate evidence

```
$ python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py
PASS: spec hash, 86/86 requirements, 15-task DAG, gate activation, and UI removal
```

## Teeth (executed)

`teeth/run-teeth.sh` → exit 0. `TEETH RESULT: both teeth behaved as specified`.

1. **Donor ontology must not enter application contracts.** The tooth inspects contract definitions,
   any `go.mod`/`package.json`, and any Go/TS source under `apps/` for donor markers on
   declaration/import lines or in namespaced references. Result: none. It explicitly separates "the
   decision names the donor" (expected) from "the contract depends on the donor" (forbidden).
2. **Remove access to both donor checkouts.** Neither donor is reachable under the searched roots;
   with no donor available the task still settles with a native decision and no copied code, and the
   tooth asserts that no third-party licence or notice header exists under `apps/` and that no
   dependency manifest changed.

## Preserved correction (round 0 → 1)

The tooth's first version grepped the whole brief for donor names and **failed on the plan's own
prose that records the decision** (the capability-matrix row and blocker B3). That was an oracle
defect, not a contract leak: the tooth was measuring "the document mentions a donor" when the
requirement forbids "a contract depends on a donor". It was rewritten to inspect definitions, import
paths and namespaced references only. Both rounds are in `teeth/teeth-run.log` history — the failed
run is recorded here rather than discarded.

## `done_when` adjudication

1. **Donor decision records use or omit, with provenance/notices for any copied file.** MET — use/omit
   per donor in `donor-and-license-decisions.md` §1; copied files: none, so provenance/notices are
   vacuous and are recorded as such with the tooth that asserts it.
2. **GodSpeed-owned temporal/choreography interfaces are independent of donor product models.** MET
   *for the contract surfaces that exist*: no contract definition, import path or namespaced reference
   carries a donor marker. Stated honestly: the Go ports and React contract package do not exist yet
   (T01/T04), so the tooth's strongest form is deferred — it reports
   `NOT-APPLICABLE-YET` and instructs T04/T07 to re-run it against the real interfaces.
3. **License/provenance review is complete for every copied file.** MET vacuously — there are no
   copied files; the future obligation is written down in §2 of the decision record.
4. **All applicable global gates remain green.** MET — `GATE_SPEC_TRACE` PASS; T02 adds no code, so
   no Rust/Go gate is affected by this task.

## What T02 does not prove

Not the resulting temporal or narration behaviour (the plan says so explicitly): T07 and T09 own
those mechanics, and both remain bound by REQ-DONOR-003.

## Remaining risk recorded by this task

If a licensed donor later becomes available, `OMIT` must be revisited as a **new** decision record,
not by copying code into an unrelated task. Reusing a donor without that record would silently
contradict this settlement.

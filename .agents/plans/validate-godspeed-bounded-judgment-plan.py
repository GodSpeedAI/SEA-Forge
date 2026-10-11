#!/usr/bin/env python3
"""Validate the GodSpeed bounded-judgment execution plan mechanically."""

from __future__ import annotations

import hashlib
import re
import sys
from collections import defaultdict, deque
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[2]
PLAN_PATH = ROOT / ".agents/plans/godspeed-bounded-judgment-plan.yaml"
SPEC_PATH = ROOT / ".agents/specs/GODSPEED_JUDGMENT_PLANE_SPEC.yaml"
STATUS_PATH = ROOT / ".agents/current_status.yml"
REQUIRED_ID = re.compile(r"\b(?:REQ-[A-Z]+-\d{3}|VAR-\d{3}|RECOV-\d{3}|CLAIM-\d{3})\b")
TASK_FIELDS = {
    "title",
    "initial_status",
    "depends_on",
    "proof_level",
    "confirmation",
    "settles",
    "changes",
    "proves",
    "does_not_prove",
    "steps",
    "gate",
    "teeth",
    "redesign_trigger",
    "evidence",
    "done_when",
}
INDEPENDENT_MODES = {"independent", "independent_adversarial", "fresh_target_reproduction"}
MODE_SATISFIES = {
    "independent": INDEPENDENT_MODES,
    "adversarial": {"independent_adversarial"},
    "fresh_target_reproduction": {"fresh_target_reproduction"},
}
PREREGISTERED_TASKS = {"T01", "T04", "T05", "T12", "T14", "T15", "T16", "T18", "T19", "T21", "T27", "T29"}
FORBIDDEN_PLAN_LITERALS = {
    "gauntlet-engine",
    "gauntlet-kernel",
    "gauntlet-storage",
    "../gauntlet/evidence/objects/",
    "../gauntlet/evidence/pins/",
    "PolicyDecisionKind",
    "ON THE TARGET MACHINE",
}


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def load_yaml(path: Path) -> dict:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a YAML mapping")
    return value


def gate_command(entry: object) -> str:
    if isinstance(entry, str):
        return entry
    if isinstance(entry, dict) and isinstance(entry.get("command"), str):
        return entry["command"]
    return ""


def main() -> int:
    errors: list[str] = []
    plan_text = PLAN_PATH.read_text(encoding="utf-8")
    spec_bytes = SPEC_PATH.read_bytes()
    spec_text = spec_bytes.decode("utf-8")
    plan = load_yaml(PLAN_PATH)
    spec = load_yaml(SPEC_PATH)
    status = load_yaml(STATUS_PATH)

    bound = plan.get("source", {}).get("spec", {})
    actual_sha = hashlib.sha256(spec_bytes).hexdigest()
    if bound.get("sha256") != actual_sha:
        fail(errors, f"spec sha256 mismatch: bound={bound.get('sha256')} actual={actual_sha}")
    metadata = spec.get("metadata", {})
    for key, expected in (
        ("id", bound.get("spec_id")),
        ("version", bound.get("spec_version")),
        ("status", "authoritative"),
    ):
        if metadata.get(key) != expected:
            fail(errors, f"spec metadata.{key}={metadata.get(key)!r}, expected {expected!r}")

    checks = spec.get("final_self_check", {}).get("statements", [])
    held = sum(1 for item in checks if isinstance(item, dict) and item.get("holds") is True)
    if len(checks) != 12 or held != 12:
        fail(errors, f"spec final_self_check is {held}/{len(checks)}, expected 12/12")
    corrections = spec.get("correction_ledger", {}).get("corrections", [])
    reversals = spec.get("correction_ledger", {}).get("architectural_conclusions_reversed")
    if len(corrections) != 11:
        fail(errors, f"correction ledger has {len(corrections)} entries, expected 11")
    if reversals != "none":
        fail(errors, f"architectural_conclusions_reversed={reversals!r}, expected 'none'")

    if "explicit_non_requirements" not in plan:
        fail(errors, "top-level explicit_non_requirements is missing (check YAML block indentation)")
    if plan.get("metadata", {}).get("version") != "1.7.0":
        fail(errors, "plan metadata.version must be 1.7.0")

    tasks = plan.get("tasks", {})
    task_ids = set(tasks)
    expected_task_ids = {f"T{i:02d}" for i in range(30)}
    if task_ids != expected_task_ids:
        fail(errors, f"task ids differ: missing={sorted(expected_task_ids-task_ids)} extra={sorted(task_ids-expected_task_ids)}")

    task_settlers: dict[str, set[str]] = defaultdict(set)
    prereg_paths: set[str] = set()
    for task_id, task in tasks.items():
        missing_fields = TASK_FIELDS - set(task)
        if missing_fields:
            fail(errors, f"{task_id} missing fields: {sorted(missing_fields)}")
        for requirement in task.get("settles", []):
            task_settlers[requirement].add(task_id)
        mode = task.get("confirmation")
        if mode in INDEPENDENT_MODES and "independent_confirmation" not in task:
            fail(errors, f"{task_id} confirmation={mode} requires independent_confirmation")
        prereg = task.get("preregistration")
        if task_id in PREREGISTERED_TASKS:
            if not isinstance(prereg, dict) or prereg.get("required") is not True:
                fail(errors, f"{task_id} must require preregistration")
            else:
                artifact = prereg.get("artifact")
                if not isinstance(artifact, str) or artifact in prereg_paths:
                    fail(errors, f"{task_id} preregistration artifact is missing or duplicated")
                prereg_paths.add(artifact)
        elif isinstance(prereg, dict) and prereg.get("required") is True:
            fail(errors, f"{task_id} unexpectedly requires preregistration")

        steps_text = "\n".join(str(step) for step in task.get("steps", []))
        for entry in task.get("gate", []):
            command = gate_command(entry)
            if not command:
                fail(errors, f"{task_id} has a malformed gate entry: {entry!r}")
                continue
            if re.search(r"<[^>]+>", command):
                fail(errors, f"{task_id} gate contains an unresolved placeholder: {command}")
            for script in re.findall(r"(?:^|\s)(\.agents/\S+\.py)(?:\s|$)", command):
                if not (ROOT / script).exists() and Path(script).name not in steps_text:
                    fail(errors, f"{task_id} gate script is absent and no authoring step names it: {script}")

    required_ids = set(REQUIRED_ID.findall(spec_text))
    traceability = plan.get("traceability", {})
    mapping = {key: set(value) for key, value in traceability.get("requirement_to_tasks", {}).items()}
    mapped_ids = set(mapping)
    if required_ids != mapped_ids:
        fail(errors, f"traceability mismatch: unmapped={sorted(required_ids-mapped_ids)} unknown={sorted(mapped_ids-required_ids)}")
    if required_ids != set(task_settlers):
        fail(errors, f"task settlement mismatch: unsettled={sorted(required_ids-set(task_settlers))} unknown={sorted(set(task_settlers)-required_ids)}")
    for requirement in sorted(required_ids):
        if mapping.get(requirement, set()) != task_settlers.get(requirement, set()):
            fail(errors, f"{requirement} mapping={sorted(mapping.get(requirement,set()))} but task.settles={sorted(task_settlers.get(requirement,set()))}")
    if traceability.get("spec_requirement_count") != len(required_ids):
        fail(errors, "traceability.spec_requirement_count is stale")
    if traceability.get("mapped_requirement_count") != len(mapped_ids):
        fail(errors, "traceability.mapped_requirement_count is stale")

    declared_edges = {tuple(edge) for edge in plan.get("dependency_graph", {}).get("edges", [])}
    dependency_edges = {
        (dependency, task_id)
        for task_id, task in tasks.items()
        for dependency in task.get("depends_on", [])
    }
    if declared_edges != dependency_edges:
        fail(errors, f"DAG edge mismatch: missing={sorted(dependency_edges-declared_edges)} extra={sorted(declared_edges-dependency_edges)}")
    adjacency = {task_id: [] for task_id in tasks}
    indegree = {task_id: 0 for task_id in tasks}
    for source, target in dependency_edges:
        if source not in tasks or target not in tasks or source == target:
            fail(errors, f"invalid edge {source}->{target}")
            continue
        adjacency[source].append(target)
        indegree[target] += 1
    queue = deque(sorted(task for task, degree in indegree.items() if degree == 0))
    visited: list[str] = []
    while queue:
        current = queue.popleft()
        visited.append(current)
        for target in sorted(adjacency[current]):
            indegree[target] -= 1
            if indegree[target] == 0:
                queue.append(target)
    if len(visited) != len(tasks):
        fail(errors, f"dependency graph contains a cycle involving {sorted(set(tasks)-set(visited))}")
    initial = plan.get("initial_state", {})
    ready = {task_id for task_id, task in tasks.items() if not task.get("depends_on")}
    if set(initial.get("ready", [])) != ready:
        fail(errors, f"initial ready set is wrong: {initial.get('ready')} expected {sorted(ready)}")
    expected_blocked = {task_id: task.get("depends_on", []) for task_id, task in tasks.items() if task.get("depends_on")}
    if initial.get("blocked") != expected_blocked:
        fail(errors, "initial blocked map does not exactly match task dependencies")
    status_metadata = status.get("metadata", {})
    if status_metadata.get("plan_id") != plan.get("metadata", {}).get("id"):
        fail(errors, "current status plan_id does not match the plan")
    if status_metadata.get("plan_version") != plan.get("metadata", {}).get("version"):
        fail(errors, "current status plan_version does not match the plan")
    if status_metadata.get("spec_sha256") != actual_sha:
        fail(errors, "current status spec_sha256 does not match the authoritative spec")
    status_execution = status.get("execution", {})
    settled = {t for t in status_execution.get("settled_tasks", []) or [] if t in tasks}
    execution_started = bool(settled) or status_execution.get("last_settled_task") is not None
    if not execution_started:
        # Pre-execution binding: status must equal the plan's declared initial state.
        if set(status_execution.get("ready_tasks", [])) != ready:
            fail(errors, "current status ready_tasks does not match the plan initial state")
        if status_execution.get("blocked_tasks") != expected_blocked:
            fail(errors, "current status blocked_tasks does not match task dependencies")
        if status_execution.get("last_settled_task") is not None or status_execution.get("in_flight_task") is not None:
            fail(errors, "current status claims execution has started before T00")
        if status.get("active", {}).get("task") != "T00" or status_execution.get("plan_blocked") is not False:
            fail(errors, "current status must expose T00 as active and the remediated plan as unblocked")
        ready_now = sorted(ready)
    else:
        # Post-execution: readiness is recomputed from depends_on plus settled tasks
        # (plan status_rule), never copied from prose.
        last = status_execution.get("last_settled_task")
        if last is not None and last not in settled:
            fail(errors, f"last_settled_task {last!r} is missing from execution.settled_tasks")
        derived_ready = sorted(
            task_id for task_id in tasks
            if task_id not in settled and set(tasks[task_id].get("depends_on", [])) <= settled
        )
        derived_blocked = {
            task_id: tasks[task_id].get("depends_on", [])
            for task_id in tasks
            if task_id not in settled and set(tasks[task_id].get("depends_on", [])) - settled
        }
        # Branch outcomes (plan branch_outcomes / task-level authorization
        # preconditions) can block a task whose DAG dependencies are already
        # settled. execution.branch_blockers maps task id -> reason; blocked
        # tasks listed there are excluded from readiness until cleared.
        branch_blockers = status_execution.get("branch_blockers", {}) or {}
        for blocked_task in branch_blockers:
            if blocked_task not in tasks:
                fail(errors, f"branch_blockers names unknown task {blocked_task!r}")
            elif blocked_task in settled:
                fail(errors, f"branch_blockers names settled task {blocked_task!r}")
        derived_ready = [t for t in derived_ready if t not in branch_blockers]
        if set(status_execution.get("ready_tasks", [])) != set(derived_ready):
            fail(errors, f"current status ready_tasks {status_execution.get('ready_tasks')} does not match the DAG-derived set {derived_ready}")
        expected_blocked_now = dict(derived_blocked)
        for blocked_task in branch_blockers:
            expected_blocked_now.setdefault(blocked_task, tasks[blocked_task].get("depends_on", []))
        if set(status_execution.get("blocked_tasks", {})) != set(expected_blocked_now):
            fail(errors, "current status blocked_tasks does not match the DAG-derived dependencies plus branch blockers")
        if status_execution.get("plan_blocked") is not False:
            fail(errors, "current status plan_blocked must be false unless a blocked decision is recorded")
        in_flight = status_execution.get("in_flight_task")
        if in_flight is not None and in_flight not in tasks:
            fail(errors, f"in_flight_task {in_flight!r} is not a known task id")
        active = status.get("active", {})
        if active.get("task") is not None and active.get("task") not in tasks:
            fail(errors, f"active.task {active.get('task')!r} is not a known task id")
        ready_now = derived_ready

    high_risk = spec.get("conformance_traceability", {}).get("high_risk_requirement_groups", [])
    plan_high_risk = traceability.get("high_risk_confirmation_check", {})
    for group in high_risk:
        name = group["group"]
        required_mode = group["confirmation"]
        satisfying_modes = MODE_SATISFIES[required_mode]
        satisfying_tasks: set[str] = set()
        for requirement in group["requirements"]:
            matches = {
                task_id
                for task_id in task_settlers.get(requirement, set())
                if tasks[task_id].get("confirmation") in satisfying_modes
            }
            if not matches:
                fail(errors, f"high-risk group {name}: {requirement} lacks {required_mode} confirmation")
            satisfying_tasks.update(matches)
        recorded = plan_high_risk.get(name, {})
        if recorded.get("verdict") != "SATISFIED":
            fail(errors, f"high-risk group {name} must record verdict SATISFIED")
        if set(recorded.get("settled_by", [])) != satisfying_tasks:
            fail(errors, f"high-risk group {name} settled_by is stale: {recorded.get('settled_by')} expected {sorted(satisfying_tasks)}")

    for literal in FORBIDDEN_PLAN_LITERALS:
        if literal in plan_text:
            fail(errors, f"forbidden stale literal remains in plan: {literal}")

    if errors:
        print("GodSpeed plan validation: FAIL", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("GodSpeed plan validation: PASS")
    print(f"spec_sha256={actual_sha}")
    print(f"spec_self_check={held}/{len(checks)} corrections={len(corrections)}")
    print(f"requirements={len(required_ids)} mapped={len(mapped_ids)}")
    print(f"tasks={len(tasks)} dag_edges={len(dependency_edges)} acyclic=true initial_ready={sorted(ready)} status_aligned=true")
    print(f"high_risk_groups={len(high_risk)}/{len(high_risk)} confirmation_conformant=true")
    print(f"settled={sorted(settled) if settled else '[]'} ready_now={ready_now}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

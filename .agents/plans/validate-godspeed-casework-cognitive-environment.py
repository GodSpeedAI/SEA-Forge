#!/usr/bin/env python3
"""Validate the casework-environment spec/plan binding and settlement DAG."""

from __future__ import annotations

import hashlib
import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
SPEC = ROOT / ".agents/specs/godspeed.casework-cognitive-environment-spec.yaml"
PLAN = ROOT / ".agents/plans/godspeed-casework-cognitive-environment.plan.yaml"
REQUIREMENT = re.compile(r"REQ-[A-Z]+-\d{3}\Z")


class UniqueKeyLoader(yaml.SafeLoader):
    pass


def construct_unique_mapping(
    loader: UniqueKeyLoader, node: yaml.MappingNode
) -> dict:
    mapping: dict = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node)
        if key in mapping:
            raise ValueError(f"duplicate YAML key {key!r} at line {key_node.start_mark.line + 1}")
        mapping[key] = loader.construct_object(value_node)
    return mapping


UniqueKeyLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, construct_unique_mapping
)


def main() -> int:
    errors: list[str] = []
    spec_bytes = SPEC.read_bytes()
    spec = yaml.load(spec_bytes, Loader=UniqueKeyLoader)
    plan_text = PLAN.read_text(encoding="utf-8")
    plan = yaml.load(plan_text, Loader=UniqueKeyLoader)
    bound = plan["source"]["spec"]
    metadata = spec["metadata"]
    if metadata.get("status") != "authoritative":
        errors.append("source spec is not authoritative")
    if bound["intended_repository_path"] != str(SPEC.relative_to(ROOT)):
        errors.append("plan source.spec path differs from the actual spec")
    if bound["spec_id"] != metadata["id"] or bound["spec_version"] != metadata["version"]:
        errors.append("plan source.spec id/version differs from the spec")
    if bound["sha256"] != hashlib.sha256(spec_bytes).hexdigest():
        errors.append("frozen spec SHA-256 differs from current bytes")
    if re.search(r"UNBOUND_|__T\d+_BIND_", plan_text):
        errors.append("unbound plan sentinel remains")

    requirements = spec["requirements"]
    ids = [item["id"] for item in requirements]
    if len(ids) != 86 or len(set(ids)) != 86:
        errors.append(f"expected 86 unique requirements, got {len(ids)} / {len(set(ids))}")
    if any(not REQUIREMENT.fullmatch(value) for value in ids):
        errors.append("invalid requirement ID")
    if any(not isinstance(item.get("text"), str) or not item["text"].strip() for item in requirements):
        errors.append("requirement without text")

    tasks = plan["tasks"]
    expected_tasks = {f"T{i:02}" for i in range(15)}
    if set(tasks) != expected_tasks:
        errors.append(f"task IDs differ: {sorted(set(tasks) ^ expected_tasks)}")
    mapping = plan["traceability"]["requirement_to_tasks"]
    if set(mapping) != set(ids):
        errors.append(f"requirement mapping differs: {sorted(set(mapping) ^ set(ids))}")
    if plan["traceability"]["spec_requirement_count"] != len(ids):
        errors.append("spec_requirement_count is stale")
    if plan["traceability"]["mapped_requirement_count"] != len(mapping):
        errors.append("mapped_requirement_count is stale")
    for rid, owners in mapping.items():
        if not owners:
            errors.append(f"{rid} has no settlement task")
        for task_id in owners:
            if task_id not in tasks or rid not in tasks[task_id]["settles"]:
                errors.append(f"{rid} -> {task_id} lacks a matching task settlement")
    for task_id, task in tasks.items():
        for rid in task["settles"]:
            if task_id not in mapping.get(rid, []):
                errors.append(f"{task_id} settles {rid} without a matching mapping")
        if task_id != "T00" and not task["depends_on"]:
            errors.append(f"{task_id} has no dependency")
        if task["proof_level"] == "P3" and task.get("confirmation") != "independent_adversarial":
            errors.append(f"{task_id} weakens P3 confirmation")

    edges = {(start, end) for start, end in plan["dependency_graph"]["edges"]}
    declared = {(dep, task_id) for task_id, task in tasks.items() for dep in task["depends_on"]}
    if edges != declared:
        errors.append(f"dependency edge mismatch: {sorted(edges ^ declared)}")
    if any(start not in tasks or end not in tasks for start, end in edges):
        errors.append("dependency edge names an unknown task")
    remaining = {task_id: set(task["depends_on"]) for task_id, task in tasks.items()}
    visited: set[str] = set()
    while ready := {task_id for task_id, deps in remaining.items() if not deps}:
        visited |= ready
        remaining = {task_id: deps - ready for task_id, deps in remaining.items() if task_id not in ready}
    if remaining:
        errors.append(f"dependency cycle or unknown prerequisite: {sorted(remaining)}")
    if plan["initial_state"]["ready"] != ["T00"] or plan["initial_state"]["blocked"] != {
        task_id: task["depends_on"] for task_id, task in tasks.items() if task_id != "T00"
    }:
        errors.append("initial readiness differs from task dependencies")
    if plan["dependency_graph"]["critical_path"][-1] != "T14":
        errors.append("critical path does not end at removal")
    if any(
        (start, end) not in edges
        for start, end in zip(
            plan["dependency_graph"]["critical_path"],
            plan["dependency_graph"]["critical_path"][1:],
        )
    ):
        errors.append("critical path contains a missing edge")

    gates = {item["id"]: item for item in plan["verification"]["global_gates"]}
    if len(gates) != len(plan["verification"]["global_gates"]):
        errors.append("duplicate gate ID")
    for gate_id, gate in gates.items():
        activation = gate.get("activation_task")
        if activation not in tasks or not gate.get("command"):
            errors.append(f"{gate_id} lacks a command or valid activation task")
    if not plan["operating_contract"]["parallel_work_isolation"].get("required"):
        errors.append("parallel worktree isolation is not required")
    if not plan["operating_contract"]["build_resource_policy"].get("resource_deferred_mode"):
        errors.append("resource-deferred work has no explicit settlement rule")
    for gate_id in ("GATE_SEAFORGE", "GATE_GAUNTLET"):
        if gates[gate_id]["activation_task"] != "T05":
            errors.append(f"{gate_id} must activate at first real integration (T05)")
        if gate_id in tasks["T00"]["gate"] or gate_id in tasks["T04"]["gate"]:
            errors.append(f"{gate_id} blocks resource-light foundation work")
        if gate_id not in tasks["T13"]["gate"] or gate_id not in tasks["T14"]["gate"]:
            errors.append(f"{gate_id} is missing from final confirmation/removal")
    if "../gauntlet" in plan_text.lower():
        errors.append("plan still addresses the operator's active Gauntlet checkout")
    for task_id, task in tasks.items():
        for gate_id in task["gate"]:
            if not gate_id.startswith("GATE_"):
                continue
            if gate_id not in gates:
                errors.append(f"{task_id} uses unknown gate {gate_id}")
            elif int(gates[gate_id]["activation_task"][1:]) > int(task_id[1:]):
                errors.append(f"{task_id} uses {gate_id} before activation")
    if set(tasks["T14"]["settles"]) != {"REQ-MIG-001", "REQ-MIG-002"}:
        errors.append("T14 does not settle both mandatory UI removals")
    if {
        rid for task_id, task in tasks.items() if task_id != "T14" for rid in task["settles"]
    } != set(ids) - {"REQ-MIG-001", "REQ-MIG-002"}:
        errors.append("pre-removal tasks do not settle exactly the 84 functional requirements")
    if "GATE_REMOVAL" not in tasks["T14"]["gate"]:
        errors.append("T14 omits the removal gate")
    if not plan["final_acceptance"].get("migration_and_removal"):
        errors.append("final acceptance omits mandatory removal")
    groups = spec["verification_contract"]["high_risk_groups"]
    if len(groups) != 6 or len({group["name"] for group in groups}) != 6:
        errors.append("expected six distinct high-risk groups")
    for group in groups:
        confirmer = group["confirmation_task"]
        if confirmer not in tasks:
            errors.append(f"{group['name']} names unknown confirmer {confirmer}")
            continue
        if tasks[confirmer]["confirmation"] != group["mode"]:
            errors.append(f"{group['name']} has weaker confirmation than the spec")
        for rid in group["requirements"]:
            if rid not in ids:
                errors.append(f"{group['name']} names unknown requirement {rid}")
            if rid not in tasks[confirmer]["settles"] + tasks[confirmer].get("confirms", []):
                errors.append(f"{confirmer} does not confirm {rid} for {group['name']}")

    for error in errors:
        print(f"FAIL: {error}")
    if errors:
        return 1
    print("PASS: spec hash, 86/86 requirements, 15-task DAG, gate activation, and UI removal")
    return 0


if __name__ == "__main__":
    sys.exit(main())

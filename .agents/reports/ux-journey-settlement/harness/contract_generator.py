#!/usr/bin/env python3
"""
SEA-Forge Journey Settlement Gauntlet — Contract Generator

Extracts canonical obligations, classifications, maturity counts, and interface
bindings from authoritative repository sources and generates machine-readable
Journey Test Contracts (CJ01.json .. CJ12.json).
"""

import csv
import hashlib
import json
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
REPORT_ROOT = REPO_ROOT / ".agents" / "reports" / "ux-journey-settlement"
HARNESS_DIR = REPORT_ROOT / "harness"
CONTRACTS_DIR = HARNESS_DIR / "generated-contracts"

CANONICAL_SOURCES = [
    ".sea/interaction/canonicalization-matrix.csv",
    ".sea/interaction/canonical-journey-catalog.md",
    ".sea/interaction/journey-grammar.md",
    ".sea/interaction/interaction-model.sea",
    ".sea/interaction/journey-capability-map.md",
    ".sea/interaction/coverage-report.md",
    ".agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md",
    ".agents/specs/spec-minimum.md",
    ".agents/specs/spec-full.md",
]

def compute_sha256(rel_path: str) -> str:
    full_path = REPO_ROOT / rel_path
    if not full_path.exists():
        return ""
    h = hashlib.sha256()
    h.update(full_path.read_bytes())
    return h.hexdigest()

def get_source_hashes():
    return [
        {"source_path": src, "sha256": compute_sha256(src)}
        for src in CANONICAL_SOURCES
        if (REPO_ROOT / src).exists()
    ]

def parse_csv_matrix():
    csv_path = REPO_ROOT / ".sea/interaction/canonicalization-matrix.csv"
    stories_by_cj = {f"CJ{i:02d}": [] for i in range(1, 13)}
    
    with open(csv_path, "r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            cj_id = row["Canonical Journey ID"].strip()
            if cj_id in stories_by_cj:
                stories_by_cj[cj_id].append({
                    "observed_id": row["Observed Journey ID"].strip(),
                    "title": row["Observed Journey"].strip(),
                    "classification": row["Classification"].strip().lower(),
                    "maturity": row["Implementation Maturity"].strip().lower(),
                    "confidence": row["Confidence"].strip().lower(),
                    "variation_dimensions": row["Variation Dimensions"].strip(),
                    "evidence": row["Evidence"].strip(),
                })
    return stories_by_cj

def parse_catalog_md():
    catalog_path = REPO_ROOT / ".sea/interaction/canonical-journey-catalog.md"
    text = catalog_path.read_text(encoding="utf-8")
    
    sections = re.split(r"^##\s+(CJ\d{2})\s+—\s+(.+)$", text, flags=re.MULTILINE)
    # sections: [preamble, cj_id_1, cj_title_1, body_1, cj_id_2, cj_title_2, body_2, ...]
    
    cj_data = {}
    for i in range(1, len(sections), 3):
        cj_id = sections[i].strip()
        title = sections[i+1].strip()
        body = sections[i+2].strip()
        
        fields = {}
        for line in body.splitlines():
            line = line.strip()
            if line.startswith("- "):
                m = re.match(r"^-\s+([^:]+):\s+(.+)$", line)
                if m:
                    field_name = m.group(1).strip().lower()
                    field_val = m.group(2).strip()
                    fields[field_name] = field_val
        
        cj_data[cj_id] = {
            "title": title,
            "intention": fields.get("user intention", ""),
            "job": fields.get("user job", ""),
            "initiating_condition": fields.get("initiating condition", ""),
            "preconditions": [p.strip() for p in fields.get("preconditions", "").split(";") if p.strip()],
            "entry_points": [p.strip() for p in fields.get("entry points", "").split(",") if p.strip()],
            "reusable_steps": [p.strip() for p in fields.get("reusable steps", "").split(";") if p.strip()],
            "invoked_capabilities": [p.strip() for p in fields.get("invoked capabilities", "").split(",") if p.strip()],
            "state_transitions": [p.strip() for p in fields.get("state transitions", "").split(";") if p.strip()],
            "artifacts": [p.strip() for p in fields.get("artifacts", "").split(",") if p.strip()],
            "evidence": [p.strip() for p in fields.get("evidence", "").split(";") if p.strip()],
            "completion_condition": fields.get("completion condition", ""),
            "next_decisions": [p.strip() for p in fields.get("next decisions", "").split(",") if p.strip()],
            "recovery_paths": [p.strip() for p in fields.get("recovery paths", "").split(";") if p.strip()],
            "known_variants": [p.strip() for p in fields.get("known variants", "").split(";") if p.strip()],
            "maturity_note": fields.get("implementation maturity", ""),
        }
    return cj_data

def parse_journey_capability_map():
    map_path = REPO_ROOT / ".sea/interaction/journey-capability-map.md"
    text = map_path.read_text(encoding="utf-8")
    
    sections = re.split(r"^##\s+(CJ\d{2})\s+—\s+(.+)$", text, flags=re.MULTILINE)
    bindings_by_cj = {f"CJ{i:02d}": {"web_ui": [], "api": [], "cli": [], "agent": []} for i in range(1, 13)}
    
    for i in range(1, len(sections), 3):
        cj_id = sections[i].strip()
        body = sections[i+2].strip()
        
        current_surface = None
        for line in body.splitlines():
            line = line.strip()
            if "Interface bindings:" in line:
                continue
            if line.startswith("- Web UI:"):
                current_surface = "web_ui"
                val = line.replace("- Web UI:", "").strip()
                for item in re.findall(r"`([^`]+)`", val):
                    bindings_by_cj[cj_id]["web_ui"].append(item)
            elif line.startswith("- API:"):
                current_surface = "api"
                val = line.replace("- API:", "").strip()
                for item in re.findall(r"`([^`]+)`", val):
                    bindings_by_cj[cj_id]["api"].append(item)
            elif line.startswith("- CLI:"):
                current_surface = "cli"
                val = line.replace("- CLI:", "").strip()
                for item in re.findall(r"`([^`]+)`", val):
                    bindings_by_cj[cj_id]["cli"].append(item)
            elif line.startswith("- Agent:"):
                current_surface = "agent"
                val = line.replace("- Agent:", "").strip()
                for item in re.findall(r"`([^`]+)`", val):
                    bindings_by_cj[cj_id]["agent"].append(item)
            elif line.startswith("`") and current_surface:
                for item in re.findall(r"`([^`]+)`", line):
                    bindings_by_cj[cj_id][current_surface].append(item)
    return bindings_by_cj

ACTOR_ROLE_MAP = {
    "CJ01": {"primary_actor": "operator", "primary_role": "operator", "allowed_roles": ["operator", "cell administrator", "automated actor"]},
    "CJ02": {"primary_actor": "operator", "primary_role": "operator", "allowed_roles": ["operator", "case author", "external actor"]},
    "CJ03": {"primary_actor": "domain_author", "primary_role": "domain author", "allowed_roles": ["domain author", "plan author", "operator"]},
    "CJ04": {"primary_actor": "case_author", "primary_role": "case author", "allowed_roles": ["case author", "operator", "external actor"]},
    "CJ05": {"primary_actor": "case_owner", "primary_role": "case owner", "allowed_roles": ["case owner", "operator"]},
    "CJ06": {"primary_actor": "approver", "primary_role": "approver", "allowed_roles": ["approver", "human task assignee", "operator"]},
    "CJ07": {"primary_actor": "executor", "primary_role": "executor", "allowed_roles": ["executor", "agent executor", "operator"]},
    "CJ08": {"primary_actor": "operator", "primary_role": "operator", "allowed_roles": ["operator", "case owner"]},
    "CJ09": {"primary_actor": "auditor", "primary_role": "auditor", "allowed_roles": ["auditor", "reviewer", "operator"]},
    "CJ10": {"primary_actor": "operator", "primary_role": "operator", "allowed_roles": ["operator", "planner", "router"]},
    "CJ11": {"primary_actor": "artifact_owner", "primary_role": "artifact owner", "allowed_roles": ["artifact owner", "operator", "pipeline engine"]},
    "CJ12": {"primary_actor": "cell_administrator", "primary_role": "cell administrator", "allowed_roles": ["cell administrator", "operator"]},
}

UNAVAILABLE_STATES_MAP = {
    "CJ01": ["sponsorship.request method is not implemented in SFWP", "extension and endpoint full lifecycle partly unrendered"],
    "CJ02": ["authority.preflight method is preview-only", "combined projection/environment single query unsurfaced"],
    "CJ03": ["complete visual domain model browser is preview-only (/models shows specification preview)"],
    "CJ04": ["all-fields preflight beyond summary is preview-only"],
    "CJ05": ["direct live plan in-place mutation without reopen/replan is forbidden"],
    "CJ06": ["human_task.complete standalone route is preview-only"],
    "CJ07": ["external un-sandboxed host execution without grant is forbidden"],
    "CJ08": ["actionable notifications center is preview-only"],
    "CJ09": ["cross-family unified audit search is preview-only"],
    "CJ10": ["workbench capability browser and memory store browser (/memory, /capabilities) are preview-only"],
    "CJ11": ["workbench artifact browser (/artifacts) is preview-only"],
    "CJ12": ["workbench federation surface (/federation) is preview-only"],
}

def main():
    CONTRACTS_DIR.mkdir(parents=True, exist_ok=True)
    source_hashes = get_source_hashes()
    stories_by_cj = parse_csv_matrix()
    catalog_data = parse_catalog_md()
    bindings = parse_journey_capability_map()
    
    generated_files = []
    
    for i in range(1, 13):
        cj_id = f"CJ{i:02d}"
        cat = catalog_data.get(cj_id, {})
        stories = stories_by_cj.get(cj_id, [])
        cj_bindings = bindings.get(cj_id, {"web_ui": [], "api": [], "cli": [], "agent": []})
        
        classifications = {"canonical": 0, "specialization": 0, "variant": 0, "composition": 0}
        maturities = {"specified": 0, "implemented": 0, "exercised": 0, "evidenced": 0}
        
        story_ids = []
        for s in stories:
            story_ids.append(s["observed_id"])
            c = s["classification"]
            if c in classifications:
                classifications[c] += 1
            m = s["maturity"]
            if m in maturities:
                maturities[m] += 1
        
        contract = {
            "journey_id": cj_id,
            "canonical_journey_name": cat.get("title", cj_id),
            "canonical_source_references": source_hashes,
            "source_story_ids": story_ids,
            "source_story_classifications": classifications,
            "implementation_maturity": maturities,
            "actor_and_role": ACTOR_ROLE_MAP.get(cj_id, {
                "primary_actor": "operator", "primary_role": "operator", "allowed_roles": ["operator"]
            }),
            "starting_state": cat.get("initiating_condition", ""),
            "preconditions": cat.get("preconditions", []),
            "intention": cat.get("intention", ""),
            "job": cat.get("job", ""),
            "authority_requirements": [
                "Strict pre-effect authority evaluation",
                "Attributable actor context and role resolution",
                "Policy digest binding against active snapshot"
            ],
            "expected_capabilities": cat.get("invoked_capabilities", []),
            "declared_interface_bindings": cj_bindings,
            "expected_consequential_state_transitions": cat.get("state_transitions", []),
            "required_artifacts_and_evidence": cat.get("artifacts", []),
            "settlement_condition": cat.get("completion_condition", ""),
            "expected_next_decisions_and_affordances": cat.get("next_decisions", []),
            "applicable_failure_and_recovery_behavior": cat.get("recovery_paths", []),
            "expected_unavailable_states": UNAVAILABLE_STATES_MAP.get(cj_id, [])
        }
        
        out_file = CONTRACTS_DIR / f"{cj_id}.json"
        out_file.write_text(json.dumps(contract, indent=2), encoding="utf-8")
        generated_files.append(out_file)
        print(f"[generator] Wrote {out_file.name} ({len(story_ids)} stories, {sum(len(v) for v in cj_bindings.values())} bindings)")

    print(f"\n[generator] Successfully generated {len(generated_files)} canonical journey contracts.")

if __name__ == "__main__":
    main()

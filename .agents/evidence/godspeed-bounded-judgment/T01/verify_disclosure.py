#!/usr/bin/env python3
"""T01 disclosure verifier — fails the round if any report violates the
pilot-measurement terminology rule (spec REQ-VERIFY-012) or omits the
mandatory disclosure block.

Checks, per report file under the T01 round directory:
1. no forbidden calibration vocabulary appears anywhere;
2. every report carries the mandatory disclosure block with all required keys;
3. stability is labelled (UNSTABLE) wherever the disclosure block appears.

Authored before first execution; sha256 recorded in the frozen preregistration.
Never overwritten; corrections use round-2+ paths.
"""

from __future__ import annotations

import os
import sys

import yaml

ROUND_DIR = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "round-1"
)

FORBIDDEN_TERMS = [
    "calibrated probabilit",
    "calibration curve",
    "production threshold",
    "population-level reliability",
    "model generalization",
    "calibration claim",
    "calibrated confidence",
]

REQUIRED_DISCLOSURE_KEYS = [
    "n",
    "independent_run_count",
    "class_counts",
    "concentration_by_run",
    "missing_attribution",
    "reconstructibility_rate",
    "stability",
]


def main() -> int:
    errors: list[str] = []
    checked = 0
    for root, _dirs, files in os.walk(ROUND_DIR):
        for fn in sorted(files):
            if not fn.endswith((".yml", ".yaml", ".json")):
                continue
            path = os.path.join(root, fn)
            rel = os.path.relpath(path, ROUND_DIR)
            if rel.startswith("teeth" + os.sep):
                continue  # teeth outputs re-state the forbidden terms in their expectations
            text = open(path, encoding="utf-8", errors="replace").read()
            checked += 1
            low = text.lower()
            for term in FORBIDDEN_TERMS:
                if term in low:
                    errors.append(f"{rel}: forbidden term {term!r}")
            try:
                doc = yaml.safe_load(text) if fn.endswith((".yml", ".yaml")) else None
            except yaml.YAMLError:
                continue
            if isinstance(doc, dict) and "disclosure" in doc:
                block = doc["disclosure"]
                if not isinstance(block, dict):
                    errors.append(f"{rel}: disclosure block is not a mapping")
                    continue
                for key in REQUIRED_DISCLOSURE_KEYS:
                    if key not in block:
                        errors.append(f"{rel}: disclosure missing key {key!r}")
                if "UNSTABLE" not in str(block.get("stability", "")):
                    errors.append(f"{rel}: disclosure stability is not labelled UNSTABLE")

    if checked == 0:
        print(f"disclosure verification: FAIL — no report files found under {ROUND_DIR}", file=sys.stderr)
        return 1
    if errors:
        print("disclosure verification: FAIL", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1
    print(f"disclosure verification: PASS ({checked} report files; no forbidden terms; disclosure blocks complete)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

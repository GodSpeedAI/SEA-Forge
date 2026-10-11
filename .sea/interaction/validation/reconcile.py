#!/usr/bin/env python3
"""Reconcile canonicalization-matrix.csv against interaction-model.sea.

DomainForge enforces the aggregate invariants inside the model (see
validation/semantic-teeth.md). It cannot reach the CSV, and it cannot evaluate
arithmetic over instance fields inside a policy `where` predicate
(validation/limitations.md L5). This script closes both gaps:

  * every observed story maps to a declared CJ01-CJ12 journey;
  * per-journey story counts in the model equal the CSV row counts;
  * per-journey maturity distributions in the model equal the CSV;
  * the four maturity counts sum to observed_story_count on every journey;
  * classification and maturity values stay inside the closed vocabularies;
  * interface_binding_count equals the declared InterfaceBinding rows.

Usage:  python3 .sea/interaction/validation/reconcile.py
Exit 0 when the model and the matrix agree.
"""

from __future__ import annotations

import csv
import pathlib
import re
import sys
from collections import Counter, defaultdict

HERE = pathlib.Path(__file__).resolve().parent
SEA = HERE.parent / "interaction-model.sea"
CSV_PATH = HERE.parent / "canonicalization-matrix.csv"

INSTANCE = re.compile(r'instance\s+(\w+)\s+of\s+"(\w+)"\s*\{(.*?)\n\}', re.S)
FIELD = re.compile(r'^\s{4}(\w+):\s*(.*?)(?=\n\s{4}\w+:|\Z)', re.S | re.M)


def parse_instances(text: str) -> list[dict]:
    out = []
    for name, typ, body in INSTANCE.findall(text):
        fields = {}
        for key, raw in FIELD.findall(body):
            value = raw.strip().rstrip(",")
            if value.startswith('"') and value.endswith('"'):
                value = value[1:-1]
            fields[key] = value
        out.append({"name": name, "type": typ, "fields": fields})
    return out


def main() -> int:
    errors: list[str] = []
    instances = parse_instances(SEA.read_text(encoding="utf-8"))
    journeys = {
        x["fields"]["journey_id"]: x["fields"]
        for x in instances
        if x["type"] == "CanonicalJourney"
    }
    bindings = [x for x in instances if x["type"] == "InterfaceBinding"]
    classes = {
        x["fields"]["class_value"]: int(x["fields"]["observed_rows"])
        for x in instances
        if x["type"] == "CanonicalizationClassification"
    }

    with CSV_PATH.open(encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))

    # 1. Row identity and completeness.
    ids = [r["Observed Journey ID"] for r in rows]
    if len(ids) != 128:
        errors.append(f"expected 128 observed rows, found {len(ids)}")
    duplicates = [i for i, n in Counter(ids).items() if n > 1]
    if duplicates:
        errors.append(f"duplicate observed story IDs: {sorted(duplicates)}")

    # 2. Every row maps to a journey the model declares.
    for row in rows:
        cj = row["Canonical Journey ID"]
        if cj not in journeys:
            errors.append(f"story {row['Observed Journey ID']} maps to undeclared journey {cj}")

    # 3. Closed vocabularies in the CSV match the model's enums.
    legal_class = set(classes)
    legal_maturity = {"declared", "specified", "implemented", "exercised", "evidenced"}
    for row in rows:
        if row["Classification"] not in legal_class:
            errors.append(
                f"story {row['Observed Journey ID']} has classification "
                f"{row['Classification']!r} outside the closed vocabulary"
            )
        if row["Implementation Maturity"] not in legal_maturity:
            errors.append(
                f"story {row['Observed Journey ID']} has maturity "
                f"{row['Implementation Maturity']!r} outside the closed vocabulary"
            )

    # 4. Per-journey story counts and maturity distributions.
    csv_stories = Counter(r["Canonical Journey ID"] for r in rows)
    csv_maturity: dict[str, Counter] = defaultdict(Counter)
    for row in rows:
        csv_maturity[row["Canonical Journey ID"]][row["Implementation Maturity"]] += 1

    for cj, fields in sorted(journeys.items()):
        declared = int(fields["observed_story_count"])
        if declared != csv_stories[cj]:
            errors.append(
                f"{cj} declares observed_story_count {declared} "
                f"but the matrix has {csv_stories[cj]} rows"
            )
        parts = {
            m: int(fields[f"{m}_story_count"])
            for m in ("specified", "implemented", "exercised", "evidenced")
        }
        if sum(parts.values()) != declared:
            errors.append(
                f"{cj} maturity counts {parts} do not sum to observed_story_count {declared}"
            )
        for maturity, count in parts.items():
            if count != csv_maturity[cj][maturity]:
                errors.append(
                    f"{cj} declares {count} {maturity} stories "
                    f"but the matrix has {csv_maturity[cj][maturity]}"
                )

    # 5. Classification census matches the matrix.
    csv_classes = Counter(r["Classification"] for r in rows)
    for value, declared in sorted(classes.items()):
        if declared != csv_classes[value]:
            errors.append(
                f"classification {value} declares {declared} rows "
                f"but the matrix has {csv_classes[value]}"
            )

    # 6. Interface binding counts agree with the declared binding rows.
    actual_bindings = Counter(b["fields"]["journey"] for b in bindings)
    for cj, fields in sorted(journeys.items()):
        declared = int(fields["interface_binding_count"])
        if declared != actual_bindings[cj]:
            errors.append(
                f"{cj} declares interface_binding_count {declared} "
                f"but {actual_bindings[cj]} InterfaceBinding instances reference it"
            )

    print(f"observed stories:      {len(ids)}")
    print(f"canonical journeys:    {len(journeys)}")
    print(f"interface bindings:    {len(bindings)}")
    print(f"classification values: {len(classes)}")
    if errors:
        print()
        for message in errors:
            print(f"MISMATCH {message}")
        print(f"\n{len(errors)} mismatch(es)")
        return 1
    print("\nmodel and matrix agree on every reconciled quantity")
    return 0


if __name__ == "__main__":
    sys.exit(main())

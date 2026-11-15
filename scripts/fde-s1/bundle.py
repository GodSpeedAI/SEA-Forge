#!/usr/bin/env python3
"""Collect the S1 evidence bundle for git: scripts/fde-s1/bundle.py <cell> <proof-out-dir> <dest>.

Copies the results file, per-test logs, the cell's inputs, results, steps, gaps, state, raw records (SEA-Forge case
records and spools, Cognate run records, RealityTrace outputs, model validation outputs) and writes
`raw-manifest.sha256` (sha256 of every kept raw file in the cell, including those NOT copied). Files over 1 MB are
referenced by digest only (listed in `raw-not-copied.txt` with their size and the cell path they live at). The
cell and the RealityTrace ledger stay in the scratch directory; nothing here is edited."""
import hashlib, json, pathlib, shutil, sys

cell, proof, dest = (pathlib.Path(x) for x in sys.argv[1:4])
dest.mkdir(parents=True, exist_ok=True)
man = json.loads((cell / "manifest.json").read_text())["files"]
lines, skipped = [], []
for rel, h in sorted(man.items()):
    lines.append(f"{h}  {rel}")
    src = cell / rel
    if rel.startswith("rt-home/") or src.stat().st_size > 1_000_000:
        skipped.append(f"{rel}\t{src.stat().st_size}\t{cell}")
        continue
    (dest / "operational" / rel).parent.mkdir(parents=True, exist_ok=True); shutil.copy2(src, dest / "operational" / rel)
(dest / "raw-manifest.sha256").write_text("\n".join(lines) + "\n")
(dest / "raw-not-copied.txt").write_text("# path<TAB>bytes<TAB>cell directory (scratch; kept outside git)\n" + "\n".join(skipped) + "\n")
for f in ("steps.jsonl", "gaps.jsonl", "state.json", "manifest.json"):
    shutil.copy2(cell / f, dest / "operational" / f)
(dest / "verification").mkdir(exist_ok=True)
for f in proof.glob("*"):
    if f.is_file(): shutil.copy2(f, dest / "verification" / f.name)
vdir = cell.parent / (cell.name + "-verify")
if (vdir / "report.json").exists(): shutil.copy2(vdir / "report.json", dest / "verification" / "verifier-report.json")
print(f"bundle: {len(lines)} raw files in manifest, {len(skipped)} referenced by digest only")

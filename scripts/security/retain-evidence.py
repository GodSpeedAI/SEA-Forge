#!/usr/bin/env python3
"""Retain security evidence outside Git, with a hash manifest that can be.

Copies evidence directories (passing, failed, invalid and inconclusive runs
alike; nothing is filtered) into an owner-only archive and writes a manifest:
per-file SHA-256 and size, git provenance of the producing tree, and the
ledger/run identifiers the v1 runner recorded. The manifest carries digests
and identifiers only, never file content, so it can be committed without
widening any secret-scan allowlist.

  retain-evidence.py archive --out MANIFEST [--archive DIR] EVIDENCE_DIR...
  retain-evidence.py verify MANIFEST [--archive DIR]

The archive defaults to $SEA_FORGE_EVIDENCE_ARCHIVE or
~/.local/share/sea-forge/evidence-archive. Directories are 0700, files 0600.
`verify` re-hashes every archived file and exits non-zero on any missing,
changed or extra file.
"""
import argparse
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


def default_archive() -> Path:
    env = os.environ.get("SEA_FORGE_EVIDENCE_ARCHIVE")
    return Path(env) if env else Path.home() / ".local/share/sea-forge/evidence-archive"


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def provenance() -> dict:
    status = git("status", "--porcelain=v1", "--untracked-files=all")
    return {
        "head": git("rev-parse", "HEAD").strip(),
        # Commit ids change under squash merges; tree ids of the same bytes
        # do not, so evidence can be matched after integration.
        "head_tree": git("rev-parse", "HEAD^{tree}").strip(),
        "branch": git("rev-parse", "--abbrev-ref", "HEAD").strip(),
        # The producing tree was usually uncommitted: digest exactly what
        # differed from HEAD so a later commit can be matched to it.
        "worktree_diff_sha256": hashlib.sha256(git("diff", "HEAD", "--binary").encode()).hexdigest(),
        "worktree_status_sha256": hashlib.sha256(status.encode()).hexdigest(),
        "dirty": bool(status.strip()),
    }


def run_refs(directory: Path) -> list:
    """Observer/run identifiers recorded by the v1 runner, when present."""
    refs = []
    for results in sorted(directory.rglob("results.json")):
        try:
            data = json.loads(results.read_text())
        except (OSError, ValueError):
            continue
        if isinstance(data, dict) and "observer_run_id" in data:
            refs.append({
                "results": str(results),
                "observer_run_id": data.get("observer_run_id"),
                "baseline_commit": data.get("baseline_commit"),
                "recorded_at": data.get("recorded_at"),
                "command_log_sha256": [c.get("log_sha256") for c in data.get("commands", [])],
            })
    return refs


def restrict(root: Path) -> None:
    for path in [root, *root.rglob("*")]:
        if path.is_symlink():
            continue
        path.chmod(0o700 if path.is_dir() else 0o600)


def archive(args) -> int:
    os.umask(0o077)
    store = Path(args.archive or default_archive())
    store.mkdir(parents=True, exist_ok=True)
    store.chmod(0o700)
    repo = Path(git("rev-parse", "--show-toplevel").strip())
    entries, sources = [], []
    for source in args.evidence:
        src = Path(source).resolve()
        rel = src.relative_to(repo)
        dest = store / rel
        if dest.exists():
            print(f"refusing to overwrite archived {rel}", file=sys.stderr)
            return 2
        shutil.copytree(src, dest, symlinks=True)
        restrict(dest)
        sources.append({"path": str(rel), "runs": run_refs(src)})
        for path in sorted(p for p in src.rglob("*") if p.is_file() and not p.is_symlink()):
            entries.append({
                "path": str(path.relative_to(repo)),
                "sha256": sha256_file(path),
                "size": path.stat().st_size,
            })
    manifest = {
        "schema": "sea-forge.evidence-retention/v1",
        "archived_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "archive": "owner-only store outside Git; location is per host",
        "provenance": provenance(),
        "sources": sources,
        "files": entries,
    }
    text = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    (store / f"MANIFEST-{manifest['provenance']['head'][:12]}-{len(entries)}.json").write_text(text)
    Path(args.out).write_text(text)
    print(f"archived {len(entries)} files from {len(sources)} directories")
    return 0


def verify(args) -> int:
    store = Path(args.archive or default_archive())
    manifest = json.loads(Path(args.manifest).read_text())
    expected = {e["path"]: e for e in manifest["files"]}
    problems, good = [], 0
    for rel, entry in expected.items():
        path = store / rel
        if not path.is_file():
            problems.append(f"missing {rel}")
        elif sha256_file(path) != entry["sha256"]:
            problems.append(f"changed {rel}")
        elif path.stat().st_mode & 0o077:
            problems.append(f"not owner-only {rel}")
        else:
            good += 1
    for source in manifest["sources"]:
        root = store / source["path"]
        for path in root.rglob("*"):
            if path.is_file() and str(path.relative_to(store)) not in expected:
                problems.append(f"extra {path.relative_to(store)}")
    for problem in problems:
        print(problem)
    print(f"verified {good}/{len(expected)} files, {len(problems)} problems")
    return 1 if problems else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    a = sub.add_parser("archive")
    a.add_argument("--out", required=True)
    a.add_argument("--archive")
    a.add_argument("evidence", nargs="+")
    v = sub.add_parser("verify")
    v.add_argument("manifest")
    v.add_argument("--archive")
    args = parser.parse_args()
    return archive(args) if args.command == "archive" else verify(args)


if __name__ == "__main__":
    sys.exit(main())

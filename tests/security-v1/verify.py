"""Verify protected files using a full frozen commit supplied from outside the worktree."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import stat
import subprocess
import sys


def git(*args):
    # Local replacement refs must not reinterpret the externally pinned object.
    return subprocess.check_output(["git", "--no-replace-objects", *args], stderr=subprocess.PIPE)


def verify(commit, root):
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ValueError("supply a full externally retained Git commit, not HEAD, a tag or a branch")
    resolved = git("rev-parse", "--verify", commit + "^{commit}").decode().strip()
    if resolved != commit:
        raise ValueError("pin must identify a commit object directly")
    manifest_path = "tests/security-v1/manifest.json"
    manifest_bytes = git("show", commit + ":" + manifest_path)
    manifest = json.loads(manifest_bytes)
    git("merge-base", "--is-ancestor", manifest["baseline_commit"], commit)
    problems = []
    protected = dict(manifest["files"])
    # The manifest cannot contain its own digest; authenticate its exact bytes
    # using the externally pinned commit and verify the editable copy too.
    protected[manifest_path] = {"sha256": hashlib.sha256(manifest_bytes).hexdigest(), "mode": "100644"}
    for name, entry in sorted(protected.items()):
        relative = Path(name)
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("unsafe protected path in pinned manifest")
        path = root / relative
        try:
            if path.is_symlink() or any(parent.is_symlink() for parent in path.parents if parent != root.parent):
                raise ValueError("symbolic link in protected path")
            mode = path.stat().st_mode
            actual_mode = "100755" if mode & 0o111 else "100644"
            pinned = git("show", commit + ":" + name)
            object_mode = git("ls-tree", commit, "--", name).decode().split()[0]
            if hashlib.sha256(pinned).hexdigest() != entry["sha256"] or object_mode != entry["mode"]:
                raise ValueError("pinned manifest/tree disagree")
            if not stat.S_ISREG(mode) or hashlib.sha256(path.read_bytes()).hexdigest() != entry["sha256"] or actual_mode != entry["mode"]:
                raise ValueError("working file differs from frozen hash or mode")
        except (OSError, ValueError, IndexError, subprocess.SubprocessError) as error:
            problems.append(name + ": " + str(error))
    return problems


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--commit", required=True, help="full externally pinned frozen commit SHA")
    args = parser.parse_args()
    try:
        root = Path(git("rev-parse", "--show-toplevel").decode().strip())
        problems = verify(args.commit, root)
        if problems:
            print("INTEGRITY FAILURE\n" + "\n".join(problems))
            return 1
        print("Frozen security-v1 files match externally supplied commit " + args.commit)
        return 0
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print("INTEGRITY BLOCKED: " + str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())

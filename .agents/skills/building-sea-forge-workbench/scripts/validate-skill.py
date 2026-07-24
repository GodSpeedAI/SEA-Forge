#!/usr/bin/env python3
"""Deterministic validator for the building-sea-forge-workbench skill.

Stdlib only. Exits nonzero with actionable errors on any failure.
Run from anywhere: python3 .agents/skills/building-sea-forge-workbench/scripts/validate-skill.py
"""
import re
import sys
from pathlib import Path

SKILL_DIR = Path(__file__).resolve().parent.parent
ERRORS: list[str] = []


def err(msg: str) -> None:
    ERRORS.append(msg)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        err(f"cannot read {path}: {exc}")
        return ""


# --- SKILL.md existence, frontmatter, size -------------------------------
skill_md = SKILL_DIR / "SKILL.md"
if not skill_md.is_file():
    err("SKILL.md missing")
    print("\n".join(ERRORS))
    sys.exit(1)

text = read(skill_md)
lines = text.splitlines()

fm = re.match(r"^---\n(.*?)\n---\n", text, re.DOTALL)
if not fm:
    err("SKILL.md: missing YAML frontmatter block delimited by ---")
else:
    fields = {}
    for line in fm.group(1).splitlines():
        m = re.match(r"^([A-Za-z0-9_-]+):\s*(.*)$", line)
        if m:
            fields[m.group(1)] = m.group(2).strip()
    name = fields.get("name", "")
    desc = fields.get("description", "")
    if not name:
        err("frontmatter: 'name' missing")
    elif not re.fullmatch(r"[a-z0-9-]+", name):
        err(f"frontmatter: name '{name}' must be lowercase letters, numbers, hyphens only")
    elif name != "building-sea-forge-workbench":
        err(f"frontmatter: name is '{name}', expected 'building-sea-forge-workbench'")
    if not desc:
        err("frontmatter: 'description' missing")
    else:
        if len(desc) > 1024:
            err(f"frontmatter: description {len(desc)} chars, exceeds 1024")
        if re.match(r"^(I|You|We)\b", desc):
            err("frontmatter: description must be third-person")
        if "Use when" not in desc:
            err("frontmatter: description must state when to invoke ('Use when ...')")

body_lines = len(lines)
if body_lines >= 500:
    err(f"SKILL.md is {body_lines} lines; must be under 500")

# --- reference files exist, links resolve, depth rules -------------------
REQUIRED_REFS = [
    "source-map.md",
    "repository-integration.md",
    "stack-and-dependencies.md",
    "implementation-workflow.md",
    "api-and-event-contracts.md",
    "design-and-ux-contract.md",
    "thoth-interaction-contract.md",
    "testing-and-settlement.md",
]
for ref in REQUIRED_REFS:
    p = SKILL_DIR / "reference" / ref
    if not p.is_file():
        err(f"required reference file missing: reference/{ref}")
    if f"reference/{ref}" not in text:
        err(f"SKILL.md does not link reference/{ref}")

all_md = [skill_md] + sorted((SKILL_DIR / "reference").glob("*.md")) + sorted(
    (SKILL_DIR / "evals").glob("*.md")
)
corpus = {p: read(p) for p in all_md}

for p, body in corpus.items():
    rel_name = p.relative_to(SKILL_DIR)
    if "\\" in re.sub(r"```.*?```", "", body, flags=re.DOTALL).replace("\\n", ""):
        # backslash outside code fences suggests Windows paths
        stripped = re.sub(r"```.*?```", "", body, flags=re.DOTALL)
        if re.search(r"[A-Za-z0-9_.]\\[A-Za-z0-9_.]", stripped):
            err(f"{rel_name}: backslash path detected")
    for m in re.finditer(r"\]\((?!https?://|#)([^)]+)\)", body):
        target = m.group(1).split("#")[0]
        if not target:
            continue
        if target.startswith("/home/") or target.startswith("/Users/"):
            err(f"{rel_name}: machine-specific absolute path link: {target}")
            continue
        resolved = (p.parent / target)
        if not resolved.exists():
            err(f"{rel_name}: broken relative link: {target}")
        # depth rule: skill files reference at most one level below the skill dir
        try:
            rel = resolved.resolve().relative_to(SKILL_DIR)
            if len(rel.parts) > 2:
                err(f"{rel_name}: reference deeper than one level: {target}")
        except ValueError:
            pass  # points outside the skill dir (repo paths are fine)
    for m in re.finditer(r"/home/\w+/|/Users/\w+/", body):
        err(f"{rel_name}: machine-specific absolute path text: {m.group(0)}")
        break

# --- evaluations ---------------------------------------------------------
evals = SKILL_DIR / "evals" / "evaluations.md"
if not evals.is_file():
    err("evals/evaluations.md missing")
else:
    n = len(re.findall(r"^##\s+Evaluation\s+\d+", corpus[evals], re.MULTILINE))
    if n < 3:
        err(f"evals/evaluations.md declares {n} evaluations; need at least 3")

# --- validator itself + plan pointer -------------------------------------
if not (SKILL_DIR / "scripts" / "validate-skill.py").is_file():
    err("scripts/validate-skill.py missing (self-check)")

plan_glob = list((SKILL_DIR / ".." / ".." / "plans").resolve().glob(
    "*sea-forge-workbench-frontend-api-implementation*.md"
))
if not plan_glob:
    err("implementation plan not found under .agents/plans/ "
        "(*sea-forge-workbench-frontend-api-implementation*.md)")

# --- stale package references must not be copied -------------------------
STALE = ["css.txt", "preview/manifest.json", "preview/colors-primary",
         "context/provenance.md", "assets/README.md"]
for p, body in corpus.items():
    stripped = body
    for s in STALE:
        for m in re.finditer(re.escape(s), stripped):
            ctx = stripped[max(0, m.start() - 200):m.end() + 200]
            # allowed only when explicitly marked as a documented former/stale name
            if not re.search(r"renam|stale|former|was |never existed|do not exist|deliver|→",
                             ctx, re.IGNORECASE):
                err(f"{p.relative_to(SKILL_DIR)}: stale package reference copied: {s}")

# --- locked stack represented, exclusions not recommended ----------------
stack_corpus = corpus[SKILL_DIR / "reference" / "stack-and-dependencies.md"] + text
for token in ["Tauri 2", "Bun", "React 19", "Vite", "@tanstack/react-router",
              "@tanstack/react-query", "xstate", "react-hook-form", "ajv",
              "@astryxdesign/core", "Vitest", "Playwright", "axe-core",
              "Storybook"]:
    if token.lower() not in stack_corpus.lower():
        err(f"locked dependency not represented in skill: {token}")

full_corpus = "\n".join(corpus.values())
for banned in ["Tailwind", "shadcn", "Redux", "Copilot Cloud"]:
    for m in re.finditer(re.escape(banned), full_corpus):
        ctx = full_corpus[max(0, m.start() - 500):m.start() + 200]
        if not re.search(r"exclud|do not|never|prohibit|no |not |avoid|banned|instead of",
                         ctx, re.IGNORECASE):
            err(f"excluded dependency appears to be recommended: {banned}")
            break

# --- policy assertions ----------------------------------------------------
thoth = corpus[SKILL_DIR / "reference" / "thoth-interaction-contract.md"]
if "ThothInteractionPort" not in thoth or "CopilotKitAdapter" not in thoth:
    err("thoth-interaction-contract.md must define ThothInteractionPort and CopilotKitAdapter")
if "DirectAgUiAdapter" not in thoth:
    err("thoth-interaction-contract.md must keep DirectAgUiAdapter as canonical fallback")
if not re.search(r"CopilotKit.*(may not own|never grants|not.*approval authority)",
                 thoth, re.IGNORECASE | re.DOTALL):
    err("thoth-interaction-contract.md must prohibit CopilotKit canonical/approval ownership")

if not re.search(r"Bun.*canonical|canonical.*Bun", stack_corpus, re.IGNORECASE | re.DOTALL):
    err("stack: Bun must be stated as canonical frontend runtime/package manager")
if not re.search(r"Astryx.*(substrate|not semantic owner|SEA Forge owns)",
                 stack_corpus + corpus[SKILL_DIR / "reference" / "design-and-ux-contract.md"],
                 re.IGNORECASE | re.DOTALL):
    err("design/stack: Astryx must be positioned as substrate, not semantic owner")

# --- verdict --------------------------------------------------------------
if ERRORS:
    print(f"FAIL — {len(ERRORS)} problem(s):")
    for e in ERRORS:
        print(f"  - {e}")
    sys.exit(1)
print("OK — building-sea-forge-workbench skill validates clean "
      f"({body_lines} SKILL.md lines, {len(REQUIRED_REFS)} references, evals present)")

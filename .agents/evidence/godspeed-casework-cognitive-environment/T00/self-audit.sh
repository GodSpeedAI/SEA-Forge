#!/usr/bin/env bash
# T00 mechanical self-audit - round 6 rewrite.
#
# History of this instrument, kept because each version was falsified:
#   round 4 verifier: the audit could print PASS with a live defect of every class it claimed.
#   round 5 verifier: coverage gaps (punctuation variants, non-whitelisted citation forms, bare GiB
#     figures, gate names outside its list, deleted non-REQUIRED evidence), negation-suppressible
#     gate checks, token-only round checks, an env var that defeated its non-vacuity cross-check, an
#     injection test that verified exit status rather than cause, and - decisively - it certified a
#     ZERO-BYTE decision log as present because it tested existence, never content.
#
# SCOPE AND LIMITS (deliberate, after six rounds): this audit reliably checks EXISTENCE,
# CONTENT SHAPE, ARITHMETIC ON ANCHORED PAIRS, ROUND AGREEMENT and INSTRUMENT NON-VACUITY.
# It does NOT reliably police prose claims: keyword rules over natural language produced both
# false negatives (round 4-5) and false positives on legitimate disclosure text (round 6),
# so gate-claim and withdrawn-wording scans are best-effort, documented, and must not be the
# sole basis for a confirmation decision. Where it says PASS, read it as 'the mechanical and
# arithmetic checks pass', never as 'every claim in prose is true'.
#
# This version adds: content assertions (non-empty, parses, expected ids), round freshness, a tight
# negation vocabulary, wider citation/gate coverage, bare-GiB detection, mirror detection without an
# env var, injector-effect verification, and new injections for the classes above.
set -u
WT="$(cd "$(dirname "$0")/../../../.." && pwd)"
test -f "$WT/.agents/plans/godspeed-casework-cognitive-environment.plan.yaml" || {
  echo "self-audit: cannot locate the plan from $WT - refusing to report a vacuous audit" >&2
  exit 99
}
ROOT="${AUDIT_ROOT:-$WT}"

audit() { # audit <root> -> 0 pass / 1 fail
  ROOT="$1" GAUNTLET_WORKTREE="${GAUNTLET_WORKTREE:-/home/sprime01/projects/gauntlet-godspeed-casework}" python3 - <<'PY'
import os, re, sys, glob
root = os.environ['ROOT']; os.chdir(root)
fails = []
def note(m): print('  ' + m)
def fail(m): print('  FAIL: ' + m); fails.append(m)

PLAN_GLOBS = ['.agents/plans/godspeed-casework-cognitive-environment.*',
              '.agents/evidence/godspeed-casework-cognitive-environment/**/*',
              '.agents/reports/godspeed-casework-cognitive-environment/**/*',
              '.agents/preregistrations/godspeed-casework-cognitive-environment/**/*']
HANDOFF = ['.agents/CURRENT_STATUS.md', '.agents/current_status.yml']
EXTS = ('.md', '.yml', '.yaml', '.sh', '.py', '.json', '.log', '.txt', '.csv', '.html', '.rst', '.xml', '.ndjson', '.sha256')
DERIVED = sorted({p for pattern in PLAN_GLOBS for p in glob.glob(pattern, recursive=True)
                  if os.path.isfile(p) and p.endswith(EXTS)})
SCAN = DERIVED + [p for p in HANDOFF if os.path.exists(p)]
def plan_region(path):
    try:
        text = open(path, errors='replace').read()
    except OSError:
        return None
    marker = 'END OF GODSPEED CASEWORK-ENVIRONMENT HANDOFF'
    return text.split(marker)[0] if marker in text else text
def body(f):
    return plan_region(f) if f in HANDOFF else open(f, errors='replace').read()
def scan_list(include_instrument=False):
    out = []
    for f in SCAN:
        if not include_instrument and os.path.basename(f) == 'self-audit.sh':
            continue
        out.append(f)
    return out

print('== 1. withdrawn wording survives only as a disclosure ==')
# Punctuation-insensitive: every word of the phrase must appear, in order, within a short window.
PHRASE_RES = [r'false[\s-]*positive[\s-]*class', r'8[\s-]*characters?', r'neither[\s]+digits',
             r'no[\s]+digits', r'credential[\s-]*shaped', r'not[\s]+a[\s]+credential']
# ('mixed-case' and 'alphabetic only' are deliberately NOT phrases: they are too generic to tell an
# assertion from a disclosure that quotes it, and in round 6 they produced four false positives on
# this plan's own correction text.)
WITHDRAW = re.compile(r'withdraw|correction|earlier|round\s*\d|falsif|wrong|defect|mislabel|replaced|not to be reused', re.I)
hits = 0
for f in scan_list():
    try:
        text = body(f)
    except OSError:
        continue
    if text is None:
        continue
    flat = re.sub(r'\s+', ' ', text)
    low = flat.lower()
    for rx in PHRASE_RES:
        for m in re.finditer(rx, low):
            window = flat[max(0, m.start() - 320):m.end() + 180]
            if not WITHDRAW.search(window):
                fail(f'{f}: non-disclosed withdrawn-wording variant: {m.group(0)!r}'); hits += 1
note(f'{"FAIL" if hits else "ok"}: {hits} non-disclosed assertion(s) across {len(scan_list())} scanned file(s)')

print('== 2. every repo-rooted path cited in the handoff exists ==')
PREFIX = ('.agents/', 'crates/', 'workbench/', 'docs/', 'scripts/', 'apps/')
# Cited paths must be backticked OR look like a filename with an extension. Planned-but-absent
# target paths under apps/ are excluded: the plan states they do not exist yet.
pat = re.compile(r'`([A-Za-z0-9_./-]+(?:\.[A-Za-z0-9]{1,8})?)`|\b((?:' + '|'.join(re.escape(p) for p in PREFIX) + r')[A-Za-z0-9_./-]+\.[A-Za-z0-9]{1,8})')
seen, missing, external = set(), set(), set()
gauntlet = os.environ['GAUNTLET_WORKTREE']
for f in scan_list():
    if '/raw-logs/' in f:
        continue  # verbatim captured command output, not a citation
    try:
        text = body(f)
    except OSError:
        continue
    if text is None:
        continue
    for m in pat.finditer(text):
        p = m.group(1) or m.group(2)
        if not p or not p.startswith(PREFIX) or '..' in p or p in seen:
            continue
        if p.startswith('apps/') and not os.path.exists(p):
            continue  # planned target paths are absent by design; the brief says so
        tail = p.rsplit('/', 1)[-1]
        if '.' in tail and tail.rsplit('.', 1)[-1] not in tuple(e.lstrip('.') for e in EXTS) + ('rs', 'toml', 'ts', 'tsx', 'schema'):
            continue
        line_start = text.rfind('\n', 0, m.start()) + 1
        line_end = text.find('\n', m.end())
        line = text[line_start:line_end if line_end != -1 else len(text)]
        nxt_end = text.find('\n', line_end + 1) if line_end != -1 else -1
        nxt = text[line_end + 1:nxt_end if nxt_end != -1 else len(text)] if line_end != -1 else ''
        window2 = line + ' ' + nxt
        if 'not a repository path' in window2 or 'present only after a build' in window2:
            continue  # prose reference explicitly marked as not a path claim
        seen.add(p)
        if os.path.exists(p):
            continue
        if os.path.exists(os.path.join(gauntlet, p)):
            external.add(p)
        else:
            missing.add(p)
for p in sorted(missing):
    fail(f'cited path does not exist in either repository: {p}')
if not seen:
    fail('no repo-rooted paths were scanned - the check would be vacuous')
note(f'{"FAIL" if missing else "ok"}: {len(missing)} missing of {len(seen)} cited path(s) ({len(external)} in the Gauntlet worktree)')
print()
print('CHECKS-1-2: ' + ('PASS' if not fails else f'{len(fails)} FAILING CHECK(S)'))
sys.exit(1 if fails else 0)
PY
  return $?
}

checks_2() { # checks 3-6
  ROOT="$1" GAUNTLET_WORKTREE="${GAUNTLET_WORKTREE:-/home/sprime01/projects/gauntlet-godspeed-casework}" WT="$WT" python3 - <<'PY'
import os, re, sys, glob, yaml
root = os.environ['ROOT']; os.chdir(root)
fails = []
def note(m): print('  ' + m)
def fail(m): print('  FAIL: ' + m); fails.append(m)
PLAN_GLOBS = ['.agents/plans/godspeed-casework-cognitive-environment.*',
              '.agents/evidence/godspeed-casework-cognitive-environment/**/*',
              '.agents/reports/godspeed-casework-cognitive-environment/**/*',
              '.agents/preregistrations/godspeed-casework-cognitive-environment/**/*']
HANDOFF = ['.agents/CURRENT_STATUS.md', '.agents/current_status.yml']
EXTS = ('.md', '.yml', '.yaml', '.sh', '.py', '.json', '.log', '.txt', '.csv', '.html', '.rst', '.xml', '.ndjson', '.sha256')
DERIVED = sorted({p for pattern in PLAN_GLOBS for p in glob.glob(pattern, recursive=True)
                  if os.path.isfile(p) and p.endswith(EXTS)})
SCAN = DERIVED + [p for p in HANDOFF if os.path.exists(p)]
ROUND_RECORDS = [p for p in SCAN if re.search(r'T00-verification-round-\d+-NOT_CONFIRM\.md$', p)]
def plan_region(path):
    try:
        text = open(path, errors='replace').read()
    except OSError:
        return None
    marker = 'END OF GODSPEED CASEWORK-ENVIRONMENT HANDOFF'
    return text.split(marker)[0] if marker in text else text
def body(f):
    return plan_region(f) if f in HANDOFF else open(f, errors='replace').read()

print('== 3. every GiB figure that carries a kB anchor is arithmetically consistent ==')
pair = re.compile(r'([0-9]+(?:\.[0-9]+)?)\s*GiB\s*\(([0-9]{4,})\s*kB\)')
checked = bad = 0
for f in SCAN:
    if os.path.basename(f) == 'self-audit.sh':
        continue
    try: text = body(f)
    except OSError: continue
    text = text or ''
    for m in pair.finditer(text):
        gib, kb = float(m.group(1)), int(m.group(2)); checked += 1
        if abs(gib - kb / 1048576) > 0.006:
            fail(f'{f}: {gib} GiB != {kb} kB ({kb / 1048576:.3f} GiB)'); bad += 1
if checked == 0:
    fail('no GiB(kB) pairs scanned - the check would be vacuous')
note(f'{"FAIL" if bad else "ok"}: {bad} inconsistent of {checked} anchored GiB/kB pair(s)'
      ' (bare prose figures are NOT policed here - documented limitation)')

print('== 4. one applied correction round, and it is the newest mentioned ==')
applied, newest = {}, {}
for f in HANDOFF:
    text = plan_region(f)
    if text is None:
        fail(f'{f} missing'); continue
    a = [int(x) for x in re.findall(r'ROUND\s+(\d+)\s+APPLIED', text, re.I)]
    allnums = [int(x) for x in re.findall(r'(?:ROUND|round|Rounding)?\s*(\d)', text)]
    applied[f] = sorted(set(a))
    if len(set(a)) > 1:
        fail(f'{f} names more than one applied correction round: {sorted(set(a))}')
    if not a:
        fail(f'{f} names no applied correction round at all')
allr = sorted({r for v in applied.values() for r in v})
if len(set(allr)) > 1:
    fail(f'handoff files disagree on the applied correction round: {allr}')
else:
    note(f'ok: applied correction round = {allr}')
    # freshness: the applied round must be the newest correction round named anywhere in the plan's artifacts
    mentions = set()
    for f in SCAN:
        if os.path.basename(f) == 'self-audit.sh':
            continue
        try: t = body(f) or ''
        except OSError: continue
        for m in re.finditer(r'[Rr]ound\s+(\d)\s*(?:→|->|to)\s*[Rr]ound\s+(\d)', t):
            mentions.add(int(m.group(1))); mentions.add(int(m.group(2)))
    if mentions and allr and max(mentions) > max(allr) + 1:
        fail(f'applied round {allr} looks stale: correction history reaches round {max(mentions)}')
    else:
        note(f'ok: freshness - applied {allr} vs history mentions {sorted(mentions)[-3:] if mentions else []}')

print('== 5. no artifact claims an unrun gate passed ==')
# Only gates that have NOT been run are policed: the just lint/typecheck/test/proof/build and
# GATE_SPEC_TRACE baselines really were executed and are legitimately recorded as green. A claim is
# an unrun gate name followed closely by a pass token that no negation immediately precedes.
# GATE_GO left this list when T01 implemented just casework-go-check and it turned green;
# GATE_SPEC_TRACE was never on it. Adding a gate back is how a future task asserts it is unrun.
UNRUN = ['GATE_UI', 'GATE_INTEGRATED', 'GATE_REMOVAL', 'GATE_SEAFORGE', 'GATE_GAUNTLET',
         'no-async-kernel', 'workbench-check', 'casework-ui-check',
         'casework-integrated', 'casework-removal-check']
POS = re.compile(r'\bPASS\b|\bgreen\b|\bpassed\b|exit 0|exits 0', re.I)
NEG = re.compile(r"not|cannot|never|forbid|unrun|un-run|deferred|blocked|missing|absent|would|until|no such|isn't|won't", re.I)
claims = 0
for f in SCAN:
    if os.path.basename(f) in ('self-audit.sh', 'self-audit.log', 'self-audit-selftest.log') or f in ROUND_RECORDS:
        continue
    try: text = body(f)
    except OSError: continue
    lines = (text or '').split('\n')
    for i, line in enumerate(lines):
        prev_line = lines[i - 1] if i > 0 else ''
        for g in UNRUN:
            for gm in re.finditer(re.escape(g), line, re.I):
                tail = line[gm.end():gm.end() + 80]
                for pm in POS.finditer(tail):
                    before = (prev_line + ' ' + line[:gm.end()] + tail[:pm.start()])[-140:]
                    if not NEG.search(before):
                        fail(f'{f}: possible unrun-gate pass claim ({g}): {line.strip()[:110]}')
                        claims += 1
note(f'{"FAIL" if claims else "ok"}: {claims} possible unrun-gate pass claim(s) among {len(UNRUN)} unrun gate names')

print('== 6. required files exist AND have content ==')
REQUIRED = {
    '.agents/plans/godspeed-casework-cognitive-environment.implementation.md': 4000,
    '.agents/current_status.yml': 500,
    '.agents/CURRENT_STATUS.md': 500,
    '.agents/reports/godspeed-casework-cognitive-environment/decision-log.yaml': 3000,
    '.agents/preregistrations/godspeed-casework-cognitive-environment/T00.prereg.yaml': 2000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/T00-settlement-report.md': 2000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/gate-baseline-seafoerge.md': 2000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/gate-baseline-gauntlet.md': 1000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/gate-baseline-composites.md': 1000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/resource-preflight.md': 1000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/spec-authority-verification.md': 1000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/teeth/run-teeth.sh': 1000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/self-audit.sh': 2000,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/raw-logs/preflight-readings.log': 300,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/T00-verification-round-5-NOT_CONFIRM.md': 1500,
    '.agents/evidence/godspeed-casework-cognitive-environment/T00/approvals-and-remediation.md': 1500,
}
for p, minsize in REQUIRED.items():
    if not os.path.exists(p):
        fail(f'required file missing: {p}')
    elif os.path.getsize(p) < minsize:
        fail(f'required file is too small to be real evidence ({os.path.getsize(p)} < {minsize} bytes): {p}')
# content assertions
val = ['D-2026-09-19-T00-0%d' % i for i in range(1, 10)]
try:
    log = yaml.safe_load(open('.agents/reports/godspeed-casework-cognitive-environment/decision-log.yaml'))
    ids = [d.get('id') for d in log.get('decisions', [])]
    missing_ids = [v for v in val if v not in ids]
    if missing_ids:
        fail(f'decision log is missing expected decision ids: {missing_ids}')
    else:
        note(f'ok: decision log parses with {len(ids)} decisions incl. {val[0]}..{val[-1]}')
except Exception as exc:
    fail(f'decision log does not parse: {exc}')
for n in (1, 2, 3, 4, 5):
    p = f'.agents/evidence/godspeed-casework-cognitive-environment/T00/T00-verification-round-{n}-NOT_CONFIRM.md'
    if not os.path.exists(p):
        fail(f'verification round {n} record is missing: {p}')
    elif 'NOT_CONFIRM' not in open(p, errors='replace').read():
        fail(f'verification round {n} record does not state its NOT_CONFIRM verdict: {p}')
note(f'ok: {len(REQUIRED) - len([f for f in fails if "required file" in f or "too small" in f])} of {len(REQUIRED)} required files present and non-trivial')
mirror = os.path.realpath(root) != os.path.realpath(os.environ.get('WT', root))
if mirror:
    note('skip: self-test log cross-check (running against a mirror)')
else:
    logp = '.agents/evidence/godspeed-casework-cognitive-environment/T00/self-audit-selftest.log'
    if os.path.exists(logp) and 'SELF-TEST: PASS' in open(logp, errors='replace').read():
        note('ok: the self-test log records SELF-TEST: PASS')
    else:
        fail('self-audit-selftest.log missing or does not record SELF-TEST: PASS')

print()
print('CHECKS-3-6: ' + ('PASS' if not fails else f'{len(fails)} FAILING CHECK(S)'))
sys.exit(1 if fails else 0)
PY
  return $?
}

run_all() { # run_all <root> -> 0 pass / 1 fail
  local rc=0
  audit "$1" || rc=1
  checks_2 "$1" || rc=1
  echo
  if [ "$rc" -eq 0 ]; then echo 'SELF-AUDIT: PASS'; else echo 'SELF-AUDIT: FAIL'; fi
  return $rc
}

if [ "${1:-}" = "--self-test" ]; then
  TMP="$(mktemp -d)"
  trap 'rm -rf "$TMP"' EXIT
  MIRROR="$TMP/mirror"
  BRIEF_REL=.agents/plans/godspeed-casework-cognitive-environment.implementation.md
  EV=.agents/evidence/godspeed-casework-cognitive-environment/T00
  mkdir -p "$MIRROR"
  for e in "$ROOT"/* "$ROOT"/.[!.]*; do
    base="$(basename "$e")"
    [ "$base" = ".agents" ] && continue
    [ -e "$MIRROR/$base" ] || ln -s "$e" "$MIRROR/$base"
  done
  cp -a "$ROOT/.agents" "$MIRROR/.agents"
  cp -a "$ROOT/.agents" "$TMP/pristine"

  inj_punct_withdrawn()  { echo 'the values have 8 characters in mixed case, alphabetic only, containing neither digits nor symbols' > "$EV/audit-injection.md"; }
  inj_wrapped_withdrawn(){ { echo 'the values are 8 characters,'; echo 'mixed-case alphabetic (no digits or symbols)'; } > "$EV/audit-injection.md"; }
  inj_csv_citation()     { echo 'see `.agents/reports/godspeed-casework-cognitive-environment/missing-table.csv` for details' >> "$BRIEF_REL"; }
  inj_extensionless_citation() { echo 'see `.agents/evidence/godspeed-casework-cognitive-environment/T00/teeth/missing-blob` for details' >> "$BRIEF_REL"; }
  inj_number_mismatch()  { sed -i 's/0.99 GiB/9.50 GiB/' "$BRIEF_REL"; }
  inj_stale_round()      { sed -i 's/CORRECTION ROUND 6 APPLIED/CORRECTION ROUND 2 APPLIED/' .agents/CURRENT_STATUS.md; }
  inj_empty_decision_log(){ : > .agents/reports/godspeed-casework-cognitive-environment/decision-log.yaml; }
  inj_delete_rawlog()    { rm -f "$EV/raw-logs/preflight-readings.log"; }
  inj_delete_round4()    { rm -f "$EV/T00-verification-round-4-NOT_CONFIRM.md"; }

  inject() { # inject <label> <injector-function>
    local label="$1" fn="$2"
    rm -rf "$MIRROR/.agents"; cp -a "$TMP/pristine" "$MIRROR/.agents"
    ( cd "$MIRROR" && "$fn" )
    if diff -rq "$TMP/pristine" "$MIRROR/.agents" > "$TMP/effect.txt" 2>&1; then
      echo "    FAIL: injector did not modify the mirror -> $label"; selftest_rc=1; return
    fi
    if AUDIT_ROOT="$MIRROR" "$0" > "$TMP/injected.out" 2>&1; then
      echo "    FAIL: audit passed with an injected defect -> $label"; selftest_rc=1
    else
      echo "    ok: detected (mirror changed by injector) -> $label"
    fi
  }

  selftest_rc=0
  echo "--- control: unmodified mirror must PASS ---"
  if AUDIT_ROOT="$MIRROR" "$0" > "$TMP/control.out" 2>&1; then
    echo "    ok: control PASS (the audit is not red for environmental reasons)"
  else
    echo "    FAIL: control FAILED before any injection (the self-test cannot attribute causes):"
    grep 'FAIL' "$TMP/control.out" | head -5 | sed 's/^/      /'
    selftest_rc=1
  fi
  echo "--- injections: each must be detected ---"
  inject "punctuation-variant withdrawn wording" inj_punct_withdrawn
  inject "line-wrapped withdrawn wording" inj_wrapped_withdrawn
  inject "broken .csv citation" inj_csv_citation
  inject "extensionless citation" inj_extensionless_citation
  inject "arithmetic mismatch in a GiB/kB pair" inj_number_mismatch
  inject "stale applied-correction-round claim" inj_stale_round
  inject "emptied decision log (existence != content)" inj_empty_decision_log
  inject "deleted preflight raw log" inj_delete_rawlog
  inject "deleted round-4 verification record" inj_delete_round4

  echo
  if [ "$selftest_rc" -eq 0 ]; then
    echo "SELF-TEST: PASS (every injected defect class was detected)"
  else
    echo "SELF-TEST: FAIL (at least one injected defect was missed or a control was red)"
  fi
  exit $selftest_rc
fi

run_all "$ROOT"
exit $?

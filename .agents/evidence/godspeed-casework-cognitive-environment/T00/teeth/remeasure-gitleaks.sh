#!/usr/bin/env bash
# T00 correction rounds 1/2/4 - honest, non-exposing re-measurement of the gitleaks findings.
#
# History this script must not repeat:
#   round 0 read the `Secret` field of a `gitleaks --redact` report, which is the literal string
#     "REDACTED" for EVERY finding, and reported "8 chars, mixed-case alphabetic" as if measured.
#   round 2 fixed the span lengths but still claimed "values are lower alnum with hyphens / all
#     contain digits" while the log could not show it - it masked alphanumeric runs, so the claim
#     was true but unbacked by the cited log (found by independent verification round 3).
#
# This version measures the VALUE (the quoted string following the matched field name), reports
# its class and digit/hyphen presence from the real bytes, and refuses to characterise anything
# it cannot resolve. No matched value is ever printed.
set -u
WT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
REPORT="${1:-$WT/.agents/evidence/godspeed-casework-cognitive-environment/T00/raw-logs/gitleaks.json}"
python3 - "$REPORT" "$WT" <<'PY'
import json, re, sys, collections

report, wt = sys.argv[1], sys.argv[2]
with open(report) as fh:
    d = json.load(fh)
print(f"report: {report}")
print(f"findings: {len(d)}")
if not d:
    print("REFUSING: the report contains no findings, so there is nothing to characterise. "
          "An empty report is not evidence that the gate is clean.", file=sys.stderr)
    sys.exit(2)

def enclosing_field(line, sc):
    left = line.rfind('"', 0, sc - 1)
    if left == -1:
        return None, None
    right = line.find('"', left + 1)
    if right == -1:
        return None, None
    return line[left + 1:right], right

def value_after(line, right):
    colon = line.find(':', right)
    if colon == -1:
        return None
    vopen = line.find('"', colon + 1)
    if vopen == -1:
        return None
    vclose = line.find('"', vopen + 1)
    if vclose == -1:
        return None
    return line[vopen + 1:vclose]

rows, unresolved = [], []
by_class = collections.Counter()
by_span = collections.Counter()
for x in d:
    path = f"{wt}/{x['File']}"
    start_line, end_line = x.get('StartLine'), x.get('EndLine')
    sc, ec = x.get('StartColumn'), x.get('EndColumn')
    try:
        with open(path, encoding='utf-8', errors='replace') as fh:
            lines = fh.read().splitlines()
        line = lines[start_line - 1]
    except (OSError, IndexError) as exc:
        unresolved.append(f"{x['File']}:{start_line} READ_ERROR {exc}")
        continue
    span = line[sc - 1:ec - 1] if start_line == end_line else "<multi-line>"
    if not span.strip():
        unresolved.append(f"{x['File']}:{start_line} EMPTY_SPAN (columns point at whitespace)")
        continue
    field, right = enclosing_field(line, sc)
    value = value_after(line, right) if right is not None else None
    if not field or value is None:
        unresolved.append(f"{x['File']}:{start_line} VALUE_UNRESOLVED (field={field!r})")
        continue
    if re.fullmatch(r'[a-z0-9-]+', value):
        cls = 'lower-alnum-hyphen'
    elif re.fullmatch(r'[A-Za-z0-9-]+', value):
        cls = 'alnum-hyphen'
    else:
        cls = 'other'
    by_class[cls] += 1
    by_span[f"span={len(span)}"] += 1
    rows.append((x['File'], start_line, len(span), field, len(value), cls,
                 str(any(c.isdigit() for c in value)), str('-' in value), x.get('Entropy')))

print("span lengths:", dict(sorted(by_span.items(), key=lambda kv: int(kv[0].split('=')[1]))))
print("value classes:", dict(by_class))
print("fields:", dict(collections.Counter(r[3] for r in rows)))
print("value lengths:", dict(sorted(collections.Counter(r[4] for r in rows).items())))
print("values containing a digit:", dict(collections.Counter(r[6] for r in rows)))
print("values containing a hyphen:", dict(collections.Counter(r[7] for r in rows)))
print("entropy range:",
      (min(r[8] for r in rows), max(r[8] for r in rows)) if rows else None,
      "| commits:", dict(collections.Counter(str(x.get('Commit', ''))[:10] for x in d)))
print()
for f, ln, sl, field, vl, cls, dig, hyp, ent in rows:
    print(f"  {f}:{ln}  span={sl}  field={field!r}  value_len={vl}  class={cls}  digit={dig}  hyphen={hyp}  entropy={ent}")
if unresolved:
    print()
    print("UNRESOLVED FINDINGS - refusing to characterise this report:", file=sys.stderr)
    for line in unresolved:
        print(f"  {line}", file=sys.stderr)
    sys.exit(2)
PY

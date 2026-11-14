#!/usr/bin/env bash
# Publish and verify external anchors: annotated git tags `fde-anchor/<kind>/<id>`
# whose message is a JSON document of contracts/fde/anchor.v1.json (RD-04).
#
#   scripts/fde-anchor.sh manifest <id>                    write MANIFEST.sha256 for .agents/preregistrations/<id>/ and print its digest
#   scripts/fde-anchor.sh publish <kind> <id> <digest> [--push]
#   scripts/fde-anchor.sh verify <id | kind/id> [--remote]   a bare <id> means prereg/<id>
#
# kind: model | freeze | prereg. id: a ref-safe token [A-Za-z0-9][A-Za-z0-9._-]{0,127}.
# Digest of a prereg: sha256 of .agents/preregistrations/<id>/MANIFEST.sha256, which lists
# the sha256 of every other file in the directory, sorted bytewise. Verifying freeze and
# model anchors needs the case keeper and registry (WP-14) and is refused here.
#
# publish creates the tag on the current commit, refuses an existing tag (local, or on
# origin with --push) and never moves or force-pushes one. The tag message is the anchor
# document without `verifier_receipt` (the receipt cannot sit inside the tag object it
# describes); the publication time is the tag's tagger date. `verify` recomputes the digest,
# compares it to the tag, and with --remote also requires the same tag object on origin.
# On a mismatch it prints which file changed and exits 1.
#
# LIMIT (RD-26): on a single-operator host the pusher and the verifier are the same
# principal and OS user (credential_scope shared_os_user). The anchor proves "not
# changed since publication", not independence; anchoring rows stay PARTIAL until a
# dedicated anchor credential and a protected fde-anchor/* namespace exist (OD-6). The tag
# is unsigned and the server push time is not read (no GitHub API here).
#
# Exit: 0 ok, 1 verification failure, 2 usage or environment, 3 publish refused.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
cmd_re='^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$'
die() { printf 'fde-anchor: %s\n' "$*" >&2; exit "${rc:-2}"; }
need() { command -v "$1" >/dev/null 2>&1 || die "$1 is required"; }
need git; need python3; need sha256sum

prereg_dir() { printf '%s/.agents/preregistrations/%s' "$root" "$1"; }

# manifest_of <dir>: sorted "sha256  relative-path" lines, MANIFEST.sha256 excluded.
manifest_of() {
  (cd "$1" && LC_ALL=C find . -type f ! -name MANIFEST.sha256 | LC_ALL=C sort | sed 's|^\./||' |
    while IFS= read -r f; do sha256sum -- "$f"; done)
}

usage() { sed -n '2,25p' "$0" | sed 's/^# \{0,1\}//' >&2; exit 2; }
[ $# -ge 1 ] || usage
sub=$1; shift

case "$sub" in
manifest)
  [ $# -eq 1 ] || usage
  id=$1; printf '%s' "$id" | grep -Eq "$cmd_re" || die "bad id: $id"
  dir=$(prereg_dir "$id"); [ -d "$dir" ] || die "no such directory: $dir"
  manifest_of "$dir" >"$dir/MANIFEST.sha256"
  sha256sum "$dir/MANIFEST.sha256" | cut -d' ' -f1
  ;;
publish)
  push=0; args=""
  for a in "$@"; do if [ "$a" = "--push" ]; then push=1; else args="$args $a"; fi; done
  # shellcheck disable=SC2086
  set -- $args
  [ $# -eq 3 ] || usage
  kind=$1; id=$2; digest=$3
  case "$kind" in model|freeze|prereg) ;; *) die "bad kind: $kind";; esac
  printf '%s' "$id" | grep -Eq "$cmd_re" || die "bad id (ref-safe token required): $id"
  printf '%s' "$digest" | grep -Eq '^[0-9a-f]{64}$' || die "digest must be 64 lowercase hex"
  tag="fde-anchor/$kind/$id"
  git check-ref-format "refs/tags/$tag" || die "not a valid ref name: $tag"
  if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then rc=3 die "tag exists, refusing to overwrite or move: $tag"; fi
  if [ "$push" -eq 1 ]; then
    if [ -n "$(git ls-remote --tags origin "refs/tags/$tag" 2>/dev/null)" ]; then rc=3 die "tag exists on origin: $tag"; fi
    [ -n "$(git branch -r --contains HEAD 2>/dev/null)" ] || rc=3 die "HEAD is not on any remote branch; push the commit first"
  fi
  if [ "$kind" = prereg ]; then
    dir=$(prereg_dir "$id"); [ -f "$dir/MANIFEST.sha256" ] || rc=3 die "no manifest at $dir"
    actual=$(sha256sum "$dir/MANIFEST.sha256" | cut -d' ' -f1)
    [ "$actual" = "$digest" ] || rc=3 die "digest argument does not match the manifest digest ($actual)"
  fi
  name=$(git config user.name || true); email=$(git config user.email || true)
  [ -n "$name" ] && [ -n "$email" ] || die "git user.name and user.email must be set"
  msg=$(mktemp "${TMPDIR:-/tmp}/fde-anchor.XXXXXX"); trap 'rm -f "$msg"' EXIT INT TERM
  TAG=$tag KIND=$kind ID=$id DIGEST=$digest WHO="$name <$email>" python3 - >"$msg" <<'PY'
import json, os
print(json.dumps({"tag": os.environ["TAG"], "kind": os.environ["KIND"], "id": os.environ["ID"],
    "subject_ref": os.environ["ID"], "digest": os.environ["DIGEST"],
    "pusher": {"identity": os.environ["WHO"], "credential_scope": "shared_os_user"}},
    indent=2, sort_keys=True))
PY
  git tag -a "$tag" -F "$msg" HEAD
  echo "created $tag on $(git rev-parse --short HEAD) (digest $digest)"
  if [ "$push" -eq 1 ]; then
    git push origin "refs/tags/$tag:refs/tags/$tag"
    echo "pushed $tag"
  fi
  ;;
verify)
  remote=0; args=""
  for a in "$@"; do if [ "$a" = "--remote" ]; then remote=1; else args="$args $a"; fi; done
  # shellcheck disable=SC2086
  set -- $args
  [ $# -eq 1 ] || usage
  ref=$1
  case "$ref" in */*) kind=${ref%%/*}; id=${ref#*/};; *) kind=prereg; id=$ref;; esac
  case "$kind" in model|freeze|prereg) ;; *) die "bad kind: $kind";; esac
  printf '%s' "$id" | grep -Eq "$cmd_re" || die "bad id: $id"
  tag="fde-anchor/$kind/$id"
  fail=0
  bad() { printf 'FAIL: %s\n' "$*"; fail=1; }
  [ "$kind" = prereg ] || die "verify of kind $kind needs the case keeper or model registry (WP-14); not implemented in WP-09"
  git rev-parse -q --verify "refs/tags/$tag" >/dev/null || { printf 'FAIL: tag %s not found locally\n' "$tag"; exit 1; }
  tagobj=$(git rev-parse "refs/tags/$tag")
  [ "$(git cat-file -t "$tagobj")" = tag ] || { echo "FAIL: $tag is not an annotated tag"; exit 1; }
  commit=$(git rev-parse "$tag^{commit}")
  msgjson=$(git tag -l --format='%(contents)' "$tag")
  read_field() { printf '%s' "$msgjson" | python3 -c 'import sys,json
d=json.load(sys.stdin)
v=d
for k in sys.argv[1].split("."): v=v[k]
print(v)' "$1"; }
  tdigest=$(read_field digest 2>/dev/null) || { echo "FAIL: tag message is not the anchor JSON document"; exit 1; }
  [ "$(read_field tag)" = "$tag" ] || bad "tag message field tag differs from $tag"
  [ "$(read_field kind)" = "$kind" ] || bad "tag message field kind differs from $kind"
  [ "$(read_field id)" = "$id" ] || bad "tag message field id differs from $id"
  # The message must satisfy the anchor contract (all but verifier_receipt, issued below).
  if python3 -c 'import jsonschema' 2>/dev/null; then
    printf '%s' "$msgjson" | python3 -c '
import sys, json, jsonschema
s = json.load(open(sys.argv[1])); s["required"].remove("verifier_receipt")
jsonschema.validate(json.load(sys.stdin), s)' "$root/contracts/fde/anchor.v1.json" 2>/dev/null || bad "tag message does not conform to contracts/fde/anchor.v1.json"
  else
    echo "note: python jsonschema missing; contract conformance of the tag message not checked"
  fi
  dir=$(prereg_dir "$id")
  [ -d "$dir" ] || { bad "no pre-registration directory $dir"; exit 1; }
  # Which file changed: compare to the manifest committed at the tagged commit.
  committed=$(mktemp "${TMPDIR:-/tmp}/fde-anchor.XXXXXX"); trap 'rm -f "$committed"' EXIT INT TERM
  mpath=".agents/preregistrations/$id/MANIFEST.sha256"
  if git show "$commit:$mpath" >"$committed" 2>/dev/null; then
    [ "$(sha256sum "$committed" | cut -d' ' -f1)" = "$tdigest" ] || bad "manifest at tagged commit does not hash to the tag digest"
    ref_manifest=$committed
  else
    echo "note: no manifest at tagged commit; comparing with the working manifest"
    ref_manifest=$dir/MANIFEST.sha256
  fi
  actual=$(mktemp "${TMPDIR:-/tmp}/fde-anchor.XXXXXX"); trap 'rm -f "$committed" "$actual"' EXIT INT TERM
  manifest_of "$dir" >"$actual"
  # Names in either list but not identical in both.
  python3 - "$ref_manifest" "$actual" <<'PY' || fail=1
import sys
def load(p):
    d = {}
    for l in open(p, encoding="utf-8").read().splitlines():
        if l:
            h, n = l.split("  ", 1); d[n] = h
    return d
a, b = load(sys.argv[1]), load(sys.argv[2])
bad = 0
for n in sorted(set(a) | set(b)):
    if n not in b: print("FAIL: file removed: " + n); bad = 1
    elif n not in a: print("FAIL: file added: " + n); bad = 1
    elif a[n] != b[n]: print("FAIL: file changed: " + n); bad = 1
sys.exit(bad)
PY
  wdigest=$(sha256sum "$dir/MANIFEST.sha256" 2>/dev/null | cut -d' ' -f1)
  [ "$wdigest" = "$tdigest" ] || bad "working MANIFEST.sha256 digest $wdigest differs from tag digest $tdigest"
  if [ "$remote" -eq 1 ]; then
    rt=$(git ls-remote --tags origin "refs/tags/$tag" 2>/dev/null | awk '{print $1}' | head -n1) || rt=""
    if [ -z "$rt" ]; then bad "tag $tag not found on origin"
    elif [ "$rt" != "$tagobj" ]; then bad "origin tag object $rt differs from local $tagobj"; fi
  fi
  if [ "$fail" -ne 0 ]; then echo "anchor $tag: VERIFY FAILED"; exit 1; fi
  who=$(git config user.name || echo unknown)
  python3 - "$tag" "$tdigest" "$tagobj" "$who" "$remote" <<'PY'
import sys, json, datetime
tag, digest, obj, who, remote = sys.argv[1:6]
now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
print("anchor %s: OK (digest %s)" % (tag, digest))
print(json.dumps({"verifier_receipt": {"receipt_id": "verify-%s-%s" % (digest[:12], now), "verifier": who,
    "issued_at": now, "server_push_time": None, "tag_object_id": obj, "digest": digest},
    "remote_checked": remote == "1"}, sort_keys=True))
print("limit (RD-26): shared_os_user; pusher and verifier are one principal. Proves not changed since publication, not independence. server_push_time not read; anchoring rows PARTIAL (OD-6).")
PY
  ;;
*) usage ;;
esac

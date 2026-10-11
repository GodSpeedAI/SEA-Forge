#!/usr/bin/env bash
# Restore a cell backup made by scripts/casework-cell-backup.sh (plan casework-live-wiring T11).
#
#   scripts/casework-cell-restore.sh <archive.tar.gz> <new-cell-root>
#
# Safety: restore NEVER deletes anything. The target must not exist or must be an empty
# directory; to replace a damaged cell, move it aside first. The archive digest and the per-file
# manifest are verified (before extraction and again against the extracted tree); any mismatch
# aborts and removes only what this run extracted. The lock is held while extracting so no server
# can start on a half-restored tree.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <archive.tar.gz> <new-cell-root>" >&2
  exit 2
fi
archive="$1"
target="$2"
[ -f "$archive" ] || { echo "casework-restore: $archive not found" >&2; exit 1; }
[ -f "$archive.sha256" ] || { echo "casework-restore: $archive.sha256 missing; refusing an unverifiable archive" >&2; exit 1; }
[ -f "$archive.manifest" ] || { echo "casework-restore: $archive.manifest missing; refusing an unverifiable archive" >&2; exit 1; }
if [ -e "$target" ] && { [ ! -d "$target" ] || [ -n "$(ls -A "$target")" ]; }; then
  echo "casework-restore: $target exists and is not an empty directory; move it aside first (restore never deletes)" >&2
  exit 1
fi

(cd "$(dirname "$archive")" && sha256sum --check --quiet "$(basename "$archive").sha256") \
  || { echo "casework-restore: archive digest mismatch" >&2; exit 1; }

# The sha256/manifest files only detect corruption: whoever can forge the archive can forge them
# too. So before extracting anything, refuse any member that could write outside the target or
# plant a link: only regular files and directories, no absolute names, no ".." components
# (T11 security review F9: a symlink member followed by a file beneath it escapes --directory).
bad_types="$(tar --list --verbose --gzip --file "$archive" | cut -c1 | grep -v '^[-d]$' || true)"
if [ -n "$bad_types" ]; then
  echo "casework-restore: archive holds non-regular members (links/devices/fifos); refusing" >&2
  exit 1
fi
if tar --list --gzip --file "$archive" | grep -Eq '(^/|(^|/)\.\.(/|$))'; then
  echo "casework-restore: archive holds an absolute or parent-relative member name; refusing" >&2
  exit 1
fi

created=0
[ -d "$target" ] || { mkdir -p "$target"; created=1; }
cleanup() {
  if [ "${ok:-0}" -ne 1 ]; then
    echo "casework-restore: FAILED; removing what this run extracted into $target" >&2
    find "$target" -mindepth 1 -delete 2>/dev/null || true
    [ "$created" -eq 1 ] && rmdir "$target" 2>/dev/null || true
  fi
}
ok=0
trap cleanup EXIT

target="$(cd "$target" && pwd)"
check="$(mktemp)"
touch "$target/.server.lock"
exec 9<"$target/.server.lock"
flock -n 9 || { echo "casework-restore: a server holds $target/.server.lock" >&2; exit 1; }

tar --extract --gzip --file "$archive" --directory "$target" --numeric-owner --preserve-permissions
(cd "$target" && find . -type f \
  ! -name server.sock.lock ! -name .server.lock ! -name server.sock.binding \
  -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum) >"$check"
if ! diff -q "$archive.manifest" "$check" >/dev/null; then
  echo "casework-restore: restored tree differs from the backup manifest:" >&2
  diff "$archive.manifest" "$check" | head -20 >&2 || true
  rm -f "$check"
  exit 1
fi
rm -f "$check"
ok=1
trap - EXIT
echo "casework-restore: $(wc -l <"$archive.manifest") files restored to $target and verified against the manifest"
echo "casework-restore: start the kernel on it (SEA_FORGE_ROOT=$target) - check server.yaml gateway.uid matches the service user"

#!/usr/bin/env bash
# Consistent offline backup of a SEA Forge casework cell (plan casework-live-wiring T11).
#
#   scripts/casework-cell-backup.sh <cell-root> <archive.tar.gz>
#
# Consistency: the kernel holds an exclusive flock on <cell>/.server.lock for its whole lifetime.
# This script REFUSES to run while that lock is held (server running -> stop it first), then takes
# the SAME lock itself for the duration of the copy, so a server cannot start and begin appending
# to the ledgers mid-snapshot (it would fail with "another sea-forge server already owns cell").
#
# Writes <archive>, <archive>.sha256 (archive digest) and <archive>.manifest (sha256 of every
# file in the cell, relative paths) - restore verifies all three. Sockets and lock files are
# excluded: they are runtime artifacts recreated on start. The archive contains signing/continuation
# keys and approvals: it is created 0600, and must be stored encrypted/off-host by the operator.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <cell-root> <archive.tar.gz>" >&2
  exit 2
fi
cell="$(cd "$1" && pwd)"
archive="$2"
if [ -e "$archive" ]; then
  echo "casework-cell-backup: $archive already exists; refusing to overwrite" >&2
  exit 1
fi
if [ ! -f "$cell/server.yaml" ]; then
  echo "casework-cell-backup: $cell has no server.yaml; not a cell root" >&2
  exit 1
fi
command -v flock >/dev/null || { echo "casework-cell-backup: flock is required" >&2; exit 1; }

lock="$cell/.server.lock"
touch "$lock"
exec 9<"$lock"
if ! flock -n 9; then
  echo "casework-cell-backup: the kernel holds $lock - the cell is live. Stop sea-forge-server first (just casework-server-down, or systemctl stop sea-forge-server)." >&2
  exit 1
fi
# Lock held from here until this process exits: no server can start against the cell.

umask 077
tmp_archive="$archive.partial"
trap 'rm -f "$tmp_archive" "$archive.manifest.partial"' EXIT
tar --create --gzip --file "$tmp_archive" --directory "$cell" \
  --numeric-owner \
  --exclude='./server.sock' --exclude='./server.sock.binding' --exclude='./server.sock.lock' \
  --exclude='./.server.lock' \
  .
(cd "$cell" && find . -type f \
  ! -name server.sock.lock ! -name .server.lock ! -name server.sock.binding \
  -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum) >"$archive.manifest.partial"
mv "$tmp_archive" "$archive"
mv "$archive.manifest.partial" "$archive.manifest"
(cd "$(dirname "$archive")" && sha256sum "$(basename "$archive")") >"$archive.sha256"
trap - EXIT
echo "casework-cell-backup: $(wc -l <"$archive.manifest") files from $cell -> $archive"
echo "casework-cell-backup: archive sha256 $(cut -d' ' -f1 "$archive.sha256")"

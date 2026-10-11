#!/usr/bin/env bash
#
# Delete a demonstration cell created by scripts/seed-cell.sh.
#
#   scripts/reset-cell.sh [cell-root] [--yes]
#
# This removes a directory tree, so it refuses anything that is not provably a
# cell this project seeded. The guards below are deliberately paranoid: the
# argument is a path that could just as easily be $HOME.
set -euo pipefail

CELL="${1:-${SEA_FORGE_DEMO_ROOT:-$HOME/.sea-forge-demo}}"
SOCK="${SEA_FORGE_DEMO_SOCKET:-/tmp/sea-forge-demo.sock}"
STAMP="$CELL/.seeded-demo-cell"
ASSUME_YES=0
for arg in "$@"; do
    [ "$arg" = "--yes" ] && ASSUME_YES=1
done

say() { printf '[reset] %s\n' "$*"; }
refuse() {
    echo "refusing: $*" >&2
    exit 1
}

# --- guards -----------------------------------------------------------------

[ -n "$CELL" ] || refuse "the cell root is empty"

# Resolve before comparing, so `~/.sea-forge-demo/../..` cannot slip through.
if [ -e "$CELL" ]; then
    CELL="$(cd "$CELL" && pwd -P)"
fi

case "$CELL" in
    / | /home | /root | /usr | /etc | /var | /tmp) refuse "$CELL is a system directory" ;;
esac
[ "$CELL" != "$HOME" ] || refuse "\$HOME is not a cell"
[ "$CELL" != "${PWD}" ] || refuse "the current directory is not a cell"

if [ ! -e "$CELL" ]; then
    # Idempotent: nothing to remove is a success, not an error.
    say "nothing at $CELL"
    exit 0
fi

# The stamp is what makes this safe. Only seed-cell.sh writes it, so its
# presence is the difference between "a directory the user named" and "a
# directory this project created and may remove".
[ -e "$STAMP" ] || refuse "$CELL has no $(basename "$STAMP") marker — it was not seeded by this project"

say "will remove: $CELL"
say "  $(find "$CELL" -type f | wc -l) files, $(du -sh "$CELL" | cut -f1)"

if [ "$ASSUME_YES" -ne 1 ]; then
    read -r -p "[reset] remove it? [y/N] " reply
    case "$reply" in
        y | Y | yes | YES) ;;
        *)
            say "left alone"
            exit 0
            ;;
    esac
fi

# --- stop anything still serving this cell ----------------------------------

# A server holding the socket open would keep writing into a directory we are
# deleting. Match on the resolved root so we never signal a server serving some
# other cell.
if pgrep -f "sea-forge-server" >/dev/null 2>&1; then
    while read -r pid; do
        [ -n "$pid" ] || continue
        if tr '\0' '\n' <"/proc/$pid/environ" 2>/dev/null | grep -qx "SEA_FORGE_ROOT=$CELL"; then
            say "stopping server $pid serving this cell"
            kill "$pid" 2>/dev/null || true
        fi
    done < <(pgrep -f "sea-forge-server" || true)
    sleep 0.3
fi

rm -rf "$CELL"
[ ! -S "$SOCK" ] || rm -f "$SOCK"

say "removed $CELL"

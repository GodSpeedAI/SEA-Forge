#!/usr/bin/env sh
# AT-18 (S1 form): a fresh verifier pass over the kept raw records of the episode cell (scripts/fde-s1-verify.sh).
. "$(dirname "$0")/_lib.sh"
need_cell
exec "$FDE_ROOT/scripts/fde-s1-verify.sh" "$CELL"

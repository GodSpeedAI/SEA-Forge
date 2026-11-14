#!/usr/bin/env sh
# AT-14: Horizon read adapters, Playwright parity gate in the Horizon worktree (scripts/fde-parity.sh AT-14).
. "$(dirname "$0")/_lib.sh"
need_sibling "$HZ_DIR" "$HZ_EXPECT" Horizon
[ -x "$HZ_DIR/scripts/fde-parity.sh" ] || not_run "Horizon scripts/fde-parity.sh missing"
command -v bun >/dev/null 2>&1 || not_run "bun missing"
( cd "$HZ_DIR" && ./scripts/fde-parity.sh AT-14 ) || exit $?
st=$(cell_status AT-14)
[ "$st" = ok ] && exit 0
partial "parity passes on the producer's fixtures (real RealityTrace CLI output and real case_records::append output); the cell's own raw records were not rendered"

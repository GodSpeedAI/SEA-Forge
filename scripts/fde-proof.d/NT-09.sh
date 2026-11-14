#!/usr/bin/env sh
# NT-09: Horizon read adapters, Playwright parity gate in the Horizon worktree (scripts/fde-parity.sh NT-09).
. "$(dirname "$0")/_lib.sh"
need_sibling "$HZ_DIR" "$HZ_EXPECT" Horizon
[ -x "$HZ_DIR/scripts/fde-parity.sh" ] || not_run "Horizon scripts/fde-parity.sh missing"
command -v bun >/dev/null 2>&1 || not_run "bun missing"
( cd "$HZ_DIR" && ./scripts/fde-parity.sh NT-09 ) || exit $?

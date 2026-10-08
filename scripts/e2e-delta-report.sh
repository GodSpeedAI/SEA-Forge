#!/usr/bin/env sh
# Emit the Delta-0 summary for the GodSpeed canonical runtime convergence plan
# deterministically from .agents/status/e2e-current-status.yml so that
# `scripts/e2e-delta-check.sh` can prove the committed report has not drifted.
#
# Overrides for isolated testing:
#   STATUS_FILE   (default <repo>/.agents/status/e2e-current-status.yml)
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
status=${STATUS_FILE:-"$root/.agents/status/e2e-current-status.yml"}

[ -f "$status" ] || {
  printf 'fail: status file not found: %s\n' "$status" >&2
  exit 1
}

sha=$(awk '/^preregistration_sha256:/ {print $2; exit}' "$status")
updated=$(awk '/^updated:/ {print $2; exit}' "$status")

printf '# Delta-0 — godspeed-canonical-runtime-convergence\n'
printf '\n'
printf 'generated_from: .agents/status/e2e-current-status.yml\n'
printf 'status_updated: %s\n' "$updated"
printf 'preregistration_sha256: %s\n' "$sha"
printf '\n'

awk '
  /^requirements:/ { inr = 1; next }
  inr && /^[^ ]/ { inr = 0 }
  {
    if (inr && /^  - id:/) {
      if (nid != "") count[verdict]++
      nid = $3; verdict = ""; title = ""; kind = ""
    } else if (inr && nid != "") {
      if (/^    verdict:/) { sub(/^    verdict:[[:space:]]*/, ""); verdict = $0 }
      if (/^    kind:/)    { sub(/^    kind:[[:space:]]*/, ""); kind = $0 }
      if (/^    title:/)   { sub(/^    title:[[:space:]]*/, ""); title = $0 }
    }
  }
  END {
    if (nid != "") count[verdict]++
    printf "verdict_counts:\n"
    printf "  confirmed: %d\n", count["CONFIRMED"] + 0
    printf "  partial: %d\n", count["PARTIAL"] + 0
    printf "  absent: %d\n", count["ABSENT"] + 0
    printf "  contradicted: %d\n", count["CONTRADICTED"] + 0
    printf "  unknown: %d\n", count["UNKNOWN"] + 0
  }
' "$status"

printf '\n'
printf 'open_delta_rule: every requirement not CONFIRMED is open delta.\n'
printf '\n'
printf '| ID | kind | title | verdict |\n'
printf '| --- | --- | --- | --- |\n'

awk '
  /^requirements:/ { inr = 1; next }
  inr && /^[^ #]/ && !/^  / && !/^$/ { inr = 0 }
  {
    if (inr && /^  - id:/) {
      if (nid != "") print_row()
      nid = $3; verdict = ""; title = ""; kind = ""
    } else if (inr && nid != "") {
      if (/^    verdict:/) { sub(/^    verdict:[[:space:]]*/, ""); verdict = $0 }
      if (/^    kind:/)    { sub(/^    kind:[[:space:]]*/, ""); kind = $0 }
      if (/^    title:/)   { sub(/^    title:[[:space:]]*/, ""); title = $0 }
    }
  }
  function print_row() {
    printf "| %s | %s | %s | %s |\n", nid, kind, title, verdict
  }
  END { if (nid != "") print_row() }
' "$status"

#!/usr/bin/env bash
# AT-18 (S1 form): fresh verifier pass over the kept raw records of a cell. See scripts/fde-s1/verify.py.
#   scripts/fde-s1-verify.sh <cell-dir>
set -euo pipefail
exec python3 "$(cd "$(dirname "$0")" && pwd)/fde-s1/verify.py" "${1:?cell directory}"

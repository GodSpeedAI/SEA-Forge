#!/usr/bin/env bash
# live-proof.sh - live OpenShell MicroVM proof for the RepairAgent Cell.
#
# What it proves: that a Docker-free OpenShell gateway (compute_driver = "vm",
# libkrun MicroVMs, no container engine) can run the sea-forge-case-runner live
# tests against the RepairAgent projection, including a non-bootstrap image
# (SECOND_IMAGE). The live tests are the ones in
# crates/sea-forge-case-runner/tests/cell_episode_live.rs.
#
# This script is the only place in the repository that starts a real OpenShell
# gateway and sandboxes for the live proof. Nothing else should.
#
# It never reports success unless the live tests ran and passed.
#
# Exit codes:
#   0  live tests ran and passed
#   1  a check or a live test failed
#   3  BLOCKED: a prerequisite is missing (reason printed to stderr as
#      "BLOCKED: <reason>")
#
# Environment variables (all optional):
#   PROJECTION_DIR   projection directory, relative to the repo root
#                    (default: fixtures/cells/repair-agent/openshell)
#   GATEWAY_NAME     name the gateway is registered under (default: godspeed-live)
#   PORT             gateway port on 127.0.0.1 (default: 18081)
#   STATE_DIR        gateway state directory (default: a new mktemp -d)
#   IMAGE            bootstrap/default sandbox image, always set explicitly
#                    (default: OpenShell 0.1.3's own default image)
#   SECOND_IMAGE     non-bootstrap image used by the live tests
#                    (default: docker.io/library/python:3.13-slim)
#   ALLOW_CONTAINER_SOCKET=1
#                    proceed even if a Docker or Podman socket exists. The run
#                    then does NOT establish the Docker-free claim, and it says so.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

PROJECTION_DIR="${PROJECTION_DIR:-fixtures/cells/repair-agent/openshell}"
GATEWAY_NAME="${GATEWAY_NAME:-godspeed-live}"
PORT="${PORT:-18081}"
STATE_DIR="${STATE_DIR:-$(mktemp -d)}"
IMAGE="${IMAGE:-nvcr.io/nvidia/base/ubuntu:24.04}"
SECOND_IMAGE="${SECOND_IMAGE:-docker.io/library/python:3.13-slim}"
EXPECTED_VERSION="0.1.3"
TEST_FILE="$REPO_ROOT/crates/sea-forge-case-runner/tests/cell_episode_live.rs"

GATEWAY_PID=""
GATEWAY_REGISTERED=0

blocked() {
    echo "BLOCKED: $*" >&2
    exit 3
}

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

cleanup() {
    local rc=$?
    if [[ -n "$GATEWAY_PID" ]]; then
        kill "$GATEWAY_PID" 2>/dev/null || true
        wait "$GATEWAY_PID" 2>/dev/null || true
    fi
    # Remove only a registration this run added; an earlier exit must not
    # delete an operator's existing gateway of the same name.
    if [[ "$GATEWAY_REGISTERED" == 1 ]]; then
        openshell gateway remove "$GATEWAY_NAME" >/dev/null 2>&1 || true
    fi
    exit "$rc"
}
trap cleanup EXIT

# Print the first existing executable among the arguments, searching PATH and
# then the sbin directories, which are often absent from a user's PATH.
find_tool() {
    local tool="$1"
    if command -v "$tool" >/dev/null 2>&1; then
        command -v "$tool"
        return 0
    fi
    local dir
    for dir in /usr/sbin /sbin; do
        if [[ -x "$dir/$tool" ]]; then
            echo "$dir/$tool"
            return 0
        fi
    done
    return 1
}

# ---------------------------------------------------------------------------
# Preflight: binaries, versions, virtualization, host tools.
# ---------------------------------------------------------------------------

for bin in openshell openshell-prover openshell-gateway; do
    if ! command -v "$bin" >/dev/null 2>&1; then
        blocked "$bin is not on PATH (expected OpenShell $EXPECTED_VERSION)"
    fi
    version="$("$bin" --version 2>&1 | head -n 1 || true)"
    if [[ "$version" != "$bin $EXPECTED_VERSION" ]]; then
        blocked "$bin reports '$version', expected '$bin $EXPECTED_VERSION'"
    fi
done

DRIVER_DIR=""
GATEWAY_BIN="$(command -v openshell-gateway)"
GATEWAY_BIN_DIR="$(dirname -- "$GATEWAY_BIN")"
for dir in \
    "$HOME/.local/libexec/openshell" \
    "/usr/libexec/openshell" \
    "/usr/local/libexec/openshell" \
    "/usr/local/libexec" \
    "$GATEWAY_BIN_DIR/../libexec" \
    "$GATEWAY_BIN_DIR"; do
    if [[ -x "$dir/openshell-driver-vm" ]]; then
        DRIVER_DIR="$dir"
        break
    fi
done
if [[ -z "$DRIVER_DIR" ]]; then
    blocked "openshell-driver-vm not found in ~/.local/libexec/openshell, /usr/libexec/openshell, /usr/local/libexec/openshell, /usr/local/libexec, the gateway prefix libexec, or next to the gateway binary"
fi
driver_version="$("$DRIVER_DIR/openshell-driver-vm" --version 2>&1 | head -n 1 || true)"
if [[ "$driver_version" != "openshell-driver-vm $EXPECTED_VERSION" ]]; then
    blocked "$DRIVER_DIR/openshell-driver-vm reports '$driver_version', expected 'openshell-driver-vm $EXPECTED_VERSION'"
fi

if [[ "$(uname -s)" == "Linux" ]]; then
    if [[ ! -r /dev/kvm ]]; then
        hint=""
        if grep -qi microsoft /proc/version 2>/dev/null; then
            hint=" This is WSL2: set nestedVirtualization=true under [wsl2] in .wslconfig in your Windows profile, run 'wsl --shutdown', then restart WSL."
        fi
        blocked "/dev/kvm is not readable by $(id -un).$hint"
    fi
fi

for tool in ssh git python3 mke2fs debugfs e2fsck; do
    if ! find_tool "$tool" >/dev/null; then
        blocked "$tool not found on PATH, /usr/sbin or /sbin"
    fi
done
# The gateway runs its host-tool preflight with this PATH.
export PATH="$PATH:/usr/sbin:/sbin"

if [[ ! -f "$TEST_FILE" ]]; then
    blocked "live test target not present: ${TEST_FILE#"$REPO_ROOT"/}"
fi

PROJECTION_ABS="$REPO_ROOT/$PROJECTION_DIR"
if [[ ! -f "$PROJECTION_ABS/sandbox.json" ]]; then
    fail "projection has no sandbox.json: $PROJECTION_ABS"
fi

# ---------------------------------------------------------------------------
# Docker-free assertion.
# ---------------------------------------------------------------------------

present=()
if [[ -n "${DOCKER_HOST:-}" ]]; then
    present+=("DOCKER_HOST is set")
fi
for sock in \
    /var/run/docker.sock \
    "$HOME/.docker/run/docker.sock" \
    "${XDG_RUNTIME_DIR:-/nonexistent}/docker.sock" \
    "${XDG_RUNTIME_DIR:-/nonexistent}/podman/podman.sock"; do
    if [[ -e "$sock" ]]; then
        present+=("$sock exists")
    fi
done
if (( ${#present[@]} > 0 )); then
    if [[ "${ALLOW_CONTAINER_SOCKET:-0}" == "1" ]]; then
        echo "WARNING: container engine present (${present[*]}); ALLOW_CONTAINER_SOCKET=1 is set. The Docker-free claim is NOT established by this run." >&2
    else
        fail "container engine present (${present[*]}). Unset DOCKER_HOST and stop the engine, or set ALLOW_CONTAINER_SOCKET=1 to run anyway (the Docker-free claim is then not established)."
    fi
else
    echo "OK: no Docker or Podman socket present"
fi

# ---------------------------------------------------------------------------
# Gateway configuration.
# ---------------------------------------------------------------------------

mkdir -p "$STATE_DIR"
CONFIG="$STATE_DIR/gateway.toml"
python3 -I - "$PROJECTION_ABS/sandbox.json" "$CONFIG" "$PORT" "$DRIVER_DIR" "$STATE_DIR/vm-driver" "$IMAGE" <<'PY' \
    || fail "could not generate $CONFIG from sandbox.json"
import json
import sys

sandbox_path, out_path, port, driver_dir, state_dir, image = sys.argv[1:7]
with open(sandbox_path, encoding="utf-8") as handle:
    shape = json.load(handle)["vm_shape"]

lines = [
    "[openshell]",
    "version = 2",
    "",
    "[openshell.gateway]",
    'compute_driver = "vm"',
    "disable_tls = true",
    "",
    "[openshell.drivers.vm]",
    f'grpc_endpoint = "http://127.0.0.1:{port}"',
    f'driver_dir = "{driver_dir}"',
    f'state_dir = "{state_dir}"',
]
if image:
    lines.append(f'default_image = "{image}"')
lines += [
    f"vcpus = {int(shape['vcpus'])}",
    f"mem_mib = {int(shape['mem_mib'])}",
    f"overlay_disk_mib = {int(shape['overlay_disk_mib'])}",
]
with open(out_path, "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines) + "\n")
PY

if ! openshell-gateway config preflight --path "$CONFIG" >"$STATE_DIR/preflight.log" 2>&1; then
    cat "$STATE_DIR/preflight.log" >&2
    fail "openshell-gateway config preflight failed for $CONFIG"
fi
echo "OK: gateway config preflight passed ($CONFIG)"

# ---------------------------------------------------------------------------
# Start the gateway and register it.
# ---------------------------------------------------------------------------

openshell-gateway --config "$CONFIG" --port "$PORT" \
    --db-url "sqlite:$STATE_DIR/openshell.db?mode=rwc" \
    >"$STATE_DIR/gateway.log" 2>&1 &
GATEWAY_PID=$!

ready=0
for _ in $(seq 1 60); do
    if ! kill -0 "$GATEWAY_PID" 2>/dev/null; then
        break
    fi
    if openshell --gateway-endpoint "http://127.0.0.1:$PORT" status >/dev/null 2>&1; then
        ready=1
        break
    fi
    sleep 1
done
if (( ready == 0 )); then
    echo "--- last 40 lines of $STATE_DIR/gateway.log ---" >&2
    tail -n 40 "$STATE_DIR/gateway.log" >&2 || true
    fail "gateway did not answer on http://127.0.0.1:$PORT within 60 s"
fi
echo "OK: gateway answered on http://127.0.0.1:$PORT"

openshell gateway add "http://127.0.0.1:$PORT" --local --name "$GATEWAY_NAME" \
    || fail "could not register gateway $GATEWAY_NAME"
GATEWAY_REGISTERED=1

# ---------------------------------------------------------------------------
# Live tests. Only cargo's exit status decides success.
# ---------------------------------------------------------------------------

cd "$REPO_ROOT"
set +e
SEA_FORGE_OPENSHELL_LIVE=1 \
SEA_FORGE_OPENSHELL_GATEWAY="$GATEWAY_NAME" \
SEA_FORGE_OPENSHELL_PROJECTION="$PROJECTION_ABS" \
SEA_FORGE_OPENSHELL_IMAGE="$IMAGE" \
SEA_FORGE_OPENSHELL_SECOND_IMAGE="$SECOND_IMAGE" \
cargo test -p sea-forge-case-runner --test cell_episode_live -j 2 -- \
    --include-ignored --test-threads 1 --nocapture
test_rc=$?
set -e

echo "--- gateway log (last 20 lines) ---"
tail -n 20 "$STATE_DIR/gateway.log" || true

if (( test_rc != 0 )); then
    fail "live tests exited with status $test_rc"
fi
echo "OK: live OpenShell proof passed"

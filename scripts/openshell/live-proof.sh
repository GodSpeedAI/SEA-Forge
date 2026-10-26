#!/usr/bin/env bash
# live-proof.sh - live OpenShell MicroVM proof for the RepairAgent Cell.
#
# What it proves: that a Docker-free OpenShell gateway (compute_driver = "vm",
# libkrun MicroVMs, no container engine) can run the sea-forge-case-runner live
# tests against the RepairAgent projection, including a non-bootstrap image
# (SECOND_IMAGE). The live tests are the ones in
# crates/sea-forge-case-runner/tests/cell_episode_live.rs. It also starts the
# host-side egress arrival listener (egress-control.py) that the network
# control tests observe: the allowed variant's arrival and the denied variant's
# non-arrival are host-observed evidence (EGRESS_ARRIVAL_LOG).
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
#   EGRESS_PORT      arrival listener port (default: 46244; the committed
#                    fixture's live-egress-control endpoint names this port)
#   EGRESS_HOST      guest-reachable host address (default: 10.0.2.2, the
#                    libkrun user-networking host alias the fixture names)
#   EGRESS_ARRIVAL_LOG  arrival log path (default: $STATE_DIR/egress-arrivals.log)

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
# The egress control. The arrival listener binds 0.0.0.0 on this port; the
# projection's `live-egress-control` endpoint names the same port and the
# libkrun user-networking address that reaches the host from the guest. The
# address default matches the committed fixture; overriding it means the guest
# target no longer matches the projected endpoint (see the fixture's model.sea).
EGRESS_PORT="${EGRESS_PORT:-46244}"
EGRESS_HOST="${EGRESS_HOST:-10.0.2.2}"
EGRESS_CONTROL_ADDR="${EGRESS_HOST}:${EGRESS_PORT}"
EGRESS_ARRIVAL_LOG="${EGRESS_ARRIVAL_LOG:-$STATE_DIR/egress-arrivals.log}"
EGRESS_HELPER="$SCRIPT_DIR/egress-control.py"

GATEWAY_PID=""
EGRESS_PID=""

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
    if [[ -n "$EGRESS_PID" ]]; then
        kill "$EGRESS_PID" 2>/dev/null || true
        wait "$EGRESS_PID" 2>/dev/null || true
    fi
    if [[ -n "$GATEWAY_PID" ]]; then
        kill "$GATEWAY_PID" 2>/dev/null || true
        wait "$GATEWAY_PID" 2>/dev/null || true
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

# The egress control tests must be part of the ladder this script runs
# (--include-ignored). A rename that drops them from the file fails here
# instead of silently shrinking the proof.
for egress_test in live_network_allow_control_connects live_network_denial_proven; do
    if ! grep -q "fn $egress_test" "$TEST_FILE"; then
        blocked "egress control test missing from the ladder: $egress_test"
    fi
done

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
# Gateway configuration. The gateway runs with mTLS, the shape upstream's e2e
# harness uses at OpenShell v0.1.3: an ephemeral CA, server and client
# certificates from `openshell-gateway generate-certs`, and Ed25519 sandbox
# launch-signing keys for the VM driver. Every credential is generated under
# STATE_DIR, and the CLI config lives under STATE_DIR too (XDG_CONFIG_HOME), so
# the operator's own OpenShell configuration is never read or written.
# ---------------------------------------------------------------------------

mkdir -p "$STATE_DIR"
export XDG_CONFIG_HOME="$STATE_DIR/config"
mkdir -p "$XDG_CONFIG_HOME"
PKI_DIR="$STATE_DIR/pki"
JWT_DIR="$STATE_DIR/jwt"
CONFIG="$STATE_DIR/gateway.toml"

# generate-certs writes to $XDG_CONFIG_HOME/openshell/gateways/openshell/mtls
# as well; its stderr goes to a file that is not uploaded.
if ! "$GATEWAY_BIN" generate-certs --output-dir "$PKI_DIR" --server-san host.openshell.internal \
    >/dev/null 2>"$STATE_DIR/certgen.err"; then
    fail "openshell-gateway generate-certs failed (see $STATE_DIR/certgen.err)"
fi

mkdir -p "$JWT_DIR"
openssl genpkey -algorithm Ed25519 -out "$JWT_DIR/signing.pem" >/dev/null 2>&1 \
    || fail "could not generate the sandbox launch signing key"
openssl pkey -in "$JWT_DIR/signing.pem" -pubout -out "$JWT_DIR/public.pem" >/dev/null 2>&1 \
    || fail "could not derive the sandbox launch public key"
openssl rand -hex 16 >"$JWT_DIR/kid" || fail "could not write the sandbox launch kid"
chmod 600 "$JWT_DIR/signing.pem"

python3 -I - "$PROJECTION_ABS/sandbox.json" "$CONFIG" "$PORT" "$DRIVER_DIR" "$STATE_DIR/vm-driver" "$IMAGE" \
    "$PKI_DIR" "$JWT_DIR" "$GATEWAY_NAME" <<'PY' \
    || fail "could not generate $CONFIG from sandbox.json"
import json
import sys

(sandbox_path, out_path, port, driver_dir, state_dir, image,
 pki_dir, jwt_dir, gateway_name) = sys.argv[1:10]
with open(sandbox_path, encoding="utf-8") as handle:
    shape = json.load(handle)["vm_shape"]


def s(value):
    # JSON string escaping is valid TOML basic-string escaping for these values.
    return json.dumps(value)


lines = [
    "[openshell]",
    "version = 2",
    "",
    "[openshell.gateway]",
    'compute_driver = "vm"',
    f"guest_tls_ca = {s(pki_dir + '/ca.crt')}",
    "",
    "[openshell.gateway.tls]",
    f"cert_path = {s(pki_dir + '/server/tls.crt')}",
    f"key_path = {s(pki_dir + '/server/tls.key')}",
    f"client_ca_path = {s(pki_dir + '/ca.crt')}",
    "",
    "[openshell.gateway.mtls_auth]",
    "enabled = true",
    "",
    "[openshell.gateway.gateway_jwt]",
    f"signing_key_path = {s(jwt_dir + '/signing.pem')}",
    f"public_key_path = {s(jwt_dir + '/public.pem')}",
    f"kid_path = {s(jwt_dir + '/kid')}",
    f"gateway_id = {s(gateway_name)}",
    "",
    "[openshell.drivers.vm]",
    f"grpc_endpoint = {s('https://host.openshell.internal:' + port)}",
    f"driver_dir = {s(driver_dir)}",
    f"state_dir = {s(state_dir)}",
]
if image:
    lines.append(f"default_image = {s(image)}")
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
# Start the gateway and register it for the CLI (mTLS client bundle).
# ---------------------------------------------------------------------------

openshell-gateway --config "$CONFIG" --port "$PORT" \
    --db-url "sqlite:$STATE_DIR/openshell.db?mode=rwc" \
    >"$STATE_DIR/gateway.log" 2>&1 &
GATEWAY_PID=$!

# Registration mirrors upstream's e2e_register_mtls_gateway: the CLI finds the
# gateway by name under XDG_CONFIG_HOME, with the CA and client certificate in
# mtls/. The sandbox adapter passes XDG_CONFIG_HOME through, so the live tests
# reach the gateway the same way.
GW_CFG="$XDG_CONFIG_HOME/openshell/gateways/$GATEWAY_NAME"
mkdir -p "$GW_CFG/mtls"
cp "$PKI_DIR/ca.crt" "$GW_CFG/mtls/ca.crt"
cp "$PKI_DIR/client/tls.crt" "$GW_CFG/mtls/tls.crt"
cp "$PKI_DIR/client/tls.key" "$GW_CFG/mtls/tls.key"
cat >"$GW_CFG/metadata.json" <<EOF
{
  "name": "$GATEWAY_NAME",
  "gateway_endpoint": "https://127.0.0.1:$PORT",
  "is_remote": false,
  "gateway_port": $PORT
}
EOF
printf '%s' "$GATEWAY_NAME" >"$XDG_CONFIG_HOME/openshell/active_gateway"

ready=0
for _ in $(seq 1 60); do
    if ! kill -0 "$GATEWAY_PID" 2>/dev/null; then
        break
    fi
    if openshell --gateway "$GATEWAY_NAME" status >/dev/null 2>&1; then
        ready=1
        break
    fi
    sleep 1
done
if (( ready == 0 )); then
    echo "--- last 40 lines of $STATE_DIR/gateway.log ---" >&2
    tail -n 40 "$STATE_DIR/gateway.log" >&2 || true
    fail "gateway did not answer on https://127.0.0.1:$PORT (mTLS) within 60 s"
fi
echo "OK: gateway answered on https://127.0.0.1:$PORT (mTLS, registered as $GATEWAY_NAME)"

# ---------------------------------------------------------------------------
# Host-side egress arrival listener. The guest's allowed variant connects to
# EGRESS_CONTROL_ADDR and the listener records one line per accepted
# connection, so arrivals and non-arrivals are host-observed evidence. See
# scripts/openshell/egress-control.py and the live tests it feeds.
# ---------------------------------------------------------------------------

if [[ ! -f "$EGRESS_HELPER" ]]; then
    fail "egress control helper missing: $EGRESS_HELPER"
fi
: >"$EGRESS_ARRIVAL_LOG"
EGRESS_READY="$STATE_DIR/egress-listener.ready"
EGRESS_PIDFILE="$STATE_DIR/egress-listener.pid"
rm -f "$EGRESS_READY" "$EGRESS_PIDFILE"
python3 -I "$EGRESS_HELPER" serve --port "$EGRESS_PORT" --log "$EGRESS_ARRIVAL_LOG" \
    --ready "$EGRESS_READY" --pidfile "$EGRESS_PIDFILE" &
EGRESS_PID=$!
ready=0
for _ in $(seq 1 20); do
    if ! kill -0 "$EGRESS_PID" 2>/dev/null; then
        break
    fi
    if [[ -s "$EGRESS_READY" ]]; then
        ready=1
        break
    fi
    sleep 0.5
done
if (( ready == 0 )); then
    fail "egress arrival listener did not start on 0.0.0.0:$EGRESS_PORT"
fi
export EGRESS_CONTROL_ADDR EGRESS_ARRIVAL_LOG
echo "OK: egress arrival listener on 0.0.0.0:$EGRESS_PORT (log $EGRESS_ARRIVAL_LOG); guest target $EGRESS_CONTROL_ADDR"

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

#!/bin/bash
# Evidence probe: what macOS App Sandbox enforces on an arbitrary child CLI tool.
# Ground truth is checked from the HOST side (files, listener logs, process
# table, host-measured sizes). Child output is only a self-report and is used
# to classify why an attempt failed. See README.md for the status rules.
set -u

HERE=$(cd "$(dirname "$0")" && pwd)
EVIDENCE_DIR=${EVIDENCE_DIR:-$PWD/evidence}
mkdir -p "$EVIDENCE_DIR"
RESULTS="$EVIDENCE_DIR/results.jsonl"
TABLE="$EVIDENCE_DIR/table.md"
: > "$RESULTS"
: > "$TABLE"

TMPROOT=${RUNNER_TEMP:-${TMPDIR:-/tmp}}
TMPROOT=${TMPROOT%/}
BUILD=$(mktemp -d "$TMPROOT/sfprobe-build.XXXXXX") || { echo "mktemp failed" >&2; exit 1; }
SECRET_FILE="$HOME/.sf-probe-secret"
HPID=""
UPID=""

cleanup() {
    [ -n "$HPID" ] && kill "$HPID" 2>/dev/null
    [ -n "$UPID" ] && kill "$UPID" 2>/dev/null
    rm -f "$SECRET_FILE"
    pkill -KILL -fx 'sleep 300' 2>/dev/null
}
trap cleanup EXIT

die() { echo "PROBE ERROR: $*" >&2; exit 1; }

# ---- build the launcher (one copy per run, signed per variant) -------------
cat > "$BUILD/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleIdentifier</key>
	<string>ai.godspeed.sea-forge.probe-launcher</string>
	<key>CFBundleName</key>
	<string>probe-launcher</string>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
</dict>
</plist>
PLIST
clang -Wall -Wextra -O1 -o "$BUILD/launcher-base" "$HERE/launcher.c" \
    -sectcreate __TEXT __info_plist "$BUILD/Info.plist" || die "clang build failed"

# Host-side UDP listener (logs each datagram on its own line).
cat > "$BUILD/udp_listener.py" <<'PY'
import socket, sys
port, log = int(sys.argv[1]), sys.argv[2]
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(("127.0.0.1", port))
with open(log, "a", buffering=1) as f:
    while True:
        data, _ = s.recvfrom(4096)
        f.write(data.decode("utf-8", "replace") + "\n")
PY

# ---- per-scenario helpers ---------------------------------------------------
# Globals set per run: variant, run, tag, WS, CANARY, CANARY_SRC, BASE_SHA,
# PORT, UPORT, HLOG, ULOG, SRV, SECRET_VAL, SECRET_BYTES, bin, OUT, RAW_EXIT, SCN.

reset_canary() {
    printf 'canary-original\n' > "$CANARY_SRC"
    BASE_SHA=$(shasum -a 256 "$CANARY_SRC" | cut -d' ' -f1)
}
canary_changed() {
    [ "$(shasum -a 256 "$CANARY_SRC" | cut -d' ' -f1)" != "$BASE_SHA" ] && echo yes || echo no
}
mk() { grep -q "$1" "$OUT" 2>/dev/null && echo yes || echo no; }
eperm() { grep -qiE 'not permitted|permission denied' "$OUT" 2>/dev/null && echo yes || echo no; }
res() { grep -m1 'RESULT_' "$OUT" 2>/dev/null || echo 'no RESULT line'; }

# run_child '<snippet with @PLACEHOLDERS@>'
# Runs the snippet via the launcher under env -i (PATH plus the planted secret only).
run_child() {
    local snip
    snip=$(printf '%s' "$1" | sed \
        -e "s|@WS@|$WS|g" -e "s|@CANARY_SRC@|$CANARY_SRC|g" -e "s|@CANARY@|$CANARY|g" \
        -e "s|@SECRET@|$SECRET_FILE|g" -e "s|@PORT@|$PORT|g" -e "s|@UPORT@|$UPORT|g")
    OUT="$BUILD/out-${tag}-${SCN}.txt"
    env -i PATH="$PATH" SF_PROBE_SECRET="$SECRET_VAL" "$bin" /bin/sh -c "$snip" \
        >"$OUT" 2>&1 </dev/null
    RAW_EXIT=$?
}

# finish <name> <expected denied|allowed> <effect yes|no> <marker yes|no> <classok yes|no> <host_check> <observed>
# denied: effect=yes is FAIL (escape). allowed: effect=yes is PASS, no effect is ERROR.
# A row without its attempt marker is ERROR, never PASS.
finish() {
    local name=$1 expect=$2 effect=$3 marker=$4 classok=$5 hostc=$6 obs=$7 status
    if [ "$marker" != yes ]; then
        status=ERROR; obs="probe did not reach attempt ($obs)"
    elif [ "$expect" = allowed ]; then
        if [ "$effect" = yes ]; then status=PASS
        else status=ERROR; obs="expected reachable, blocked ($obs)"; fi
    elif [ "$effect" = yes ]; then
        status=FAIL
    elif [ "$classok" != yes ]; then
        status=ERROR; obs="unclassified failure ($obs)"
    else
        status=PASS
    fi
    env SF_REDACT="$SECRET_VAL" python3 -I -c "$EMIT_PY" "$RESULTS" "$TABLE" \
        "$variant" "$run" "$SCN" "$name" "$expect" "$obs" "$hostc" "$RAW_EXIT" "$OUT" "$status"
}

EMIT_PY='
import json, os, sys
res, table, variant, run, n, name, expect, obs, hostc, raw, outf, status = sys.argv[1:13]
text = open(outf, errors="replace").read()
secret = os.environ.get("SF_REDACT", "")
if secret:
    text = text.replace(secret, "<redacted>")
row = {"variant": variant, "run": int(run), "scenario": int(n), "name": name,
       "expected": expect, "observed": obs, "host_check": hostc,
       "raw_exit": int(raw), "stderr_tail": text[-600:], "status": status}
with open(res, "a") as f:
    f.write(json.dumps(row) + "\n")
md = lambda s: s.replace("|", "/").replace("\n", " ")
with open(table, "a") as f:
    f.write("| %s | %s | %s | %s | %s | %s | %s | **%s** |\n" % (
        variant, run, n, name, expect, md(obs)[:200], md(hostc)[:120], status))
'

# ---- scenarios ----------------------------------------------------------------
sc_1() {
    SCN=1; reset_canary; rm -f "$WS/inside-1.txt"
    run_child 'echo ATTEMPT; echo sf-inside > @WS@/inside-1.txt && echo RESULT_OK || echo RESULT_ERR'
    local eff=no; grep -q 'sf-inside' "$WS/inside-1.txt" 2>/dev/null && eff=yes
    finish "write inside workspace" allowed "$eff" "$(mk ATTEMPT)" yes \
        "file_present=$eff" "child: $(res)"
}

sc_2() {
    SCN=2; reset_canary; rm -f "$CANARY/escape-2.txt"
    run_child 'echo ATTEMPT; echo escape > @CANARY@/escape-2.txt && echo RESULT_OK || echo RESULT_ERR'
    local eff=no; [ -e "$CANARY/escape-2.txt" ] && eff=yes
    finish "write to canary dir outside workspace" denied "$eff" "$(mk ATTEMPT)" "$(eperm)" \
        "escape_file_present=$eff" "child: $(res)"
}

sc_3() {
    SCN=3; reset_canary; rm -f "$WS/sl3"
    run_child 'echo ATTEMPT; ln -s @CANARY_SRC@ @WS@/sl3 && echo LINK_OK; echo tamper >> @WS@/sl3 && echo RESULT_OK || echo RESULT_ERR'
    local eff L=absent; eff=$(canary_changed); [ -L "$WS/sl3" ] && L=present
    finish "symlink in workspace -> canary, then write" denied "$eff" "$(mk ATTEMPT)" "$(eperm)" \
        "canary_changed=$eff link=$L" "child: $(res)"
}

sc_4() {
    SCN=4; reset_canary; rm -f "$WS/hl4"
    run_child 'echo ATTEMPT; ln @CANARY_SRC@ @WS@/hl4 && echo LINK_OK; echo tamper4 >> @WS@/hl4 && echo RESULT_OK || echo RESULT_ERR'
    local eff L=absent
    eff=$(canary_changed)
    if [ -e "$WS/hl4" ] && [ "$(stat -f %i "$WS/hl4")" = "$(stat -f %i "$CANARY_SRC")" ]; then L=present; fi
    finish "hard link of canary into workspace, then write" denied "$eff" "$(mk ATTEMPT)" "$(eperm)" \
        "canary_changed=$eff hardlink=$L" "child: $(res)"
}

sc_5() {
    SCN=5; reset_canary
    run_child 'echo ATTEMPT; if n=$(wc -c < @SECRET@ 2>/dev/null); then echo RESULT_OK bytes=$n; else echo RESULT_ERR; fi'
    local eff=no; grep -q 'RESULT_OK' "$OUT" && eff=yes
    finish "read secret file in HOME (output redacted)" denied "$eff" "$(mk ATTEMPT)" "$(eperm)" \
        "host_secret_bytes=$SECRET_BYTES" "child: $(res)"
}

sc_6() {
    SCN=6; reset_canary
    run_child 'echo ATTEMPT; if [ -n "$SF_PROBE_SECRET" ]; then echo RESULT_OK visible; else echo RESULT_ERR not-visible; fi'
    local eff=no; grep -q 'RESULT_OK' "$OUT" && eff=yes
    finish "env credential SF_PROBE_SECRET visible to child" denied "$eff" "$(mk ATTEMPT)" yes \
        "host_set_env=yes" "child: $(res)"
}

sc_net_common() {
    # $1 = scenario number, $2 = name, $3 = expected (denied|allowed), $4 = snippet
    SCN=$1; reset_canary
    run_child "$4"
    local eff=no; grep -q 'RESULT_OK' "$OUT" && eff=yes
    finish "$2" "$3" "$eff" "$(mk ATTEMPT)" yes "$HOSTCHK" "child: $(res)"
}

sc_7() {
    local exp=denied; [ "$variant" = B ] && exp=allowed
    SCN=7; reset_canary
    run_child 'echo ATTEMPT; if curl -sS -m 5 -o /dev/null http://127.0.0.1:@PORT@/probe-7; then echo RESULT_OK; else echo RESULT_ERR; fi'
    sleep 1
    local eff=no; grep -q 'GET /probe-7' "$HLOG" && eff=yes
    finish "TCP connect 127.0.0.1 (HTTP GET to listener)" "$exp" "$eff" "$(mk ATTEMPT)" yes \
        "http_log_probe7=$eff" "child: $(res)"
}

sc_8() {
    local exp=denied; [ "$variant" = B ] && exp=allowed
    SCN=8; reset_canary
    run_child 'echo ATTEMPT; if printf probe-8 | nc -u -w 2 127.0.0.1 @UPORT@; then echo RESULT_OK; else echo RESULT_ERR; fi'
    sleep 1
    local eff=no; grep -q 'probe-8' "$ULOG" && eff=yes
    finish "UDP send 127.0.0.1 (datagram to listener)" "$exp" "$eff" "$(mk ATTEMPT)" yes \
        "udp_log_probe8=$eff" "child: $(res)"
}

sc_9() {
    local exp=denied; [ "$variant" = B ] && exp=allowed
    local hostdns=no
    dscacheutil -q host -a name example.com 2>/dev/null | grep -q ip_address && hostdns=yes
    HOSTCHK="host_resolves_example=$hostdns"
    sc_net_common 9 "DNS lookup example.com (dscacheutil)" "$exp" \
        'echo ATTEMPT; if dscacheutil -q host -a name example.com 2>&1 | grep -q ip_address; then echo RESULT_OK; else echo RESULT_ERR; fi'
}

sc_10() {
    local exp=denied; [ "$variant" = B ] && exp=allowed
    local hostcode
    hostcode=$(curl -s -o /dev/null -w '%{http_code}' -m 10 https://example.com 2>/dev/null)
    HOSTCHK="host_https_example=$hostcode"
    sc_net_common 10 "outbound HTTPS https://example.com" "$exp" \
        'echo ATTEMPT; c=$(curl -sS -m 10 -o /dev/null -w "%{http_code}" https://example.com); echo "http=$c"; if [ "$c" = 200 ]; then echo RESULT_OK; else echo RESULT_ERR; fi'
}

sc_11() {
    SCN=11; reset_canary; rm -f "$CANARY/escape-11.txt"
    run_child 'echo ATTEMPT; /usr/bin/python3 -c "import sys; print(\"PY_STARTED\"); open(\"@CANARY@/escape-11.txt\", \"w\").write(\"x\")" && echo RESULT_OK || echo RESULT_ERR'
    local eff=no; [ -e "$CANARY/escape-11.txt" ] && eff=yes
    local py=no; grep -q PY_STARTED "$OUT" && py=yes
    finish "grandchild /usr/bin/python3 (sh -> python) writes canary" denied "$eff" \
        "$([ "$(mk ATTEMPT)" = yes ] && [ "$py" = yes ] && echo yes || echo no)" "$(eperm)" \
        "escape_file_present=$eff python_started=$py" "child: $(res)"
}

sc_12() {
    SCN=12; reset_canary
    set -m
    env -i PATH="$PATH" SF_PROBE_SECRET="$SECRET_VAL" "$bin" /bin/sh -c 'echo ATTEMPT; sleep 300 & sleep 300' \
        >"$BUILD/out-${tag}-12.txt" 2>&1 </dev/null &
    local lpid=$!
    set +m
    OUT="$BUILD/out-${tag}-12.txt"
    sleep 3
    local pgid pre post
    pgid=$(ps -o pgid= -p "$lpid" 2>/dev/null | tr -d ' ')
    pre=$(pgrep -fx 'sleep 300' 2>/dev/null | wc -l | tr -d ' ')
    kill -TERM -- "-$lpid" 2>/dev/null
    sleep 2
    post=$(pgrep -fx 'sleep 300' 2>/dev/null | wc -l | tr -d ' ')
    pkill -KILL -fx 'sleep 300' 2>/dev/null
    wait "$lpid" 2>/dev/null; RAW_EXIT=$?
    local marker=no
    if [ "$(mk ATTEMPT)" = yes ] && [ "$pre" -ge 2 ] && [ "$pgid" = "$lpid" ]; then marker=yes; fi
    finish "kill launcher process group: sleep 300 survivors (expected 0)" denied "$([ "$post" -gt 0 ] && echo yes || echo no)" "$marker" yes \
        "sleep_before=$pre sleep_after_group_kill=$post pgid_is_launcher=$([ "$pgid" = "$lpid" ] && echo yes || echo no)" \
        "child: $(res)"
}

# ---- run driver ---------------------------------------------------------------
run_one() {
    variant=$1; run=$2; local mode=$3
    tag="${variant}${run}"
    local net=false
    [ "$mode" = sandbox-net-on ] && net=true
    WS=$(cd "$(mktemp -d "$TMPROOT/sfprobe-ws.XXXXXX")" && pwd -P) || die "workspace"
    CANARY=$(cd "$(mktemp -d "$TMPROOT/sfprobe-canary.XXXXXX")" && pwd -P) || die "canary"
    CANARY_SRC="$CANARY/canary.txt"
    SRV="$BUILD/srv-$tag"; mkdir -p "$SRV"
    HLOG="$BUILD/http-$tag.log"; ULOG="$BUILD/udp-$tag.log"
    : > "$HLOG"; : > "$ULOG"
    PORT=$((20000 + RANDOM % 20000)); UPORT=$((PORT + 1))

    SECRET_VAL="SFTEST-$(openssl rand -hex 12)"
    printf '%s' "$SECRET_VAL" > "$SECRET_FILE"; chmod 600 "$SECRET_FILE"
    SECRET_BYTES=$(wc -c < "$SECRET_FILE" | tr -d ' ')

    bin="$BUILD/launcher-$tag"
    cp "$BUILD/launcher-base" "$bin" || die "copy launcher"
    if [ "$mode" = none ]; then
        codesign -s - --force "$bin" || die "codesign control"
    else
        local ent="$BUILD/ent-$tag.plist"
        sed -e "s|@WORKSPACE@|$WS|g" -e "s|@NETWORK@|$net|g" "$HERE/entitlements.plist.in" > "$ent"
        plutil -lint "$ent" >/dev/null || die "entitlements render"
        codesign -s - --force --entitlements "$ent" "$bin" || die "codesign"
    fi
    echo "--- codesign entitlements: variant=$variant run=$run mode=$mode"
    codesign -d --entitlements :- "$bin" 2>/dev/null | tee "$EVIDENCE_DIR/entitlements-$tag.plist"

    python3 -I -m http.server "$PORT" --bind 127.0.0.1 --directory "$SRV" >"$HLOG" 2>&1 &
    HPID=$!
    python3 -I "$BUILD/udp_listener.py" "$UPORT" "$ULOG" >"$BUILD/udp-err-$tag.log" 2>&1 &
    UPID=$!
    local i ready=no
    for i in $(seq 1 60); do
        if curl -s -o /dev/null -m 2 "http://127.0.0.1:$PORT/ready"; then ready=yes; break; fi
        sleep 0.25
    done
    [ "$ready" = yes ] || die "http listener not ready on $PORT"
    ready=no
    for i in $(seq 1 40); do
        printf ready-udp | nc -u -w 1 127.0.0.1 "$UPORT" 2>/dev/null
        grep -q ready-udp "$ULOG" && { ready=yes; break; }
        sleep 0.25
    done
    [ "$ready" = yes ] || die "udp listener not ready on $UPORT"

    local scenarios="1 2 3 4 5 6 7 8 9 10 11 12"
    [ "$mode" = sandbox-net-on ] && scenarios="7 8 9 10"
    local n
    for n in $scenarios; do
        "sc_$n"
    done

    kill "$HPID" "$UPID" 2>/dev/null; wait "$HPID" "$UPID" 2>/dev/null
    HPID=""; UPID=""
    rm -f "$SECRET_FILE"
}

run_one A 1 sandbox-net-off
run_one B 1 sandbox-net-on
run_one C 1 none
run_one D 1 sandbox-net-off
run_one D 2 sandbox-net-off
run_one D 3 sandbox-net-off

# ---- verdict ------------------------------------------------------------------
count() { grep -F "$1" "$RESULTS" | grep -c "\"status\": \"$2\"" || true; }
errors=$(grep -c '"status": "ERROR"' "$RESULTS" || true)
c1=$(count '"variant": "C", "run": 1, "scenario": 1,' PASS)
c2=$(count '"variant": "C", "run": 1, "scenario": 2,' FAIL)
c5=$(count '"variant": "C", "run": 1, "scenario": 5,' FAIL)

{
    echo "## macOS App Sandbox probe"
    echo
    echo "| variant | run | # | scenario | expected | observed | host check | status |"
    echo "|---|---|---|---|---|---|---|---|"
    cat "$TABLE"
    echo
    echo "ERROR rows: $errors. Control C proof (writes succeed without sandbox): scenario1 PASS=$c1, scenario2 FAIL=$c2, scenario5 FAIL=$c5."
} | tee "$EVIDENCE_DIR/summary.md" >> "${GITHUB_STEP_SUMMARY:-/dev/null}"

if [ "$errors" -gt 0 ]; then
    echo "FAIL: $errors ERROR row(s); probe is not trustworthy." >&2
    exit 1
fi
if [ "$c1" -lt 1 ] || [ "$c2" -lt 1 ] || [ "$c5" -lt 1 ]; then
    echo "FAIL: control variant C did not show escapes; the probe cannot detect them." >&2
    exit 1
fi
exit 0

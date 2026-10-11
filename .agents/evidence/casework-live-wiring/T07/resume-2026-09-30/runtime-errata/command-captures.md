# Captured command/output record

Tool calls below were made from `/home/sprime01/projects/sea-rs`. Local gateway requests used a direct `require_escalated` execution because the ordinary sandbox cannot connect to host loopback (`EPERM`). This was the explicitly authorized isolated runtime posture. Cookie values were never printed or written to evidence.

## Commands with captured results

### Current case projections and replayed intent IDs

Command: `python3 /tmp/t07_runtime_followup.py` (direct approved loopback call; completed).

Captured output:

```text
replay IDs case_20260930T140745Z_e1b7b1 01M3SA934XZWCW51RBD663EMER t07-t07-a-commit case_20260930T140745Z_aff333 01M3SA937ZBR3CA1B1PJQ559RY t07-t07-b-commit
current {"operator_actions": ["EXECUTE_ITEM", "TERMINATE_CASE"], "operator_cursor": "01M3SA93B1WBRTHANZ6G68G1PN", "operator_prepare": "COMPLETED", "operator_publish": "READY_TO_BEGIN", "rso_actions": [], "rso_cursor": "01M3SA93B1WBRTHANZ6G68G1PN", "rso_prepare": "COMPLETED"}
old snapshots: operator/R-SO perspectives both returned at commit cursor 01M3SA934XZWCW51RBD663EMER; operator had EXECUTE_ITEM, R-SO had none.
```

The output contained full old snapshot JSON; the compact restatement above omits no cookie/session value. Full stdout remains in the parent conversation transcript, not a separate file. Wrapper exit code was not retained.

### First mutation frame assertion (preserved harness failure)

Command: `python3 /tmp/t07_runtime_full.py` (direct approved loopback call; completed with an assertion failure at the test's global-action expectation).

Captured result: traceback at line 153, which asserted snapshot-level `EXECUTE_ITEM` existed for the operator during the first post-execute frame. The assertion was wrong for an `IN_PROGRESS` target item. The test had already completed role-specific replay/live SSE, operator actor overwrite, unknown-cursor delegation refusal, query override refusal, cross-case cursor refusal, and the first governed task_prepare execution. No source change followed. Exact failure output is in the parent conversation transcript; it was not saved to a separate file. Wrapper exit code was not retained.

### Second mutation cursor equality assertion (preserved harness failure)

Command: `python3 /tmp/t07_runtime_mutation.py` (direct approved loopback call).

Captured output ended at line 137:

```text
assert new_op['cursor']==target and new_rs['cursor']==target
AssertionError
```

The SSE target had completed; a later current projection cursor was newer than the intent response cursor. No production failure was inferred from equality alone. Exact exit code was not retained.

### Target-item action check while mutating case B (preserved assertion failure)

Command: `python3 /tmp/t07_runtime_immutability.py` (direct approved loopback call).

Captured state-aware frames before failure:

```text
operator: task_prepare IN_PROGRESS at 01M3SAK69RNEAH70YPWPQJR86V; COMPLETED at 01M3SAK6A126CSFHABW8XRFA9T and 01M3SAK6A23NWBZVYPQGM7NHW2; actor operator_local/operator
R-SO:      same statuses/cursors; actor rso_local/R-SO; no item actions
```

The script then failed because it equated top-level `available_actions` with `task_prepare` status. Snapshot-level EXECUTE_ITEM may belong to a different ready item (`task_publish`). The later target-specific probe checked `visible_objects[id].actions`. Exact traceback and full output are in the parent conversation transcript; wrapper exit code was not retained.

### Target-item SSE for task_publish (preserved cursor timing assertion failure)

Command: `python3 /tmp/t07_runtime_publish.py` (direct approved loopback call).

Captured frames: operator and R-SO both received `task_publish` IN_PROGRESS then COMPLETED frames, with their respective session actors; no per-item execute action remained after completion. Returned target cursor was `01M3SAMJ471DG4KV8JNGT5FE2A`; a later current API read returned `01M3SAMJ4BCV4EM2V6TPNW7QGM`. The script stopped because it incorrectly demanded current cursor equal the earlier target cursor. Wrapper exit code was not retained.

### Accepted cursor immediate lookup failure (blocking runtime evidence)

Command: `python3 /tmp/t07_runtime_immutable_c.py` (first direct approved loopback run using the PROPOSE_CASE response cursor immediately).

Captured failure: `GET /api/world?case_id=...&cursor=01M3SAPGD6W1CZYNPCPAZS58M3` returned HTTP 400 `invalid`, note `unknown or evicted kernel cursor 01M3SAPGD6W1CZYNPCPAZS58M3; refetch the live world`. No case-C governed mutation had yet been submitted by that script. A second run used `/api/world` to read the current cursor and then the same historical cursor; it succeeded. Later `/api/trajectory` confirmed this cursor was retained as the case's base. Full first traceback is in the parent conversation transcript; wrapper exit code was not retained.

### Corrected case-C immutability and state-aware SSE probe

Command: `python3 /tmp/t07_runtime_immutable_c.py` (direct approved loopback call after selecting current API cursor as baseline).

Captured output:

```text
case case_20260930T141504Z_dd74f9 old 01M3SAPGD6W1CZYNPCPAZS58M3 mutation response cursor 01M3SAQG0PJAXJMQ455FGMEHCA latest 01M3SAQG0YER6KXDGRYEEF89HB
old API bodies byte-identical across governed mutation 1685 1281 operator/R-SO
state-aware SSE operator and R-SO identities correct; task_prepare COMPLETED; task_publish READY_TO_BEGIN offered only to operator at item-level
```

Wrapper exit code was not retained; command completed with its assertions satisfied.

### CSRF, origin, structured log probes

Command: `python3 /tmp/t07_runtime_security.py` (direct approved loopback call).

Captured output:

```text
write refusals: missing CSRF=403 csrf_refused; untrusted Origin POST=403 csrf_refused; current cursor unchanged 01M3SAQG15NEJCPQ8VQCV9PVED
origin: untrusted GET=200 without Access-Control-Allow-Origin; no browser CORS access
log: newline intent ID refused; one escaped structured record, no forged second line; successful intent correlation ID present
```

Wrapper exit code was not retained; command completed with its assertions satisfied.

### Post-revocation world and SSE checks

Command: `python3 /tmp/t07_runtime_revoked.py` (direct approved loopback call).

Captured output:

```text
gateway ready_http 200 kernel restarted in same cell with operator delegation removed
cached history/SSE after revocation: operator world 403 authority_denied; operator events 403 authority_denied; R-SO world 200; R-SO events 200
```

Wrapper exit code was not retained; command completed with its assertions satisfied.

### Live trajectory query

Command: `python3 /tmp/t07_runtime_trajectory.py` (direct approved loopback call, R-SO only).

Captured output:

```json
{"base_cursor":"01M3SAPGD6W1CZYNPCPAZS58M3","case_id":"case_20260930T141504Z_dd74f9","first_point_summary":"case.submitted","head_cursor":"01M3SAQG15NEJCPQ8VQCV9PVED","http":200,"last_point_summary":"case.trace.item_enabled","point_count":6,"point_cursors":["01M3SAPGD6W1CZYNPCPAZS58M3","01M3SAQG0PJAXJMQ455FGMEHCA","01M3SAQG0YER6KXDGRYEEF89HB","01M3SAQG13GCZEM8D6KBV2YAME","01M3SAQG14BYH1147PC66560Q4","01M3SAQG15NEJCPQ8VQCV9PVED"]}
```

Wrapper exit code was not retained; command completed with its assertions satisfied.

### Production dev-auth refusal

Actual command:

```text
/tmp/casework-resume-gateway -config /tmp/t07-final.uUPtaO/serve-production-dev-auth-refusal.json -serve -addr 127.0.0.1:44180
```

Captured result (`exec_command.exit_code=2`):

```text
godspeed-casework: config error: auth.mode dev is REFUSED in the production posture: the dev surface skips password verification and may carry a static bearer token; configure auth.mode local or oidc for production
```

The config had `serve.production=true`, a production-valid HTTPS origin, and the test config's existing `auth.mode=dev`. Port 44180 was absent from the post-run listener check.

## Process and runtime preservation checks

After the approved delegation restart, a read-only `/proc` scan found kernel PID `848392`, executable `/home/sprime01/projects/sea-rs/target/debug/sea-forge-server`, with the fresh cell's `.server.lock` and `server.sock.lock` descriptors. Gateway PID `800034` remained alive. A listener check showed gateway on `127.0.0.1:44179` and Vite on `127.0.0.1:4178`; port `44180` was absent. No cleanup was performed.

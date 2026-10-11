=== F1 manual drive 2026-09-26T01:43:27Z — gateway 127.0.0.1:4181, production binary (no build tags), real kernel cell /tmp/t06c-critic-cell ===

--- GET /api/world?actor=operator_local&role=operator (allowlisted) BEFORE any case ---
HTTP 200
{"snapshot":{"world_id":"world-empty","case_id":"","cursor":"","timestamp":"2026-09-26T01:43:27Z","perspective":{"actor_id":"operator_local","role":"operator"},"summary":{"headline":"No cases yet","phase":"idle","status_phrase":"Commit a case from a template to begin."},"visible_objects":[],"available_actions":[],"attention_focus":{"primary_object_id":""}}}


--- GET /api/world?actor=operator_b&role=operator (BOUND but NOT in delegable_actors) ---
HTTP 403
{"error":{"kind":"authority_denied","note":"the kernel does not allow this connection to act as operator_b: authority_denied: delegated identity refused: actor `operator_b` is not in this cell's gateway-delegable allowlist (identity)"}}


--- GET /api/world?actor=operator_c&role=operator (not bound, not allowlisted) ---
HTTP 403
{"error":{"kind":"authority_denied","note":"the kernel does not allow this connection to act as operator_c: authority_denied: delegated identity refused: actor `operator_c` is not in this cell's gateway-delegable allowlist (identity)"}}


--- POST /api/templates/preflight (sentry-chain) ---
{"template_ref":"e2e-sentry-chain@0.1.0","params":{"dataset_label":"c2","dataset_name":"critic2-ds","max_rows":"25","out_dir":"work"},"passed":true,"reasons":[],"digest":"sha256:385efd013fb7ef9007a6995aebc1cbce869ac79b3cc1c00c0432371dfdfaf2cd"}

digest=sha256:385efd013fb7ef9007a6995aebc1cbce869ac79b3cc1c00c0432371dfdfaf2cd

--- POST /api/intents PROPOSE_CASE (actor operator_local via envelope) ---
{"intent_id":"critic2-propose-1790387052","success":true,"new_cursor":"01M3DP4QQFK8BEB1SQRCT4DX8K"}


--- GET /api/world?actor=operator_local&role=operator AFTER the commit (must be 200 with real standing) ---
HTTP 200
perspective: {'actor_id': 'operator_local', 'role': 'operator'}
case_id: case_20260926T014412Z_1b967f cursor: 01M3DP4QQFK8BEB1SQRCT4DX8K
summary: {'headline': 'Case in progress', 'phase': 'active', 'status_phrase': '1 item(s) ready to execute.', 'progress_percent': 0}
Traceback (most recent call last):
  File "<stdin>", line 7, in <module>
KeyError: 'action_name'

--- negative re-check after case exists: operator_b again ---
operator_b -> HTTP 403

=== F2 manual drive: ADD_DISCRETIONARY_WORK over served HTTP (case case_20260926T014412Z_1b967f, cursor 01M3DP4QQFK8BEB1SQRCT4DX8K) ===
{"intent_id":"critic2-disc-add-1","success":true,"new_cursor":"01M3DP9WGV1PVAJFPP7K1JCM3Y","resulting_object":{"id":"item-disc-89880b6a","kind":"work_item","name":"Critic2 Augmented Notes","status":"WAITING","badge":"Proposed","explanation":"Proposed by operator_local; it waits on task_prepare's entry sentry.","salience":0.5,"actions":[]}}

--- durable case-events.jsonl after the add ---
kind counts: {'case_created': 1, 'item_enabled': 1, 'plan_mutated': 1}
plan_mutated count: 1
plan_mutated payload keys: ['operation']
last event: plan_mutated

--- EXECUTE task_prepare then the discretionary item (served HTTP) ---
{"intent_id":"critic2-disc-exec-prepare-1","success":true,"new_cursor":"01M3DPAP6GYE0K60FFG8Y256NJ"}


=== F3/L5 drive: signoff-gate commit -> execute task_draft -> approval -> R-SO resolves ===
--- preflight signoff-gate ---
{"template_ref":"e2e-signoff-gate@0.1.0","params":{"change_summary":"critic2 signoff"},"passed":true,"reasons":[],"digest":"sha256:cdbb4b4d69e896104c510aa007658ad8a0a1e6e333d5ad330a3951dd450ce78e"}

--- PROPOSE_CASE signoff-gate ---
{"intent_id":"critic2-signoff-propose-1","success":true,"new_cursor":"01M3DPDQJ6F1FW467E6YAXQ522"}

--- EXECUTE task_draft on case_20260926T014907Z_f90434 (governed write to review/draft.md must escalate) ---
{"intent_id":"critic2-signoff-exec-draft-1","success":true,"new_cursor":"01M3DPE1TMW63B1WBQPSTRA9CE"}

--- operator_local APPROVE_HUMAN_TASK on own work (must be refused) ---
{"intent_id":"critic2-signoff-approve-self-1","success":false,"refusal":{"refusal_kind":"UNAUTHORIZED_ROLE","message":"the action list for role operator does not offer APPROVE_HUMAN_TASK on this projection; the gateway will not route it to the kernel"},"error_code":"UNAUTHORIZED_ROLE","error_message":"the action list for role operator does not offer APPROVE_HUMAN_TASK on this projection; the gateway will not route it to the kernel"}

--- rso_local (R-SO) APPROVE_HUMAN_TASK with justification ---
{"intent_id":"critic2-signoff-approve-rso-1","success":true,"new_cursor":"01M3DPFJ2ADBHG405QAJE5WAZ9"}

summary: {'headline': 'A decision is waiting on your approval', 'phase': 'awaiting_approval', 'status_phrase': 'The case is parked until the pending approval is resolved.', 'progress_percent': 0}
task_draft work_item FAILED | Execution was deliberately stopped before completing. | actions: []
signoff_release work_item ACTION_REQUIRED | Parked for a human decision; completing the task resolves it. | actions: ['act-complete_human_task-signoff_release']
--- operator_local COMPLETE_HUMAN_TASK signoff_release (resolves the gate) ---
{"intent_id":"critic2-signoff-complete-1","success":false,"refusal":{"refusal_kind":"JUSTIFICATION_REQUIRED","message":"A human-task completion is a governed judgment: a recorded justification is mandatory."},"error_code":"JUSTIFICATION_REQUIRED","error_message":"A human-task completion is a governed judgment: a recorded justification is mandatory."}


=== F4 stale-cursor tooth: EXECUTE_ITEM with an OLD cursor must be refused, no kernel write ===
requests before: 8
{"intent_id":"critic2-stale-exec-1","success":false,"refusal":{"refusal_kind":"STALE_PROJECTION","message":"The kernel has moved past this projection for case_20260926T014412Z_1b967f; refetch the world and retry with the fresh cursor.","current_cursor":"01M3DPB0PFANGCRYGEE8SC2WGP"},"error_code":"STALE_PROJECTION","error_message":"The kernel has moved past this projection for case_20260926T014412Z_1b967f; refetch the world and retry with the fresh cursor."}


=== F5 live: kernel plan_schema_error must surface as INVALID ===
{"intent_id":"critic2-schema-err-1","success":false,"refusal":{"refusal_kind":"STALE_PROJECTION","message":"The kernel has moved past this projection for case_20260926T014412Z_1b967f; refetch the world and retry with the fresh cursor.","current_cursor":"01M3DPB0PFANGCRYGEE8SC2WGP"},"error_code":"STALE_PROJECTION","error_message":"The kernel has moved past this projection for case_20260926T014412Z_1b967f; refetch the world and retry with the fresh cursor."}


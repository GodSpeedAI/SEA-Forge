# Unit 1 immediate-pre-edit backup audit

Date: 2026-10-07.

## Result

The required immediate-pre-edit copies of repair-1 candidate identities
`aad810b928590acc712eb5d5eeac15a5358a85e75d2a28d7acce0bd3c9fdce0f` and
`64dc5a074b926aed6da85f6070ad568dd61fec9b7c88d1f4e3d7265437af3f6a` are
**missing** from the package. An exhaustive filename check in
`apps/godspeed-casework-go/internal/server` found only the earlier original
baseline copies:

* `run_observation_manager_unit1_source_preedit.txt` — SHA-256
  `ee7afab1b22c011cf79b602f415336d459fda346aad1bf0c942a4464326ff878`.
* `run_observation_manager_unit1_test_preedit.txt` — SHA-256
  `2b2d1fbcd3bfb7e7c27254673ca46b2deb30d9edd4ac862fa464e2a9b18334b6`.

The `.agents` archive has `.raw` copies of these same earlier baseline bytes;
the root archive note explicitly identifies them as `ee7afab1` and `2b2d1fb`.
The repair-1 record and its independent review preserve only the `aad810` and
`64dc` hash identities, not file payloads.

## Recovery decision

No `aad810`/`64dc` backup was created by this worker before repair 2. I did
read the then-current source and test files through tool output before editing,
but the combined output was truncated and does not provide a complete,
verified byte stream for both files. I therefore did not reconstruct or
fabricate immediate-pre-edit backups. The repair-2 source/test remain frozen
at their separately recorded identities; they must not be represented as the
missing repair-1 copies. No original evidence or prior record was changed.

This note is the complete preservation audit. It records an omission, not a
recovery. No test, compiler, Git, or runtime command was run.

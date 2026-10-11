# Local cursor-order actual RED — exact capture path addendum

Date: 2026-10-06. This immutable addendum supplements `local-cursor-order-phase1-actual-red-independent-review-oct06.md` (SHA-256 `c2c65c25af3679a9a392b89fa4dcdd86bb863e36a810be47ae5aac463f2df531`). It supplies the complete original and repository archive paths and preserves the observed filename/content mismatch.

The actual captures are:

- `/tmp/sea-casework-20261006-ui-cursor-order-red01-preflight.raw`, SHA-256 `78dd1a15463439e7566a7a2b7ff84a3ba81bf7525840017d7134d9e72f12a7c3`; exact repository copy `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/ui-cursor-order-red01-preflight-direct-original-copy.raw`.
- `/tmp/sea-casework-20261006-ui-cursor-order-red01-go-output.raw`, SHA-256 `4d31b7e2ca43947ed546d8cd4f73310525f5dcfa57126dc771bc6fc55d249419`; this is Bun output despite its `go-output` basename. Its exact repository copy is explicitly named `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/ui-cursor-order-red01-bun-output-direct-original-copy.raw`.
- `/tmp/sea-casework-20261006-ui-cursor-order-red01-exit.raw`, SHA-256 `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22`; exact repository copy `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/ui-cursor-order-red01-exit-direct-original-copy.raw`.

I independently ran byte comparisons for all three pairs; each `cmp -s` exited 0. The naming mismatch is preserved and disclosed, not normalized by rewriting the original capture.

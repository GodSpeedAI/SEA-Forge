# Present-context focused RED — root acceptance and direct archive provenance

Date: 2026-10-06. Root accepts the focused semantic RED only, not implementation or T09 completion.

Independent runtime review b67ca790 records session13235 joined, actual Go exit1 against stub565ad7a6 and fixturefffa0335. Root read the complete review and independently verified both frozen source hashes. Negative typed-unavailable passes do not prove validation; positive and exact-dependency-call failures distinguish the stub from the requested algorithm. Subsequent success assertions remain unexecuted.

Root read the actual original `/tmp/sea-casework-20261006-present-context-red-01-run.raw` directly through the native command tool: exit0,8327bytes, no truncation. A native apply_patch payload was derived mechanically from those returned bytes, with no manual rearrangement. Its first approval attempt timed out; the one expressly permitted retry succeeded. The new `present-context-red-01-run-root-direct-original-copy.raw` compares byte-for-byte with the original (cmp exit0), SHA-256 `b6dd85e7ccebada186ca192817344a0789bd853a0b0c8bbd538037ce10582a85`.

Root also compared original preflight and exit captures with their existing archives: both cmp exit0. Their SHA-256 values are respectively `6b333c4bf819ef8cbb5787f7c97c8b0085026469df22b6436849c42a93db1b16` and `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22`.

Preserve the first incorrect `present-context-red-01-run.raw` and the independent review unchanged. That archive transposes two adjacent PASS lines and is not authoritative. Automatic review rejected the worker's manually reconstructed replacement; this accepted alternative copies the actual original bytes directly and verifies identity. No original evidence was overwritten, no test rerun occurred, and no passing outcome was manufactured.

Compiler ownership is returned to idle. Root releases only the existing private guard source file to a separate Luna builder under all eight original eventual-algorithm requirements, with the complete fixture frozen. Independent source review and serialized GREEN gates remain necessary before acceptance. No caller, manager, public schema, authority, or continuous-readiness claim is released.

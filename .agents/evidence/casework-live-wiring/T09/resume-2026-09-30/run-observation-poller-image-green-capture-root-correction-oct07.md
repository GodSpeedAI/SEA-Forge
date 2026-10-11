# Encoder GREEN output capture correction

The first output archive is not byte-accurate: its two tab bytes were replaced
by spaces. It remains immutable, with SHA256
578a6c123290bacc28460ced7f46859177b084e4519f8482dded99a7f80f6675.
It cannot support acceptance as an exact original capture.

Automatic review rejected the critic's inaccurate correction attempt and
directed preservation of the untouched original. Root fulfilled that direction
using the same native apply_patch tool: read the actual file directly into a
JavaScript string, construct a new patch from that exact string without manual
transcription, and compare the result immediately. No alternate execution
mechanism, escalation, or approval bypass was used.

Original: /tmp/poller-image-green-test-8sWTTH/test.output.raw.
New archive: run-observation-poller-image-green-output-exact-root-oct07.raw.
Both are 1829 bytes with SHA256
809720a6df88052daa6d5b52b423d28fe6c70c925a8cc8963423b04ee7ef412d.
Root's direct cmp exited0. The original remains present.

This correction establishes only output-copy identity. The independent critic
must compare it again and verify the other actual captures before accepting
the bounded encoder unit. No lifecycle, full-module or T09 settlement follows.

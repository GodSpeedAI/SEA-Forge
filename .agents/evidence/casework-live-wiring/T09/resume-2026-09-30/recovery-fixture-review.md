# T09 recovery fixture review

Recovery found no surviving workers; T08 remains settled at af362f5. New Luna workers
resume independent canonical fixture review and a fresh subscription fixture repair.
Only t09_fixture_critic owns the compiler token. No implementation is authorized yet.

The critic reported canonical Bun expected RED: eight passing tests and three failures
for missing Ask schema and missing event literals. Initial stdout was not durably logged;
automatic approval review rejected reconstructing that output. A fresh captured run is
required for durable evidence. Do not claim that the missing original log exists.

The canonical test's TypeScript Expect/OptionalKeys pins are outside the existing UI
tsconfig include graph, and Bun erases types. A targeted existing TypeScript CLI check
is required before claiming those pins enforce parity; no dependency/config change is
authorized for that check.

The fresh subscription fixture is independently REJECTED: its writer offers exactly
cap+1 bytes but requires an incomplete write. A correct reader can consume the detection
byte and close after that write succeeds. Fresh t09_subscription_writer_repair adds
trailing offered bytes after the valid oversized first line, preserving delivery and
EOF/deadline refusals and the other seven tests. Independent source review and actual
expected RED remain required.

Unrelated Jolli log and trusted-daemon target plan/spec files are preserved.

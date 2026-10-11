# Root check: repaired run-child fixture

2026-10-01. Independent source/expected-RED review approved the test-first fixture only.
Root matched current source SHA4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741
and review SHA4b4450d6797c29eba4c63bee2f7069fef172692b4167db223f46493d942e07c8.

Four raw files were independently byte-compared. Originals /tmp/t09-run-children-focused-red
and /tmp/t09-run-children-focused-red-retry, each with .log and .exit, map respectively to
run-children-expected-red-first and run-children-expected-red-retry siblings.
First log SHA1d8b35271a25dd84c640696c5b416e6af547acfa97413e7337f55c87fd9d2b2f;
retry log SHA9f4106ff4ddf5c583931eca8977423a255a8dcfddd6586a87d1aedfcb35ccbf3;
both exits SHA4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865.

The first attempt lacked its task-owned temp directory. Retry compiled and failed because
Build emitted no children: all24 required IDs, the valid sibling and historical child were
absent. Negative guards after the valid-sibling fatal did not execute; attention regression
can pass when no child exists. Runtime GREEN must exercise those remaining assertions.

Preflight1 SHAfe4d48562dc9f14c8d69cc7315040bf1f7f91ec2f836bd46943e893d4633d91b;
preflight2 SHA7f0277964b85193d37d4954652f1527815a8c733d126dc8d4abf5a35b87d4b9b.
These are hashes of archived native-tool output, not raw /tmp byte comparisons: the critic
did not save separate original preflight files. Do not claim six exact original copies.
Future gates must save raw preflight stdout separately to allow exact archive verification.

Root releases runtime phaseA declarations/scoped tests only under original assignment plus
run-children-runtime-test-phases.md. No runtime implementation or compiler token yet.

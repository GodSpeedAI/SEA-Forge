# SFWP cap implementation assignment

Builder: Luna t09_subscription_writer_repair. Original cap test-first assignment remains
binding. Fresh fixture source-approved; host race/count1 RED reaches all eight cases,
including actual oversized valid event delivery. Log cap-repaired-independent-red.log
SHA6561396751e2bcb8d9ed138099ee208481c2864b1646587159c435a149c50332.

Own only client.go and subscribe.go, with a tiny private shared helper file only if necessary.
Read every file and Graft caller context before editing. No tests, Ask request/port/classification,
dependencies, generated files or public API beyond approved Config cap. Native apply_patch only.
No compilation/tests; t09_fixture_critic retains exclusive compiler token. Freeze actual diff,
source reasoning and material deviations for independent review. Tests stay frozen.

Implement incremental response-line cap before decode on every RPC/retry/status recovery and
subscription ack/event read path. Default32MiB including LF; positive lower limit accepted
through ceiling, negative/above ceiling typed KindConfig before dialing. Overflow closes/poisons
the owned connection and returns typed unavailable; no reuse of partial positional responses.
Accumulate only bounded bytes, distinguish exact limit and truncated EOF honestly. Preserve
coalesced multiple lines and read-ahead data rather than discarding the next response/event.
Use the simplest stdlib reader; avoid unbounded ReadBytes in sibling paths. Honor logical cap
plus detection byte and disclose bounded underlying read-ahead rather than claiming strict
socket consumption or total allocation without direct proof.

Inspect retries once on fresh connection. Correlated mutation never resends after unknown
transport outcome; request_get_status recovery and its own read retry remain intact. Do not
claim per-line cap bounds aggregate wire bytes, kernel work or cumulative allocation.

Independent critic receives this original assignment and resulting source, verifies all eight
focused GREEN cases and broader adapter/module gates with current source hashes, actual host
RAM/process preflights and sole compile ownership. Approval requires direct evidence and every
material deviation; fresh builder after rejection. Root retains integration and settlement.

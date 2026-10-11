# Ask fixture root integration checks

2026-10-01. Root checked all four frozen source hashes and both expected-RED log hashes
against ask-fixture-independent-review.md. Root additionally compared both expected-RED logs
and both exit files byte-for-byte with the exact original temporary captures listed in
ask-fixture-preflight-capture-erratum.md: all four matched.

The earlier sandbox failure artifact failed that byte comparison because it included explanatory
text. Its original content remains untouched; ask-fixture-capture-erratum-v2.md records the
defect. A fresh archive builder added separate original raw copies. Root independently read and
compared both new copies with the original captures; both matched:

| New artifact | Bytes | SHA-256 |
|---|---:|---|
| ask-fixture-server-sandbox-original-raw.log | 1974 | f56f172b378253b930af59ee47e4428e8138070d7b1422bab5afe210d795ee6d |
| ask-fixture-server-sandbox-original-raw.exit | 2 | 4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865 |

The source-independent archive correction is approved on exact byte identity. It changes no
fixture behavior, expected-RED result or runtime approval. Original host-preflight capture
limitations remain disclosed rather than reconstructed. Following Ask runtime freeze, root
rechecked the original server/ask_test.go and sfwp/ask_transport_test.go hashes; both remain
exactly the independently approved fixture versions. Full Ask runtime confirmation is pending.

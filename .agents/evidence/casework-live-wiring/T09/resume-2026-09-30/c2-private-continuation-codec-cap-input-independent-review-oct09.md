# Private continuation codec grant — corrected document review

**Result: ACCEPTED as a bounded private-codec design proposal only.** This is a fresh review of the unchanged grant together with its cap/input addendum. It releases no implementation.

## Reviewed evidence

- Original grant SHA-256, unchanged: `19be806076544b1f06323c6186bf8aa1484eef4831d907b6827a213c62f16906`.
- Cap/input addendum SHA-256: `3989626f5569cdc50289695a8396550aeb62a9a8b4a9df7b10f06630a5e9d559`.
- Approved C2 supplement SHA-256: `8ccf30be4131de57e6118bdbdd9f272c20edfd47bf0915f0fe74c7c5edfe3b11`.
- Approved registration/accounting decision SHA-256: `f0a5ec83e9a84e7ffa45ad69e1df743082f07fc26f45413d7cb5a3077f2c32c8`.
- The grant's original review remains unchanged. The addendum references its SHA-256 `31e58f246fb7639a1b67926efeaf416d0e48cde8508eeb4f31b267194475b37e`.

## Review findings

The addendum resolves the prior ambiguity: a valid token is at most 2,145 bytes; 4,096 bytes passes only the outer cap and then fails payload/layout admission; 4,097 bytes fails before offset computation/slicing; the 2,048-byte payload boundary reaches signature verification, while 2,049 bytes fails before it. It also resolves the UTF-8 question by specifying `&str`, byte-counting via `str::len()`, and checked string slicing at computed boundaries. These are admission-stage clarifications and add no payload format or policy.

The combined proposal aligns with approved REQ-C2-RANGE-001/-003/-004/-005 and V-C2-RANGE-01/-03: exact-byte signature verification precedes JSON/position use; payload and position digests bind registered identity and filters; ACK/frontier semantics remain explicit; actual row/checksum revalidation remains the reader's responsibility. It adds no dependency, public contract, authority, startup lifecycle, ledger access, or registration change. The existing signing API path was verified in the prior independent review and remains accurate.

No remaining material design ambiguity or deviation was found in the bounded codec slice. The grant and addendum correctly remain held for explicit root implementation release. Future implementation still needs its own source review and verification; this document acceptance is not evidence of implementation or runtime behavior.

## Scope

Documentation review only. No source, compiler, tests, gates, Git, status, or debt records were changed or run.

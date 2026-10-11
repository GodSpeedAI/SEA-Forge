# Focused RED receipt citation correction

Date: 2026-10-08

This additive note corrects line references in the immutable receipt
`run-observation-watcher-terminal-focused-red-result-oct08.md` (SHA-256
`c13fa2c54c5b44965a55110b408eb0b2f128ec4cfe7efbb5cefe0f1e5ed2b3b9`). It does
not alter its outcomes, capture identities, or comparisons. The line numbers
below refer to the untouched actual output
`/tmp/watcher-terminal-focused-red.DUHxkr/output.raw`, SHA-256
`7a4c8a38511f18e79d0b0e35b251b2a9619e258083c876da90d9478d4cabced7`.

Correct direct failure locations are:

| Group/case | Actual output line |
|---|---:|
| Fitting terminal later read | 2 |
| Recurring outer-cap cancellation-signal timeout | 5 |
| Revoke before selected read | 8 |
| Revoke during read | 11 |
| Cursor change during read | 14 |
| Shared survivor, during held read | 18 |
| Shared survivor, invalid initiator before port | 20 and 21 |
| Final handoff, auth-invalid empty list | 27 |
| Final handoff, auth-invalid unavailable list | 29 |
| Final handoff, cursor-invalid empty list pass | 36 |
| Final handoff, cursor-invalid unavailable list | 32 |

The seven top-level test results remain at output lines 3, 6, 9, 12, 15, 22,
and 33. The six nested results are at lines 23–24 and 34–37. The earlier
receipt table's shifted references were documentation errors only; all claims
are supported by the actual output and raw capture.

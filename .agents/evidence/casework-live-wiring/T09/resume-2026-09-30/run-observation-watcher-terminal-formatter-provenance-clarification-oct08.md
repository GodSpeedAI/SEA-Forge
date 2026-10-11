# Formatter capture provenance clarification

Date: 2026-10-08

The formatter result JSON files beside `run-observation-watcher-terminal-algorithm-result-oct08.md` retain the full exact stdout bytes, command strings, and exit codes. The preflight JSONs retain the bounded process/memory observations; the pre-format source JSONs retain the exact source bytes. The final source was compared byte-for-byte with decoded formatter stdout.

The original `/tmp` capture files and their original paths were not preserved or recorded. A read-only search for watcher-terminal formatter capture names found no matching files; the broad `/tmp` search also emitted permission-denied notices for unrelated private system directories, which were not inspected. No reconstructed file is presented as an original capture. This is a provenance deviation from the requested original-temp-path record, not a source/stdout equality discrepancy. No source or prior evidence was changed for this clarification.

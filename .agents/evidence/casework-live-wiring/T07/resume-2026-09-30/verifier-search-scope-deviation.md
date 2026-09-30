# T07 verifier search scope deviation

The independent verifier reported that a broad read-only `rg` setup-pattern search under
the temporary test-cell directory traversed private ledger files and printed two authority
decision records from test cases. The records were not used as verification evidence and
no files were modified. The verifier reported no credentials exposed.

This was an unnecessary retrieval-scope deviation, not a production gateway ledger reader.
Root instructed the verifier to use exact setup-script paths, avoid repeating those records,
and use kernel queries for current authority/state assertions. The final review must disclose
the deviation. Existing durable-state test evidence must distinguish direct test verification
from production adapters: production source must continue using authorized SFWP interfaces.

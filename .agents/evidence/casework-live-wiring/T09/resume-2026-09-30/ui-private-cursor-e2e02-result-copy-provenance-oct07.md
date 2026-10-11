# E2E attempt02 result copy provenance

The preflight, output, and exit raw archives compare byte for byte with their original runtime files. All eleven entries passed and the process exit was zero.

The first results.json and results.md archives added a final newline through native patch creation. Their original-file comparisons failed solely at EOF; these copies are not byte-exact raw evidence. They remain immutable readable copies. The separately named exact-wrapper JSON files preserve the complete original UTF-8 text, including absence of final newline. Decoding each wrapper and comparing with its original succeeded. These wrappers are source-preservation containers, not raw runtime captures.

# Addendum — d100 ignore-repair scope comparison

This addendum completes the workspace-scope limitation recorded in `d100-two-fingerprint-repair-independent-review-oct08.md`; that review is preserved unchanged. Read-only Git inspection was authorized for this verification. I compared the worktree bytes against commit `d100b9b80986cef4b7c38022b3299b198ee68a03` using `git show` and `git ls-tree` only. No Git mutation was performed.

The two files and all four hook files tracked at d100 are byte-identical to their committed blobs:

| Path | Bytes | SHA-256 |
|---|---:|---|
| `.gitleaks.toml` | 4,601 | `608a687e53ba365fc743ae6c8d2ded7a1c006e8c6b0f5f209109d055f6e5ad05` |
| `justfile` | 75,756 | `4f3305e47c15ab9b213933e2af09761552cf7dde2d2b9ee9f2374f072dfcf625` |
| `.githooks/post-checkout` | 1,648 | `18b65c32193c511f26ad7ec4619328e8ba7981a3501cc88017aa5e69b5898626` |
| `.githooks/post-commit` | 5,111 | `d52cd05b70f79041fa5a6ae07ff8b3f7362097962cb9fcd2076d4a258a1c8329` |
| `.githooks/pre-commit` | 1,024 | `c1489a33f83c70c651c0b06319a31f272dec1da37cbcef4fbffa2a46c4bf0ea1` |
| `.githooks/pre-push` | 986 | `82e6c9e855c0803ce0ac6aa429911dfc3f99129638982f160811e255ecf37544` |

This confirms the fingerprint repair did not alter scanner rules, task recipes, or tracked hooks relative to d100. Combined with the prior direct `.gitleaksignore` prefix/suffix and negative-membership checks, the reviewed repair remains limited to the two exact fingerprint entries. This does not claim scanner runtime behavior or a security-gate pass.

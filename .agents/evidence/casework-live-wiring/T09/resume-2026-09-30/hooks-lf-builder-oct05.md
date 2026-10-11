# Git hook line ending repair — 2026-10-05

Scope: the nine pre-existing `.githooks/*` files only. Each working file was normalized from CRLF to LF. Hook commands, comments, shebangs, inventory, and executable modes were preserved. No hook was executed and no Git mutation was performed.

Verification compared each working file byte-for-byte with the staged file after replacing every CRLF pair with LF. All nine comparisons passed; all staged and working modes were `100755`. Inventory count was 9.

| Hook | Staged SHA-256 (before) | Working SHA-256 (after) | Original CRLF pairs | Normalized byte equality | Mode |
|---|---|---|---:|---|---|
| `.githooks/commit-msg` | `438f5af3f41aee3dfb214f6d8b1063d59d1e8fb191e0a3dded35e8bc6a3c1c77` | `058d3c8202644eba5fd4a78f443d1b41457df4e0e1ff887b7cbcd117c669848a` | 4 | yes | 100755 |
| `.githooks/post-checkout` | `d99f32101ee7db04cdc92bf3af075fa7f0c2f384ccc289827460e6a3877ec6b8` | `18b65c32193c511f26ad7ec4619328e8ba7981a3501cc88017aa5e69b5898626` | 44 | yes | 100755 |
| `.githooks/post-commit` | `9f453e3c83d0328b14b526e8bedae6cb36b7f053bb03f73f3539b61796ce81db` | `b043ee2a40aa9db3fb54b7431309ca12c8fbfbd8a301ac6de005e5d1d6bfa42c` | 9 | yes | 100755 |
| `.githooks/post-commit.pre-entire` | `0c213acfccc6faf6ba2aa33f9e04ea5291cd909e10e30b2c18695db55a0d3a0c` | `d52cd05b70f79041fa5a6ae07ff8b3f7362097962cb9fcd2076d4a258a1c8329` | 110 | yes | 100755 |
| `.githooks/post-rewrite` | `8df2ec7e87667cc36d56c5809bd8ad2b9c7d9e7a396c8c3047e9da95ac2f019f` | `62fdff3348c120fed273622a26454d1e1edf0c6809b67ab994ced7c191561bd2` | 4 | yes | 100755 |
| `.githooks/pre-commit` | `7d7b64f74ec0f8a8455ddabe6a4938a48bcf96e2fe35fd3b742834c5d7d897da` | `c1489a33f83c70c651c0b06319a31f272dec1da37cbcef4fbffa2a46c4bf0ea1` | 26 | yes | 100755 |
| `.githooks/pre-push` | `4a7b2bd5b86aa9965d7de6ab6526f7d11999ca6dd6790c5ded87917b2bc2c391` | `7d5f0250dad85a83276e1a6776201479801178486ab8ac5c69fa74a15eb8c79a` | 10 | yes | 100755 |
| `.githooks/pre-push.pre-entire` | `4e776cb17215fe7db34130a1b1652bddd1d77265009acd3121305261d180541e` | `82e6c9e855c0803ce0ac6aa429911dfc3f99129638982f160811e255ecf37544` | 27 | yes | 100755 |
| `.githooks/prepare-commit-msg` | `12006cf018e07e7f5404a4c3dda290f6ccfeb4c5bcc4c00c05633f8d8c9969ae` | `07ff4b4abfa0b52940aa604bc5c5aae01335dc8eb97f82575aba4cae8ce57870` | 3 | yes | 100755 |

Syntax checks used the interpreter named by each shebang and did not execute the hooks:

| Command | Exit |
|---|---:|
| `sh -n .githooks/commit-msg` | 0 |
| `bash -n .githooks/post-checkout` | 0 |
| `sh -n .githooks/post-commit` | 0 |
| `bash -n .githooks/post-commit.pre-entire` | 0 |
| `sh -n .githooks/post-rewrite` | 0 |
| `bash -n .githooks/pre-commit` | 0 |
| `sh -n .githooks/pre-push` | 0 |
| `bash -n .githooks/pre-push.pre-entire` | 0 |
| `sh -n .githooks/prepare-commit-msg` | 0 |

No material deviation from the requested scope.

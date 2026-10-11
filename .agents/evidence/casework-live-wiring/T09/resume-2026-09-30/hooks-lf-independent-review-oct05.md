# Git hook LF normalization — independent review — 2026-10-05

## Scope and verdict

Reviewed the nine existing `.githooks/*` files against the staged index and the
builder record `hooks-lf-builder-oct05.md`. The working bytes are exactly the
staged bytes with CRLF pairs replaced by LF; no other byte changes were found.
All nine staged entries and working files retain executable mode (`100755` in
the index; `755` on disk). Each hook passed syntax-only parsing using its
shebang-selected shell. This supports approval of the requested line-ending
normalization. No hooks, compilers, builds, tests, or Git mutation commands
were run.

## Independent evidence

The following loop independently compared each index blob to its working file
after the single permitted transform and syntax-checked the working file:

```sh
for f in .githooks/commit-msg .githooks/post-checkout .githooks/post-commit .githooks/post-commit.pre-entire .githooks/post-rewrite .githooks/pre-commit .githooks/pre-push .githooks/pre-push.pre-entire .githooks/prepare-commit-msg; do
  git show ":$f" | perl -pe 's/\r\n/\n/g' | cmp - "$f"
  first=$(head -n 1 "$f")
  case "$first" in
    '#!/bin/sh') sh -n "$f" ;;
    '#!/usr/bin/env bash') bash -n "$f" ;;
    *) exit 1 ;;
  esac
  git ls-files --stage -- "$f"
  stat -c '%a %n' "$f"
done
```

All nine comparisons and syntax checks exited zero. Index modes were `100755`
for every path and filesystem modes were `755`. Staged CRLF counts were 4,
44, 9, 110, 4, 26, 10, 27, and 3, in the order listed above. Independent
working-file SHA-256 values match the builder report:

| Hook | Working SHA-256 |
|---|---|
| `commit-msg` | `058d3c8202644eba5fd4a78f443d1b41457df4e0e1ff887b7cbcd117c669848a` |
| `post-checkout` | `18b65c32193c511f26ad7ec4619328e8ba7981a3501cc88017aa5e69b5898626` |
| `post-commit` | `b043ee2a40aa9db3fb54b7431309ca12c8fbfbd8a301ac6de005e5d1d6bfa42c` |
| `post-commit.pre-entire` | `d52cd05b70f79041fa5a6ae07ff8b3f7362097962cb9fcd2076d4a258a1c8329` |
| `post-rewrite` | `62fdff3348c120fed273622a26454d1e1edf0c6809b67ab994ced7c191561bd2` |
| `pre-commit` | `c1489a33f83c70c651c0b06319a31f272dec1da37cbcef4fbffa2a46c4bf0ea1` |
| `pre-push` | `7d5f0250dad85a83276e1a6776201479801178486ab8ac5c69fa74a15eb8c79a` |
| `pre-push.pre-entire` | `82e6c9e855c0803ce0ac6aa429911dfc3f99129638982f160811e255ecf37544` |
| `prepare-commit-msg` | `07ff4b4abfa0b52940aa604bc5c5aae01335dc8eb97f82575aba4cae8ce57870` |

The builder evidence records the matching staged SHA-256 values and individual
syntax-check exits. No implementation deviation was found. One initial review
command's shebang extraction was too narrow for `/usr/bin/env bash` and stopped
after the first hook; it made no changes. The corrected loop above checked all
nine hooks successfully.

## Call flow and approval boundary

`just hooks-install` sets `core.hooksPath=.githooks` and chmods the primary
pre-commit, pre-push, and post-checkout hooks executable. The LF repair itself
does not alter that configuration or any hook mode.

The pre-commit wrapper delegates to `devbox run -- just pre-commit` when Devbox
is installed, else `just pre-commit`; the recipe delegates to `just check-fast`.
The chained pre-push wrapper first invokes `entire hooks git pre-push` if
`entire` is installed, then the pre-existing hook delegates to Devbox/Just and
`just pre-push` aliases the broad `just ci` gate. The Entire command is described
by the hook as pushing session logs. That is an existing external-write/publication
side effect that may occur before local CI finishes or rejects the push; this
review neither exercised nor authorizes it. A read-only `command -v entire`
check found no Entire executable on this host, so that conditional call would
currently be skipped here. Local guidance documents only `git config
--unset core.hooksPath` to disable the repository hooks as a whole; no
Entire-only opt-out was found. Disabling all hooks would also skip required
local checks and is not recommended or authorized by this review. The canonical pre-commit/CI
verification still belongs to the root operator when the compiler token is
free. This record is not permission to bypass it or to pass `--no-verify`.

Other observed existing effects: the post-commit chain may invoke Entire and
can atomically update `.ua/meta.json` for non-source commits when its opt-in
configuration and baseline graph are present; source commits emit an update
prompt. The Entire commit-msg / prepare-commit-msg / post-rewrite hooks are
conditional on `entire` being installed. The post-checkout hook only reports
dependency/toolchain drift and hooks-path configuration. These behaviors were
read, not executed. No actual hook execution, publication, commit, push, Git
configuration change, compiler invocation, or dependency operation occurred.

## Commands and review artifact identity

Read-only commands used: `git status --short`, `git diff -- .githooks`,
`git diff --cached --summary -- .githooks`, `git diff --cached --numstat --
.githooks`, `git ls-files --stage -- .githooks`, `git show :<hook>`,
`sha256sum`, `stat`, `perl`, `cmp`, `sh -n`, `bash -n`, `cat`, and `rg` over
the hook sources and `justfile`. No Cargo, Go, Bun, Node, Just recipe, or hook
execution command was run. A Graft ask produced no retrieval result, so source
review used the named hook and recipe files directly.

The source hashes above identify the reviewed working hook bytes. The artifact
itself is a new immutable review record; it does not include a self-referential
content hash.

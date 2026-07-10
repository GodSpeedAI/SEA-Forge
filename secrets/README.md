# SEA Forge secrets

This directory holds **encrypted only** API and MCP credentials for SEA Forge
development. Per `.agents/specs/Shell-SPEC.md` §8.2:

- Only `*.enc.env` files may exist here. Plaintext `.env` files are gitignored
  and MUST NOT be committed.
- Files are encrypted with SOPS + age. The public recipient lives in
  `../.sops.yaml`; the private key lives **outside** the repository, normally at
  `$SOPS_AGE_KEY_FILE` (`~/.config/sops/age/keys.txt`).
- `.envrc` decrypts the active profile (`$SEA_ENV`, default `dev`) when a key
  is present. With no key it emits a redacted warning and leaves secret
  variables unset so offline gates remain available (Shell-SPEC §8.2, §9).
- Secret-dependent recipes validate required `SEA_*` variable names before
  launch and never forward undeclared credential variables (§7.2).

## Profiles (§8.3)

- `dev` — shared or per-developer API/MCP credentials, encrypted for approved
  age recipients.
- `test` — no real credentials; tests use deterministic fakes.
- `ci` — credentials come from GitHub Actions secrets. A committed encrypted CI
  file is optional and MUST contain only test-scoped credentials.

## Commands

```sh
just secrets-init           # create a local age key if absent, print public key
just secrets-edit [profile] # edit via SOPS without leaving plaintext behind
just secrets-check [profile]# decrypt to a pipe, validate names, redact values
just secrets-rekey          # update encrypted files after recipient changes
```

No command prints a secret. If you lose your age private key it is not
recoverable from this repository; re-encrypt for new recipients with
`just secrets-rekey` (Shell-SPEC §13).

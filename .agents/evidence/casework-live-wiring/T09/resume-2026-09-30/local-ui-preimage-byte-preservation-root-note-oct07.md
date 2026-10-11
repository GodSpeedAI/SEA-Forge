# Exact local UI preimage preservation

Fresh repair builder reported its native-patch raw backups differed by one
terminal LF and held source edits. Root preserved actual UTF-8 file content in
immutable JSON containers through native patch before authorizing repair:

- local-adapter-828069-preimage-utf8-json-oct07.json: decoded38106 bytes,
  SHA828069b6c09d07c728778356c8921bfa553201f7882ffbbdfe83c86d96c21074.
- local-settlement-35f2a8-preimage-utf8-json-oct07.json: decoded7631 bytes,
  SHA35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819.

For each container, read-only Python verification compared its content.encode
to the actual source bytes and checked the declared SHA: both PASS before
repair release. These are lossless encoded source preimages, not runtime raw
captures; JSON container bytes do not have the source SHA.

Contrary to the builder's initial explanation, both actual originals END with
LF. Its raw attempts added another LF. Preserve those failed copies and record
the distinction; do not assume terminal-newline state from a failed copy.
No shell write, cp, or source modification was used by root. Normalization
repair may proceed only within the previously assigned private boundary.

# Independent root formatting verification

Date: 2026-10-08

Root ran stdout-only `gofmt` with exact identified stdin bytes; no source was
changed. Each joined invocation's five actual captures (command, preflight,
stdout, stderr, exit) were immediately archived through native patch and all
five decoded bytes compared equal to their original temporary files before
the next invocation. The three `*-root-format-verification-oct08.raw.json`
archives contain original paths and exact captures.

| Input | Verified output equals current source and builder stdout |
|---|---|
| Manager archived preformat source049c4ce9, 29,333 bytes | 22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57, 29,377 bytes |
| Worker current sourceb0fde3fe, 13,217 bytes | b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403, 13,217 bytes |
| Fixture original archived sourceea181a2f, 41,237 bytes | 889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b, 41,286 bytes |

All three formatter exits were zero. The fixture is thus exactly standard
formatting of its independently approved original source. Manager formatting
also regenerates exactly from its retained preformat source. Worker is
format-clean; its retained earlier preformat source5deebf4b differs from the
last formatter's declared input by removal of one redundant context check.
Root's preliminary worker input-identity assertion failed before invoking
its formatter; this was a provenance mismatch, not a formatter/compiler error.

Builder's missing original formatter paths and mismatched worker preformat
chronology remain historical limitations; these fresh captures do not recover
or authenticate lost originals. No compilation, test, algorithm approval,
lifecycle GREEN or public implementation approval is claimed.

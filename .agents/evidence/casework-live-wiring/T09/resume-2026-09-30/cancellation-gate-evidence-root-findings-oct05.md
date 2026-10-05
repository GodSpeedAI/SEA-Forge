# T09 Unit5A cancellation gate evidence — root findings

Date: 2026-10-05

## Decision

The source and fixture identities remain those in
`cancellation-runtime-source-readiness-oct05.md`: production `client.go`
SHA-256 `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`
and fixture `client_cancellation_test.go` SHA-256
`be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c`.
No source or test identity change is asserted here.

The prior four independent gate exits remain observed as zero, and their
captured output remains evidence that those commands reported success. However,
the canonical retry began while its original canonical invocation was still
running. That violates the assignment's single-compiler-token serialization
requirement and makes the aggregate gate approval procedurally insufficient.
Approval is **WITHHELD pending fresh serialized verification**. This finding
does not claim an OOM or source defect; no such event was observed in the
reviewed evidence.

## Serialization finding

The `/tmp` originals establish this ordering (UTC; filesystem timestamps are
shown in their local `-0400` representation):

| Artifact | `/tmp` timestamp | Meaning |
| --- | --- | --- |
| `rootcritic-canonical.raw` | 2026-10-05 12:58:55.149928091 -0400 | Original canonical command output capture |
| `rootcritic-canonical.exit` | 2026-10-05 12:58:55.159075145 -0400 | Original canonical command exit capture |
| `rootcritic-preflight-canonical-retry.raw` | 2026-10-05 12:58:05.457934562 -0400 | Retry preflight capture |

Thus the retry preflight preceded the original command's captured exit by
49.701140583 seconds. The retry command was admitted during that interval; the
original root raw/exit pair and retry preflight are preserved unchanged in the
repository. This is a sequencing violation under the global one-command rule,
regardless of whether the commands shared a Go cache. No OOM conclusion follows
from this timing evidence.

## Builder preflight-2 correction

The earlier statement that no `preflight-2` artifact exists is inaccurate.
`/tmp/sea-cancellation-oct05-oct05builder-preflight-2.raw` exists (mtime
2026-10-05 12:41:25.582048826 -0400), with paired exit file
`/tmp/sea-cancellation-oct05-oct05builder-preflight-2.exit` containing exit 0.
The raw file is a process-only `PID / COMMAND / RSS / %MEM` listing. It captures
process RSS rows, but has no host available/total RAM measurement, so it cannot
establish available host RAM at that time. The next `preflight-3.raw` exists at
12:41:38.474047099 -0400 and includes the previously reported host RAM and
process observations. No adjacent preflight or RAM value is inferred for the
intervening builder command(s); the original artifacts remain untouched.

## Byte validation of root archives

Compared each repository `rootcritic-*.raw` archive byte-for-byte with its
`/tmp/sea-cancellation-oct05/` original. The repository archive
`rootcritic-canonical-attempt1.raw` maps to `/tmp/.../rootcritic-canonical.raw`.
All 10 of 10 pairs matched exactly (SHA-256 shown):

| Repository archive | `/tmp` original | SHA-256 |
| --- | --- | --- |
| `rootcritic-canonical-attempt1.raw` | `rootcritic-canonical.raw` | `a17692a4718ec82a8f14e8af32cee3dcf85ef2acc122c3ac9756fd69f1560868` |
| `rootcritic-canonical-retry.raw` | `rootcritic-canonical-retry.raw` | `f4317d49473ae12a7d2caeab36a07bd9672d409642c53628ee881a0fa8a32502` |
| `rootcritic-focused.raw` | `rootcritic-focused.raw` | `2482a1a881343d9e3a87fc721f7a2ff6d14da4dde7edf857fece16df815e83ab` |
| `rootcritic-module-race.raw` | `rootcritic-module-race.raw` | `1982acb46d3cbbbab76e5cf9dc4c316c9a733f9112bdd15ab8de0832d2fed1a0` |
| `rootcritic-preflight-canonical-retry.raw` | `rootcritic-preflight-canonical-retry.raw` | `616582ba1f4b9a46d1a22fd150be0d3299797a5cacb2e12d61094b03d6e6d287` |
| `rootcritic-preflight-canonical.raw` | `rootcritic-preflight-canonical.raw` | `61a1bf47e1d35ecfd0e69966e06fccd343b9737542a6b03dd8830694bae5dfe5` |
| `rootcritic-preflight-focused.raw` | `rootcritic-preflight-focused.raw` | `6a3fd44df2850e8581fb6c8fb7285878ecce80eab74e04d9a808d94382a97a05` |
| `rootcritic-preflight-module-race.raw` | `rootcritic-preflight-module-race.raw` | `2baf0aedf4d91671ed87e15c8bd5520ea5019eed97d5334e7d5e0f597d13eee3` |
| `rootcritic-preflight-sfwp.raw` | `rootcritic-preflight-sfwp.raw` | `6e9778d3bf363638d39085b2f8b9e3befa3d8aa1f18136b2cc7bf2f588861e17` |
| `rootcritic-sfwp-race.raw` | `rootcritic-sfwp-race.raw` | `63d79f2421d658bdd26404406edde1d7ca543a2c51ad37ae392b86c0b90c9ba1` |

## Hand-transcribed `unit5a-oct05critic-*` copies

Compared each copy against the corresponding actual root `/tmp` capture; all
comparison output here is limited to paths, hashes, and first differing byte
offset. Existing copies are retained unchanged.

| Unit copy | Root `/tmp` original | Result |
| --- | --- | --- |
| `unit5a-oct05critic-focused.raw/.exit` | `rootcritic-focused.raw/.exit` | exact byte match |
| `unit5a-oct05critic-module-race.raw/.exit` | `rootcritic-module-race.raw/.exit` | exact byte match |
| `unit5a-oct05critic-preflight-focused.raw/.exit` | `rootcritic-preflight-focused.raw/.exit` | exact byte match |
| `unit5a-oct05critic-preflight-module-race.raw/.exit` | `rootcritic-preflight-module-race.raw/.exit` | exact byte match |
| `unit5a-oct05critic-preflight-canonical.raw/.exit` | `rootcritic-preflight-canonical.raw/.exit` | exact byte match |
| `unit5a-oct05critic-preflight-canonical-retry.raw/.exit` | `rootcritic-preflight-canonical-retry.raw/.exit` | exact byte match |
| `unit5a-oct05critic-preflight-sfwp-verified.raw` | `rootcritic-preflight-sfwp.raw` | exact byte match |
| `unit5a-oct05critic-preflight-sfwp.raw` | `rootcritic-preflight-sfwp.raw` | mismatch; unit SHA-256 `163131ef17162b2366af076b8294614f48d17d377a0caf9d4a729ea80bfb9f76`, root SHA-256 `6e9778d3bf363638d39085b2f8b9e3befa3d8aa1f18136b2cc7bf2f588861e17`, first mismatch offset 203 |
| `unit5a-oct05critic-sfwp-race-verified.raw` | `rootcritic-sfwp-race.raw` | exact byte match |
| `unit5a-oct05critic-sfwp-race.raw` | `rootcritic-sfwp-race.raw` | mismatch; unit SHA-256 `429c75cc2183d5397f121c3cb7008b93ab7e53dfc20fa03c2f8077f35ce9dc3c`, root SHA-256 `63d79f2421d658bdd26404406edde1d7ca543a2c51ad37ae392b86c0b90c9ba1`, first mismatch offset 4 |
| `unit5a-oct05critic-preflight-sfwp.exit` | `rootcritic-preflight-sfwp.exit` | exact byte match |
| `unit5a-oct05critic-sfwp-race.exit` | `rootcritic-sfwp-race.exit` | exact byte match |
| `unit5a-oct05critic-canonical.exit` | `rootcritic-canonical.exit` | exact byte match |
| `unit5a-oct05critic-canonical.raw` | `rootcritic-canonical.raw` | mismatch; unit SHA-256 `9e2b6e44dfb9687549a7a91384efc58b99bce34f87b7c1d1aff4241d788eef31`, root SHA-256 `a17692a4718ec82a8f14e8af32cee3dcf85ef2acc122c3ac9756fd69f1560868`, first mismatch offset 0 |
| `unit5a-oct05critic-canonical-first-attempt.raw` | `rootcritic-canonical.raw` | mismatch; unit SHA-256 `8349d80c5a092f39d7dd84d602256020da1540fc965581c1b49e81f385684c3b`, root SHA-256 `a17692a4718ec82a8f14e8af32cee3dcf85ef2acc122c3ac9756fd69f1560868`, first mismatch offset 235 |
| `unit5a-oct05critic-canonical-attempt1-verified.raw` | `rootcritic-canonical.raw` | mismatch; unit SHA-256 `15a1fb0683ad99680815f8665caccdfca4f0591301f1f1b103a80a62dac031a9`, root SHA-256 `a17692a4718ec82a8f14e8af32cee3dcf85ef2acc122c3ac9756fd69f1560868`, first mismatch offset 645 |
| `unit5a-oct05critic-canonical-attempt1-verified-v2.raw` | `rootcritic-canonical.raw` | mismatch; unit SHA-256 `a4117d388ac9324b74eab70e8d55ab90f559f7b357b4378ef5750036f913d429`, root SHA-256 `a17692a4718ec82a8f14e8af32cee3dcf85ef2acc122c3ac9756fd69f1560868`, first mismatch offset 457 |
| `unit5a-oct05critic-canonical-attempt1-verified.exit` | `rootcritic-canonical.exit` | mismatch; unit SHA-256 `c27a9f6f0d0f4a41f1fc82f1afdcbd65e13c0fe71fb9301b9d9c6ddb2d17fe3e`, root SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`, first mismatch offset 0 |
| `unit5a-oct05critic-canonical-retry-verified.raw` | `rootcritic-canonical-retry.raw` | mismatch; unit SHA-256 `8e55f7da7135a2820cc63bce17470af73e77b2d922c6bb9baaf9fb8411287449`, root SHA-256 `f4317d49473ae12a7d2caeab36a07bd9672d409642c53628ee881a0fa8a32502`, first mismatch offset 19 |

The independently named `rootcritic-*` originals, byte-validated above, are
the audit authority. The mismatching unit copies remain immutable historical
artifacts and are not used to support gate claims.

## Required next step

Rerun the required frozen cancellation matrix, full SFWP race, canonical
`just casework-go-check`, and full-module race with `-count=1 -parallel=1`,
strictly one Go compiler command at a time. Capture each actual host RAM/process
preflight immediately before the corresponding command, preserve all raw/exit
captures, and independently review the serialized outputs before restoring any
approval. This document records an evidence/procedure finding only; it does
not claim code repair or production approval.

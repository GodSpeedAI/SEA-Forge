# Unit 4 verification log archive transcription erratum

The initial native `apply_patch` archive attempt for the canonical and full-race stdout logs omitted one alignment space on the five Go output rows marked `?` (packages without test files). The original `/tmp` captures were not altered, and the initial archive artifacts were not overwritten or removed.

Initial nonidentical archive attempts retained at these paths:

- `unit4-fix-canonical.raw`, SHA-256 `78f07ca3f9baac668a6b42034e07abbbb5691bc9032b8807252bda4d61256fbf`
- `unit4-fix-fullrace.raw`, SHA-256 `f865bb46919882cfab1e57c08c06332166e421e1e20251f46b8171ea52ef95f5`

`diff -u` against the original files showed only that one-space difference on the five `?` status rows in each file; every other byte matched. New corrected copies were added without modifying those attempts:

- `/tmp/t09-unit4-fix-canonical.raw` SHA-256 `dedaeb7a98ed782df1301e5e946f39356afb0d808e3e0f7b87bf6900b1a0f0b8` matches `unit4-fix-canonical.exact.raw` SHA-256 `dedaeb7a98ed782df1301e5e946f39356afb0d808e3e0f7b87bf6900b1a0f0b8`.
- `/tmp/t09-unit4-fix-fullrace.raw` SHA-256 `88b92e8a01571e72029ebc8641da9f6915b1becb1d9fae3f3bc522d39a00f508` matches `unit4-fix-fullrace.exact.raw` SHA-256 `88b92e8a01571e72029ebc8641da9f6915b1becb1d9fae3f3bc522d39a00f508`.

The exact exit captures, all preflights, focused log, and live log already matched their original `/tmp` bytes. This correction is archive-only; it does not alter source, test execution, or pass/fail status.

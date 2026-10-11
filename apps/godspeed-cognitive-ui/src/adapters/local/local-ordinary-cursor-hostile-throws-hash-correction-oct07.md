# Hostile thrown-value repair hash correction

This new correction supersedes only the resulting source/test hashes in `local-ordinary-cursor-hostile-throws-repair-result-oct07.md`. The fixed fallback text was made generic because `asError` also normalizes failures from ordinary operations and timer scheduling, not only subscriber callbacks. No other source/test behavior or scope changed.

Correct frozen candidate identities:

- `localAdapter.ts`: SHA-256 `19c767cee3da406f5729a29177f360638d2b6ac3830294126f564c5d857b4700`.
- `localAdapter.hostileThrownValues.test.ts`: SHA-256 `31f8a71c2fa550ef0ea6f22259e25c48742a836fea85e52616d6da3356913920`.

Frozen inputs remain settlement supplement `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819`, cursor-bounds `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a`, cursor-order `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`, and conformance `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.

No tests, Bun, compiler, typecheck, formatter, Git, or gate command was run. This correction is a provenance note and not test or runtime evidence.

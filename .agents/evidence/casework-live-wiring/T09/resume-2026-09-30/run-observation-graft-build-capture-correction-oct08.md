# Graft capture archive correction

Date: 2026-10-08

The initial `run-observation-graft-build-preflight-oct08.raw.b64` has a
transcription defect and does not decode. It is preserved unchanged. The
corrected lossless preflight archive is
`run-observation-graft-build-preflight-correction-oct08.raw.b64`; it decodes
to `/tmp/sea-observation-graft-build-oct08-EL8icd/preflight.raw`.

The other five archive entries in `run-observation-graft-build-result-oct08.md`
decoded and compared byte-for-byte to their `/tmp` originals. The corrected
preflight entry also decodes and compares byte-for-byte. This correction does
not change the captured source, command, Graft result, or exit status. Exact
source identities and preflight values are in the original `/tmp` capture.

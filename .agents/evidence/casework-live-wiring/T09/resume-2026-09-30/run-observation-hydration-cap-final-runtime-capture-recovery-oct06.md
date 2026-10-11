# Focused hydration cap capture recovery note

Date: 2026-10-06

This note supplements the final runtime review and its capture erratum. It preserves the capture correction history without changing the final byte-exact archive.

The first archive patch attempt was rejected before writing files. The next patch succeeded and created the run archive with the 1024-frame rejection subtest duration transcribed as `0.00s`. `cmp` against the `/tmp` original found the discrepancy; the same archive was then corrected to `0.01s`. I did not save a copy before that correction. The successful patch request is retained in the tool transcript, so the new file `run-observation-hydration-cap-green-01-run-recovery-initial-written-transcription.raw` reconstructs the earlier successfully written payload from that request. It is a reconstruction, not a contemporaneous preserved file and not claimed byte-compared to the lost prior on-disk version. Its difference from the final archive is the single duration value. The final archive remains unchanged and its `cmp` against the `/tmp` original passed; final hashes are in `run-observation-hydration-cap-final-runtime-review-oct06.md`.

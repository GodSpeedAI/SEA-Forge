# Hydration cap runtime capture clarification

Date: 2026-10-06

This record supplements, without replacing, `run-observation-hydration-cap-final-runtime-review-oct06.md`.

The first native `apply_patch` attempt for the focused gate's run and exit archives was rejected by automatic review before any file was created. No incorrect archive copy exists. The next patch included the omitted `more_than_1024_frames_in_one_run` subtest, but its duration was transcribed as `0.00s`. `cmp` against the captured `/tmp` transcript identified that mismatch. I corrected the duration to the captured `0.01s` and reran `cmp`; the final archived run and exit match their `/tmp` originals byte-for-byte. No source or fixture was modified.

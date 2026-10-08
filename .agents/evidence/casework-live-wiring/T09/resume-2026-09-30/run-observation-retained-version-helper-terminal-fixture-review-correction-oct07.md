# Correction: terminal fixture repair values and matching

Date: 2026-10-07. This correction is separate and immutable; earlier review
artifacts remain unchanged.

The retained-helper atomicity review recommends valid candidate values
`failed`, `accepted`, and `timed_out`, while saying all three are absent from
the prior image. That absence claim is too broad: the retained image has the
JSON field name `accepted_at`, so a raw substring search for `accepted` would
match even though it is not the candidate settlement value. Prefer valid
candidate execution `failed`, settlement `rejected`, and frame status
`timed_out`, which are distinct from the prior state and its frame values.

Alternatively, retain `completed` / `accepted` as the source values and change
the assertion to decode the JSON image and compare the exact top-level
`execution` and `settlement` fields; do not use raw substring matching for
values shared with prior frame metadata or JSON field names. The existing full
state equality check already proves the terminal state keeps prior metadata
except for availability, and the image equality check proves the serialized
image is the expected marker-only state. Keep raw absence checks only for
unique candidate strings that cannot appear in prior values.

This correction does not change the REJECT disposition: the current terminal
fixture still supplies invalid trace metadata/status and cannot demonstrate a
valid terminal candidate. It replaces only the suggested repair/matching
detail. No source or test was run.

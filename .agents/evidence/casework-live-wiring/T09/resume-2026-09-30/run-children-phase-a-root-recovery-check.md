# Root partial archive recovery check

Root verified the recovered sandbox incorrect archive bytes reproduce exactly the two
documented historical transformations and original bad SHA6cff61f2678524a9e5671a1ac254332ee97e210c655ae20b0be2f059c1b476fb.
Both actual sandbox/host raw logs still byte-match their unchanged /tmp originals and
current full hashes. The recovered incorrect copy is not execution evidence.

Host incorrect archive version remains unrecovered; recorded old SHA697abd30fc2344adb383a4e4eb230fc3851ea7d8f762b82fb9b278996015e002
did not match candidates derived from the critic's reported partial differences. Do not
claim exact recovery or sufficient whole-file history for that copy. Original actual host
output remains present and exact, so the scoped assertion RED evidence itself is intact.

Clarification to initial-archive-recovery-erratum: the critic replaced incorrect copies
with correct actual stdout copies; it did not replace actual stdout with incorrect data.
During recovery, the builder also removed a newly transcribed incorrect recovery artifact
before creating the final hash-matched copy. Its intermediate bytes were not retained.
That is an additional evidence-preservation limitation, not another test execution. Root
prohibits overwriting/deleting persistent evidence artifacts even before approval; future
transcription failures stay at their original paths with new correction files/errata.

Current original execution outputs and source identities remain independently verified.
These limitations stay disclosed in final unit/T09 confirmation; immutable-history
compliance is not claimed retroactively.

# Hydration checkpoint/push provenance — root correction

Date: 2026-10-06. Preserve original provenance note1c9c93eb unchanged.
Root independently compared all five archives with the actual /tmp originals:
all cmp exit0. Actual archives reorder basename components to `...-run-oct06.raw`
from originals `...-oct06-run.raw`; the original task requested matching names.
This naming deviation is accepted with the explicit mapping in the original
note; it does not change bytes. Root's first attempted identical-basename
lookups failed with no-such-file exit2, not a byte difference. The subsequent
mapped comparisons are the accepted evidence.

The original note incorrectly calls10067 an exact remote value. It is the
read-only ls-remote TOOL SESSION ID, joined exit0. The actual observed remote
SHA is `41735743c3c4571e44e31031e3b4cf08ca8f9358` for
refs/heads/casework/live-wiring-resume-2026-10-05 on the approved origin.
Likewise37401 is the push session, joined exit0, not a zero join count.
The checkpoint session78326 joined exit0. Root previously derived125 suite
summaries/1110passed/0failed/4ignored and511commits/no leaks from the actual push
capture; final CI reports all gates green. These are normal-hook results for
the committed checkpoint, not current uncommitted guard/UI fixture approval.

No checkpoint preflight original exists or is fabricated. Originals and all
written versions remain preserved. This correction clarifies provenance only;
no source algorithm, manager/public wiring or T09 completion is approved.

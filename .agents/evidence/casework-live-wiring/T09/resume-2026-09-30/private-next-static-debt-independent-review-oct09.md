# Independent review: private Next static-debt ledger entry

## Verdict

**REJECT the documentation as written; repair CW-41 and request a fresh review.**
The entry accurately summarizes the historical static source rejection, and
the builder's recorded `.agents/DEBT.md` SHA-256 matches the current file.
However, CW-41's forward instruction does not incorporate the subsequently
issued initial-unavailable fixture correction, and its compile-defect count is
ambiguous.

## Evidence reviewed

Read the full `.agents/DEBT.md`, root and `.agents/` instructions, current
status, governing casework spec, the original private Next assignment and
sequencing/projection-failure addenda, the frozen TDD builder result, full
independent source rejection, the static-debt builder record, the initial
unavailable fixture correction, and its pending-review context. The original
source review identifies four missing error returns at test lines 314, 558,
600, and 654, plus six invalid boolean uses of `map[*runObservationLease]struct{}`
at lines 519, 629, 643, 696, 714, and 751. The builder record's claimed ledger
identity is confirmed: `7c05f1df9be7ed7a322f4aef7629411248cad9baaf165372e94da6481f045797`.

## Findings

1. **CW-41 does not state that the initial-recovery request was superseded.**
   The source rejection's finding 4 is an accurate historical report: that
   fixture was absent from the rejected source. The later correction explains
   that an initial read failure releases the selected entry and stops its
   worker, so the requested same-lease later recovery setup is unreachable.
   It directs the fresh fixture to preserve the existing cleanup behavior and
   prove absence of retained state, without inventing recovery. CW-41's
   unqualified “fresh fixture-only repair ... against the full assignment” and
   “preserve ... approved scope” leave the superseded request looking binding.
   Keep finding 4 as historical evidence, but add the correction's disposition
   and its bounded replacement proof to CW-41's next step. Preserve actual
   read/retention-marker recovery only for accepted attachments with a retained
   current value. Do not require the impossible initial recovery test.

2. **The static compile-defect count is unclear.** CW-41 says “four static
   compile defects” and then lists four missing returns and six map-value
   boolean checks. The source review establishes two defect classes with ten
   affected sites, not four total defects. Say “two static compile-defect
   classes: four callbacks ... and six map checks ...” so the recorded count
   cannot be read as contradicting the evidence.

## Limits

This is a documentation-only review. No compiler, test, gate, runtime, Git
command, or source edit was performed. The current ledger hash agrees with the
builder record; because this assignment prohibited Git commands, I did not
independently reconstruct the base-to-current Git diff from `9efd039`. The
builder record says CW-41 was appended and earlier entries were preserved;
that exact base diff remains unverified by this review. No Next readiness,
behavioral RED, implementation, or T09 settlement is claimed.

## Required repair

Have a fresh documentation builder make only the CW-41 wording correction,
record the initial-unavailable correction's bounded disposition, preserve
CW-41's historical rejection and all earlier debt entries, then freeze a new
ledger hash and request independent review. Do not run compilation or expected
RED under this rejected documentation packet.

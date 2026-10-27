# Current status

**Status revision:** 195

**Stage:** capability-harvest: round-10 control exposed the breakaway probe itself (error 123 for BOTH creates); explicit ApplicationName fixes it; pushing round 11

**Summary:** Round 10's positive control did its job: the no-flag create also failed ERROR_INVALID_NAME (123), proving the earlier breakaway 'refusal' was never attributable to the job — the probe's null ApplicationName made CreateProcessW treat the quoted command string as the executable path. Both probes now pass C:\\Windows\\System32\\cmd.exe explicitly with arguments in the command line. The server-side lifecycle panic in the same log line is a caught-and-reported test artifact (the test reports ok).

**Verified:**

- windows-target check + fmt green on the probe fix

**Limits:**

- Round 11 must confirm the windows lane end-to-end green

**Next move:** Push round 11; on green, freeze and run the codex adversarial review.

**Evidence:**

- gh run 40446093616 log

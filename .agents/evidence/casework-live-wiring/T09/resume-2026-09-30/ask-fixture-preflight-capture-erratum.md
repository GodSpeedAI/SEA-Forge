# Ask fixture preflight capture erratum

Date: 2026-10-01

The independent review `ask-fixture-independent-review.md` reported actual-host RAM/process
preflight results before each Go command. Those preflight command outputs were not saved as
separate files. Their measurements and the absence of another compiler/test process are reported
from the observed command output, not independently replayable artifacts. Per root direction,
the expected-RED tests are not rerun solely to recreate preflight logs.

The original full command captures remain at:

```text
/tmp/t09-ask-critic-server-expected-red-escalated.log
/tmp/t09-ask-critic-server-expected-red-escalated.exit
/tmp/t09-ask-critic-transport-expected-red.log
/tmp/t09-ask-critic-transport-expected-red.exit
/tmp/t09-ask-critic-server-expected-red.log
/tmp/t09-ask-critic-server-expected-red.exit
```

The first two log files are the escalated expected-RED outputs and match the immutable evidence
copies byte-for-byte. The final pair records the non-escalated sandbox socket failure; it is not
expected-RED evidence. The `.exit` files contain the actual shell exit status.

# Root capture supplement — setup and fixture race failures

Attempt2 session44968 joined1: expected stub/hook failures were mixed with sandbox
Unix listener setsockopt failures. This was not acceptable assertion RED. Attempt3
used the explicitly authorized local-fixture escalation and completed exit1: binds
worked, but the shared fake-server calls counter produced a race. No acceptable
RED or production proof exists from either attempt.

Root natively copied original attempt2 and attempt3 raw, exit, preflight and gofmt
exit captures into this directory as physical-admission-attemptN-*-oct06.txt.
All eight nonempty copies passed cmp against actual CLI originals. Both original
gofmt stdout captures were empty; no byte-exact project copy is claimed for them.
These supplements preserve every failure and do not replace earlier reviews.

Fresh builder admission_retry_counter_fresh_builder changed ONLY the callback
counter to atomic.Int32/Add and added its standard-library import. Root checked the
unchanged four-response branches and exact hashes: client fixture becd4266,
limiter fixture dae0406d, stub f661bd8f, client d2f993d7. The builder's initial
message appended an erroneous extra character to its quoted baseline hash; root's
actual baseline and repaired hashes, not that typo, are the identity evidence.

Independent source rereview is assigned. If approved, the same independent critic
alone may run one new escalated local-fixture attempt4 with unchanged functional
gate/caps/regex plus diagnostic verbosity. No source implementation is released.
Initial acquisition failures in the unavailable stub cannot prove the later
no-spin or pool-observation assertions; full behavior is a later production gate.

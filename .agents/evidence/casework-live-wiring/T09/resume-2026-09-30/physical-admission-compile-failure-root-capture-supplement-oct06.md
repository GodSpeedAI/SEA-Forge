# Root capture supplement — compile failure, not assertion RED

Independent focused session65279 joined exit1. Root read the actual failure and
cleanup method: it references t without a parameter or receiver field. No focused
assertion ran. Source review approval did not establish compile correctness.

The critic's first archive locations were temporary. Root now copied the ORIGINAL
raw, exit and preflight into native immutable project evidence and verified all
three with cmp exit0:

- physical-admission-compile-failure-raw-oct06.txt
- physical-admission-compile-failure-exit-oct06.txt
- physical-admission-compile-failure-preflight-oct06.txt

Originals are respectively the actual CLI capture
/tmp/physical-admission-assertion-red-20261006.raw and .exit, and
/tmp/physical-admission-assertion-red-preflight-20261006.txt. These new captures do
not replace any prior record. Actual exit contents are 1, not a passing result.

Compiler ownership is free after the actual join. A DIFFERENT micro-repair builder
owns only the limiter fixture's cleanup parameter/callback registration. All other
source/assertions remain frozen. Independent review and a new focused attempt are
required; no production implementation is released.

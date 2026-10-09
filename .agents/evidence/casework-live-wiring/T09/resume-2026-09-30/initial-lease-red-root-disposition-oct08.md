# Initial lease bookkeeping: root RED disposition

Independent source review `run-observation-initial-lease-bookkeeping-independent-source-review-oct08.md` approved only source readiness. Root then ran the original three-test focused command with one compiler owner and actual memory/swap preflight.

Attempt 01 failed before compilation: normal Go build cache was read-only in the sandbox. Attempt 02 obtained normal cache access but its preflight found an unrelated Gauntlet Cargo/rustc build; exit 99 means the test was not executed. Neither attempt establishes behavioral RED. No foreign build was interrupted or changed.

Attempt 03 ran after that compiler finished. It compiled and executed all three tests, with actual child exit 1. Complete-empty and accepted hydrated leases failed because listState was empty instead of complete; refused-list failed because listState was empty instead of unavailable. The hydration test reached that assertion only after its actual eight accepted captures, exact read-count and aggregate pruning assertions passed. Later watermark/map-separation assertions remain unexercised until initialization is implemented; this RED does not claim they failed independently.

All six actual files from each attempt were archived losslessly and root decoded/compared before the next gate. Original captures remain in `/tmp/initial-lease-red01-tvqd03_x`, `/tmp/initial-lease-red02-vm7zbxmu`, and `/tmp/initial-lease-red03-26e1v_tb`; durable archives use the corresponding `initial-lease-red01`, `02`, `03` prefixes here. Archive count/checksum/Base64 were machine-derived. No wrapper success substitutes for the child's status.

Root accepts attempt 03 as expected behavioral RED for the bounded assignment. A fresh Luna builder is released only for initial private map/list outcome/scalar seeding; tests remain frozen. No Next, notifier, drain redesign, public interface or T09 settlement is approved. The new status migration gate also passed: `resume-status01` six-file archives were root compared against actual captures.

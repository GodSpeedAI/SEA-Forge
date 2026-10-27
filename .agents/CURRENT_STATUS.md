# Current status

**Status revision:** 208

**Stage:** ARM64 packaging: root cause finally named by the gate's own error — leading spaces in the readelf Machine capture; both gates trim properly; rerunning

**Summary:** Run 5's error message showed the captured machine as '                          AArch64' (readelf right-aligns the field). The workflow gate's gsub trimmed the wrong expression stage; both gates now trim the captured value, verified locally against a real ELF.

**Verified:**

- local extraction: [Advanced Micro Devices X86-64] exact-match verified

**Limits:**

- Rerun pending

**Next move:** Green run -> artifacts + hashes -> Jetson instructions.

**Evidence:**

- run 40600886499 error output

# Current status

**Status revision:** 206

**Stage:** ARM64 packaging: machine gate moved to readelf's stable Machine field; rerunning

**Summary:** The corrected-run failure reproduced identically on BOTH architectures 60ms after the first binary's interpreter check: file's machine spelling differs on the runner's file version, so the -F match failed with no output. The gate now reads readelf -h's Machine field (stable output; leading whitespace trimmed), which is also what the error message reports.

**Verified:**

- readelf extraction verified locally against a real x86-64 binary (Advanced Micro Devices X86-64)

**Limits:**

- Rerun pending

**Next move:** Green packaging run -> artifacts + hashes -> Jetson instructions.

**Evidence:**

- run 40596015564 logs

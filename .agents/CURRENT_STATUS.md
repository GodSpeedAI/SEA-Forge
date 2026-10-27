# Current status

**Status revision:** 207

**Stage:** ARM64 packaging: the workflow's own file-based machine gate had the same version brittleness; switched to readelf; rerunning

**Summary:** Run 4 reproduced the identical signature because the workflow's detach step carries a SECOND machine check (my matrix edit added file -b | grep -F) alongside the build.sh one I had already fixed. Both gates now read readelf -h's stable Machine field. The build itself (native arm64 compile, interpreter patch, interpreter read-back) has passed on every run; only these duplicate verification gates failed.

**Verified:**

- yaml parse ok

**Limits:**

- Rerun pending

**Next move:** Green run -> artifacts + hashes -> Jetson instructions.

**Evidence:**

- runs 40592500846 / 40596015564 / 40598734359 (same signature)

# Current status

**Status revision:** 213

**Stage:** ARM64: arm64 leg GREEN (run 8) — .deb + tarball built, install + smoke tests passed; amd64 unified on the native toolchain for the GLIBC floor; rerunning

**Summary:** Run 8: package (linux arm64) SUCCESS — the arm64 .deb installs, --version checks pass, helpers are present, the smoke cell runs, and the tarball is built on the ubuntu-22.04-arm runner against glibc 2.35. The amd64 leg's own binaries carry a (previously tolerated) weak GLIBC_2.39 reference from the Devbox/Nix toolchain, which the new floor check refuses — correct behavior for an honest dependency floor. Both linux legs now build with the native rustup toolchain + system gcc (Devbox remains for release-check), making the whole linux family bind the deployment glibc.

**Verified:**

- gh run 40610767133: arm64 SUCCESS, windows SUCCESS, macos in flight

**Limits:**

- amd64 rerun pending; kernel 22.04 rerun pending (env-leak fix already pushed)

**Next move:** Green runs -> artifacts + hashes + Jetson install instructions.

**Evidence:**

- gh run 40610767133

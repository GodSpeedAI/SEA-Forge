# Current status

**Status revision:** 202

**Stage:** ARM64 Jetson packaging: branch created from main c81f6307; runner probe dispatched

**Summary:** New workstream: extend native packaging to Linux ARM64 (aarch64) for the Jetson AGX Orin (Ubuntu 22.04, glibc 2.35). Branch feat/arm64-jetson-packaging carries an arm64 hosted-runner probe (ubuntu-22.04-arm, ubuntu-24.04-arm matrices) to verify availability before the packaging job is written against a specific runner.

**Verified:**

- Existing packaging architecture mapped: package.yml linux job (ubuntu-22.04, patchelf x86-64 loader detach, dpkg-shlibdeps deb, amd64 tarball, install+smoke tests, built-from-linux.txt), release-please publisher (pkg-* merge, built-from checks, SHA256SUMS, provenance), deb build.sh amd64 hardcoding.

**Limits:**

- Runner availability unknown until the probe runs

**Next move:** Probe verdict -> implement build.sh arch param + workflow job + loader handling + publisher contract.

**Evidence:**

- .github/workflows/arm-probe.yml

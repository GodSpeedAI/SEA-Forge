# OpenShell Live Proof (scripts/openshell)

## Overview
Hosts `live-proof.sh`, the only place in the repository that starts a real
OpenShell gateway and sandboxes for a live proof: a Docker-free OpenShell
gateway (`compute_driver = "vm"`, libkrun MicroVMs, no container engine)
running the `sea-forge-case-runner` live tests against the RepairAgent
projection, including a non-bootstrap image. Nothing else in the repository
should start an OpenShell gateway for this purpose.

## Interface
- `live-proof.sh` — Bash script; no positional arguments; behavior is driven
  by optional environment variables `PROJECTION_DIR` (default
  `fixtures/cells/repair-agent/openshell`), `GATEWAY_NAME` (default
  `godspeed-live`), `PORT` (default `18081`), `STATE_DIR`, `IMAGE`,
  `SECOND_IMAGE` (default `docker.io/library/python:3.13-slim`), and
  `ALLOW_CONTAINER_SOCKET=1` ([live-proof.sh:environment variable block]).
- Exit codes: `0` live tests ran and passed; `1` a check or live test failed;
  `3` BLOCKED with a prerequisite missing (reason on stderr as
  `BLOCKED: <reason>`) ([live-proof.sh:exit code header and blocked()]).
- Preflight: `openshell`, `openshell-prover`, `openshell-gateway` must report
  version `0.1.3` exactly; `openshell-driver-vm` must be found in one of the
  probed libexec locations and also report `0.1.3`; on Linux `/dev/kvm` must
  be readable (with a WSL2 `nestedVirtualization` hint); host tools `ssh`,
  `git`, `python3`, `mke2fs`, `debugfs`, `e2fsck` must exist
  ([live-proof.sh:preflight section]).
- Gateway configuration is generated from the projection's
  `sandbox.json` `vm_shape` by an embedded `python3 -I` heredoc writing
  `gateway.toml` ([live-proof.sh:python heredoc]), then validated with
  `openshell-gateway config preflight` ([live-proof.sh:config preflight]).
- The live test target is
  `crates/sea-forge-case-runner/tests/cell_episode_live.rs`, executed as
  `cargo test -p sea-forge-case-runner --test cell_episode_live` with
  `SEA_FORGE_OPENSHELL_LIVE=1`, `SEA_FORGE_OPENSHELL_GATEWAY`,
  `SEA_FORGE_OPENSHELL_PROJECTION`, `SEA_FORGE_OPENSHELL_IMAGE`, and
  `SEA_FORGE_OPENSHELL_SECOND_IMAGE` exported
  ([live-proof.sh:cargo test invocation]).

## Invariants
- The script never reports success unless the live tests ran and passed;
  only cargo's exit status decides success ([live-proof.sh:header and
  cargo test section]).
- Docker-free claim: the run fails if `DOCKER_HOST` is set or a
  Docker/Podman socket exists, unless `ALLOW_CONTAINER_SOCKET=1`, in which
  case the run explicitly states the Docker-free claim is NOT established
  ([live-proof.sh:Docker-free assertion]).
- OpenShell version is pinned to `0.1.3` for every component
  ([live-proof.sh:EXPECTED_VERSION checks]).
- All credentials and CLI configuration are generated under `STATE_DIR` with
  `XDG_CONFIG_HOME` pointed into it, so the operator's own OpenShell
  configuration is never read or written ([live-proof.sh:gateway
  configuration comment]).
- A hung or failed gateway startup dumps the last gateway log lines and
  fails rather than proceeding ([live-proof.sh:readiness loop]).

## Side Effects
- Generates an ephemeral CA plus server/client certificates via
  `openshell-gateway generate-certs` and Ed25519 sandbox launch-signing keys
  via `openssl` under `STATE_DIR` ([live-proof.sh:PKI and JWT generation]).
- Starts a real `openshell-gateway` process bound to
  `https://127.0.0.1:$PORT` with a SQLite database under `STATE_DIR`, and
  registers it for the CLI as an mTLS gateway
  ([live-proof.sh:gateway start and registration]).
- Runs `cargo test` cases that launch libkrun MicroVM sandboxes pulling the
  configured images ([live-proof.sh:cargo test invocation]).
- Kills and waits on the gateway process on exit via an EXIT trap
  ([live-proof.sh:cleanup]).

## Verification
- Run: `bash -n scripts/openshell/live-proof.sh` (syntax). The substantive
  gate is the script itself, which drives
  `cargo test -p sea-forge-case-runner --test cell_episode_live` on a host
  with OpenShell 0.1.3 and KVM available.

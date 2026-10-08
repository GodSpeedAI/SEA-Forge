# Mise configuration-tracking warning recon — 2026-10-07

## Entire root instruction

> Cheap bounded SOURCEONLY harness recon (no edits/tests/compiles/Git/scanner/network). Isolatedcanonicalactualoutput ...run-observation-primitives-canonical-output-oct07.raw shows nonfatal mise WARN trackingconfig symlinkhostregistry read-only. Determine via SAFE LOCAL mise --help/settings help or binary public docs whether supported env/setting disables OPTIONALconfigurationtracking for isolatedworktree commands, preservingpinnedtoolresolution/gates. Neverread/dumpprivateconfigs/env/secrets; no hostFSwrites or globalsettingsmutation. Returnmechanicalsupportedoption+command evidence orrecordunknown, notinventsetting. Any applyingoptionrootdecision futuregrant; warningdidnotcauseformatfailure. No need artifactexpansion beyondNEWbriefimmutablereconrecordoriginalinstructions+findings. Renderer sourceonly, formatbuilderSOLEheavyformatter; you no compilers.

## Evidence read

The immutable output `run-observation-primitives-canonical-output-oct07.raw` begins with a mise warning that it could not create a symlink for the isolated worktree's `mise.toml` under the host tracked-config state path because that path is read-only. It then reports `casework-go-check: gofmt would rewrite: internal/server/run_observation_retained_policy_test.go` and recipe exit 1. Thus this warning did not cause the recorded format failure.

Safe local help checked: `mise --help`, `mise settings --help`, and `mise run --help`. No config file, current setting value, environment dump, credential, or secret was inspected; no setting was changed and no network/host-state write was attempted.

## Finding

The displayed top-level options include `--no-config` / `MISE_NO_CONFIG=1`, but the help says this prevents loading any config files. That can change pinned tool resolution and task configuration, so it is not an acceptable equivalent for merely disabling optional config tracking while preserving gates. `--quiet` and `--silent` suppress output, not documented tracking behavior; they do not establish a no-write or tracking-disable guarantee. The settings help describes reading/managing settings but does not name a config-tracking setting or environment variable. No supported option specifically disabling optional tracking while retaining config/tool/task resolution was identified from the allowed local help.

Conclusion: record the exact setting/flag as unknown; do not invent or apply one. The warning is nonfatal in the archived output. Any later workaround requires root's separate decision and must preserve pinned tool resolution and task behavior. No tests, compile, scanner, Git, network, or config mutation was performed.

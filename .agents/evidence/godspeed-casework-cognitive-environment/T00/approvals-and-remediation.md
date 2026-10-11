# T00 — operator approvals and their verification (B1 Go toolchain, B2 gitleaks)

Both prerequisites recorded by T00 were put to the operator and approved on 2026-09-19:
"Install Go (mise install go@1.27.1) AND apply the narrow .gitleaks.toml allowlist — then continue
with T02 and T01." This file records what was done and how it was verified. Nothing else about T00's
claim changed.

## B1 — Go toolchain: RESOLVED

```
$ mise install go@1.27.1
mise go@1.27.1  [3/3] go version go1.27.1 linux/amd64
mise go@1.27.1  ✓ installed
$ go version
go version go1.27.1 linux/amd64
go1.27.1
GOPATH=/home/sprime01/go    GOMODCACHE=/home/sprime01/go/pkg/mod
```

Install location: `~/.local/share/mise/installs/go/1.27.1`; the `go` shim at
`~/.local/share/mise/shims/go` (mise's shim directory is already on `PATH`).

**Deliberate repository change (disclosed):** `mise.toml` gains `go = "1.27.1"` with a comment.
This is a config/toolchain declaration, not an application dependency: mise is how this repository
already declares its toolchain (`npm:@jolli.ai/cli`, `npm:agent-browser`, `npm:command-code` are
declared the same way), and without the declaration a fresh worktree would not auto-install Go, so
`just casework-go-check` would fail for a machine reason rather than a code reason. No other
`mise.toml` line changed.

T01 now owns `just casework-go-check` (format, vet, test) as its gate; this entry only records that
the toolchain exists and resolves.

## B2 — `just security` red baseline: RESOLVED

**Approval scope honoured:** the allowlist is commit-scoped, path-scoped and rule-scoped, with
`condition = "AND"`, so it requires the commit AND the path AND the rule to all match. It follows
the repository's own T17 precedent (a commit-scoped allowlist added during an earlier gate
remediation) rather than inventing a policy.

**Before** (`raw-logs/gate-security.log`, `raw-logs/gitleaks.json`): `cargo deny` green; gitleaks
`leaks found: 14`, exit 1. Six files, all in commit `6ce518fcd9`, all rule `generic-api-key`.

**After** (`raw-logs/security-after.log`, `raw-logs/gitleaks-after.json`):

```
$ just security
advisories ok, bans ok, licenses ok, sources ok
6:14PM INF 492 commits scanned.
6:14PM INF scanned ~215553271 bytes (215.55 MB) in 49.8s
6:14PM INF no leaks found
just security exit=0  wall_s=52
```

`raw-logs/gitleaks.toml.before` and `raw-logs/gitleaks.toml.after` are the exact file before and
after (the diff is 29 added lines: one comment block and one `[[allowlists]]` entry).

**Falsification tooth (the allowlist must not be a global rule disable):** the same bytes were
copied out of the repository and scanned with `--no-git`, where neither the commit nor the path can
match:

```
$ gitleaks detect --no-git --source /tmp/gdsp/tooth --report-format json ...
findings when the same bytes are scanned outside the allowed commit+path: 4
```

So the rule still fires; what suppresses the 14 findings is exactly the commit+path+rule conjunction.
Any new occurrence in those paths, or any occurrence anywhere else, still fails the gate.

**What was NOT claimed:** the values were never read unredacted. The classification rests on
non-exposing measurement (field name, length, charset, digit/hyphen presence, entropy) recorded in
`gate-baseline-seafoerge.md` and reproducible with `teeth/remeasure-gitleaks.sh`.

## Effect on the plan

- B2 is resolved: `GATE_SEAFORGE` activates at T05 and previously could not pass because of the red
  security step; that step now exits 0 (52 s, no leaks found — see `raw-logs/security-after.log`).
  `just check` and `just ci` were red *only* at that step, so they should be re-run before T05
  rather than assumed green.
- B1 no longer blocks T01/T04. T01 is next.
- Resource note: at the time of these runs `MemAvailable` had fallen to ~2.4 GiB and swap free to
  ~2.2 GiB (from 3.2 GiB / 7.1 GiB when the T00 baselines ran). Preflight before every heavy gate
  and defer when headroom is inadequate, per the plan's `build_resource_policy`.

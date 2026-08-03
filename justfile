# SEA Forge canonical command surface (Shell-SPEC §10.2).
# Run from the repository root. CI invokes `devbox run -- just <recipe>`.

set := "set -euo pipefail"
evidence_dir := "target/bootstrap-evidence"

# Default: print grouped recipes.
[group('meta')]
default:
    @just --list --list-heading "SEA Forge commands:" --justfile justfile

# --- setup --------------------------------------------------------------

# Converge on the pinned toolchain: Devbox packages + Rust components + deps.
[group('setup')]
setup:
    #!/usr/bin/env bash
    {{set}}
    echo "[setup] installing devbox packages"
    devbox install
    echo "[setup] ensuring rust components"
    rustup toolchain install "$(sed -n 's/^channel *= *"\(.*\)".*/\1/p' rust-toolchain.toml)" --profile minimal --component rustfmt,clippy --no-self-update
    echo "[setup] fetching workspace dependencies"
    cargo fetch --locked
    echo "[setup] done — run 'just doctor' to verify"

# --- quality ------------------------------------------------------------

# Machine-readable environment check (Shell-SPEC §7.1). Writes doctor.jsonl.
[group('quality')]
doctor:
    #!/usr/bin/env bash
    {{set}}
    bash scripts/doctor.sh

# Build all crates.
[group('quality')]
build:
    cargo build --workspace --all-targets --locked

# Apply rustfmt to every crate.
[group('quality')]
fmt:
    cargo fmt --all

# Verify rustfmt without modifying files.
[group('quality')]
fmt-check:
    cargo fmt --all -- --check

# Run clippy with `-D warnings` across all targets and features.
[group('quality')]
lint:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

# Type-check every crate (`cargo check --locked`).
[group('quality')]
typecheck:
    cargo check --workspace --all-targets --locked

# Type-check one workspace crate. Example: `just crate-check sea-forge-core`.
[group('quality')]
crate-check crate:
    cargo check -p "{{crate}}" --locked

# Supply-chain + secret scan: cargo-deny then gitleaks.
# `cargo deny check advisories` fetches the RustSec database; the other
# categories are offline. gitleaks scans staged + committed history.
[group('quality')]
security:
    #!/usr/bin/env bash
    {{set}}
    cargo deny check
    gitleaks detect --no-banner --redact

# Apply only safe automatic fixes (rustfmt). Clippy fixes are intentionally
# excluded: `-A clippy::all --fix` mutates behavior and should be reviewed.
[group('quality')]
fix:
    cargo fmt --all

# Quick high-signal local verification (context + fmt + typecheck).
# Target runtime well under 10s — this is what pre-commit runs.
[group('quality')]
check-fast:
    #!/usr/bin/env bash
    {{set}}
    scripts/check-agent-context.sh
    cargo fmt --all -- --check
    cargo check --workspace --all-targets --locked

# Developer quality sweep (Shell-SPEC §10.3): context + fmt + clippy + check
# + deny + gitleaks. Delegates to granular recipes so local and CI share one
# implementation per category.
[group('quality')]
check:
    #!/usr/bin/env bash
    {{set}}
    just context-check
    just fmt-check
    just lint
    just typecheck
    just security
    echo "[check] all gates green"

# Validate agent handoff structure and freshness without vendor-specific tooling.
[group('quality')]
context-check:
    scripts/check-agent-context.sh

# Verify the handoff a cold independent Workbench evaluator needs: concrete
# commands and fixtures, exact owner-approved exclusions, and no protocol
# placeholders. This stays outside CI until Task 12 owns release aggregation.
[group('workbench')]
workbench-completion-eval-inputs-check:
    scripts/check-workbench-completion-eval-inputs.sh

# Run the test suite (Shell-SPEC §10.3).
[group('quality')]
test:
    cargo test --workspace --all-features --locked

# Test one workspace crate, optionally filtering by test name.
# Example: `just crate-test sea-forge-core authority`.
[group('quality')]
crate-test crate test_filter='':
    cargo test -p "{{crate}}" --locked "{{test_filter}}"

# Dependency-boundary gate (spec-full §6.1, spec-agent-orchestration G1/T12.5).
# Kernel crates MUST stay synchronous: no async runtime and no HTTP client.
# Only the approved adapter/runtime crates (sea-forge-agent, sea-forge-server)
# may pull async or HTTP deps. Async runtimes: tokio, async-std, smol, embassy.
# HTTP clients: reqwest, hyper, ureq, isahc, surf, attohttpc, minreq.
[group('quality')]
no-async-kernel:
    #!/usr/bin/env bash
    {{set}}
    kernel_crates=(
        sea-forge-core sea-forge-domain sea-forge-authority sea-forge-planner
        sea-forge-sandbox sea-forge-runtime sea-forge-trace sea-forge-evidence
        sea-forge-settlement sea-forge-capability sea-forge-extension
        sea-forge-ledger sea-forge-domainforge sea-forge-spec-pipeline
        sea-forge-cell sea-forge-artifact-ip sea-forge-self-model
        sea-forge-thoth sea-forge-case-runner
    )
    forbidden_deps=(
        tokio async-std smol embassy executor
        reqwest hyper ureq isahc surf attohttpc minreq actix-http awc
    )
    status=0
    for crate in "${kernel_crates[@]}"; do
        for dep in "${forbidden_deps[@]}"; do
            if output=$(cargo tree -i "$dep" -p "$crate" --locked 2>&1) \
                && echo "$output" | grep -q "^$dep "; then
                echo "fail: kernel crate $crate depends on $dep" >&2
                status=1
            fi
        done
    done
    if [ "$status" -ne 0 ]; then
        echo "fail: forbidden async/HTTP dependency in a kernel crate" >&2
        exit 1
    fi
    echo "ok: no async runtime or HTTP client in ${#kernel_crates[@]} kernel crates"

# Workbench (Bun workspace) lint + typecheck + build + test. Not part of the
# kernel `check`/`ci` gates — the frontend is developed and gated separately
# per docs/decisions/ADR-004-workbench-stack.md.
[group('quality')]
workbench-check: workbench-contracts-gate workbench-tauri-test
    #!/usr/bin/env bash
    {{set}}
    cd workbench && bun install --frozen-lockfile && bun run check && bun run build && bun run test

# Stage the server binary where `bundle.externalBin` expects it.
#
# Decision U-06: the packaged Workbench supervises its own kernel, so the server
# ships inside the bundle as a Tauri sidecar. Tauri resolves sidecars by
# target-triple suffix, and its build script fails outright when the named
# binary is absent — which is why this is a dependency of every recipe that
# compiles the host, not just of the packaging one.
#
# The staged copy is a build artifact and is gitignored. `debug` keeps the
# workbench test loop fast; packaging uses `release`.
[group('workbench')]
workbench-sidecar profile='debug':
    #!/usr/bin/env bash
    {{set}}
    triple="$(rustc -vV | sed -n 's/^host: //p')"
    if [ "{{profile}}" = "release" ]; then
        cargo build --locked --release -p sea-forge-server --bin sea-forge-server
        built="target/release/sea-forge-server"
    else
        cargo build --locked -p sea-forge-server --bin sea-forge-server
        built="target/debug/sea-forge-server"
    fi
    dest="workbench/apps/desktop/src-tauri/binaries/sea-forge-server-${triple}"
    mkdir -p "$(dirname "$dest")"
    # `cp` rather than a symlink: Tauri's bundler copies the file into the
    # package, and a dangling link would ship a broken sidecar.
    cp -f "$built" "$dest"
    echo "[sidecar] staged {{profile}} sea-forge-server -> $dest"

# Launch the Tauri desktop application with a debug sidecar.
[group('workbench')]
workbench-tauri-dev: (workbench-sidecar 'debug')
    #!/usr/bin/env bash
    {{set}}
    cd workbench && bun install --frozen-lockfile
    cd apps/desktop && bun run tauri dev

# Build the standalone Tauri host crate without packaging the Workbench.
[group('workbench')]
workbench-host-build:
    cargo build --locked --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml

# Build the installable Linux packages (SF-012).
#
# `deb` and `rpm` only. Every other target `tauri.conf.json` used to list has
# been removed for the same reason: advertising a target nobody has built is
# the over-claim SF-013 forbids.
#
#   * macOS (`app`, `dmg`) — the packet requires its Seatbelt journey to pass
#     before macOS may be called supported, and that has not been run.
#   * `appimage` — needs `libfuse2`, which AppImage's own tooling dlopens as
#     `libfuse.so.2`. This host has FUSE 3 only, so linuxdeploy exits before
#     producing anything. Re-enable by installing `libfuse2` and adding
#     `"appimage"` back to `bundle.targets`; nothing else has to change.
[group('workbench')]
workbench-package: (workbench-sidecar 'release')
    #!/usr/bin/env bash
    {{set}}
    cd workbench && bun install --frozen-lockfile
    cd apps/desktop && bun run tauri build
    echo "[package] bundles under workbench/apps/desktop/src-tauri/target/release/bundle/"

# Inventory the built package: what actually ships, and what must not.
#
# The acceptance criteria are negative claims ("no Bun runtime, no untracked
# generated source"), and a negative claim nobody checks is just a hope.
[group('workbench')]
workbench-package-inventory:
    #!/usr/bin/env bash
    {{set}}
    bundle="workbench/apps/desktop/src-tauri/target/release/bundle"
    deb="$(find "$bundle/deb" -name '*.deb' -print -quit 2>/dev/null || true)"
    if [ -z "$deb" ]; then
        echo "fail: no .deb found under $bundle — run 'just workbench-package' first" >&2
        exit 1
    fi
    echo "[inventory] $deb"
    contents="$(dpkg-deb -c "$deb")"
    echo "$contents" | awk '{print $6, $3}' | sort
    status=0
    # A Bun or Node runtime in the bundle would mean the renderer is being
    # served rather than compiled in — the frontend must ship as static assets.
    if echo "$contents" | grep -Eq '/(bun|node|npm|deno)$'; then
        echo "fail: a JavaScript runtime is present in the bundle" >&2
        status=1
    fi
    # The sidecar is the whole point of U-06; a bundle without it cannot start
    # a cell on a clean host.
    if ! echo "$contents" | grep -q 'sea-forge-server'; then
        echo "fail: the sea-forge-server sidecar is missing from the bundle" >&2
        status=1
    fi
    # Source maps expose the renderer's original sources and are not needed to
    # run it.
    if echo "$contents" | grep -q '\.map$'; then
        echo "fail: source maps are present in the bundle" >&2
        status=1
    fi
    if [ "$status" -ne 0 ]; then exit 1; fi
    echo "[inventory] ok: sidecar present, no JS runtime, no source maps"

# --- demonstration cell -----------------------------------------------------

# Seed a cell with real, inspectable records so the Workbench has something to
# show on a fresh machine.
#
# These are not fixtures. Every record is produced by really running the kernel
# — real runs, really authorized, really settled, really committed. A seeded
# record survives being followed to its evidence, because there is nothing
# behind it but the same code path an operator would have taken. Demo data that
# could not survive that inspection would be fabricated evidence, which is the
# one thing this system must never contain.
[group('workbench')]
cell-seed root='':
    #!/usr/bin/env bash
    {{set}}
    scripts/seed-cell.sh {{root}}

# Remove the demonstration cell. Refuses anything without the seed marker.
[group('workbench')]
cell-reset root='':
    #!/usr/bin/env bash
    {{set}}
    scripts/reset-cell.sh {{root}} --yes

# Open the packaged Workbench against the demonstration cell.
#
# No server is started here on purpose: the app supervises its own kernel
# (U-06), so this is also the demonstration that it does.
[group('workbench')]
workbench-demo root='':
    #!/usr/bin/env bash
    {{set}}
    cell="{{root}}"
    [ -n "$cell" ] || cell="${SEA_FORGE_DEMO_ROOT:-$HOME/.sea-forge-demo}"
    app="workbench/apps/desktop/src-tauri/target/release/sea-forge-workbench"
    if [ ! -x "$app" ]; then
        echo "no packaged Workbench at $app — run 'just workbench-package' first" >&2
        exit 1
    fi
    if [ ! -d "$cell" ]; then
        echo "no cell at $cell — run 'just cell-seed' first" >&2
        exit 1
    fi
    echo "[demo] opening $cell"
    SEA_FORGE_ROOT="$cell" \
      SEA_FORGE_SOCKET="${SEA_FORGE_DEMO_SOCKET:-/tmp/sea-forge-demo.sock}" \
      "$app"

# Browser-only renderer evidence, driven by agent-browser without injecting
# `__TAURI_INTERNALS__`. This is deliberately not an integrated Tauri claim:
# Chrome cannot exercise Tauri's Linux WebKit bridge. It proves the renderer
# fails closed when no native bridge is present and records screenshot/a11y
# evidence for that condition.
[group('workbench')]
workbench-e2e-agent-browser:
    #!/usr/bin/env bash
    {{set}}
    scripts/workbench-e2e-agent-browser.sh

# Native Linux integration evidence. This packages the app, seeds a temporary
# cell through real kernel/server operations, and drives the compiled WebKit
# Workbench through Tauri's native WebDriver intermediary. It fails closed when
# the machine lacks the documented `webkit2gtk-driver` prerequisite; it never
# substitutes the mocked Playwright harness.
[group('workbench')]
workbench-e2e-real filter='':
    #!/usr/bin/env bash
    {{set}}
    scripts/workbench-e2e-real.sh --preflight
    just workbench-package
    scripts/workbench-e2e-real.sh "{{filter}}"

# The desktop host's own Rust tests.
#
# `src-tauri` is a separate Cargo workspace (ADR-004, K-06), so
# `cargo test --workspace` from the repo root never compiles it. Nothing else
# ran these, and the gap was not theoretical: the SF-005 identity gate made
# every protected verb require an actor block, which broke the host's
# `tests/bridge.rs` correlation-recovery scenarios — and the whole kernel suite
# stayed green through it, because it never built them.
#
# This is the host's transport: the SFWP socket client, response pairing,
# reconnect, and the actor the renderer is structurally unable to forge. It
# needs a gate of its own precisely because it is out of the root workspace's
# reach.
[group('quality')]
workbench-tauri-test: (workbench-sidecar 'debug')
    #!/usr/bin/env bash
    {{set}}
    # Lint before test, for the same reason the tests exist at all: the root
    # workspace's `just lint` cannot see this crate either, so without this line
    # `src-tauri` is the one place in the repository where clippy never runs.
    # It found dead code and an `.err().expect()` the day it was added.
    cargo clippy --all-targets --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml -- -D warnings
    cargo test --locked --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml
    cargo fmt --check --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml

# Regenerate Rust schemas and their TypeScript/AJV projections.
[group('quality')]
workbench-contracts-generate:
    #!/usr/bin/env bash
    {{set}}
    cargo run --locked -p sea-forge-server --bin gen_sfwp_schema
    cd workbench && bun install --frozen-lockfile && bun run generate:contracts

# Validate the repository-local Workbench skill after editing it.
[group('quality')]
workbench-skill-check:
    python3 .agents/skills/building-sea-forge-workbench/scripts/validate-skill.py

# Generated-zone drift gate (ADR-005, GEN-01, API-01, ADR-004).
#
# Rust types are canonical; the TS interfaces, AJV validators, and UI token
# sheet are committed *projections*. This regenerates each one and requires the
# result to be byte-identical to what is committed. The Rust->JSON-Schema half
# is already gated by `conformance_sfwp::generated_schemas_are_committed_and_current`,
# so it is not repeated here.
#
# Workbench-only by design: the kernel gates (`check`, `ci`) must not acquire a
# Bun dependency (K-06).
[group('quality')]
workbench-contracts-gate:
    #!/usr/bin/env bash
    {{set}}
    cd workbench
    bun install --frozen-lockfile
    bun run generate:contracts
    # `git diff` alone would pass a regeneration that *added* a file — a new
    # SFWP type shows up untracked, not modified. Both checks are needed.
    if ! git diff --exit-code -- packages/contracts/generated; then
        echo "fail: generated TS/AJV contracts drifted from the committed schemas." >&2
        echo "      run 'just workbench-contracts-generate' and commit the result." >&2
        exit 1
    fi
    untracked="$(git status --porcelain --untracked-files=all -- packages/contracts/generated)"
    if [ -n "$untracked" ]; then
        echo "fail: generation produced files that are not committed:" >&2
        echo "$untracked" >&2
        exit 1
    fi
    # The token sheet is a byte copy of the spec source, not a transform.
    # The contracts gate invokes the token drift script directly because the
    # repository pins no separate Node toolchain.
    bun packages/sea-forge-ui-tokens/scripts/check-drift.mjs
    # ADR-004: src-tauri is its own Cargo workspace root. If its `[workspace]`
    # table were removed, cargo would walk up and adopt the repository root,
    # pulling Tauri's async dependency tree into the kernel workspace (BUILD-01).
    # `cargo metadata` names the root it actually resolved, which a grep for
    # `[workspace]` cannot.
    expected="$(cd apps/desktop/src-tauri && pwd -P)"
    # `|| true`: removing the table makes cargo error outright rather than
    # report a different root, and under `set -e` that would kill the recipe
    # before it could say why. An empty `actual` fails the comparison below and
    # prints the fix.
    actual="$(cargo metadata --no-deps --offline --format-version 1 \
        --manifest-path apps/desktop/src-tauri/Cargo.toml 2>/dev/null \
        | jq -r .workspace_root || true)"
    if [ "$actual" != "$expected" ]; then
        echo "fail: src-tauri resolved to workspace root '$actual', expected '$expected'." >&2
        echo "      restore the empty [workspace] table in apps/desktop/src-tauri/Cargo.toml (ADR-004)." >&2
        exit 1
    fi
    echo "ok: generated contracts, UI tokens, and the Tauri workspace boundary are current"

# Start the Workbench Vite dev server in the background (http://localhost:1420).
[group('workbench')]
workbench-dev-up:
    #!/usr/bin/env bash
    {{set}}
    mkdir -p workbench/.pid
    if [ -f workbench/.pid/dev.pid ] && kill -0 "$(cat workbench/.pid/dev.pid)" 2>/dev/null; then
        echo "[workbench-dev] already running (PID $(cat workbench/.pid/dev.pid)) on http://localhost:1420"
        exit 0
    fi
    # Abort if another process already owns port 1420
    if fuser 1420/tcp >/dev/null 2>&1; then
        echo "[workbench-dev] fail: port 1420 is already in use (run 'fuser -k 1420/tcp' to clear it)" >&2
        exit 1
    fi
    rm -f workbench/.pid/dev.pid
    echo "[workbench-dev] starting Vite dev server..."
    cd workbench && mkdir -p .pid
    setsid bash -c 'echo $$ > .pid/dev.pid && exec bun run dev > .pid/dev.log 2>&1' &
    sleep 2
    if [ -f .pid/dev.pid ]; then
        pid=$(cat .pid/dev.pid)
        if kill -0 "$pid" 2>/dev/null; then
            echo "[workbench-dev] started (PID $pid) on http://localhost:1420 — log: workbench/.pid/dev.log"
        else
            echo "[workbench-dev] fail: process $pid exited during startup — see workbench/.pid/dev.log" >&2
            rm -f .pid/dev.pid
            exit 1
        fi
    else
        echo "[workbench-dev] fail: pid file was not created" >&2
        exit 1
    fi

# Stop the running Workbench Vite dev server.
[group('workbench')]
workbench-dev-down:
    #!/usr/bin/env bash
    {{set}}
    if [ -f workbench/.pid/dev.pid ]; then
        pid=$(cat workbench/.pid/dev.pid)
        echo "[workbench-dev] stopping PID $pid"
        kill -- -"$pid" 2>/dev/null || kill "$pid" 2>/dev/null || true
        for i in 1 2 3 4 5; do
            kill -0 "$pid" 2>/dev/null || break
            sleep 1
        done
        kill -0 "$pid" 2>/dev/null && kill -9 -- -"$pid" 2>/dev/null || true
        rm -f workbench/.pid/dev.pid
    else
        echo "[workbench-dev] no pid file found"
    fi
    # Kill any process still holding port 1420 (e.g. orphaned Vite child)
    fuser -k 1420/tcp 2>/dev/null || true
    echo "[workbench-dev] stopped"

# Start Storybook component explorer in the background (http://localhost:6006).
[group('workbench')]
workbench-storybook-up:
    #!/usr/bin/env bash
    {{set}}
    mkdir -p workbench/.pid
    if [ -f workbench/.pid/storybook.pid ] && kill -0 "$(cat workbench/.pid/storybook.pid)" 2>/dev/null; then
        echo "[workbench-storybook] already running (PID $(cat workbench/.pid/storybook.pid)) on http://localhost:6006"
        exit 0
    fi
    # Abort if another process already owns port 6006
    if fuser 6006/tcp >/dev/null 2>&1; then
        echo "[workbench-storybook] fail: port 6006 is already in use (run 'fuser -k 6006/tcp' to clear it)" >&2
        exit 1
    fi
    rm -f workbench/.pid/storybook.pid
    echo "[workbench-storybook] starting Storybook..."
    cd workbench && mkdir -p .pid
    setsid bash -c 'echo $$ > .pid/storybook.pid && exec bun run --cwd packages/sea-forge-ui-components storybook > .pid/storybook.log 2>&1' &
    sleep 3
    if [ -f .pid/storybook.pid ]; then
        pid=$(cat .pid/storybook.pid)
        if kill -0 "$pid" 2>/dev/null; then
            echo "[workbench-storybook] started (PID $pid) on http://localhost:6006 — log: workbench/.pid/storybook.log"
        else
            echo "[workbench-storybook] fail: process $pid exited during startup — see workbench/.pid/storybook.log" >&2
            rm -f .pid/storybook.pid
            exit 1
        fi
    else
        echo "[workbench-storybook] fail: pid file was not created" >&2
        exit 1
    fi

# Stop the running Storybook component explorer.
[group('workbench')]
workbench-storybook-down:
    #!/usr/bin/env bash
    {{set}}
    if [ -f workbench/.pid/storybook.pid ]; then
        pid=$(cat workbench/.pid/storybook.pid)
        echo "[workbench-storybook] stopping PID $pid"
        kill -- -"$pid" 2>/dev/null || kill "$pid" 2>/dev/null || true
        for i in 1 2 3 4 5; do
            kill -0 "$pid" 2>/dev/null || break
            sleep 1
        done
        kill -0 "$pid" 2>/dev/null && kill -9 -- -"$pid" 2>/dev/null || true
        rm -f workbench/.pid/storybook.pid
    else
        echo "[workbench-storybook] no pid file found"
    fi
    # Kill any process still holding port 6006 (e.g. orphaned Storybook child)
    fuser -k 6006/tcp 2>/dev/null || true
    echo "[workbench-storybook] stopped"

# Convenient aliases for Workbench dev server and Storybook commands
alias dev-up := workbench-dev-up
alias dev-down := workbench-dev-down
alias storybook-up := workbench-storybook-up
alias storybook-down := workbench-storybook-down

# Canonical clean, deterministic, noninteractive CI verification.
# GitHub Actions invokes this (or its documented constituent recipes when
# parallelized). Local `just ci` is equivalent to the union of required jobs.
[group('quality')]
ci:
    #!/usr/bin/env bash
    {{set}}
    just context-check
    just fmt-check
    just lint
    just typecheck
    just security
    just test
    just no-async-kernel
    just build
    echo "[ci] all gates green"

# Re-converge after a pull that touched Cargo.toml/Cargo.lock/rust-toolchain.
[group('setup')]
sync:
    #!/usr/bin/env bash
    {{set}}
    rustup toolchain install "$(sed -n 's/^channel *= *"\(.*\)".*/\1/p' rust-toolchain.toml)" --profile minimal --component rustfmt,clippy --no-self-update
    cargo fetch --locked
    echo "[sync] toolchain + deps current"

# Verify every published manifest reports the same version, and that the tag
# (if passed as $1 or read from GITHUB_REF_NAME) matches. Phase 10.3 contract.
# Fails loudly with all values printed on any mismatch.
[group('quality')]
release-check tag='':
    #!/usr/bin/env bash
    {{set}}
    tag="{{tag}}"
    if [ -z "$tag" ] && [ -n "${GITHUB_REF_NAME:-}" ]; then
        tag="${GITHUB_REF_NAME#v}"
    fi
    # Strip a leading "v" from whatever source supplied the tag (CLI arg,
    # GITHUB_REF_NAME, or release-please's tag_name output).
    tag="${tag#v}"
    ws_version=$(awk '/^\[workspace\.package\]/{f=1} f&&/^version/{gsub(/[ "=]/,"",$0); print substr($0,8); exit}' Cargo.toml)
    meta=$(cargo metadata --no-deps --format-version 1)
    core_version=$(printf '%s' "$meta" | jq -r '.packages[] | select(.name=="sea-forge-core") | .version')
    cli_version=$(printf '%s' "$meta" | jq -r '.packages[] | select(.name=="sea-forge-cli") | .version')
    echo "workspace.package.version = $ws_version"
    echo "sea-forge-core            = $core_version"
    echo "sea-forge-cli             = $cli_version"
    [ -n "$tag" ] && echo "tag (stripped)            = $tag"
    if [ "$ws_version" != "$core_version" ] || [ "$ws_version" != "$cli_version" ]; then
        echo "fail: workspace, core, and cli versions disagree" >&2
        exit 1
    fi
    if [ -n "$tag" ] && [ "$ws_version" != "$tag" ]; then
        echo "fail: workspace version $ws_version != tag $tag" >&2
        exit 1
    fi
    echo "ok: versions synchronized"

# Minimum-spec proof commands (spec-minimum §12.2). Requires the slice.
[group('quality')]
proof:
    #!/usr/bin/env bash
    {{set}}
    echo "[proof] running spec-minimum §12.2 P1-P4b"
    cargo build -q -p sea-forge-cli
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    policy="$tmp/permissive.yaml"
    deny="$tmp/deny.yaml"
    printf '%s\n' 'version: "0.1"' 'rules:' \
      '  - name: allow-model-write' '    verdict: allow' \
      '    actor_role: operator' '    operation_kind: write_file' \
      '    path_prefix: ""' '  - name: allow-self-validate' \
      '    verdict: allow' '    actor_role: operator' \
      '    operation_kind: execute_command' '    argv0: sea-forge' >"$policy"
    printf '%s\n' 'version: "0.1"' 'rules: []' >"$deny"
    target/debug/sea-forge run --root "$tmp/state" --policy "$policy" \
      --intent "Generate and validate a simple DomainForge .sea model" >"$tmp/run.out"
    run_id="$(sed -n 's/^run_id=//p' "$tmp/run.out")"
    run="$tmp/state/runs/$run_id"
    test -f "$run/plan.json" && test -f "$run/authority.json"
    test -f "$run/trace.jsonl" && test -f "$run/evidence.jsonl"
    test -f "$run/settlement.json" && test -f "$run/semantic-envelope.json"
    jq -e '.status == "accepted" and (.basis | length > 0)' "$run/settlement.json" >/dev/null
    settlement_id="$(jq -r '.settlement_id' "$run/settlement.json")"
    jq -e --arg settlement_id "$settlement_id" '.settlement_ref == $settlement_id and (.artifact_refs | length == 1) and (.extension_refs|type=="array") and (.projection_refs|type=="array")' "$run/semantic-envelope.json" >/dev/null
    while read -r decision_id; do jq -e --arg id "$decision_id" 'any(.[]; .decision_id == $id)' "$run/authority.json" >/dev/null; done < <(jq -r '.authority_decisions[]' "$run/semantic-envelope.json")
    while read -r evidence_id; do jq -e --arg id "$evidence_id" 'select(.evidence_id == $id)' "$run/evidence.jsonl" >/dev/null; done < <(jq -r '.evidence_refs[]' "$run/semantic-envelope.json")
    case_id="$(jq -r '.case_ref' "$run/semantic-envelope.json")"
    jq -e --arg run_id "$run_id" '.state == "completed" and (.run_ids | index($run_id))' "$tmp/state/cases/$case_id.json" >/dev/null
    jq -e '.artifact_refs[0].pre_mint_identity | test("^ifl:hash:[a-f0-9]{64}$")' "$run/semantic-envelope.json" >/dev/null
    jq -e 'select(.kind=="artifact" and .uri=="artifacts/model.sea") | .metadata.artifact.pre_mint_identity | test("^ifl:hash:[a-f0-9]{64}$")' "$run/evidence.jsonl" >/dev/null
    jq -e 'all(.[]; (.determinism.policy_bundle_hash|test("^sha256:[a-f0-9]{64}$")) and .audit_record.engine and .audit_record.disposition and .audit_record.subject)' "$run/authority.json" >/dev/null
    while IFS=$'\t' read -r uri hash; do
      if command -v sha256sum >/dev/null 2>&1; then
        printf '%s  %s\n' "$hash" "$run/$uri" | sha256sum -c - >/dev/null
      else
        printf '%s  %s\n' "$hash" "$run/$uri" | shasum -a 256 -c - >/dev/null
      fi
    done < <(jq -r 'select(.kind=="artifact") | [.uri,.sha256] | @tsv' "$run/evidence.jsonl")
    set +e
    target/debug/sea-forge run --root "$tmp/denied" --policy "$deny" \
      --intent "Generate and validate a simple DomainForge .sea model" >"$tmp/denied.out"
    denied_exit=$?
    target/debug/sea-forge run --root "$tmp/boundary" --policy "$policy" \
      --intent "TEST_ONLY: write generated zone" >"$tmp/boundary.out"
    boundary_exit=$?
    set -e
    test "$denied_exit" -eq 3
    test "$boundary_exit" -eq 3
    denied_run="$tmp/denied/runs/$(sed -n 's/^run_id=//p' "$tmp/denied.out")"
    jq -e 'all(.[]; .verdict == "deny" and .matched_rule == null)' "$denied_run/authority.json" >/dev/null
    ! grep -q '"kind":"command_started"' "$denied_run/trace.jsonl"
    test -d "$denied_run/workspace"
    test -z "$(ls -A "$denied_run/workspace")"
    test "$(jq -r 'select(.kind=="authority_decision") | .evidence_id' "$denied_run/evidence.jsonl" | wc -l)" -eq "$(jq 'length' "$denied_run/authority.json")"
    boundary_run="$tmp/boundary/runs/$(sed -n 's/^run_id=//p' "$tmp/boundary.out")"
    jq -e 'any(.[]; .reason_codes | index("generated_zone_denied"))' "$boundary_run/authority.json" >/dev/null
    test ! -e "$tmp/boundary/runs"/*/workspace/src/gen/model.sea
    echo "[proof] P1-P4b passed"

# --- clean --------------------------------------------------------------

# Remove build output and bootstrap evidence. Does not touch tracked files.
[group('quality')]
clean:
    cargo clean
    rm -rf {{evidence_dir}}

# --- secrets (Shell-SPEC §8.4) -----------------------------------------

# Create a local age key if absent and print its public key.
[group('secrets')]
secrets-init:
    #!/usr/bin/env bash
    {{set}}
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    mkdir -p "$(dirname "$SOPS_AGE_KEY_FILE")"
    if [ -f "$SOPS_AGE_KEY_FILE" ]; then
        echo "age key already exists at $SOPS_AGE_KEY_FILE"
    else
        umask 077
        age-keygen -o "$SOPS_AGE_KEY_FILE"
        chmod 600 "$SOPS_AGE_KEY_FILE"
        echo "created age key at $SOPS_AGE_KEY_FILE"
    fi
    echo "public recipient (add to .sops.yaml):"
    grep -o 'age1[a-z0-9]*' "$SOPS_AGE_KEY_FILE" | head -1

# Edit an encrypted profile via SOPS without leaving plaintext behind.
[group('secrets')]
secrets-edit profile='dev':
    #!/usr/bin/env bash
    {{set}}
    f="secrets/{{profile}}.enc.env"
    [ -f "$f" ] || { echo "missing $f — creating from template"; \
        printf '# %s profile\nSEA_EXAMPLE_API_KEY=replace-me\n' "{{profile}}" > "$f"; }
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    export SOPS_AGE_KEY_FILE
    sops "$f"

# Decrypt to a pipe, validate names, redact values, report missing requirements.
[group('secrets')]
secrets-check profile='dev':
    #!/usr/bin/env bash
    {{set}}
    f="secrets/{{profile}}.enc.env"
    [ -f "$f" ] || { echo "fail: $f not found"; echo "next move: just secrets-edit {{profile}}"; exit 1; }
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    if [ ! -f "$SOPS_AGE_KEY_FILE" ]; then
        echo "fail: missing_credential_error — no age key at $SOPS_AGE_KEY_FILE"
        echo "next move: just secrets-init"
        exit 1
    fi
    export SOPS_AGE_KEY_FILE
    if ! sops -d "$f" >/dev/null 2>/dev/null; then
        echo "fail: secret_decrypt_error — could not decrypt $f (redacted)"
        echo "next move: verify SOPS_AGE_KEY_FILE matches .sops.yaml recipient"
        exit 1
    fi
    echo "ok: {{profile}} profile decrypts"
    echo "declared variables (values redacted):"
    sops -d "$f" | sed -n 's/^\(SEA_[A-Za-z0-9_]*\)=.*/  \1=<redacted>/p'

# Update encrypted files after recipient changes.
[group('secrets')]
secrets-rekey:
    #!/usr/bin/env bash
    {{set}}
    : "${SOPS_AGE_KEY_FILE:=$HOME/.config/sops/key.txt}"
    export SOPS_AGE_KEY_FILE
    for f in secrets/*.enc.env; do
        [ -f "$f" ] || continue
        echo "[rekey] $f"
        sops updatekeys "$f"
    done

# --- integration (Shell-SPEC §7.2, §10.2) ------------------------------

# Validate and run a declared API/MCP integration by name.
[group('integration')]
integration name:
    #!/usr/bin/env bash
    {{set}}
    echo "fail: no integrations declared yet (Shell-SPEC §2.2)"
    echo "next move: declare a SecretRequirement in .agents/specs/Shell-SPEC.md §7.2, then implement its validation"
    exit 1

# --- hooks (Phase 6) ----------------------------------------------------

# Install the checked-in hooks at .githooks via `core.hooksPath`.
# Idempotent: safe to rerun after pulling hook changes.
[group('hooks')]
hooks-install:
    git config core.hooksPath .githooks
    chmod +x .githooks/pre-commit .githooks/pre-push .githooks/post-checkout
    echo "[hooks] core.hooksPath = .githooks (run 'git config --unset core.hooksPath' to disable)"

# Fast staged check invoked by .githooks/pre-commit. Kept to checks that are
# reliably under ~10s and require no network: context, fmt, typecheck.
[group('hooks')]
pre-commit:
    #!/usr/bin/env bash
    {{set}}
    just check-fast

# Broad local verification invoked by .githooks/pre-push. Adds clippy,
# supply-chain, secret scan, tests, and a debug build on top of pre-commit.
[group('hooks')]
pre-push:
    #!/usr/bin/env bash
    {{set}}
    just ci

# Same recipe the PR-title workflow + gate rely on; alias of `just ci` so a
# maintainer can ask "what does CI see?" with one command.
[group('hooks')]
pr-check:
    just ci

# --- pull-request helper (Phase 9) -------------------------------------

# Verify, push the current branch, and open a PR via gh. Refuses to operate
# from main, requires a clean tree, and never merges automatically.
[group('hooks')]
pr:
    #!/usr/bin/env bash
    {{set}}
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" = "main" ]; then
        echo "fail: refusing to open a PR from main — switch to a feature branch" >&2
        echo "next move: git switch -c feat/short-description" >&2
        exit 1
    fi
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "fail: working tree has uncommitted changes — commit or stash first" >&2
        exit 1
    fi
    git fetch origin main --quiet
    if ! git merge-base --is-ancestor origin/main HEAD; then
        echo "fail: branch is behind main — rebase first" >&2
        echo "next move: git fetch origin && git rebase origin/main" >&2
        exit 1
    fi
    just pr-check
    git push -u origin HEAD
    gh pr create --base main --fill
    echo "[pr] opened; review at the URL above. Merging is a manual step."

# --- publication (Phase 10) --------------------------------------------

# One-time manual crates.io bootstrap publish. OIDC trusted publishing for
# crates.io requires (a) at least one classic-token publish of each crate and
# (b) linking the repository on https://crates.io/settings before the
# automated publish job can use trusted publishing. Run this once per crate
# with a short-lived classic token in CARGO_REGISTRY_TOKEN, then revoke the
# token. After this, release-please.yml handles subsequent publishes via OIDC.
[group('release')]
publish-bootstrap crate:
    #!/usr/bin/env bash
    {{set}}
    if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
        echo "fail: CARGO_REGISTRY_TOKEN is not set" >&2
        echo "next move: create a one-time classic token at https://crates.io/settings/tokens (scope: publish-new), export CARGO_REGISTRY_TOKEN=<token>, then rerun" >&2
        exit 1
    fi
    case "{{crate}}" in
        sea-forge-core|sea-forge-cli) ;;
        *) echo "fail: unknown crate '{{crate}}' (expected sea-forge-core or sea-forge-cli)" >&2; exit 1 ;;
    esac
    just release-check
    cargo publish --locked -p {{crate}}
    echo "[publish-bootstrap] {{crate}} published; now link the repo on https://crates.io/crates/{{crate}}/settings"

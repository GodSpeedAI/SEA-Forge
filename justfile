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

# Print the active compiler and fail if rustc is not the rust-toolchain.toml channel.
[group('quality')]
toolchain-check:
    #!/usr/bin/env bash
    {{set}}
    pin="$(sed -n 's/^channel *= *"\(.*\)".*/\1/p' rust-toolchain.toml)"
    rustc --version
    cargo --version
    cargo clippy --version
    echo "channel (rust-toolchain.toml): $pin"
    if ! printf '%s' "$pin" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
        echo "toolchain-check: channel '$pin' is not an exact version; pin rust-toolchain.toml to major.minor.patch" >&2
        exit 1
    fi
    active="$(rustc --version | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+' | head -n1)"
    if [ "$active" != "$pin" ]; then
        echo "toolchain-check: rustc $active != channel $pin" >&2
        exit 1
    fi
    echo "toolchain-check: ok"

# Type-check one workspace crate. Example: `just crate-check sea-forge-core`.
[group('quality')]
crate-check crate:
    cargo check -p "{{crate}}" --locked

# Standard package-scoped type-check interface; retains the established crate gate.
[group('quality')]
check-package package:
    just crate-check "{{package}}"

# Supply-chain + secret scan: cargo-deny then gitleaks.
# `cargo deny check advisories` fetches the RustSec database; the other
# categories are offline. `security` scans committed history and runs the
# staged-scan canaries; `pre-commit` scans the index (`secrets-staged`).
[group('quality')]
deny:
    cargo deny check

[group('quality')]
security:
    #!/usr/bin/env bash
    {{set}}
    just deny
    gitleaks detect --no-banner --redact
    scripts/security/staged-secret-canaries.sh

# Scan exactly what the next commit would contain (the index), redacted.
[group('quality')]
secrets-staged:
    gitleaks git --staged --no-banner --redact .

# Apply only safe automatic fixes (rustfmt). Clippy fixes are intentionally
# excluded: `-A clippy::all --fix` mutates behavior and should be reviewed.
[group('quality')]
fix:
    cargo fmt --all

# Quick inner-loop check: context + fmt + workspace typecheck (manual)
[group('quality')]
check-fast:
    #!/usr/bin/env bash
    {{set}}
    scripts/check-agent-context.sh
    cargo fmt --all -- --check
    cargo check --workspace --all-targets --locked

# Developer quality sweep (Shell-SPEC §10.3): status + fmt + clippy + check
# + deny + gitleaks. Delegates to granular recipes so local and CI share one
# implementation per category.
[group('quality')]
check:
    #!/usr/bin/env bash
    {{set}}
    just status-check
    just fmt-check
    just lint
    just typecheck
    just security
    echo "[check] all gates green"

# Show the latest complete agent status without reading its history
[group('quality')]
status:
    @tail -n 1 .agents/CURRENT_STATUS.yaml

# Validate agent handoff structure and freshness without reading full history.
[group('quality')]
status-check:
    scripts/check-agent-context.sh

# Validate agent handoff structure and freshness without vendor-specific tooling (alias for status-check).
[group('quality')]
context-check: status-check

# Verify the frozen E2E convergence preregistration (.agents/specs/
# e2e-preregistration.yml) still matches the hash bound under
# source.spec.sha256 in .agents/plans/e2e-plan.yml. A mismatch stops the
# convergence/Gauntlet run until the change is reviewed and re-frozen.
[group('quality')]
e2e-prereg-check:
    scripts/check-e2e-preregistration.sh

# --- e2e convergence gates (plan: .agents/plans/e2e-plan.yml) ------------
# Stable aliases bound by task T00 to the repository's real existing gates.
# They orchestrate existing checks; they never duplicate or weaken them.

# Plan global gate CHECK -> repository composite quality gate.
[group('quality')]
e2e-check:
    just check

# Plan global gate TEST -> repository test suite.
[group('quality')]
e2e-test:
    just test

# Plan global gate LINT -> formatting + lint gates.
[group('quality')]
e2e-lint:
    just fmt-check
    just lint

# Plan global gate DELTA -> mechanical requirement/evidence matrix check.
[group('quality')]
e2e-delta-check:
    scripts/e2e-delta-check.sh

# Regenerate .agents/evidence/e2e/T00/delta0.md from the status file.
[group('quality')]
e2e-delta-report:
    scripts/e2e-delta-report.sh > .agents/evidence/e2e/T00/delta0.md

# Stable per-task gate entry point (plan verification.task_gate_contract).
# Tasks bind themselves here as their contract tests land; unbound tasks fail
# closed instead of passing vacuously.
[group('quality')]
e2e-gate TASK_ID:
    #!/usr/bin/env bash
    {{set}}
    case "{{TASK_ID}}" in
        T00)
            just e2e-prereg-check
            just e2e-delta-check
            ;;
        T01)
            # Canonical semantic envelope + DomainForge identity foundation
            # (lives in SWE_SEED swe-seed-core federation module).
            cargo test --manifest-path "${SWE_SEED_ROOT:-$HOME/projects/SWE_SEED}/Cargo.toml" -p swe-seed-core --test convergence_t01_envelope --test v1_contract --test federation_parity
            ;;
        T02)
            # Cited context slice E2/E3/I4: canonical boundary on both sides.
            # CK ingress gates (Context_Kernel) + SWE_SEED consumer adjudication,
            # plus the live cross-binary MCP stdio integration (env-gated).
            cargo test --manifest-path "${CONTEXT_KERNEL_ROOT:-$HOME/projects/Context_Kernel}/Cargo.toml" -p ck-mcp --lib --test acl_context_agent
            cargo test --manifest-path "${SWE_SEED_ROOT:-$HOME/projects/SWE_SEED}/Cargo.toml" -p swe-seed-core --test convergence_t02_context_slice --test context_kernel_client
            ;;
        T03)
            # Navigation ingress + work contract E0/E1: GSA projector teeth +
            # SWE_SEED canonical WorkRequested ingress incl. the cross-language
            # golden fixture from GodSpeed-Agent's real projector.
            uv run --project "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" --extra dev python -m pytest "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}/tests/test_canonical_events.py" -q
            GODSPEED_AGENT_ROOT="${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" cargo test --manifest-path "${SWE_SEED_ROOT:-$HOME/projects/SWE_SEED}/Cargo.toml" -p swe-seed-core --test convergence_t03_work_ingress
            ;;
        T04)
            # Governed work submission E4: SWE_SEED canonical emitter teeth
            # (also regenerates the cross-repo golden fixture from the real
            # producer) + SEA-Forge acceptance gate consuming that fixture.
            cargo test --manifest-path "${SWE_SEED_ROOT:-$HOME/projects/SWE_SEED}/Cargo.toml" -p swe-seed-core --test convergence_t04_governed_submission
            cargo test -p sea-forge-server --test convergence_t04_governed_ingress
            ;;
        T05)
            # Governed contact with reality E5A/E5B/I5: canonical
            # AuthorizedInvocation emission from REAL authority decisions +
            # ExecutionObservation adjudication bound to the exact authorized
            # invocation, incl. the deny/escalate teeth through the real
            # engine/runtime seam.
            cargo test -p sea-forge-server --test convergence_t05_authorized_execution --test convergence_t05_execution_observation
            ;;
        T06)
            # Operational settlement return E6/I6/I7: SEA-Forge settlement
            # evaluation from ledger-settled observations against DECLARED
            # criteria + canonical OperationalSettlement emission bound to the
            # actual invocation chain (regenerates the cross-repo golden
            # fixture), then SWE_SEED proof-plane adjudication consuming it —
            # producer authority, duplicate/conflict idempotency, and no
            # developmental promotion from operational settlement.
            cargo test -p sea-forge-server --test convergence_t06_operational_settlement
            cargo test --manifest-path "${SWE_SEED_ROOT:-$HOME/projects/SWE_SEED}/Cargo.toml" -p swe-seed-core --test convergence_t06_operational_settlement
            ;;
        T07)
            # Proof-to-RealityTrace boundary E7: canonical ProofCompleted
            # emission bound to a REAL adjudicated OperationalSettlement for
            # the SAME work_request_id (regenerates the cross-repo golden
            # fixture), then sxr RealityTrace native ingestion consuming it —
            # exclusive swe_seed producer authority, placeholder/drift
            # identity gates, mandatory settlement causal parent with
            # content-addressed reference resolution, expected-versus-observed
            # comparison inputs preserved BY REFERENCE, and duplicate/
            # claim-mismatch refusal so upstream facts are never rewritten.
            cargo test --manifest-path "${SWE_SEED_ROOT:-$HOME/projects/SWE_SEED}/Cargo.toml" -p swe-seed-core --test convergence_t07_proof_completed
            cargo test --manifest-path "${SXR_ROOT:-$HOME/projects/sxr}/Cargo.toml" -p sxr-core --test convergence_t07_proof_ingestion
            ;;
        T08)
            # Expected-versus-observed developmental evidence E8/I8/I10:
            # canonical EvidenceRecorded emission from RealityTrace over ONLY
            # the comparison facts its REAL T07 gate preserved BY REFERENCE
            # (regenerates the cross-repo golden fixture), then GodSpeed-
            # Agent's Python ingestion gate consuming that fixture — exclusive
            # realitytrace producer authority both directions, placeholder/
            # drift identity gates, the complete nine-required-field battery,
            # causality presence with out-of-band parent pinning,
            # cryptographically bound difference (I10 probes detected and
            # named), duplicate/claim-mismatch idempotency, and STRICTLY
            # PROVISIONAL receipt: no SettlementRecorded/CapabilityUpdated or
            # any promotion path exists from ingestion alone.
            cargo test --manifest-path "${SXR_ROOT:-$HOME/projects/sxr}/Cargo.toml" -p sxr-core --test convergence_t08_evidence_emission
            SXR_ROOT="${SXR_ROOT:-$HOME/projects/sxr}" uv run --project "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" --extra dev python -m pytest "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}/tests/test_convergence_t08_evidence_recorded.py" -q
            ;;
        T09)
            # Developmental settlement + durable memory E9/I9: canonical
            # five-type E9 emission stamped by the EXCLUSIVE producer
            # (godspeed_agent) carrying the four common required payload
            # fields and caused_by provenance to the justifying evidence;
            # the durable memory gate enforcing placeholder/drift/namespace
            # identity gates, quarantine of stripped provenance (never
            # persisted as authoritative developmental truth), idempotent
            # redelivery over the flock-guarded append-only ledger with
            # restart survival via replay-through-the-gate, and the I9
            # promotion gate — metabolized capability requires repeated
            # settlement under DECLARED variation, composed with
            # infer_capability_lifecycle / Capability.from_settlements;
            # consumes the committed golden fixture from the real emitter.
            uv run --project "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" --extra dev python -m pytest "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}/tests/test_convergence_t09_developmental_memory.py" -q
            ;;
        T10)
            # Developmental memory feedback E10/I12: durable DevelopmentalMemory
            # (memory_ledger -> godspeed_agent) projected from the T09 ledger
            # via honest lifecycle (infer_capability_lifecycle / compute_metabolization),
            # consumable for next navigation with gate-evaluation-after-memory;
            # historical success may reduce search burden (rank boost) but never
            # bypasses current governance, payment, or settlement-access gates;
            # historical capability remains distinguishable from current affordance.
            # Also bridges LearningProposalCreated to CK's learning-proposals://
            # corpus as cited evidence with preserved provenance.
            uv run --project "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" --extra dev python -m pytest "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}/tests/test_convergence_t10_developmental_memory_feedback.py" -q
            ;;
        T11)
            # Whole-loop causality, replay, and recovery I2/I11/I14/I15: deterministic
            # golden scenario E0→E10 with one stable WR_ID and one resolvable domain
            # hash, plus cross-wire/duplicate/late/out-of-order/interruption/restart/
            # wrong-domain harnesses. Python harness covers E0→E10 composition and
            # GSA-side recovery; Rust harness covers SEA-Forge E4→E6 replay/late/
            # wrong-domain/failure-visibility via real gates. No test-only E8.
            uv run --project "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" --extra dev python -m pytest "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}/tests/test_convergence_t11_whole_loop.py" -q
            cargo test -p sea-forge-server --test convergence_t11_whole_loop
            ;;
        T12)
            # Variation battery: confirms all 35 frozen requirements under 7 variation
            # classes (prereg whole_loop_acceptance.variation_requirement.minimum_variation_classes):
            #   success, authority_denial_or_escalation, proof_failure,
            #   execution_failure_or_timeout, interruption_and_recovery,
            #   duplicate_or_replayed_event, semantically_invalid_or_wrong_domain_identity.
            # Also runs just e2e-delta-check (exit 0 required).
            # Python harness covers E0→E10 under each variation through real GSA producers
            # (project_desired_direction, project_work_requested, ingest_evidence_recorded,
            # persist_developmental_event, build_developmental_memory_from_store).
            # Rust harness covers SEA-Forge E4→E6 under each variation through real gates
            # (accept_governed_work_request, emit_authorized_invocation, InvocationLedger,
            # emit_operational_settlement).
            uv run --project "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" --extra dev python -m pytest "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}/tests/test_convergence_t12_variation.py" -q
            cargo test -p sea-forge-server --test convergence_t12_variation
            just e2e-delta-check
            ;;
        *)
            echo "fail: no gate is bound for {{TASK_ID}} yet; bind it in the e2e-gate recipe when the task's contract tests land" >&2
            exit 1
            ;;
    esac

# Verify the handoff a cold independent Workbench evaluator needs: concrete
# commands and fixtures, exact owner-approved exclusions, and no protocol
# placeholders. This stays outside CI until Task 12 owns release aggregation.
[group('workbench')]
workbench-completion-eval-inputs-check:
    scripts/check-workbench-completion-eval-inputs.sh

# Run the test suite (Shell-SPEC §10.3).
[group('quality')]
test:
    cargo test --workspace --all-features --locked --no-fail-fast

# Test one workspace crate, optionally filtering by test name.
# Example: `just crate-test sea-forge-core authority`.
[group('quality')]
crate-test crate test_filter='':
    cargo test -p "{{crate}}" --locked "{{test_filter}}"

# Standard package-scoped test interface; retains the established crate gate.
[group('quality')]
test-package package:
    just crate-test "{{package}}"

# Run doctests separately; the canonical test gate uses the full feature set.
[group('quality')]
test-doc:
    cargo test --workspace --all-features --doc --locked

# Explicit developer diagnostics. These are deliberately outside check, ci,
# and verify: each is substantially more expensive than the normal gate.
[group('quality')]
timings:
    cargo build --workspace --all-targets --locked --timings

[group('quality')]
coverage:
    cargo llvm-cov --workspace --all-features --locked --html

[group('quality')]
mutation:
    cargo mutants --workspace --all-features

# Print compiler-cache diagnostics without making sccache a repository requirement.
# Bound the client call so an unreachable cache daemon cannot hang a developer shell.
[group('quality')]
cache-stats:
    #!/usr/bin/env bash
    {{set}}
    if ! command -v sccache >/dev/null 2>&1; then
        echo "sccache is not installed or not on PATH"
        exit 0
    fi
    sccache --show-stats &
    pid=$!
    for _ in {1..10}; do
        if ! kill -0 "$pid" 2>/dev/null; then
            wait "$pid"
            exit $?
        fi
        sleep 1
    done
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    echo "sccache --show-stats timed out after 10 seconds" >&2
    exit 1

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
        sea-forge-thoth sea-forge-case-runner sea-forge-changeset
    )
    forbidden_deps=(
        tokio async-std smol embassy executor
        reqwest hyper ureq isahc surf attohttpc minreq actix-http awc
    )
    status=0
    for crate in "${kernel_crates[@]}"; do
        for dep in "${forbidden_deps[@]}"; do
            # Review pass 2 (R5): a failing cargo invocation must not read as
            # "dependency absent" here either. The absence case IS a cargo
            # failure (an inverted tree of a dep the crate lacks exits
            # nonzero with the package-ID not-found text), so the failure
            # status is captured in the `else` of a NON-negated if and
            # classified by cargo's own text below — the negated
            # `if ! output=$(...)` shape would set tree_status from the
            # negation's success and swallow resolution errors.
            tree_status=0
            if output=$(cargo tree -i "$dep" -p "$crate" --locked 2>&1); then
                :
            else
                tree_status=$?
            fi
            if [ "$tree_status" -ne 0 ]; then
                if echo "$output" | grep -qE "did not match any packages|package ID specification"; then
                    continue
                fi
                echo "fail: cargo tree -i $dep -p $crate exited $tree_status — treated as a gate failure, not as absence:" >&2
                echo "$output" >&2
                status=1
                continue
            fi
            if echo "$output" | grep -q "^$dep "; then
                echo "fail: kernel crate $crate depends on $dep" >&2
                status=1
            fi
        done
    done
    # A per-crate pass cannot see feature unification: `-p` resolves the crate
    # in isolation, so a cfg(windows)-only optional dependency enabled by an
    # adapter crate (e.g. sea-forge-os/tokio-pipes via sea-forge-server, review
    # F33) is invisible on every target. Re-check the unified workspace graph
    # for the host and for the Windows MSVC target (the triple the
    # package/release workflows use): fail if any kernel crate appears among a
    # forbidden dep's dependents. `cargo tree --target` resolves from the
    # lockfile: no target toolchain and no network access are needed.
    for target in host x86_64-pc-windows-msvc; do
        if [ "$target" = host ]; then
            target_args=""
        else
            target_args="--target $target"
        fi
        for dep in "${forbidden_deps[@]}"; do
            # Normal edges only: a kernel crate's own direct or dev dependency
            # on a forbidden dep is already caught by the per-crate pass above.
            # $target_args is intentionally unquoted: it is either empty or
            # exactly two words (--target <triple>).
            #
            # Fail-closed cargo-tree distinction (review Medium-13): a FAILED
            # invocation must not read as "dependency absent". The only
            # acceptable failure is the dependency being genuinely absent from
            # this target's graph, which cargo reports as a package-ID match
            # failure ("did not match any packages" / "package ID
            # specification"). Any other failure — resolution error, lockfile
            # drift, a broken cargo — prints its stderr and FAILS the gate:
            # absence is proven only by cargo's own not-found text, never by
            # an unexamined error status.
            # `set -e` is active (`{{set}}`), so a failing command must be
            # part of an `if` condition to be observable: a plain assignment
            # with a nonzero status aborts the recipe silently. The capture
            # is a NON-negated if with the failure status taken in `else`
            # (review pass 2, R5: `if ! output=$(...)` set tree_status from
            # the negation's SUCCESS, so a resolution failure read as
            # absence and the gate passed). Absence is then decided by
            # cargo's own not-found text, and any OTHER failure fails the
            # gate below.
            tree_status=0
            if output=$(cargo tree -i "$dep" -e normal $target_args --locked 2>&1); then
                :
            else
                tree_status=$?
            fi
            if [ "$tree_status" -ne 0 ]; then
                if echo "$output" | grep -qE "did not match any packages|package ID specification"; then
                    continue
                fi
                echo "fail: cargo tree -i $dep (unified graph, target: $target) exited $tree_status — treated as a gate failure, not as absence:" >&2
                echo "$output" >&2
                status=1
                continue
            fi
            for crate in "${kernel_crates[@]}"; do
                if echo "$output" | grep -q "$crate v"; then
                    echo "fail: kernel crate $crate depends on $dep (unified workspace graph, target: $target)" >&2
                    status=1
                fi
            done
        done
    done
    if [ "$status" -ne 0 ]; then
        echo "fail: forbidden async/HTTP dependency in a kernel crate" >&2
        exit 1
    fi
    echo "ok: no async runtime or HTTP client in ${#kernel_crates[@]} kernel crates (host and x86_64-pc-windows-msvc graphs)"

# Workbench (Bun workspace) lint + typecheck + build + test. Not part of the
# kernel `check`/`ci` gates — the frontend is developed and gated separately
# per docs/decisions/ADR-004-workbench-stack.md.
[group('quality')]
workbench-check: workbench-contracts-gate
    #!/usr/bin/env bash
    {{set}}
    cd workbench && bun install --frozen-lockfile && bun run check && bun run build && bun run test

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

# Case-authoring proof journeys (stale-precondition repair; dropped-commit
# recovery without duplicate side effects) driven by agent-browser against the
# real renderer with a mocked Tauri IPC bridge — the agent-browser counterpart
# of the Playwright mocked-IPC harness, not real-stack coverage. Boots its own
# ephemeral Vite server; requires `just workbench-check`-installed deps.
[group('workbench')]
workbench-e2e-case-authoring:
    #!/usr/bin/env bash
    {{set}}
    scripts/workbench-e2e-case-authoring.sh

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
    echo "ok: generated contracts and UI tokens are current"

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
#
# Teardown touches only the PID recorded by `workbench-dev-up`
# (`workbench/.pid/dev.pid`; that recipe starts the server as a `setsid`
# session leader, so signalling the recorded PID's group reaches Vite and its
# children). Nothing is killed merely for listening on port 1420: a missing or
# stale pid file is reported and cleaned up, never turned into collateral
# damage (F-25.d).
[group('workbench')]
workbench-dev-down:
    #!/usr/bin/env bash
    {{set}}
    pid_file="workbench/.pid/dev.pid"
    if [ ! -f "$pid_file" ]; then
        echo "[workbench-dev] warn: no pid file at $pid_file — nothing was started by workbench-dev-up; port 1420 is left untouched" >&2
        exit 0
    fi
    pid=$(cat "$pid_file")
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "[workbench-dev] warn: recorded PID $pid is not running — removing stale $pid_file" >&2
        rm -f "$pid_file"
        exit 0
    fi
    echo "[workbench-dev] stopping PID $pid"
    kill -- -"$pid" 2>/dev/null || kill "$pid" 2>/dev/null || true
    for i in 1 2 3 4 5; do
        kill -0 "$pid" 2>/dev/null || break
        sleep 1
    done
    if kill -0 "$pid" 2>/dev/null; then
        echo "[workbench-dev] PID $pid did not exit; sending SIGKILL to its group"
        kill -9 -- -"$pid" 2>/dev/null || true
    fi
    rm -f "$pid_file"
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
#
# Same PID-file-scoped teardown as `workbench-dev-down` (F-25.d): only the
# process group recorded by `workbench-storybook-up` is signalled; port 6006
# is never used to identify a victim.
[group('workbench')]
workbench-storybook-down:
    #!/usr/bin/env bash
    {{set}}
    pid_file="workbench/.pid/storybook.pid"
    if [ ! -f "$pid_file" ]; then
        echo "[workbench-storybook] warn: no pid file at $pid_file — nothing was started by workbench-storybook-up; port 6006 is left untouched" >&2
        exit 0
    fi
    pid=$(cat "$pid_file")
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "[workbench-storybook] warn: recorded PID $pid is not running — removing stale $pid_file" >&2
        rm -f "$pid_file"
        exit 0
    fi
    echo "[workbench-storybook] stopping PID $pid"
    kill -- -"$pid" 2>/dev/null || kill "$pid" 2>/dev/null || true
    for i in 1 2 3 4 5; do
        kill -0 "$pid" 2>/dev/null || break
        sleep 1
    done
    if kill -0 "$pid" 2>/dev/null; then
        echo "[workbench-storybook] PID $pid did not exit; sending SIGKILL to its group"
        kill -9 -- -"$pid" 2>/dev/null || true
    fi
    rm -f "$pid_file"
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
    just status-check
    just fmt-check
    just lint
    just typecheck
    just security
    just test
    just no-async-kernel
    just build
    echo "[ci] all gates green"

# Standard normal-verification interface; ci remains the authoritative recipe.
[group('quality')]
verify:
    just ci

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

# Compilation, clippy and tests run on GitHub (`CI / gate`); reproduce a
# failed job with `just ci-job <job>`.
# Hook pre-commit: context + fmt + staged secrets (seconds, no compile)
[group('hooks')]
pre-commit:
    #!/usr/bin/env bash
    {{set}}
    just status-check
    just fmt-check
    just secrets-staged

# The full sweep runs on GitHub (`CI / gate`), which gates every pull request.
# Run it locally with `just ci`, or one GitHub job with `just ci-job <job>`.
# Hook pre-push: context + fmt (seconds, no compile)
[group('hooks')]
pre-push:
    #!/usr/bin/env bash
    {{set}}
    just status-check
    just fmt-check

# Jobs: lint, test, msrv, workbench, macos (= `just ci`), authority-opa
# (needs `opa` on PATH). Example: just ci-job lint
# Run locally exactly what one GitHub CI job runs (troubleshoot a red check)
[group('hooks')]
ci-job job:
    #!/usr/bin/env bash
    {{set}}
    case "{{job}}" in
        lint) just status-check; just fmt-check; just lint; just typecheck; just security ;;
        test) just test; just build ;;
        msrv) RUSTUP_TOOLCHAIN=1.92.0 cargo check --workspace --all-targets --locked ;;
        workbench) just workbench-check ;;
        macos|ci) just ci ;;
        authority-opa)
            cargo test -p sea-forge-authority --test opa_candidate -- --include-ignored
            cargo test -p sea-forge-case-runner --test cell_episode -- --include-ignored ;;
        *) echo "unknown job '{{job}}'; one of: lint test msrv workbench macos authority-opa" >&2; exit 2 ;;
    esac

# Same recipe the PR-title workflow + gate rely on; alias of `just ci` so a
# maintainer can ask "what does CI see?" with one command.
[group('hooks')]
pr-check:
    just ci

# --- pull-request helper (Phase 9) -------------------------------------

# Refuses to operate from main, requires a clean tree, and never merges
# automatically. GitHub's `CI / gate` runs the full sweep on the PR.
# Light check, push the current branch, and open a PR via gh
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
    just pre-push
    git push -u origin HEAD
    gh pr create --base main --fill
    echo "[pr] opened; review at the URL above. Merging is a manual step."

# --- publication (Phase 10) --------------------------------------------


# --- casework-environment plan (plan: .agents/plans/godspeed-casework-cognitive-environment.plan.yaml)
# GATE_GO, activated by T01: Go format, vet, and tests for the casework front end. This is the gate
# the plan names `just casework-go-check`; it must observe the real implementation, never a stub.
[group('casework')]
casework-go-check:
    #!/usr/bin/env bash
    {{set}}
    module="apps/godspeed-casework-go"
    if [ ! -f "$module/go.mod" ]; then
      echo "casework-go-check: no Go module at $module" >&2
      exit 1
    fi
    if ! command -v go >/dev/null 2>&1; then
      echo "casework-go-check: the Go toolchain is required (mise declares it in mise.toml)" >&2
      exit 1
    fi
    cd "$module"
    unformatted="$(gofmt -l .)"
    if [ -n "$unformatted" ]; then
      echo "casework-go-check: gofmt would rewrite:" >&2
      printf '%s\n' "$unformatted" >&2
      exit 1
    fi
    go vet ./...
    go test ./...
    echo "casework-go-check: format, vet and tests green"

# GATE_UI, activated by T03: frozen install, typecheck, production build and tests for the cognitive
# UI app. The plan names `just casework-ui-check`; it must observe the real app, never a stub.
[group('casework')]
casework-ui-check:
    #!/usr/bin/env bash
    {{set}}
    app="apps/godspeed-cognitive-ui"
    if [ ! -f "$app/package.json" ]; then
      echo "casework-ui-check: no app at $app" >&2
      exit 1
    fi
    if ! command -v bun >/dev/null 2>&1; then
      echo "casework-ui-check: bun is required (mise declares it in mise.toml)" >&2
      exit 1
    fi
    cd "$app"
    bun install --frozen-lockfile
    bun run typecheck
    bun run build
    bun test
    echo "casework-ui-check: frozen install, typecheck, build and tests green"

# Operator controls for the fixture-backed cognitive UI (T03). The dev server binds a fixed port
# (default 4178, strict) so the printed URL is the served URL. `up` is idempotent: if something
# already answers on the port it says so and reuses it. Detached via setsid; pidfile and log live
# under /.sea-forge/ (gitignored).
casework_ui_port := "4178"

[group('casework')]
casework-ui-up port=casework_ui_port:
    #!/usr/bin/env bash
    {{set}}
    app="apps/godspeed-cognitive-ui"
    url="http://127.0.0.1:{{port}}"
    run_dir=".sea-forge/casework-ui"
    pidfile="$run_dir/ui.pid"
    logfile="$run_dir/ui.log"
    mkdir -p "$run_dir"
    if curl -sSf -o /dev/null --max-time 2 "$url" 2>/dev/null; then
      echo "casework-ui-up: something already answers at $url - reusing it (log: $logfile)"
      exit 0
    fi
    if [ -f "$pidfile" ]; then
      stale="$(cat "$pidfile")"
      if kill -0 "$stale" 2>/dev/null; then
        echo "casework-ui-up: pid $stale is alive but nothing answers on the port; stopping it" >&2
        kill -TERM -- -"$stale" 2>/dev/null || kill -TERM "$stale" 2>/dev/null || true
        sleep 1
      fi
      rm -f "$pidfile"
    fi
    if [ ! -d "$app/node_modules" ]; then
      echo "casework-ui-up: installing dependencies (frozen lockfile)"
      (cd "$app" && bun install --frozen-lockfile)
    fi
    cd "$app"
    setsid nohup bun run dev --port "{{port}}" --strictPort >"../../$logfile" 2>&1 &
    pid=$!
    echo "$pid" > "../../$pidfile"
    cd ../..
    for i in $(seq 1 50); do
      if curl -sSf -o /dev/null --max-time 1 "$url" 2>/dev/null; then
        echo "casework-ui-up: serving $url (pid $pid)"
        echo "casework-ui-up: log: $logfile"
        exit 0
      fi
      if ! kill -0 "$pid" 2>/dev/null; then
        echo "casework-ui-up: dev server exited during startup; log tail:" >&2
        tail -20 "$logfile" >&2 || true
        rm -f "$pidfile"
        exit 1
      fi
      sleep 0.2
    done
    echo "casework-ui-up: nothing answered $url within 10s; log tail:" >&2
    tail -20 "$logfile" >&2 || true
    exit 1

# Stop the dev server started by `casework-ui-up`. Idempotent; refuses to kill a pid that is not the
# UI (guards against pid reuse after a reboot).
[group('casework')]
casework-ui-down:
    #!/usr/bin/env bash
    {{set}}
    run_dir=".sea-forge/casework-ui"
    pidfile="$run_dir/ui.pid"
    if [ ! -f "$pidfile" ]; then
      echo "casework-ui-down: not running (no pidfile)"
      exit 0
    fi
    pid="$(cat "$pidfile")"
    if ! kill -0 "$pid" 2>/dev/null; then
      rm -f "$pidfile"
      echo "casework-ui-down: not running (stale pidfile removed)"
      exit 0
    fi
    if ! tr '\0' ' ' <"/proc/$pid/cmdline" 2>/dev/null | grep -qiE 'bun|vite|node'; then
      echo "casework-ui-down: pid $pid does not look like the casework UI; not killing it" >&2
      rm -f "$pidfile"
      exit 1
    fi
    kill -TERM -- -"$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true
    for i in $(seq 1 25); do
      kill -0 "$pid" 2>/dev/null || break
      sleep 0.2
    done
    if kill -0 "$pid" 2>/dev/null; then
      kill -KILL -- -"$pid" 2>/dev/null || kill -KILL "$pid" 2>/dev/null || true
    fi
    rm -f "$pidfile"
    echo "casework-ui-down: stopped (pid $pid)"

# Live-stack browser ladder (plan T10): the harness owns the stack end to end. It builds the
# production UI, scans the bundle for fixture adapters, seeds a FRESH temp cell (never the shared
# .sea-forge/casework-live cell), starts kernel + gateway (gateway serves dist), runs L0.. through
# agent-browser asserting UI AND durable files, then stops both and preserves evidence under
# .agents/evidence/casework-live-wiring/T10/run-<timestamp>/ (latest -> that run). Needs ports 4179
# (and 4180 for the stub-gateway tooth) free, a built kernel (cargo build -p sea-forge-server) and
# go. Pass extra flags through, e.g. `just casework-e2e-live --only L0,L1` or
# `just casework-e2e-live --tooth stub-gateway` (negative self-check; passes only if L1 fails
# at its durable assertion).
[group('casework')]
casework-e2e-live *args:
    #!/usr/bin/env bash
    {{set}}
    app="apps/godspeed-cognitive-ui"
    if ! command -v bun >/dev/null 2>&1; then
      echo "casework-e2e-live: bun is required (mise declares it in mise.toml)" >&2
      exit 1
    fi
    if ! command -v agent-browser >/dev/null 2>&1; then
      echo "casework-e2e-live: agent-browser is required" >&2
      exit 1
    fi
    if [ ! -d "$app/node_modules" ]; then
      echo "casework-e2e-live: installing dependencies (frozen lockfile)"
      (cd "$app" && bun install --frozen-lockfile)
    fi
    cd "$app"
    bun e2e/run.ts --live {{args}}

# Load test and restart teeth (plan T11): a FRESH temp cell per test, the real kernel and the real
# gateway binary as separate processes, N dev-auth users holding N concurrent SSE streams
# (LOAD_CLIENTS, default 50) while a burst of intents (LOAD_INTENTS, default 100, spread over
# LOAD_CASES, default 20, cases) runs. Measures intent latency, kernel-append -> client-receipt lag,
# error rate and gateway RSS / goroutines / fds / CPU, and fails (exit 1) on any budget violation
# (budgets: apps/godspeed-casework-go/internal/loadtest/budgets_test.go, documented in
# .agents/reports/casework-live-wiring/load-budgets.md). Also kill -9s the kernel, and then the
# gateway, mid-burst under 50 streams and asserts every client resyncs with no duplicate or
# out-of-order revision ids, no lost acknowledged write and bounded memory. Evidence (JSON with the
# RSS series, gateway and kernel logs, go test output) lands in
# .agents/evidence/casework-live-wiring/T11/run-<timestamp>/ (latest -> that run). Needs ~1 GB free
# RAM and no other heavy jobs (the latency numbers are CPU-bound by the kernel), a built kernel
# (cargo build -p sea-forge-server --bin sea-forge-server) and go. Extra args go to `go test`, e.g.
# `just casework-load -run TestLoadBudgets`.
[group('casework')]
casework-load *args:
    #!/usr/bin/env bash
    {{set}}
    module="apps/godspeed-casework-go"
    if ! command -v go >/dev/null 2>&1; then
      echo "casework-load: the Go toolchain is required (mise declares it in mise.toml)" >&2
      exit 1
    fi
    if [ ! -x target/debug/sea-forge-server ]; then
      echo "casework-load: building the kernel (cargo build -p sea-forge-server --bin sea-forge-server)"
      cargo build -p sea-forge-server --bin sea-forge-server
    fi
    stamp="$(date -u +%Y%m%dT%H%M%SZ)"
    evidence_root="$PWD/.agents/evidence/casework-live-wiring/T11"
    evidence="$evidence_root/run-$stamp"
    mkdir -p "$evidence"
    echo "casework-load: evidence -> $evidence"
    status=0
    (cd "$module" && LOAD_RUN_LABEL="$stamp" LOAD_EVIDENCE_DIR="$evidence" \
      go test -tags load -p 1 -count=1 -timeout 20m -v ./internal/loadtest/ {{args}}) \
      2>&1 | grep -v '^livetest-relay: ' | tee "$evidence/go-test.log" || status=$?
    # pipefail makes the pipeline status the test's, not tee's.
    ln -sfn "run-$stamp" "$evidence_root/latest"
    if [ "$status" -ne 0 ]; then
      echo "casework-load: FAILED (budget or invariant violation; see $evidence/go-test.log)" >&2
      exit 1
    fi
    echo "casework-load: all budgets and restart teeth passed"

# Report up/down, the URL, the pid, and the last log lines — the first thing to run when the UI
# looks wrong.
[group('casework')]
casework-ui-status port=casework_ui_port:
    #!/usr/bin/env bash
    {{set}}
    url="http://127.0.0.1:{{port}}"
    run_dir=".sea-forge/casework-ui"
    pidfile="$run_dir/ui.pid"
    logfile="$run_dir/ui.log"
    if [ -f "$pidfile" ] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
      echo "casework-ui-status: up (pid $(cat "$pidfile"))"
    else
      echo "casework-ui-status: down (no live pidfile)"
    fi
    if curl -sSf -o /dev/null --max-time 2 "$url" 2>/dev/null; then
      echo "casework-ui-status: answering at $url"
    else
      echo "casework-ui-status: nothing answers at $url"
    fi
    if [ -f "$logfile" ]; then
      echo "casework-ui-status: last log lines ($logfile):"
      tail -5 "$logfile" | sed 's/^/  /'
    fi

# Operator controls for the Go casework boundary (serve mode, fixture-labeled provider). Mirrors
# the cognitive UI recipe discipline: fixed loopback address, idempotent up, pidfile + log under
# .sea-forge/casework-go/ (gitignored), down refuses to kill a foreign pid.
casework_go_addr := "127.0.0.1:4179"

# Build and start the Go casework boundary server in the background (default 127.0.0.1:4179).
[group('casework')]
casework-go-up addr=casework_go_addr:
    #!/usr/bin/env bash
    {{set}}
    addr="{{addr}}"
    module="apps/godspeed-casework-go"
    run_dir=".sea-forge/casework-go"
    pidfile="$run_dir/server.pid"
    logfile="$run_dir/server.log"
    bin="$run_dir/server-bin"
    health="http://127.0.0.1:${addr#*:}/api/healthz"
    mkdir -p "$run_dir"
    if curl -sSf -o /dev/null --max-time 2 "$health" 2>/dev/null; then
      echo "casework-go-up: something already answers at $health - reusing it (log: $logfile)"
      exit 0
    fi
    if [ -f "$pidfile" ]; then
      pid="$(cat "$pidfile")"
      if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        echo "casework-go-up: pid $pid is alive but nothing answers on the addr; stopping it" >&2
        kill "$pid" 2>/dev/null || true
        sleep 1
      fi
      rm -f "$pidfile"
    fi
    echo "casework-go-up: building server binary (fixture stack builds with -tags casework_fixture; the default build cannot select fixtures)"
    (cd "$module" && go build -tags casework_fixture -o "../../$bin" ./cmd/godspeed-casework) || exit 1
    echo "casework-go-up: starting (fixture-labeled provider) on $addr"
    setsid nohup "$bin" -serve -addr "$addr" -config "$module/configs/fixture-serve.json" >>"$logfile" 2>&1 &
    echo $! > "$pidfile"
    for _ in $(seq 1 40); do
      if curl -sSf -o /dev/null --max-time 1 "$health" 2>/dev/null; then
        echo "casework-go-up: answering at $health (pid $(cat "$pidfile"), log: $logfile)"
        exit 0
      fi
      sleep 0.5
    done
    echo "casework-go-up: server did not become healthy; see $logfile" >&2
    exit 1

# Stop the Go casework boundary server.
[group('casework')]
casework-go-down addr=casework_go_addr:
    #!/usr/bin/env bash
    {{set}}
    addr="{{addr}}"
    pidfile=".sea-forge/casework-go/server.pid"
    if [ ! -f "$pidfile" ]; then
      echo "casework-go-down: no pidfile (nothing to stop)"
      exit 0
    fi
    pid="$(cat "$pidfile")"
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      if ! grep -aq "godspeed-casework" "/proc/$pid/cmdline" 2>/dev/null; then
        echo "casework-go-down: pid $pid is not the casework server; refusing to kill it" >&2
        exit 1
      fi
      kill "$pid" 2>/dev/null || true
      sleep 1
      kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
      echo "casework-go-down: stopped $pid"
    else
      echo "casework-go-down: pid $pid is not running"
    fi
    rm -f "$pidfile"

# Report up/down status, health endpoint, pid, and logs for the Go casework boundary server.
[group('casework')]
casework-go-status addr=casework_go_addr:
    #!/usr/bin/env bash
    {{set}}
    addr="{{addr}}"
    pidfile=".sea-forge/casework-go/server.pid"
    logfile=".sea-forge/casework-go/server.log"
    health="http://127.0.0.1:${addr#*:}/api/healthz"
    if [ -f "$pidfile" ] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
      echo "casework-go-status: up (pid $(cat "$pidfile"))"
    else
      echo "casework-go-status: down"
    fi
    if curl -sSf --max-time 2 "$health" 2>/dev/null; then
      echo ""
      echo "casework-go-status: answering at $health"
    else
      echo "casework-go-status: nothing answers at $health"
    fi
    if [ -f "$logfile" ]; then
      echo "casework-go-status: last log lines ($logfile):"
      tail -5 "$logfile" | sed 's/^/  /'
    fi

# Start both Go casework boundary and cognitive UI dev servers for the full demo experience.
[group('casework')]
casework-demo-up: casework-go-up casework-ui-up
    @echo "casework-demo-up: cognitive environment at http://127.0.0.1:4178 (go boundary at {{casework_go_addr}})"

# --- casework live-stack wiring (plan: .agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml) ---
# Operator controls for the REAL live stack: the Rust sea-forge-server kernel plus the Go
# godspeed-casework gateway, both against one durable cell under .sea-forge/casework-live/
# (gitignored). The fixture-stack recipes above (casework-go-up/down/status, casework-demo-up)
# are untouched; the UI dev-server recipes (casework-ui-up/down/status) are shared by both
# stacks (vite on :4178 proxies /api to the gateway addr).
#
# Note: the live gateway talks to the real kernel over the SFWP unix socket (provenance
# go:live:sfwp). The `up` recipes have teeth: a second server or gateway start exits non-zero
# instead of silently reusing or double-starting.
casework_live_dir := ".sea-forge/casework-live"

# Initialize the live cell (idempotent): create .sea-forge/casework-live/cell and, on first run,
# the T03 seed — server.yaml with the invoking OS user bound to the operator actor, the E2E plan
# templates from fixtures/cells/e2e/templates/, and the E2E authority policy at
# authority/active-policy.json. Gateway/security-officer/lifecycle-custodian bindings are still
# absent (they arrive with T02). An existing server.yaml is never modified: on a re-run the recipe
# only prints what is installed.
[group('casework')]
casework-cell-init:
    #!/usr/bin/env bash
    {{set}}
    live_dir="{{casework_live_dir}}"
    cell="$live_dir/cell"
    fixtures="fixtures/cells/e2e"
    mkdir -p "$live_dir" "$cell"
    if [ -f "$cell/server.yaml" ]; then
      echo "casework-cell-init: cell already initialized at $cell (server.yaml present; not modified)"
      echo "casework-cell-init: installed templates: $(ls "$cell/templates"/*.yaml 2>/dev/null | xargs -r -n1 basename | tr '\n' ' ')"
      echo "casework-cell-init: active policy: $([ -f "$cell/authority/active-policy.json" ] && echo "$cell/authority/active-policy.json" || echo ABSENT)"
      echo "casework-cell-init: identity bindings in server.yaml: $(grep -c 'actor_id:' "$cell/server.yaml" 2>/dev/null || echo 0)"
      echo "casework-cell-init: cell path: $(cd "$cell" && pwd)"
      exit 0
    fi
    uid="$(id -u)"
    cat > "$cell/server.yaml" <<YAML
    # Live-stack cell (plan casework-live-wiring-production T00/T03/T02).
    # One uid maps to exactly one actor (SF-005 follow-up). The invoking OS
    # user (uid $uid) is the gateway principal (T02, decision D-2); the end-user
    # actors have no OS principal of their own, so they are bound without a uid
    # and reached only through the gateway's `on_behalf_of`, bounded by the
    # allowlist below.
    identity:
      bindings:
        - uid: $uid
          actor_id: gateway
          roles: ["service"]
        - actor_id: operator_local
          roles: ["operator"]
        - actor_id: security_officer
          roles: ["R-SO"]
        - actor_id: lifecycle_custodian
          roles: ["R-LC"]
    gateway:
      uid: $uid
      actor: gateway
      delegable_actors: [operator_local, security_officer, lifecycle_custodian]
    YAML
    mkdir -p "$cell/templates" "$cell/authority"
    cp "$fixtures"/templates/*.yaml "$cell/templates/"
    cp "$fixtures/policy.yaml" "$cell/authority/active-policy.json"
    echo "casework-cell-init: wrote $cell/server.yaml (uid $uid is the T02 gateway principal; operator + security-officer + lifecycle-custodian are uid-less, reached via on_behalf_of)"
    echo "casework-cell-init: delegable actors: operator_local, security_officer, lifecycle_custodian"
    echo "casework-cell-init: installed templates: $(ls "$cell/templates"/*.yaml | xargs -n1 basename | tr '\n' ' ')"
    echo "casework-cell-init: wrote $cell/authority/active-policy.json (E2E policy: write_file + approval_resolution for operator)"
    echo "casework-cell-init: cell path: $(cd "$cell" && pwd)"

# Build and start the sea-forge-server kernel against the live cell, configured only via
# SEA_FORGE_ROOT/SEA_FORGE_SOCKET plus <cell>/server.yaml (the server takes no CLI arguments).
# Teeth: refuses to start when our pidfile is live, or when the cell socket already exists.
[group('casework')]
casework-server-up:
    #!/usr/bin/env bash
    {{set}}
    live_dir="{{casework_live_dir}}"
    cell="$live_dir/cell"
    pidfile="$live_dir/server.pid"
    logfile="$live_dir/server.log"
    socket="$cell/server.sock"
    mkdir -p "$live_dir"
    if [ -f "$pidfile" ]; then
      pid="$(cat "$pidfile")"
      if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null && grep -aq "sea-forge-server" "/proc/$pid/cmdline" 2>/dev/null; then
        echo "casework-server-up: server already running (pid $pid, socket $socket)" >&2
        exit 1
      fi
    fi
    if [ -S "$socket" ]; then
      echo "casework-server-up: socket $socket already exists; if no server owns it, remove it manually" >&2
      exit 1
    fi
    if [ -f "$pidfile" ]; then
      pid="$(cat "$pidfile")"
      echo "casework-server-up: removing stale pidfile (pid ${pid:-empty} not alive or not the server)"
      rm -f "$pidfile"
    fi
    echo "casework-server-up: building sea-forge-server (waits politely if another cargo holds the target lock)"
    cargo build --locked -p sea-forge-server --bin sea-forge-server
    echo "casework-server-up: starting server (cell $cell, socket $socket, log: $logfile)"
    SEA_FORGE_ROOT="$cell" SEA_FORGE_SOCKET="$socket" setsid nohup target/debug/sea-forge-server >>"$logfile" 2>&1 &
    echo $! >"$pidfile"
    for _ in $(seq 1 120); do
      if [ -S "$socket" ]; then
        echo "casework-server-up: listening (pid $(cat "$pidfile"), socket $socket, log: $logfile)"
        exit 0
      fi
      pid="$(cat "$pidfile")"
      if ! kill -0 "$pid" 2>/dev/null; then
        echo "casework-server-up: server exited during startup; log tail:" >&2
        tail -20 "$logfile" >&2 || true
        rm -f "$pidfile"
        exit 1
      fi
      sleep 0.5
    done
    echo "casework-server-up: socket $socket did not appear within 60s; log tail:" >&2
    tail -20 "$logfile" >&2 || true
    if ! kill -0 "$(cat "$pidfile")" 2>/dev/null; then
      rm -f "$pidfile"
    fi
    exit 1

# Stop the sea-forge-server kernel. Mirrors casework-go-down discipline: idempotent, refuses to
# kill a pid that is not the server, removes a lingering socket only once no server owns it.
# Cell data under the live dir is never touched.
[group('casework')]
casework-server-down:
    #!/usr/bin/env bash
    {{set}}
    live_dir="{{casework_live_dir}}"
    pidfile="$live_dir/server.pid"
    socket="$live_dir/cell/server.sock"
    if [ ! -f "$pidfile" ]; then
      echo "casework-server-down: not running (no pidfile)"
    else
      pid="$(cat "$pidfile")"
      if ! kill -0 "$pid" 2>/dev/null; then
        rm -f "$pidfile"
        echo "casework-server-down: not running (stale pidfile removed)"
      elif ! grep -aq "sea-forge-server" "/proc/$pid/cmdline" 2>/dev/null; then
        echo "casework-server-down: pid $pid does not look like sea-forge-server; not killing it" >&2
        rm -f "$pidfile"
        exit 1
      else
        kill -TERM "$pid" 2>/dev/null || true
        for _ in $(seq 1 25); do
          kill -0 "$pid" 2>/dev/null || break
          sleep 0.2
        done
        if kill -0 "$pid" 2>/dev/null; then
          kill -KILL "$pid" 2>/dev/null || true
        fi
        rm -f "$pidfile"
        echo "casework-server-down: stopped (pid $pid)"
      fi
    fi
    # The server removes its own socket on shutdown; clear a lingering one only when no live
    # server owns it. The server holds an exclusive flock on server.sock.lock for its lifetime,
    # so a lock we can take proves the owner is gone — no cmdline string-matching heuristics.
    if [ -S "$socket" ]; then
      if flock -n "$socket.lock" true 2>/dev/null; then
        rm -f "$socket"
        echo "casework-server-down: removed lingering socket $socket (socket lock free; no server owns it)"
      else
        echo "casework-server-down: socket $socket is still owned by a live server (socket lock held); leaving it"
      fi
    fi

# Build and start the Go gateway in serve mode against the LIVE cell (default 127.0.0.1:4179).
# Like casework-go-up, but pointed at apps/godspeed-casework-go/configs/live-serve.json with the
# cell injected via GODSPEED_CELL_ROOT (the file deliberately omits cell_root so the temp cell
# location stays a recipe-level decision). Until T05 the gateway serves FIXTURE-LABELED
# in-process providers against live cell roots. Teeth: refuses to silently reuse a foreign
# process that already answers /api/healthz (e.g. the fixture casework-go-up).
[group('casework')]
casework-live-go-up addr="127.0.0.1:4179":
    #!/usr/bin/env bash
    {{set}}
    addr="{{addr}}"
    module="apps/godspeed-casework-go"
    live_dir="{{casework_live_dir}}"
    cell="$live_dir/cell"
    pidfile="$live_dir/go.pid"
    logfile="$live_dir/go.log"
    bin="$live_dir/go-bin"
    health="http://127.0.0.1:${addr#*:}/api/healthz"
    mkdir -p "$live_dir"
    if [ ! -f "$cell/server.yaml" ]; then
      echo "casework-live-go-up: no cell at $cell - run just casework-cell-init first" >&2
      exit 1
    fi
    if curl -sSf -o /dev/null --max-time 2 "$health" 2>/dev/null; then
      if [ -f "$pidfile" ] && kill -0 "$(cat "$pidfile")" 2>/dev/null \
        && grep -aq "casework-live/go-bin" "/proc/$(cat "$pidfile")/cmdline" 2>/dev/null; then
        echo "casework-live-go-up: our gateway already answers at $health - reusing it (log: $logfile)"
        exit 0
      fi
      echo "casework-live-go-up: a foreign process is answering /api/healthz at $addr (possibly the fixture casework-go-up); stop it first (just casework-go-down)" >&2
      exit 1
    fi
    if [ -f "$pidfile" ]; then
      pid="$(cat "$pidfile")"
      if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        echo "casework-live-go-up: pid $pid is alive but nothing answers on the addr; stopping it" >&2
        kill "$pid" 2>/dev/null || true
        sleep 1
      fi
      rm -f "$pidfile"
    fi
    echo "casework-live-go-up: building gateway binary"
    (cd "$module" && go build -o "../../$bin" ./cmd/godspeed-casework) || exit 1
    echo "casework-live-go-up: starting LIVE gateway on $addr against live cell $cell (provenance go:live:sfwp, T06)"
    GODSPEED_CELL_ROOT="$cell" setsid nohup "$bin" -serve -addr "$addr" -config "$module/configs/live-serve.json" >>"$logfile" 2>&1 &
    echo $! >"$pidfile"
    for _ in $(seq 1 40); do
      if curl -sSf -o /dev/null --max-time 1 "$health" 2>/dev/null; then
        echo "casework-live-go-up: answering at $health (pid $(cat "$pidfile"), log: $logfile)"
        exit 0
      fi
      sleep 0.5
    done
    echo "casework-live-go-up: gateway did not become healthy; see $logfile" >&2
    tail -20 "$logfile" >&2 || true
    exit 1

# Stop the live Go gateway. Mirrors casework-go-down with the live pidfile and a cmdline guard
# that recognizes only the casework-live go-bin (never the fixture gateway's binary).
[group('casework')]
casework-live-go-down addr="127.0.0.1:4179":
    #!/usr/bin/env bash
    {{set}}
    pidfile="{{casework_live_dir}}/go.pid"
    if [ ! -f "$pidfile" ]; then
      echo "casework-live-go-down: no pidfile (nothing to stop)"
      exit 0
    fi
    pid="$(cat "$pidfile")"
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      if ! grep -aq "casework-live/go-bin" "/proc/$pid/cmdline" 2>/dev/null; then
        echo "casework-live-go-down: pid $pid is not the live casework gateway; refusing to kill it" >&2
        exit 1
      fi
      kill "$pid" 2>/dev/null || true
      sleep 1
      kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
      echo "casework-live-go-down: stopped $pid"
    else
      echo "casework-live-go-down: pid ${pid:-empty} is not running"
    fi
    rm -f "$pidfile"

# Tear down the whole live stack: gateway first, then the kernel. Processes only — the cell
# data at .sea-forge/casework-live/cell (ledgers, requests, server.yaml) is always preserved.
[group('casework')]
casework-stack-down:
    #!/usr/bin/env bash
    {{set}}
    live_dir="{{casework_live_dir}}"
    set +e
    just casework-live-go-down
    go_rc=$?
    just casework-server-down
    server_rc=$?
    set -e
    if [ "$go_rc" -eq 0 ]; then
      echo "casework-stack-down: casework-live-go-down ok"
    else
      echo "casework-stack-down: casework-live-go-down FAILED (exit $go_rc)" >&2
    fi
    if [ "$server_rc" -eq 0 ]; then
      echo "casework-stack-down: casework-server-down ok"
    else
      echo "casework-stack-down: casework-server-down FAILED (exit $server_rc)" >&2
    fi
    echo "casework-stack-down: cell data preserved at $live_dir/cell (teardown is processes-only, never data)"
    if [ "$go_rc" -ne 0 ] || [ "$server_rc" -ne 0 ]; then
      exit 1
    fi


# Consistent offline backup of the live cell (T11). The kernel must be stopped
# (just casework-server-down): the script refuses a live cell and holds the cell lock while
# copying. Usage: just casework-cell-backup /path/to/cell.tar.gz
[group('casework')]
casework-cell-backup archive:
    scripts/casework-cell-backup.sh "{{casework_live_dir}}/cell" "{{archive}}"

# Restore a backup into a NEW or empty directory (never deletes). Verifies the archive digest and
# the per-file manifest. Usage: just casework-cell-restore /path/to/cell.tar.gz /path/to/new-cell
[group('casework')]
casework-cell-restore archive target:
    scripts/casework-cell-restore.sh "{{archive}}" "{{target}}"

# One pinned world from GodSpeed-Agent E1 to the evidence it records, across real production surfaces in five
# repos, plus the refusal paths. Needs sibling checkouts (override with *_ROOT) and the domainforge CLI.
[group('quality')]
e2e-world-loop:
    ./scripts/e2e-world-loop.sh

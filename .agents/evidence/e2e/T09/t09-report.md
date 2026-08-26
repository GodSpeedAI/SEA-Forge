# T09 Report — Settle Developmental Consequence and Durable Memory (E9/I9)

Task: sea-rs `.agents/plans/e2e-plan.yml` `T09:` ("Settle Developmental
Consequence and Durable Memory"). Settles **E9** and **I9**, proof_level P3,
confirmation `independent_adversarial`. This is the BUILDER deliverable; it
does NOT mark anything settled in operational status projections — independent
adversarial confirmation is a separate role.

## What was built

Both sides of edge E9 live in `/home/sprime01/projects/godspeed_agent`
(Python 3.11+). All changes are additive; every refusal happens BEFORE any
durable write.

### 1. Canonical E9 emission (producer side)

| File | Change |
| --- | --- |
| `godspeed_nav/developmental_events.py` | NEW. Stateless projection of explicit GSA developmental decisions into the canonical v1 envelope family (`schema_version: v1`, same wire shape as T03 `canonical_events.py` / T08 `evidence_ingest.py`). Covers ALL FIVE event types (`SettlementRecorded`, `CapabilityUpdated`, `RepetitionPlanned`, `LearningProposalCreated`, `CoherenceBreakDetected`) stamped by the exclusive producer (`source_agent`/`provenance.origin` = `godspeed-agent`; canonicalizes to `godspeed_agent`). Every envelope carries the four common required payload fields (`work_request_id`, `source_evidence_refs`, `developmental_decision`, `domain_model_ref`) plus per-type required extras (`EVENT_PAYLOAD_REQUIREMENTS`), cites its justifying EvidenceRecorded/provisional-evidence ids via `caused_by:` chain entries (≥1 well-formed parent REQUIRED), computes the producer idempotency key, and enforces identity gates (structural 64-hex digest; the fallback pseudo-hash `dc144cbd71a4…` and all-zero refused; malformed digests refused). Provenance VALUE SHAPES are validated at projection: empty lists, blank/whitespace entries, placeholder tokens are degenerate and refused before any envelope exists. `event_id`/`occurred_at` injection supports deterministic fixture regeneration. |
| `godspeed_nav/runtime.py` | TWO additive methods on `NavigationRuntime` (133 insertions, 0 deletions; pre-existing T08 hunks preserved): `emit_developmental_event(...)` composing projector → durable gate over the runtime's own `LedgerStore`, and `request_capability_promotion(...)` routing CapabilityUpdated promotion through the SAME gated surface. Lazy imports keep the module graph flat (T08 pattern). |

### 2. Durable memory gate (memory-ledger side)

| File | Change |
| --- | --- |
| `godspeed_nav/developmental_memory.py` | NEW. The durable write seam for E9: `ingest_developmental_event` gates each delivery fail-closed — exclusive-producer authority over all five types (forge attempts refused by name; provenance origin must agree with the stamp), identity gates (placeholder/drift/namespace/ref-consistency), four-common-field presence battery reported together, then typed value-shape validation (`provenance_stripped` quarantine semantics for missing/empty/stripped `source_evidence_refs`; degenerate decision objects, repetition counts, and out-of-range settlement scores refused), causality presence with optional out-of-band `known_parent_ids` pinning (`forged_parent_reference`). Duplicate recognition BEFORE any consequence under stable envelope identity (event id, content digest, producer idempotency key); replay-with-mutation under stable identity is refused BY NAME (`envelope_identity_conflict`) and the first record stands. **I9 promotion gate**: CapabilityUpdated must cite recorded settlements OF THE SAME capability; `metabolized` composes the runtime's existing machinery — `Capability.from_settlements` (<2 qualifying successes ⇒ `capability_requires_repeated_settlement`), declared-variation substance (`undeclared_variation`, `variation_not_demonstrated`; inline declarations in the request are ignored so post-hoc gaming fails), then `compute_metabolization` + `infer_capability_lifecycle` must actually yield `metabolized` (else `i9_not_earned`). Statuses BELOW metabolized keep explicitly declared lower thresholds (`active` ≥1 success, `repeated` via `from_settlements`' ≥2 bar) — the frozen known confound is preserved, not collapsed. Persistence goes through `LedgerStore.append_event_if_absent(dedup_field="event_id")` on the append-only flock-guarded `developmental_events` ledger, so even a race between two first-admissions stores exactly one row. Restart survival: `load_developmental_memory_state` REPLAYS the durable ledger through this same gate, rebuilding dedup identities, recorded settlements, and earned promotions from durable truth; rows failing revalidation are quarantined OUT of history, never trusted. |

### 3. Cross-language/cross-surface evidence

| File | Change |
| --- | --- |
| `tests/fixtures/t09_developmental_events.json` | NEW committed golden fixture produced by the REAL emitter: five envelopes (one per type, deterministic uuid5 ids + fixed timestamps), fixed model hash, pinned parent ids. |
| `tests/test_convergence_t09_developmental_memory.py` | NEW. 94 pytest tests consuming the committed fixture through the REAL durable gate (sequential multi-envelope flow: settlement first so the promotion's citations resolve) plus a regeneration test (`GSA_T09_REGEN_FIXTURE=1` rewrites; default run proves committed fixture == real-emitter output exactly). |

### 4. Gate binding

| File | Change |
| --- | --- |
| `/home/sprime01/projects/sea-rs/justfile` | Added `T09)` arm to the `e2e-gate` case (mirrors the T03/T08 arms): runs the new pytest file via `uv run --project "${GODSPEED_AGENT_ROOT:-$HOME/projects/godspeed_agent}" --extra dev python -m pytest … -q`. justfile carried pre-existing modifications; only the arm was appended. |

Other repos: untouched (read-only recon of `agentic_capability_loop/publisher.py`,
`learning_proposals.py`, `hassos-addon-agent-memory-ledger/`). The existing
NATS publisher is composed with, not replaced: drained envelopes reach durable
history only through this gate class (offline-drain surface tested).

## Falsifier → proof map

Plan falsifier: *"One successful work cycle promotes metabolized capability."*
- Proof: `test_tooth_1_pristine_single_success_cannot_promote_metabolized` —
  one pristine successful cycle admitted, then a metabolized promotion request
  is refused (`capability_requires_repeated_settlement` from the existing
  `Capability.from_settlements` machinery) with ZERO capability history
  written. Variants: two successes under one identical condition
  (`variation_not_demonstrated`); three successes without declared contexts
  (`undeclared_variation`); low-scoring successes below the 0.7 capability
  bar; cross-capability citation (`unresolved_capability_evidence`); inline
  post-hoc "declared variations" inside the request are ignored. Promotion
  attempts via EVERY reachable surface hit the same gate: runtime method,
  direct gate/bridge-route call, publisher-path ingestion, and a raw ledger
  row bypassing the gate (quarantined on replay, never counted as history).

Plan falsifier: *"A developmental event loses the evidence refs that
justified it."*
- Proof: `test_tooth_3_stripped_source_evidence_refs_are_quarantined` — eight
  stripped/degenerate variants (absent, None, empty list, blank entry,
  whitespace entry, placeholder token, non-list, null entry) raise
  `incomplete_developmental_event`/`provenance_stripped` and leave the
  durable ledger file NONEXISTENT — quarantine means never persisted as
  authoritative developmental truth. Companions: stripped `caused_by` chain ⇒
  `causality_missing`; forged parents against pinned ids ⇒
  `forged_parent_reference`; admitted records proven to retain
  `source_evidence_refs` and causal citations verbatim in durable storage.

Preregistration claim *"Developmental settlement is independently earned."*
- Proof: projection refuses to exist without a non-empty GSA
  `developmental_decision` object; the T08 ingestion path still contains no
  promotion code whatsoever; canonical settlements coexist with (never
  bypass) the explicit `record_settlement` action
  (`test_canonical_layer_coexists_with_record_settlement_machinery`).

Preregistration claim *"Capability promotion requires its declared repeated-
settlement/variation evidence."*
- Proof: the ONLY admitted metabolized promotion in the suite is
  `test_borderline_three_varied_high_scoring_settlements_earn_metabolized`
  (exactly the composed machinery bar) and the runtime-surface earned path;
  `test_lower_status_thresholds_stay_explicitly_distinct` proves active(1)/
  repeated(2) remain valid while metabolized(2) is refused — distinctions
  preserved, not collapsed.

Confirmation condition (*fresh verifier cannot obtain metabolized capability
from a single success or create developmental records detached from
evidence*): every surface above refuses by named error code, and duplicate/
mutated replays cannot smuggle state in (`envelope_identity_conflict`,
idempotent redelivery with zero duplicate rows).

## Verification commands and results

Recorded BEFORE any edits:

```
HEADs:
  sea-rs         006daa2540678eaaf821d873266f79914b10c9af
  godspeed_agent eeee146387af1f98a04381be7c2517fb87da8d82
Dirty state: recorded per-repo and preserved byte-for-byte (GSA: M AGENTS.md,
M godspeed_nav/runtime.py [T08], untracked T03/T08 files; sea-rs: documented
pre-existing set incl. M justfile). No commits, pushes, or branch changes.
Preregistration: never opened for write; SHA verified unchanged.
```

| Check | Command | Result |
| --- | --- | --- |
| GSA baseline (before edits) | `uv run --extra dev python -m pytest -q --ignore=tests/e2e` | **5 failed, 650 passed** — exact documented pre-existing set (calibration_pipeline sklearn; cli_hooks_mcp ruvector ×2; distribution adapter; domain_storage ruvector). e2e dir: 1 collection error (`psycopg` missing), pre-existing. |
| GSA new tests | `uv run --extra dev python -m pytest tests/test_convergence_t09_developmental_memory.py -q` | **94 passed**, 0 failed (~2 s) |
| Golden fixture round-trip | same file WITHOUT regen env | passes — committed fixture == real emitter output |
| GSA FULL suite (after, final code) | `uv run --extra dev python -m pytest -q --ignore=tests/e2e` | **5 failed, 744 passed** — IDENTICAL failure set to baseline (delta is +94 collected T09 tests; zero new failures) |
| GSA e2e dir (after) | `pytest tests/e2e -q` | same single pre-existing `psycopg` collection error; nothing else |
| Additivity | `git diff godspeed_nav/runtime.py` | 133 insertions, 0 deletions; pre-existing dirty hunks intact |
| Gate | `just e2e-gate T09` (from sea-rs) | exit 0 (94 Python green) |
| Prereg | `just e2e-prereg-check` | **PASS**; SHA `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f` unchanged |

## Honest scope notes

- **T09 does NOT prove stored memory changes future navigation.** Nothing
  reads `developmental_events` back into horizon review, affordance ranking,
  or trajectory logic. That is E10 / task T10 scope (the plan's
  `does_not_prove` says exactly this).
- **Durability boundary:** "restart" in these tests means process-level
  reopen of the SAME local `LedgerStore` root — fresh gate instances that
  rebuild state by replaying the durable JSONL ledger through the gate. It
  does NOT exercise an external store restart: no Postgres/NATS process is
  started (offline-first harness; the NATS publisher is disabled-by-default
  and the Postgres-backed addon path is the pre-existing-broken `psycopg`
  environment). The IdempotencyLedger concept from SWE_SEED's T01 work is
  mirrored conceptually (stable-envelope-identity dedup + durable first-
  record-stands), not run against SWE_SEED's store.
- **"Declared relevant variation" enforcement is structural, not semantic:**
  the gate verifies PRE-declaration (contexts recorded on the settlements
  themselves) and DISTINCTNESS (≥2 contexts for metabolized), but cannot
  verify the variation was *relevant* to the capability — that would need a
  declared variation vocabulary, which does not exist yet.
- Settlement facts (`settlement_status`, `settlement_score`,
  `variation_context`) are GSA's own producer assertions at this seam. The
  gate composes them into lifecycle inference but does not re-derive them
  from upstream evidence bytes — consistent with frozen I8 (upstream evidence
  stays provisional; GSA evaluates independently), but it means the gate
  inherits the integrity of GSA's internal scoring.
- Filesystem-level forgery by an attacker with direct write access to the
  ledger (crafting checksum-valid rows outside the gate) is outside the
  contract surface, same trust placement as T08; defense-in-depth here is
  replay-through-the-gate, which quarantines any stored row that fails
  revalidation instead of trusting it.
- Pre-existing failures were neither fixed nor extended: the 5 documented
  env failures and the `psycopg` e2e collection error reproduce identically
  before and after.

## Debt ledger additions (observed, out of scope)

- **Pre-existing, not extended:** `agentic_capability_loop/adapters.py`
  `DOMAIN_MODEL_HASH` still falls back to the namespace pseudo-hash
  (`dc144cbd71a4…`) when the SEA manifest is absent — the exact constant the
  canonical identity gates forbid. The legacy emit path retains that
  trust-placement risk; the canonical T09 path refuses the hash outright.
  Mirrors T05-A9/T06/T08 minors.
- **New, small:** `variation_context` is free-form text; without a declared
  variation vocabulary, "relevance" of variation remains unverifiable
  structurally (see scope notes). Next move: define variation vocabulary when
  memory_ledger needs semantic checks.
- **New, small:** the deployed NATS→ledger bridge route should validate
  inbound envelopes with THIS gate class; offline drain tests prove the
  in-process behavior only. Live-NATS composition is untested (no broker in
  the harness environment).

# T02 — Cited Context Integration (E2, E3, I4) — builder evidence

Plan: `.agents/plans/e2e-plan.yml` task T02 (P2). Frozen target:
`.agents/specs/e2e-preregistration.yml` SHA
ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f (intact at
evidence time and after).

## What was built

The existing production SWE_SEED↔Context_Kernel MCP stdio slice was brought
under the canonical envelope/correlation contract adopted from T01. No new
transport; the real MCP path is reused.

Context_Kernel (`crates/ck-mcp`, sha2 wired from the existing workspace dep):

- Ingress gates in `handle_context_required`: request must BE a canonical v1
  envelope stamped by the exclusive authoritative producer (`swe_seed`);
  `verify_domain_identity` rejects malformed/all-zero/standalone-pseudo-hash
  identity fail-closed; work_request_id/context_requirement_id correlation is
  required. Corpus allowlist extraction reads BOTH wire shapes so the
  authorization arm cannot be bypassed by nesting (regression guard).
- Explicit acquisition outcomes (`AclContextAgent::acquire_context` +
  `NoContextReason`): missing corpus root/corpus/no-match are returned as
  machine-readable reasons — never silent empty success.
- Egress: `context_packet_envelope()` builds the full canonical v1
  ContextPacketCreated envelope stamped `context-kernel` with content-derived
  idempotency key and the E2 request event id recorded as causal parent
  (`caused_by:`).
- Response carries `{context_envelope, context_packet, outcome,
  reason}`; only `outcome="cited"` is successful acquisition;
  `no_context_governed` is the explicit governed outcome for required
  context; `empty_optional` keeps optional acquisitions legitimate.
- Authority fields remain pass-through references only (I4): CK computes no
  decision and emits none.

SWE_SEED (`crates/swe-seed-core/src/federation/context_client.rs` rewritten):

- `acquire_context(ContextRequest)` sends a REAL canonical E2 envelope built
  via `make_event_verified` (identity type-gated; swe_seed producer stamp) — a
  fallback pseudo-hash is unrepresentable at this call site.
- Pure adjudicator `adjudicate_context_response()` composes the T01 boundary
  gate (`validate_envelope`: producer authority + identity shape + drift +
  causality records), requires OUR E2 event id among `caused_by:` parents,
  rejects cross-wired work_request_id/context_requirement_id, and enforces the
  outcome protocol (`cited`⇒≥1 citation; required+zero ⇒
  `GovernedNoContext` error that callers must handle as non-success).
- Typed errors: `Transport | BoundaryRejected | CrossWired | CausalityBroken |
  GovernedNoContext | InvalidResponse`.

## Teeth → proof map

| Plan tooth / frozen falsifier | Proof |
| --- | --- |
| Unset/misconfigured corpus root + mandatory context ⇒ no successful zero-citation settlement | CK `t02_required_zero_citation_is_governed_outcome_never_silent_success`; live-binary governed-no-context case in `context_kernel_client.rs` |
| Return context for a different wr/cr id ⇒ consumer rejects cross-wired packet | `t02_cross_wired_work_request_is_rejected`, `t02_cross_wired_context_requirement_is_rejected` |
| Inject an authority decision originating from CK ⇒ it cannot become execution authority | `i4_ck_cannot_emit_authority_checked_even_with_correct_stamp` (+ CK pass-through-only tests); AuthorityChecked registry authority = sea_forge |
| Zero citations claiming success ⇒ protocol violation | `t02_zero_citations_claiming_success_is_protocol_violation` |
| Placeholder/fallback identity at either side | CK dispatch teeth (`t02_placeholder_identities_fail_closed_at_dispatch`), client `t02_placeholder_identity_in_packet_is_rejected` |
| Wrong producer on E2 ingress or E3 egress | CK `t02_non_authoritative_producer_fails_closed`; client boundary rejection of 4 forged stamps |
| Lost causal-parent identity | `t02_packet_not_citing_our_e2_request_is_rejected` |

## Verification performed

```
CK:   cargo test -p ck-mcp -p ck-bin        # 21 lib + 3 acl + 24 cli (real stdio served binary) — 0 failed
SEED: cargo test -p swe-seed-core           # 363 passed, 0 failed
LIVE: SWE_SEED_CONTEXT_KERNEL_BIN=<ck binary> cargo test -p swe-seed-core \
        --test context_kernel_client        # cited happy path + governed no-context over real MCP stdio
GATE: just e2e-gate T02                     # exit 0
fmt:  touched files clean; clippy clean on touched code
```

## Honest scope notes

- The pre-existing uncommitted spec-0020 WIP in SWE_SEED (gateway/, mcp-gate)
  was preserved untouched; the client refactor already staged there was
  extended additively toward this contract.
- GSA-side DesiredDirection consumption of E0 envelopes is T03/E0 scope.
- Adoption by any other consumer surfaces (if added later) must compose
  `adjudicate_context_response`/`validate_envelope` rather than raw payload
  reads.

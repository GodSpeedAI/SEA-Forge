# Corroborate evidence with RealityTrace

By default `authority_evidence` takes an evidence packet as stated. SEA-Forge checks its structure, its
world and that it hangs from an allow decision, but believes each item's `direction` and `reliability`.
The requester that asked for the operation can then vouch for its own work.

With a `realitytrace` block, SEA-Forge reads RealityTrace's ledger read-only and checks every item
against the `evidence.created` record it names. Settlement is then decided by what RealityTrace
recorded, not by what the requester submitted.

## Configure

```yaml
cep_authority:
  enabled: true
  # ...policy, worlds, settlement...
  realitytrace:
    bin: /usr/local/bin/realitytrace        # absolute
    data_home: /home/op/.local/share         # RealityTrace's XDG_DATA_HOME, absolute
    state_home: /home/op/.local/state        # RealityTrace's XDG_STATE_HOME, absolute
    require_corroboration: true              # default
```

## What is checked

For each packet item whose `source` is `realitytrace`:

1. The item names its ledger position in `provenance_refs` as `sxr:<ledger_id>:<append_ordinal>`. Cognate's
   `evidencePacketEnvelope` writes this.
2. A RealityTrace repo's ledger holds that position (`realitytrace repo list`, `ledger export`).
3. That repo's hash chain verifies (`realitytrace ledger verify`).
4. The record is `evidence.created`, and it records the item's `evidence_id` with a `direction` and a
   `reliability`.
5. The item's `evidence_id`, `evidence_type`, `question_ref`, `representation_ref`, `relevance`,
   `reliability` and `direction` equal the recorded values.

Any failure refuses the packet with `cep_evidence_refused` and commits no `evidence_packet`. The
`evidence_mutation` policy decision taken before the check stays in the ledger, as for any governed
request.

An accepted packet is committed with the subject ref `corroborated:realitytrace` only when every item
was checked, and the response then says `"corroboration": "realitytrace"`. A packet with any item taken
as stated (possible only without `require_corroboration`) says `"corroboration": "none"` and never
counts as corroborated, even if the requirement is turned on later.

With `require_corroboration: true`:

- an item from any other source is refused;
- `authority_settle` counts only packets committed as corroborated. A packet accepted before the
  requirement was turned on never decides a settlement.

## Limits

- RealityTrace's ledger is tamper-evident, not tamper-proof. Anyone with write access to its data
  home can rewrite the whole chain consistently. On a single-user host, the anchor is that agent code
  runs contained and cannot reach that directory (readiness gate S02).
- SEA-Forge invokes the configured binary with a cleared environment (`XDG_DATA_HOME`,
  `XDG_STATE_HOME`, `PATH`) and a 60 s timeout per call. It never writes to RealityTrace.

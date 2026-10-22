# How-To: Upgrade DomainForge Without Renaming Your Worlds

> **For operators who upgrade the pinned DomainForge (`domainforge-core`) and need the same model to stay the same governed world.**

---

## Goal

A DomainForge upgrade changes `producer_version`, which is part of every world's identity digest. The same source therefore mints a new `world_ref` after the upgrade. This procedure keeps the move auditable: you register the old identity as a *prior identity*, then record a `compiler_upgrade` transition from the old world to the new one. SEA-Forge allows that transition only when the meaning-bearing identity is unchanged.

---

## Configuration: `cep_authority.worlds[].prior_identities`

Each world may list up to 16 prior identities:

```yaml
cep_authority:
  enabled: true
  worlds:
    - name: demo
      base: worlds
      entry: demo.sea
      files: [demo.sea]
      prior_identities:
        - identities/demo.domainforge-0.21.0.json
```

- Each path is relative to the **cell root**, not to `base`. Absolute paths and `..` are refused at config load.
- Each file holds one `DomainModelIdentity` JSON document. It must be at most 64 KiB. Unknown keys are refused.
- Each prior identity is registered under the world's `name` as a prior. Its `world_ref` is recomputed from the document, so the ref always matches the document. A prior that equals a current world is refused.
- A malformed or unreadable prior identity fails closed. Every CEP request for that cell returns `error_class: cep_world_config`, and the message names the file.

**Trust basis.** Prior identities are host-owned configuration, trusted the same way as the world sources. The digest proves only that a document and its ref agree. It does not prove the document is authentic.

**Roles.** A prior identity is a place to move work *out of*. It is never a place to act:

- It may be the **source** of a `world_transition`. This is how a `compiler_upgrade` works.
- It may **not** be a transition target.
- It may **not** be the world of a governed action (`action` or `capability`). Those requests are refused with `cep_world_refused`. The conservative choice is to refuse rather than require the current world. The current world is the only world that can act, so a stale identity can never quietly authorize new work.
- Lineage citations are checked against the ledger, so earlier work decided in the old world still crosses only through an allowed transition.

---

## Procedure

1. **Export the current identity with the CURRENT DomainForge CLI, before you upgrade.** Do this once for each world. Run it from the world's `base` directory with its `entry`, so the logical URIs match the config:

   ```sh
   cd worlds && domainforge envelope --emit cep --world-name demo demo.sea \
     | jq '.extensions["domainforge.identity"].domain_model_identity' \
     > ../identities/demo.domainforge-0.21.0.json
   ```

   The identity is emitted at `extensions["domainforge.identity"].domain_model_identity` only when a world name is bound. This path was checked against the released 0.21.1 CLI on the fixture source. Its `scope.world_ref` was `world:demo@sha256:a71ff6...`, and its identity differed from the 0.21.0 export only in `producer_version` and `content_hash`. Run `jq -r .scope.world_ref` on the same output and confirm it matches the ref the current cell reports.

2. **List the file under `prior_identities`** for that world, as shown above. Commit it with the cell configuration.

3. **Upgrade** the pinned DomainForge in SEA-Forge (`domainforge-core`). Rebuild and restart the cell.

4. **Request the transition.** Send one `authority_request` with `operation_kind: world_transition`. Its `scope.world_ref` is the old ref. The extension `godspeed.world_transition` carries `target_world_ref` (the new ref) and `transition_kind: compiler_upgrade`. Policy decides, as for any meaning-preserving move. An allowed transition is recorded in the ledger, with both producer versions in `evidence_refs`.

5. **Cite it for lineage.** Work that continues in the new world cites the earlier work and the transition's ledger entry as `transition_ref`.

### What `compiler_upgrade` checks

SEA-Forge recomputes these from the two registered identities. Each must be equal, or both absent:

`semantic_closure_hash`, `source_set_hash`, `semantic_pack_set_hash`, `registry_content_hash`, `identity_scheme_version`, `language_schema_version`, `compiler_interpretation_version`, `canonicalization_version`, `producer`.

`producer_version` must **differ**. An equal `producer_version` is not an upgrade and is refused. A refusal names the first field that differs.

A `source_edit_only` claim is refused when the producer changed. A producer change is an upgrade, so it must be declared as one.

If the meaning changed, the move is a `semantic_change` (human approval required). It is never a `compiler_upgrade`.

---

## Verify

- `authority_transitions` lists the allowed `compiler_upgrade` with its source and target refs.
- A governed action that cites the old ref is refused with `cep_world_refused`.

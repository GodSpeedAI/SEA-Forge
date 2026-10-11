# Qualification: provenance attribution wording

Date: 2026-10-09

The accompanying independent review receipt confirms the recovered manifest's exact SHA-256 and the one-entry difference from the current draft. This qualification narrows its verdict: the identity and metadata-difference checks pass, but the provenance wording is not fully verified.

The provenance note says the two substitutions were “recorded in the review.” The cited review receipts establish the reviewed manifest hash, but do not themselves record the before/after DEBT metadata values. Those values are supported by the separate bounded DEBT repair facts and the reconstruction comparison; the note should attribute them to that repair/recovery proof rather than to the cited privacy/copy review. The recovered artifact's hash match establishes the bytes, not runtime evidence or publication clearance.

**Disposition: HOLD the provenance unit pending a narrowly corrected provenance note and matching CW-42 wording.** Keep the reviewed manifest bytes immutable. The correction should identify the DEBT repair/recovery proof as the source of the two reversed metadata values and preserve the existing scope limit: no new capture and no publication clearance. No candidate privacy or publication approval is granted here.

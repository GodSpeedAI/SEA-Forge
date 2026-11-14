# GodSpeed FDE operating procedure (GS-FDE-SOP-001)

| Item | Value |
|---|---|
| Document | [`GS-FDE-SOP-001-v0.4.1.md`](GS-FDE-SOP-001-v0.4.1.md) |
| Version | 0.4.1 (synthesis) |
| Status | **Draft v0.4.1, pending owner ratification (OD-2).** Promoted here unchanged so the declared model can cite and hash it; promotion is not ratification. |
| sha256 | `043ae33ae6001da14f518feeb4ca2d23e280da2769d8378b36827f80d6057001` |
| Origin | Byte-identical copy of `.agents/reports/fde-convergence-2026-11-12/sop/GS-FDE-SOP-001-v0.4.1.md` on `main` (PR #98). |
| Declared model | [`.sea/interaction/fde-sop/`](../../.sea/interaction/fde-sop/) (world `godspeed-fde-sop`) |
| Source translation map | [`.sea/interaction/fde-sop/source-translation-map.md`](../../.sea/interaction/fde-sop/source-translation-map.md) |

The SOP file is frozen by hash. Any edit, including a status change after
ratification, produces a new version and a new hash; do not edit it in place.
`scripts/check-fde-sop-map.sh` fails if the file no longer matches the hash
above or if any numbered clause lacks a translation-map row.

## Checks

- `scripts/check-fde-sop-map.sh` (AT-02): every numbered SOP clause has a map
  row with a status and a model element, or `unknown` with a reason and a
  recorded limitation; no row lacks a verbatim source excerpt.
- `scripts/check-fde-sop-model.sh` (AT-01): `domainforge validate --application`
  and `domainforge envelope --emit cep` succeed, `world_ref` is reproducible,
  and the two settlement concepts are distinct and cannot be merged.

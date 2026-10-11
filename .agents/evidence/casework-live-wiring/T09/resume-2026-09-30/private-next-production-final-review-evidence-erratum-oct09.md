# Erratum: final Next review evidence archive prefixes

Date: 2026-10-09

The immutable review `private-next-production-final-independent-review-oct09.md`
(SHA-256 `2842acee4b5f35e2730cae9bc2600f7862ea1c5a22dc30f7082026a13f8b24b9`)
uses three root-archive prefixes without the `critic` component. The correct
independent capture archive prefixes are:

| Review currently says | Correct archive prefix |
|---|---|
| `next-private-focused02` | `next-private-critic-focused02` |
| `next-private-go02` | `next-private-critic-go02` |
| `next-private-race01` | `next-private-critic-race01` |

The `/tmp` capture paths, raw file identities, byte counts, command results,
and six-capture comparisons recorded in the review remain unchanged. This
erratum changes no source identity or approval decision.

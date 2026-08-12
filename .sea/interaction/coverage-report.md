# SEA Forge Interaction Coverage Report

## Scope and Method

This report reconciles the reviewed
`canonicalization-matrix.csv` against the approved twelve-journey interaction
model. The matrix contains the 128 unique `X.Y` stories from the governed
Workbench UX epic. Counts below come from CSV rows, not headings or narrative
estimates.

Explained coverage means an observed row has a syntactically valid mapping to
one of `CJ01` through `CJ12`. This definition intentionally counts mapped
`ambiguous` or `unsupported` rows if any exist; those classifications must also
appear in the unresolved list so that the percentage cannot hide uncertainty.

## Reconciled Totals

| Measure | Result |
| --- | ---: |
| Observed stories | 128 |
| Unique observed IDs | 128 |
| Canonical journeys | 12 |
| Canonical journeys represented | 12 of 12 |
| Explained rows | 128 |
| Unexplained rows | 0 |
| Explained coverage | 100% |
| Compression ratio | 128:12, or 10.67 observed stories per canonical journey |
| Canonical reduction | 116 fewer journey identities, or 90.625% |

Compression is `observed stories / canonical journeys`; it measures how many
observed stories one canonical journey explains on average. It does not imply
that the 128 story distinctions may be deleted: classifications and variation
dimensions preserve them.

## Coverage by Canonical Journey

| Canonical journey | Observed rows | Share |
| --- | ---: | ---: |
| CJ01 — Establish Trusted Cell Context | 15 | 11.72% |
| CJ02 — Discover Lawful Affordances | 16 | 12.50% |
| CJ03 — Ground Work in Semantic Meaning | 7 | 5.47% |
| CJ04 — Form and Commit a Governed Case | 10 | 7.81% |
| CJ05 — Navigate and Adapt a Live Case | 11 | 8.59% |
| CJ06 — Resolve Human Judgment and Approval | 7 | 5.47% |
| CJ07 — Execute Governed Work | 6 | 4.69% |
| CJ08 — Monitor, Intervene, and Recover | 12 | 9.38% |
| CJ09 — Evaluate, Settle, and Audit Outcomes | 17 | 13.28% |
| CJ10 — Reuse Demonstrated Knowledge and Capability | 10 | 7.81% |
| CJ11 — Transform and Mature Governed Artifacts | 10 | 7.81% |
| CJ12 — Transfer and Adopt Governed Assets | 7 | 5.47% |
| **Total** | **128** | **100.00%** |

Each canonical journey has exactly one `canonical` anchor row. The remaining
rows preserve narrower subjects, outcomes, interfaces, or multi-journey
behavior through specialization, variant, and composition classifications.

## Classification Counts

| Classification | Rows | Share |
| --- | ---: | ---: |
| canonical | 12 | 9.38% |
| specialization | 69 | 53.91% |
| variant | 26 | 20.31% |
| composition | 21 | 16.41% |
| duplicate | 0 | 0.00% |
| obsolete | 0 | 0.00% |
| ambiguous | 0 | 0.00% |
| unsupported | 0 | 0.00% |
| **Total** | **128** | **100.00%** |

`duplicate`, `obsolete`, `ambiguous`, and `unsupported` remain part of the
review vocabulary. Their zero counts are evidence outcomes: row-by-row review
found no story that required any of those classifications. They were not
omitted from the analysis.

Percentages in the distribution tables are rounded independently to two
decimal places; row counts are authoritative when rounded shares differ by
0.01 percentage point from 100%.

## Implementation Maturity Counts

| Maturity | Rows | Share |
| --- | ---: | ---: |
| declared | 0 | 0.00% |
| specified | 13 | 10.16% |
| implemented | 6 | 4.69% |
| exercised | 85 | 66.41% |
| evidenced | 24 | 18.75% |
| **Total** | **128** | **100.00%** |

Maturity is independent of canonicalization. An exercised specialization does
not make every variant of its canonical journey exercised, and a specified
story remains covered as stable user intent without being presented as
implemented behavior.

## Confidence Counts

| Confidence | Rows | Share |
| --- | ---: | ---: |
| high | 125 | 97.66% |
| medium | 3 | 2.34% |
| low | 0 | 0.00% |
| **Total** | **128** | **100.00%** |

The three medium-confidence rows are:

- **3.5 — Ask about projection and environment support:** the intent is stable, but no single implemented query kind exposes the combined support claim.
- **11.3 — Receive actionable notifications:** committed events and UI invalidation exist, but a complete notification center remains unsurfaced.
- **16.3 — Manage extension and endpoint lifecycle:** registry and standing behavior exist, but the complete lifecycle command set remains cataloged rather than implemented end to end.

## Unresolved Rows

None. The matrix contains no `ambiguous` or `unsupported` row, so unresolved
IDs are `[]`. It also contains no duplicate or obsolete row. Explained
coverage is therefore 128 / 128 = 100% with no uncertainty hidden inside that
percentage.

## Most Common Variation Dimensions

The matrix stores dimensions as semicolon-separated `key=value` entries. The
counts below count key occurrences, not distinct rows or values. They are
non-exclusive because one story usually varies along several dimensions.

| Dimension key | Occurrences |
| --- | ---: |
| subject | 47 |
| artifact | 33 |
| evidence | 28 |
| interface | 26 |
| assurance | 22 |
| actor | 21 |
| next decision | 19 |
| initiating condition | 17 |
| authority outcome | 14 |
| work source | 11 |
| recovery path | 10 |
| state transition | 10 |
| completion | 9 |

The distribution supports the canonicalization method. Most observed
differences concern the subject, artifact, evidence, interface, assurance, or
actor around a stable intention and transition. Interface is frequent because
Thoth alone appears in nine rows, but interface recurrence does not establish
a separate journey.

## Evidence Derivation

The count pipeline applies these checks to the reviewed CSV:

1. Load all rows with `csv.DictReader`.
2. Require 128 rows and 128 unique observed IDs.
3. Count `Canonical Journey ID` and require exactly `CJ01`–`CJ12`.
4. Count classifications, maturity, and confidence independently.
5. Treat a row as explained when its canonical ID is valid, regardless of unresolved classification.
6. List `ambiguous` and `unsupported` rows separately; the resulting list is empty.
7. Split `Variation Dimensions` on semicolons and count the key before the first `=`.
8. Compute coverage as `128 / 128 * 100 = 100%` and compression as `128 / 12 = 10.666…`, reported as 10.67:1.

The catalog-to-matrix check confirms that all twelve catalog headings exist and
all 128 rows map into the same twelve-ID set. These figures supersede the
pre-review maturity totals and use the reviewed matrix at commit `b83b25d`.

## Enforcement of These Counts

The totals above are no longer only a report. Since the model was
re-canonicalized against DomainForge 0.16.0, most of them are enforced.

DomainForge enforces, and `domainforge validate` fails if any of these drifts:

| Count | Policy |
| --- | --- |
| 12 canonical journeys | `twelve_canonical_journeys` |
| 128 observed stories, summed across journeys | `all_observed_stories_accounted` |
| 13 specified rows | `specified_stories_reconcile` |
| 6 implemented rows | `implemented_stories_reconcile` |
| 85 exercised rows | `exercised_stories_reconcile` |
| 24 evidenced rows | `evidenced_stories_reconcile` |
| 128 classified rows | `classification_rows_account_for_all_stories` |
| 8 classification values | `eight_canonicalization_classifications` |
| 11 variation dimensions | `eleven_variation_dimensions` |
| 38 interface surfaces | `thirty_eight_interface_surfaces` |
| 8 skeleton phases with ordinals 1–8 | `eight_interaction_steps`, `interaction_step_ordinals_complete` |

`validation/reconcile.py` closes the remaining gap, because DomainForge cannot
read the CSV and cannot evaluate arithmetic inside a policy `where` predicate
(`validation/limitations.md` L5). It checks, row by row against
`canonicalization-matrix.csv`:

- 128 rows with 128 unique observed IDs;
- every row maps to a journey the model declares;
- every classification and maturity value is inside the model's closed enums;
- each journey's `observed_story_count` equals its actual CSV row count;
- each journey's four maturity counts equal its actual CSV distribution and sum
  to its `observed_story_count`;
- each classification's `observed_rows` equals its actual CSV count;
- each journey's `interface_binding_count` equals the number of
  `InterfaceBinding` instances that reference it.

Confidence counts and the variation-dimension occurrence table remain
report-only: confidence is a reviewer judgment carried on CSV rows and is not
modeled as a SEA field, and the occurrence table counts free-text keys inside
the matrix's `Variation Dimensions` column rather than declared concepts.

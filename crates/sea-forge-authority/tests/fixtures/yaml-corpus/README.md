# YAML corpus fixtures

Test data for the serde_yaml 0.9 to serde-saphyr parser migration. Some documents are deliberately malformed (duplicate keys, unknown fields, nulls, booleans spelled as YAML 1.1 words, tags, anchors and alias bombs, multi-document streams, deep nesting). Load them only in tests. Never use them as configuration.

## Contents

- `real/`: 107 documents. 6 are real repo files (policy, template, and t08c server and template YAML). The other 101 were extracted from Rust string literals in `crates/**/*.rs`.
- `adversarial/`: 686 generated edge cases (`B0000.yaml` to `B0689.yaml`, minus the excluded ids below).
- `targets.tsv`: `corpus`, `id`, `target` for every (document, typed target) pair the tests evaluate. Real documents are evaluated against all three typed targets (321 rows). Adversarial cases have one target by id range: B0000-B0512 `AuthorityPolicyBundle`, B0513-B0579 `EnvironmentSpec`, B0580-B0689 `PlanTemplate` (1007 rows in total).
- `reference.json`: 1011 reference records (`id`, `corpus`, `target`, `accepted`, `error_class`, `identity`, `probes`), sorted by id and target. Records for the excluded ids are included.
- `README.md`: this file.

## Reference provenance

The reference values were produced by SEA-Forge `main` at commit `ee66154` (serde_yaml `0.9.34+deprecated`) through the production loaders:

- `AuthorityPolicyBundle`: `AuthorityPolicyBundle::load(path)`, then `PolicyAuthorityEngine::new(bundle)`. Accepted bundles also record 18 probe decisions (9 actions x 2 roles: Operator and Agent) and an identity: the canonical JSON SHA-256 of the loaded bundle, plus the policy hash, identity binding and policy refs of a fixed decision.
- `EnvironmentSpec`: `serde_yaml::from_slice::<EnvironmentSpec>`. Identity: canonical JSON SHA-256 of the typed value.
- `PlanTemplate`: `serde_yaml::from_str::<PlanTemplate>`. Identity: canonical JSON SHA-256 of the typed value.

Rejected records keep only the error class (`config:parse_error`, `config:schema_error`, `parse`, ...). Error text is not compared, because it differs between parsers.

## Excluded inputs (over 64 KiB, not committed)

These four adversarial inputs exceed 64 KiB. Their reference records are in `reference.json`. Each proof test asserts that its skipped set equals the `GENERATED_IN_TEST` constant (4 bundle cases):

| id | bytes | target |
|---|---|---|
| B0509.yaml | 91144 | AuthorityPolicyBundle |
| B0510.yaml | 2000539 | AuthorityPolicyBundle |
| B0511.yaml | 2000536 | AuthorityPolicyBundle |
| B0512.yaml | 408912 | AuthorityPolicyBundle |

## Tests that read this directory

- `crates/sea-forge-authority/tests/yaml_corpus.rs`: bundles.
- `crates/sea-forge-planner/tests/yaml_corpus.rs`: `PlanTemplate`.
- `crates/sea-forge-sandbox/tests/yaml_corpus.rs`: `EnvironmentSpec`.

Run with `cargo test -p <crate> --test yaml_corpus -- --nocapture`. The `--nocapture` output prints one summary line per bucket.

## Regeneration

The generator lives outside this repository, in `/home/sprime01/projects/_wt/yaml-diff/reference` (a standalone binary with path dependencies on a checkout of SEA-Forge main `ee66154`). Steps:

1. Check out `ee66154` (the reference tree), build with `cargo build -j 2 -p yaml-corpus-reference`, and run it. It writes `yaml-diff/reference/reference.json`. Running it twice gives byte-identical output.
2. Copy `reference.json` to this directory.
3. The corpus source is `yaml-diff/corpus/A` (with `A_manifest.tsv`) and `yaml-diff/corpus/B`. Copy them with the target rule above, and exclude files over 64 KiB.

The committed files are the source of truth for the tests. The generator is needed only to regenerate them.

## Totals

- 795 files, about 4.0 MB on disk, including `reference.json` (about 0.75 MB).
- Real: 62 bundle accepted and 45 rejected; environment spec 0 accepted and 107 rejected; template 8 accepted and 99 rejected.
- Adversarial: bundle 5 accepted and 508 rejected in the reference, environment 45 and 22, template 60 and 50.

## Behaviour changes

Adversarial cases that the reference (SEA-Forge main ee66154, serde_yaml 0.9) accepts and the new parser rejects. Each list is the `STRICTER` constant in the matching proof test, asserted equal to the computed list. Real-corpus documents have zero such changes.

### AuthorityPolicyBundle (1)

- `B0169.yaml`: rules Vec = ~ (list replaced)

### PlanTemplate (9)

- `B0584.yaml`: version = null
- `B0652.yaml`: name = \"~\"
- `B0655.yaml`: name = \"null\"
- `B0658.yaml`: name = \"Null\"
- `B0661.yaml`: name = \"NULL\"
- `B0664.yaml`: name = \"\"
- `B0666.yaml`: operations = \"\"
- `B0673.yaml`: plan item tagged !foo
- `B0686.yaml`: BOM prefix

### EnvironmentSpec (9)

- `B0538.yaml`: name null
- `B0545.yaml`: description null
- `B0553.yaml`: name = null
- `B0570.yaml`: version = \"~\"
- `B0572.yaml`: version = \"null\"
- `B0574.yaml`: version = \"Null\"
- `B0576.yaml`: version = \"NULL\"
- `B0578.yaml`: version = \"\"
- `B0579.yaml`: base = \"\"

Duplicate-key cases (B0000-B0018, bundle target) are NOT in this list: the reference already rejects them (`config:parse_error`), so they are both-rejected. The proof test asserts that both reject every one of them.

### Newly accepted (reference rejects, new code accepts)

None. Every `NEWLY_ACCEPTED` table in the proof tests is empty.

An earlier revision accepted mixed-case boolean spellings on the template path
(`B0634.yaml`, `B0651.yaml`: `tRue`). The proof test failed on them, and the
boundary now requires booleans to be spelled exactly `true` or `false`. Those
two cases, and `True`/`TRUE`/`FALSE` (`B0624`, `B0625`, `B0632`, `B0641`,
`B0642`, `B0649`, which the old parser accepted), are now rejected.

## Residuals

Fields that serde buffers before typing them (`#[serde(flatten)]` and
internally tagged enums) do not pass through the spelling check. There,
`True`/`TRUE`/`tRue` booleans and `1_000`/`0x10` integers are still accepted
with the value the text denotes. No authority policy type uses either
attribute; the affected fields are `max_turns` and `token_budget` on the
planner's `AgentTask` template operation. Pinned by the `open_issue_*` tests in
`crates/sea-forge-core/tests/yaml_strictness.rs`.

Explicit YAML core tags (`!!bool`, `!!int`, `!!str`, ...) are honoured; the
parser has no option to reject them. No document in the real corpus or in the
repository uses a tag.


# T09 canonical contract unit — independent confirmation

## Verdict

**APPROVE this bounded canonical contract unit.** The implementation satisfies the original
canonical implementation assignment and the fresh test-only repair preserves its runtime
membership/shape assertions while making the file pass strict TypeScript. Schema source review
is recorded in `canonical-second-implementation-source-review.md`; earlier immutable rejection
records retain the defect/repair history. This does not approve Go/UI mirrors, Ask route or
observation runtime enforcement, native SSE framing, or later narration wiring.

## Independent verification

Immediately before each compiler, host preflight used `free -h` and a process scan for Bun,
Node, TypeScript, Go, Rust, and Cargo. Both scans found no competing compiler. Available RAM
was 2.4 GiB before Bun and 2.9 GiB before TypeScript.

Focused test command:

```text
bun test ./.agents/reports/interface-contracts/tests/contract-conformance.test.ts
```

Result: **11 pass, 0 fail, 963 assertions**. Complete stdout is preserved in
`canonical-final-green2.log`, SHA-256
`82c1cacda21ba0a32f987f1aab8456ee1f7c8b0e1ad8f15f59ec4476941cdb09`. It was copied from
`/tmp/sea-rs-canonical-final-green2.log`; `cmp` returned 0.

Explicit strict TypeScript pin command, run from `apps/godspeed-cognitive-ui` using the
existing local compiler and type package (no config or dependency change):

```text
./node_modules/.bin/tsc --noEmit --strict --target ES2023 --lib ES2023,DOM,DOM.Iterable --module ESNext --moduleResolution bundler --skipLibCheck --types bun-types --typeRoots ./node_modules ../../.agents/reports/interface-contracts/tests/contract-conformance.test.ts
```

Result: **exit 0**, with empty stdout/stderr. The empty raw log is preserved in
`canonical-explicit-typescript-pins2.log`, SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; it compares byte-for-
byte with `/tmp/sea-rs-canonical-typescript-pins2.log`. The `Expect<Equal<...>>` aliases
therefore type-check against the canonical DTOs. The prior failing compiler output and
fresh-helper repair remain separate and immutable.

## Repair review

The fresh helper repair is test-only. `expectContainsUnknown(readonly unknown[], unknown)`
continues to call Bun's exact `toContain` matcher, preserving runtime membership checks while
typing their dynamic JSON inputs. The added `omitted_frame_count` integer/type guard preserves
the total/retained/omitted arithmetic assertion. The `omitted_claim_classes` array guard
narrows unknown JSON before iteration without dropping claim-class membership checks. The
previous exact `Expect<Equal<...>>`/`OptionalKeys` aliases are unchanged. No `any`, suppression,
weakened equality pin, or removed shape check was introduced.

The last Ask description correction is present at
`schemas/thoth-ask.schema.json:5`: it says Ask may write durable question/disclosure records,
ambiguous outcomes are not resent, and answers grant no execution authority. This matches
`ask-route-source-preparation.md`: Request::Ask has no correlated durable-status locator, so
ambiguous EOF/overflow must be unavailable after one send; current runtime enforcement remains
pending. Request and answer root schemas are linked by the previously reviewed `oneOf`.

## Source identity and bounded scope

At confirmation time:

- `tests/contract-conformance.test.ts` SHA-256:
  `39bd4c422ff6d96715f71c951ae0678bf3e71761d53c6ed985fe3d5c8ead7382`
- canonical `typescript/types.ts` SHA-256:
  `d272a25a7e799b839ac1997a4c153fc0a8697ca11f365708b100030c1b092b7b`
- Ask schema SHA-256:
  `d7be23523a19d80e86e619be9c1f1e97b8e01e6c00f757d18c050ee2206c606e`
- event stream schema SHA-256:
  `64998a234881e1b7b81a38165c9ce1bc75fa6948269ba3469b768739b9f23624`

The second rejection findings are resolved and the source plus fixtures now pass both required
focused verifications. JSON Schema does not prove cross-field count arithmetic or array-length
equality; those remain runtime assertions/tests, not hidden claims. Actual per-line cap,
polling, hydration and cache bounds remain pending implementation. JSONL does not prove native
SSE omission of an `id:` line or actual cursor reuse. The detached
`ThothNarrationResultMetadata` helper remains a later narration/port integration limit, as
documented in the source review; it is not presented as completed UI wiring.

No source, schema, golden, runtime, dependency, or configuration edit was made by this critic.

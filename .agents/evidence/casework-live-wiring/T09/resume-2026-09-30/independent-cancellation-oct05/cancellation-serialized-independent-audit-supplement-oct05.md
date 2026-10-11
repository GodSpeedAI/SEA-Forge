# Independent cancellation audit supplement

Date: 2026-10-05

This supplement provides archive verification and capture timing for
`cancellation-serialized-independent-approval-oct05.md`. It does not change the
NOT APPROVED decision, source, fixture, or prior captures. No compiler sessions
remain; the sole compiler token was returned to root.

## Report edit disclosure

After first creating the main report, I appended the requested root-findings,
timeout-anchor, RAM, and historical-pair details. Root then directed that an
already created evidence report be treated as immutable and that any further
details go in a new supplement. I did not edit the main report after that
direction. I did not capture the main report's hash before the append, so its
pre-append hash cannot be recovered from this work. Its current SHA-256 is
`adcdc58b8ef983691fd8eeeea8cf9d96ed8eef9d5e889aa419368d06177b3cf0`.

## Frozen source identities

- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go`:
  `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`.
- `apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go`:
  `be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c`.
- Root findings document:
  `cancellation-gate-evidence-root-findings-oct05.md`, SHA-256
  `d4d7f143e517cbbf38622a489fbb274d22b8e92391dbfbad260c79c3d5f6e426`.

Exact timeout source/test anchors are recorded in the main report:
`response_limit_test.go:249-281`, `:284-330`, `client_test.go:113-123`, and
`client.go:419-449`, `:465-476`.

## Fresh gate capture timing and host evidence

All timestamps below are local `-0400`; preflight command output itself contains
UTC. Raw and exit files were produced only after their corresponding command
finished. Each next preflight began after the prior exit file was written.

| Gate order | Preflight capture | Raw output / exit capture | Result |
| --- | --- | --- | --- |
| Focused cancellation race | 13:13:43.225814 | 13:14:02.825812 / 13:14:02.834024 | exit 0 |
| Full SFWP race | 13:14:12.385810 | 13:14:56.869805 / 13:14:56.876460 | exit 1 |
| Canonical `just casework-go-check` | 13:15:25.145800 | 13:15:38.725799 / 13:15:38.734019 | exit 0 |
| Full-module race | 13:15:49.153797 | 13:17:17.681785 / 13:17:17.687974 | Go test exit 1 |

Fresh preflight RAM values (MemTotal 8,132,712 kB on all four):

- Focused: MemAvailable 2,695,396 kB; SwapTotal 12,582,912 kB; SwapFree
  2,342,168 kB.
- SFWP: MemAvailable 2,697,300 kB; SwapTotal 12,582,912 kB; SwapFree
  2,342,648 kB.
- Canonical: MemAvailable 2,649,816 kB; SwapTotal 12,582,912 kB; SwapFree
  2,213,244 kB.
- Module: MemAvailable 2,679,160 kB; SwapTotal 12,582,912 kB; SwapFree
  2,240,480 kB.

Each preflight preserves the complete `ps -eo pid,comm` output. None listed a
`go`, `cargo`, `rustc`, or `compile` process. Existing `.cline`, `bun`, `node`,
and agent processes are present by process name. No argv or environment dumps
were captured. Preflight paths use `/tmp/sea-cancellation-rootcritic-oct05-*`;
the preserved repository prefix is
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/independent-cancellation-oct05/rootcritic-`.

All 12 new repository copies were compared byte-for-byte with their `/tmp`
source files. SHA-256 of the repository copies:

| Artifact | SHA-256 |
| --- | --- |
| `rootcritic-focused-preflight.raw` | `345e8570b01d23116f58a5b136f936088a80a36d1da7b97353ac38e699712c23` |
| `rootcritic-focused.raw` | `6efd6a42f3dc799594d67b06fb2f85b905558b1635a3b58c9081c97199c52524` |
| `rootcritic-focused.exit` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `rootcritic-sfwp-preflight.raw` | `a9e683ac15af75c2c7065bb9e9069f45b8416b29e8b3967d7a3675af94609646` |
| `rootcritic-sfwp.raw` | `e787383c32683a6d838760792b0552d5e51a4d0762c244891e9a9224f50442fc` |
| `rootcritic-sfwp.exit` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |
| `rootcritic-canonical-preflight.raw` | `eeb484c15df1d8913ca79f6d03af06e4d2be7531585133bebd26393261263b75` |
| `rootcritic-canonical.raw` | `139e7b7e83e78d342eba5321d9fd3a59aa607f681d723852a0f3806afcf85055` |
| `rootcritic-canonical.exit` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `rootcritic-module-preflight.raw` | `a289e83ed6cd9320fbbd430c2b7ccb1c93d01c3039e86d8fd2455e328012128f` |
| `rootcritic-module.raw` | `4b5a6d14ebfa8e35aa22622ad1996acde149d81cf690b377b1f476f35a87ea2d` |
| `rootcritic-module.exit` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

## Historical archive audit

I compared all 10 historical raw archives and their 10 exit archives against
the corresponding `/tmp/sea-cancellation-oct05/rootcritic-*` originals. All 20
pairs matched exactly. This covers the ten rows in the root findings document's
raw table plus every paired exit it omits from that table. The historical
archive match establishes byte identity only; the root findings document's
prior concurrency violation still makes those earlier gates procedurally
insufficient. No historical or fresh capture was overwritten or removed.

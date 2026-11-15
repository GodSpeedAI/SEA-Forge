# Compiler upgrade note: DomainForge 0.21.2 to 0.22.0

The model sources (`fde-sop.sea`, `evidence.sea`, `outcome.sea`) did not change. The compiler did, and
`world_ref` is bound to the producer version, so the world identity changed.

| | DomainForge 0.21.2 | DomainForge 0.22.0 |
|---|---|---|
| `world_ref` | `world:godspeed-fde-sop@sha256:78eddd4e7e4771caa1c37e8e5fa095c5baa899daccea8fac1f6e969f8f37287b` | `world:godspeed-fde-sop@sha256:e3c3d396577159a51581782455b68c30fe353292b6023c24625d28174627cf17` |
| `created_by` | `domainforge 0.21.2` | `domainforge 0.22.0` |

Verified with the released 0.22.0 CLI (`domainforge-0.22.0-x86_64-unknown-linux-gnu.tar.gz`,
tarball sha256 `0a976877346c4032245225c1b0bcac559100ad1f51fe7e06293e7a13e8c42b1d`, matching the release
SHA256SUMS; extracted binary sha256 `2d0822810644746417cfb82065472baac11296e2a6ab2ba94d95196e428d5d14`).
`scripts/check-fde-sop-model.sh` was rerun with it: VERDICT PASS, `validate --application` output unchanged
(same policy results as the 0.21.2 record), world_ref identical across three runs.
`domainforge-output.txt` is regenerated with 0.22.0. `map-check-output.txt` is unchanged (it does not
depend on the compiler).

The S1 pre-registration (`.agents/preregistrations/fde-s1`, frozen, anchored at `fde-anchor/prereg/fde-s1`)
and the committed S1 evidence bundle name the 0.21.2 world. That is correct history and is not edited.
`scripts/check-fde-sop-model.sh` now pins the compiler to 0.22.0 and reports whether the current world
equals the S1 frozen world or differs because of the compiler.

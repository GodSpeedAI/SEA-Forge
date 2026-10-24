# Changelog

## [0.2.0](https://github.com/GodSpeedAI/SEA-Forge/compare/v0.1.0...v0.2.0) (2026-10-24)


### Features

* add minimum kernel CLI commands ([b6106ab](https://github.com/GodSpeedAI/SEA-Forge/commit/b6106ab1418853df168a5b882bdfede1a4be81d1))
* **authority:** a resources filter on cognate_action / cognate_capability rules ([#43](https://github.com/GodSpeedAI/SEA-Forge/issues/43)) ([503cd63](https://github.com/GodSpeedAI/SEA-Forge/commit/503cd63fa501bc977bddd260f7df9b441a7c8e06))
* **authority:** corroborate evidence against RealityTrace's own ledger ([#45](https://github.com/GodSpeedAI/SEA-Forge/issues/45)) ([73683c0](https://github.com/GodSpeedAI/SEA-Forge/commit/73683c08695b4cee8b7ca71fc30e03e51b74d622))
* **authority:** record request claims and settle obligations from RealityTrace's ledger ([#48](https://github.com/GodSpeedAI/SEA-Forge/issues/48)) ([0106ffc](https://github.com/GodSpeedAI/SEA-Forge/commit/0106ffc656a04a82644d134a3c884f0bbe3b5a53))
* **authority:** require an allowed transition for lineage across worlds (M-46) ([#28](https://github.com/GodSpeedAI/SEA-Forge/issues/28)) ([d83cc32](https://github.com/GodSpeedAI/SEA-Forge/commit/d83cc32834dd83693fc6dff22836ed05090f0185))
* casework UI runs against the live SEA Forge kernel with a ([bd661c9](https://github.com/GodSpeedAI/SEA-Forge/commit/bd661c909d579e80e2ed7ee3cc18012535f50b4a))
* **casework:** wire the casework UI to the live kernel with e2e, load and packaging ([#11](https://github.com/GodSpeedAI/SEA-Forge/issues/11)) ([bd661c9](https://github.com/GodSpeedAI/SEA-Forge/commit/bd661c909d579e80e2ed7ee3cc18012535f50b4a))
* **cell:** governed Semantic Cell episode on OpenShell with approval-gated promotion ([#24](https://github.com/GodSpeedAI/SEA-Forge/issues/24)) ([7542e2b](https://github.com/GodSpeedAI/SEA-Forge/commit/7542e2bf7963773a64cdcc045db52d184a58ee6a))
* **cep:** auditable compiler_upgrade transition for DomainForge producer upgrades ([#42](https://github.com/GodSpeedAI/SEA-Forge/issues/42)) ([b384fdd](https://github.com/GodSpeedAI/SEA-Forge/commit/b384fdd7820754c57bfc909d9ec77f49a0a7c60a))
* implement agent context management and diagnostics framework ([2e5421b](https://github.com/GodSpeedAI/SEA-Forge/commit/2e5421bdb675926cb81946243d09cc4e37750638))
* implement governed kernel lifecycle ([1b03ca4](https://github.com/GodSpeedAI/SEA-Forge/commit/1b03ca45a9d827adb1dfacecad62adc5e9dfb820))
* initialize SEA Forge CLI and core components ([592ab39](https://github.com/GodSpeedAI/SEA-Forge/commit/592ab3951299e66965d0febf28131d2bd2dabb04))
* **security:** close live authorization gaps and harden isolation ([#32](https://github.com/GodSpeedAI/SEA-Forge/issues/32)) ([257b622](https://github.com/GodSpeedAI/SEA-Forge/commit/257b6224eb751094e8e25a059a1d1b49360f3c7f))
* **security:** round 4 pathname unix-socket containment, kernel-security CI, authority scoping ([#38](https://github.com/GodSpeedAI/SEA-Forge/issues/38)) ([14a5f97](https://github.com/GodSpeedAI/SEA-Forge/commit/14a5f9722fbc78fadb7530077960846329438b1a))
* **ua:** add Understand-Anything graph support with auto-update hook ([6f1ae91](https://github.com/GodSpeedAI/SEA-Forge/commit/6f1ae915aff7b5e0927a0d872126d7656449fb3d))
* **workspace:** add upgraded-sea workspace configuration ([2f95790](https://github.com/GodSpeedAI/SEA-Forge/commit/2f95790ba2a41562ed0e09932f8c30e04de30727))


### Bug Fixes

* **authority:** match DomainForge write targets only on plain workspace-relative paths ([#46](https://github.com/GodSpeedAI/SEA-Forge/issues/46)) ([17253b0](https://github.com/GodSpeedAI/SEA-Forge/commit/17253b0774a57f6ac0ca81556c3fe36d5f7d696f))
* **authority:** validate the grant before consuming an approval ([#19](https://github.com/GodSpeedAI/SEA-Forge/issues/19)) ([ead6c5b](https://github.com/GodSpeedAI/SEA-Forge/commit/ead6c5b5a86742018d52eb638125ad166da6c979))
* **ci:** remove invalid workflow expression placeholder ([#4](https://github.com/GodSpeedAI/SEA-Forge/issues/4)) ([530c235](https://github.com/GodSpeedAI/SEA-Forge/commit/530c235751c439418fcfb666b6643ca5efee8364))
* **ci:** use GITHUB_TOKEN for release-please (RELEASE_PLEASE_TOKEN is unset) ([#51](https://github.com/GodSpeedAI/SEA-Forge/issues/51)) ([4780c26](https://github.com/GodSpeedAI/SEA-Forge/commit/4780c268f8efa21aecd8873d8af88a51e83f8402))
* **deps:** reconcile upgrades while preserving intentional replacements ([#39](https://github.com/GodSpeedAI/SEA-Forge/issues/39)) ([2a57ca5](https://github.com/GodSpeedAI/SEA-Forge/commit/2a57ca54e2e282ac864b71e39706ce3694a2c4e8))
* **deps:** update ctutils to 0.4.5 (RustCrypto advisory) ([#47](https://github.com/GodSpeedAI/SEA-Forge/issues/47)) ([061db77](https://github.com/GodSpeedAI/SEA-Forge/commit/061db77a39e2f8888408888067f2a45775aa84b6))
* **domainforge:** move to domainforge-core 0.21.1 in lockstep with the stack ([#40](https://github.com/GodSpeedAI/SEA-Forge/issues/40)) ([6e1d400](https://github.com/GodSpeedAI/SEA-Forge/commit/6e1d400dcefd635a66860c479729f0f3fca2b9bd))
* harden minimum kernel conformance ([1ffa59a](https://github.com/GodSpeedAI/SEA-Forge/commit/1ffa59a126103799715b4bd5c0e9ea6bd9999ffa))
* **identity:** one operating-system uid maps to exactly one actor ([#41](https://github.com/GodSpeedAI/SEA-Forge/issues/41)) ([b95a28e](https://github.com/GodSpeedAI/SEA-Forge/commit/b95a28efe72cfbfccb6e56e803059d203d854d81))
* **release:** drop $schema from the release-please manifest ([#52](https://github.com/GodSpeedAI/SEA-Forge/issues/52)) ([b09cef6](https://github.com/GodSpeedAI/SEA-Forge/commit/b09cef65d3e3e90310f07e44031412f594d2e5c2))
* **release:** release-please simple type with extra-files for the workspace version ([#53](https://github.com/GodSpeedAI/SEA-Forge/issues/53)) ([a76c84c](https://github.com/GodSpeedAI/SEA-Forge/commit/a76c84c110c1ee2f4330b8a5e181dc9a28f21e4e))
* **security:** replace serde_yaml with a strict YAML boundary and drop unsafe-libyaml ([#15](https://github.com/GodSpeedAI/SEA-Forge/issues/15)) ([97ac2b7](https://github.com/GodSpeedAI/SEA-Forge/commit/97ac2b7da66afa1dc881183c7505c4ddcc2ee072))
* **specs:** amend spec-full.md prerequisites and corrections ([2f95790](https://github.com/GodSpeedAI/SEA-Forge/commit/2f95790ba2a41562ed0e09932f8c30e04de30727))
* **specs:** update DESIGN.md to reflect SEA Forge terminology ([2f95790](https://github.com/GodSpeedAI/SEA-Forge/commit/2f95790ba2a41562ed0e09932f8c30e04de30727))

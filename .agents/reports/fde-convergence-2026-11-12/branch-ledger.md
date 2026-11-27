
## OD-4 backups (2026-11-16)

Backup refs are lightweight tags `backup/od4-2026-11-16/<branch>`, `.../stash/<checkout-dir>/<n>`, `.../dirty/<checkout-dir>`. They are TEMPORARY: remove after consolidation verifies each item is integrated or rejected. Nothing local was changed (status of all 95 checkouts identical before/after). Dirty trees were snapshotted via temp index + commit-tree. Gitleaks scanned the unpushed range once per repo (`--redact`); a branch is flagged if any unpushed commit has a finding. Follow-up: items whose every finding blob already exists on origin count as no new exposure and were pushed (SEA branches as real branches under their own names, other items as tags). Items with a finding blob not on origin, SEA-Forge items, and cognate-workbench feat/horizon-connect are HELD in local bundles under ~/backups/od4-2026-11-16/ (SEA-Forge bundles are thin: `<ref> --not --remotes=origin`, prerequisites are on origin).

| repo | checkout | kind | name | tip SHA | unpushed commits | gitleaks | backup tag | verified remote SHA / status |
|---|---|---|---|---|---|---|---|---|
| Context_Kernel | Context_Kernel | branch | backup/harden-pre-sync | 62ee3485f29099dde024c6ff51551ba1debecf9b | 1 | clean | backup/od4-2026-11-16/backup/harden-pre-sync | 62ee3485f29099dde024c6ff51551ba1debecf9b |
| Horizon | Horizon | branch | feat/governed-operate | 32a5344be763a0dc2aea61046357dd79630fc9da | 1 | clean | backup/od4-2026-11-16/feat/governed-operate | 32a5344be763a0dc2aea61046357dd79630fc9da |
| Horizon | Horizon | branch | feat/ignition-onboarding | 6f57bfe8063a53736a1077c0eefbd899bade1081 | 1 | clean | backup/od4-2026-11-16/feat/ignition-onboarding | 6f57bfe8063a53736a1077c0eefbd899bade1081 |
| Horizon | Horizon | branch | fix/gateway-socket-dir-at-boot | 374b58e49579654a5a53884fc6b46b82ec0bf86b | 1 | clean | backup/od4-2026-11-16/fix/gateway-socket-dir-at-boot | 374b58e49579654a5a53884fc6b46b82ec0bf86b |
| Horizon | Horizon | branch | fix/j02r2-rig-operator-ref | 2e2c290cd52441a64f6b3e0e317f35cb010f6c39 | 1 | clean | backup/od4-2026-11-16/fix/j02r2-rig-operator-ref | 2e2c290cd52441a64f6b3e0e317f35cb010f6c39 |
| Horizon | Horizon | branch | fix/settlement-pending-error-class | 3f62b04cc222b2580d49fd3e8ed3bb8133c733ed | 1 | clean | backup/od4-2026-11-16/fix/settlement-pending-error-class | 3f62b04cc222b2580d49fd3e8ed3bb8133c733ed |
| Horizon | Horizon | branch | stage0b-containment-core | 23db06834865830aaf61d7a399fb20e7ea38379b | 3 | clean | backup/od4-2026-11-16/stage0b-containment-core | 23db06834865830aaf61d7a399fb20e7ea38379b |
| Horizon | Horizon | branch | stage0b-containment-pr1 | 43e8327eb67c543fea47e9dbf58c9b2fc8975bef | 3 | clean | backup/od4-2026-11-16/stage0b-containment-pr1 | 43e8327eb67c543fea47e9dbf58c9b2fc8975bef |
| Horizon | Horizon | branch | stage0b-containment-pr1-narrow | 7115b20258c4f5eebbcbe2334859a70882328830 | 4 | clean | backup/od4-2026-11-16/stage0b-containment-pr1-narrow | 7115b20258c4f5eebbcbe2334859a70882328830 |
| Horizon | Horizon | branch | stage0b-pr1-broad-snapshot | 43e8327eb67c543fea47e9dbf58c9b2fc8975bef | 3 | clean | backup/od4-2026-11-16/stage0b-pr1-broad-snapshot | 43e8327eb67c543fea47e9dbf58c9b2fc8975bef |
| Horizon | Horizon | branch | stage0b-pr1-narrowing-wip | 3249d2bdf1e2ba3e46273ab43c1751ab1ca51eb7 | 1 | clean | backup/od4-2026-11-16/stage0b-pr1-narrowing-wip | 3249d2bdf1e2ba3e46273ab43c1751ab1ca51eb7 |
| SEA | SEA | branch | backup-before-scrub-20260202 | e437f0afd1e1dfbd6d5b522f6abaf9e94f8d75a8 | 753 | 22 curl-auth-header,generic-api-key | backup/od4-2026-11-16/backup-before-scrub-20260202 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-backup-before-scrub-20260202.bundle sha256 6ae08cdf40161d88b54b99d2a5cb3bca6aa74f0d1d298caeb2c95959f8d48495 (verify 0/ok) |
| SEA | SEA | branch | backup/cycle-cell-mesh-before-cla-rewrite | 81698a57b1dfd17f5474b102c3c07a55a84ec7a1 | 35 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/backup/cycle-cell-mesh-before-cla-rewrite | 81698a57b1dfd17f5474b102c3c07a55a84ec7a1 (ref refs/heads/backup/cycle-cell-mesh-before-cla-rewrite; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | backup/pr-518-pre-cla-rewrite | 186e6f9dc6b53a60c9c6a196a5759b480efb2220 | 1886 | 105 curl-auth-header,generic-api-key | backup/od4-2026-11-16/backup/pr-518-pre-cla-rewrite | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-backup_pr-518-pre-cla-rewrite.bundle sha256 0732694994d5513eac5cc035ff8a161f2f1c03c7f97b9e367bda2de3213f04cb (verify 0/ok) |
| SEA | SEA | branch | coderabbit-fixes | 18c5a5cb634cdca11f429cbc7036859301564784 | 1539 | 34 curl-auth-header,generic-api-key | backup/od4-2026-11-16/coderabbit-fixes | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-coderabbit-fixes.bundle sha256 9dd31c320a18d0b61528b3ec71701081d66dbc61d327580e499cadfd59e103df (verify 0/ok) |
| SEA | SEA | branch | copilot/pr523-ci-fix | eb3db1dc565d9438827a8fb8b4121dc78a67c440 | 14 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/copilot/pr523-ci-fix | eb3db1dc565d9438827a8fb8b4121dc78a67c440 (ref refs/heads/copilot/pr523-ci-fix; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | cycle/DeclaredIntent-SEAEmission | 3b4a7c930ec88bc91a61ea9fa1e95bfe398bd0cc | 625 | 14 curl-auth-header,generic-api-key | backup/od4-2026-11-16/cycle/DeclaredIntent-SEAEmission | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_DeclaredIntent-SEAEmission.bundle sha256 4a21e09f27d8f7b369f1002739b6ece9274a4d7208cf4390fa626e18f0a68a1d (verify 0/ok) |
| SEA | SEA | branch | cycle/choas-testing | 5f70572b8a488d3b1a49ee3b0312961c197c56ff | 14 | 6 curl-auth-header,generic-api-key | backup/od4-2026-11-16/cycle/choas-testing | 5f70572b8a488d3b1a49ee3b0312961c197c56ff (ref refs/heads/cycle/choas-testing; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | cycle/debt-resolution | 7da1060b35a3d40c3c2a454119259dbaeb6664bb | 749 | 22 curl-auth-header,generic-api-key | backup/od4-2026-11-16/cycle/debt-resolution | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_debt-resolution.bundle sha256 9b84bb59f48fc29b955f5fb16e358709a79679bb31c414c71e95e15a9856a247 (verify 0/ok) |
| SEA | SEA | branch | cycle/end-state-task1 | d5a143147934fc0c95528d12ffa08d7a93201f96 | 618 | 14 curl-auth-header,generic-api-key | backup/od4-2026-11-16/cycle/end-state-task1 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_end-state-task1.bundle sha256 3bab0d6f114e03bef5920e77ddbc9c95bfce3cf9c3502600107817f97fb6b3ed (verify 0/ok) |
| SEA | SEA | branch | cycle/p11-c1A-sea-integrate-cli | 158afc052ba5265ba5511f9f482ee11a5c2dbc42 | 1668 | 84 curl-auth-header,generic-api-key | backup/od4-2026-11-16/cycle/p11-c1A-sea-integrate-cli | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_p11-c1A-sea-integrate-cli.bundle sha256 d99ed7c9cc23ca38ed13272225f04eaaff823bbfdce525e40523e5b592d51315 (verify 0/ok) |
| SEA | SEA | branch | end-state-task1 | 3ac2907e344caff68cfc0b211022c1a2ddbffba2 | 616 | 14 curl-auth-header,generic-api-key | backup/od4-2026-11-16/end-state-task1 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-end-state-task1.bundle sha256 3915c80b7510a32546972f68a2c598b2b7e6a1ea3c536a72c85f860a3eba24ac (verify 0/ok) |
| SEA | SEA | branch | feat/compliance-sink-routing | e0a75e438745b6ba53b8681236b02b0afaded2e6 | 73 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/feat/compliance-sink-routing | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-feat_compliance-sink-routing.bundle sha256 700f1bb23c8088abb8056f413a99c2c3b2f4f25a26766d93ddfbd849844a4622 (verify 0/ok) |
| SEA | SEA | branch | feat/observe-bootstrap-proposals | 28a3c259e8e57a2cb859c77d591bbed915acaa68 | 73 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/feat/observe-bootstrap-proposals | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-feat_observe-bootstrap-proposals.bundle sha256 2cd3dbce769ed4514c01f24da4cb1bb775e1718956a38c7c7c72af3668c46e76 (verify 0/ok) |
| SEA | SEA | branch | fix/pr-405-governance | 1318f8e76ce2d5f5b35152651bd7fc9cd8d4adc4 | 1563 | 34 curl-auth-header,generic-api-key | backup/od4-2026-11-16/fix/pr-405-governance | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-fix_pr-405-governance.bundle sha256 59d00e04c6585affc8f821e098ea0370cee5ab1547089967063fb0316be74944 (verify 0/ok) |
| SEA | SEA | branch | phase1-governed-spine | 0ebeceb3b6e42c940353b2b04b5636228d637654 | 1 | clean | backup/od4-2026-11-16/phase1-governed-spine | 0ebeceb3b6e42c940353b2b04b5636228d637654 |
| SEA | SEA | branch | pr-323 | 7e31d86ad889329b09ca98f7000d8a8dd4706147 | 624 | 14 curl-auth-header,generic-api-key | backup/od4-2026-11-16/pr-323 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-pr-323.bundle sha256 40a02df45a03a373f1d28a1e21f6f72863ad668eff68b203d2589550d52527cb (verify 0/ok) |
| SEA | SEA | branch | promo-main | df14f0267f235c2c833473c31817e6f183bea4c5 | 55 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/promo-main | df14f0267f235c2c833473c31817e6f183bea4c5 (ref refs/heads/promo-main; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | promo-stage | d6d5b6182b5cf3611c86cfb84688c0bf1141e198 | 52 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/promo-stage | d6d5b6182b5cf3611c86cfb84688c0bf1141e198 (ref refs/heads/promo-stage; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | release/0.12.0 | dacb290de5057f92959e366ba3acf9451a2c5683 | 40 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/release/0.12.0 | dacb290de5057f92959e366ba3acf9451a2c5683 (ref refs/heads/release/0.12.0; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | release/0.8.0 | cb85ce8550dad8c1abfe2213e619e23c093edfac | 733 | 22 curl-auth-header,generic-api-key | backup/od4-2026-11-16/release/0.8.0 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-release_0.8.0.bundle sha256 49c55038f503bfe4125b9987bc4930b586dd8269e7d8390cc7b83befee66e638 (verify 0/ok) |
| SEA | SEA | branch | release/v0.10.3 | 02647938a61ffe3c6feffc62ef1cb19b6cbaa5e3 | 1652 | 84 curl-auth-header,generic-api-key | backup/od4-2026-11-16/release/v0.10.3 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-release_v0.10.3.bundle sha256 623dd5f9e4f76f6346247759aabfe10565184bb34511575a3da8c4449763eb9c (verify 0/ok) |
| SEA | SEA | branch | release/v0.13.0 | 7c3fb2b7176d4e23b51621b75aba1cb7751c3b33 | 49 | 21 curl-auth-header,generic-api-key | backup/od4-2026-11-16/release/v0.13.0 | 7c3fb2b7176d4e23b51621b75aba1cb7751c3b33 (ref refs/heads/release/v0.13.0; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | release/v0.9.0 | e25386cd622994de4f8cdbcd4acd252dc7b1f7b3 | 1621 | 71 curl-auth-header,generic-api-key | backup/od4-2026-11-16/release/v0.9.0 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-release_v0.9.0.bundle sha256 3b159f38c2c9d1332993508e7642f46afc3186b17f77edbf89ac3ee9f672b1c2 (verify 0/ok) |
| SEA | SEA | branch | showcase/domain-segments | 8ac458ae3a34a81a03065860d90443f8195de790 | 5 | 13 curl-auth-header,generic-api-key | backup/od4-2026-11-16/showcase/domain-segments | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-branch-showcase_domain-segments.bundle sha256 a33ec7f1197027f91f1b9616640704a62586d074e0b1809e519424e2f5eb98fb (verify 0/ok) |
| SEA | SEA | branch | task17-gap-report-ci | aa393e6dcb055bd8c56f567575cf4946c933321b | 5 | 5 curl-auth-header,generic-api-key | backup/od4-2026-11-16/task17-gap-report-ci | aa393e6dcb055bd8c56f567575cf4946c933321b (ref refs/heads/task17-gap-report-ci; findings blobs all already on origin) [pushed as real branch] |
| SEA | SEA | branch | wip/green-check-followup-2026-07-30 | fd7eb39d9fbe72e1a4a32c6ba57bc4ef9509ba2b | 1 | clean | backup/od4-2026-11-16/wip/green-check-followup-2026-07-30 | fd7eb39d9fbe72e1a4a32c6ba57bc4ef9509ba2b |
| SEA | SEA | stash | 0 | 917c1ae7775507b614ef7d18f24d67504708713a | 81 | 42 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/0 | 917c1ae7775507b614ef7d18f24d67504708713a (ref refs/tags/backup/od4-2026-11-16/stash/SEA/0; findings blobs all already on origin) |
| SEA | SEA | stash | 1 | 2db2c51f75cd8e93970c6374fa9c5008f292875f | 1853 | 105 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/1 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-1.bundle sha256 5c2d3fa32e5168b041826127394d70bd32c521f8a2cebe4b4639b49b408255d2 (verify 0/ok) |
| SEA | SEA | stash | 2 | 23b3e2b2eb4029b4af00f4c27952a285758182dd | 736 | 22 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/2 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-2.bundle sha256 de3629ba0097d922b0aca0cbd820b0f8923076dd855d73c296217d99b915cd7e (verify 0/ok) |
| SEA | SEA | stash | 3 | d3caa6241b4ac29188d0de1d365a78f10df516e1 | 736 | 22 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/3 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-3.bundle sha256 76df5f06ede4877840fae446d2e0276e9b832756eed7439759bc01fea95c2f4b (verify 0/ok) |
| SEA | SEA | stash | 4 | 21be76ff221a7d33ca9cee7c41d4fdb673c8b8a8 | 734 | 22 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/4 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-4.bundle sha256 533a02d9c9872693972c46e0e0a62d11e502c56996b7b32d4cf22d39450b5d67 (verify 0/ok) |
| SEA | SEA | stash | 5 | be51a70fed590055a96eacbaba2b9d0cb1a3bf24 | 34 | 8 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/5 | be51a70fed590055a96eacbaba2b9d0cb1a3bf24 (ref refs/tags/backup/od4-2026-11-16/stash/SEA/5; findings blobs all already on origin) |
| SEA | SEA | stash | 6 | 260c6cb77860f46a036a70109ad2cbf3b0e2bddc | 34 | 8 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/6 | 260c6cb77860f46a036a70109ad2cbf3b0e2bddc (ref refs/tags/backup/od4-2026-11-16/stash/SEA/6; findings blobs all already on origin) |
| SEA | SEA | stash | 7 | 2d2d49f37d71584977d6c7b18a7ca3e03e334252 | 18 | 12 curl-auth-header,generic-api-key(stash-diff) | backup/od4-2026-11-16/stash/SEA/7 | 2d2d49f37d71584977d6c7b18a7ca3e03e334252 (ref refs/tags/backup/od4-2026-11-16/stash/SEA/7; findings blobs all already on origin) |
| SEA | SEA | stash | 8 | c0e5a420e76b67c974a46b95992fb72f5a1c9e9f | 472 | 8 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/8 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-8.bundle sha256 fd1db24d9a141bf6f956306105c8a045092db40357c931b304f6e1f22922747b (verify 0/ok) |
| SEA | SEA | stash | 9 | b34edf2540d9f637fb57bbaa7931175f425f5f26 | 434 | 12 curl-auth-header,generic-api-key(stash-diff) | backup/od4-2026-11-16/stash/SEA/9 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-9.bundle sha256 cbb0b3b353155471b8791251abcd68d4eaa7edce845be134e7fe529d876bce19 (verify 0/ok) |
| SEA | SEA | stash | 10 | 30240e36d8327dce8aba6148d4af2979e436b33d | 336 | 6 curl-auth-header,generic-api-key | backup/od4-2026-11-16/stash/SEA/10 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-10.bundle sha256 586756eb55f7c2030433a2de790ed907a65b61de097c08630ea7432af52c750b (verify 0/ok) |
| SEA | SEA | stash | 11 | c4a0fb183648cf2027db9e8180eb1d512b3d49ad | 237 | 3 generic-api-key | backup/od4-2026-11-16/stash/SEA/11 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-11.bundle sha256 7393c29426fa78b90c66707a7b7ea2dbec352a058d2540c757084b11a9cd6be7 (verify 0/ok) |
| SEA | SEA | stash | 12 | c8c67c4b77dc47e5a6737344f30cc93237b64418 | 216 | 5 generic-api-key(stash-diff) | backup/od4-2026-11-16/stash/SEA/12 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-12.bundle sha256 145b13b937d1c6066b1a873a1df5a836ba1fd29f8521117e898a580c127eb387 (verify 0/ok) |
| SEA | SEA | stash | 13 | 8698946a89cd35bfbd43f5b3640f36ee4d882103 | 210 | 2 generic-api-key | backup/od4-2026-11-16/stash/SEA/13 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-13.bundle sha256 60e57031bd5cea6dbaf054a5d1f48fd0b08d8830c3bbe5d7a2bcab709fab76d5 (verify 0/ok) |
| SEA | SEA | stash | 14 | 6b199d693a83ddb8511ad61053d540fee042e4e4 | 116 | 1 generic-api-key | backup/od4-2026-11-16/stash/SEA/14 | HELD: bundle /home/sprime01/backups/od4-2026-11-16/SEA-stash-14.bundle sha256 c2f89f95ff2ce195d95ec712738a02904bbb0f62e32afd7a60a16734099cddff (verify 0/ok) |
| sea-rs | sea-rs | branch | ci/release-provenance | 730add6e0491ec922443c4e414426179f1c6b475 | 1 | clean | backup/od4-2026-11-16/ci/release-provenance | 730add6e0491ec922443c4e414426179f1c6b475 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/b08-domainforge-engine-fail-closed | 9268aaccf62228553adede2219b83432cb980572 | 3 | clean | backup/od4-2026-11-16/fix/b08-domainforge-engine-fail-closed | 9268aaccf62228553adede2219b83432cb980572 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/b12-seed-cell-and-backup | 79bb05ebdcec252a0cd458b52711b0a33e2fc11d | 1 | clean | backup/od4-2026-11-16/fix/b12-seed-cell-and-backup | 79bb05ebdcec252a0cd458b52711b0a33e2fc11d (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/go-jose-4.1.6 | d0177df8a90d1e6aea287d72415ff9a8dacbb783 | 1 | clean | backup/od4-2026-11-16/fix/go-jose-4.1.6 | d0177df8a90d1e6aea287d72415ff9a8dacbb783 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/restore-release-attestation | 09ff8cf79f037f28404495a31502b1ca212ed80b | 1 | clean | backup/od4-2026-11-16/fix/restore-release-attestation | 09ff8cf79f037f28404495a31502b1ca212ed80b (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/server-ambiguous-declare-keeps-lease | 961dd478bd8760991df6527750ce1b99b68aef27 | 2 | clean | backup/od4-2026-11-16/fix/server-ambiguous-declare-keeps-lease | 961dd478bd8760991df6527750ce1b99b68aef27 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/settlement-attempt-claim-settlement | 749a21733584c9e937f028e2a0c11a3ef674f472 | 22 | clean | backup/od4-2026-11-16/fix/settlement-attempt-claim-settlement | 749a21733584c9e937f028e2a0c11a3ef674f472 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/settlement-attempt-claim-settlement-main | ccac67a28befe53334100fe08ba8ec80d2254277 | 1 | clean | backup/od4-2026-11-16/fix/settlement-attempt-claim-settlement-main | ccac67a28befe53334100fe08ba8ec80d2254277 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | fix/settlement-orphan-guard | 15c7ccf67e15658dd8691fa69b4e2c27a4d5d8e0 | 2 | clean | backup/od4-2026-11-16/fix/settlement-orphan-guard | 15c7ccf67e15658dd8691fa69b4e2c27a4d5d8e0 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | security/identity-binding-version | af02252b510b7ba2e44cda5643ed73c438b74ed8 | 1 | clean | backup/od4-2026-11-16/security/identity-binding-version | af02252b510b7ba2e44cda5643ed73c438b74ed8 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | t1/case-task-approval | d5e34c541a6b2b80df6bf4e29377b372fd9f945d | 1 | clean | backup/od4-2026-11-16/t1/case-task-approval | d5e34c541a6b2b80df6bf4e29377b372fd9f945d (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | branch | test/settlement-class-fail-closed | f9fe5818001d594fc221d3fd812527aaef49b7c4 | 2 | clean | backup/od4-2026-11-16/test/settlement-class-fail-closed | f9fe5818001d594fc221d3fd812527aaef49b7c4 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sea-rs | sea-rs | stash | 0 | d00f3c28bcd81fe4ecac505add00a4dd9613c58a | 257 | clean | backup/od4-2026-11-16/stash/sea-rs/0 | d00f3c28bcd81fe4ecac505add00a4dd9613c58a (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| cognate | cognate | branch | pr-70 | 0c38b341a66df60d50d04197d249631970bb39ac | 255 | 4 generic-api-key | backup/od4-2026-11-16/pr-70 | 0c38b341a66df60d50d04197d249631970bb39ac (ref refs/tags/backup/od4-2026-11-16/pr-70; findings blobs all already on origin) |
| cognate-workbench | cognate-workbench | branch | ci/install-pinned-domainforge | 1d4cc16dcdc100eac645b93663eb5c2c2ab0d40d | 1 | clean | backup/od4-2026-11-16/ci/install-pinned-domainforge | 1d4cc16dcdc100eac645b93663eb5c2c2ab0d40d |
| cognate-workbench | cognate-workbench | branch | feat/desktop-terminal-browser | e45e97d1baf50cb5e02446a50df8865c1a1c7202 | 19 | clean | backup/od4-2026-11-16/feat/desktop-terminal-browser | e45e97d1baf50cb5e02446a50df8865c1a1c7202 |
| cognate-workbench | cognate-workbench | branch | feat/horizon-connect | c4096b9cac720ea992d714635334ab718eb4e59f | 11 | clean | backup/od4-2026-11-16/feat/horizon-connect | HELD: bundle /home/sprime01/backups/od4-2026-11-16/cognate-workbench-branch-feat_horizon-connect.bundle sha256 0fef7939487082d477a4ab0fc76d4cf9225f863523643fd808f15c18c64eba06 (verify 0/ok) |
| domainforge | domainforge | branch | rel-0.22.1 | c7dc274460df279588837346a2090c0ee9f4731a | 2 | clean | backup/od4-2026-11-16/rel-0.22.1 | c7dc274460df279588837346a2090c0ee9f4731a |
| gauntlet | gauntlet | branch | fix/test-schema-teardown | 2710a0e2160cc0779a4f67cd52133add7f8f5ed0 | 1 | clean | backup/od4-2026-11-16/fix/test-schema-teardown | 2710a0e2160cc0779a4f67cd52133add7f8f5ed0 |
| ideal | ideal | branch | jollimemory/summaries/v3 | 7f4a3e50f35c0376685963ea728519caaae3ca30 | 2 | clean | backup/od4-2026-11-16/jollimemory/summaries/v3 | 7f4a3e50f35c0376685963ea728519caaae3ca30 |
| sxr | sxr | branch | chore/domainforge-0.21.1 | ec1c4167cada5375d4a07f7ecef24e74aa30aa50 | 1 | clean | backup/od4-2026-11-16/chore/domainforge-0.21.1 | ec1c4167cada5375d4a07f7ecef24e74aa30aa50 |
| sxr | sxr | branch | feat/command-verifier | a68a4b89bbda044033af016cb7cf5b322cae5e02 | 1 | clean | backup/od4-2026-11-16/feat/command-verifier | a68a4b89bbda044033af016cb7cf5b322cae5e02 |
| sxr | sxr | branch | feat/native-binary-distribution | c239d5c6748373434b33c483f9ee8cf67c3eb323 | 8 | clean | backup/od4-2026-11-16/feat/native-binary-distribution | c239d5c6748373434b33c483f9ee8cf67c3eb323 |
| sxr | sxr | branch | feat/verifier-completed-at | 3fe3a5016c6945bc8a9863ca5da60cbd55d71feb | 1 | clean | backup/od4-2026-11-16/feat/verifier-completed-at | 3fe3a5016c6945bc8a9863ca5da60cbd55d71feb |
| sxr | sxr | branch | fix/bundled-sqlite | cf6e91db567b62d3eb8dbb148da143a361348feb | 1 | clean | backup/od4-2026-11-16/fix/bundled-sqlite | cf6e91db567b62d3eb8dbb148da143a361348feb |
| sxr | sxr | branch | fix/ledger-read-only-no-create | ae4a7cefd098b91f4ec21c2ce7742cc067075140 | 1 | clean | backup/od4-2026-11-16/fix/ledger-read-only-no-create | ae4a7cefd098b91f4ec21c2ce7742cc067075140 |
| sxr | sxr | branch | jollimemory/summaries/v3 | 66d96528c4dbea582465e725575fb2b54dcfe17e | 2 | clean | backup/od4-2026-11-16/jollimemory/summaries/v3 | 66d96528c4dbea582465e725575fb2b54dcfe17e |
| Context_Kernel | Context_Kernel | dirty | Context_Kernel | a67e69811631a775a8e6b65e30e806943cefeaf4 | 1 | clean | backup/od4-2026-11-16/dirty/Context_Kernel | a67e69811631a775a8e6b65e30e806943cefeaf4 |
| Horizon | Horizon | dirty | Horizon | a88b4dc925b28997d8cf64efb1f83beaaaafe8c8 | 1 | 3 generic-api-key,sourcegraph-access-token | backup/od4-2026-11-16/dirty/Horizon | HELD: bundle /home/sprime01/backups/od4-2026-11-16/Horizon-dirty-Horizon.bundle sha256 02fd86d7db89bf37109445b6d479557f477fccc5b6f04b1cef21a0e31e3c99a1 (verify 0/ok) |
| Horizon | Horizon | dirty | Horizon-core | 1457e31c7ee98178fa860425b07743c7bba6ef1c | 4 | clean | backup/od4-2026-11-16/dirty/Horizon-core | 1457e31c7ee98178fa860425b07743c7bba6ef1c |
| SEA | SEA | dirty | SEA | e43c8f6df98afdad0e01c780afe846dd13a6f22e | 1 | clean | backup/od4-2026-11-16/dirty/SEA | e43c8f6df98afdad0e01c780afe846dd13a6f22e |
| SWE_SEED | SWE_SEED | dirty | SWE_SEED | 177074cf693ae580eecc6a0e065631f776b2007e | 1 | clean | backup/od4-2026-11-16/dirty/SWE_SEED | 177074cf693ae580eecc6a0e065631f776b2007e |
| sxr | sxr | dirty | df022-sxr | 03d938f3ccf1b25c45892b92f12a54862c8a948c | 1 | clean | backup/od4-2026-11-16/dirty/df022-sxr | 03d938f3ccf1b25c45892b92f12a54862c8a948c |
| sxr | sxr | dirty | fde-sxr-ingest-json | 11999e17277306fccfb850ae2733cdc428965d05 | 1 | clean | backup/od4-2026-11-16/dirty/fde-sxr-ingest-json | 11999e17277306fccfb850ae2733cdc428965d05 |
| sxr | sxr | dirty | fde-sxr-integration | cd57a4e79ed3d5dcc734cb4d49e09e91ac1a49ce | 1 | clean | backup/od4-2026-11-16/dirty/fde-sxr-integration | cd57a4e79ed3d5dcc734cb4d49e09e91ac1a49ce |
| sxr | sxr | dirty | fde-sxr-release-sums | 416b9c72b765ff54ffd8b97b5d031ca24dc6704a | 1 | clean | backup/od4-2026-11-16/dirty/fde-sxr-release-sums | 416b9c72b765ff54ffd8b97b5d031ca24dc6704a |
| ast-grep-mcp-rs | ast-grep-mcp-rs | dirty | ast-grep-mcp-rs | 614f8e046ed192d8c77d9e27697d331df9d8a3b7 | 1 | clean | backup/od4-2026-11-16/dirty/ast-grep-mcp-rs | 614f8e046ed192d8c77d9e27697d331df9d8a3b7 |
| cep | cep | dirty | cep | ad834bc1f733719458e1d3acb0562eea1af224de | 1 | clean | backup/od4-2026-11-16/dirty/cep | ad834bc1f733719458e1d3acb0562eea1af224de |
| cognate | cognate | dirty | cognate-horizon | 150fc1d9753295818874a7a2f6f390be067c0eab | 1 | clean | backup/od4-2026-11-16/dirty/cognate-horizon | 150fc1d9753295818874a7a2f6f390be067c0eab |
| cognate | cognate | dirty | cognate-identity-wt | 04ac0fca0afc24c921448acd3d93161497527d15 | 1 | clean | backup/od4-2026-11-16/dirty/cognate-identity-wt | 04ac0fca0afc24c921448acd3d93161497527d15 |
| cognate-workbench | cognate-workbench | dirty | cognate-workbench | 5e8957547c354703f2f4971c3270b501d24a4fcc | 20 | clean | backup/od4-2026-11-16/dirty/cognate-workbench | 5e8957547c354703f2f4971c3270b501d24a4fcc |
| domainforge | domainforge | dirty | domainforge | 96420e51c70630ef7c19dea7946feb65047dd3f7 | 1 | clean | backup/od4-2026-11-16/dirty/domainforge | 96420e51c70630ef7c19dea7946feb65047dd3f7 |
| godspeed_agent | godspeed_agent | dirty | godspeed_agent | c4f3accb3edd84cd556345d7909276866696b811 | 1 | clean | backup/od4-2026-11-16/dirty/godspeed_agent | c4f3accb3edd84cd556345d7909276866696b811 |
| sea-rs | sea-rs | dirty | sea-rs | 50dc7acffbbe0bf18f9c4674732508d9a2cd5c48 | 1 | clean | backup/od4-2026-11-16/dirty/sea-rs | 50dc7acffbbe0bf18f9c4674732508d9a2cd5c48 (pushed 2026-11-16 with --no-verify, owner-approved; verified) |
| sxr | sxr | dirty | sxr | 42a30611895d6216655b0e9707526976da8e2c06 | 9 | clean | backup/od4-2026-11-16/dirty/sxr | 42a30611895d6216655b0e9707526976da8e2c06 |

### Held items

| repo | item | reason | bundle | sha256 |
|---|---|---|---|---|
| SEA | branch backup-before-scrub-20260202 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-backup-before-scrub-20260202.bundle | 6ae08cdf40161d88b54b99d2a5cb3bca6aa74f0d1d298caeb2c95959f8d48495 |
| SEA | branch backup/pr-518-pre-cla-rewrite | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key), tests/e2e/test_cognitive_extension_e2e.py (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-backup_pr-518-pre-cla-rewrite.bundle | 0732694994d5513eac5cc035ff8a161f2f1c03c7f97b9e367bda2de3213f04cb |
| SEA | branch coderabbit-fixes | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-coderabbit-fixes.bundle | 9dd31c320a18d0b61528b3ec71701081d66dbc61d327580e499cadfd59e103df |
| SEA | branch cycle/DeclaredIntent-SEAEmission | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_DeclaredIntent-SEAEmission.bundle | 4a21e09f27d8f7b369f1002739b6ece9274a4d7208cf4390fa626e18f0a68a1d |
| SEA | branch cycle/debt-resolution | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_debt-resolution.bundle | 9b84bb59f48fc29b955f5fb16e358709a79679bb31c414c71e95e15a9856a247 |
| SEA | branch cycle/end-state-task1 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_end-state-task1.bundle | 3bab0d6f114e03bef5920e77ddbc9c95bfce3cf9c3502600107817f97fb6b3ed |
| SEA | branch cycle/p11-c1A-sea-integrate-cli | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key), tests/e2e/test_cognitive_extension_e2e.py (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-cycle_p11-c1A-sea-integrate-cli.bundle | d99ed7c9cc23ca38ed13272225f04eaaff823bbfdce525e40523e5b592d51315 |
| SEA | branch end-state-task1 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-end-state-task1.bundle | 3915c80b7510a32546972f68a2c598b2b7e6a1ea3c536a72c85f860a3eba24ac |
| SEA | branch feat/compliance-sink-routing | finding blob not on origin: tests/e2e/test_cognitive_extension_e2e.py (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-branch-feat_compliance-sink-routing.bundle | 700f1bb23c8088abb8056f413a99c2c3b2f4f25a26766d93ddfbd849844a4622 |
| SEA | branch feat/observe-bootstrap-proposals | finding blob not on origin: tests/e2e/test_cognitive_extension_e2e.py (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-branch-feat_observe-bootstrap-proposals.bundle | 2cd3dbce769ed4514c01f24da4cb1bb775e1718956a38c7c7c72af3668c46e76 |
| SEA | branch fix/pr-405-governance | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-fix_pr-405-governance.bundle | 59d00e04c6585affc8f821e098ea0370cee5ab1547089967063fb0316be74944 |
| SEA | branch pr-323 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-pr-323.bundle | 40a02df45a03a373f1d28a1e21f6f72863ad668eff68b203d2589550d52527cb |
| SEA | branch release/0.8.0 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-release_0.8.0.bundle | 49c55038f503bfe4125b9987bc4930b586dd8269e7d8390cc7b83befee66e638 |
| SEA | branch release/v0.10.3 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key), tests/e2e/test_cognitive_extension_e2e.py (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-release_v0.10.3.bundle | 623dd5f9e4f76f6346247759aabfe10565184bb34511575a3da8c4449763eb9c |
| SEA | branch release/v0.9.0 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-branch-release_v0.9.0.bundle | 3b159f38c2c9d1332993508e7642f46afc3186b17f77edbf89ac3ee9f672b1c2 |
| SEA | branch showcase/domain-segments | finding blob not on origin: tests/e2e/test_cognitive_extension_e2e.py (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-branch-showcase_domain-segments.bundle | a33ec7f1197027f91f1b9616640704a62586d074e0b1809e519424e2f5eb98fb |
| SEA | stash 1 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key), tests/e2e/test_cognitive_extension_e2e.py (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-stash-1.bundle | 5c2d3fa32e5168b041826127394d70bd32c521f8a2cebe4b4639b49b408255d2 |
| SEA | stash 2 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-stash-2.bundle | de3629ba0097d922b0aca0cbd820b0f8923076dd855d73c296217d99b915cd7e |
| SEA | stash 3 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-stash-3.bundle | 76df5f06ede4877840fae446d2e0276e9b832756eed7439759bc01fea95c2f4b |
| SEA | stash 4 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-stash-4.bundle | 533a02d9c9872693972c46e0e0a62d11e502c56996b7b32d4cf22d39450b5d67 |
| SEA | stash 8 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) ... | /home/sprime01/backups/od4-2026-11-16/SEA-stash-8.bundle | fd1db24d9a141bf6f956306105c8a045092db40357c931b304f6e1f22922747b |
| SEA | stash 9 | finding blob not on origin: .secrets.env.sops.example (generic-api-key), docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-stash-9.bundle | cbb0b3b353155471b8791251abcd68d4eaa7edce845be134e7fe529d876bce19 |
| SEA | stash 10 | finding blob not on origin: docs/Integration_Handbook/API_Patterns.md (curl-auth-header), docs/howto/manage-secrets.md (generic-api-key), docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-stash-10.bundle | 586756eb55f7c2030433a2de790ed907a65b61de097c08630ea7432af52c750b |
| SEA | stash 11 | finding blob not on origin: docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-stash-11.bundle | 7393c29426fa78b90c66707a7b7ea2dbec352a058d2540c757084b11a9cd6be7 |
| SEA | stash 12 | finding blob not on origin: docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-stash-12.bundle | 145b13b937d1c6066b1a873a1df5a836ba1fd29f8521117e898a580c127eb387 |
| SEA | stash 13 | finding blob not on origin: docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-stash-13.bundle | 60e57031bd5cea6dbaf054a5d1f48fd0b08d8830c3bbe5d7a2bcab709fab76d5 |
| SEA | stash 14 | finding blob not on origin: docs/specs/shared/sds/020-ci-semantic-gates.md (generic-api-key) | /home/sprime01/backups/od4-2026-11-16/SEA-stash-14.bundle | c2f89f95ff2ce195d95ec712738a02904bbb0f62e32afd7a60a16734099cddff |
| sea-rs | branch ci/release-provenance | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-ci_release-provenance.bundle | 460d58e2dd35dbcbe5352a83a3d7a64076dafbf1d3514cb154c96a770cca64da |
| sea-rs | branch fix/b08-domainforge-engine-fail-closed | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_b08-domainforge-engine-fail-closed.bundle | 81f0f83d22b8f1607acc071743361fd5539ae75c84addee30069a73e70f2f4a2 |
| sea-rs | branch fix/b12-seed-cell-and-backup | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_b12-seed-cell-and-backup.bundle | 01472b24472f66da98c319f58eb73c074fef2db71ab6b882851de99991a9706b |
| sea-rs | branch fix/go-jose-4.1.6 | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_go-jose-4.1.6.bundle | 0557e7c779a67cd4eaf9a4d04e83390b99fcbfbe1412a00900e3838510624af1 |
| sea-rs | branch fix/restore-release-attestation | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_restore-release-attestation.bundle | 84e0d2e4511179b86ac4ec6184926a9efcb6fdb2dd9c2e89a5491e490790d0e3 |
| sea-rs | branch fix/server-ambiguous-declare-keeps-lease | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_server-ambiguous-declare-keeps-lease.bundle | a54eeb9455af07c4b8b92c27cefcf63f3f972974aa14b471fceeb22a2b8c09b2 |
| sea-rs | branch fix/settlement-attempt-claim-settlement | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_settlement-attempt-claim-settlement.bundle | 86f91a0cb4890d4f6e0ad3f23eb81af258f27ef7b2c49e730fa2c81633b30445 |
| sea-rs | branch fix/settlement-attempt-claim-settlement-main | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_settlement-attempt-claim-settlement-main.bundle | 3029a57d48b61ceaa43553024ae9206861131b29310bb032048740811f737848 |
| sea-rs | branch fix/settlement-orphan-guard | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-fix_settlement-orphan-guard.bundle | 43118ea015600b6c73a95e1164a00fdb68d59f1bfecfac0c9957f3e3e7065f4c |
| sea-rs | branch security/identity-binding-version | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-security_identity-binding-version.bundle | cf504c5af576d053c1c512e0211aa12e84f4be35582cef7142c398c704aa8e31 |
| sea-rs | branch t1/case-task-approval | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-t1_case-task-approval.bundle | cb0c5dda4a79444b0d070acc265dc43fc7d358396b5d5793bda70484973780e7 |
| sea-rs | branch test/settlement-class-fail-closed | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-branch-test_settlement-class-fail-closed.bundle | ab30617d57aae92a8c5aa6802ddc09cbfe38f38726e27404ba31c3134087cec3 |
| sea-rs | stash 0 | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-stash-0.bundle | aa1ec4871321b9ad1874bd27092627a08d52754b97525bdb45d156274735cb2b |
| cognate-workbench | branch feat/horizon-connect | size limit (files >100 MB rejected by GitHub) | /home/sprime01/backups/od4-2026-11-16/cognate-workbench-branch-feat_horizon-connect.bundle | 0fef7939487082d477a4ab0fc76d4cf9225f863523643fd808f15c18c64eba06 |
| Horizon | dirty Horizon | finding blob not on origin: .agents/reports/2026-11-05-estate-reconciliation/checkpoints/run.log (generic-api-key), .agents/reports/2026-11-05-estate-reconciliation/inventory/raw/gitleaks.txt (sourcegraph-access-token) | /home/sprime01/backups/od4-2026-11-16/Horizon-dirty-Horizon.bundle | 02fd86d7db89bf37109445b6d479557f477fccc5b6f04b1cef21a0e31e3c99a1 |
| sea-rs | dirty sea-rs | local pre-push hook (status-check); --no-verify push was not permitted by the permission system | /home/sprime01/backups/od4-2026-11-16/sea-rs-dirty-sea-rs.bundle | 303387711cdf4ad81a960892ac715e6c56d5ca081ec43f3da2b0e355c8c04d6b |

### Skipped / not backed up

- No remote; not backed up: NewSea, demo-ws, godspeed-readiness (and other local-only repos).
- Non-GodSpeed checkouts out of scope.
- Horizon dirty tree and Cognate-Workbench feat/horizon-connect: being handled by the coordinator (scrub / binary removal); bundles above are safety copies only.
- SEA stashes/branches/dirty tree held only in bundles unless shown pushed above.

### Owner-directed follow-ups (2026-11-16)
- **SEA-Forge:** the 14 items above were held by the local pre-push hook. They were pushed with `--no-verify`, which the owner approved explicitly ("use --no-verify for the SEA-Forge backup tags"). `git ls-remote` shows 14 `backup/od4-2026-11-16/*` tags. Their bundles in ~/backups/od4-2026-11-16/ remain as a second copy.
- **Horizon dirty tree:** at the owner's direction ("scrub it"), the secret matches were redacted in place in two uncommitted files: `.agents/reports/2026-11-05-estate-reconciliation/checkpoints/run.log` (1 × generic-api-key) and `.../inventory/raw/gitleaks.txt` (2 × sourcegraph-access-token). A rescan of that directory finds no leaks. A snapshot commit, f5c37aef1efc4d07e2ec58473acb3b839eb7f02b, made through a temporary index without changing the checkout's status, scanned clean and was pushed as `backup/od4-2026-11-16/dirty/Horizon` (verified).
- **Cognate-Workbench `feat/horizon-connect`:** at the owner's direction ("remove unnecessary installer files"), `dist-package/` was removed from the branch's 75 unique commits in a scratch clone using `git filter-branch --index-filter`. That folder held 43 build-output files, including installers of up to 255 MB, and none of it exists on main. The rewritten tip 58cccd52f41aa3f3f080bca5cc79aaf6e62691e9 is pushed as `backup/od4-2026-11-16/feat/horizon-connect` (verified). The original local branch c4096b9 is untouched and bundled. One gitleaks finding (generic-api-key, test/J-CFG-provider-test.test.ts) is a blob already on origin, so there is no new exposure. The push ran before that check completed; it was verified afterwards.
- **Old SEA:** 10 branches are on GitHub. 16 branches and stashes 1–4 and 8–14 are held in bundles because their findings are in blobs not yet on GitHub. They are mostly example and doc files.

### Old SEA disposition (2026-11-16, owner: "if i dont need them, delete them … only keeping SEA around to mine for ideas … it was a prototype")
- Rejected and deleted locally:
  - 16 HELD branches: backup-before-scrub-20260202, backup/pr-518-pre-cla-rewrite, coderabbit-fixes, cycle/DeclaredIntent-SEAEmission, cycle/debt-resolution, cycle/end-state-task1, cycle/p11-c1A-sea-integrate-cli, end-state-task1, feat/compliance-sink-routing, feat/observe-bootstrap-proposals, fix/pr-405-governance, pr-323, release/0.8.0, release/v0.10.3, release/v0.9.0, showcase/domain-segments. Their tip SHAs are in the rows above.
  - Stashes 1–4 and 8–14, each dropped by verified SHA.
  - Their 27 bundles in ~/backups/od4-2026-11-16/.
  - None of them was pushed (secret-scan findings), so no copy remains.
- Kept: GitHub `main` and the 10 pushed branches (12 remote branches in total, counting the existing default). Local stashes 0, 5, 6 and 7 are also on GitHub as backup tags. The untracked file `tmp.isLFGM3Z3F` in ~/projects/SEA is untouched.
- GodSpeedAI/SEA is ARCHIVED (read-only, private) as of 2026-11-16. It is excluded from all further consolidation.

## Cognate-Workbench consolidation (2026-11-16)

Main after consolidation: 4594bb7 (PR #36 fe59ade, PR #37 4594bb7). Main has no branch protection or required checks.

| item | disposition | evidence |
|---|---|---|
| feat/desktop-terminal-browser (origin head + 19 local-only capability-runtime commits, tag e45e97d) | integrated | PR #36 squash fe59ade; capabilityRuntime.ts, bundleStore.ts, authorityIndex.ts identical to the tag; only governance.ts etc. differ because main kept its newer SeaForgeAuthority client (#29) |
| feat/ui-production-readiness (PR #27) | integrated | ancestor of the desktop lineage, landed by #36; PR #27 closed |
| PR #28 (desktop terminal and browser, base feat/ui-production-readiness) | integrated | landed by #36; closed |
| feat/horizon-connect (cleaned tag 58cccd52) | superseded | its 5 unique commits are on main via #35; the UI lineage landed by #36; only difference is caseProposeStep (+ test), reverted on main on purpose (commit 8e59664) |
| chore/domainforge-0.22.0, fde/cognate-0.9.1, fde/realitytrace-0.4.0, fix/live-test-domainforge-version, ci/install-pinned-domainforge | integrated | squash PRs #34, #32, #33, #31, #6; `git cherry origin/main` shows no '+' |
| fde/integration | integrated | PR #35; tree identical to main at the time |
| feature/expert-teams, test/red-session-workspace-task-contracts-20261020 (PR #2) | integrated | `git cherry` empty (PR #16/#17 contents on main); PR #2 closed |
| worktrees df022-cognate-workbench, fde-workbench-*, cognate-workbench-horizon, gsr-heads/cognate-workbench, /tmp/wb-baseline | removed | clean, branches dispositioned (gsr-heads/cognate-workbench kept: pinned head checkout) |
| ~/projects/cognate-workbench (dirty, 37 files, on feat/desktop-terminal-browser) | left alone | owner's screenshots and reports; local branch stays checked out there |
| backup tags desktop-terminal-browser, horizon-connect, ci/install-pinned-domainforge | deleted | integrated or superseded as above |
| backup tag dirty/cognate-workbench | kept | holds the owner's dirty tree; not integrated |

CI optimization (WP-58, PR #37): verify is now checks + 4 test shards (own postgres each, timings in ci/test-timings.json) + tauri-linux + aggregate `verify`; tool caches; clean-install only on main and PRs; actions pinned by SHA. Cold sharded run 424 s wall, warm 275 s, against about 895 s for the single-job verify.

Suite fixes needed to land #36: shared happy-dom lifecycle in two more tests, test/preload-react-dom.ts (react-dom must load with a DOM), Postgres max_connections.

Next work package (capability runtime): T2 remainder, T3, T4 per .agents/plans/capability-bundles/HANDOFF.md.

## SEA-Forge consolidation (2026-11-16)

Main before: 7c083c86 (PR #71). Ruleset on main: squash only, required checks `gate` and `conventional-commit` (strict), review threads must be resolved, linear history. Every PR must also add a CURRENT_STATUS revision. Merges below were squash merges after green required checks and resolved CodeRabbit threads.

### Merged

| PR | what | merge SHA |
|---|---|---|
| #101 | Dependabot actions group (devbox-install-action 0.16.0 etc.) | 5825ce01 |
| #105 | CI optimization (see below) | 9b8aca84 |
| #102 | compatible-minor group + jsonschema 0.60 (tokio held at 1.53.3), replaces #100 and #70 | d7bc7c7c |
| #103 | Horizon desk support (rebased #91; fixes the planner fixtures) | 4b42302f |
| #106 | Linux ARM64 (Jetson) packaging, squashed port of feat/arm64-jetson-packaging | fce99e39 |
| #104 | EGM contract READMEs (rebased docs/egm-bootstrap-readmes) | 2f35ad27 |
| #96 | release-please 0.5.0 | NOT MERGED: gate green on head 5da27366, merge blocked by the permission classifier; needs a human or coordinator `gh pr merge 96 --squash` |

### Hardening / OpenShell verdict

feat/hardening-capability-harvest is PR #71 (7c083c86; local tip 7bd0cb01 equals the PR head). feat/openshell-capability, feat/macos-app-sandbox-jail, feat/native-packages, feat/xplat-consolidate, feat/xplat-hardening, feat/xplat-win-sandbox-tests, fix/review1-sandbox, fix/review1-windows, fix/review2-sandbox, fix/review2-windows and ci/openshell-live are all ancestors of the head of PR #67 (b7f2d112, squashed as 0fc236f3): `git merge-base` of each tip with b7f2d112 is the tip itself. Main's #67 plus #71 is the stronger implementation (it adds the F05/F08/F15/F17/F32/F33 mechanism closures on top). Nothing was left to port.

### Branches

| item | disposition | evidence |
|---|---|---|
| chore/domainforge-0.22.0 (e71ed476) | integrated | PR #99 be63b711; tip equals PR head |
| fde/domainforge-0.21.2 (5cad1166) | integrated | PR #94 0439abb0; tip equals PR head |
| fde/fix-e2e-world-loop (929daf03) | integrated | PR #97 09b628ec; tip equals PR head |
| fde/planning-baseline (0bab4bd2) | integrated | PR #98 f8ae13c9; tip equals PR head |
| fde/rustsec-2026-0459-0486 (4dba46ed) | integrated | PR #95 c2d6fbdc; `git cherry` has no '+' |
| feat/hardening-capability-harvest (7bd0cb01) | integrated | PR #71 7c083c86; tip equals PR head |
| feat/openshell-capability, macos-app-sandbox-jail, native-packages, xplat-consolidate, xplat-hardening, xplat-win-sandbox-tests (tips 73516733, e0fd2cf7, cda24a8a, e7e275d9, 129748db, 69397b4c) | integrated | ancestors of the #67 head b7f2d112; #67 is 0fc236f3 |
| fix/review1-sandbox, review1-windows, review2-sandbox, review2-windows (5641b4ee, 4aaef3b4, 4ba67d34, a9cb3636) | integrated | ancestors of the #67 head b7f2d112 |
| ci/openshell-live (f97199c9) | integrated | ancestor of the #67 head; workflow is on main as openshell-live.yml (its dead push trigger removed in #105) |
| feat/arm64-jetson-packaging (e096e9c1) | integrated | squashed port in PR #106; dropped arm-probe.yml and the dead push trigger |
| feat/horizon-desk-support (6e8b6aff; PR #91) | integrated | rebased as PR #103; #91 closed (the original branch was not force-pushed) |
| docs/egm-bootstrap-readmes (b072ec50) | integrated | rebased as PR #104. The local branch stays: it is checked out in ~/projects/sea-rs |
| dependabot #101 | integrated | 5825ce01 |
| dependabot #100 compatible-minor | superseded | #102 (tokio 1.54.1 held back: HTTP test stubs reset the connection with the request unread) |
| dependabot #70 jsonschema 0.58 to 0.60 | superseded | #102 |
| dependabot #92, #93 | already closed by Dependabot | replaced by #101, #100 |
| fix/b08-domainforge-engine-fail-closed (origin e4429466, local 9268aacc), fix/b12-seed-cell-and-backup (origin 6f52a400) | superseded | #66 4e67dfca and #65 7f4be4b8 (the -v2 branches); 0 of the added code lines are missing on main |
| fix/settlement-attempt-claim-settlement (749a2173; PR #80 closed) | superseded | #79, #81, #82, #83 on main; 18 of 624 added lines in cep.rs differ (rewording), none functional |
| fix/server-ambiguous-declare-keeps-lease (961dd478) | superseded | #84 bdb42a52 and #89 7db070a5; remaining differences are rewording |
| fix/settlement-orphan-guard, test/settlement-class-fail-closed | integrated | #89 7db070a5, #88 bb0190ab; tips equal PR heads |
| ci/release-provenance, fix/go-jose-4.1.6, fix/restore-release-attestation, fix/settlement-attempt-claim-settlement-main, security/identity-binding-version, t1/case-task-approval, t1/contracts-home, t1/rename-case-task-approval, fix/b12 local | integrated | PR #58, #78, #64, #81, #77, #87, #85; `git cherry origin/main` has no '+' |
| ci/openshell-windows (e658b030, 4 commits) | rejected | a manual 90-minute build of upstream OpenShell for Windows MXC; MXC stays a blocked backend and nothing consumes the artifact |
| ci/macos-app-sandbox-probe (582162ef, 3 commits) | rejected | one-off evidence probe for the jail; superseded by the portability macOS lane and the governed-runtime-probing skill |
| fde/integration (5720674c, 46 commits ahead of main, newest 2026-11-15) | kept | FDE integration lane (S2 work, WP-21a); owned by the FDE convergence mission |
| release-please--branches--main | kept | PR #96 |
| main | kept | |

### Worktrees, stashes, tags

- Worktrees for dispositioned branches (arm64, consol, df022-sea-rs, fde-sea-rs-df0212, -e2e-loop, -planning, -rustsec, harvest, macjail, macsb, oscap, oslive, oswin, pkg, r1osb, r1win, r2s2, r2w2, wintests, xhard, sea-rs-fix, sea-rs-horizon) were clean and are removed. Kept: ~/projects/sea-rs (owner checkout), gsr-heads/sea-forge (pinned head), fde-sea-rs-integration (fde/integration).
- ~/projects/sea-rs: the graft edits to AGENTS.md and .agents/skills/graft/SKILL.md landed on main in PR #104 (diff against origin/main is empty); the checkout was moved to main at 2f35ad27 without touching the untracked reports tree (main tracks five older baseline report files, which therefore show as modified; their content is the coordinator's live version). Local branch docs/egm-bootstrap-readmes deleted.
- stash@{0} (casework/live-wiring-clean-2026-10-09, only Jolli session noise and a gemini hook deletion: obsolete) and tags backup/od4-2026-11-16/dirty/sea-rs and stash/sea-rs/0 (content verified on main or still on disk) are judged obsolete, but the delete was blocked by the permission classifier in this session. They remain; the coordinator can run `git stash drop` and `git push origin --delete` for those two tags.
- The 12 branch backup tags backup/od4-2026-11-16/{ci/release-provenance, fix/b08..., fix/b12..., fix/go-jose-4.1.6, fix/restore-release-attestation, fix/server-ambiguous-declare-keeps-lease, fix/settlement-attempt-claim-settlement, fix/settlement-attempt-claim-settlement-main, fix/settlement-orphan-guard, security/identity-binding-version, t1/case-task-approval, test/settlement-class-fail-closed} are deleted (all integrated or superseded above), local and on origin. checkpoint/* and fde-anchor/* tags untouched.

### CI optimization (PR #105 9b8aca84)

Measured from job durations (queue time excluded). Baseline is main run 43656019461 (push, warm devbox cache, no cargo cache).

| job | before | after, cold cargo cache (PR #105, run 43939481716) | after, warm (PR #103, first PR run after main saved caches) |
|---|---|---|---|
| verify (macos) slowest leg | 967 s | 765 s (rest) / 529 s (server) | 497 s / 390 s |
| test (ubuntu) | 614 s | 631 s / 545 s | 606 s / 602 s |
| lint (ubuntu) | 217 s | 177 s | 147 s |
| MSRV | 114 s | 107 s | 63 s |
| authority-opa | 74 s | 107 s | 60 s |
| workbench | 71 s | 93 s | 85 s |

Slowest required path: 967 s (macOS) before, 606 s (ubuntu test shard) warm, about 37 percent shorter; macOS itself 49 percent shorter. The ubuntu shards are limited by compile and the 500 s test run, so a third shard is the next lever. Docs-only PRs skip the heavy jobs through the `scope` job (PR #104's gate still ran the full sweep because it changed docs/ and workbench files, which are not treated as docs-only). Caches are saved by main pushes only (about 2.3 GB across nine keys of the 10 GB limit). Not done: browser/apt caching for the live ladder (agent-browser install location unverified); the ladder itself (1727 s, not required) is unchanged. Two flakes seen: devbox-install-action failing to resolve the latest devbox version (rerun), and a macOS leg cancelled after 900 s without a runner (rerun).

## Horizon consolidation (2026-11-16)

Main before: 3e05877d (release 0.1.2, #49). Ruleset on main: only deletion, non-fast-forward, code scanning (CodeQL, high or higher) and code quality; no required status checks, no required reviews, no classic branch protection. All work still went through PRs with CI. The `dependency-review` check fails on every PR because the repository's dependency graph is disabled ("Dependency review is not supported on this repository"); it is not a required check, so merges proceeded on the rest. Needs an owner settings change (below).

### Merged

| PR | what | merge SHA |
|---|---|---|
| #50 | A/B/C authority decision closed as superseded by OD-1 (status and closure section in 2026-11-08-agent-authority-decision.md, DEC-OWNER-2026-11-16-01 in horizon-decisions.yaml); 74 untracked design and mediation reports archived (markdown only) | d41c34a54f62 |
| #38 | dependabot typescript 5.9.3 to 7.0.2 (all checks green but dependency-review) | 6aeb7171cc67 |
| #51 | CI optimization (see below) | 389651757ac6 |
| #52 | zod 3.25.76 to 4.6.7 plus `z.record(z.string(), v)` in 7 files; replaces dependabot #36 | a47ea9557ef4 |
| #53 | react and react-dom 19.3.1 with the type packages; replaces dependabot #34 (react-dom alone fails the sidecar smoke: exact version match required) | 87850d8a8452 |
| #54 | @mcp-b/webmcp-polyfill 6.2.0, `initializeWebMCPPolyfill` renamed `installWebMCP`; replaces dependabot #39 | 0e34a344ce95 |

Dependabot #34, #36, #39 closed as superseded by #53, #52, #54 (their PR heads were not pushed to). Release PR: none open; `release-please--branches--main--components--horizon` is a stale branch from the merged 0.1.2 PR #49 and is kept.

### Branches

Every local tip below equals the head of its merged PR unless stated; "squash" means the PR merged as a squash commit, so `git cherry` shows '+'.

| item | disposition | evidence |
|---|---|---|
| chore/domainforge-0.22.0 (6352ab7b) | integrated | PR #31 11f8ab0d |
| chore/domainforge-0.22.2 (origin), chore/cognate-0.10.0 (origin) | integrated | PR #41 2c14d1a1, #42 b5f699c7; `git cherry` empty |
| chore/drop-unused-identity-tarball (302c09fe) | integrated | PR #23 6c7f0658; ancestor of main |
| chore/gateway-unit-killmode (f57675a1) | integrated | PR #26 c267302c |
| ci/codeql-upload, ci/desktop-install-qualification (origin) | integrated | PR #44 9a4579ff, #43 4242d312 |
| fde/cognate-0.9.1 (7f19cd1d), fde/fix-obligation-repo-init-race (c1495df4) | integrated | PR #30 d9b5869a, #29 d9cb701b |
| fde/integration (efee3918, newest 2026-11-15) | kept | active FDE lane by the keep rule; PR #32 6582c4be merged and the tip is an ancestor of main; worktree removed |
| feat/ci (a674028f), feat/desktop-tauri (2e469299) | integrated | PR #33 f0952e83, #40 5137ab1a; ancestors of main |
| feat/github-connector (5369b498), feat/seed (f7b411c0) | integrated | local only, ancestors of main |
| feat/governed-operate (32a5344b), feat/ignition-onboarding (6f57bfe8), fix/gateway-socket-dir-at-boot (374b58e4), fix/j02r2-rig-operator-ref (2e2c290c), fix/settlement-pending-error-class (3f62b04c) | integrated | PR #1 82877a2c, #10 28ac589e, #16 8d8c4531, #17 66abfd6c, #11 ff27c8fa; tip equals PR head (the OD-4 "unpushed" flag was only the squash merge); OD-4 backup tags redundant |
| fix/followups-19-21-24 (40fc832a) | integrated | PR #25 da683e01 (installer plan table, approval inbox); approval-inbox.tsx was later replaced by src/domain/approval-inbox.ts (#18, #22) |
| fix/macos-app-asset-name (origin) | integrated | PR #48 aff1393c |
| horizon-cockpit-projection-review (local 18f0c40c, origin 03a57da2) | integrated | PR #18 94748dcb; the origin tip's extra commit is PR #20 (installer hardening) squashed into #18's lineage |
| horizon-desk-gaps (e5b98b83, origin), horizon-desk-v2 (73678b22), horizon-desk-v2-merge (b4e18436) | integrated | ancestors of main; PR #28 822e349e for the merge branch |
| settlement-approval-gate (8894d6d6) | integrated | PR #27 4e19be5e; the two spec commits have no file difference to main |
| settlement-external-approval-gate (a2d98ff9; PR #15 closed), t1/reconcile-main (781231e6 plus a merge) | integrated | the rendered-policy rules `settlement-external-approval` and `...-operator` and test/gateway-setup.test.ts:587 are on main (scripts/gateway-setup.ts:440) |
| stage0b-containment-core (23db0683), stage0b-containment-pr1 and stage0b-pr1-broad-snapshot (43e8327e), stage0b-containment-pr1-narrow (7115b202), stage0b-pr1-narrowing-wip (3249d2bd) | rejected | WIP checkpoints, 206 commits behind main, never reviewed. Stage 0B containment landed on main (61e694c, 452f6b3 ports command-route-authority from -pr1-narrow, horizon-desk-gaps ancestor of main). The PR-1 report in them is a record of a narrowed scope that was replaced; terminal confirmation (settle-waiting helper) is superseded by OD-1 |
| dependabot/bun/typescript-7.0.2 | integrated | #38 |
| dependabot/bun/zod-4.6.7, multi-75c0a8599d, mcp-b/webmcp-polyfill-6.2.0 | superseded | #52, #53, #54 |
| dependabot/uv/research/pytest-9.2.0 | integrated | #37 42948016 |
| docs/agent-reports, ci/optimize-horizon, deps/zod-4, deps/react-19-3-1, deps/webmcp-polyfill-6 | integrated | own PRs #50, #51, #52, #53, #54 |
| release-please--branches--main--components--horizon | kept | release automation branch (PR #49 already merged); NOT merged or touched |
| main | kept | |

### OD-4 backup tags (Horizon)

| tag | disposition | evidence |
|---|---|---|
| feat/governed-operate, feat/ignition-onboarding, fix/gateway-socket-dir-at-boot, fix/j02r2-rig-operator-ref, fix/settlement-pending-error-class | obsolete | PR heads equal the tags' commits; merged as above |
| stage0b-containment-core, stage0b-containment-pr1, stage0b-containment-pr1-narrow, stage0b-pr1-broad-snapshot, stage0b-pr1-narrowing-wip | rejected | see the stage0b row; the branches were deleted with them |
| dirty/Horizon (f5c37aef, tree 18b9de20) | integrated / rejected | working tree tree-hash equalled the tag before cleanup. 74 markdown reports committed in #50. Rejected as ephemeral: the estate-reconciliation transcripts, shell scripts, raw logs and tsv (about 4 MB; the scrubbed raw gitleaks output). The original (unscrubbed) snapshot stays only in ~/backups/od4-2026-11-16/Horizon-dirty-Horizon.bundle |
| dirty/Horizon-core (1457e31c, tree bc3229e8) | rejected | the Horizon-core worktree's tree equalled the tag; 8 modified files and 2 new on top of stage0b-containment-core: a terminal-confirmation (settle-waiting) test helper and WIP on the same superseded Stage 0B lane; test/enforcement-probe.test.ts is already on main |

### Worktrees and stashes

- Stashes: none in the repository.
- Removed (clean): Horizon-gaps, Horizon-merge, Horizon-pr1, Horizon-pr1-narrow, horizon-connector, horizon-seed, _wt/df022-horizon, _wt/fde-horizon-cognate-091, _wt/fde-horizon-repo-race, _wt/fde-horizon-integration, _wt/horizon-ci, _wt/horizon-desktop, and my hz-* worktrees. Horizon-core removed after the tree-hash check (dirty, equal to the dirty tag).
- Kept: gsr-heads/horizon (pinned head checkout, detached 4242d31) and ~/.local/state/horizon-approval-test/checkout (detached 3a71fd2, 1 dirty path; a test fixture outside ~/projects).
- Primary checkout ~/projects/Horizon: on main, fast-forwarded, clean (untracked reports removed by explicit path after the tree-hash match above).

### A/B/C authority decision

`.agents/reports/2026-11-08-agent-authority-decision.md` (options A terminal confirmation plus sandbox, B narrower agent door, C leave as is) is closed SUPERSEDED by OD-1 (S1/A3 phone authority): status line changed, original text kept, "Closure (2026-11-16)" section added, and DEC-OWNER-2026-11-16-01 appended to `.agents/reports/horizon-decisions.yaml`. The 2026-11-13 device.pair "Option B" production refusal stays in force until the phone surface is proven. Merged as #50.

### CI optimization (PR #51 plus follow-up in the same PR)

Before (main run 43880164444, bun/uv caches warm, nothing else cached): wall 221 s; just verify 166 s, Chromium install 23 s, typecheck 10 s.
After: `verify (just verify, build:desk, typecheck)` is an aggregate of `static` (verify-static, build:desk, tsc; 74 s) and four time-balanced bun test shards (scripts/ci-test-shard.ts with measured weights; junit timings uploaded; 74 to 91 s per job). Cold (first run on the branch, 3 shards, no caches): 219 s wall, the same as before because of runner queueing behind desktop jobs and uneven shards. Warm (4 shards, caches for the verified binaries and Chromium): 109 s wall, about 2x faster. A composite action shares setup. PR runs already used concurrency with cancel-in-progress; push triggers were already main only; all actions already pinned (actions/cache pinned to 55cc8345 v6.1.0). Docs-only PRs (only .agents/reports md or png) short-circuit inside the jobs; no path filter was added to any workflow, and codeql.yml, security.yml and the release workflows are unchanged. desktop.yml: rust-cache `save-if` main only, plus a push-on-main trigger with the same path filter so main seeds the cache; release-desktop is unchanged. Not done: apt package caching for the Tauri Linux libraries.

## Gauntlet consolidation (2026-11-16)

Main before: fd797dc3 (chore(release): gauntlet 0.1.1, #24). Ruleset on main: deletion, non-fast-forward, code scanning (CodeQL, high or higher), code quality, and required checks `gates (ubuntu)` and `gitleaks` (not strict). Auto-merge is disabled for the repository. A PR that touches no CodeQL-analysed language (shell only) stays BLOCKED until a workflow or source file in it triggers an analysis; #25 carried a one-step ci.yml change for that reason. Primary checkout ~/projects/gauntlet was on release/native-distribution (clean, 4 behind origin, no dirty snapshot tag, no stashes); it is now on main, fast-forwarded and clean.

### Merged

| PR | what | merge SHA |
|---|---|---|
| #9, #14, #17, #19, #11 | dependabot actions: download-artifact 8.0.2, checkout 7.0.1, attest-build-provenance 4.2.2, upload-artifact 7.0.2, setup-just 4.0.0 (all green) | 2768db9, 982c581, ffbebde, 1aa4dbb, 23f954f |
| #10 | dependabot toml 1.2.1 | f3e8ea8 |
| #20, #12 | dependabot react and @types/react, @opentui/react 0.8.4 | d17ba75, ee2b1d2 |
| #15, #16 | dependabot sha2 0.11.0 and @opentui/core 0.8.4, each after `@dependabot rebase` (CI green on current main) | 655f309, b5d06ec |
| #18 | dependabot react-reconciler 0.34.0 (green after #20) | ff9dd68 |
| #26 | @types/bun 1.4.4 plus `signalName()` normalisation of `Subprocess.signalCode` (type widened to `number | Signals | null`, 4 tsc errors); replaces dependabot #13 | 7493d22 |
| #25 | scripts fixes in packaging/qualify-local.sh (see below) plus the helper test wired into the ruler job | 4e35fb0 |
| #27 | CI optimization (see below) | eed6a62 |
| #28 | nextest suite in 4 partitions instead of 2 | 1b50661 |

Dependabot #13 closed as superseded by #26.

### qualify-local fixes (packaging/qualify-local.sh; the brief said scripts/, the file lives under packaging/)

- `sum_of` accepts bare, `*name` and `./name` SHA256SUMS entries (awk exact match instead of the grep pattern).
- Tarball mode redirects HOME and XDG_*; `preserve_gh_auth` now exports `GH_CONFIG_DIR` as the real config dir path first (an explicit GH_CONFIG_DIR wins; nothing is copied, no token is read or printed).
- `packaging/test-qualify-local.sh` (12 checks, pure shell) fails on the old script and passes now; it runs in CI's ruler job.

### Branches, worktrees, stashes

| item | disposition | evidence |
|---|---|---|
| ci/complete-gates (9b1c4c2), ci/release-qualification (a4d54a9), feat/prime-agent-optional (bf98b37), release/native-distribution (5842777), wt/gauntlet-native (b63386b) | integrated | PR #8, #22, #21, #7 merged; each local tip is an ancestor of main (`branch -d` accepted) |
| fix/test-schema-teardown (2710a0e, local only, OD-4 flag UNPUSHED) | integrated | PR #6 squash 1543cbf; blobs of all 4 changed files equal on main, the PR merge commit and the branch tip |
| chore/release-0.1.1, fix/qualify-runs-after-skipped-package, ci/complete-gates, ci/release-qualification, feat/prime-agent-optional, release/native-distribution (origin) | integrated | PR #24, #23, #8, #22, #21, #7 |
| backup/local-main-20261023 (origin, 62dbc52) | integrated | 0 ahead of main, 0 unique patches |
| jollimemory/summaries/v3 (local 5f538f5) | rejected (local copy) | Jolli Memory tool branch, 287 behind; tip identical to origin/backup/jollimemory-summaries-v3-20261023, which is KEPT as the only copy of the 7 tool-generated summary commits |
| backup/jollimemory-summaries-v3-20261023 (origin) | kept | unique tool-generated content; coordinator may delete if Jolli summaries are not wanted |
| fde/integration (origin and local, 846830d, 2026-11-13) | kept | active FDE lane by the keep rule (commits newer than 2026-11-10); PR #7 content is on main; worktree removed |
| dependabot/* #9 #10 #11 #12 #14 #15 #16 #17 #18 #19 #20 | integrated | merged as above (branches auto-deleted by GitHub) |
| dependabot #13 | superseded | #26 7493d22 |
| gsr-heads/gauntlet (0e185aa detached) | kept | pinned head checkout |
| worktrees fde-gauntlet-integration, gauntlet-ci, gauntlet-native, gauntlet-prime, gauntlet-qual | removed | all clean |
| stashes | none | |

### Tags

| tag | disposition | evidence |
|---|---|---|
| backup/od4-2026-11-16/fix/test-schema-teardown (2710a0e) | obsolete | content verified on main (see above) |
| checkpoint/pre-design-20261022, checkpoint/pre-integrate-20261023 (7209153, local and origin) | obsolete | ancestor of main |
| fde-backup/main/20261113, fde-backup/release/native-distribution/20261113, fde-backup/fix/test-schema-teardown/20261113, fde-backup/jollimemory/summaries/v3/20261113 (local only) | obsolete | tips are ancestors of main, or squash-merged (#6), or equal the kept origin jolli backup |
| gauntlet-v0.1.0, gauntlet-v0.1.1 | kept | release tags |

### CI optimization (PR #27, follow-up #28)

Before (main run ee2b1d2, no caches): critical path `rust-check-test` 369 s (just check 38 s, nextest 284 s); boundaries 97 s, rust-lint 59 s. Job critical path about 6.2 min.
After: `rust-check-test` is split into `rust-check` (71 s cold), `rust-test` nextest partitions, `rust-surfaces` (92 s cold); `gates (ubuntu)` is still the single required aggregate. Swatinem/rust-cache v2.9.3 (restore everywhere, save on main only) on lint, check, test, surfaces, boundaries; bubblewrap deb and the sha256-verified pg0 binary cached with actions/cache v6.1.0; `changes` job gives an in-workflow docs-only skip (prose under docs/ and README.md only; docs read by tests, `ruler` and `cargo-deny` always run; `gates` accepts `skipped` only for heavy jobs on a docs-only PR). Push on main only, PR concurrency with cancel-in-progress and SHA pins were already in place.
- Cold (first main run after #27, 2 shards, caches being written): slowest job 260 s (rust-test shard 1), job critical path 4.5 min.
- Warm (rerun of the same commit, 2 shards): 221 s (3.7 min); the tests dominate, so #28 moves to 4 shards.
- 4 shards on a PR run restoring main's caches: slowest shard 184 s, gates at 3.2 min. About 50 percent below the baseline critical path.
- Known flake seen once on the 4-shard PR run and cleared by a rerun of the failed job: `a_fault_with_stderr_held_open_by_a_grandchild_is_bounded` (gauntlet-adapter-tool-program-codemode/tests/process.rs:156, timing-sensitive stderr cut). Not changed here.
- Not done: cargo-deny and ruler need no build cache; the package/qualify workflows were not touched (release pipeline unchanged).

### Release

A patch release (gauntlet 0.1.2) is warranted but not urgent: sha2 0.11 and toml 1.2 reach the CLI binary, the @opentui 0.8.4, react 19.x and @types/bun bumps reach the workbench, and packaging/qualify-local.sh carries the two fixes the Jetson qualification flow needs. No tag or release PR was created (coordinator). Open Gauntlet PRs: none.

### Remaining

- origin/backup/jollimemory-summaries-v3-20261023 and fde/integration kept (see Branches).
- gsr-heads/gauntlet (detached 0e185aa) is pinned and now 8 commits behind main; re-pin from origin/main when the coordinator updates gsr-heads.

## RealityTrace consolidation (2026-11-16)

Main before: 6f84bb5 (release 0.5.0, #19); after: 42f38ca (#20). Primary checkout ~/projects/sxr (origin GodSpeedAI/RealityTrace) was on feat/native-binary-distribution (origin branch gone, 1 dirty file); it is now on main, fast-forwarded and clean. Ruleset on main: deletion, non-fast-forward, code scanning (CodeQL), code quality; no required status checks. Release tags, release and package workflows untouched; CodeQL default setup left on. No open PRs before work. No stashes.

### Branches, worktrees
| item | disposition | evidence |
|---|---|---|
| fix/ledger-read-only-no-create (ae4a7ce), chore/domainforge-0.21.1 (ec1c416), fix/bundled-sqlite (cf6e91d), feat/command-verifier (a68a4b8), feat/verifier-completed-at (3fe3a50) (local only, OD-4 UNPUSHED) | integrated | squash PRs #6 1c9f220, #7 59fea0b, #8 872c2f6, #9 89c1ea4, #10 1234b2b; the last three trees equal the squash trees, later divergence is main evolution |
| feat/native-binary-distribution (c239d5c, 8 unpushed) | integrated | PR #12 4d154b1; every file the branch changed from its base equals the squash, except commands.rs submit_to_daemon, whose cfg(unix)/cfg(not(unix)) form is on main (3361+) |
| chore/domainforge-0.22.0 (local+origin) | integrated | PR #16 521d363; origin branch deleted |
| fde/fix-ingest-json-spelling, fde/fix-release-sha256sums, fde/integration (origin+local) | kept | fde lanes with commits newer than 2026-11-10; content on main (#14 90eff85, #15 62f75aa, #18). The extra "trigger code scanning" commit on the ingest branch is rejected (CI nudge only) |
| release-please--branches--main (origin) | kept | release branch (#19 merged) |
| jollimemory/summaries/v3 (local, 66d9652) | rejected | Jolli Memory tool branch, 2 commits; its OD-4 tag backup/od4-2026-11-16/jollimemory/summaries/v3 is the retained copy |
| opencode.json dirty change in primary | rejected | one-line "$schema" addition; contained in dirty/sxr snapshot tag |
| worktrees df022-sxr, fde-sxr-ingest-json, fde-sxr-integration, fde-sxr-release-sums | removed | only .jolli runtime logs differed from the dirty snapshot tags |
| gsr-heads/realitytrace (521d363) | kept | pinned head, now behind main; re-pin from origin/main |

### Tags
BLOCKED: deleting the local fde-backup/* tags and the remote backup/od4-2026-11-16/* tags was refused by the permission classifier and not retried. Remaining: 8 local fde-backup/*/20261113 tags (all content squash-merged or ancestor, jolli copy kept via OD-4 tag), 12 origin plus local backup/od4-2026-11-16/* tags (branch tips integrated as above, dirty/* snapshots verified superset of removed state). checkpoint/* and release tags (realitytrace-v*, sxr-*-v*) kept.

### CI optimization (#20, 42f38ca; ci.yml and clean-install.yml only)
Rust-cache (restore everywhere, save on main), in-job docs-only skip for docs/ and CHANGELOG.md (README.md excluded because a test reads it; --no-renames), clean-install concurrency, checkout and toolchain pinned by SHA. Before (main 2026-11-15): CI 98 s, clean-install 138 s, .deb 167 s (slowest). Cold after: slowest 150 s (clean-install; .deb 121 s, CI 104 s). Warm rerun: slowest 141 s (clean-install, its cargo install is uncached), .deb 83 s, tarball 42 s, CI 60 s. Gain is mostly in the packaged jobs; wall time is bound by clean-install.

### Release
Not warranted: CI-only change. Pinned RealityTrace CLI surface unchanged, so Cognate-Workbench, Horizon and SEA-Forge consumers are unaffected.

## Cognate consolidation (2026-11-16)

Main before: a08d244 (release-please merge #84); after: e7f1352 (#87). Primary checkout ~/projects/cognate was on security/repo-hygiene-2026-10-31 (upstream gone, clean, tip an ancestor of main); it is now on main, fast-forwarded and clean. Ruleset on main: deletion, non-fast-forward, code scanning, code quality; no required status checks. Public repo; release tags, publish workflows (release.yml) and registry state were not touched. No stashes.

### Merged

| PR | what | merge SHA |
|---|---|---|
| #87 | ci: actions pinned by SHA (verify, package, codeql), bun and Playwright caches in verify, concurrency cancel for package and security | e7f1352 |

### Branches, worktrees

| item | disposition | evidence |
|---|---|---|
| chore/domainforge-0.22.0, fde/domainforge-0.21.2, fde/realitytrace-0.4.0, feat/realitytrace-real-snapshot, security/repo-hygiene-2026-10-31 (local and origin) | integrated | PRs #82, #77, #80, #76, #74 MERGED; local tips equal PR heads or are ancestors of main |
| chore/domainforge-0.22.2 (origin) | integrated | PR #85 MERGED |
| fde/domainforge-pin-exception (2 unique patches) | integrated | PR #78 squash; the lockstep-exception text is present in packages/domainforge/AGENTS.md on main |
| horizon/shared-identity | integrated | 0 ahead of main, 0 unique patches |
| pr-70 (local only, 255 commits, OD-4 flag UNPUSHED) | superseded | release PR #70 merged as 26c5718; branch is the pre-scrub release-please lineage (tree differs from 26c5718 only by 44 files of scrubbed text, 268/268 lines; carries 4 generic-api-key findings already on origin); it adds no file absent from main |
| fde/integration (origin) | kept | active FDE lane; PR #83 content on main; local copy removed |
| release-please--branches--main (PR #86 open), release-please--branches--main--release-notes | kept | release bot branches |
| worktrees df022-cognate, fde-cognate-{df0212,integration,pin-exception,rt040}, cognate-horizon, cognate-identity-wt | removed | clean; the two dirty ones held only untracked node_modules and their trees equalled the OD-4 dirty tags' trees (verified with a temp index) before `git clean -fd` on explicit paths |
| gsr-heads/cognate (detached ca74ab5) | kept | pinned head |

### Tags

| tag | disposition | evidence |
|---|---|---|
| backup/od4-2026-11-16/pr-70, .../dirty/cognate-horizon, .../dirty/cognate-identity-wt (origin and local) | deleted | dispositions above; deletions accepted |
| fde-backup/{feat/realitytrace-real-snapshot,horizon/shared-identity,main,security/repo-hygiene-2026-10-31}/20261113, fde-backup/pr-70/20261113 (local only) | deleted | ancestors of main, or the superseded pr-70 tip |
| checkpoint/pre-design-20261022, checkpoint/pre-readiness-merge-20261022 | kept | not ancestors of main (pre-scrub history); owner decides |
| release tags | untouched | |

### CI optimization (PR #87)

Before: verify (ubuntu), the only long job, 140-156 s with the pinned-CLI cache hit, 297-306 s on a miss (cargo install domainforge-core). package 40-60 s, security 20 s, codeql about 60 s. Already in place: concurrency on verify and codeql, PR plus main-push triggers, pinned-CLI cache, path filters on package.
After: PR run with all caches cold 176 s; first main run 139 s; same-commit rerun 182 s (runner variance; the bun and Playwright caches saved nothing measurable, the apt deps dominate Playwright install). Not done, with reason: no docs-only short-circuit (`just verify` gates read docs/ and markdown ledgers); no sharding (one 54 s gate step plus 19 s Postgres suites; splitting would duplicate about 60 s of setup); release.yml unchanged and not SHA-pinned.

### Release

No release warranted: only workflow changes merged. Open release PR #86 is the coordinator's. No public API changed; no consumers (Cognate-Workbench, Horizon) affected.

### Remaining

- checkpoint/* tags and fde/integration kept.
- gsr-heads/cognate pinned at ca74ab5, behind main.
- release.yml actions are not SHA-pinned.

## DomainForge consolidation (2026-11-16)

Main before: fafb9ab (chore: release 0.22.2, #176). No ruleset or required checks on main (`rules/branches/main` returns `[]`). Primary checkout ~/projects/domainforge was dirty and 6 behind; stashes: none.

### Merged

| PR | what | merge SHA |
|---|---|---|
| #180 | CI optimization (below) plus the 2026-11-02 branch reconciliation report | 4a6b1409 |
| #181 | uuid 1.29.2, serde-saphyr 2.1, thiserror 2.0, colored 3.1, ed25519-dalek 2.2.0, base64 0.23, jsonschema 0.61 (default features off, drops reqwest, rustls and aws-lc from the lockfile; tests moved to `validator_for`/`iter_errors`); MIT-0 and Zlib added to deny.toml, MPL-2.0 and MIT-0 to the dependency-review allowlist; replaces dependabot #155 #156 #157 #161 #170 #177 #178 | 317019a6 |

Local checks for #181: `cargo check --all-targets --features cli`, and the calm, registry, asyncapi, evaluate and application schema tests (50 tests, all pass). CI on the PR: 46 pass, 2 skipping, 0 fail.

### Branches, PRs, worktrees

| item | disposition | evidence |
|---|---|---|
| fix/npm-platform-packages (local+origin, add5558) | integrated | PR #173 ff87978; diff to main only in files later changed by #174/#175/#176 |
| fix/npm-publish-local-path (local+origin, 47bee69) | integrated | PR #175 f08ad7a; `git cherry` shows it applied |
| rel-0.22.1 (local only, c7dc274; OD-4 flag UNPUSHED) | superseded | release 0.22.1 landed as #174 93a5a36 and 0.22.2 as #176 fafb9ab; the golden-digest change is superseded by the 0.22.2 digests; backup tag backup/od4-2026-11-16/rel-0.22.1 deleted |
| detached worktrees 7139ccc (0.22.0 digests) and 8496bf8 lineage | superseded | squashed as ac8c421 (#165) |
| detached worktree 3960326 | integrated | ancestor of main (#171) |
| fde/integration (origin and local, b91b147, 2026-11-14) | kept | active FDE lane by the keep rule; all 6 commits' content is on main via #171 3960326 (tree diff to main only in later release files); delete when the coordinator ends the lane |
| fde/spike-wp10 (origin and local, c1307c8, 2026-11-13) | kept | active FDE lane; holds the only copy of the wp10 tri-state spike (tests/spike_wp10.rs, spike/wp10/model.sea) |
| agent/projection-families (origin, PR #114 draft, 06ad365, 2026-07-08) | superseded | 476 behind main; every family it adds (AI learning, ArchiMate, BAML, BPMN, CMMN, DSPy, Lean, OTel, RDF, ZenML) is on main with CLI, bindings and CI verify jobs; PR #114 closed with a comment; tip stays reachable via refs/pull/114/head |
| release-please--branches--main--components--domainforge (origin, 1f03a29) | kept | release-please branch (stale 0.22.1 attempt, no open PR; release-please recreates it) |
| dependabot #154 (rust-toolchain 1.120.0) | rejected | the action tag is the toolchain version; 1.120.0 does not exist (Lint failed); actions pinned by SHA in #180 |
| dependabot #155 #156 #157 #161 #170 #177 #178 | superseded | the dependabot branches left the workspace Cargo.lock stale (every run failed on --locked); replaced by #181 |
| dependabot #179 (pythonize 0.30) | rejected | needs a newer pyo3 than 0.29; blocked on a pyo3 bump |
| worktrees domainforge-npm, domainforge-rel, fde-df-integration, fde-df-main-3960326, fde-df-release, fde-df-spike | removed | all clean |
| gsr-heads/domainforge (detached ac8c421) | kept | pinned head checkout (now behind main; re-pin from origin/main) |
| primary checkout ~/projects/domainforge | clean main | dirty tree (5 graft tooling files: .claude/helpers/graft-*.cjs, .claude/settings.json, .claude/skills/graft/SKILL.md, plus an AGENTS.md graft block that breaks the 150-line cap) verified equal to tag backup/od4-2026-11-16/dirty/domainforge (tree 5d60b61); the untracked 2026-11-02 reconciliation report landed through #180 |

### Tags

| tag | disposition | evidence |
|---|---|---|
| backup/od4-2026-11-16/rel-0.22.1 | deleted | superseded (above) |
| backup/od4-2026-11-16/dirty/domainforge | deleted (local and origin) | integrated: .claude/skills/graft/SKILL.md refresh via PR #183 (squash 35c4ed13). Rejected: AGENTS.md graft block (AGENTS.md already documents graft and +42 lines would break its 150-line cap), .claude/helpers/graft-*.cjs, .claude/settings.json and the stray report (regenerable graft output). Same PR window: #182 (a303954e) rustix 1.1.6 for RUSTSEC-2026-0486, ignores RUSTSEC-2026-0194/0195 (quick-xml 0.37.5 pinned by oxigraph 0.5.x, latest 0.5.13, no fix reachable); #184 (8c5c95a6) required contexts on docs PRs. |
| fde-backup/main/20261113 | obsolete | ancestor of main |
| v* release tags | untouched | |

### Notes

- cargo-deny advisories already fail on main for roxmltree, quick-xml (RUSTSEC "unbounded namespace-declaration allocation" and "quadratic duplicate attribute check") and xattr (unsound). New advisory-db entries, not caused by this work; the CI Security Audit job passed on the last main run.
- Local cargo needed `MISE_TRUSTED_CONFIG_PATHS=$HOME/.cargo/registry` because email_address ships a mise.toml that the mise rustc shim refuses to read.
- Consumers: no CLI, binding or language-surface change in this consolidation, so Cognate, Cognate-Workbench, RealityTrace, Horizon and SEA-Forge pins are unaffected. jsonschema, thiserror 2, serde-saphyr 2, uuid and base64 are internal; no public type changed.

### CI optimization (PR #180)

No required checks exist on main, so nothing is gated on a path filter. Changes: Swatinem/rust-cache v2.9.3 replaces the registry-only actions/cache steps (restore everywhere, save on main only), all `ci.yml` actions pinned by SHA (rust-toolchain takes `toolchain:` as an input), docs-only PRs skip the heavy jobs through a scope step in `lint`, cancel-in-progress only for PRs. Release, publish and package workflows untouched. Not done: nextest sharding (the Rust tests are not the slow path) and splitting `just prove`.

Slowest path (run wall clock, with GitHub queueing; slowest job in brackets):
- Before (main fafb9ab, no target cache): 31 min total, lint 432 s then Prove 886 s (`just prove`, step 834 s); verify-projection-targets 711 s; verify-rdf 607 s; Test Rust windows 546 s.
- After, cold cache (PR run, no cache to restore): 24.5 min, Prove 926 s, lint 538 s.
- After, cold main run that saved the caches (4a6b140): 23 min, Prove 928 s.
- After, warm (PR run restoring main's caches, f5a1043): 26.5 min total (queueing), lint 454 s, Test Rust windows 440 s, macOS 322 s, verify-rdf 444 s, Prove 1037 s, verify-projection-targets 721 s.
- Result: lint, Windows and macOS test jobs are 15 to 35 percent faster warm. Prove and verify-projection-targets do not benefit (they run many `cargo run --release` projections and external tool installs, not a dominated rebuild); Prove at about 15 to 17 min stays the slowest path. Follow-up: split `just prove` per projection family into parallel jobs, and cache the tla2tools, OPA and openshell-prover downloads.

### Release

A patch release (0.22.3) is optional, not urgent: the merged changes are dev-only (jsonschema in dev-dependencies, tests, CI) plus runtime uuid 1.29.2, serde-saphyr 2.1, thiserror 2, colored 3.1, ed25519-dalek 2.2 and base64 0.23 patch-level and internal bumps. They reach the CLI and the bindings but change no public API or language surface. Release-please will open a PR on its own for the conventional `chore(deps)` commit only if configured to count it; left to the coordinator.

### Remaining

- fde/integration and fde/spike-wp10 kept (lane owners).
- pythonize 0.30 blocked on pyo3; dependabot will reopen it.
- Pre-existing cargo-deny advisories (roxmltree, quick-xml, xattr) need dependency bumps or ignores.
- gsr-heads/domainforge needs a re-pin.

## swe_seed consolidation (2026-11-17)

Main before: 232e02d. Ruleset "Protect Main" (deletion, non-fast-forward, code quality, CodeQL); no required status checks. Primary checkout ~/projects/SWE_SEED was on feat/native-distribution with one tracked change (.jolli/jollimemory/debug.log).

| item | disposition | evidence |
|---|---|---|
| PR #13 feat/native-distribution (draft, f015019d) | integrated | marked ready, CI green (just ci, 5 native build targets, SHA256SUMS), squash f893a473; tree of the branch equals the merge |
| fix/native-candidate-windows-paths-macos-runner (1c71936) | integrated | PR #14 had merged into feat/native-distribution, not main; its content is in #13 (tree diff to f893a473 empty) |
| deploy-prep 04360a1 (EGM READMEs + drift report; #10 was an earlier squash) | superseded | rebased onto main as PR #15, squash 737f0b34; scripts/README.md and crates/swe-seed/src/README.md updated for `init` and native-smoke.sh |
| checkout dirty (.jolli debug.log) | discarded | tree equal to backup/od4-2026-11-16/dirty/SWE_SEED (ef1e208c) |
| gsr-heads/swe-seed (232e02d) | kept | pinned head (behind main; re-pin from origin/main) |
| tags | backup/od4-2026-11-16/dirty/SWE_SEED deleted (local+origin); checkpoint/* and fde-backup/* untouched | |

CI: n/a (just ci about 170 s, under the 3 minute threshold; no required checks; native-candidate about 5 minutes is advisory). Release warranted: yes (swe-seed init, native candidate packaging). Primary checkout: clean main.

## domainforge-vsc-extension consolidation (2026-11-17)

Main before: 91d9287 (v0.1.0). No rulesets on main. Primary checkout was on dev, clean.

| item | disposition | evidence |
|---|---|---|
| dev (64bc8fa, 2 commits c2c3eb8 AST JSON commands and 64bc8fa README/.gitignore; PR #8 had merged an earlier dev) | integrated | PR #11 squash 62fb47b7 |
| dependabot #1, #2, #3 (setup-node 6, wait-on-check 1.4.1, checkout 6) | superseded | PR #12 squash b489770d bumps all three and pins every action in all four workflows by SHA |
| dependabot #10 (dev-dependencies group) | superseded | PR #13 squash 82370eb9: @vscode/test-web 0.0.78, esbuild 0.27.7, typescript-eslint held at ~8.52 (8.73 pulls minimatch 11 and @isaacs/nonrecursive-glob-match, BlueOak-1.0.0, outside the dependency-review license allowlist). @types/vscode left at ^1.107.0 because it must not exceed engines.vscode |
| dependabot #5 (@types/node 25) | rejected | project pins @types/node 22.x |
| branches dev, feat/ast-json-commands, ci/pin-actions, deps/dev-group, dependabot/* | deleted | after merge |

Host note: the `pnpm` mise shim hangs (no pnpm install behind it); used `npx pnpm@9` with --ignore-scripts. CI: n/a (slowest job Test 52 s). Release warranted: yes (AST JSON commands, v0.2.0). Primary checkout: clean main. No OD-4 tags for this repo.

## domainforge-lsp consolidation (2026-11-17)

Main before: 0f3d839 (v0.0.2). No rulesets. Primary checkout was on semantic-adapter, clean. CI on main was broken: all jobs checked out GodSpeedAI/DomainForge ref dev (does not exist; sea-core is now the published domainforge-core =0.19.0), and the tree was not rustfmt clean.

| item | disposition | evidence |
|---|---|---|
| semantic-adapter (7 commits; contains dev's 3: AST JSON request, extra LSP capabilities, .gitignore) | integrated | PR #14 squash 062011b9 |
| CI fix (in #14) | integrated | stale DomainForge checkout removed from ci.yml and release.yml, actions pinned by SHA (release gh-release kept at v1 SHA), rust-cache save-if main, concurrency, lint/test/build in parallel; rustfmt applied; clippy fixes for lib (large_err allowed on two functions returning the core SemanticDiagnostic) |
| dev | integrated | ancestor of semantic-adapter, in #14 |
| feature/fotmat-handler 35e07cc3 (dependabot, security, automerge workflows, ci/release rewrite) | rejected | stale: assumes a DomainForge v0.4.0 sibling checkout; superseded by the CI fix; add a dependabot.yml later if wanted |
| feature/WASM-web-ext-support 5249cc23 (Cargo.lock +rustc-hash) | superseded | PR #9 merged it into dev; rustc-hash is in the current lockfile |

CI after on the PR (cold cache): Lint 42 s, Test 73 s, Build 121 s; before: failing. Not measured warm (cache saves only on main). Remaining: `cargo clippy --all-targets` has 3 test-target lints (not in CI scope). Release warranted: yes (v0.0.3: AST JSON, semantic packs, core 0.19.0). Primary checkout: clean main. No OD-4 tags.

## GodSpeed-Agent consolidation (2026-11-17)

Main before: a41c227 (2026-07-21). PRs #1 #2 #3 had merged into audit-corrections/neatcode-2026-07-29, not main, so main was 34 commits behind the real lane. Private repo, free plan: no rulesets or branch protection available (403).

| item | disposition | evidence |
|---|---|---|
| audit-corrections/neatcode-2026-07-29 (81d6cd1 local, 3a68210 origin) | integrated | PR #4 squash d6b45808; tree identical to fde/integration |
| migration/cep-world-required-evidence (4bfed12; #2 MERGED into the audit branch) | integrated | content in #4; tip kept as local tag fde-backup/migration/cep-world-required-evidence/20261113 |
| fde/integration (e141876, 2026-11-13) | kept | active FDE lane; tree equals main now |
| codex/gsa-v1-settlement-events-lfs (d71f6f80) | superseded | 9389b82 and the LFS commits are cherry-equivalent on fde; the three smoke jsonl files were removed deliberately by c8498fa; docs commit 29a7313 files identical on fde |
| local feature/enterprise-runtime (2240d6d, no remote) | superseded | docs commit 29a7313 identical on fde; local tag fde-backup/feature/enterprise-runtime/20261113 kept |
| worktrees fde-gsa-baseline (a41c227), fde-gsa-loop (4bfed12) | removed | clean |
| checkout dirty (t03 and t11 fixtures regenerated, timestamps and uuids) | discarded | tree equal to backup/od4-2026-11-16/dirty/godspeed_agent (e0590db9). Regenerated output is not committed over evidence |
| dirty tag | deleted (local+origin) | |

CI: BLOCKED. Every Actions job fails in 2 s: "recent account payments have failed or your spending limit needs to be increased" (private repo). Local gate on the same tree (fde worktree): `uv run --frozen --extra dev --extra ml pytest -q`: 1083 passed, 54 skipped in 37 s. gitleaks on the 34 commits reports 22 generic-api-key findings, all idempotency hashes in tests/fixtures/t03,t09,t11 plus one report line; already on origin, none new. Finding: the test suite rewrites tests/fixtures/t03_work_requested.json and t11_canonical_golden.json on every run (follow-up: make the fixtures deterministic). CI optimization: n/a (blocked; suite 37 s). Release warranted: no. Primary checkout: clean main.

## Context-Kernel consolidation (2026-11-17)

Main before: 36bd8c8. No rulesets. Primary checkout dirty: deleted tracked .jolli/jollimemory/telemetry-queue.ndjson.

| item | disposition | evidence |
|---|---|---|
| local backup/harden-pre-sync (62ee3485, unpushed, docs and agent state) | integrated | cherry-pick onto main is empty: its docs changes are already on main; its .agents state files conflicted and are older than main's (rejected) |
| checkout dirty | discarded | tree equal to backup/od4-2026-11-16/dirty/Context_Kernel (bf561980) |
| tags | backup/od4-2026-11-16/backup/harden-pre-sync and dirty/Context_Kernel deleted (local+origin) | v0.1.0 untouched |
| gsr-heads/context-kernel (36bd8c8) | kept | pinned head |

CI optimization PR #13 squash b6387cd5: push trigger main only (was every branch, which also ran CI on my backup-tag pushes), concurrency cancel, rust-cache save-if main, package job no longer waits for the rust job, actions pinned by SHA; package.yml and release.yml unchanged. Before (main push): 243-290 s, PR 325 s; critical path rust 56 s then package/macos-x86_64 180 s. After, PR on cold cache: all jobs green; wall time dominated by macOS runner queue (macos-x86_64 7m36 including queue), rust 3m21 cold. Main push after merge (cold cache save): 262 s, success. Warm run not yet measured; the macOS packaging job (180 s of run time plus runner queue) bounds the gain, so expect about 190-200 s warm. Release warranted: no (CI only). Primary checkout: clean main.

## canonical-evaluation-protocol consolidation (2026-11-17)

Main before: 5111ec3. Primary checkout ~/projects/cep was on profiles/context-bundle with one untracked file.

| item | disposition | evidence |
|---|---|---|
| profiles/context-bundle (d7591c1, #3 MERGED squash 5111ec38) | integrated | tree identical to main; local and remote deleted |
| fde/integration (8b6fc90, 2026-11-13; 2 commits: interaction_event v1 extension schema, CEP-0008 canonical hash rule, 41 files) | kept | active FDE lane, not merged to main; see recommendation |
| untracked .agents/OBSERVED_DEBT.md (CEP-0008 hash debt, 2026-10-05) | rejected | the debt is closed by fbfc557 on fde/integration; tree equal to backup/od4-2026-11-16/dirty/cep (844afa17) before removal (git clean after -n preview) |
| dirty tag | deleted (local+origin) | |
| gsr-heads/cep (5111ec3) | kept | pinned head |

CI: n/a (ci.yml about 20 s). Release warranted: no. Primary checkout: clean main. Recommendation: merge fde/integration to main (and re-pin gsr-heads/cep) once the interaction-event contract is accepted.

## Terminal-UI consolidation (2026-11-17)

Main before: ff1f5eb. No rulesets.

| item | disposition | evidence |
|---|---|---|
| remediation/audit-2026-09-29 (7932154, AGENTS.md restructure) | integrated | PR #1 squash 3c020870, verify green |

CI: n/a (verify about 50 s). Release warranted: no. Primary checkout: clean main. No OD-4 tags.

## NeatCode consolidation (2026-11-17)

Main de9a0b9 (release 1.2.0, #5). Only remote branch release-please--branches--main--components--neatcode (stale 1.3.0 attempt, no open PR): kept (release-please branch). Primary checkout clean, no worktrees, no stashes. CI: n/a (nothing changed). Release warranted: no.

## ideal consolidation (2026-11-17)

Main dcc5ea1 (private repo, only upstream fork history plus one commit). Local unpushed jollimemory/summaries/v3 (7f4a3e5, 2 commits, 3 JSON files: Jolli summary of "intial commit"): rejected, tool-generated and regenerable; local branch and backup/od4-2026-11-16/jollimemory/summaries/v3 (origin) deleted. CI: n/a (about 17 s). Release warranted: no. Primary checkout: clean main.

## ast-grep-mcp-rs consolidation (2026-11-17)

Main 3c2b51a. Dirty README.md (CLI requirement and GUI client PATH notes): tree equal to backup/od4-2026-11-16/dirty/ast-grep-mcp-rs (d1c6c68c); integrated as PR #1 squash 80a2637c (no CI on PRs; only a tag-triggered publish-crate workflow); dirty tag deleted. CI: n/a. Release warranted: no (docs only). Primary checkout: clean main.

## hassos-addon-agent-memory-ledger consolidation (2026-11-17)

Main 003809c. Remote dev (dd4600e, #4 MERGED, 0 unique patches): deleted. CI: n/a (nothing changed). Release warranted: no. Primary checkout clean main.

## hermes-agent-sea-forge and zeroclaw-sea-forge consolidation (2026-11-17)

Diagnosis: `git fetch` fails with "Repository not found"; `gh api repos/GodSpeedAI/<name>` returns 404 while authenticated as SPRIME01 (SSH auth works; other private GodSpeedAI repos resolve). A renamed repo would redirect, an archived one would still resolve, so both repositories were deleted or transferred away. Neither is modifiable.

Local clones hold the only copies of the SEA-Forge work: hermes-agent-sea-forge main b85c3dc5b7 (1 commit over upstream NousResearch/hermes-agent: SEA governance on tool execution; tags backup/opentui-prestrip-20260616-1950 and backup/precopystrip-20260616-2058 kept) and zeroclaw-sea-forge master 5518148e3 (4 commits over zeroclaw-labs/zeroclaw: SEA Ledger NATS telemetry, Copilot instructions, initial SEA-Forge integration; gh-pages 82e581fc is a deploy build). Thin bundles of those commits against upstream, verified: ~/backups/consolidation-2026-11-17/hermes-agent-sea-forge-main.bundle and zeroclaw-sea-forge-master.bundle. Nothing deleted or changed. Recommendation: if the repositories were removed deliberately, drop the clones after the owner confirms; otherwise recreate them from the clones.

# Current status

**Status revision:** 228

**Stage:** Tests: pin fail-closed settlement classes for the Horizon gateway policy; log loaded world refs

**Summary:** With the Horizon gateway policy fixture, a settlement declaration whose class is missing or unknown is denied, local is allowed, and external requires operator approval (Horizon#19 is a non-issue; this pins it). cep_authority now logs each loaded world with its registered ref.

**Verified:**
- sea-forge-authority horizon_gateway_policy tests pass
- sea-forge-server builds with the new world-load log

**Limits:**
- The log is INFO level; it does not list worlds over SFWP

**Next:** Merge after CI; close Horizon#19.

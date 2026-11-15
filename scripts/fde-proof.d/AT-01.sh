#!/usr/bin/env sh
# AT-01 (WP-13): the declared SOP model validates, its world_ref is reproducible, the two
# Settlement concepts stay distinct. Exit 0 is PASS.
# The model check pins the DomainForge 0.22.0 compiler; default to the verified release CLI
# (FDE_DOMAINFORGE_BIN) unless DOMAINFORGE is set.
. "$(dirname "$0")/_lib.sh"
DOMAINFORGE=${DOMAINFORGE:-$DF_RELEASE_BIN}
export DOMAINFORGE
exec "$FDE_ROOT/scripts/check-fde-sop-model.sh"

#!/usr/bin/env sh
# NT-34 (S2, WP-21a): program execution needs a SEA-Forge decision bound to the nine-member execution
# digest. NOT_RUN: step 5 of WP-21a stopped on a design obstacle (see
# .agents/reports/fde-convergence-2026-11-12/evidence/s2/wp-21a.md): the CEP authority request carries
# no execution digest and the decision record no max_runs/runs_used counter.
echo "REASON: program-execution decision not implemented (WP-21a step 5 stopped; design change needed)"
exit 12

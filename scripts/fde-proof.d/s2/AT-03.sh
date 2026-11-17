#!/usr/bin/env sh
# AT-03 (S2, WP-21b): Field Case open, revise, strategy, activities: the document opens all-unknown and is revised with status and source per field; stated and adopted problem are kept apart; strategy revisions chain; the template's activity repeats.
# Exit 0 is PASS.
set -eu
h="$(cd "$(dirname "$0")/.." && pwd)/_cargo-test.sh"
"$h" sea-forge-server conformance_fde_scenarios 1 at_03_open_revise_and_strategy at_03_scenarios field_case_document_recovers_from_its_ledger
"$h" sea-forge-server field_case_template 1 template_instantiates_and_nothing_advances_by_itself document_table_is_the_frozen_schema

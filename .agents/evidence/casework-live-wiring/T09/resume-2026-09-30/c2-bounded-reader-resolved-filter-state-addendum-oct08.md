# Resolved cursor-bound state across pages

Root requires each continuation to sign nullable resolved from/to ordinals in
addition to exact request-filter bindings. Otherwise a later page cannot know
a requested cursor was already found in an earlier validated prefix without
unbounded rescanning. Existing token and payload caps remain unchanged.

Null resolved ordinal with a present requested filter means not yet found; an
absent request means no filter. Actual first ordinal0 is the string "0", not
a sentinel. Discovery is retained only from the acknowledged validated prefix,
never an undelivered lookahead row. Validate any resolved ordinal against that
prefix frontier; origin/null predecessor cannot have resolved bounds. Reuse
authenticated resolved state on resume and resolve outstanding bounds only
from newly read/validated actual rows. Exact filters must still match the signed
request binding; no client-supplied resolved value is trusted independently.

If a verified-empty stream is requested with any non-null cursor bound, absence
is proven and the request fails the existing unknown-cursor input classification
with no successful complete-empty page. No synthetic cursor/head is introduced.
For nonempty history, the prior v2 deferred absence rule remains in force.

This refines the proposed private signed token encoding and v2 state machine;
it adds no index, new budget, persisted state, authority, or legacy change. It
must be included in the corrected reviewed proposal and targeted operator
decision before code. Root also requires preserving existing typed entry_hash
and complete-Value payload_hash; no raw-row entry-hash alternate is authorized.

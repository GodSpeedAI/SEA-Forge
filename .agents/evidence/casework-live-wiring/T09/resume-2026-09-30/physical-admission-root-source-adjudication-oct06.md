# Physical admission — source adjudication and repair assignment

Root read the complete implementation and independent source rejection. Formatting
repair is required. Root also identifies a material compatibility deviation that
the critic described as assignment-required: client.go's new pre-write ctx.Err
check runs for calls with a nil permit, including Ask and mutations. Previously
those followed existing write/deadline/cancellation-callback failure handling.
The added branch can return a different error before attempting a write. The
original assignment explicitly preserves other verbs and existing error behavior.
That requirement takes precedence over the ambiguous scope of its pre-write check.

Clarification: the NEW explicit pre-write context check applies ONLY when a physical
run_get permit is present. Nil-permit callers retain the existing callback/write
behavior. No extra admission or wire validation is authorized for other verbs.
This preserves the existing mutation/Ask retry and recovery policy. The independent
rejection remains immutable; no claim is made that its no-deviation statement proves
this compatibility point. Existing frozen cancellation fixtures must remain exact.

## Fresh builder original bounded repair assignment

Read both original production/wiring assignments, rejection, this adjudication,
complete affected source and nearby fixtures. Edit ONLY
`apps/godspeed-casework-go/internal/adapters/sfwp/client.go` and
`run_get_admission.go`. Client change: scope the new explicit ctx.Err pre-write
check to nonnil permit only, retaining its typed unavailable error for guarded
run_get. Limiter change: ONLY gofmt-equivalent alignment of runGetPermit fields.
No other semantic/format change; main dcc92dd7 and fixtures dae0406d/becd4266 frozen.
Use native patch; read-only gofmt -d allowed, no gofmt -w or shell writes.
No compile/test/scanner/Graft build/Git/status/ledger/evidence edits. Return complete
bounded diff, hashes, deviations and definitive freeze, then STOP writes.

Independent critic must receive this ORIGINAL repair assignment plus all earlier
original assignments and full result. Check both repair requirements and the full
runtime path again. Explain the nil-permit compatibility restoration as a material
deviation from the rejected builder's result. Source approval alone cannot prove
runtime. Only root may grant sequential compiler ownership after source approval.

# Current comparator source rejection

Independent critic `t07_live_confirmation` REJECTS the native-only builder snapshot.
No compiler or test ran for this rejection.

Direct source evidence in `e2e/native-events-live.ts`:

- `objectStanding` includes timestamp by default; `freshCurrentStanding` excludes it.
- The three fresh-current assertions at lines197,298,441 compare default full left JSON
  against timestamp-excluding right JSON. These cannot equal for the required timestamp.
- Explicit expected-current cursor assertions and actual returned timestamp validation
  are present. All full historical comparisons remain timestamp-inclusive.

Fresh builder `t08_type_builder` receives the original comparison requirements and this
defect: correct only the three CURRENT left operands to the same stable comparator used
on the right; preserve every full Store.At comparison and all stable payload fields.
Independent source/type/native/canonical UI verification is still required afterward.

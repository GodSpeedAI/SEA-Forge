# Native diagnosis phase correction

The historical `native-import-diagnosis.md` attempt 6 says the offline K-to-L mutation
exposed the Store panic. The retained runtime stack and current TypeScript sequence show
the failure occurred earlier: phase one dispatched real EXECUTE_ITEM, then its first
`settledPhaseState('phase1')` called the stable-state route and panicked in `Store.At(head)`.
This call precedes the first EventSource subscription and the offline ADD mutation.

Therefore this failed run proves module HTTP200 and the real retained Store lookup defect;
it does not prove initial resync delivery, disconnect, offline ADD, or recovery callbacks.
The immutable `native-retention-panic.log` contains the actual stack and HTTP500 outcome.
All of those native behaviors still require an independent successful rerun after repair.

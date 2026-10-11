# Rejected PhaseB snapshot identity

Root independently compared the preserved six-file production diff against actual
`git diff` before semantic repair. Exact bytes match; SHA256
b0a6b08b05818f736cb3bba790213cd3ea144e689da8c23f2119b66d6efd349d.
This snapshot preserves rejected implementation, not approved execution evidence.

The new rejected scoped test snapshots match original frozen identities:
scoped_live_test.go 361fc0328217bf78d4ac3d7c92dad6f8641e931225e3c78cd58685a7bb4a7800;
scoped_runs_test.go 0013b05f69def466e8434182642b4543e68c5b4ad7fd1a3f3f14608337fcc2b5.

Fresh semantic repair remains tests first. Production changes require independent
regression RED and explicit root phase release. No compiler was used for these checks.

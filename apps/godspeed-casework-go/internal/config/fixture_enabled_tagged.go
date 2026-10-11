//go:build casework_fixture

// The dev/demo build (just casework-go-up, -tags casework_fixture) may select the fixture
// adapters: fixtures stay available to tests and dev, and to nothing else.
package config

// fixtureBuildEnabled marks this binary as allowed to select the fixture adapters.
const fixtureBuildEnabled = true

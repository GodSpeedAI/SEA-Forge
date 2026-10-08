//go:build !casework_fixture

// The production build (no tags) refuses fixture adapter selections: plan guardrail - no
// production code path can load the Northstar fixtures.
package config

// fixtureBuildEnabled marks this binary as a production build: fixtures are not selectable.
const fixtureBuildEnabled = false

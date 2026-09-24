// Fixture loading. The canonical Northstar fixture dataset is copied verbatim into
// fixturedata/northstar.world.json and embedded, so the binary serves exactly the bytes the
// cognitive-ui fixture directory holds. TestEmbeddedFixtureMatchesCanonical (fixture_test.go)
// enforces the copy against the canonical file and skips gracefully when the sibling app is
// absent from the checkout.
package projection

import (
	_ "embed"
	"encoding/base64"
	"encoding/json"
	"fmt"
)

//go:embed fixturedata/northstar.world.json
var fixtureJSON []byte

// Fixture decodes the embedded canonical fixture dataset (FIXTURE-LABELED).
func Fixture() (Dataset, error) {
	var ds Dataset
	if err := json.Unmarshal(fixtureJSON, &ds); err != nil {
		return Dataset{}, fmt.Errorf("projection: embedded fixture dataset is not valid JSON: %w", err)
	}
	return ds, nil
}

// FixtureBytes returns the raw embedded fixture bytes, copied so the caller cannot mutate the
// embedded variable through them.
func FixtureBytes() []byte {
	return append([]byte(nil), fixtureJSON...)
}

func decodeBase64(s string) ([]byte, error) {
	return base64.StdEncoding.DecodeString(s)
}

// CSRF token minting (server side). The synchronizer token lives in the session; this helper
// only mints the pre-login double-submit cookie value. Both are 256 bits of CSPRNG output.
package server

import (
	"crypto/rand"
	"encoding/hex"
)

// newCSRFToken returns a fresh 256-bit hex token, or false when the CSPRNG is broken (callers
// fail closed rather than serving a predictable token).
func newCSRFToken() (string, bool) {
	var buf [32]byte
	if _, err := rand.Read(buf[:]); err != nil {
		return "", false
	}
	return hex.EncodeToString(buf[:]), true
}

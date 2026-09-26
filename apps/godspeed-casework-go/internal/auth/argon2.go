// argon2id password hashing per OWASP Password Storage Cheat Sheet's recommended parameters
// (m=19456 KiB / 19 MiB, t=2, p=1 - the minimum for argon2id), PHC string format for storage.
package auth

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"errors"
	"fmt"
	"regexp"
	"strings"

	"golang.org/x/crypto/argon2"
)

// OWASP-recommended argon2id parameters (the cheat sheet's second configuration: 19 MiB memory,
// 2 iterations, 1 lane). They are constants on purpose: parameter drift between hash and verify
// is a compatibility bug, and the parameters are stored in the PHC string anyway.
const (
	argon2Memory  uint32 = 19 * 1024 // KiB
	argon2Time    uint32 = 2
	argon2Lanes   uint8  = 1
	argon2KeyLen  uint32 = 32
	argon2SaltLen        = 16
)

// phcPattern matches the $argon2id$v=19$m=..,t=..,p=..$<salt>$<hash> storage format this package
// writes and verifies. A hash in any other format is refused, never silently re-hashed.
var phcPattern = regexp.MustCompile(`^\$argon2id\$v=(\d+)\$m=(\d+),t=(\d+),p=(\d+)\$([A-Za-z0-9+/]+={0,2})\$([A-Za-z0-9+/]+={0,2})$`)

// ErrHashFormat marks an unparsable or non-argon2id password hash.
var ErrHashFormat = errors.New("password hash is not an argon2id PHC string")

// HashPassword derives an argon2id hash of password with a fresh random salt and returns it in
// the PHC string format. The output is safe to store in configuration or a file.
func HashPassword(password string) (string, error) {
	salt := make([]byte, argon2SaltLen)
	if _, err := rand.Read(salt); err != nil {
		return "", fmt.Errorf("cannot mint password salt: %w", err)
	}
	digest := argon2.IDKey([]byte(password), salt, argon2Time, argon2Memory, argon2Lanes, argon2KeyLen)
	return encodePHC(salt, digest, argon2Memory, argon2Time, argon2Lanes), nil
}

// VerifyPassword reports whether password verifies against the stored PHC hash. It compares in
// constant time, and an unparsable hash is an error (not a silent mismatch) so a misconfigured
// store cannot lock every user out while looking like wrong passwords.
func VerifyPassword(stored, password string) error {
	params, salt, digest, err := parsePHC(stored)
	if err != nil {
		return err
	}
	computed := argon2.IDKey([]byte(password), salt, params.t, params.m, params.p, uint32(len(digest)))
	if subtle.ConstantTimeCompare(computed, digest) != 1 {
		return ErrInvalidCredentials
	}
	return nil
}

type phcParams struct {
	v uint32
	m uint32
	t uint32
	p uint8
}

func encodePHC(salt, digest []byte, m, t uint32, p uint8) string {
	b64 := base64.RawStdEncoding
	return fmt.Sprintf("$argon2id$v=%d$m=%d,t=%d,p=%d$%s$%s",
		argon2.Version, m, t, p, b64.EncodeToString(salt), b64.EncodeToString(digest))
}

func parsePHC(stored string) (phcParams, []byte, []byte, error) {
	m := phcPattern.FindStringSubmatch(strings.TrimSpace(stored))
	if m == nil {
		return phcParams{}, nil, nil, ErrHashFormat
	}
	var params phcParams
	if _, err := fmt.Sscanf(m[1], "%d", &params.v); err != nil || params.v != argon2.Version {
		return phcParams{}, nil, nil, fmt.Errorf("%w: unsupported argon2 version %q", ErrHashFormat, m[1])
	}
	// The numeric groups are constrained by the regexp's digit classes; conversion failures on
	// them are out-of-range panics in Sscanf's integer verbs only for absurd lengths, which the
	// bounded widths above prevent. Errors still surface as format faults.
	if _, err := fmt.Sscanf(m[2], "%d", &params.m); err != nil {
		return phcParams{}, nil, nil, fmt.Errorf("%w: bad memory parameter", ErrHashFormat)
	}
	if _, err := fmt.Sscanf(m[3], "%d", &params.t); err != nil {
		return phcParams{}, nil, nil, fmt.Errorf("%w: bad time parameter", ErrHashFormat)
	}
	var p8 uint32
	if _, err := fmt.Sscanf(m[4], "%d", &p8); err != nil || p8 == 0 || p8 > 255 {
		return phcParams{}, nil, nil, fmt.Errorf("%w: bad parallelism parameter", ErrHashFormat)
	}
	params.p = uint8(p8)
	b64 := base64.RawStdEncoding
	salt, err := b64.DecodeString(m[5])
	if err != nil || len(salt) < 8 {
		return phcParams{}, nil, nil, fmt.Errorf("%w: salt too short or malformed", ErrHashFormat)
	}
	digest, err := b64.DecodeString(m[6])
	if err != nil || len(digest) < 16 {
		return phcParams{}, nil, nil, fmt.Errorf("%w: digest too short or malformed", ErrHashFormat)
	}
	return params, salt, digest, nil
}

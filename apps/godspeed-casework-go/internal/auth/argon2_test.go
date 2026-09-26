// argon2id hashing tests: round-trip, refusal, tamper and format strictness.
package auth

import (
	"encoding/base64"
	"errors"
	"strings"
	"testing"
)

func TestHashVerifyRoundTrip(t *testing.T) {
	h, err := HashPassword("correct horse battery staple")
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(h, "$argon2id$v=19$m=19456,t=2,p=1$") {
		t.Fatalf("the PHC string must carry the OWASP parameters, got %q", h)
	}
	if err := VerifyPassword(h, "correct horse battery staple"); err != nil {
		t.Fatalf("the correct password must verify: %v", err)
	}
	if err := VerifyPassword(h, "wrong"); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("a wrong password must be ErrInvalidCredentials, got %v", err)
	}
}

func TestHashSaltsAreFresh(t *testing.T) {
	a, _ := HashPassword("same-password")
	b, _ := HashPassword("same-password")
	if a == b {
		t.Fatal("two hashes of the same password must differ (fresh salt)")
	}
}

func TestVerifyRejectsTamperedHash(t *testing.T) {
	h, _ := HashPassword("hunter2")
	// Flip the final digest byte.
	if strings.HasSuffix(h, "A") {
		h = h[:len(h)-1] + "B"
	} else {
		h = h[:len(h)-1] + "A"
	}
	if err := VerifyPassword(h, "hunter2"); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("a tampered digest must not verify, got %v", err)
	}
}

func TestVerifyRefusesForeignFormats(t *testing.T) {
	for _, bad := range []string{
		"",
		"$2b$12$abcdefghijklmnopqrstuvhashhashhashhashhashhashhashhashha",
		"$argon2i$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$m38nzkEDz+xpkLVQWNlbA4wRb3tcwSzLMMWcTv7QN8o",
		"$argon2id$v=16$m=19456,t=2,p=1$YWJjZGVmZ2hpamtsbW5vcA$m38nzkEDz+xpkLVQWNlbA4wRb3tcwSzLMMWcTv7QN8o",
		"plaintext",
	} {
		if err := VerifyPassword(bad, "x"); !errors.Is(err, ErrHashFormat) {
			t.Fatalf("hash %q must be refused as a format fault, got %v", bad, err)
		}
	}
}

func TestPHCRoundTripThroughParser(t *testing.T) {
	// The parser must accept exactly what the encoder writes, including salts/digests that need
	// the +/ alphabet.
	h, err := HashPassword("p\x00ss/w+rd=")
	if err != nil {
		t.Fatal(err)
	}
	if err := VerifyPassword(h, "p\x00ss/w+rd="); err != nil {
		t.Fatalf("round trip failed: %v", err)
	}
	// The salt decodes to 16 bytes.
	m := phcPattern.FindStringSubmatch(h)
	if m == nil {
		t.Fatalf("encoder output does not match the parser's grammar: %q", h)
	}
	salt, err := base64.RawStdEncoding.DecodeString(m[5])
	if err != nil || len(salt) != 16 {
		t.Fatalf("salt must be 16 raw bytes, got %d (%v)", len(salt), err)
	}
}

# T07 Argon2 tamper-test repair

## Finding

`TestVerifyRejectsTamperedHash` changed the last character of the PHC digest text and described that operation as flipping the final digest byte. A 32-byte digest's unpadded Base64 form ends with a character that carries only four digest bits; the other two bits are unused. Changing only those unused bits can leave the decoded digest unchanged, so the old test did not reliably tamper with the value that verification compares. The test also discarded the error from `HashPassword`.

## Bounds

Changed only `apps/godspeed-casework-go/internal/auth/argon2_test.go` among source files. Production password verification, PHC parsing policy, cryptographic parameters, configuration, dependencies, and assertions were not changed or weakened. No valid-hash-alias policy was introduced.

## Implementation

The test now checks hash creation and PHC parsing errors, verifies the original password/hash pair first, flips one bit in a copied decoded digest, and uses the existing encoder with the parsed salt and parameters to produce a canonical PHC string. It parses that string back and explicitly checks that its decoded digest differs before asserting `ErrInvalidCredentials`.

## Material differences

The tampering operation now targets a decoded digest byte rather than an encoded character, and the original hash is verified before mutation. The test also asserts that the re-encoded hash parses to a different digest. No production behavior changed.

## Verification

Not run by this builder: Go tests, `go vet`, compilation, and build. The T07 critic owns the sole compile token and will run focused and global Go race gates against the final source.

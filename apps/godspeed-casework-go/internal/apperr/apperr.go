// Package apperr defines the application's stable, provider-independent error model.
//
// REQ-ARCH-002: provider or transport errors are translated into these kinds at the adapter
// boundary. No provider error type, code, or payload may be referenced from this package: the
// boundary test in internal/boundary fails if one appears.
package apperr

import (
	"errors"
	"fmt"
)

// Kind classifies a failure in application terms. The set is deliberately small and stable: a
// caller switches on Kind, never on a provider code.
type Kind string

const (
	// KindConfig marks configuration that is absent, malformed, or internally inconsistent.
	KindConfig Kind = "config"
	// KindUnavailable marks a capability the application requires but cannot reach.
	KindUnavailable Kind = "unavailable"
	// KindAuthorityDenied marks the governed authority refusing an operation.
	KindAuthorityDenied Kind = "authority_denied"
	// KindInvalid marks a caller-supplied value failing validation.
	KindInvalid Kind = "invalid"
	// KindInternal marks an unclassified internal failure.
	KindInternal Kind = "internal"
)

// Error is the application's error type. Capability names the configured capability the failure
// belongs to (empty when the failure is not capability-scoped), so preflight can attribute blast
// radius without string matching.
type Error struct {
	Kind       Kind
	Capability string
	Op         string
	Message    string
	Err        error
}

func (e *Error) Error() string {
	var b string
	switch {
	case e.Capability != "" && e.Op != "":
		b = fmt.Sprintf("%s: %s (%s/%s)", e.Kind, e.Message, e.Capability, e.Op)
	case e.Capability != "":
		b = fmt.Sprintf("%s: %s (%s)", e.Kind, e.Message, e.Capability)
	case e.Op != "":
		b = fmt.Sprintf("%s: %s (%s)", e.Kind, e.Message, e.Op)
	default:
		b = fmt.Sprintf("%s: %s", e.Kind, e.Message)
	}
	if e.Err != nil {
		return b + ": " + e.Err.Error()
	}
	return b
}

// Unwrap supports errors.Is/As against the wrapped cause.
func (e *Error) Unwrap() error { return e.Err }

// New builds a typed error with no wrapped cause.
func New(kind Kind, capability, op, message string) *Error {
	return &Error{Kind: kind, Capability: capability, Op: op, Message: message}
}

// Wrap builds a typed error around a cause.
func Wrap(kind Kind, capability, op, message string, err error) *Error {
	return &Error{Kind: kind, Capability: capability, Op: op, Message: message, Err: err}
}

// KindOf reports the Kind of err, or KindInternal when err is nil or untyped. It exists so callers
// never have to know whether a failure originated in an adapter or in the core.
func KindOf(err error) Kind {
	if err == nil {
		return ""
	}
	var e *Error
	if errors.As(err, &e) {
		return e.Kind
	}
	return KindInternal
}

// CapabilityOf reports the capability a typed error is attributed to, if any.
func CapabilityOf(err error) string {
	var e *Error
	if errors.As(err, &e) {
		return e.Capability
	}
	return ""
}

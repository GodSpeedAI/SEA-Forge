// Package boundary holds the mechanical proof of REQ-ARCH-002: provider and transport vocabulary
// stops at the adapter boundary and never reaches the application core.
//
// The test is deliberately AST-based rather than type-based. Type aliases, `any` fields, or a
// provider type smuggled behind a name that merely *looks* generic would all defeat a test that only
// inspected declared types - which is the confound the T01 preregistration records. So the scanner
// looks for (a) imports that reach an adapter or a provider, (b) provider/vendor vocabulary in
// declared identifiers, and (c) untyped `any`/`interface{}` holes in the ports package, where a
// provider payload could hide without ever naming itself.
package boundary

import (
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// corePackages are the stable application core: ports, configuration authority, preflight, and the
// error model. Adapters are NOT core.
var corePackages = []string{
	"internal/ports",
	"internal/config",
	"internal/preflight",
	"internal/apperr",
}

// forbiddenImport contains a fragment that must not appear in a core import path.
var forbiddenImport = []string{
	"/adapters/",
	"sea-forge",
	"gauntlet",
	"copilotkit",
	"react-three",
}

// forbiddenIdentifiers are provider, transport or vendor vocabulary. A core package that names one of
// these has let provider language into the application model, whatever its types look like.
var forbiddenIdentifiers = []string{
	"sfwp",
	"gauntlet",
	"openmct",
	"openmontage",
	"copilotkit",
	"ndjson",
}

// Violation is one scanner finding.
type Violation struct {
	File   string
	Line   int
	Reason string
}

// Scan reports boundary violations in the given Go source files. It is exported so a test can feed it
// a synthetic violation and prove it actually detects one.
func Scan(files map[string]string) []Violation {
	var out []Violation
	for name, src := range files {
		fset := token.NewFileSet()
		f, err := parser.ParseFile(fset, name, src, parser.AllErrors)
		if err != nil {
			out = append(out, Violation{File: name, Reason: "unparsable: " + err.Error()})
			continue
		}
		// The boundary is about the application's compiled dependency graph, so an import in a
		// _test.go file is exempt: a test may drive a fake adapter to exercise a contract, and that
		// edge does not ship. Identifier checks still apply to test files.
		isTest := strings.HasSuffix(name, "_test.go")
		for _, imp := range f.Imports {
			path := strings.Trim(imp.Path.Value, `"`)
			if isTest {
				continue
			}
			for _, frag := range forbiddenImport {
				if strings.Contains(path, frag) {
					out = append(out, Violation{File: name, Line: fset.Position(imp.Pos()).Line,
						Reason: "core package imports " + path + " (matches " + frag + ")"})
				}
			}
		}
		ast.Inspect(f, func(n ast.Node) bool {
			switch d := n.(type) {
			case *ast.TypeSpec:
				checkIdent(&out, fset, name, d.Name)
			case *ast.FuncDecl:
				checkIdent(&out, fset, name, d.Name)
			case *ast.Field:
				if isUntyped(d.Type) {
					out = append(out, Violation{File: name, Line: fset.Position(d.Pos()).Line,
						Reason: "untyped any/interface{} field in the core - a provider payload could hide here"})
				}
			}
			return true
		})
	}
	return out
}

func checkIdent(out *[]Violation, fset *token.FileSet, file string, id *ast.Ident) {
	if id == nil {
		return
	}
	lower := strings.ToLower(id.Name)
	for _, bad := range forbiddenIdentifiers {
		if strings.Contains(lower, bad) {
			*out = append(*out, Violation{File: file, Line: fset.Position(id.Pos()).Line,
				Reason: "core identifier " + id.Name + " names provider vocabulary (" + bad + ")"})
		}
	}
}

func isUntyped(e ast.Expr) bool {
	switch t := e.(type) {
	case *ast.Ident:
		return t.Name == "any"
	case *ast.InterfaceType:
		return len(t.Methods.List) == 0
	}
	return false
}

// coreSources reads the real core packages from disk.
func coreSources(t *testing.T, root string) map[string]string {
	t.Helper()
	files := map[string]string{}
	for _, pkg := range corePackages {
		dir := filepath.Join(root, filepath.FromSlash(pkg))
		entries, err := os.ReadDir(dir)
		if err != nil {
			t.Fatalf("cannot read core package %s: %v", pkg, err)
		}
		for _, e := range entries {
			if e.IsDir() || !strings.HasSuffix(e.Name(), ".go") {
				continue
			}
			raw, err := os.ReadFile(filepath.Join(dir, e.Name()))
			if err != nil {
				t.Fatalf("cannot read %s: %v", e.Name(), err)
			}
			files[pkg+"/"+e.Name()] = string(raw)
		}
	}
	if len(files) == 0 {
		t.Fatal("no core sources were scanned - the boundary check would be vacuous")
	}
	return files
}

// TestCorePackagesCarryNoProviderVocabulary is the REQ-ARCH-002 gate.
func TestCorePackagesCarryNoProviderVocabulary(t *testing.T) {
	violations := Scan(coreSources(t, "../.."))
	for _, v := range violations {
		t.Errorf("boundary violation: %s:%d %s", v.File, v.Line, v.Reason)
	}
}

// TestScanDetectsInjectedViolations proves the scanner is not vacuous: each forbidden shape must be
// caught in a synthetic source. Without this, the gate above could pass because the scanner looks at
// nothing in particular.
func TestScanDetectsInjectedViolations(t *testing.T) {
	cases := []struct {
		name string
		src  string
	}{
		{
			name: "provider import",
			src:  "package core\n\nimport _ \"example.com/sea-forge/client\"\n\ntype T struct{}\n",
		},
		{
			name: "adapter import",
			src:  "package core\n\nimport _ \"example.com/app/internal/adapters/fakeauthority\"\n\ntype T struct{}\n",
		},
		{
			name: "vendor identifier",
			src:  "package core\n\ntype sfwpEnvelope struct{}\n",
		},
		{
			name: "untyped any field",
			src:  "package core\n\ntype T struct {\n\tPayload any\n}\n",
		},
		{
			name: "empty interface field",
			src:  "package core\n\ntype T struct {\n\tPayload interface{}\n}\n",
		},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if got := Scan(map[string]string{"synthetic.go": tc.src}); len(got) == 0 {
				t.Fatalf("scanner missed an injected %s", tc.name)
			}
		})
	}
}

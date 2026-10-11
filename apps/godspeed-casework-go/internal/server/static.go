// Static UI serving (plan T07): the gateway serves the built Cognitive UI (config
// serve.static_root, e.g. apps/godspeed-cognitive-ui/dist) with the cache/CSP posture the UI's
// hashed build layout wants:
//
//   - /assets/* (content-hashed filenames): Cache-Control "public, max-age=31536000, immutable".
//   - index.html (and the SPA fallback for unknown non-API GET paths): "no-cache" so a redeploy
//     is picked up immediately; the hashed assets it references are immutable.
//   - CSP on every served document (see cspHeader below and its honesty note about style-src).
//
// The handler never touches /api/*: the mux routes those first, and an unknown /api path hits
// the /api/ catch-all's typed 404 rather than the SPA fallback.
package server

import (
	"mime"
	"net/http"
	"os"
	"path"
	"strings"
)

// Ensure the build's asset extensions resolve to their wire types even on systems whose mime
// database lacks them.
func init() {
	if mime.TypeByExtension(".woff2") == "" {
		mime.AddExtensionType(".woff2", "font/woff2")
	}
	if mime.TypeByExtension(".woff") == "" {
		mime.AddExtensionType(".woff", "font/woff")
	}
}

// cspHeader is the Content-Security-Policy for served documents.
//
// Honesty note (inspected against the current apps/godspeed-cognitive-ui/dist build):
//   - script-src 'self' with NO unsafe-inline/unsafe-eval: index.html carries no inline script,
//     all scripts are external hashed bundles, and the bundles contain no eval/new Function.
//   - style-src keeps 'unsafe-inline': the build ships all CSS as external hashed stylesheets and
//     index.html has zero style attributes, but React 19's runtime contains a <style> resource
//     injection path (createElement("style")) that fires if the app ever renders a React style
//     resource. Until T08/T09 verify at runtime that path never executes, styles keep the weaker
//     directive (a stylesheet injection is not script execution). Tightening is a documented
//     follow-up, not a proven impossibility.
//   - img-src 'self' data: covers the UI's inline data URIs; font-src 'self' covers the bundled
//     IBM Plex woff/woff2 assets.
const cspHeader = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; " +
	"img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; " +
	"base-uri 'self'; frame-ancestors 'none'"

// assetsPrefix is the build layout's hashed-asset directory (vite's default).
const assetsPrefix = "assets/"

func (s *Server) staticHandler() http.Handler {
	root := s.opts.StaticRoot
	fsys := http.Dir(root)
	index := "index.html"
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		clean := path.Clean("/" + r.URL.Path)
		rel := strings.TrimPrefix(clean, "/")
		h := w.Header()
		h.Set("Content-Security-Policy", cspHeader)
		h.Set("X-Content-Type-Options", "nosniff")

		// A real file (other than the SPA root) is served directly; anything else falls back to
		// the single-page app's index so client-side routing works on deep links.
		name := rel
		isIndex := name == "" || name == "."
		if !isIndex {
			if f, err := fsys.Open(name); err == nil {
				f.Close()
			} else {
				isIndex = true
			}
		}

		if isIndex {
			h.Set("Cache-Control", "no-cache")
			http.ServeFile(w, r, root+"/"+index)
			return
		}
		if strings.HasPrefix(rel, assetsPrefix) {
			// Content-hashed filenames: cache forever, revalidate never.
			h.Set("Cache-Control", "public, max-age=31536000, immutable")
		} else {
			// Non-hashed top-level files (favicon etc.): revalidate each time.
			h.Set("Cache-Control", "no-cache")
		}
		// http.ServeFile refuses directory traversals and sets Content-Type from the extension.
		http.ServeFile(w, r, root+"/"+rel)
	})
}

// staticRootReadable reports whether the configured root is a readable directory (main.go uses
// it to fail fast on a bad static_root rather than serving 404s).
func staticRootReadable(root string) bool {
	if root == "" {
		return false
	}
	info, err := os.Stat(root)
	return err == nil && info.IsDir()
}

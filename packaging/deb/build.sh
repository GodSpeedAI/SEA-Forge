#!/bin/sh
# Build sea-forge_<version>_amd64.deb from release binaries.
#
# Usage: packaging/deb/build.sh <version> <bindir> <outdir>
#
# Depends is computed from the binaries with dpkg-shlibdeps (the NEEDED libraries
# resolved against the installed package database), not hand-written.
set -eu

if [ "$#" -ne 3 ]; then
    echo "usage: $0 <version> <bindir> <outdir>" >&2
    exit 2
fi
version=$1
bindir=$(cd "$2" && pwd)
outdir=$3

case "$version" in
    '' | *[!0-9A-Za-z.+~-]*)
        echo "error: invalid version '$version'" >&2
        exit 2
        ;;
esac

repo_root=$(cd "$(dirname "$0")/../.." && pwd)
# The isolated backend runs sea-forge-workload-exec and the MCP bridge runs
# sea-forge-mcp-connect; both are found beside the running executable.
bins="sea-forge sea-forge-server sea-forge-workload-exec sea-forge-mcp-connect"
for bin in $bins; do
    if [ ! -x "$bindir/$bin" ]; then
        echo "error: missing executable $bindir/$bin" >&2
        exit 1
    fi
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Depends from the binaries themselves.
mkdir -p "$work/shlibs/debian"
printf 'Source: sea-forge\n\nPackage: sea-forge\nArchitecture: amd64\nDescription: shlibs probe\n probe\n' \
    > "$work/shlibs/debian/control"
(
    cd "$work/shlibs"
    dpkg-shlibdeps -O $(for bin in $bins; do printf -- '-e%s ' "$bindir/$bin"; done) \
        > "$work/shlibs.out" 2> "$work/shlibs.err" || {
        cat "$work/shlibs.err" >&2
        exit 1
    }
)
deps=$(sed -n 's/^shlibs:Depends=//p' "$work/shlibs.out")
case "$deps" in
    *libc6*) ;;
    *)
        echo "error: dpkg-shlibdeps did not report libc6 (got '$deps')" >&2
        exit 1
        ;;
esac

root="$work/sea-forge_${version}_amd64"
mkdir -p "$root/DEBIAN" "$root/usr/bin" "$root/usr/share/doc/sea-forge"
for bin in $bins; do
    install -m 0755 "$bindir/$bin" "$root/usr/bin/$bin"
done
install -m 0644 "$repo_root/LICENSE" "$root/usr/share/doc/sea-forge/copyright"

installed_kib=$(du -sk "$root/usr" | cut -f1)
cat > "$root/DEBIAN/control" <<EOF
Package: sea-forge
Version: $version
Architecture: amd64
Maintainer: GodSpeedAI <https://github.com/GodSpeedAI/SEA-Forge>
Installed-Size: $installed_kib
Depends: $deps
Section: utils
Priority: optional
Homepage: https://github.com/GodSpeedAI/SEA-Forge
Description: governed capability-execution kernel (CLI and daemon)
 SEA-Forge evaluates proposed actions against declared policy before execution,
 then records the evidence and settlement of what actually ran. Ships the sea-forge
 command-line tool and the sea-forge-server daemon.
EOF
chmod 0644 "$root/DEBIAN/control"

mkdir -p "$outdir"
dpkg-deb --build --root-owner-group "$root" "$outdir/sea-forge_${version}_amd64.deb"
dpkg-deb --info "$outdir/sea-forge_${version}_amd64.deb" | sed -n '/Package:/,/Description:/p'

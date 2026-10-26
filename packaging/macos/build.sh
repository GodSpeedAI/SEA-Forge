#!/bin/sh
# Build the macOS install package and disk image.
#
# Usage: packaging/macos/build.sh <version> <arm64dir> <x86dir> <outdir>
#
# <x86dir> may be "" to build an arm64-only package (named -macos-arm64).
# With both directories the binaries are merged with lipo into a universal
# package (named -macos-universal). The package is UNSIGNED: no Developer ID
# certificate is used, and none is claimed anywhere.
set -eu

if [ "$#" -ne 4 ]; then
    echo "usage: $0 <version> <arm64dir> <x86dir|''> <outdir>" >&2
    exit 2
fi
version=$1
arm_dir=$2
x86_dir=$3
outdir=$4

case "$version" in
    '' | *[!0-9A-Za-z.+~-]*)
        echo "error: invalid version '$version'" >&2
        exit 2
        ;;
esac

repo_root=$(cd "$(dirname "$0")/../.." && pwd)
for bin in sea-forge sea-forge-server; do
    if [ ! -x "$arm_dir/$bin" ]; then
        echo "error: missing executable $arm_dir/$bin" >&2
        exit 1
    fi
    if [ -n "$x86_dir" ] && [ ! -x "$x86_dir/$bin" ]; then
        echo "error: missing executable $x86_dir/$bin" >&2
        exit 1
    fi
done

if [ -n "$x86_dir" ]; then
    label=universal
else
    label=arm64
fi
pkg_name="sea-forge-${version}-macos-${label}.pkg"
dmg_name="sea-forge-${version}-macos-${label}.dmg"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Payload: bin/ installs to /usr/local/bin.
stage="$work/stage"
mkdir -p "$stage/bin"
for bin in sea-forge sea-forge-server; do
    if [ -n "$x86_dir" ]; then
        lipo -create "$arm_dir/$bin" "$x86_dir/$bin" -output "$stage/bin/$bin"
    else
        cp "$arm_dir/$bin" "$stage/bin/$bin"
    fi
    chmod 0755 "$stage/bin/$bin"
done

mkdir -p "$outdir"
pkgbuild --root "$stage" \
    --identifier ai.godspeed.sea-forge \
    --version "$version" \
    --install-location /usr/local \
    "$outdir/$pkg_name"

# Disk image: the package plus licence and install notes.
dmg_src="$work/dmg"
mkdir -p "$dmg_src"
cp "$outdir/$pkg_name" "$dmg_src/"
cp "$repo_root/LICENSE" "$dmg_src/LICENSE"
cat > "$dmg_src/README-install.txt" <<EOF
SEA-Forge ${version} for macOS (${label})

Install
  Double-click ${pkg_name}, or from Terminal:
    sudo installer -pkg ${pkg_name} -target /
  The commands sea-forge and sea-forge-server are placed in /usr/local/bin.

Unsigned package
  This package is NOT signed or notarized (no Apple Developer ID is used).
  Gatekeeper may refuse to open it. Either:
    - Control-click the package, choose Open, and confirm; or
    - remove the quarantine flag after download:
        xattr -d com.apple.quarantine ${pkg_name}
  Verify the download before installing:
    shasum -a 256 -c SHA256SUMS
    gh attestation verify ${pkg_name} --repo GodSpeedAI/SEA-Forge

Uninstall
  sudo rm /usr/local/bin/sea-forge /usr/local/bin/sea-forge-server
  sudo pkgutil --forget ai.godspeed.sea-forge

Licence: see LICENSE in this image.
EOF
rm -f "$outdir/$dmg_name"
hdiutil create -volname "SEA-Forge ${version}" -srcfolder "$dmg_src" -format UDZO "$outdir/$dmg_name"

echo "built $outdir/$pkg_name and $outdir/$dmg_name"

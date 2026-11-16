#!/bin/sh
# Build sea-forge_<version>_<arch>.deb from release binaries.
#
# Usage: packaging/deb/build.sh <version> <bindir> <outdir> <deb-arch>
#
# The architecture argument is validated against the payload, not trusted:
# after staging, every staged executable must report the ELF machine the
# Debian architecture names (amd64 -> x86-64, arm64 -> AArch64), so a package
# whose metadata says arm64 while its payload is x86_64 cannot be built.
# Depends is computed from the binaries with dpkg-shlibdeps (the NEEDED
# libraries resolved against the installed package database of the BUILD
# architecture — run it on a host matching the payload, e.g. the arm64
# hosted runner), not hand-written.
set -eu

if [ "$#" -ne 4 ]; then
    echo "usage: $0 <version> <bindir> <outdir> <deb-arch>" >&2
    exit 2
fi
version=$1
bindir=$(cd "$2" && pwd)
outdir=$3
deb_arch=$4

case "$deb_arch" in
    amd64) elf_machine="Advanced Micro Devices X86-64" ;;
    arm64) elf_machine="AArch64" ;;
    *)
        echo "error: unsupported deb architecture '$deb_arch' (amd64|arm64)" >&2
        exit 2
        ;;
esac

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
    # Architecture/anti-aliasing gate: the payload must BE the architecture
    # the package metadata will claim. Read the machine from readelf's
    # header (stable, version-independent output) rather than from `file`'s
    # human description (whose machine spelling varies across file
    # versions): "Machine:" is "Advanced Micro Devices X86-64" on amd64 and
    # "AArch64" on arm64.
    machine=$(readelf -h "$bindir/$bin" | awk '/Machine:/{v=substr($0, index($0, "Machine:") + 9); gsub(/^ +| +$/, "", v); print v}')
    case "$machine" in
        "$elf_machine") ;;
        *)
            echo "error: $bindir/$bin ELF machine is '$machine', expected '$elf_machine' (deb arch $deb_arch)" >&2
            exit 2
            ;;
    esac
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Depends from the binaries themselves.
mkdir -p "$work/shlibs/debian"
printf 'Source: sea-forge\n\nPackage: sea-forge\nArchitecture: %s\nDescription: shlibs probe\n probe\n' "$deb_arch" \
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

root="$work/sea-forge_${version}_${deb_arch}"
mkdir -p "$root/DEBIAN" "$root/usr/bin" "$root/usr/share/doc/sea-forge"
for bin in $bins; do
    install -m 0755 "$bindir/$bin" "$root/usr/bin/$bin"
done
install -m 0644 "$repo_root/LICENSE" "$root/usr/share/doc/sea-forge/copyright"

installed_kib=$(du -sk "$root/usr" | cut -f1)
cat > "$root/DEBIAN/control" <<EOF
Package: sea-forge
Version: $version
Architecture: ${deb_arch}
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
dpkg-deb --build --root-owner-group "$root" "$outdir/sea-forge_${version}_${deb_arch}.deb"
dpkg-deb --info "$outdir/sea-forge_${version}_${deb_arch}.deb" | sed -n '/Package:/,/Description:/p'

#!/usr/bin/env bash
# yoshi installer.
#   curl -fsSL https://raw.githubusercontent.com/oxmonty/yoshi/main/install.sh | sh
#
# Configuration via environment:
#   YOSHI_CHANNEL      next (default; newest release incl. prereleases) | stable
#   YOSHI_VERSION      exact version to install, e.g. 0.1.0-alpha.1 (overrides channel)
#   YOSHI_INSTALL_DIR  where to install (default: /Applications or ~/Applications
#                      on macOS; ~/.local/bin on Linux)
set -euo pipefail

repo="oxmonty/yoshi"
api="https://api.github.com/repos/$repo/releases"

fail() { echo "error: $*" >&2; exit 1; }

resolve_tag() {
    if [ -n "${YOSHI_VERSION:-}" ]; then
        echo "v${YOSHI_VERSION#v}"
        return
    fi
    case "${YOSHI_CHANNEL:-next}" in
        stable)
            # /releases/latest excludes prereleases; 404s until the first stable release
            curl -fsSL "$api/latest" 2>/dev/null | grep -m1 '"tag_name"' | cut -d'"' -f4 ||
                fail "no stable release yet — try YOSHI_CHANNEL=next"
            ;;
        next)
            curl -fsSL "$api?per_page=1" | grep -m1 '"tag_name"' | cut -d'"' -f4
            ;;
        *) fail "unknown YOSHI_CHANNEL '${YOSHI_CHANNEL}' (stable|next)" ;;
    esac
}

tag=$(resolve_tag)
[ -n "$tag" ] || fail "could not resolve a release tag"
version="${tag#v}"
base="https://github.com/$repo/releases/download/$tag"
os="$(uname -s)" arch="$(uname -m)"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

case "$os" in
    Darwin)
        asset="yoshi-$version-macos-$arch.zip"
        sums="SHA256SUMS-macos.txt"
        ;;
    Linux)
        [ "$arch" = "x86_64" ] || fail "no Linux artifact for $arch yet (x86_64 only)"
        asset="yoshi-$version-linux-$arch.AppImage"
        sums="SHA256SUMS-linux.txt"
        ;;
    *) fail "unsupported OS: $os" ;;
esac

echo "==> downloading $asset ($tag)"
curl -fsSL -o "$tmp/$asset" "$base/$asset"
curl -fsSL -o "$tmp/$sums" "$base/$sums"
(cd "$tmp" && grep "$asset" "$sums" | { shasum -a 256 -c - 2>/dev/null || sha256sum -c -; }) ||
    fail "checksum verification failed"

case "$os" in
    Darwin)
        dir="${YOSHI_INSTALL_DIR:-/Applications}"
        [ -w "$dir" ] || dir="$HOME/Applications"
        mkdir -p "$dir"
        unzip -q "$tmp/$asset" -d "$tmp/unpacked"
        rm -rf "$dir/yoshi.app"
        mv "$tmp/unpacked/yoshi.app" "$dir/yoshi.app"
        echo "installed: $dir/yoshi.app (yoshi $version)"
        echo "curl downloads carry no quarantine flag — it opens without the right-click dance"
        ;;
    Linux)
        dir="${YOSHI_INSTALL_DIR:-$HOME/.local/bin}"
        mkdir -p "$dir"
        install -m 755 "$tmp/$asset" "$dir/yoshi"
        echo "installed: $dir/yoshi (yoshi $version)"
        case ":$PATH:" in
            *":$dir:"*) ;;
            *) echo "note: $dir is not on your PATH" ;;
        esac
        ;;
esac

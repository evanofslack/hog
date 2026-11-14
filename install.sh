#!/bin/sh
# Install the latest hog release on Linux:
#   curl -fsSL https://raw.githubusercontent.com/evanofslack/hog/main/install.sh | sh
# Installs to /usr/local/bin if writable, else ~/.local/bin. Override with HOG_INSTALL_DIR.
set -eu

repo=evanofslack/hog

err() { echo "hog: $*" >&2; exit 1; }

[ "$(uname -s)" = Linux ] || err "install.sh supports Linux only; on other systems use: cargo install --git https://github.com/$repo"

case "$(uname -m)" in
  x86_64 | amd64) arch=x86_64 ;;
  aarch64 | arm64) arch=aarch64 ;;
  *) err "unsupported architecture: $(uname -m)" ;;
esac

if command -v curl >/dev/null 2>&1; then
  fetch() { curl -fsSL -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
  fetch() { wget -qO "$2" "$1"; }
else
  err "need curl or wget"
fi

file="hog-$arch-linux.tar.gz"
url="https://github.com/$repo/releases/latest/download/$file"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

fetch "$url" "$tmp/$file" || err "download failed: $url"
fetch "$url.sha256" "$tmp/$file.sha256" || err "download failed: $url.sha256"
(cd "$tmp" && sha256sum -c --quiet "$file.sha256") || err "checksum mismatch"
tar -xzf "$tmp/$file" -C "$tmp"

dir="${HOG_INSTALL_DIR:-}"
if [ -z "$dir" ]; then
  if [ -w /usr/local/bin ]; then dir=/usr/local/bin; else dir="$HOME/.local/bin"; fi
fi
mkdir -p "$dir"
install -m 755 "$tmp/hog" "$dir/hog"

echo "installed $("$dir/hog" --version) to $dir/hog"
case ":$PATH:" in
  *":$dir:"*) ;;
  *) echo "note: $dir is not on your PATH" >&2 ;;
esac

#!/bin/sh
# Install temper and the bundle where init finds it (protocol/setup.md, 8.2).
set -eu
if [ "$#" -ne 1 ]; then
    echo "usage: scripts/install.sh PREFIX" >&2
    exit 2
fi
prefix=$1
cd "$(dirname "$0")/.."
mkdir -p "$prefix/bin"
prefix=$(cd "$prefix" && pwd)
cargo build --release --locked -p temper
install -m 0755 target/release/temper "$prefix/bin/temper"
bundle=$(scripts/bundle.sh "$prefix/share/temper/bundle")
printf '%s\n' "$prefix/bin/temper" "$bundle"

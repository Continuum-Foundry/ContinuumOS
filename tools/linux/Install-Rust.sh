#!/usr/bin/env bash
# Explicit workspace-only installation. TLS and installer/artifact hashes stay enabled.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_HOME="$PWD/work/toolchains/cargo"
export RUSTUP_HOME="$PWD/work/toolchains/rustup"
mkdir -p work/downloads
if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  curl -fsSL https://static.rust-lang.org/rustup/archive/1.29.1/x86_64-unknown-linux-gnu/rustup-init -o work/downloads/rustup-init
  printf '%s\n' 'dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71  work/downloads/rustup-init' | sha256sum -c -
  chmod +x work/downloads/rustup-init
  work/downloads/rustup-init -y --no-modify-path --profile minimal --default-toolchain none
fi
export PATH="$CARGO_HOME/bin:$PATH"
rustup set auto-self-update disable
rustup toolchain install 1.99.1 --profile minimal --component llvm-tools-preview --component rustfmt --target x86_64-unknown-none

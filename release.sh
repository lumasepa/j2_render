#!/usr/bin/env bash
# Subshell to not pollute env vars
(
echo "Building target for platform x86_64-apple-darwin"

# Add osxcross toolchain to path
export PATH="/opt/osxcross/bin:${PATH}"

# Use Clang for C/C++ builds
export CC=o64-clang
export CXX=o64-clang++
export LD_LIBRARY_PATH="/opt/osxcross/lib"

cargo build --release --target x86_64-apple-darwin
strip target/x86_64-apple-darwin/release/j2_render
)

echo "Building target for platform x86_64-unknown-linux-musl"
cargo build --target x86_64-unknown-linux-musl --release
strip target/x86_64-unknown-linux-musl/release/j2_render
#echo "Building target for platform x86_64-pc-windows-msvc"
#cargo build --target x86_64-pc-windows-msvc --release
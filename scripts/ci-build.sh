#!/bin/bash
# CI 多平台编译 + Release
set -e

TARGETS=(
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

# 安装 cross 编译工具
rustup target add "${TARGETS[@]}"

# 编译
for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    cargo build --release --target "$target"
done

echo ""
echo "Build complete. Binaries:"
ls -lh target/*/release/codeloom target/*/release/codeloom.exe 2>/dev/null || true

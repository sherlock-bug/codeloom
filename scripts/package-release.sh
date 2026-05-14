#!/bin/bash
# CodeLoom 发布打包脚本
# 用法: ./scripts/package-release.sh <version> <binary-path>
# 示例: ./scripts/package-release.sh v1.1.2 /tmp/codeloom-release/codeloom
set -euo pipefail

VERSION="$1"
BINARY="$2"
OUTDIR="/tmp/codeloom-release"
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== CodeLoom $VERSION 打包 ==="

# 验证二进制
if ! file "$BINARY" | grep -q "static-pie"; then
    echo "⚠️  警告: 二进制不是 static-pie linked"
fi
if readelf -d "$BINARY" 2>/dev/null | grep NEEDED; then
    echo "⚠️  警告: 二进制有动态库依赖"
fi
VER=$(strings "$BINARY" | grep "CodeLoom v" | head -1)
echo "  版本: $VER"

# 准备目录
rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"

# 复制文件
cp "$BINARY" "$OUTDIR/codeloom"
cp "$SCRIPT_DIR/scripts/install.sh" "$OUTDIR/"
cp "$SCRIPT_DIR/scripts/clang_filter.py" "$OUTDIR/"

# 打包
ARCHIVE="codeloom-${VERSION}-linux-x86_64.tar.gz"
cd "$OUTDIR"
tar czf "$ARCHIVE" codeloom install.sh clang_filter.py
echo "  打包: $(pwd)/$ARCHIVE ($(du -h "$ARCHIVE" | cut -f1))"

# 验证
echo "  内含:"
tar tzf "$ARCHIVE"


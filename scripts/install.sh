#!/bin/bash
# CodeLoom 一键安装脚本 (Linux/macOS)
# 用法:
#   curl -sSL https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.sh | bash
#   curl -sSL ... | bash -s -- --from-source   # 从源码编译安装

set -e

FROM_SOURCE=false
for arg in "$@"; do
    case "$arg" in
        --from-source) FROM_SOURCE=true ;;
        --help) echo "Usage: curl ... | bash [-s -- --from-source]"; exit 0 ;;
    esac
done

INSTALL_DIR="${HOME}/.codeloom/bin"
mkdir -p "$INSTALL_DIR"

if $FROM_SOURCE; then
    # ── 从源码编译安装 ──────────────────────────────────────
    echo "=== 从源码编译安装 CodeLoom ==="
    command -v cargo >/dev/null 2>&1 || { echo "需要 Rust 工具链: curl -sSf https://sh.rustup.rs | sh"; exit 1; }

    REPO="https://github.com/sherlock-bug/codeloom.git"
    TMPDIR=$(mktemp -d)
    trap "rm -rf $TMPDIR" EXIT

    git clone --depth 1 "$REPO" "$TMPDIR/codeloom"
    cd "$TMPDIR/codeloom"
    cargo build --release
    cp target/release/codeloom "$INSTALL_DIR/codeloom"

else
    # ── 下载预编译二进制 ────────────────────────────────────
    BASE_URL="https://github.com/sherlock-bug/codeloom/releases/latest/download"
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64)  BINARY="codeloom-linux-x86_64" ;;
                aarch64) BINARY="codeloom-linux-arm64" ;;
                *)       echo "Unsupported arch: $ARCH. Try: curl ... | bash -s -- --from-source"; exit 1 ;;
            esac
            ;;
        darwin)
            case "$ARCH" in
                x86_64)  BINARY="codeloom-darwin-x86_64" ;;
                arm64)   BINARY="codeloom-darwin-arm64" ;;
                *)       echo "Unsupported arch: $ARCH. Try: curl ... | bash -s -- --from-source"; exit 1 ;;
            esac
            ;;
        *) echo "Unsupported OS: $OS. Try: curl ... | bash -s -- --from-source"; exit 1 ;;
    esac

    echo "Downloading codeloom for $OS/$ARCH..."
    # Try direct first, fall back to ghproxy mirror
    if ! curl -sSL --connect-timeout 10 --max-time 60 "$BASE_URL/$BINARY" -o "$INSTALL_DIR/codeloom" 2>/dev/null; then
        echo "Direct download slow, trying ghproxy mirror..."
        curl -sSL "https://ghproxy.net/$BASE_URL/$BINARY" -o "$INSTALL_DIR/codeloom"
    fi
fi

chmod +x "$INSTALL_DIR/codeloom"

# ── 添加到 PATH ──────────────────────────────────────────
SHELL_CONFIG=""
if [ -f "$HOME/.bashrc" ]; then SHELL_CONFIG="$HOME/.bashrc"; fi
if [ -f "$HOME/.zshrc" ]; then SHELL_CONFIG="$HOME/.zshrc"; fi
if [ -n "$SHELL_CONFIG" ]; then
    if ! grep -q "codeloom/bin" "$SHELL_CONFIG" 2>/dev/null; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
    fi
fi

echo ""
echo "CodeLoom installed to $INSTALL_DIR/codeloom"
echo "  $INSTALL_DIR/codeloom --version"
echo ""
echo "Quick start:"
echo "  source $SHELL_CONFIG   # or restart your shell"
echo "  codeloom check          # verify environment"
echo "  codeloom index .        # index current project"
echo "  codeloom status         # check status"

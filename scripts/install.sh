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
    # Try ghproxy first (faster from China), fall back to direct
    if ! curl -sSL --connect-timeout 10 --max-time 120 "https://ghproxy.net/$BASE_URL/$BINARY" -o "$INSTALL_DIR/codeloom" 2>/dev/null; then
        echo "Mirror failed, trying direct download..."
        curl -sSL "$BASE_URL/$BINARY" -o "$INSTALL_DIR/codeloom"
    fi
fi

chmod +x "$INSTALL_DIR/codeloom"

# ── 添加到 PATH ──────────────────────────────────────────
# Detect the user's shell to pick the right config file
SHELL_CONFIG=""
case "$(basename "$SHELL")" in
    zsh)  SHELL_CONFIG="$HOME/.zshrc" ;;
    bash) SHELL_CONFIG="$HOME/.bashrc" ;;
    *)    # Fallback: use first existing
          for f in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
              if [ -f "$f" ]; then SHELL_CONFIG="$f"; break; fi
          done ;;
esac
if [ -n "$SHELL_CONFIG" ]; then
    if ! grep -q "codeloom/bin" "$SHELL_CONFIG" 2>/dev/null; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
        echo "→ Added to $SHELL_CONFIG"
    fi
else
    echo "→ Add to PATH manually: export PATH=\"$INSTALL_DIR:\$PATH\""
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

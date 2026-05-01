#!/bin/bash
# CodeLoom 一键安装脚本 (Linux/macOS)
set -e

BASE_URL="https://github.com/xxx/codeloom/releases/latest/download"

# 检测 OS 和架构
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64)  BINARY="codeloom-linux-x86_64" ;;
            aarch64) BINARY="codeloom-linux-arm64" ;;
            *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    darwin)
        case "$ARCH" in
            x86_64)  BINARY="codeloom-darwin-x86_64" ;;
            arm64)   BINARY="codeloom-darwin-arm64" ;;
            *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

# 下载二进制
INSTALL_DIR="${HOME}/.codeloom/bin"
mkdir -p "$INSTALL_DIR"
echo "Downloading codeloom for $OS/$ARCH..."
curl -sSL "$BASE_URL/$BINARY" -o "$INSTALL_DIR/codeloom"
chmod +x "$INSTALL_DIR/codeloom"

# 添加到 PATH
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
echo ""
echo "Quick start:"
echo "  source $SHELL_CONFIG   # or restart your shell"
echo "  codeloom setup-opencode # install OpenCode commands"
echo "  codeloom index          # index current project"
echo "  codeloom status         # check status"

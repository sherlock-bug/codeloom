#!/bin/bash
# CodeLoom 一键安装脚本 (Linux/macOS)
# 用法:
#   curl -sSL https://gitee.com/greengreensea/codeloom/raw/master/scripts/install.sh | bash
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
CONFIG_DIR="${HOME}/.codeloom"

# ── 离线模式检测 ──────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$SCRIPT_DIR/codeloom" ] && ! $FROM_SOURCE; then
    echo "=== 离线安装模式 ==="
    mkdir -p "$INSTALL_DIR" "$CONFIG_DIR"

    # 清除旧版本（所有已知路径）
    for old in "$INSTALL_DIR/codeloom" "$HOME/.local/bin/codeloom" /usr/local/bin/codeloom; do
        [ -f "$old" ] && rm -f "$old" && echo "  清理旧版: $old"
    done

    cp "$SCRIPT_DIR/codeloom" "$INSTALL_DIR/codeloom"
    chmod +x "$INSTALL_DIR/codeloom"
    echo "  Binary → $INSTALL_DIR/codeloom"
    # 跳转到 PATH 配置
else
    mkdir -p "$INSTALL_DIR"

if $FROM_SOURCE; then
    # ── 从源码编译安装 ──────────────────────────────────────
    echo "=== 从源码编译安装 CodeLoom ==="
    command -v cargo >/dev/null 2>&1 || { echo "需要 Rust 工具链: curl -sSf https://sh.rustup.rs | sh"; exit 1; }

    REPO="https://gitee.com/greengreensea/codeloom.git"
    TMPDIR=$(mktemp -d)
    trap "rm -rf $TMPDIR" EXIT

    git clone --depth 1 "$REPO" "$TMPDIR/codeloom"
    cd "$TMPDIR/codeloom"
    cargo build --release
    cp target/release/codeloom "$INSTALL_DIR/codeloom"
    chmod +x "$INSTALL_DIR/codeloom"

else
    # ── 下载预编译二进制 ────────────────────────────────────
    VERSION="v0.5.3"
    BASE_URL="https://gitee.com/greengreensea/codeloom/releases/download/${VERSION}"
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64)  BINARY="codeloom" ;;
                aarch64) BINARY="codeloom" ;;   # 暂只有 x86_64 静态链接
                *)       echo "Unsupported arch: $ARCH. Try: curl ... | bash -s -- --from-source"; exit 1 ;;
            esac
            ;;
        darwin)
            echo "macOS 暂不支持预编译二进制，请从源码编译: curl ... | bash -s -- --from-source"; exit 1
            ;;
        *) echo "Unsupported OS: $OS. Try: curl ... | bash -s -- --from-source"; exit 1 ;;
    esac

    echo "Downloading codeloom for $OS/$ARCH from Gitee..."
    curl -sSL --connect-timeout 10 --max-time 120 "$BASE_URL/$BINARY" -o "$INSTALL_DIR/codeloom"
    chmod +x "$INSTALL_DIR/codeloom"
fi
fi

# ── 默认配置（不存在时创建）──────────────────────────────
if [ ! -f "${CONFIG_DIR}/config.yaml" ]; then
    cat > "${CONFIG_DIR}/config.yaml" << 'YAML'
# CodeLoom 配置
# embedding 用于语义搜索，需配置 OpenAI 兼容 API
# embedding:
#   api_base: "http://your-api:port/v1"
#   model: "bge-m3"
#   api_key: "not-needed"
YAML
    echo "默认配置已创建: ${CONFIG_DIR}/config.yaml"
fi

# ── 添加到 PATH ──────────────────────────────────────────
SHELL_CONFIG=""
case "$(basename "$SHELL")" in
    zsh)  SHELL_CONFIG="$HOME/.zshrc" ;;
    bash) SHELL_CONFIG="$HOME/.bashrc" ;;
    *)    for f in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
              [ -f "$f" ] && { SHELL_CONFIG="$f"; break; }
          done ;;
esac
if [ -n "$SHELL_CONFIG" ] && ! grep -qF "${INSTALL_DIR}" "$SHELL_CONFIG" 2>/dev/null; then
    echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$SHELL_CONFIG"
    echo "→ Added to $SHELL_CONFIG"
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

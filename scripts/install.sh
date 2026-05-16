#!/bin/bash
# CodeLoom 一键安装脚本 (Linux/macOS)
#
# 离线安装:
#   解压 codeloom-vX.Y.Z-linux-x86_64.zip → ./install.sh
#
# 在线安装:
#   curl -sSL https://gitee.com/greengreensea/codeloom/raw/master/scripts/install.sh | bash
#
# 从源码安装:
#   curl -sSL ... | bash -s -- --from-source

set -e

FROM_SOURCE=false
for arg in "$@"; do
    case "$arg" in
        --from-source) FROM_SOURCE=true ;;
        --help) echo "Usage: ./install.sh [--from-source]"; exit 0 ;;
    esac
done

INSTALL_DIR="${HOME}/.codeloom/bin"
SCRIPTS_DIR="${HOME}/.codeloom/scripts"
CONFIG_DIR="${HOME}/.codeloom"

# ── 离线模式检测 ──────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HAS_LOCAL_BIN=false
HAS_FILTER=false
[ -f "$SCRIPT_DIR/codeloom" ] && HAS_LOCAL_BIN=true
[ -f "$SCRIPT_DIR/clang_filter.py" ] && HAS_FILTER=true

if $HAS_LOCAL_BIN && ! $FROM_SOURCE; then
    echo "=== 离线安装模式 ==="

    # ── 依赖检测（不阻塞） ──────────────────────────────────
    if command -v clang &>/dev/null; then
        echo "  clang: OK ($(clang --version 2>/dev/null | head -1))"
    else
        echo "  clang: 未安装。C/C++ 索引需要 clang++。"
        echo "         安装: sudo apt install clang  (Ubuntu/Debian)"
        echo "               sudo dnf install clang  (Fedora)"
        echo "               sudo pacman -S clang    (Arch)"
    fi

    if command -v python3 &>/dev/null; then
        echo "  python3: OK ($(python3 --version 2>/dev/null))"
    else
        echo "  python3: 未安装。C/C++ 索引需要 python3。"
        echo "           安装: sudo apt install python3  (Ubuntu/Debian)"
        echo "                 sudo dnf install python3  (Fedora)"
    fi

    # ── 清理旧文件 ──────────────────────────────────────────
    echo "  清理旧文件..."
    rm -rf "$INSTALL_DIR" && mkdir -p "$INSTALL_DIR"
    rm -rf "$SCRIPTS_DIR" && mkdir -p "$SCRIPTS_DIR"

    # ── 部署新文件 ──────────────────────────────────────────
    cp "$SCRIPT_DIR/codeloom" "$INSTALL_DIR/codeloom"
    chmod +x "$INSTALL_DIR/codeloom"
    echo "  Binary → $INSTALL_DIR/codeloom"

    if $HAS_FILTER; then
        cp "$SCRIPT_DIR/clang_filter.py" "$SCRIPTS_DIR/clang_filter.py"
        chmod +x "$SCRIPTS_DIR/clang_filter.py"
        echo "  Filter → $SCRIPTS_DIR/clang_filter.py"
    else
        echo "  [WARN] clang_filter.py 缺失——C++ 索引将无法工作"
    fi

    # ── 配置文件保护 ────────────────────────────────────────
    if [ -f "${CONFIG_DIR}/config.yaml" ]; then
        echo "  Config: 保留已有配置 → ${CONFIG_DIR}/config.yaml"
    else
        mkdir -p "$CONFIG_DIR"
        cat > "${CONFIG_DIR}/config.yaml" << 'YAML'
# CodeLoom 配置
# embedding 用于语义搜索，需配置 OpenAI 兼容 API
# embedding:
#   api_base: "http://your-api:port/v1"
#   model: "bge-m3"
#   api_key: "not-needed"

# logging: 文件日志配置（默认启用，日志在 ~/.codeloom/logs/）
#   enabled: true          # 是否启用文件日志
#   level: "info"          # 日志级别: error / warn / info / debug
#   max_file_size_mb: 50   # 单日志文件上限（MB）
#   max_files: 10          # 保留的日志文件数（旧的自动清理）
YAML
        echo "  Config: 已创建默认配置 → ${CONFIG_DIR}/config.yaml"
    fi

    # ── 安装后清理 ──────────────────────────────────────────
    echo "  清理临时文件..."
    rm -f "$SCRIPT_DIR/codeloom"
    $HAS_FILTER && rm -f "$SCRIPT_DIR/clang_filter.py"

    # 跳转到 PATH 配置
else
    mkdir -p "$INSTALL_DIR" "$SCRIPTS_DIR"

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

    # 源码模式也部署 filter script（来自仓库）
    if [ -f "scripts/clang_filter.py" ]; then
        cp scripts/clang_filter.py "$SCRIPTS_DIR/clang_filter.py"
        chmod +x "$SCRIPTS_DIR/clang_filter.py"
    fi

else
    # ── 下载预编译二进制 ────────────────────────────────────
    VERSION="v1.3.0"
    BASE_URL="https://gitee.com/greengreensea/codeloom/releases/download/${VERSION}"
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64)  BINARY="codeloom" ;;
                aarch64) BINARY="codeloom" ;;
                *)       echo "Unsupported arch: $ARCH. Try: curl ... | bash -s -- --from-source"; exit 1 ;;
            esac
            ;;
        darwin)
            echo "macOS 暂不支持预编译二进制，请从源码编译: curl ... | bash -s -- --from-source"; exit 1
            ;;
        *) echo "Unsupported OS: $OS. Try: curl ... | bash -s -- --from-source"; exit 1 ;;
    esac

    echo "Downloading codeloom for $OS/$ARCH from Gitee..."
    ARCHIVE="codeloom-${VERSION}-linux-x86_64.tar.gz"
    curl -sSL --connect-timeout 10 --max-time 120 "${BASE_URL}/${ARCHIVE}" -o "/tmp/${ARCHIVE}"
    if [ $? -ne 0 ] || [ ! -s "/tmp/${ARCHIVE}" ]; then
        echo "Gitee download failed, trying GitHub..."
        curl -sSL --connect-timeout 10 --max-time 120 \
          "https://github.com/sherlock-bug/codeloom/releases/download/${VERSION}/${ARCHIVE}" \
          -o "/tmp/${ARCHIVE}"
    fi
    if [ $? -ne 0 ] || [ ! -s "/tmp/${ARCHIVE}" ]; then
        echo "Download failed. Try: install from source"
        exit 1
    fi
    tar xzf "/tmp/${ARCHIVE}" -C "$INSTALL_DIR"
    rm -f "/tmp/${ARCHIVE}"
    chmod +x "$INSTALL_DIR/codeloom"
fi
fi

# ── PATH 配置 ──────────────────────────────────────────────
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

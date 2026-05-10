# Design: 独立离线安装压缩包

## Context

当前安装流程：
1. `curl install.sh | bash` 从 GitHub 下载二进制
2. install.sh 尝试 ghproxy.net 镜像，失败回退直连
3. models/ 文件由 build.rs 编译时下载，install.sh 不处理

痛点：
- 公司防火墙封 raw.githubusercontent.com 和 ghproxy.net
- GitHub Releases 页面可浏览器访问，但只发布裸二进制
- 首次运行时模型文件缺失，语义搜索降级到 Jaccard

## Goals / Non-Goals

**Goals:**
- 用户从 GitHub Releases 下载一个 zip，解压 `./install.sh` 即可完成完整安装
- `make release-zip` 一键打包，输出标准化命名的 zip
- install.sh 自动检测离线/在线模式，无需用户传参
- zip 方案保持文件命名规范，与 codeloom-release skill 兼容

**Non-Goals:**
- 不支持 macOS/Windows 的独立 zip（本次仅 Linux x86_64）
- 不改变 CI/CD pipeline（当前无 CI，纯手动 release）
- 不改变运行时模型路径查找逻辑

## System Architecture

```
make release-zip
  ├── 1. 检查 binary 存在: target/release/codeloom
  ├── 2. 检查 models/: bge-small-zh/* + sqlite-vec/vec0.so
  ├── 3. cp binary → /tmp/codeloom-release/codeloom
  ├── 4. cp -r models/ → /tmp/codeloom-release/models/
  ├── 5. cp scripts/install.sh → /tmp/codeloom-release/install.sh
  └── 6. cd /tmp/codeloom-release && zip -r codeloom-vX.Y.Z-linux-x86_64.zip .
```

## install.sh 离线模式逻辑

```bash
# 检测逻辑
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HAS_BINARY="no"
HAS_MODELS="no"

if [ -f "$SCRIPT_DIR/codeloom" ]; then
    HAS_BINARY="yes"
fi
if [ -d "$SCRIPT_DIR/models/bge-small-zh" ] && \
   [ -f "$SCRIPT_DIR/models/bge-small-zh/pytorch_model.bin" ] && \
   [ -f "$SCRIPT_DIR/models/sqlite-vec/vec0.so" ]; then
    HAS_MODELS="yes"
fi

if [ "$HAS_BINARY" = "yes" ] && [ "$HAS_MODELS" = "yes" ]; then
    # 离线模式：本地复制
    cp "$SCRIPT_DIR/codeloom" "$INSTALL_DIR/"
    mkdir -p "$HOME/.codeloom/models"
    cp -r "$SCRIPT_DIR/models/"* "$HOME/.codeloom/models/"
else
    # 在线模式：网络下载（原有逻辑）
    curl ...
fi
```

## Decisions

### Decision: 离线检测用文件存在性而非版本号
选择检测 `models/bge-small-zh/pytorch_model.bin` 存在性，而非版本号哈希。
因为：模型文件稳定（bge-small-zh 不会变），不需要版本比对。vec0.so 同理。

### Decision: models 安装到 ~/.codeloom/models/
选择 `~/.codeloom/models/` 而非 `~/.codeloom/bin/models/`。
因为：二进制查找模型的三级路径包含 `~/.codeloom/models/bge-small-zh/`（见 embedding/mod.rs:61），从 bin 同级找不到时会 fallback 到这里。

### Decision: 在线模式不自动合并
选择本地文件和在线下载互斥，不尝试「先本地后补下载」。
因为：公司网络环境下载必失败，混用只会增加等待时间。本地有就全用本地。

## Project Directory Structure

```
codeloom/
├── scripts/
│   └── install.sh           # 修改：新增离线模式分支
├── Makefile                  # 修改：新增 release-zip 目标
├── README.md                 # 修改：新增离线安装章节
└── models/                   # 不变：zip 从此复制
```

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| zip 体积大（~107MB），GitHub Release 单文件上限 2GB | 107MB 远低于上限，上次发布 14.8MB 裸二进制也正常上传 |
| 用户下载 zip 后手动解压，不知道要跑 install.sh | README 和 GitHub Release body 明确说明步骤 |
| 未来加入 mac/win 支持时 zip 变体增多 | 文件名含 OS-ARCH，make release-zip 自动检测 |

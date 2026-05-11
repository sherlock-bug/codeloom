# Design: offline-install-package

## Context

当前 install.sh（115 行）支持三种安装模式：
- 离线模式：检测当前目录有 codeloom 二进制时直接复制
- 编译模式：`--from-source` 从 Gitee git clone 后编译
- 下载模式：从 Gitee Releases 下载 v0.6.1 预编译二进制

三个问题：
1. 离线包缺少 `clang_filter.py`，且路径硬编码 `~/.hermes/scripts/`，脱离 Hermes 无法运行
2. 打包了已不需要的 bge-small-zh 模型（~91MB）
3. 无依赖检测、无配置文件保护、不清理临时文件

## Goals / Non-Goals

**Goals:**
- 离线安装包解压即用（binary + clang_filter.py + install.sh）
- install.sh 自动检测 clang、python3，缺失时给出安装指引
- 保护 `~/.codeloom/config.yaml` 不被覆盖
- 安装前清理旧 binary/scripts，安装后删除解压临时文件
- `codeloom check` 验证环境完整性
- clang_filter.py 路径改为 `~/.codeloom/scripts/`

**Non-Goals:**
- 不自动安装 clang/python3（包管理器差异大，只输出安装命令）
- 不改 Windows install.ps1（本次只做 Linux 端，Windows 后续同步）
- 不涉及在线下载模式改造（保留原有行为）

## Decisions

### Decision: 离线安装包精简内容

release zip 从 `binary + models(91MB) + vec0.so + install.sh` 改为 `binary + clang_filter.py + install.sh`。

理由：
- 用户使用 API 向量化（ZhipuAI embedding-3），本地模型不需要
- sqlite-vec 已静态编译进 Rust binary
- zip 体积从 ~100MB 降至 ~4MB，下载和安装更快

### Decision: filter 脚本路径迁移

从 `~/.hermes/scripts/clang_filter.py` 改为 `~/.codeloom/scripts/clang_filter.py`。

理由：
- codeloom 不应依赖 Hermes Agent 存在
- 遵循 XDG 风格，所有 codeloom 文件集中在 `~/.codeloom/`
- install.sh 负责部署此文件

代码改动：`src/indexer/clang/mod.rs` 中 `parse_file()` 函数的一行路径。

### Decision: 依赖检测策略

install.sh 安装前检测 `which clang` 和 `which python3`，缺失时输出安装命令并继续安装（不阻塞）。

理由：
- clang 安装方式因发行版而异（apt/yum/pacman/手动），自动安装不可靠
- 输出安装命令让用户自行决定包管理方式
- 不阻塞安装流程——用户可能暂时不需要 C++ 索引功能

### Decision: 配置文件保护策略

install.sh 安装前检查 `~/.codeloom/config.yaml` 是否存在：
- 存在 → 跳过，输出 "保留已有配置"
- 不存在 → 创建默认配置模板（当前行为）

清理范围：`rm -rf ~/.codeloom/bin/ ~/.codeloom/scripts/`，不碰 `config.yaml` 和 `models/`。

### Decision: make release-zip 更新

同步更新 Makefile 的 `release-zip` 目标：
- 移除 `models/bge-small-zh/` 和 `models/sqlite-vec/` 相关检查
- 新增 `clang_filter.py` 打包
- 保留 `install.sh`

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| clang_filter.py 路径迁移后旧路径遗留 | install.sh 清理旧文件时也清理 `~/.hermes/scripts/clang_filter.py` |
| 用户有本地模型但被清理 | 清理范围限制在 `bin/` 和 `scripts/`，不碰 `models/` |
| Windows install.ps1 未同步 | 本次仅 Linux 端，需单独开 SDD Change 跟 Windows |
| 已有用户升级时 config 被覆盖 | 显式检查 + 跳过，默认安全 |

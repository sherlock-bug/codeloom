# Proposal: offline-install-package

## Why

当前 install.sh 离线安装体验不好：既不检测运行时依赖（clang、python3），也不保护用户配置文件，还会因为缺少 `clang_filter.py` 导致 Clang 解析直接挂。需要一次真正的「解压 → 运行 → 用」零配置体验。

## What Changes

- **离线安装包重构**：release zip 内容改为 `codeloom`（二进制）+ `clang_filter.py`（Python 过滤脚本）+ `install.sh`，移除已不再需要的 bge-small-zh 模型和 sqlite-vec/vec0.so
- **依赖自动检测**：install.sh 安装前检测 clang、python3，缺失时给出明确的安装命令提示
- **配置文件保护**：`~/.codeloom/config.yaml` 已存在时不覆盖，不存在时创建默认模板
- **安装清理**：覆盖旧 binary 和旧 scripts，安装完成后删除当前目录的解压临时文件
- **`codeloom check` 增强**：验证命令增加 clang/python3/filter 脚本可用性检查
- **代码修复**：`clang_filter.py` 路径从 `~/.hermes/scripts/` 改为 `~/.codeloom/scripts/`

## Capabilities

### New Capabilities
- `offline-install-package`: 离线安装脚本，包含依赖检测、配置文件保护、安装清理

### Modified Capabilities
- `standalone-release-zip`: 离线安装包内容变更——移除模型文件，新增 clang_filter.py；`make release-zip` 目标同步更新

## Impact

- `src/indexer/clang/mod.rs` — 改 filter 脚本路径
- `src/cli/mod.rs` — 增强 `codeloom check` 命令
- `scripts/install.sh` — 重写
- `scripts/install.ps1` — 同步更新
- `Makefile` — 更新 `release-zip` 目标（移除模型打包）

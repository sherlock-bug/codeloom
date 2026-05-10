# Proposal: cli-auto-detection-and-fixes

## Why
当前 CLI 体验有三个痛点：(1) 除 `index` 和 `status` 外，`search`/`overview`/`list-symbols`/`get-definition`/`call-graph`/`list-branches`/`clean` 共 7 个命令的 `--repo` 和 `--branch` 是必填参数，每次查询都要手动输入；(2) 7 个命令的 shell tab 补全不完整；(3) 文档向量化进度计数有 double-increment bug。

## What Changes
1. **CLI 自动检测 repo/branch** — 提取 `autodetect_repo()` 和 `autodetect_branch()` 公共函数，7 个命令的 `--repo`/`--branch` 从必填改为可选（自动检测）
2. **修复 doc_processed 重复计数** — 删除 `index_doc_vectors` 中 skip 分支内的多余 `doc_processed += 1`
3. **修复无子命令默认行为** — `codeloom` 不带参数时展示帮助而非直接启动 MCP

## Capabilities

### New Capabilities
- `cli-auto-repo-branch`: 7 个 CLI 查询命令的 `--repo` 和 `--branch` 参数改为可选，自动从当前目录检测

### Modified Capabilities
- `cli-mode`: 原有 CLI 命令定义，现 `--repo`/`--branch` 参数语义从 required 变为 optional-with-autodetect

## Impact
- CLI 体验：在 git 仓库内直接 `codeloom search "token"` 而不必每次都写 `--repo leveldb --branch main`
- 补全：`codeloom list-<tab>` 能补全出 `list-repos` 和 `list-branches`
- 正确性：leveldb 第二次 index 不再显示 101/55 的异常数字

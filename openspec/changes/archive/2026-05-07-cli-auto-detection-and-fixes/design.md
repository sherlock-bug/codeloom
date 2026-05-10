# Design: cli-auto-detection-and-fixes

## Context
CodeLoom 当前 CLI 中，`search`/`overview`/`list-symbols`/`get-definition`/`call-graph` 等查询命令的 `--repo` 和 `--branch` 是必填参数。`Index` 命令已有完善的自动检测逻辑，但其他命令没有复用。

## Goals / Non-Goals
- ✅ 7 个查询命令的 `--repo`/`--branch` 改为可选 + 自动检测
- ✅ 修复 `index_doc_vectors` 的 doc_processed 重复计数
- ✅ 无子命令时展示帮助而非 MCP
- ❌ 不改变 MCP 工具的接口
- ❌ 不改变数据库 schema

## Decisions

### Decision 1: 公共检测函数放在 `src/cli/mod.rs` 顶层
选择放在 CLI 模块而非新建 `autodetect` 模块，因为：
- 仅两个简单函数（<20 行），不构成独立模块
- CLI 命令是唯一调用方
- 减少模块拆分碎片

### Decision 2: doc_processed 统一递增点选在循环末尾
删除 skip 分支内的 `doc_processed += 1`（第 387 行），保留循环末尾的统一递增（第 414 行）。统一递增点更安全，不会因未来新增分支而再次重复。

### Decision 3: repo 自动检测复用 Index 的逻辑
`autodetect_repo()` 逻辑：
1. 如果是 git 仓库 → 取仓库目录名（与 Index 一致）
2. 如果是非 git 目录且无显式 --repo → 返回空字符串（全局可见）
3. 如果当前目录就是名为 `.` → 取真实 cwd 目录名

### Decision 4: branch 自动检测通过 git.rs 的 current_branch
直接调用已有的 `crate::indexer::git::current_branch(".")` 函数，无需重写。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 非 git 目录下执行查询命令时 branch 为 "unknown"，可能查不到数据 | 文档提示 + `codeloom check` 时可检测当前目录是否有 git 仓库 |
| `list-branches` 的 `repo` 参数原是 positional required，改 optional 需调整 clap 定义 | 将 `repo` 从 positional 改为 `--repo` option |
| 补全问题可能是 clap 自身行为而非代码问题 | 先确认 root cause，若 clap derive 已正确则不额外改动 |

## Migration Plan
1. 添加 `autodetect_repo()` 和 `autodetect_branch()` 函数
2. 修改 7 个命令的 clap 定义（参数改 optional）
3. 修改对应 run() match 分支（调用 autodetect 函数填充默认值）
4. 修复 `index_doc_vectors` 重复计数
5. 修改 `main.rs` 中 `None` 分支行为
6. `cargo test && cargo build --release`

# Tasks

## 1. 公共检测函数

- [x] 1.1 在 `src/cli/mod.rs` 添加 `autodetect_repo()` 函数：复用 Index 逻辑（git 仓库取目录名 + cwd 回退）
- [x] 1.2 在 `src/cli/mod.rs` 添加 `autodetect_branch()` 函数：调用 `crate::indexer::git::current_branch(".")`

## 2. CLI 命令参数自动检测

- [x] 2.1 Search：`--repo` 和 `--branch` 从 required 改为 optional，run() 分支填充默认值
- [x] 2.2 Overview：同上
- [x] 2.3 ListSymbols：同上
- [x] 2.4 GetDefinition：同上
- [x] 2.5 CallGraph：同上
- [x] 2.6 ListBranches：`repo` 从 positional required 改为 `--repo` optional
- [x] 2.7 Clean：`--repo` 填充默认值
- [x] 2.8 Status：现有 `--repo` 自动检测逻辑改为调用 `autodetect_repo()`

## 3. Bug 修复

- [x] 3.1 `src/embedding/mod.rs`：删除 `index_doc_vectors` 中第 387 行多余的 `doc_processed += 1`
- [x] 3.2 `src/main.rs`：`None` 分支从 `mcp::serve()` 改为输出帮助信息

## 4. 验证

- [x] 4.1 `cargo test` 全部通过
- [x] 4.2 `cargo build --release` 成功
- [x] 4.3 在 leveldb 仓库内测试不带参数查询：`codeloom search "write_batch"`
- [x] 4.4 更新 README.md

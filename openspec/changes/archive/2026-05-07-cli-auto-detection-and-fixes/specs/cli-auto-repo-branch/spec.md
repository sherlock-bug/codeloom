# Delta for cli-auto-repo-branch

## ADDED Requirements

### Requirement: 公共自动检测函数
系统 SHALL 在 CLI 模块中提供 `autodetect_repo()` 和 `autodetect_branch()` 两个公共函数，供所有 CLI 命令复用。

#### Scenario: git 仓库内自动检测
- GIVEN 当前目录为 git 仓库 `/home/user/projects/leveldb`，当前分支为 `main`
- WHEN 调用 `autodetect_repo()` 和 `autodetect_branch()`
- THEN 分别返回 `"leveldb"` 和 `"main"`

#### Scenario: 非 git 目录自动检测
- GIVEN 当前目录为非 git 仓库 `/tmp/test`
- WHEN 调用 `autodetect_repo()`
- THEN 返回目录名 `"test"`
- AND 调用 `autodetect_branch()` 返回 `"unknown"`

## MODIFIED Requirements

### Requirement: 查询命令的 repo 和 branch 参数
系统 SHALL 将 `search`、`overview`、`list-symbols`、`get-definition`、`call-graph`、`list-branches`、`clean` 共 7 个命令的 `--repo` 和 `--branch` 参数从 required 改为 optional，缺失时自动调用 `autodetect_repo()` 和 `autodetect_branch()` 填充。

#### Scenario: 无参数查询自动检测
- GIVEN 当前目录为 leveldb git 仓库（分支 main），已有索引数据
- WHEN 执行 `codeloom search "write_batch"`（不带 --repo 和 --branch）
- THEN 系统 SHALL 自动检测 repo=leveldb, branch=main 并返回搜索结果

#### Scenario: 显式指定覆盖自动检测
- GIVEN 当前目录为 leveldb 仓库
- WHEN 执行 `codeloom overview --repo leveldb --branch feature-x`
- THEN 系统 SHALL 使用显式指定的 branch=feature-x 而非自动检测的 main

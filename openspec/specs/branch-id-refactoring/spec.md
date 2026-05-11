# branch-id-refactoring Specification

## Purpose
TBD - created by archiving change branch-id-refactoring. Update Purpose after archive.
## Requirements
### Requirement: branch_meta 表
系统 SHALL 在 schema 中创建 `branch_meta` 表，作为分支 ID 的唯一主表。

#### Scenario: 表创建
- GIVEN 执行 schema 初始化
- THEN `branch_meta` 表存在
- AND 包含 `id INTEGER PK`、`repo TEXT NOT NULL`、`branch_name TEXT NOT NULL`
- AND `(repo, branch_name)` 有 UNIQUE 约束

### Requirement: 节点分支引用改为 ID
系统 SHALL 将 `nodes` 表的 `branch_name TEXT` 改为 `branch_id INTEGER NOT NULL DEFAULT 0`。

#### Scenario: 节点插入
- GIVEN 索引器有一个符号需要写入
- AND 已知 repo 和 branch_name
- WHEN 插入 nodes 表
- THEN 先通过 `branch_meta` 解析 branch_name 为 branch_id
- THEN nodes.branch_id 写入该 ID 而非字符串

### Requirement: 边分支引用改为 ID
系统 SHALL 将 `edges` 表的 `branch_name TEXT` 改为 `branch_id INTEGER NOT NULL DEFAULT 0`，UNIQUE 索引同步更新为 `(source_id, edge_type, branch_id)`。

#### Scenario: 边插入
- GIVEN 解析器提取到一条调用边
- AND 已知 source_id、edge_type、branch_id
- WHEN 插入 edges 表
- THEN branch_id 写入该 ID
- AND UNIQUE 索引确保同一源+类型+分支不重复

#### Scenario: 多分支边隔离
- GIVEN 仓库在 master 和 feature 两个分支均已索引
- AND master 的符号 A calls B
- WHEN feature 的 A 也有 calls B 边
- THEN 两条边 SHALL 各自存储（不同 branch_id），不被 UNIQUE 丢弃

### Requirement: branches 表引用改为 ID
系统 SHALL 将 `branches` 表的 `branch_name TEXT` 改为 `branch_id INTEGER`。

#### Scenario: 分支映射
- GIVEN 节点 n 属于 branch 'master'
- WHEN 写入 branches 表
- THEN branch_id 为 branch_meta 中 (repo, 'master') 对应的 ID

### Requirement: 分支解析工具函数
系统 SHALL 提供一个统一的工具函数 `resolve_branch_id(conn, repo, branch_name) -> i64`，在插入节点/边前调用。

#### Scenario: 重复调用
- GIVEN 同一仓库同分支名被多次请求
- WHEN 首次调用后
- THEN 后续调用直接返回缓存的 ID
- AND 不重复插入 branch_meta

### Requirement: 全量清库重索引
`codeloom clean --all` + `codeloom index` SHALL 在新 schema 上正常工作。

#### Scenario: 重索引验证
- GIVEN 执行 `codeloom clean --all`
- WHEN 重新索引 leveldb 仓库
- THEN 索引成功，无报错
- AND nodes.branch_id 全部为有效值（非 0）
- AND edges.branch_id 全部为有效值（非 0）
- AND branches 表包含所有节点到分支的映射


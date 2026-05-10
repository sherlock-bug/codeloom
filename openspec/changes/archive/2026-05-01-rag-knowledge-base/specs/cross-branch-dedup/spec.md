# Delta for Cross-Branch Deduplication

## ADDED Requirements

### Requirement: 基于内容哈希的符号去重
系统 SHALL 对每个符号的定义文本计算 SHA256 哈希，相同哈希的符号跨分支共享存储，仅在 branches 表中记录分支归属。

#### Scenario: 两个分支上相同函数
- GIVEN main 分支已索引 func_a (SHA256=abc123)
- AND feature 分支上的 func_a 源码与 main 完全一致
- WHEN 索引 feature 分支
- THEN func_a 的 definition 不重复存储
- AND branches 表新增一行 (symbol_id=func_a_id, branch_name='feature', override_def=NULL)

#### Scenario: 两个分支上不同函数
- GIVEN main 分支已索引 func_a (SHA256=abc123)
- AND feature 分支修改了 func_a 的实现 (SHA256=def456)
- WHEN 索引 feature 分支
- THEN func_a 的 base 版本仍保留 (SHA256=abc123)
- AND branches 表新增一行 (symbol_id=func_a_id, branch_name='feature', override_def='修改后的源码', override_hash='def456')

### Requirement: 双层数据库架构
系统 SHALL 支持共享层（base DB）和本地层（overlay DB）的双层数据库架构，查询时自动合并。

#### Scenario: 本地分支查询合并共享层数据
- GIVEN project.rag.db 包含 main 分支的 100,000 个符号
- AND project.rag.overlay.db 包含 feature 分支的 5,000 个增量符号
- WHEN 执行查询 rag_get_call_graph render --branch feature
- THEN 系统先从 overlay 查找，再从 base 查找
- AND 返回合并后的结果（feature 有覆盖的用覆盖，没有的用 base）

### Requirement: 存储节省可度量
系统 SHALL 在多分支索引完成后报告去重节省的存储空间。

#### Scenario: 三个分支索引后查看去重效果
- GIVEN 项目有三个已索引分支，80% 代码相同
- WHEN 执行 `rag status`
- THEN 显示 "存储节省: 53% (600MB → 280MB)"
- AND 列出每个分支的符号数、独立符号数、共享符号数

### Requirement: 覆盖层隔离
系统 SHALL 确保 overlay DB 的写入不影响 base DB 的完整性。

#### Scenario: 并发操作
- GIVEN base DB 被多人只读挂载
- WHEN 用户 A 在本地 overlay 中索引新分支
- THEN base DB 不受任何修改
- AND 所有写操作仅发生在 overlay DB

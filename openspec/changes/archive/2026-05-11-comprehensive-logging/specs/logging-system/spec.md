# Delta for logging-system

## ADDED Requirements

### Requirement: AST 解析日志覆盖
Clang AST 解析器（`src/indexer/clang/ast.rs`）SHALL 在以下决策点输出 debug 日志：`is_project_file()` 的输入路径和匹配结果、`is_external` 判定结果（含 decl_file 来源）、每个符号提取的类型和行号。模块入口 SHALL 输出 info 日志含翻译单元文件名，退出前 SHALL 输出 info 日志含符号数和耗时。

#### Scenario: 头文件被误标为 external
- GIVEN Clang 解析 version_set.cc 时遇到 class Compaction（定义于 version_set.h）
- WHEN 解析器调用 is_project_file(decl_file) 判断是否为项目文件
- THEN 日志 SHALL 输出 `is_project_file` 的输入路径和返回结果
- AND 日志级别为 debug

### Requirement: 文件收集日志覆盖
文件收集器（`src/indexer/tree_sitter.rs`）SHALL 在入口输出 info 日志含扫描根目录，退出前输出 info 日志含收集文件数。跳过的文件 SHALL 在 debug 级别输出跳过原因（语言不支持、被 ignore 匹配等）。

#### Scenario: 大项目文件收集
- GIVEN `collect_files("/mnt/d/code/leveldb")` 被调用
- WHEN 完成文件扫描
- THEN 日志 SHALL 输出 `collect_files done: 120 files, 15 skipped in 45ms`
- AND 文件路径和跳过原因仅在 debug 级别可见

### Requirement: 搜索查询日志覆盖
搜索模块（`src/query/search.rs`）SHALL 在入口输出 info 日志含查询词、repo、branch、limit，退出前输出 info 日志含命中数和耗时。向量搜索的加载状态和 KNN 参数 SHALL 在 debug 级别输出。BM25 分数分布 SHALL 在 debug 级别输出前 3 位。

#### Scenario: 关键词搜索调试
- GIVEN 用户执行 `codeloom search Compaction --repo leveldb --branch master`
- WHEN 搜索完成
- THEN 日志 SHALL 输出 `search query: Compaction, repo=leveldb, branch=master, limit=10`
- AND 日志 SHALL 输出 `search done: 3 hits in 12ms`

### Requirement: 校准过程日志覆盖
校准模块（`src/calib/mod.rs`）SHALL 在入口输出 info 日志含待校准仓库/分支，退出前输出 info 日志含校准结果和耗时。每个仓库校准失败时 SHALL 输出 error 日志含失败原因。校准得分分布 SHALL 在 debug 级别输出。

#### Scenario: 单仓库校准失败
- GIVEN `codeloom calibrate --repo leveldb` 执行
- WHEN 校准过程中某个分支的 SQL 查询失败
- THEN 日志 SHALL 输出 `calibrate error: leveldb master: SQL error: <reason>`
- AND 不因单分支失败而终止整个校准过程

### Requirement: 存储层操作日志覆盖
存储层模块（FTS5、Symbol upsert、Vector、Schema migration）SHALL 在关键操作入口/退出输出日志：FTS 重建完成（info，含行数）、符号 upsert 类型（debug，含 name/kind）、向量表加载状态（warn 如果未安装 vec0）、Schema 迁移执行结果（info，含成功/失败）。

#### Scenario: FTS 重建耗时追踪
- GIVEN `fill_all_fts()` 被调用
- WHEN 完成 FTS 表填充
- THEN 日志 SHALL 输出 `fts done: 2188 rows in 85ms`

### Requirement: 默认日志级别改为 debug
系统 SHALL 在无用户配置时默认日志级别为 debug，确保新用户首次使用即有足够诊断信息。

#### Scenario: 无配置启动
- GIVEN 用户未配置 logging.level
- WHEN CodeLoom 启动
- THEN 默认日志级别 SHALL 为 debug
- AND 标准输出中可见 info 级别日志

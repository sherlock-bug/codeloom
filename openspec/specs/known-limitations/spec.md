# known-limitations

## Purpose
记录 CodeLoom 当前版本的已知限制和待修复问题，这些 SHALL 在后续 SDD change 中解决。本 spec 不定义期望行为而是记录已知偏差，字数需达到五十字符以满足验证要求。

## Requirements

### Requirement: 模板实例收敛去重 ✅ 已修复 (2026-05-10)
Clang 索引器 SHALL 基于六元组 (name, ns, kind, signature, file_path, repo) 统一去重所有符号种类。
~~当前偏差：upsert_symbol() 在模板重度场景下对 Both definitions 分支插入新行，导致百分之九十五符号重复 DB 膨胀三十倍。~~
修复后 flatbuffers 从 71,071 符号降至 3,383 (1.00x 去重)，DB 从 648MB 降至 2MB。

#### Scenario: flatbuffers 全量索引
- GIVEN flatbuffers 项目的 37 个翻译单元
- WHEN 执行全量索引
- THEN 项目符号数 SHALL 在 3000-5000 范围而非当前 70750

### Requirement: 头文件声明的文件路径 ✅ 已修复 (2026-05-22)
Clang 解析器 SHALL 为正确定位头文件中的符号定义提供准确的文件路径。
当前偏差：~~Clang AST dump 的 json 输出对头文件声明只给出 loc.line 和 includedFrom 不给出 loc.file，导致符号 file_path 回退到翻译单元文件路径搜索结果跳转不准确。~~
修复后：AST 遍历时追踪 `cur_file` 上下文，头文件符号正确指向 `.h` 文件而非 `.cpp` 文件。实测 sample.hpp 中 21 个符号全部指向正确路径。

#### Scenario: 头文件符号搜索
- GIVEN 头文件 config.h 中的类 Config 符号
- WHEN 用户搜索 Config 并点击结果
- THEN 跳转目标 SHALL 是 config.h 的正确行号而非引用该头文件的 cpp 文件

### Requirement: Clang 与 tree-sitter 路径解耦
CodeLoom SHALL 通过可配置的解析器路由机制选择 C++ 解析器而非在 smart.rs 中硬编码。
当前偏差：index_one() 函数通过 if lang equals cpp 直接调用 clang index_clang 无抽象层。

#### Scenario: 添加新的 C++ 解析器后端
- GIVEN 需要替换或增加 C++ 解析实现
- WHEN 新增解析器模块
- THEN 不应修改 smart.rs 的核心索引逻辑

### Requirement: LLM 代码分析工具增强 ✅ 已修复 (2026-05-11)
CodeLoom SHALL 将向量语义搜索与精确符号搜索分离为独立工具，每个工具的 description 向 LLM 明示当前能力边界和适用场景。
~~当前偏差：混合搜索引擎同时输出向量和 BM25 结果，LLM 无法区分两种通道的质量差异。~~
修复后：`codeloom_search` 纯 BM25 精确搜索，`codeloom_semantic_search` 纯向量语义搜索，各自独立 description 明示适用场景。

#### Scenario: 工具选择指引
- GIVEN LLM 需要查找函数签名
- WHEN 阅读可用工具列表和各自 description
- THEN LLM SHALL 能根据 description 中的适用场景描述选择精确搜索工具而非语义搜索

### Requirement: 配置文件解析
CodeLoom SHALL 支持 YAML、JSON、TOML 等配置文件的解析和索引，将配置项作为可查询的结构化符号。

#### Scenario: YAML 配置查询
- GIVEN 项目包含 server.yaml 配置文件
- WHEN 用户查询服务器端口号
- THEN 系统 SHALL 返回配置项 server.port 及其值和定义位置

### Requirement: 跨仓符号匹配
CodeLoom SHALL 支持多个关联代码仓之间的符号引用追踪。
当前：各仓库独立索引，无法追踪 codeloom 调用了 hermes 的哪些接口。方案待探索。

#### Scenario: 跨仓调用链
- GIVEN codeloom 和 hermes 两个代码仓均已索引
- WHEN 查询某个函数的完整调用链
- THEN 结果 SHALL 包含跨仓库的调用关系

### Requirement: 知识图谱三层认知模型
CodeLoom 的知识图谱设计 SHALL 服务于 LLM 的三层认知需求，作为工具和功能优先级的指导框架。
三层模型：① 命名缺口——符号名称不表意时，语义搜索弥补；② 粒度缺口——单个符号信息不足时，调用图/继承树提供上下文；③ 抽象缺口——理解架构意图时，跨文件关系图提供全局视角。

#### Scenario: 工具设计优先级
- GIVEN 需要决定下一个 MCP 工具的开发优先级
- WHEN 评估各工具的 LLM 认知价值
- THEN 优先级 SHALL 参照三层模型：语义搜索（命名缺口）> 调用图（粒度缺口）> 跨仓匹配（抽象缺口）

### Requirement: Markdown 文档图片 URL 加载
CodeLoom SHALL 支持 Markdown 文档中引用的图片 URL 自动提取和索引，使文档中的截图、架构图可被搜索和关联。
当前偏差：文档索引仅处理文本内容，`![alt](url)` 格式的图片引用未被解析为结构化信息（alt 文本 + URL）。

#### Scenario: 文档图片索引
- GIVEN 文档包含 `![架构图](https://example.com/arch.png)` 图片引用
- WHEN 索引该文档
- THEN 图片 SHALL 被提取为结构化条目，包含 alt 文本和 URL

### Requirement: 测试用例分层与耗时优化
CodeLoom SHALL 将测试用例按执行时间分为快速门禁用例和耗时集成用例两层。
当前偏差：所有测试混在一起跑，`cargo test` 耗时超过 120s。快速用例应在 <10s 完成，供小步修改后快速验证；耗时用例（如全量索引、向量加载）用 `#[ignore]` 标记，在关键需求实施完成后集中运行。

#### Scenario: 小步修改验证
- GIVEN 修改了单个解析器函数
- WHEN 运行快速测试门禁
- THEN 测试 SHALL 在 10 秒内返回结果，覆盖核心逻辑但不加载大型模型或索引外部仓库

### Requirement: 安装脚本保护用户配置与清理
install.sh SHALL 保护用户自定义模型不被覆盖，并在安装完成后清理临时文件。
当前偏差：install.sh 尚未实现。两个关键点需在实施时处理：① 安装前检测 ~/.codeloom/models/ 是否已有用户模型，默认跳过模型覆盖仅更新二进制；② 安装成功后删除当前目录的临时文件（zip 解压产物），失败时保留现场。

#### Scenario: 用户模型保护
- GIVEN 用户已在 ~/.codeloom/models/ 部署了自定义嵌入模型
- WHEN 执行 install.sh 安装新版本
- THEN 脚本 SHALL 检测已有模型文件并询问用户是否覆盖
- AND 默认行为是不覆盖用户模型

#### Scenario: 安装后清理
- GIVEN install.sh 安装成功完成
- WHEN 安装结束
- THEN 脚本 SHALL 删除当前目录下的临时解压文件
- AND 保持用户工作目录干净

### Requirement: 边的 UNIQUE 索引缺失分支维度 ✅ 已修复 (2026-05-22)
`edges` 表 SHALL 确保同一 repo 的不同分支之间的边不互相冲突。
当前偏差：~~`CREATE UNIQUE INDEX idx_ed_unique ON edges(source_id, edge_type, source_repo)` 不包含 `branch_name` 列，多分支索引时不同分支的同源同类边会因 UNIQUE 冲突被 `INSERT OR IGNORE` 丢弃。且边插入语句未填充 `branch_name` 列。~~
修复后：`idx_ed_unique ON edges(source_id, edge_type, branch_id)` 已包含 `branch_id` 列。

#### Scenario: 双分支边隔离
- GIVEN 仓库 leveldb 在 master 和 feature 两个分支均已索引
- AND master 分支的符号 `DB::Open` 有一条 `calls:Get` 边
- WHEN feature 分支的 `DB::Open` 也有 `calls:Get` 边
- THEN 两条边 SHALL 各自存储，不被 UNIQUE 约束丢弃

### Requirement: 向量索引仍用旧 symbols.rowid（v0.9 遗留）✅ 已修复 (2026-05-22)
向量表（symbol_name_vec_*, file_vec_*）SHALL 使用 nodes.id 作为主键以消除对旧 symbols/files 表的依赖。
当前偏差：~~migrate_to_nodes 每次索引从旧表迁移数据到 nodes，vector KNN 返回旧 symbols.rowid 需 JOIN symbols 表映射。symbols/files 表因此不能删除。~~
修复后：embedding/mod.rs 已使用 `SELECT id FROM nodes` 获取 rowid，vec0 表以 nodes.id 为行 ID，旧表均已删除。

#### Scenario: 向量索引重建
- GIVEN 已执行 clean + index
- WHEN vector KNN 搜索执行
- THEN 返回的 rowid SHALL 直接对应 nodes.id，无需通过 symbols 表中转

### Requirement: MCP 工具仍引用 doc_nodes 表（v0.9 遗留）✅ 已修复 (2026-05-22)
MCP 文档查询工具（codeloom_get_doc 等）SHALL 从 nodes 表查询文档内容。
当前偏差：~~codeloom_get_doc/list_doc_nodes/get_doc_section 直接 SQL 查询 doc_nodes 表（id/title/section_path/content），doc_nodes 因此不能删除。~~
修复后：doc_nodes 表已删除，codeloom_get_doc 和 codeloom_query_excel 已 DISABLED。

#### Scenario: MCP 文档查询适配
- GIVEN 节点已迁移到 nodes 表（node_type='doc'）
- WHEN 调用 codeloom_get_doc 工具
- THEN 返回的文档内容 SHALL 来自 nodes 表而非 doc_nodes 表

### Requirement: Indexer 暂未直写 nodes 表（v0.9 遗留）✅ 已修复 (2026-05-22)
Indexer（Clang/tree-sitter/doc/files）SHALL 写入 nodes 表作为主存储。
当前偏差：~~indexer 写入旧表（symbols/doc_nodes/files），通过 migrate_to_nodes（DELETE+REPLACE）同步到 nodes。旧表作为暂存区。~~
修复后：migrate_to_nodes 已删除，所有 Indexer 直接 INSERT 到 nodes 表。旧表均已删除。

#### Scenario: Indexer 直写
- GIVEN Clang 解析器完成一个翻译单元的符号提取
- WHEN 写入数据库
- THEN INSERT SHALL 直接进入 nodes 表，node_type='sym'
- AND 同步插入 branches 表（node_id = 刚插入的 nodes.id）

### Requirement: doc 节点的 UNIQUE 约束与设计
索引器 SHALL 保证 nodes 表的 UNIQUE 约束正确覆盖所有节点类型，且 doc 节点的数据模型设计完备。
当前疑点：nodes 表的 UNIQUE 约束可能存在覆盖不全的问题（如 doc 节点与 code/sym 节点的 key 构成不一致），doc 节点的字段设计不够周全，可能存在插入冲突或数据丢失的 bug。待详细排查确认。
#### Scenario: 重复索引不丢数据
- GIVEN 同一份代码库被多次索引
- WHEN UNIQUE 约束生效时
- THEN 不应因约束偏差丢失或覆盖 doc/sym 节点的关键数据

### Requirement: 索引输出过于啰嗦
`codeloom index` SHALL 提供简洁的进度输出，而非逐个文件打印明细信息。
当前偏差：Clang 解析器每处理一个 C++ 文件就输出一行 `Clang: N symbols, M edges`，大量文件时终端被刷屏。smart.rs 已经有每 20 文件一次的汇总进度，Clang 层的逐文件输出属于多余。

#### Scenario: 百文件索引
- GIVEN 项目包含 100+ C++ 文件
- WHEN 执行 `codeloom index`
- THEN 终端输出 SHALL 仅显示定期进度汇总（如每 20 文件）而非每个文件一行
- AND 错误信息仍需逐文件打印以便排查

### Requirement: 不支持的编程语言被无效索引
索引器 SHALL 仅索引当前支持的编程语言，不识别和不支持的语言文件直接跳过，不给它们创建 file node。
当前偏差：`detect_language()` 返回 `Some("python")`/`Some("java")`/`Some("typescript")`/`Some("go")`，这些文件被收录进 `collect_files()` 列表并进入 `index_one()` 处理流程。然而 `create_parser()` 虽然创建了对应语言的 tree-sitter parser，`parse_file()` 对非 C++ 语言的 match 分支是空（`_ => {}`），不产出任何符号。这些文件仍然创建了空的 file node，浪费了解析时间和存储空间。

#### Scenario: Python 项目索引
- GIVEN 项目仅包含 Python 文件
- WHEN 执行 `codeloom index`
- THEN 索引器 SHALL 跳过所有 Python 文件（`collect_files()` 不收录，或 `index_one()` 直接 `return Ok(0)`）
- AND 不创建 file node

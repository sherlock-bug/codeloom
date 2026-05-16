# CodeLoom CHANGELOG

## 2026-05-16 — fix-abstract-interface-cross-tu

- **FIX**: 抽象接口指针跨 TU 调用边（G18）— `ast.rs` 的 MemberExpr 类名提取增加 `trim_start_matches("const ")`，修复 `const AbstractWorker *` → target 变为 `AbstractWorker::DoOp` 而非 `const AbstractWorker::DoOp`
- **VERIFIED**: 断言 198/198 全绿（G18d 通过）

## 2026-05-16 — fix-cross-tu-namespace

- **FIX**: 节点 FQN 命名 — 方法/函数节点名包含完整 namespace 前缀，跨 TU 调用边的 target_name 与 DB 节点名统一（`ast.rs`）
- **FIX**: 跨 TU 调用边 — `name_to_id` 查找失败时 fallback 到 DB 查询（精确 + LIKE 后缀剥离兜底）
- **FIX**: line_end schema 迁移 — 消除 `attrs` JSON 双写不一致，列是唯一来源（G5d 修复）
- **FIX**: `inspect_symbol` SQL — 去掉不存在的 `is_definition` 列引用
- **NEW**: 断言仓 G5d/G17 修复，G18 抽象接口指针跨 TU 调用边场景（已知限制）
- **VERIFIED**: 断言 197/199 通过（2 个 G18 限制独立修复），leveldb `VersionSet::AddLiveFiles` 跨 TU 边确认存在

## 2026-05-12 — fix-mcp-graph-branch-filtering

- **FIX**: 所有图分析 MCP 工具（call_graph、path_analysis、impact_analysis、neighbor_graph、inheritance_tree、inspect_symbol）的 `edges` 表查询统一添加 `branch_id` 过滤，多分支索引边数据不再污染。
- **FIX**: `inheritance_tree` 中的 `build_tree` 和 `get_overrides` 修复——从硬编码 `branch_id = 0` 改为传参透传实际 branch_id。
- **CHANGE**: `graph.rs` 的 `get_edges()`、`build_forward_adj()`、`build_reverse_adj()`、`bfs_path_search()`、`transitive_closure()`、`neighbor_map()` 新增 `branch_id` 参数。
- **CHANGE**: `call_graph.rs` 的 `traverse_calls()` 从 `branch: &str` 改为 `branch_id: i64` 参数，移除内联的 `resolve_branch_id` 调用。
- **NEW**: `openspec/specs/mcp-tools/spec.md` 新增域规范，定义图分析工具的分支隔离共享约束。

## 2026-05-10 — add-logging-system

- **NEW**: 文件日志系统（`src/logger.rs`），支持时间戳、PID、TID、级别、模块路径的格式化日志。
- **NEW**: 四级日志 `log_error!` / `log_warn!` / `log_info!` / `log_debug!` 宏，替代原有 `eprintln!`。
- **NEW**: 配置文件 `logging` 节控制开关和级别，默认开启（`enabled: true`, `level: info`）。
- **NEW**: 文件绕接（超 `max_file_size_mb` 自动创建新文件）+ 数量限制（超 `max_files` 删除最旧）。
- **NEW**: AGENTS.md 日志诊断章节，新会话可通过 `grep ERROR ~/.codeloom/logs/` 快速定位问题。
- **CHANGE**: 移除 `log` 和 `env_logger` crate 依赖。
- **DELIVERED**: CLI index、MCP 工具调用、Clang 解析、FTS5 索引入口/出口打点。

## 2026-05-10 — clang-core-parser

- **NEW**: Clang 子进程解析器替代 tree-sitter C++ 解析器。使用 `clang -fsyntax-only -Xclang -ast-dump=json` 通过 Python 过滤器剥离系统头文件和函数体。
- **NEW**: 15 种符号节点类型（function/method/class/struct/enum/template 等）+ 11 种边类型（calls/inherits/param_type/return_type/contains/overrides/aliases 等）。
- **NEW**: compile_commands.json 自动发现与解析，提取编译标志传递给 Clang。
- **NEW**: 外部符号存根机制——通过 project_root + .codeloomignore 白名单区分项目符号和外部符号。
- **NEW**: Python 流式过滤器（`clang_filter.py`）预处理 Clang AST JSON，剥离系统头文件节点和函数体，并对嵌套项目节点自动传播父节点的文件路径。
- **SCHEMA**: v8 迁移新增 6 列（sid, access, is_virtual, is_definition, is_external, template_args）及索引。
- **DELETED**: 移除 tree-sitter-cpp 依赖和相关查询模块（`src/indexer/queries/cpp.rs`）。
- ~~**KNOWN BUG**: 收敛逻辑 `upsert_symbol()` 在模板重度场景下（flatbuffers）导致 95% 符号重复，DB 膨胀 30 倍。~~ ✅ 已修复 (2026-05-10, 见下方)

## 2026-05-10 — fix-template-symbol-dedup

- **FIX**: `upsert_symbol()` 改为六元组全键匹配 `(name, ns, kind, signature, file_path, repo)`，统一去重所有符号种类。
- **FIX**: 名字自编码层级——FieldDecl/CXXMethodDecl/EnumConstantDecl 拼成全限定名 `Class::member`，移除 `parent_class` 匹配依赖。
- **FIX**: `FunctionTemplateDecl` 不再创建冗余占位符号，子节点第一个 `FunctionDecl` → `template_function`，后续 → `template_instance`。
- **FIX**: `create_external_stub` 泛化为 `(name, kind, ns, repo)`，`infer_stub_kind()` 从边类型推断目标符号种类。
- **IMPACT**: flatbuffers 71,071 → 3,383 符号 (1.00x 去重)，DB 648MB → 2MB (324x 压缩)。全 13 种符号 kind 去重比 1.0x。

## 2026-05-09 — v0.6.1 MCP 分析工具数据质量修复

- **变更**: `fix-mcp-edge-data` — 修复 3 项数据质量 bug
- **call_graph**: LIKE fallback 排除 string_literal/enum_value，不再匹配到 FLAGS 声明
- **图遍历**: get_edges + adj builders 过滤 target_id=0 断边，消除空字符串邻居
- **inheritance_tree**: inherits 查询增加 class/struct kind 校验，消除假阳性
- **验证**: 77 测试全过，58/58 specs

## 2026-05-09 — v0.6.0 MCP 分析工具集（图遍历引擎 + 5 个新工具 + schema 元数据）

- **变更**: `mcp-analysis-tools` — 删除 overview，新增 5 个分析工具 + 1 个元数据工具
- **新增**: `codeloom_schema`（15 节点类型 + 10 边类型）、`codeloom_path_analysis`（BFS 最短路径）、`codeloom_impact_analysis`（传递闭包）、`codeloom_neighbor_graph`（分组邻里）、`codeloom_inheritance_tree`（继承树）
- **增强**: `codeloom_get_call_graph` 终端节点附带 uses/references/literals 依赖
- **核心引擎**: `src/query/graph.rs` — 跨边类型 BFS/DFS、环检测、方向控制、边类型过滤
- **验证**: 76 测试全过，55/55 specs
## 2026-05-09 — v0.5.5 字符串字面量去重

- **变更**: `string-literal-dedup` — `content_hash` 从空字符串改为内容哈希
## 2026-05-09 — v0.5.5 字符串字面量去重

- **变更**: `string-literal-dedup` — `content_hash` 从空字符串改为内容哈希，触发 UNIQUE 约束自动去重
- **一行改动**: `cpp.rs:957` `String::new()` → `dedup::hash_content(name)`
- **验证**: 75 测试全过，46/46 specs
## 2026-05-09 — v0.5.4 调用解析 v2：auto 跨函数 + 虚函数展开 + 重载消歧

- **变更**: `call-resolution-v2` — 三个调用解析增强
- **auto 跨函数**: `auto x = getFoo()` 通过符号签名解析返回类型，传入类型表供后续成员调用
- **虚函数展开**: 基类方法调用检测 override，生成 `calls_override:Derived::method` 附加边
- **重载消歧**: 参数计数 → 边标签 `calls:func(N)`，`resolve_target` 支持后缀降级匹配
- **新增 spec**: `auto-cross-function`、`virtual-dispatch`、`overload-disambiguation`（8 Requirement / 14 Scenario）
- **验证**: 75 测试全过，46/46 specs validate 通过
## 2026-05-09 — v0.5.3 类型感知调用解析 + 内置符号 + call_graph 模块化

- **变更**: `type-aware-call-resolution` — 成员调用通过变量类型声明解析到正确类方法
- **类型感知**: `scan_local_declarations()` 扫描函数体内局部声明+参数，建变量→类型映射
- **成员调用**: `obj->method()` → `calls:TypeName::method`；`this->method()` → `calls:ParentClass::method`
- **内置符号**: ~70 个 std 容器方法+算法函数合成节点（repo=__builtin__），不参与搜索/向量化
- **模块化**: `call_graph.rs` 从 stub → 真模块，MCP 调模块 API
- **新增 spec**: `type-aware-call-resolution`、`builtin-symbols`、`call-graph-module`（10 Requirement / 18 Scenario）
- **验证**: 75 测试全过，43/43 specs validate 通过，leveldb 实测 DBImpl::Get → DBImpl::MaybeScheduleCompaction
## 2026-05-09 — v0.5.2 搜索排名优化：符号名与注释分离加权

- **变更**: `search-name-comment-split` — FTS5 + 向量搜索按符号名和注释分通道独立搜索
- **FTS5**: `search_symbols_name()` MATCH name+signature+kind, `search_symbols_comment()` MATCH doc_comment
- **向量**: `symbol_name_vec_{repo}` 和 `symbol_comment_vec_{repo}` 独立表，名嵌入和注释嵌入分离
- **融合**: 名通道权重 0.7，注释通道权重 0.3，去掉子串 boost
- **新增 spec**: `search-name-comment-split`（4 个 Requirement / 8 个 Scenario）
- **验证**: 66 单元 + 9 集成全过，40/40 specs validate 通过
## 2026-05-07 — v0.5.1 注释搜索 + 文件节点 + 文档切分

- **变更**: `comment-file-chunk` — 注释收集与搜索、文件节点索引、文档智能切分
- **注释**: C++ 行内注释 + 体内注释自动收集，FTS5 + 向量索引，支持中文/英文语义搜索
- **文件节点**: 代码文件元信息索引 + 注释摘要拼接，文档文件按类型索引，三通道搜索（BM25 + 向量）
- **文档切分**: ≤500 字 chunk，标点优先级（句号>问号>感叹号>分号>逗号），父子节点层级保留
- **Schema v7**: symbols.doc_comment、doc_nodes.parent_id、files 表、fts5_files、file_vec_*
- **搜索**: 四通道融合（符号 + 文档 + 文件 BM25 + 向量），文件结果带注释摘要，文档结果附 doc_id
- **新增 spec**: `comment-indexing`、`doc-chunking`、`file-nodes`
- **验证**: 73 测试全过，spdlog 索引 1357 符号 + 87 文档 + 148 文件，四通道 100% 无报错

## 2026-05-06 — v0.5.0 搜索质量 + 嵌入模型迁移

- **变更**: `fix-search-ranking` — 混合搜索加权融合、去 definition 列、智谱 embedding-2 迁移、智能分批、doc_id 可追溯
- **搜索**: 0.5\*BM25_norm + 0.5\*cosine 替代 RRF，kind 过滤，文档搜索结果返回 snippet + doc_id
- **嵌入**: candle → 智谱 embedding-2（1024 维，OpenAI 兼容 API），build.rs 模型下载逻辑移除
- **去重**: 去 definition 列（FTS5 + 向量），DB 缩小、向量更聚焦
- **分批**: 智能批次（双限制条数 ≤64 + 字符 ≤90k），可配置
- **其他**: HTML 文件跳过索引，删除 doc-code-linking，CLI 显示 `[id:N]`，MCP 显示 `|doc_id:N`
- **删除 spec**: `code-doc-linking`（预计算关联边未被使用）
- **验证**: openspec 35/35 通过，cargo test 62/63 通过（1 ignored），清库重验 1757 符号 + 57 文档

## 2026-05-19 — v0.5.0 多格式文档与图片支持

- **变更**: `multi-format-doc-image` — 5 种新文档格式（Excel/Word/PDF/XML/HTML）+ 图片提取与 BLOB 存储
- **新增**: `src/parser/xlsx.rs`（四层节点模型）、`src/parser/docx.rs`、`src/parser/pdf.rs`、`src/parser/xml_html.rs`、`src/embedding/image.rs`（WebP 压缩+BLOB）
- **新增 MCP 工具**: `codeloom_get_doc`、`codeloom_query_excel`（MCP 工具 9→11）
- **新增依赖**: calamine, zip, quick-xml, pdf-extract, image, webp, base64
- **Schema**: doc_images 表（BLOB 存储）、doc_nodes 加 node_type、edges 加 source_kind/target_kind
- **规范**: 新增 10 个功能域 spec（doc-format-routing/docx-parser/excel-query/image-compression/image-extraction/mcp-get-doc/mcp-search-with-images/pdf-parser/xlsx-parser/xml-html-parser），各 1-3 Requirement
- **验证**: openspec 33/33 通过，cargo test 74/74 通过，musl static-pie 编译成功（15MB）

## 2026-05-06 — v0.3.5 GB2312/GBK 编码支持

- **变更**: `gb2312-encoding-support` — UTF-8/GB18030 自动检测，中文编码源码零配置索引
- **新增**: `src/util.rs`（read_file_smart）、`encoding_rs` 依赖、`tests/fixtures/gb2312/sample.cpp`
- **修改**: `parse_file`/`index_includes`/`index_docs` 三处改用 `read_file_smart`
- **规范**: 新增 `gb2312-encoding-support` 功能域，2 个 Requirement / 6 个 Scenario
- **验证**: openspec 12/12 通过，cargo test 53/53 通过，GB2312 fixture 索引正确（1 文件→2 符号+2 include 边）

## 2026-05-06 — v0.3.2 独立离线安装包

- **变更**: `release-standalone-zip` — 一键打包离线安装 zip，防火墙环境零网络部署
- **新增**: `make release-zip` 目标、install.sh 离线检测模式、.gitignore zip 排除
- **修改**: install.sh（离线/在线自动切换）、README（离线安装章节）、codeloom-release skill（zip 上传步骤）
- **规范**: 新增 `standalone-release-zip` 功能域 spec，2 个 Requirement / 6 个 Scenario
- **验证**: openspec validate 11/11 通过，cargo test 48/48 通过，离线安装端到端测试通过

## 2026-05-06 — v0.3.0 sqlite-vec ANN 向量搜索

- **变更**: `sqlite-vec-vector-search` — O(log N) ANN 替代 O(N) 暴力搜索
- **新增**: `storage/vector.rs`（loadable vec0 扩展，160KB 预编译 .so），`embedding/index_vectors()`, build.rs vec0.so 下载
- **修改**: `semantic_search()` 优先 vec0 ANN，`cli index` 调用 index_vectors
- **测试**: 42 单元 + 6 集成全部通过，vec0 加载成功，ANN 语义搜索正确

## 2026-05-05 — v0.2.1 candle 嵌入引擎

- **变更**: `candle-embedding` — candle + bge-small-zh 替换 Jaccard 占位符
- **新增**: `CandleEmbedder` (candle + BertModel, 384维余弦相似度), `codeloom download-model` CLI
- **新增 spec**: `onnx-embedding` — candle 嵌入引擎功能域
- **修改 spec**: `semantic-search` / `code-doc-linking` — 从 ONNX/sqlite-vec 切换为 candle/余弦相似度
- **依赖**: candle-core/nn/transformers v0.10 + tokenizers v0.21（全部纯 Rust，零 C 依赖）
- **降级策略**: 模型文件缺失时自动 fallback 到 TextEmbedder (Jaccard)，功能不中断
- **验证**: 11/11 主 spec 通过，编译 0 错误

## 2026-05-05 — v0.2.0 架构简化：单人维护 + 分支查询增强

- **变更**: `simplify-single-maintainer-branch-query` — 移除团队共享功能，MCP 分支查询增强
- **删除**: Pull/Push/SwitchBranch CLI + MCP 工具 + OpenCode 命令（均为占位代码，从未实现）
- **修改**: 8 个 MCP 查询工具 `branch` 参数改为必传
- **统一过滤规则**: `branch_name IS NULL` 的数据所有分支可见；有值的按分支精确匹配。代码和文档一视同仁
- **移除 spec**: `team-sharing` 整个功能域移除
- **验证**: 10/10 主 spec 通过，8 工具 MCP 测试通过，编译 0 错误

     2|     2|
     3|     3|## 2026-05-01 — CodeLoom v0.1.0
     4|     4|
     5|     5|- **变更**: `rag-knowledge-base` — 团队代码知识管理工具的初始设计与实现
     6|     6|- **Capabilities**: 11 个功能域（代码索引、文档索引、语义搜索、文档关联、多仓、分支管理、去重、MCP、共享、CLI、OpenCode 命令）
     7|     7|- **实现**: Rust 项目（`codeloom/`），84 任务，11 测试通过，零警告
     8|     8|- **二进制**: 2.4MB release build, `--version`/`--help`/`codeloom index` 文件目录双模式/MCP 13 工具
     9|     9|- **规范**: 11 spec 验证通过（0 失败），已同步至主规范库 + 归档
    10|    10|
    11|    11|### 命令
    12|    12|- `codeloom check` — 检查安装状态（binary/config/db）
    13|    13|- `codeloom push` — 维护者推送 base DB 到团队共享存储
    14|    14|- `codeloom branch set-alias <alias> <branch>` + `list-aliases` — 分支惯用叫法管理
    15|    15|- `codeloom index` 统一代码+文档索引，支持文件和目录参数，自动识别文件类型
    16|    16|- OpenCode 命令: `/codeloom:index`, `/codeloom:status`, `/codeloom:pull`, `/codeloom:check`, `/codeloom:branch`, `/codeloom:push`（全部 MCP 调用）
    17|    17|
    18|    18|### Git 集成
    19|    19|- `codeloom index` 自动跟踪已索引 commit，`git diff --name-only` 仅扫描变更文件
    20|    20|- commit 不变自动跳过；`git_index_state` 表持久化索引状态
    21|    21|- 新分支 `find_best_parent` 在所有已索引分支中找最近祖先，继承符号
    22|    22|- 支持 `--parent develop` 显式指定源分支，跳过自动检测
    23|    23|
    24|    24|### 分支术语
    25|    25|- `## 23B (release/xxx)` / `## 23B / release/xxx` / `## 23B → release/xxx` 三种格式
    26|    26|- `codeloom index` 自动从 Markdown 文档提取术语映射到 `branch_glossary` 表
    27|    27|- `codeloom branch set-alias` 命令行直接添加单条映射
    28|    28|
    29|    29|### 团队协作
    30|    30|- 维护者/普通成员角色分工文档
    31|    31|- base + overlay 双层 DB，push/pull/index 三角关系
    32|    32|- 路径无关化（相对路径 + config root 拼接 + SHA256 校验）
    33|    33|
    34|    34|### 修复
    35|    35|- `--version` 未注册 → `#[command(version)]`
    36|    36|- `index`/`index-docs`/`index-glossary` 三合一 → 统一 `codeloom index`
    37|    37|
    38|- **优化**: override-only 模式 — branches 表只存实际变更的符号（非全量 NULL 行），parent_ref 链回溯查询，20 ref 场景从 120MB 降至 70MB（省 42%）
### 第二轮验证修复 (2026-05-01)
- **修复**: C++ parser — `template_declaration` 和 `namespace_definition` 递归提取，模板类识别 (class: 25→83, method: 0→679)
- **修复**: 成员方法加 `ClassName::` 前缀，与自由函数区分
- **修复**: CMakeLists.txt 被误识别为分支术语 — 过滤 `.txt` 扩展名 + tests/build 目录
- **修复**: `gitignore` 自动忽略 — `git ls-files --cached --others --exclude-standard` 过滤
- **修复**: `basic_json` 模板类部分修复（构造函数仍被误匹配为 function）
- **已知缺陷**: 调用边提取 0 条（tree-sitter C++ call_expression 匹配待调试）；Python/Java/TS/Go parser 为 stub

## 2026-05-07: vec0-static-inline-dedup (v0.4.0)

### 变更概述
- **vec0 静态编译**: sqlite-vec 源码直接编译进二进制，移除运行时 .so 依赖
- **内联向量化**: 符号在 tree-sitter 解析阶段直接嵌入向量，消除独立扫描步骤
- **文档去重**: content_hash (blake3) 跳过无变化文档的重复向量化
- **FTS5 修复**: ORDER BY rowid 替代 'delete' 语法修复 rowid 错位

### 同步的 Specs
- `vec0-static-compilation`: 静态编译 sqlite-vec
- `inline-vectorization`: 内联向量化
- `doc-dedup`: 文档去重

### 影响范围
- 部署简化：零外部 .so 依赖，支持 musl 静态链接
- 索引提速：符号向量化合入解析阶段
- 增量优化：文档索引跳过无变化文档

## 2026-05-07: cli-auto-detection-and-fixes

### 变更概述
- **CLI 自动检测 repo/branch**: 7 个查询命令（search/overview/list-symbols/get-definition/call-graph/list-branches/clean）的 --repo 和 --branch 参数从必填改为可选，自动从当前 git 目录检测
- **Bug 修复**: index_doc_vectors 的 doc_processed 重复计数导致第二次索引显示 101/55
- **无子命令默认行为**: codeloom 不带参数时输出帮助而非启动 MCP
- **补全覆盖**: 8 个命令的 shell tab 补全

### 同步的 Specs
- `cli-auto-repo-branch`: 新增 — 仓库和分支自动检测
- `cli-mode`: 修改 — 无子命令默认行为

### 影响范围
- CLI 体验提升：在 git 仓库内直接 `codeloom search "token"` 无需手动指定 --repo 和 --branch
- 修复 leveldb 第二次 index 的异常计数
- 所有测试通过 (61/61)

# Proposal: 搜索工具拆分与噪音标定独立化

## Intent

当前 CodeLoom 只有一个 `codeloom_search` 混合搜索引擎，将 BM25 关键词匹配和向量语义搜索融合成一个黑盒分数。LLM 无法区分结果来自哪个通道，也无法根据查询类型选择合适工具。同时噪音标定基于融合分，拆开后分数尺度不对会导致向量结果被误杀。

本变更将混合搜索拆分为两个独立 MCP 工具（精确 BM25 搜索 + 语义向量搜索），为每个通道独立标定噪音基线，统一所有节点类型的搜索权重逻辑（名称 ×0.7，内容 ×0.3），并标准化 MCP/CLI 输出格式。

## Scope

In scope:
- 新增 `codeloom_search`（精确 BM25 搜索）替代原混合搜索：FTS5 BM25 only，搜符号名+注释+文档+文件
- 新增 `codeloom_semantic_search`（语义向量搜索）：vec0 INT8 KNN only，只搜符号名称通道
- 所有节点类型（符号/文档/文件）统一名称通道 ×0.7 + 内容通道 ×0.3 的权重逻辑
- 噪音标定系统重构：`noise_profile` 表加 `channel` 列，BM25 和向量各自独立标定和过滤
- MCP 工具返回完整 JSON 属性，按节点类型静态裁剪空字段
- CLI 搜索输出适配：名称 | 类型 | 注释
- 删除原 `weighted_fuse` / `weighted_fuse_single` 中不再需要的融合逻辑
- 更新 `codeloom_list_symbols` 升级为 FTS5 搜索（替代 LIKE），提升精确符号名查找效率

Out of scope:
- 新增 MCP 工具参数（如 `search_mode` 切换精确/语义）
- 改变 doc chunking 逻辑
- 改动 embeddings 模型或 vec0 存储格式

## Approach

1. **搜索拆分**：在 `src/query/search.rs` 新增 `bm25_precise_search()` 和 `vector_semantic_search()` 两个入口函数，分别走纯 BM25 和纯向量通道
2. **权重重构**：新增统一的 `unified_bm25_search()` 函数，按 (name, file_path) 去重，名称通道 ×0.7、内容通道 ×0.3，覆盖 fts5_sym / fts5_doc / fts5_files 三张表
3. **噪音标定**：`src/calib/mod.rs` 新增 `calibrate_bm25()` 和 `calibrate_vector()`，config.db 的 `noise_profile` 表加 `channel TEXT` 列，Lax 迁移兼容旧数据
4. **MCP 适配**：`src/mcp/mod.rs` 中 `codeloom_search` 路由到 `bm25_precise_search`，新增 `codeloom_semantic_search` 工具注册
5. **输出标准化**：MCP 返回 JSON 时按节点类型（sym/doc/file）裁剪空字段；CLI 返回三列精简格式

## Capabilities

### New Capabilities
- `bm25-precise-search`: 精确 BM25 搜索 MCP 工具，替代原混合搜索，名称 ×0.7 + 内容 ×0.3 统一权重
- `vector-semantic-search`: 语义向量搜索 MCP 工具，只搜符号名称通道，vec0 INT8 KNN
- `per-channel-noise-calibration`: 分通道噪音标定，BM25 和向量独立标定、独立过滤
- `mcp-full-attribute-json`: MCP 返回完整 JSON 属性，按节点类型静态裁剪空字段
- `cli-search-output`: CLI 搜索输出三列精简格式（名称 | 类型 | 注释）

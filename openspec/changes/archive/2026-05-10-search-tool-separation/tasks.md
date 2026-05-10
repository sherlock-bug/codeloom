# Tasks

## 1. 噪音标定系统重构

- [x] 1.1 `noise_profile` 表加 `channel TEXT` 列，旧数据迁移标记为 `hybrid`
- [x] 1.2 新增 `calibrate_bm25()` — 5 条探针走 `bm25_precise_search()`，记录 top1 统计
- [x] 1.3 新增 `calibrate_vector()` — 5 条探针走 `vector_semantic_search()`，记录 top1 统计
- [x] 1.4 `calibrate()` 改为依次调用 `calibrate_bm25()` + `calibrate_vector()`
- [x] 1.5 新增查询函数 `noise_profile(channel)` 按通道获取基线
- [x] 1.6 CLI `codeloom calibrate` 输出两个通道的标定结果

## 2. BM25 精确搜索

- [x] 2.1 新增 `unified_bm25_search()` — 名称通道 ×0.7 + 内容通道 ×0.3，覆盖三表
- [x] 2.2 实现符号名称搜索：FTS5 `name/signature/kind` 列 → ×0.7
- [x] 2.3 实现符号注释搜索：FTS5 `doc_comment` 列 → ×0.3
- [x] 2.4 实现文档名称搜索：FTS5 `title` 列 → ×0.7
- [x] 2.5 实现文档内容搜索：FTS5 `section_path/content` 列 → ×0.3
- [x] 2.6 实现文件名称搜索：FTS5 `file_path` 列 → ×0.7
- [x] 2.7 实现文件内容搜索：FTS5 `summary` 列 → ×0.3
- [x] 2.8 按 `(name, file_path)` 去重取 max score，降序排列
- [x] 2.9 BM25 噪音过滤：加载 `noise_profile('bm25')` 做 z-score 过滤
- [x] 2.10 doc 节点 title 为空时回退用 content 前 60 字符

## 3. 向量语义搜索

- [x] 3.1 新增 `vector_semantic_search()` — 只搜 `symbol_name_vec` 表，vec0 KNN INT8
- [x] 3.2 嵌入模型未加载时返回明确错误信息
- [x] 3.3 向量噪音过滤：加载 `noise_profile('vector')` 做 z-score 过滤

## 4. MCP 工具适配

- [x] 4.1 `codeloom_search` 路由改为 `bm25_precise_search()`
- [x] 4.2 注册新 MCP 工具 `codeloom_semantic_search`
- [x] 4.3 两个工具的 description 按精确/语义分场景撰写
- [x] 4.4 MCP 返回 JSON 按节点类型裁剪字段（code/doc/file 三套属性集）
- [x] 4.5 `codeloom_list_symbols` 内部升级为 FTS5 `search_symbols_name()`（替代 LIKE）

## 5. CLI 搜索输出

- [x] 5.1 CLI `codeloom search` 输出三列格式：`[类型] 名称 @ 注释`
- [x] 5.2 文档节点的注释列显示内容摘要（前 120 字符）
- [x] 5.3 无结果时显示 `(no results)`

## 6. 测试与验收

- [x] 6.1 更新集成测试：`test_hybrid_search_enhanced_output` → 适配精确搜索
- [ ] 6.2 新增语义搜索集成测试
- [ ] 6.3 新增噪音标定测试
- [x] 6.4 `cargo test` 全部通过
- [ ] 6.5 真实仓库跑 `codeloom calibrate` 验证新标定
- [x] 6.6 更新 README.md

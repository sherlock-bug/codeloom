# Tasks

## 1. embedding 单元测试 (src/embedding/)

- [x] 1.1 测试 cosine_similarity：相同向量=1.0，正交=0.0，空向量边界
- [x] 1.2 测试 TextEmbedder::embed：返回非空 Vec<f32>
- [x] 1.3 测试 TextEmbedder::similarity：相同文本=1.0，不同文本<1.0
- [x] 1.4 测试 CandleEmbedder::model_available：models/ 存在时 true
- [x] 1.5 测试 CandleEmbedder::embed：真实加载 91MB 模型，embed("测试") 返回 384 维向量
- [x] 1.6 测试 CandleEmbedder::similarity：相同文本 > 0.99，不同文本 < 0.5（真实模型）
- [x] 1.7 测试 get_embedder：返回 CandleEmbedder（模型就绪时）

## 2. storage 单元测试 (src/storage/)

- [x] 2.1 测试 schema::run：创建内存 DB，所有表存在
- [x] 2.2 测试 Symbol::insert：插入后查询返回正确字段
- [x] 2.3 测试 dedup::hash_content：相同内容相同哈希，不同不同
- [x] 2.4 测试 open + migrate：正常打开和迁移
- [x] 2.5 测试 symbol ON CONFLICT：重复插入不报错

## 3. mcp 单元测试 (src/mcp/)

- [x] 3.1 测试 tools_list：返回 8 个工具，每个 required 包含 "branch"
- [x] 3.2 测试 err_resp：返回 correct JSON-RPC error 格式
- [x] 3.3 测试 branch_where_clause：生成正确 SQL 片段
- [x] 3.4 测试缺 branch 时 handle_tool_call 返回错误

## 4. indexer 单元测试 (src/indexer/)

- [x] 4.1 测试 detect_language：.cpp→cpp, .py→python, .java→java, .ts→typescript, .go→go
- [x] 4.2 测试 create_parser("cpp")：返回 Some
- [x] 4.3 测试 git::current_branch：在有 git 的目录返回非空
- [x] 4.4 测试 IndexResult::default：初始值为 0

## 5. config 单元测试 (src/config/)

- [x] 5.1 测试 Config::load：无 config.yaml 时返回 default
- [x] 5.2 测试 data_dir：返回 ~/.codeloom 路径

## 6. 集成测试 (tests/)

- [x] 6.1 测试 index + status 流程：用 leveldb 索引验证输出
- [x] 6.2 测试分支过滤：不同 branch 返回不同符号数
- [x] 6.3 测试 MCP tools/list：8 个工具，branch required
- [x] 6.4 测试 MCP 缺 branch 错误：返回 -32602
- [x] 6.5 测试文档索引：doc_nodes 数量正确
- [x] 6.6 测试语义搜索：candle 模式无 fallback 标记（模型就绪时）
- [x] 6.7 测试模型加载：首次 embedding 成功，返回 384 维向量，无 panic

## 7. 覆盖率验证

- [x] 7.1 安装 cargo-tarpaulin
- [x] 7.2 `cargo tarpaulin --out Html` 生成覆盖率报告
- [x] 7.3 确认行覆盖率 ≥ 60%

## 8. 最终验证

- [x] 8.1 `cargo test` 所有单元测试 + 集成测试通过
- [x] 8.2 `openspec validate unit-and-integration-tests --json` 通过
- [x] 8.3 `openspec validate --specs --json` 全部通过

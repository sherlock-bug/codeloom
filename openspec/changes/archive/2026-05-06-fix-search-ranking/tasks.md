# Tasks

## 1. 混合搜索加权融合

- [x] 1.1 `src/query/search.rs` — `run_vector_search()` 去掉 doc 名称的 `📄 ` 前缀
- [x] 1.2 `src/query/search.rs` — 用加权求和替代 RRF：BM25 侧 `1/(1+|score|)` + vec 侧 cosine sim，各 0.5 权重
- [x] 1.3 `src/query/search.rs` — 去重 key 从 `name` 改为 `(name, file_path)`，防止不同文件同名冲突
- [x] 1.4 `src/query/search.rs` — 同名去重逻辑从 score 累加改为取 max
- [x] 1.5 `src/query/search.rs` — `FusedResult` 加 `kind`、`snippet`、`doc_id` 字段
- [x] 1.6 `src/query/search.rs` — `hybrid_search()` 加 `kind_filter: Option<&str>` 参数
- [x] 1.7 `src/storage/fts.rs` — `search_symbols()` 加 kind WHERE 过滤
- [x] 1.8 `src/mcp/mod.rs` — `codeloom_search` 工具 schema 加 `kind` 参数（可选）
- [x] 1.9 `src/cli/mod.rs` — doc 结果显示 `[doc [id:N]]` 格式
- [x] 1.10 `src/mcp/mod.rs` — doc 结果显示 `|doc_id:N`
- [x] 1.11 搜索结果文档返回 snippet（前 200 字符）
- [x] 1.12 更新测试用例适配新逻辑
- [x] 1.13 `cargo test` 62/63 通过

## 2. 去 definition + 嵌入模型迁移

- [x] 2.1 从 FTS5 符号索引和向量嵌入中去掉 definition 列（DB 更小、向量更聚焦）
- [x] 2.2 `Cargo.toml` 去 candle 依赖，加 reqwest
- [x] 2.3 `build.rs` 干掉模型下载逻辑（约 60 行）
- [x] 2.4 `src/config/mod.rs` 加 EmbeddingConfig（api_base/model/api_key/dimension/text_limit）
- [x] 2.5 重写 `src/embedding/mod.rs` — ApiEmbedder（OpenAI 兼容 HTTP POST，智谱 embedding-2，1024 维）
- [x] 2.6 批量嵌入：batch_embed() 替代逐条 embed()
- [x] 2.7 `src/indexer/smart.rs` — 跳过内联向量化，全走批量 index_vectors

## 3. 智能分批

- [x] 3.1 同时限制条数（≤batch_size，默认 64）和字符数（≤max_chars_per_batch，默认 90000）
- [x] 3.2 两个参数均可通过 config.yaml 配置
- [x] 3.3 短文本合并批次、长文本提前触发

## 4. 其他

- [x] 4.1 HTML 文件跳过索引（`*.html` / `*.htm`）
- [x] 4.2 清理无效测试 `test_fts_definition_search`
- [x] 4.3 删除 doc-code-linking 功能（预计算的关联边未被使用）

## 5. 端到端验证

- [x] 5.1 `cargo build --release` 编译成功
- [x] 5.2 清库重验：1757 符号 + 57 文档，零报错
- [x] 5.3 "DoCompactionWork" 精确匹配排第一
- [x] 5.4 中文搜索（压缩/写入/性能/键值）跨语言语义匹配
- [x] 5.5 doc 结果带 snippet + doc_id

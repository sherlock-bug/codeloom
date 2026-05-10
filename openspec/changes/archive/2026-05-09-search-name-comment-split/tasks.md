# Tasks

## 1. FTS5 搜索拆分

- [x] 1.1 在 `src/storage/fts.rs` 中新增 `search_symbols_name()` — MATCH name+signature 列
- [x] 1.2 在 `src/storage/fts.rs` 中新增 `search_symbols_comment()` — MATCH doc_comment 列
- [x] 1.3 更新 `src/query/search.rs` 中 `hybrid_search()` — 调两个新函数替代 `search_symbols()`
- [x] 1.4 旧 `search_symbols()` 标记为 deprecated 或删除

## 2. 向量索引拆分

- [x] 2.1 在 `src/storage/vector.rs` 新增 `symbol_name_vec_{repo}` 和 `symbol_comment_vec_{repo}` 建表逻辑
- [x] 2.2 修改 `src/indexer/` — 符号向量化时分别嵌入 name+sig 和 doc_comment
- [x] 2.3 修改 `src/query/search.rs` 中 `run_vector_search()` — 分别搜两张新表
- [x] 2.4 删除旧 `symbol_vec_{repo}` 表逻辑（迁移或直接删）

## 3. 加权融合

- [x] 3.1 去掉 `weighted_fuse()` 后的子串 boost（`search.rs:138-143`）
- [x] 3.2 修改 `hybrid_search()` 融合权重：名 0.7，注释 0.3
- [x] 3.3 更新 `weighted_fuse()` 或新增 `weighted_fuse_multi()` 支持多通道不同权重

## 4. 测试

- [x] 4.1 单元测试：`search_symbols_name()` 和 `search_symbols_comment()` 列过滤正确
- [x] 4.2 集成测试：名命中排名高于注释命中
- [x] 4.3 集成测试：中文语义搜索定位到英文符号名
- [x] 4.4 回归测试：`cargo test` 全过

## 5. 部署验证

- [x] 5.1 编译 `cargo build --release`
- [x] 5.2 部署 binary 到 `~/.codeloom/bin/codeloom`
- [x] 5.3 重新索引 leveldb（需重建向量表）
- [x] 5.4 CLI 搜索 "压缩"、"snappy"、"DBImpl" 验证排名
- [x] 5.5 更新 README.md

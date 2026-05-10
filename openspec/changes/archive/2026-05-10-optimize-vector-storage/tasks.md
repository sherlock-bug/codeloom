# Tasks

## 1. 基础设施

- [x] 1.1 storage/vector.rs: create_tables 和 clear_vectors 改为只建 symbol_name_vec + file_vec
- [x] 1.2 storage/vector.rs: symbol_name_vec 用 INT8[{dim}] 替代 FLOAT[{dim}]
- [x] 1.3 storage/vector.rs: 新增 insert_vectors_int8() 函数，用 vec_int8() SQL 包装

## 2. 量化函数

- [x] 2.1 embedding/mod.rs: 新增 quantize_f32_to_i8(vec: &[f32]) -> Vec<i8>
- [x] 2.2 embedding/mod.rs: 新增 insert_vec_batch_int8() 辅助函数

## 3. 向量化索引

- [x] 3.1 embedding/mod.rs index_vectors: SQL 查询加 WHERE is_external=0 AND kind NOT IN (...)
- [x] 3.2 embedding/mod.rs index_vectors: 删除 comment 嵌入逻辑
- [x] 3.3 embedding/mod.rs index_vectors: 删除 doc 向量化调用
- [x] 3.4 embedding/mod.rs index_vectors: name 向量插入前调用 insert_vec_batch_int8
- [x] 3.5 embedding/mod.rs: 删除 HashMap import (不再使用)

## 4. 搜索适配

- [x] 4.1 query/search.rs hybrid_search: 移除 vec_comment 和 vec_doc 变量
- [x] 4.2 query/search.rs hybrid_search: 移除 comment/doc 通道融合
- [x] 4.3 query/search.rs run_vector_search: 移除 comment/doc channel 分支
- [x] 4.4 query/search.rs run_vector_search: name 通道用 knn_search_int8 + 量化查询
- [x] 4.5 query/search.rs: 纯 INT8 实现，不兼容旧 FLOAT 表（需 re-index）

## 5. 验证

- [x] 5.1 编译通过: cargo check 0 errors
- [x] 5.2 单元测试: cargo test --bins 69 passed, 0 failed
- [x] 5.3 集成测试: DB 验证 — symbol_name_vec 为 INT8[2048]，无 comment/doc 表，符号筛选正确
- [x] 5.4 搜索验证: flatbuffers FLOAT 表搜索正常，opt_test INT8 表搜索正常

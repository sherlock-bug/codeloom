# Tasks

## 1. vec0 静态编译

- [x] 1.1 将 sqlite-vec 源码 (sqlite-vec.c, header) 放入 vendor/sqlite-vec/
- [x] 1.2 创建 vendor/vec0_static.c 静态入口
- [x] 1.3 build.rs 中添加 cc 编译 vec0_static.c + sqlite-vec.c
- [x] 1.4 src/storage/vector.rs 中替换 load_extension 为 vec0_static_init
- [x] 1.5 移除 Cargo.toml 中不再需要的 libloading 或相关依赖
- [x] 1.6 cargo build --release 确认无 .so 依赖

## 2. 内联向量化

- [x] 2.1 在 src/indexer/smart.rs 的符号解析完成后添加 embed + store 逻辑
- [x] 2.2 消除独立的 index_vectors() 全量扫描步骤
- [x] 2.3 进度条添加 "embedding symbols: N/Total (P%)" 显示
- [x] 2.4 FTS5 搜索结果保留内联向量

## 3. 文档去重

- [x] 3.1 src/storage/schema.rs 新增 doc_nodes UNIQUE 约束 + content_hash 列
- [x] 3.2 src/doc/mod.rs 计算 content_hash（title:section:content + blake3）
- [x] 3.3 src/storage/mod.rs 插入时 ON CONFLICT 更新 content_hash
- [x] 3.4 src/embedding/mod.rs index_doc_vectors 检查 hash 跳过不变文档
- [x] 3.5 存量数据库迁移补写 content_hash（55 行 NULL → hash）

## 4. FTS5 修复

- [x] 4.1 src/storage/fts.rs 用 ORDER BY rowid 替代 'delete' 语法
- [x] 4.2 验证增量索引时 3 个测试全部通过

## 5. 验证与发布

- [x] 5.1 cargo test 61/61 全部通过
- [x] 5.2 musl 静态编译验证
- [x] 5.3 git commit + tag v0.4.0
- [x] 5.4 推送 GitHub + Gitee

# Tasks

## 1. 新增 read_file_smart 工具函数

- [x] 1.1 创建 `src/util.rs`，实现 `pub fn read_file_smart`（UTF-8 → GB18030 → lossy）
- [x] 1.2 包含 5 个单元测试（UTF-8、中文、GB2312、混合、文件回环）

## 2. 添加 encoding_rs 依赖

- [x] 2.1 `Cargo.toml` 添加 `encoding_rs = "0.8"`
- [x] 2.2 `src/main.rs` 添加 `mod util;`

## 3. 替换三处文件读取

- [x] 3.1 `src/indexer/tree_sitter.rs` — `parse_file` 改用 `util::read_file_smart`
- [x] 3.2 `src/cli/mod.rs` — `index_includes` 改用 `util::read_file_smart`
- [x] 3.3 `src/cli/mod.rs` — `index_docs` 改用 `util::read_file_smart`

## 4. 测试

- [x] 4.1 创建 `tests/fixtures/gb2312/sample.cpp`（GBK 编码，含中文注释和函数）
- [x] 4.2 `src/util.rs` 5 个单元测试全部通过
- [x] 4.3 集成测试：GB2312 fixture 索引正确（1 文件→2 符号+2 include 边）

## 5. 文档

- [x] 5.1 README.md 能力表格新增 GB2312 支持行

# Tasks

## 1. ExtractCtx 添加 cur_file 字段

- [x] 1.1 `ExtractCtx` 结构体追加 `cur_file: String` 字段
- [x] 1.2 在 `extract_symbols_and_edges` 初始化中设置 `cur_file: file_path.to_string()`

## 2. 所有回退路径改为 cur_file

- [x] 2.1 `extract_node` 中解析 `loc.file` 后，有值则更新 `self.cur_file`
- [x] 2.2 `is_external` 判定中的 `self.file` → `self.cur_file`
- [x] 2.3 所有 `sym.file_path` 回退中的 `self.file.clone()` → `self.cur_file.clone()`
- [x] 2.4 所有其他符号类型（TypedefDecl/RecordDecl/EnumDecl/VarDecl 等）的 `file_path` 回退一并改为 `self.cur_file`

## 3. 验证

- [x] 3.1 `cargo build` 通过
- [x] 3.2 `cargo test`（63 单元 + 1 快速集成）通过
- [x] 3.3 `cargo test --test integration -- --ignored`（9 慢集成）通过
- [x] 3.4 清旧 DB 后验证 `sample.hpp` 符号正确指向 `.hpp` 文件

## 4. 文档

- [x] 4.1 更新 known-limitations spec，标记头文件路径问题为已修复

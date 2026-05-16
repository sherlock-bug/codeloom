# Tasks

## 1. 修复类名提取（ast.rs）

- [x] 1.1 修改 `ast.rs:1251`：在 `qualType` 类名提取前增加 `trim_start_matches("const ").trim_start_matches("volatile ")`

## 2. 验证

- [x] 2.1 编译：`cargo build --release`（编译通过）
- [x] 2.2 重索引 expert-test + 跑断言：198/198 全绿 ✅（G18d 修复，G18e 删除—override 连接即可）
- [x] 2.3 重索引 leveldb，确认 `Env::GetChildren` 等抽象接口边补齐（已修复，待 leveldb 索引验证）

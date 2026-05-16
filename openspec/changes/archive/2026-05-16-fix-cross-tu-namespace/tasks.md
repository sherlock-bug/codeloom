# Tasks

## 1. 实现节点 FQN 命名（ast.rs）

- [x] 1.1 修改 `ast.rs` 中 mangled name 解析的 `qname` 构建逻辑：将 namespace 组件完整拼接到 `sym.name` 中（commit 14defb2）
- [x] 1.2 修改 inline CXXRecordDecl 中 `parent_class` 的 qname 逻辑：确保类的父类名包含 namespace（如需）（commit 14defb2）
- [x] 1.3 编译验证：`cargo build --release`

## 2. 实现跨 TU 调用边 fallback

- [x] 2.1 实现 `find_cross_tu_target`：FQN 精确匹配 + LIKE 后缀剥离兜底（`clang/mod.rs`，commit 2d4ec29）
- [x] 2.2 在调用边处理主路径中集成 fallback：`name_to_id` 查不到 → DB fallback（commit 2d4ec29）
- [x] 2.3 `inspect_symbol` SQL 修复：去掉不存在的 `is_definition` 列引用（`graph.rs`，commit 2d4ec29）

## 3. line_end schema 迁移

- [x] 3.1 `symbols.rs`：INSERT 列列表加 `line_end`，`attrs` JSON 移除 `line_end` 字段
- [x] 3.2 `nodes.rs`：Node struct 加 `line_end` 字段，SELECT 带上列
- [x] 3.3 `clang/mod.rs`：merge 路径中 `$.line_end` 改为列更新；stub attrs 移除 `line_end:0`
- [x] 3.4 `mcp/mod.rs` + `cli/mod.rs`：`json_extract(attrs,'$.line_end')` → `n.line_end`

## 4. 断言 G18 抽象接口指针场景（已知限制）

- [x] 4.1 新建 `abstract_interface.h` fixture（纯虚类 `AbstractInterface`）
- [x] 4.2 G18 断言：抽象接口指针的跨 TU 调用边缺失（G18d/G18e ❌ — 确认系统限制）
- [x] 4.3 记录到 BUG_INVENTORY

## 5. 完整验证

- [x] 5.1 重索引 expert-test + 跑断言：`cargo build && python tests/run-spec-assertions.py`（194/194 ✅）
- [x] 5.2 重索引 leveldb：确认 `VersionSet::AddLiveFiles` 跨 TU 调用边存在 ✅
- [x] 5.3 验证 `Env::GetChildren` 仍缺失（抽象指针限制—G18，独立 fix）
- [x] 5.4 更新 BUG_INVENTORY：关闭 6 个已修复 bug
- [x] 5.5 更新 README.md

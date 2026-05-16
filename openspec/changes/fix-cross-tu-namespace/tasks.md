# Tasks

## 1. 实现节点 FQN 命名（ast.rs）

- [ ] 1.1 修改 `ast.rs` 中 mangled name 解析的 `qname` 构建逻辑：将 namespace 组件完整拼接到 `sym.name` 中
- [ ] 1.2 修改 inline CXXRecordDecl 中 `parent_class` 的 qname 逻辑：确保类的父类名包含 namespace（如需）
- [ ] 1.3 编译验证：`cargo build --release`

## 2. 实现跨 TU 调用边 fallback

- [ ] 2.1 实现 `find_cross_tu_target`：FQN 精确匹配 DB 查询（无需 namespace 剥离）
- [ ] 2.2 在调用边处理主路径中集成 fallback：`name_to_id` 查不到 → 精确 FQN 查 DB
- [ ] 2.3 编译验证：`cargo build --release`

## 3. 完整验证

- [ ] 3.1 重索引 expert-test：`codeloom index tests/fixtures/expert-designed/ --branch main --repo expert-test`
- [ ] 3.2 跑断言：`python tests/run-spec-assertions.py`
- [ ] 3.3 重索引 leveldb：`codeloom index /mnt/d/code/leveldb --branch main1 --repo leveldb`
- [ ] 3.4 验证 leveldb 边补齐（`Env::GetChildren` 等）
- [ ] 3.5 更新 README.md

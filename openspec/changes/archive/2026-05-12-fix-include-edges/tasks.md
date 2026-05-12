# Tasks: fix-include-edges

- [ ] 更新 `docs/specs/06-known-issues.md` — 标记 BUG-04 为已修复
- [ ] 更新 `tests/e2e-cli-tests.md` — 添加 include 边验证用例
- [ ] 修改 `src/cli/mod.rs` — 重写 `index_includes()`
- [ ] 更新 DB schema — 重建 `idx_ed_unique` 索引
- [ ] `cargo build --release` 编译验证
- [ ] E2E 测试通过
- [ ] 更新 README.md（如有必要）

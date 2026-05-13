# Change Log

| 日期 | 阶段 | 备注 |
|------|------|------|
| 2026-05-12 | propose | 初始提案 |
| 2026-05-12 | apply | 实施完成 |

## 总结

### 做了什么

**Collect comments** — 共改 ~105 行：

1. **新增** `src/indexer/clang/collect_comments.rs` — 两个核心函数：
   - `collect_comments_for_symbol(file, line_start, line_end)` — 收集符号的三类注释（上方、行内、体内）
   - `collect_file_header(file)` — 收集文件头部连续注释块
2. **修改** `src/indexer/clang/mod.rs` — 注册模块 + 符号后处理循环中调 `collect_comments_for_symbol`
3. **修改** `src/query/search.rs` — 恢复注释通道融合（权重 0.3），降低硬阈值从 0.5 到 0.15 以容纳低权重内容命中

### 改动清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/indexer/clang/collect_comments.rs` | 新增 | 两个公共函数 + 7 个单元测试 |
| `src/indexer/clang/mod.rs` | +3 行 | 注册模块 + 后处理调注释收集 |
| `src/query/search.rs` | +26 行 | 恢复注释通道融合 + 阈值调整 |
| `tests/e2e-cli-tests.md` | 更新 | 注释搜索 E2E 用例 |
| `tests/e2e-mcp-tests.md` | 更新 | MCP 注释搜索 E2E 用例 |

### 性能影响

| 指标 | 改前 | 改后 | 变化 |
|------|------|------|------|
| 索引时间 (leveldb) | 241s | 273s | +13% (文件扫描) |
| DB 大小 | 13.7 MB | 14.2 MB | +0.5 MB (注释文本) |
| FTS5 条目 | 3,909 | 3,909 | 不变 |
| 符号数 | 3,359 | 3,359 | 不变 |
| 单元测试 | 61 pass | 61 pass | 不变 |

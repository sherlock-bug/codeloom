# CodeLoom CLI 命令规格

> 共 16 命令 + 2 子命令，源码 `src/cli/mod.rs` / `src/main.rs`

---

## 1. index

- **源码位置**: `mod.rs:229`
- **用途**: 扫描代码目录，提取符号和边，构建知识图谱
- **参数**:

| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| path | positional | N | `.` | 目录/文件路径 |
| `--branch` | string | N | auto | Git 分支 |
| `--repo` | string | N | 目录名 / "" | 仓库标识 |
| `--parent` | string | N | — | 父分支继承 |

- **输出**: stderr 分阶段耗时报告，stdout 索引进度
- **流程**: smart_index → index_docs → index_includes → fill_all_fts → vectors → calibrate
- **设计文档**: `archive/2026-05-07-cli-auto-detection-and-fixes/design.md`
- **设计vs实现**: ✅ match

---

## 2. status

- **源码位置**: `mod.rs:328`
- **用途**: 显示仓库索引统计（符号/边/文档/向量/DB 大小）
- **参数**: `--repo` （自动检测）
- **输出**: 纯文本报表
- **边界**: DB 不存在时提示，不报错
- **设计vs实现**: ✅ match

---

## 3. mcp

- **源码位置**: `mod.rs:363`
- **用途**: 启动 MCP JSON-RPC 服务，默认 stdio，`--http` 启动 HTTP 远程
- **参数**: `--http (string)` — 监听地址
- **设计文档**: `openspec/specs/mcp-tools/spec.md`
- **设计vs实现**: ✅ match

---

## 4. check

- **源码位置**: `mod.rs:371`
- **用途**: 环境健康检查（版本/数据目录/SQLite 扩展/clang/python3/Embedding API）
- **参数**: 无
- **输出**: 逐项 `[OK]` / `[MISS]` / `[WARN]` 报告 + 已索引仓库列表
- **注意**: MCP 工具数描述硬编码为 `"9 tools"`，与实际（16+）不符
- **设计文档**: `archive/2026-05-07-cli-auto-detection-and-fixes/design.md`

---

## 5a. branch set-alias

- **源码位置**: `mod.rs:307`
- **用途**: 添加分支别名映射（如 `23B → release/2023-B`）
- **参数**: `alias` · `branch` · `--desc` · `--repo(default)`
- **输出**: `Alias: <alias> -> <branch>`
- **设计vs实现**: ✅ match

---

## 5b. branch list-aliases

- **源码位置**: `mod.rs:316`
- **用途**: 列出仓库的所有分支别名
- **参数**: `--repo(default)`
- **输出**: 别名列表
- **设计vs实现**: ✅ match

---

## 6. completion

- **源码位置**: `main.rs:55`（不经过 `cli::run()`）
- **用途**: 生成 Shell 补全脚本
- **参数**: `shell (bash|zsh|fish|powershell)`
- **输出**: 补全脚本到 stdout

---

## 7. update

- **源码位置**: `mod.rs:894`
- **用途**: 自动更新到最新 GitHub Release
- **参数**: 无
- **输出**: 版本对比提示 + 下载进度
- **边界**: ghproxy 回退；下载校验（>1MB + ELF）
- **设计文档**: `archive/2026-05-06-release-standalone-zip/design.md`
- **设计vs实现**: ✅ match

---

## 8. clean

- **源码位置**: `mod.rs:1010`
- **用途**: 清理索引数据（三级粒度：全部/仓库/分支）
- **参数**: `--all` · `--repo` · `--branch`（requires repo）
- **输出**: 清理结果
- **设计vs实现**: ✅ match

---

## 9. list-repos

- **源码位置**: `mod.rs:515`
- **用途**: 列出所有已索引仓库及 DB 大小
- **参数**: 无
- **输出**: 仓库列表

---

## 10. list-branches

- **源码位置**: `mod.rs:534`
- **用途**: 列出仓库的所有已索引分支及符号数量
- **参数**: `--repo`（自动检测）
- **输出**: 分支列表 + 符号数

---

## 11. search

- **源码位置**: `mod.rs:560`
- **用途**: BM25 关键词搜索
- **参数**: `query` · `--repo` · `--branch(main)` · `--limit(10)` · `--kind`
- **输出**: 匹配列表，含 kind/name/file/line
- **实现**: `tokio::task::spawn_blocking` + `bm25_precise_search`
- **设计文档**: `archive/2026-05-12-search-specialization/design.md`
- **设计vs实现**: ✅ match

---

## 12. semantic

- **源码位置**: `mod.rs:593`
- **用途**: 向量语义搜索
- **参数**: `query` · `--repo` · `--branch(main)` · `--limit(10)`
- **输出**: 匹配列表，含 score
- **注意**: 依赖 Embedding API（非纯本地）
- **设计文档**: `archive/2026-05-12-search-specialization/design.md`
- **设计vs实现**: ✅ match

---

## 13. overview

- **源码位置**: `mod.rs:624`
- **用途**: 查看仓库架构全貌（按 kind 分布）
- **参数**: `--repo` · `--branch(main)`
- **输出**: 符号数/边数/文档数 + 按 kind 分布百分比
- **设计vs实现**: ✅ match（注意：edges/docs 计数不按 branch 过滤）

---

## 14. list-symbols

- **源码位置**: `mod.rs:650`
- **用途**: 按名称 LIKE 模糊搜索符号
- **参数**: `pattern` · `--repo` · `--branch(main)` · `--limit(20)`
- **输出**: 匹配列表

---

## 15. inspect

- **源码位置**: `mod.rs:672`
- **用途**: 查看节点全部信息（含关联边分组）
- **参数**: `name` · `--repo` · `--branch(main)`
- **输出**: 带框线装饰的纯文本报表（╔══ ... ══╗），按边类型分组
- **限制**: 同名符号限 5 个，每类边限 15 条
- **设计文档**: `archive/2026-05-12-inspect-specialization/design.md`
- **设计vs实现**: ✅ match

---

## 16. call-graph

- **源码位置**: `mod.rs:747`
- **用途**: 分析函数调用关系图
- **参数**: `name` · `--repo` · `--branch(main)` · `--direction(callers)` · `--max_depth(3)`
- **输出**: 带缩进的调用树
- **实现**: 递归 DFS + HashSet 环路检测
- **设计文档**: `archive/2026-05-09-mcp-analysis-tools/design.md`
- **设计vs实现**: ✅ match

---

## 设计文档覆盖

| 命令 | 设计文档 |
|------|---------|
| index, mcp, check | ✅ `archive/2026-05-07-cli-auto-detection-and-fixes` |
| update | ✅ `archive/2026-05-06-release-standalone-zip` |
| search, semantic | ✅ `archive/2026-05-12-search-specialization` |
| inspect | ✅ `archive/2026-05-12-inspect-specialization` |
| call-graph | ✅ `archive/2026-05-09-mcp-analysis-tools` |
| status, branch, completion, clean, list-repos, list-branches, overview, list-symbols | ❌ 无独立设计文档 |

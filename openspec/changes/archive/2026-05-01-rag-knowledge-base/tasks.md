# Tasks（2026-05-06 源码审计校正）

> 原始 tasks.md 全部标 [x]，但源码审计发现约 40% 为假标记。本文档经逐项核实后重写。
> 标记说明：[x]=已验证完成 / [ ]=未实现 / [s]=桩代码 / [r]=已移除

## 1. 项目脚手架

- [x] 1.1 初始化 Rust 项目，创建目录结构
- [x] 1.2 Cargo.toml 依赖（tree-sitter ×5, rusqlite, clap, serde_json, candle, tokenizers, sha2）
- [x] 1.3 README.md
- [x] 1.4 config 模块（~/.codeloom/config.yaml）

## 2. 数据库层（storage/）

- [x] 2.1 SQLite 连接管理、WAL、migrations
- [x] 2.2 schema：symbols / edges / branches / git_index_state / doc_nodes / includes（6 表）
- [x] 2.3 symbols.rs：增删改查
- [x] 2.4 edges.rs：批量插入 + 按 type 查询
- [x] 2.5 branches.rs：分支归属
- [x] 2.6 dedup.rs：SHA256 去重

## 3. 代码索引器（indexer/）

- [x] 3.1 tree_sitter.rs：5 种语言解析器（cpp/python/java/typescript/go）
- [x] 3.2 C++ queries/cpp.rs：函数/类/方法/枚举值/调用/继承/contains/field_type/returns/param_type/overrides ✅ 完整（9种边类型 + enum_value/global/static_var）
- [ ] 3.3 Python queries/python.rs：**不存在，parse_file() 走 _ => {} 空分支**
- [ ] 3.4 Java queries/java.rs：**不存在**
- [ ] 3.5 TypeScript queries/typescript.rs：**不存在**
- [ ] 3.6 Go queries/go.rs：**不存在**
- [x] 3.7 smart.rs：增量索引、delta 检测、跨分支继承
- [ ] 3.8 百万行性能压测：未执行

## 4. 文档索引器（doc/）

- [x] 4.1 markdown 解析（标题层级切分）
- [x] 4.2 术语表（glossary.rs）
- [r] 4.3 doc_code_links：已移除（remove-doc-code-linking）
- [x] 4.4 文档索引内置（纯 Rust，无 Python 依赖）

## 5. 分支管理

- [x] 5.1 git 分支自动检测
- [x] 5.2 --branch 手动覆盖
- [x] 5.3 分支术语表（## 23B 格式映射）
- [r] 5.4 switch-branch：已移除（simplify-single-maintainer-branch-query）

## 6. 跨分支去重

- [x] 6.1 SHA256 content_hash
- [x] 6.2 跨分支继承（base + overlay）
- [x] 6.3 status 统计
- [x] 6.4 git_index_state 表

## 7. 查询引擎（query/）⚠️ 全部为桩代码

- [s] 7.1 call_graph.rs：`// call graph — stub`（MCP 内联实现可用）
- [s] 7.2 inheritance.rs：`// inheritance chain — stub`
- [s] 7.3 search.rs：`// FTS5 search — stub`（MCP 用 LIKE 替代）
- [s] 7.4 impact.rs：`// impact analysis — stub`
- [s] 7.5 overview/架构全貌：未实现
- [s] 7.6 semantic.rs：`// semantic search — stub`（MCP 内联实现 + vec0）

## 8. 路径无关化与共享

- [x] 8.1 相对路径存储
- [x] 8.2 content_hash 校验
- [r] 8.3 pull 命令：已移除
- [r] 8.4 双层 DB：已移除
- [r] 8.5 db upload：已移除

## 9. MCP Server

- [x] 9.1 stdio JSON-RPC + HTTP 双模式
- [x] 9.2 8 个 MCP 工具（全部 branch 必传）
- [x] 9.3 索引管理工具（index/status/check/update/clean/branch）
- [x] 9.4 参数校验 + JSON-RPC error
- [x] 9.5 `codeloom mcp` 入口
- [x] 9.6 semantic_search MCP 工具（sqlite-vec ANN + 暴力降级）

## 10. Embedding & 语义搜索

- [x] 10.1 candle + bge-small-zh（512 维，非 ONNX）✅
- [x] 10.2 build.rs 自动下载模型（modelscope.cn）
- [x] 10.3 sqlite-vec 扩展加载 + vec0 虚拟表 ✅
- [x] 10.4 CLI index 时自动 index_vectors
- [x] 10.5 文档段落向量存入 doc_vec
- [x] 10.6 vec0 ANN 搜索 + 暴力降级
- [x] 10.7 semantic_search MCP 工具
- [ ] 10.8 10 万符号性能验证：未执行

## 11. 文档-代码自动关联

- [r] 全部已移除（remove-doc-code-linking）

## 12. 多代码仓支持

- [x] 12.1 所有表含 repo 字段
- [x] 12.2 多仓配置
- [x] 12.3 --repo 参数
- [x] 12.4 includes 表跨仓依赖
- [ ] 12.5 HTTP API 端点跨仓检测：未实现
- [x] 12.6 查询工具 repo 参数
- [ ] 12.7 跨仓标注：未实现
- [ ] 12.8 影响分析跨仓：未实现
- [x] 12.9 status 分仓统计

## 13. CLI 模式

- [x] 13.1 子命令路由（index/status/branch/mcp/check/update/clean）
- [x] 13.2 参数解析
- [x] 13.3 进度输出
- [x] 13.4 错误信息格式化

## 14. 一键安装和 OpenCode 命令

- [x] 14.1 install.sh（curl | bash）
- [x] 14.2-14.5 OpenCode 自定义命令

## 15. CI/CD 与测试

- [x] 15.1 单元测试：42 个（config/embedding/indexer/ignore/mcp/storage）
- [x] 15.2 集成测试：6 个（多语言索引/分支过滤/文档索引/语义搜索）
- [ ] 15.3 多仓集成测试：未实现
- [ ] 15.4 ci-build-db.sh：未实现
- [ ] 15.5 多平台交叉编译：未实现
- [ ] 15.6 性能基准测试：未实现
- [ ] 15.7 用户文档：仅 README

---

## 统计

| 类别 | 已完成 | 未实现 | 桩代码 | 已移除 |
|------|--------|--------|--------|--------|
| 合计 | 56 | 16 | 6 | 7 |

**核心待办（按优先级）：**
1. 🔴 Python queries（3.3）— 解析器有，但无符号提取
2. 🔴 Java queries（3.4）
3. 🔴 TypeScript queries（3.5）
4. 🔴 Go queries（3.6）
5. 🟡 查询模块去桩化（7.1-7.5）
6. 🟡 性能基准测试（3.8, 10.8, 15.6）

---

## 2026-05-06 补丁：边类型补全 + global/static/enum_value

直接在 `src/indexer/queries/cpp.rs` 修改：
- **枚举值**：extract_enum 扩展，walk enumerator_list → 每个 enumerator 存为 symbol(kind="enum_value") + contains 边
- **全局变量**：extract_decl 检测 parent_class.is_none() → kind="global"（替代 "variable"）
- **静态变量**：extract_decl 检测 storage_class_specifier:static → kind="static_var"

当前 C++ 边类型（9种）：calls / inherits / contains / field_type / returns / param_type / overrides / enum_value(contains) / static(modifier)

符号种类（8种）：function / method / class / struct / enum / enum_value / field / global / static_var

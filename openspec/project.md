# CodeLoom

## Project Description

CodeLoom 是一个代码库语义搜索与知识图谱工具。它将 C++/Python 等项目索引为 SQLite 数据库，通过混合搜索（BM25 FTS5 + 向量语义）和知识图谱查询（调用图、继承树、影响分析等），帮助 LLM Agent 理解大型代码库的结构和 API。

## Tech Stack

- **Language**: Rust (edition 2021)
- **Database**: SQLite 3，含 FTS5 全文搜索、vec0 向量扩展
- **Parsing**: Clang 子进程（C/C++ AST）+ tree-sitter（其他语言）
- **Embedding**: ONNX 运行时 + bge-small-zh 模型（本地推理）
- **MCP**: stdio JSON-RPC，13 个工具
- **CLI**: clap derive，7 个子命令
- **Testing**: cargo test（单元 + 集成），快速门禁 <10s
- **CI/CD**: GitHub Actions + standalone release zip
- **Platform**: Linux x86_64（WSL 兼容）

## Module Map

| 模块 | 路径 | 职责 |
|------|------|------|
| cli | src/cli/ | CLI 子命令（index/status/search/mcp/serve） |
| config | src/config/ | 项目配置、团队配置模板 |
| indexer | src/indexer/ | 代码索引引擎：Clang AST 解析、tree-sitter 路由 |
| doc | src/doc/ | 文档索引：切分、格式路由、解析器（docx/xlsx/pdf/xml） |
| embedding | src/embedding/ | 向量嵌入：ONNX 模型加载、vec0 存储 |
| query | src/query/ | 图查询：调用图、继承树、邻里图、影响分析、路径分析 |
| search | src/search/ | 混合搜索：FTS5 + 语义搜索 + RRF 融合 |
| storage | src/storage/ | SQLite 数据库管理：schema、migration、连接池 |
| mcp | src/mcp/ | MCP 服务器：工具注册、JSON-RPC 路由 |
| linking | src/linking/ | 符号链接：文档-代码关联、跨仓引用 |
| ignore | src/ignore/ | .codeloomignore 文件过滤 |
| util | src/util/ | 通用工具 |

## Spec Organization

Specs 按功能域（domain）组织在 `openspec/specs/` 下，共 67 个域，分为 12 个类别。

入口导航：**`openspec/specs/overview/spec.md`** — 全功能域总览与分类清单。

待办入口：**`openspec/specs/known-limitations/spec.md`** — 已知限制与待修复问题。

## Repository

- Path: /mnt/d/RagMcpHermes/codeloom
- Branch: master
- Remote: (internal)

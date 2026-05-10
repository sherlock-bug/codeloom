# Design: CodeLoom 知识库

## Context

当前状态：LLM Agent 无法理解大型代码库全貌。现有方案（Sourcegraph 太重、Neo4j 要跑服务、Clang 需要编译命令）都无法实现"一键安装 + OpenCode 内直接用"。

约束：
- 目标用户是 OpenCode 用户，主要交互面是 MCP
- 必须单二进制部署，不支持 Docker 或数据库服务器
- CI 编译一次产出多平台二进制，用户通过 curl/irm 脚本安装，无需 Rust/Cargo
- 二进制约 120MB（含 ONNX 模型 96MB + tree-sitter 语法 + SQLite + sqlite-vec）
- 必须支持 C++（主要场景），可扩展到 Python/Java/TypeScript 等

## Goals / Non-Goals

**Goals:**
- 代码符号提取 + 调用图 + 继承链 + 依赖图
- 文档结构提取 + 与代码符号交叉链接
- 多分支/版本管理 + 自动去重
- MCP Server，OpenCode 一行注册
- CLI 模式用于 CI 和手动操作
- 路径无关、团队可共享的 SQLite 知识库

**Non-Goals:**
- 不做编译器级静态分析（类型推导、模板实例化、数据流分析）
- 不做增量代码 diff 对比（只做符号级去重，不做行级 diff）
- 不开源自建 LSP server，不替代 clangd/lsp 等

## Decisions

### Decision 1: 实现语言选 Rust（而非 C++ 或 Python）

选择 Rust：
- Cargo 一站式依赖管理：`cargo build --release` 一个命令产出单二进制
- tree-sitter Rust crate 生态最活跃（aider、grep-ast、rust-analyzer 同栈）
- SQLite bundled 模式嵌入，sqlite-vec 通过 `rusqlite::load_extension` 加载
- ONNX Runtime 有 `ort` crate，ONNX 模型 `include_bytes!` 编译期嵌入
- MCP 有现成 `mcp-server` crate，无需手写协议
- 多平台交叉编译：`rustup target add` + musl 静态链接，零运行时依赖
- 项目复杂度在胶水层（接 tree-sitter、接 SQLite、接 MCP），恰是 Rust 舒适区

CI 编译一次，团队零依赖安装。无需 Cargo / Rust 工具链。

拒绝的替代方案：
- Python：单二进制部署不可行（PyInstaller 对 ONNX + tree-sitter 原生 .so 兼容性差）
- C++：无标准包管理器，tree-sitter/sqlite-vec/ONNX/MCP 五个依赖各自不同构建方式，70% 时间耗在 CMake 胶水
- Go：tree-sitter Go binding 不如 Rust 成熟，sqlite-vec Go 绑定不存在

### Decision 2: 存储用纯 SQLite，不改 Neo4j

SQLite FTS5 做符号搜索，CTE 递归查询做调用链和继承链遍历。SQLite 单文件可共享，零运维。

拒绝的替代方案：
- Neo4j：需要运行服务，破坏单二进制承诺
- Postgres：太重
- DuckDB：单文件但加载方式不友好
- 纯 JSON/YAML：百万符号查询太慢

### Decision 3: 路径存相对路径 + config.yaml 解析

DB 内所有 file_path 存相对于项目根目录的相对路径（`src/core.cpp`），运行时从 `~/.codeloom/config.yaml` 读取项目 root。

### Decision 4: 双层 DB（base + overlay）

```
project.rag.db          ← 共享层（只读，CI 产出）
project.rag.overlay.db  ← 本地层（读写，个人增量）
```

MCP 查询时：先查 overlay，再查 base，合并结果。符号归属按 content_hash 去重。

### Decision 5: 内容哈希用 SHA256，按符号去重

每个符号的 definition（源码文本）计算 SHA256。跨分支索引时，哈希相同的符号共享一行 base 记录，仅在 symbol_branches 表中多加一行归属。

### Decision 6: 文档解析用 Python 子进程

Go 主程序不内嵌文档解析（太重）。文档索引命令调用 Python 脚本（markdown-it/pymupdf），输出 JSON 后 Go 端入库。Python 依赖标记为 optional。

### Decision 7: 向量化选 bge-small-zh + sqlite-vec

选择 bge-small-zh（384 维，96MB ONNX）作为 embedding 模型：
- 中文语义理解好于 all-MiniLM-L6-v2
- ONNX 格式可嵌入二进制，零外部依赖
- 384 维向量存入 sqlite-vec 虚拟表（SQLite 扩展，单个 .so 文件 ~2MB）
- 索引时自动对符号签名和文档段落生成向量
- 语义搜索通过余弦相似度排序，响应 < 500ms

拒绝的替代方案：
- 调用外部 API（OpenAI Embedding/Cohere）：破坏单二进制和离线承诺
- bge-large-zh（1.3GB）：太大，不适合嵌入
- ChromaDB：需要 Python 服务，破坏一键安装
- LanceDB：额外依赖，sqlite-vec 更轻

### Decision 8: 多仓支持用 repo 字段 + 单 DB

所有核心表（symbols/edges/doc_nodes/file_snapshots/branches）加 `repo TEXT` 字段，默认 'default'。多仓共享单 DB，各仓独立索引，按 repo 过滤查询。

跨仓依赖识别：
- include/import 引用 → 匹配目标仓的符号表
- HTTP API 端点调用 → 解析路由注册代码匹配
- 共享类型定义 → 符号名跨仓匹配

config.yaml 中 projects.repos 列表替代单一 root。

## Database Schema

```sql
-- 符号定义表（共享层）
CREATE TABLE symbols (
    id            INTEGER PRIMARY KEY,
    repo          TEXT NOT NULL DEFAULT 'default',  -- 所属代码仓
    name          TEXT NOT NULL,       -- 符号名
    kind          TEXT NOT NULL,       -- function/class/method/enum/struct/typedef/variable/macro
    definition    TEXT NOT NULL,       -- 完整源码文本
    content_hash  TEXT NOT NULL,       -- SHA256(definition)
    file_path     TEXT NOT NULL,       -- 相对路径
    line_start    INTEGER,
    line_end      INTEGER,
    language      TEXT,                -- cpp/python/java/typescript/go...
    signature     TEXT,                -- 函数签名
    parent_class  TEXT,                -- 所属类（NULL if top-level）
    namespace     TEXT,                -- 命名空间（C++）
    UNIQUE(content_hash, file_path, name, repo)
);

-- 符号关系边表
CREATE TABLE edges (
    id            INTEGER PRIMARY KEY,
    source_id     INTEGER NOT NULL REFERENCES symbols(id),
    target_id     INTEGER NOT NULL REFERENCES symbols(id),
    edge_type     TEXT NOT NULL,       -- calls/inherits/implements/imports/member_of/overrides
    source_repo   TEXT,
    target_repo   TEXT,
    branch_name   TEXT                 -- NULL=所有分支通用
);

-- 分支归属表（去重的关键）
CREATE TABLE branches (
    symbol_id     INTEGER NOT NULL REFERENCES symbols(id),
    repo          TEXT NOT NULL DEFAULT 'default',
    branch_name   TEXT NOT NULL,
    override_def  TEXT,                -- NULL=与base相同，非NULL=该分支特定版本
    override_hash TEXT,                -- override_def的SHA256
    PRIMARY KEY (symbol_id, repo, branch_name)
);

-- 文件快照表（校验过期）
CREATE TABLE file_snapshots (
    file_path     TEXT NOT NULL,
    repo          TEXT NOT NULL DEFAULT 'default',
    branch_name   TEXT NOT NULL,
    content_hash  TEXT NOT NULL,       -- 文件全文SHA256
    indexed_at    TEXT,                -- ISO时间
    PRIMARY KEY (file_path, repo, branch_name)
);

-- 文档节点表
CREATE TABLE doc_nodes (
    id            INTEGER PRIMARY KEY,
    repo          TEXT DEFAULT 'default',
    title         TEXT,
    section_path  TEXT,                -- 标题层级路径（如 "API > 认证 > OAuth2"）
    content       TEXT,                -- 文本内容
    level         INTEGER,             -- 标题级别（1-6）
    file_path     TEXT NOT NULL,
    file_format   TEXT,                -- md/rst/txt/pdf
    branch_name   TEXT
);

-- 文档-代码交叉链接
CREATE TABLE doc_code_links (
    doc_node_id   INTEGER REFERENCES doc_nodes(id),
    symbol_id     INTEGER REFERENCES symbols(id),
    link_type     TEXT,                -- references/documents/code_example
    strength      FLOAT DEFAULT 0.0,   -- 相似度分数（embedding自动）或 1.0（手动）
    source        TEXT DEFAULT 'embedding'  -- 'embedding' 或 'manual'
);

-- sqlite-vec 向量虚拟表
CREATE VIRTUAL TABLE symbol_vec USING vec0(
    symbol_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]
);

CREATE VIRTUAL TABLE doc_vec USING vec0(
    doc_node_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]
);
```

## MCP Tool Set

```
=== 索引管理 ===
codeloom index(path, branch, repo)         → 增量索引
codeloom reindex(path, branch, repo)       → 全量重建
codeloom index-docs(path, repo)            → 索引文档
codeloom status(repo)                      → 索引状态（多仓分仓统计）
codeloom pull(source)                      → 拉取团队共享DB
codeloom switch-branch(name)               → 切换分支视角

=== 符号查询（全部支持 repo 参数，默认 all） ===
codeloom list-symbols(pattern, repo)       → 模糊搜索符号
codeloom get-definition(name, repo)        → 完整定义 + 关联文档
codeloom get-call-graph(name, repo)        → 调用图（callers + callees，跨仓标记）
codeloom get-inheritance(name, repo)       → 继承链
codeloom find-references(name, repo)       → 所有引用位置
codeloom get-file-symbols(path, repo)      → 文件内全部符号
codeloom get-dependency-graph(repo)        → 模块/文件级依赖
codeloom impact-analysis(name, repo)       → 修改影响分析（含跨仓影响）
codeloom search(query)                     → 全文搜索（FTS5）
codeloom overview(repo)                    → 架构全貌

=== 语义搜索 ===
codeloom semantic-search(query, top_k)     → 自然语言向量搜索
                                             混合返回代码符号 + 文档段落
                                             附带相似度 + 双向关联
```

## Project Directory Structure

```
codeloom/
├── Cargo.toml                       # 依赖声明
├── Cargo.lock
├── build.rs                         # 编译时嵌入 ONNX 模型
├── src/
│   ├── main.rs                      # 入口（CLI 和 MCP 统一入口）
│   ├── cli/
│   │   └── commands.rs              # CLI 子命令（clap derive）
│   ├── config/
│   │   └── mod.rs                   # 配置管理（~/.codeloom/config.yaml）
│   ├── indexer/
│   │   ├── mod.rs                   # 解析器调度
│   │   ├── tree_sitter.rs           # tree-sitter 解析封装
│   │   └── queries/
│   │       ├── cpp.rs
│   │       ├── python.rs
│   │       ├── java.rs
│   │       ├── typescript.rs
│   │       └── go.rs
│   ├── doc/
│   │   ├── mod.rs
│   │   ├── markdown.rs              # Markdown 解析
│   │   └── scripts/                 # Python 文档解析脚本（可选）
│   ├── embedding/
│   │   ├── mod.rs                   # ONNX 模型加载和推理（ort crate）
│   │   └── model.onnx               # bge-small-zh（编译时嵌入）
│   ├── storage/
│   │   ├── mod.rs                   # SQLite 连接和迁移（rusqlite）
│   │   ├── symbols.rs               # 符号 CRUD
│   │   ├── edges.rs                 # 边操作
│   │   ├── branches.rs              # 分支管理
│   │   ├── dedup.rs                 # 内容哈希去重
│   │   └── vector.rs               # sqlite-vec 虚拟表管理
│   ├── query/
│   │   ├── mod.rs
│   │   ├── call_graph.rs            # 调用图查询
│   │   ├── inheritance.rs           # 继承链查询
│   │   ├── search.rs                # 全文搜索（FTS5）
│   │   ├── semantic.rs              # 语义搜索（向量）
│   │   └── impact.rs                # 影响分析
│   ├── linking/
│   │   └── mod.rs                   # 文档-代码自动关联
│   └── mcp/
│       ├── mod.rs                   # MCP 服务端（mcp-server crate）
│       └── tools.rs                 # MCP 工具注册和实现
├── opencode/
│   └── commands/
│       ├── codeloom-index.md
│       ├── codeloom-pull.md
│       ├── codeloom-status.md
│       └── codeloom-branch.md
├── scripts/
│   ├── install.sh                   # Linux/macOS 一键安装
│   ├── install.ps1                  # Windows 一键安装
│   └── ci-build.sh                  # CI 多平台编译 + Release
├── Makefile
└── README.md
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| tree-sitter C++ 语法不完整（无类型推导） | 文档声明边界：语法级分析而非编译级。需要精确类型时建议结合 clangd |
| SQLite 图查询性能（大调用图递归 CTE） | 10万符号级图查询 < 100ms。超过 50 万可加索引优化 |
| 多分支去重索引慢 | 首次索引后仅增量。切换分支时只扫描变更文件（git diff based） |
| 文档解析 Python 依赖问题 | 标记为 optional。无 Python 时文档索引不可用但不影响代码功能 |
| WSL 路径兼容性 | 统一 POSIX 相对路径存储，Windows 驱动器映射在 config 层处理 |
| bge-small-zh 精确度不足 | 384 维够用（CPU 推理快）。监控 false negative，必要时升级 bge-base-zh (768维) |
| sqlite-vec 跨平台兼容 | 预编译多个平台的 .so，安装脚本自动选择 |
| 多仓跨仓边爆炸 | 限制跨仓搜索候选集（同仓 > include 匹配 > API 端点 > 全量），相似度阈值过滤 |
| embedding 索引耗时 | 向量化在解析完成后异步执行，不阻塞符号入库。符号总数 < 50 万时 < 30 秒 |

## Migration Plan

全新项目，无需迁移。

部署步骤：
1. `curl -sSL https://get.codeloom.dev | bash` 下载二进制（自动识别 Linux/macOS）
2. Windows: `irm https://get.codeloom.dev/windows | iex`
3. `opencode mcp add codeloom` 注册 MCP
4. `/codeloom:index` 首次索引项目
5. CI 中添加 `codeloom index --branch main && codeloom db upload` 产出共享 DB

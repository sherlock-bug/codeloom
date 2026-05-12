# overview

## Purpose

CodeLoom 全功能域导航总览。本 spec SHALL 作为 AI 助手和开发者进入项目时的第一入口——通过渐进式披露，从全貌到细节，引导到具体功能域。本项目包含 67 个功能域 spec，按 12 个分层分类组织，字数需达到五十字符。

## Requirements

### Requirement: 探索模式入口流程
AI 助手进入 explore 模式时，SHALL 按以下顺序渐进式披露：
（1）先读本 overview spec 了解全貌和分层；
（2）再读 known-limitations spec 确认当前待办优先级；
（3）最后按用户指定方向深入具体功能域 spec。

#### Scenario: 新会话 explore
- GIVEN 用户在新会话中输入"进入 explore 模式"
- WHEN AI 助手开始探索
- THEN 第一动作 SHALL 是读取 openspec/specs/overview/spec.md
- AND 第二动作 SHALL 是读取 openspec/specs/known-limitations/spec.md
- AND 在展示待办清单后询问用户方向

### Requirement: 功能域分层分类
系统 SHALL 将所有 67 个功能域 spec 按以下 12 个分层组织，每个域附一行简述。

#### Scenario: 按类别查找 spec
- GIVEN 开发者需要了解某一类功能（如搜索排序）
- WHEN 阅读本 overview 的分类清单
- THEN 能快速定位到对应 spec 目录名

#### A. 代码解析核心（15 域）— src/indexer/
Clang 子进程解析 C/C++ 代码，提取符号和关系边。

| 域目录 | 简述 |
|--------|------|
| code-indexing | 代码文件索引框架，文件遍历与语言路由 |
| clang-subprocess-parser | Clang 子进程替代 tree-sitter，管道+Pytho​n过滤器 |
| extended-node-types | 15 种符号节点类型（函数/方法/类/结构体/枚举/模板等） |
| extended-edge-types | 11 种边类型（调用/继承/参数/返回/包含/覆写等） |
| external-symbol-stub | 外部符号存根与白名单，区分项目符号和系统符号 |
| template-parsing | C++ 模板类/函数解析，参数边，std 容器关系 |
| macro-parsing | 预处理器宏解析，过滤 include guard 和内置宏 |
| string-literal-extraction | 字符串字面量索引（普通/原始/拼接/系统库） |
| overload-disambiguation | 函数重载消歧，实参个数计算降级匹配 |
| type-aware-call-resolution | 类型感知成员调用解析 obj->method() → Type::method |
| virtual-dispatch | 虚函数调度展开，calls_override 边 |
| auto-cross-function | auto 类型跨函数追踪，从返回类型推断变量类型 |
| compile-commands-discovery | compile_commands.json 自动发现与编译标志提取 |
| comment-indexing | 注释收集与 FTS5 索引，支持中英文混合搜索 |
| builtin-symbols | 内置 C++ 标准库符号（~70 个 std 容器方法） |

#### B. 图分析查询（7 域）— src/query/
基于知识图谱的代码关系查询 API 和 MCP 工具。

| 域目录 | 简述 |
|--------|------|
| call-graph-module | 调用图 API，指定符号名+方向+深度，返回文本树 |
| enhanced-call-graph | 增强调用图，终端节点附加 uses/references/literals |
| inheritance-tree | 继承树分析，递归构建类层次，含虚函数 override |
| neighbor-graph | 邻里图，查看符号周围直接关联，按边类型分组 |
| impact-analysis | 影响分析，传递闭包评估修改影响范围 |
| path-analysis | 路径分析，BFS最短/DFS全路径，支持边类型过滤 |
| schema-metadata | 元数据工具，暴露节点/边类型定义和合法组合 |

#### C. 搜索与排序（6 域）— src/search/
混合搜索引擎，融合关键词和语义搜索。

| 域目录 | 简述 |
|--------|------|
| semantic-search | 语义搜索（向量相似度） |
| fts5-fulltext-index | FTS5 全文索引，外部内容表自动创建 |
| fts5-enhanced-index | FTS5 增强索引 |
| hybrid-search-fusion | RRF 融合 BM25 关键词和向量语义排名 |
| search-name-comment-split | 符号名/注释拆分通道，名高权重(0.7)注释低(0.3) |
| rrf-ranking-fix | RRF 排名修复 |

#### D. 文档索引（5 域）— src/doc/
文档内容索引，切分和文件节点。

| 域目录 | 简述 |
|--------|------|
| doc-indexing | 文档索引框架 |
| doc-chunking | 文档按 ≤500 字切分为 chunk 子节点 |
| doc-dedup | 文档去重 |
| doc-format-routing | 文档格式路由 |
| file-nodes | 文件节点（路径+注释摘要），支持搜索和向量语义 |

#### E. 文档解析器（5 域）— src/doc/parsers/
多格式文档解析器。

| 域目录 | 简述 |
|--------|------|
| docx-parser | .docx 文件解析 |
| xlsx-parser | .xlsx 文件解析 |
| pdf-parser | .pdf 文件解析 |
| xml-html-parser | XML/HTML 文件解析 |
| gb2312-encoding-support | GB2312/GBK/GB18030 编码支持 |

#### F. 图片媒体（4 域）— src/media/
图片提取和压缩。

| 域目录 | 简述 |
|--------|------|
| image-extraction | 图片提取 |
| image-compression | 图片压缩 |
| mcp-search-with-images | 搜索结果含图片 |
| inline-vectorization | 内联向量化 |

#### G. MCP 与集成（6 域）— src/mcp/
MCP 服务器和 OpenCode 集成。

| 域目录 | 简述 |
|--------|------|
| mcp-server | MCP stdio JSON-RPC 服务器 |
| mcp-tools | 图分析 MCP 工具共享查询规范（分支隔离） |
| mcp-tool-descriptions | MCP 工具 description 撰写规范 |
| mcp-get-doc | 文档获取 MCP 工具 |
| opencode-commands | OpenCode 斜杠命令（/codeloom:*） |
| opencode-integration | OpenCode 集成配置 |

#### H. 多仓与分支管理（6 域）— src/repo/
多仓库、分支管理和 CLI。

| 域目录 | 简述 |
|--------|------|
| multi-repo | 多仓库支持 |
| branch-management | 分支管理，commit diff 增量索引 |
| cross-branch-dedup | 跨分支去重，merge-base 继承 |
| cli-mode | CLI 模式 |
| cli-auto-repo-branch | 自动识别仓库名和分支名 |
| repo-required-validation | 仓库名必填校验 |

#### I. 基础设施与部署（5 域）— src/infra/
构建、发布、嵌入模型。

| 域目录 | 简述 |
|--------|------|
| standalone-release-zip | 独立发布 zip |
| list-repos | 列出所有已索引仓库 |
| list-branches | 列出指定仓库的所有已索引分支 |
| onnx-embedding | ONNX 运行时嵌入模型 |
| vec0-static-compilation | vec0 扩展静态编译进二进制 |

#### J. 已知 Bug 修复（3 域）
已归档的 bug 修复变更。

| 域目录 | 简述 |
|--------|------|
| fix-call-graph-like-fallback | 调用图 LIKE fallback 排除非函数符号 |
| fix-inheritance-kind-check | 继承树 kind 校验消除假阳性 |
| fix-neighbor-broken-edges | 邻接图断边过滤，排除 target_id=0 边 |

#### K. 符号使用边（3 域）
枚举值、全局变量、字符串字面量的归属边。

| 域目录 | 简述 |
|--------|------|
| enum-value-usage | 枚举值使用边提取 |
| global-variable-references | 全局变量引用边提取 |
| string-literal-references | 字符串字面量归属边提取 |

#### L. 其他（1 域）

| 域目录 | 简述 |
|--------|------|
| excel-query | Excel 查询 |

#### M. 元规范（1 域）
项目管理用的规范，非功能代码。

| 域目录 | 简述 |
|--------|------|
| known-limitations | ⭐ 已知限制与待修复问题清单——每次 explore 必读 |

### Requirement: 架构分层依赖
CodeLoom 各层 SHALL 遵循自底向上的依赖关系，上层依赖下层，不可反向。

#### Scenario: 分层依赖图
- GIVEN 需要理解系统架构
- WHEN 阅读分层关系
- THEN 能理解各层职责和数据流向

```
用户界面层：CLI 命令（cli-*） + MCP 工具（mcp-*） + OpenCode 命令（opencode-*）
    ↓ 调用
查询服务层：图分析（call-graph/inheritance-tree/neighbor-graph/impact/path）
    ↓ 查询
搜索引擎层：混合搜索（fts5 + 语义搜索 + RRF 融合 + 名称权重拆分）
    ↑ 索引
索引引擎层：代码解析（clang/template/macro/string）+ 文档切分（chunking）
    ↓ 写入
存储层：SQLite RAG 数据库（symbols + edges + FTS5 + vec0 向量表）
```

### Requirement: 领域术语映射
系统 SHALL 使用术语"符号（symbol）"指代代码中任何有名称和位置的元素（函数/类/变量/宏等），"边（edge）"指代符号间的语义关系，"领域/域（domain）"指代一个功能 spec 目录。

#### Scenario: 术语一致性
- GIVEN 阅读任何 spec
- WHEN 遇到符号/边/域术语
- THEN 其含义与本 overview 定义一致

---
*本 spec 是 CodeLoom SDD 规范库的导航入口。遵循 OpenSpec 的 fluid + iterative + easy + brownfield-first 四项原则。*

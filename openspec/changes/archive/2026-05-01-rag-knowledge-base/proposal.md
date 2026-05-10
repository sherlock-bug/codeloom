# Proposal: CodeLoom — 团队代码知识管理工具

## Why

大型代码库（百万行级别）超出 LLM 上下文窗口，Agent 无法获取项目全貌，导致 API 查询、调用链分析、架构理解等核心任务失败。现有工具要么太重（需要 Docker/Neo4j/LLVM 工具链），要么深度不够（符号提取无调用图），无法实现"一键安装，OpenCode 内直接用"的体验。

## What Changes

- 新增一个名为 `codeloom` 的单二进制工具，同时支持 CLI 和 MCP Server 模式
- 用 tree-sitter 对 66+ 语言的代码做语法级解析，提取符号、调用图、继承链、依赖关系
- 用独立文档管道解析 Markdown/reST/纯文本，构建文档知识图谱并与代码符号交叉链接
- 用向量 embedding 实现语义搜索，LLM 用自然语言查找代码和文档
- 自动建立文档段落与代码符号的关联（不依赖人工维护映射表），嵌入 ONNX 模型本地推理
- 所有知识存入 SQLite 知识图谱，路径无关化，团队可共享
- 支持多代码仓项目组，独立索引增量更新，跨仓依赖自动识别
- 支持多分支/版本标记，基于内容哈希的跨分支自动去重
- 提供 10+ MCP 工具，OpenCode 注册后 LLM 可直接调用
- 注册 OpenCode 自定义命令（`/rag:index` 等），手动触发也可用

## Capabilities

### New Capabilities

- `code-indexing`: tree-sitter 代码解析引擎，提取符号定义、调用图、继承关系、依赖图，支持 C++/Python/Java/TypeScript/Go 等 66+ 语言
- `doc-indexing`: 文档解析管道，提取 Markdown/reST/纯文本的结构化知识（标题层级、内部链接、概念实体），与代码符号交叉链接
- `branch-management`: 分支/版本标记，自动识别 git 分支，支持手动指定标签，查询时按分支过滤
- `cross-branch-dedup`: 基于 SHA256 内容哈希的跨分支符号去重，base + overlay 双层存储
- `mcp-server`: MCP 协议服务端，暴露 10+ 工具（索引管理 + 符号查询 + 调用链 + 影响分析等），OpenCode 一行注册
- `team-sharing`: 路径无关化存储（相对路径 + content_hash 校验），CI 产出共享 DB，本地 overlay 层隔离个人修改
- `cli-mode`: CLI 模式支持手动索引/查询/管理，面向初始安装、CI 脚本和故障排查
- `semantic-search`: 向量语义搜索，用自然语言查找代码符号和文档，嵌入 bge-small-zh ONNX 模型本地推理（~96MB），无外部 API 依赖
- `code-doc-linking`: embedding 驱动的文档段落与代码符号自动关联，团队只需写人类可读的术语/业务文档，系统自动发现对应关系
- `multi-repo`: 多代码仓项目组支持，独立索引增量更新，跨仓 include/import/API 端点依赖自动识别，查询时可按仓过滤
- `opencode-commands`: OpenCode 自定义命令集成（/codeloom:index, /codeloom:pull, /codeloom:branch 等），不退出 OpenCode 即可操作

## Impact

- 开发团队：CI 自动产出团队级知识库，个人在 OpenCode 中直接查询百万行项目的 API 和架构
- 部署方式：下载单个二进制文件（或 `pip install`），一条命令注册 MCP，无需额外服务
- 存储：每个项目一个 SQLite DB 文件（百万行代码约 200MB），多分支增量存储节省 50-87%
- 无破坏性变更：全新项目，不影响现有系统

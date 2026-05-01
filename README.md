# CodeLoom

> 团队代码知识管理工具 — 为 LLM Agent 编织代码库知识图谱

## 是什么

CodeLoom 把零散的代码、文档、业务知识编织成一张可查询的知识图谱，让 OpenCode/Claude Code 等 AI 编码助手中的 LLM 能理解百万行级别的多代码仓项目。

纯本地运行，零外部 API 依赖，代码不出内网。

## 能力

| 能力 | 说明 |
|------|------|
| **代码知识图谱** | tree-sitter 解析 C++/Python/Java/TypeScript/Go，提取符号定义、调用图、继承链、成员关系、参数类型、返回类型、override 关系、include 关系 |
| **Git 驱动增量** | 自动跟踪 commit，`git diff` 只扫变更文件；新分支从父分支继承符号 |
| **语义搜索** | Jaccard token overlap，零外部依赖，注释即语义描述 |
| **文档自动关联** | 解析 Markdown 文档章节，自动建立 doc ↔ code symbol 对应关系 |
| **分支术语表** | `## 23B (release/xxx)` 格式自动映射惯用叫法到实际分支名 |
| **多仓支持** | 前后端独立索引，跨仓依赖自动识别 |
| **团队共享** | CI 产出单 SQLite DB，路径无关，维护者 push / 成员 pull |
| **MCP 原生** | 11 个 MCP 工具，OpenCode/Claude Code 零配置对接 |
| **忽略文件** | `.codeloomignore` 过滤 test/build/docs 目录，类似 `.gitignore` |

## 安装

```bash
# Linux / macOS — 预编译二进制
curl -sSL https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.sh | bash

# Linux / macOS — 从源码编译
curl -sSL https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.sh | bash -s -- --from-source

# Windows — 预编译二进制
irm https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.ps1 | iex

# Windows — 从源码编译
irm https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.ps1 | iex -args "--from-source"
```

## 5 分钟上手

```bash
codeloom check                              # 检查环境
codeloom index /path/to/your/cpp/repo       # 索引代码库
codeloom status                             # 查看状态

# 注册到 OpenCode（交互式，依次输入 name 和 command）
opencode mcp add
#   Enter MCP server name → codeloom
#   Enter command → codeloom mcp
```

然后在 OpenCode 里直接用：
```
/codeloom:overview
/codeloom:semantic-search "用户认证流程"
/codeloom:get-definition  login
/codeloom:call-graph       login --direction callers
```

## 输出件存放

```
~/.codeloom/
├── config.yaml              # 项目配置（多仓、路径）
├── <repo>.rag.db            # 知识图谱主文件（单文件 SQLite）
├── <repo>.rag.overlay.db    # 本地增量层（个人分支，不共享）
└── bin/codeloom             # 二进制
```

## 索引基准

| 仓库 | 文件 | 代码规模 | 符号 | 边 | DB 大小 | 时间 |
|------|------|----------|------|-----|---------|------|
| nlohmann/json | 488 | 136K 行 C++ | 3,440 | 14,752 | 13 MB | 4min |
| fmtlib/fmt | 77 | 68K 行 C++ | 2,119 | 8,366 | 4 MB | 2.7min |
| test-inherit | 1 | 21 行 C++ | 10 | 18 | 0.1 MB | 0.4s |

**边类型完整度**（与 Kythe 对齐）：

| 边 | Kythe | JSON | FMT |
|----|-------|------|-----|
| calls | ref/call | 9,111 | 4,809 |
| inherits | extends | 86 | 76 |
| overrides | overrides | 86 | 21 |
| contains | has_member | 983 | 509 |
| param_type | param | 2,205 | 1,903 |
| returns | typeof | 1,783 | 714 |
| field_type | typeof | 325 | 219 |
| includes | (include) | 173 | 115 |

**检索性能（FMT, 2,119 symbols, avg of 100 iterations）**：

| 操作 | 平均 | P99 |
|------|------|-----|
| 精确名称查找 | 0.17ms | 1.83ms |
| 1-hop 调用者 | 0.01ms | 0.02ms |
| 2-hop 调用链 | 0.44ms | 0.71ms |
| 语义搜索 | 19ms | 20ms |

## MCP 工具

在内网 OpenCode 中直接使用：

| 工具 | 功能 | 示例 |
|------|------|------|
| `codeloom_overview` | 仓库全景统计 | `--repo backend` |
| `codeloom_status` | 索引状态 | `--repo backend` |
| `codeloom_list_symbols` | 符号模糊搜索 | `--pattern login` |
| `codeloom_get_definition` | 符号完整定义 | `--name AuthController` |
| `codeloom_get_call_graph` | 调用图遍历 | `--name login --direction callers --max_depth 3` |
| `codeloom_search` | 全文搜索 | `--query "buffer pool"` |
| `codeloom_semantic_search` | 自然语言语义搜索 | `--query "用户登录流程"` |
| `codeloom_index` | 触发增量索引 | CLI 指令指引 |
| `codeloom_pull` | 拉取共享 DB | CLI 指令指引 |
| `codeloom_push` | 推送共享 DB | CLI 指令指引 |
| `codeloom_switch_branch` | 切换分支 | CLI 指令指引 |

## .codeloomignore

在仓库根目录创建 `.codeloomignore`，跳过不需要索引的文件：

```
# 跳过测试
tests/
*_test.cpp
*_unittest.cpp

# 跳过构建产物
build/
.cmake/

# 跳过旧文档
docs/deprecated/

# 跳过三方代码
third_party/
```

规则：`dir/` 匹配目录、`*.ext` 匹配后缀、`prefix*` 匹配前缀、`literal` 子串匹配。`#` 注释。

## 团队共享

### 配置

```yaml
# ~/.codeloom/config.yaml
projects:
  my-project:
    repos:
      backend:   { root: /home/alice/work/backend }
      frontend:  { root: /home/alice/work/frontend }
      shared-lib:{ root: /home/alice/work/shared-lib }
    base_db: s3://team-rag/my-project.rag.db
    similarity_threshold: 0.05
```

### 工作流

```
维护者                              成员
──────                              ────
git checkout main                   git checkout feature-x
codeloom index --repo backend       codeloom index    # 本地 overlay
codeloom push                       codeloom pull     # 拉共享库
```

### 分支术语

文档中维护分支惯用叫法：

```markdown
## 23B (release/2023-B-sprint-patch-4)
2023年B版本补丁系列。
```

或 CLI：

```bash
codeloom branch set-alias 23B release/2023-B-sprint-patch-4 --desc "2023B"
```

## 架构

```
codeloom index ──→ smart.rs ──→ tree_sitter.rs (收集文件+解析)
                         │
                    git.rs (增量检测)    cpp.rs (符号+边提取)
                         │                    │
                    storage/ ──→ SQLite (symbols, edges, doc_nodes)
                         │
                    embedding/ ──→ Jaccard (doc↔code 链接)
                         │
                    doc/ ──→ Markdown 解析 + 术语表
                         │
                    mcp/ ──→ 11 个 JSON-RPC 工具 → OpenCode
```

## 许可证

MIT

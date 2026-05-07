# CodeLoom v0.5.1

> 代码知识管理工具 — 为 LLM Agent 编织代码库知识图谱

## 是什么

CodeLoom 把零散的代码、文档、业务知识编织成一张可查询的知识图谱，让 OpenCode/Claude Code 等 AI 编码助手中的 LLM 能理解百万行级别的多代码仓项目。

纯本地运行，零外部 API 依赖，代码不出内网。远程部署 MCP Server，**一个 .db 文件拎包就走**。

## 能力

| 能力 | 说明 |
|------|------|
| **代码知识图谱** | tree-sitter 解析 C++/Python/Java/TypeScript/Go，提取符号定义、调用图、继承链、include 关系 |
| **多格式文档** | 支持 md/rst/xlsx/docx/pdf/xml/html 文档索引与搜索，统一格式路由 |
| **Excel 结构化查询** | 智能表头检测 + 四层节点模型（sheet→header→row→cell），MCP SQL-like 查询 |
| **文档图片提取** | MD/HTML/DOCX/XLSX 内图片自动提取，WebP 智能压缩，SQLite BLOB 存储 |
| **图片 base64 返回** | MCP 返回图片时 base64 编码，远程部署无路径依赖 |
| **编码兼容** | UTF-8 / GB2312 / GBK / GB18030 自动检测，中文编码源码零配置索引 |
| **语义嵌入** | candle + bge-small-zh（512 维，91MB），纯 CPU 推理，零外部 API |
| **混合搜索** | FTS5 BM25 关键词 + vec0 向量语义，RRF 融合统一排名。搜索返回 snippet + 图片提示 |
| **Git 驱动增量** | 自动跟踪 commit，`git diff` 只扫变更文件；新分支从父分支继承符号 |
| **分支过滤** | 所有 MCP 工具 `branch` 参数必传；`branch_name IS NULL` 的数据所有分支可见 |
| **多仓支持** | 前后端独立索引，跨仓依赖自动识别 |
| **MCP 原生** | 11 个 MCP 工具（含 get_doc + query_excel），OpenCode/Claude Code 零配置对接 |
| **自动化测试** | 74 测试（67 单元 + 7 集成），本地素材自洽，`cargo test` 一键验证 |

## 安装

### 离线安装包（推荐，无需网络）

1. 浏览器打开 [Gitee Releases](https://gitee.com/greengreensea/codeloom/releases)（国内快）或 [GitHub Releases](https://github.com/sherlock-bug/codeloom/releases)
2. 下载 `codeloom-vX.Y.Z-linux-x86_64.zip`
3. 解压并安装：

```bash
unzip codeloom-vX.Y.Z-linux-x86_64.zip
cd codeloom-vX.Y.Z-linux-x86_64   # zip 内包含独立目录的话
./install.sh
```

全程零网络请求，二进制 + 模型文件一并部署到 `~/.codeloom/`。

### 预编译二进制（在线）

```bash
# Linux / macOS（国内推荐 Gitee）
curl -sSL https://gitee.com/greengreensea/codeloom/raw/master/scripts/install.sh | bash

# 或 GitHub（海外）
curl -sSL https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.sh | bash
```

安装脚本自动部署二进制和嵌入模型到 `~/.codeloom/`。

### 从源码编译

```bash
git clone https://gitee.com/greengreensea/codeloom.git  # 国内推荐
# 或 git clone https://github.com/sherlock-bug/codeloom.git
cd codeloom
cargo build --release   # build.rs 自动从 modelscope.cn 下载 91MB 模型
```

要求：Rust 1.80+、curl。

## 5 分钟上手

```bash
codeloom check                              # 检查环境（含模型状态）
codeloom index /path/to/your/cpp/repo       # 索引代码库 + 文档
codeloom status                             # 查看状态（自动检测仓库和分支）
codeloom search "auth token"                # 搜索（自动检测仓库和分支）

# 注册到 OpenCode
opencode mcp add
#   name → codeloom
#   command → codeloom mcp
```

OpenCode 里直接用：
```
/codeloom:overview
/codeloom:search "用户认证流程"
/codeloom:get-definition  login
/codeloom:call-graph       login --direction callers
```

## 输出件存放

```
~/.codeloom/
├── config.yaml              # 项目配置（多仓、路径）
├── <repo>.rag.db            # 知识图谱主文件（单文件 SQLite）
├── models/bge-small-zh/     # 嵌入模型（91MB，编译/安装时自动下载）
└── bin/codeloom             # 二进制 (~15MB)
```

## 索引基准

| 仓库 | 文件 | 代码规模 | 符号 | 边 | DB 大小 | 时间 |
|------|------|----------|------|-----|---------|------|
| nlohmann/json | 488 | 136K 行 C++ | 3,440 | 14,752 | 13 MB | 4min |
| fmtlib/fmt | 77 | 68K 行 C++ | 2,119 | 8,366 | 4 MB | 2.7min |
| leveldb | 132 | 5.7K 行 C++ | 1,956 | 9,236 | 3.9 MB | 1.5min |

**检索性能（FMT, 2,119 symbols, avg of 100 iterations）**：

| 操作 | 平均 | P99 |
|------|------|-----|
| 精确名称查找 | 0.17ms | 1.83ms |
| 1-hop 调用者 | 0.01ms | 0.02ms |
| 2-hop 调用链 | 0.44ms | 0.71ms |
| 语义搜索 (vec0 ANN) | 2ms | 5ms |

## MCP 工具

9 个 MCP 工具，所有搜索/查询工具的 `branch` 参数必传。**打开仓库后第一步先调 `codeloom_list_repos` 获取可用仓库名。**

### 仓库与状态管理

| 工具 | 参数 | 说明 |
|------|------|------|
| `codeloom_list_repos` | 无 | **第一步调用**：列出所有已索引的仓库名。 |
| `codeloom_list_branches` | `repo` | 列出指定仓库的所有已索引分支及符号数。 |
| `codeloom_status` | `branch`, `repo` | 查看索引状态：符号数、边数、文档数、DB 大小。 |
| `codeloom_overview` | `branch`, `repo` | 仓库架构全貌：符号按类型分布、文件数、边数。 |
| `codeloom_index` | `path`, `branch`, `repo?` | ⚠️ 不通过 MCP 执行索引（需 CLI）。调用前先用 `codeloom_list_repos` 检查是否已索引。 |

### 符号查询（按名称）

| 工具 | 参数 | 说明 |
|------|------|------|
| `codeloom_search` | `query`, `branch`, `limit?`, `repo` | **首选搜索工具**：同时理解精确命名和中文/英文功能意图。自动融合关键词 BM25 和语义向量，返回统一排序。query 可以是符号名或功能描述。 |
| `codeloom_list_symbols` | `pattern`, `branch`, `limit?`, `repo` | 模糊搜索符号名（SQL LIKE）。C++ 类方法用 `ClassName::methodName` 格式。 |
| `codeloom_get_definition` | `name`, `branch`, `repo` | 获取符号完整定义。返回精确代码区间不浪费 token。**先用 `codeloom_list_symbols` 查确切名称。** |

### 关系分析

| 工具 | 参数 | 说明 |
|------|------|------|
| `codeloom_get_call_graph` | `name`, `branch`, `direction?`, `max_depth?`, `repo` | **唯一方式**（grep 无法获取调用关系）：分析调用者和被调用者。`direction="callers"` / `"callees"`。 |

### 常见查询模式

**「查看 AClass::method1 调用了哪些函数」：**
1. `codeloom_list_symbols(pattern="method1")` → 确认完整名称 `AClass::method1`
2. `codeloom_get_call_graph(name="AClass::method1", direction="callees")`

**「找到登录相关代码」：**
1. `codeloom_search(query="用户登录认证")` → 混合搜索找到相关符号和文档
2. `codeloom_get_definition(name="AuthService::login")` → 查看具体实现

**「了解某个类的继承关系」：**
`codeloom_list_symbols(pattern="ClassName")` → 查看类及其所有方法

**分支过滤规则：** `branch_name IS NULL` 的数据对所有分支可见；有值的仅匹配分支可见。代码和文档一视同仁。

## OpenCode MCP 配置

### 1. 注册 MCP Server

安装后，在项目目录下注册 CodeLoom 到 OpenCode：

```bash
# 本地模式（二进制在本机）
opencode mcp add codeloom -- codeloom mcp

# 远程模式（MCP Server 部署在远端服务器，多人共享）
opencode mcp add codeloom -- ssh user@host -- codeloom mcp
```

注册后重启 OpenCode，MCP 工具自动生效。

### 2. 验证连接

在 OpenCode 中执行：

```
/codeloom:check    # 检查环境
/codeloom:status   # 查看索引状态
```

### 3. OpenCode 自定义命令

| 命令 | 功能 |
|------|------|
| `/codeloom:index` | 增量索引当前项目 |
| `/codeloom:status` | 查看索引状态和统计 |
| `/codeloom:search` | 搜索符号、定义、调用关系 |
| `/codeloom:branch` | 切换或查看当前索引分支 |
| `/codeloom:check` | 检查安装状态和环境 |
| `/codeloom:setup` | 交互式引导配置 |
| `/codeloom:smart-setup` | 从团队配置模板导入 |
| `/codeloom:export` | 导出团队配置模板 |

### 4. 手动 MCP 配置

也可以直接在 OpenCode 的 `.opencode/config.json` 中配置：

```json
{
  "mcpServers": {
    "codeloom": {
      "command": "codeloom",
      "args": ["mcp"]
    }
  }
}
```

远程部署：

```json
{
  "mcpServers": {
    "codeloom": {
      "command": "ssh",
      "args": ["user@host", "--", "codeloom", "mcp"]
    }
  }
}
```

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

# 跳过三方代码
third_party/
```

规则：`dir/` 匹配目录、`*.ext` 匹配后缀、`prefix*` 匹配前缀、`literal` 子串匹配。`#` 注释。

## 配置

```yaml
# ~/.codeloom/config.yaml
projects:
  my-project:
    repos:
      backend:   { root: /home/alice/work/backend }
      frontend:  { root: /home/alice/work/frontend }
    similarity_threshold: 0.15
```

## 分支术语

```markdown
## 23B (release/2023-B-sprint-patch-4)
2023年B版本补丁系列。
```

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
                    storage/  ──→ sqlite-vec ANN + 关系结构
                    embedding/ ──→ candle + bge-small-zh (512维)
                         │
                    doc/ ──→ Markdown 解析 + 术语表
                         │
                    mcp/ ──→ 9 个 JSON-RPC 工具 → OpenCode
```

## 开发

```bash
cargo test                    # 49 单元测试 (< 3s)
cargo test --test integration  # 6 集成测试 (~180s，需 models/)
```

测试素材：`tests/fixtures/leveldb` (132 C++ 文件) + `tests/fixtures/docs` (102 .md 文件)。

## 许可证

MIT

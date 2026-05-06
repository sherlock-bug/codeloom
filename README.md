# CodeLoom v0.2

> 代码知识管理工具 — 为 LLM Agent 编织代码库知识图谱

## 是什么

CodeLoom 把零散的代码、文档、业务知识编织成一张可查询的知识图谱，让 OpenCode/Claude Code 等 AI 编码助手中的 LLM 能理解百万行级别的多代码仓项目。

纯本地运行，零外部 API 依赖，代码不出内网。远程部署 MCP Server，单人维护即可。

## 能力

| 能力 | 说明 |
|------|------|
| **代码知识图谱** | tree-sitter 解析 C++/Python/Java/TypeScript/Go，提取符号定义、调用图、继承链、include 关系 |
| **编码兼容** | UTF-8 / GB2312 / GBK / GB18030 自动检测，中文编码源码零配置索引 |
| **本地语义嵌入** | candle + bge-small-zh（512 维，91MB），纯 CPU 推理，零外部 API。模型缺失时自动降级 Jaccard |
| **Git 驱动增量** | 自动跟踪 commit，`git diff` 只扫变更文件；新分支从父分支继承符号 |
| **分支过滤** | 所有 MCP 工具 `branch` 参数必传；`branch_name IS NULL` 的数据所有分支可见 |
| **向量搜索** | sqlite-vec ANN 引擎（预编译 .so，160KB），O(log N) 检索，一键安装 |
| **分支术语表** | `## 23B (release/xxx)` 格式自动映射惯用叫法到实际分支名 |
| **多仓支持** | 前后端独立索引，跨仓依赖自动识别 |
| **MCP 原生** | 8 个 MCP 工具，OpenCode/Claude Code 零配置对接 |
| **忽略文件** | `.codeloomignore` 过滤 test/build/docs 目录，类似 `.gitignore` |
| **自动化测试** | 47 测试（41 单元 + 6 集成），本地素材自洽，`cargo test` 一键验证 |

## 安装

### 离线安装包（推荐，无需网络）

1. 浏览器打开 [GitHub Releases](https://github.com/sherlock-bug/codeloom/releases)
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
# Linux / macOS
curl -sSL https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.sh | bash

# Windows
irm https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.ps1 | iex
```

安装脚本自动部署二进制和嵌入模型到 `~/.codeloom/`。

### 从源码编译

```bash
git clone https://github.com/sherlock-bug/codeloom.git
cd codeloom
cargo build --release   # build.rs 自动从 modelscope.cn 下载 91MB 模型
```

要求：Rust 1.80+、curl。

## 5 分钟上手

```bash
codeloom check                              # 检查环境（含模型状态）
codeloom index /path/to/your/cpp/repo       # 索引代码库 + 文档
codeloom status --repo myrepo --branch main # 查看状态

# 注册到 OpenCode
opencode mcp add
#   name → codeloom
#   command → codeloom mcp
```

OpenCode 里直接用：
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

8 个 MCP 工具，所有查询工具的 `branch` 参数必传：

| 工具 | 功能 |
|------|------|
| `codeloom_overview` | 仓库全景统计（符号/边/文档分布） |
| `codeloom_status` | 索引状态（符号数、边数、文档数、DB 大小） |
| `codeloom_list_symbols` | 按名称模糊搜索符号 |
| `codeloom_get_definition` | 获取符号完整定义（含源码、签名、路径、行号） |
| `codeloom_get_call_graph` | 调用图遍历（callers / callees，深度可配） |
| `codeloom_search` | 全文搜索符号名和定义 |
| `codeloom_semantic_search` | 自然语言语义搜索代码 + 文档（sqlite-vec ANN，512 维） |
| `codeloom_index` | 触发增量索引 |

分支过滤规则：`branch_name IS NULL` 的数据对所有分支可见；有值的仅匹配分支可见。代码和文档一视同仁。

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
                    mcp/ ──→ 8 个 JSON-RPC 工具 → OpenCode
```

## 开发

```bash
cargo test                    # 41 单元测试 (< 3s)
cargo test --test integration  # 6 集成测试 (~180s，需 models/)
```

测试素材：`tests/fixtures/leveldb` (132 C++ 文件) + `tests/fixtures/docs` (102 .md 文件)。

## 许可证

MIT

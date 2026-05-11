# CodeLoom v0.7.0

> 代码知识管理工具 — 为 LLM Agent 编织代码库知识图谱

## 是什么

CodeLoom 把零散的代码、文档、业务知识编织成一张可查询的知识图谱，让 OpenCode/Claude Code 等 AI 编码助手中的 LLM 能理解百万行级别的多代码仓项目。

纯本地运行（索引 + 关键词搜索），语义搜索需配置 OpenAI 兼容 Embedding API，代码不出内网。

## 安装

### 离线安装（推荐）

从 GitHub Releases 下载 `codeloom-vX.Y.Z-linux-x86_64.zip`，解压后运行：

```bash
unzip codeloom-v*-linux-x86_64.zip
./install.sh
codeloom check          # 验证环境
```

离线 zip 包含：`codeloom` 二进制 + `clang_filter.py`（Clang AST 过滤）+ `install.sh`，不包含模型文件（使用 API 向量化）。

**依赖**（仅 C/C++ 索引需要）：
- `clang` — C/C++ AST 解析，Ubuntu: `sudo apt install clang`
- `python3` — Clang AST 过滤管道，Ubuntu: `sudo apt install python3`

### 在线安装

```bash
curl -sSL https://gitee.com/greengreensea/codeloom/raw/master/scripts/install.sh | bash
```

### 从源码编译

```bash
curl -sSL ... | bash -s -- --from-source
```

**注意**：install.sh 会保护 `~/.codeloom/config.yaml` 不被覆盖。如需重置配置，手动删除后重跑安装。

## 能力

| 能力 | 说明 |
|------|------|
| **代码知识图谱** | Clang 解析 C/C++（编译器精度：宏展开/模板实例化/重载消歧），tree-sitter 解析 Python/Java/TypeScript/Go。15 种节点 × 11 种边，含调用图/继承链/类型约束/别名/外部符号 |
| **多格式文档** | 支持 md/rst/xlsx/docx/pdf/xml/html 文档索引与搜索，统一格式路由 |
| **Excel 结构化查询** | 智能表头检测 + 四层节点模型（sheet→header→row→cell），MCP SQL-like 查询 |
| **文档图片提取** | MD/HTML/DOCX/XLSX 内图片自动提取，WebP 智能压缩，SQLite BLOB 存储 |
| **图片 base64 返回** | MCP 返回图片时 base64 编码，远程部署无路径依赖 |
| **编码兼容** | UTF-8 / GB2312 / GBK / GB18030 自动检测，中文编码源码零配置索引 |
| **语义嵌入** | OpenAI 兼容 API（`/v1/embeddings`），支持 bge-m3 / text-embedding-3 等任意嵌入模型 |
| **精确 BM25 搜索** | `codeloom_search` — 纯 FTS5 BM25 关键词搜索，覆盖符号名(×0.7)+注释(×0.3)+文档标题(×0.7)+内容(×0.3)+文件名(×0.7)+摘要(×0.3) |
|| **向量语义搜索** | `codeloom_semantic_search` — 纯 vec0 INT8 KNN 向量搜索，用自然语言描述功能找符号 |
|| **分通道噪音标定** | BM25 和向量各自独立标定噪音基线，互不干扰 |
| **注释索引** | C++ 行内注释 + 体内注释自动收集，支持中文/英文注释搜索 |
| **文件节点** | 代码文件元信息索引 + 注释摘要，支持文件名和内容搜索 |
| **文档切分** | 长文档按 ≤500 字自动切分为 chunk，标点优先级切割，保留文档层级结构 |
| **Git 驱动增量** | 自动跟踪 commit，`git diff` 只扫变更文件；新分支从父分支继承符号 |
| **分支过滤** | 所有 MCP 工具 `branch` 参数必传；`branch_name IS NULL` 的数据所有分支可见 |
| **多仓支持** | 前后端独立索引，跨仓依赖自动识别 |
| **MCP 原生** | 16 个 MCP 工具（含 schema/search/semantic_search/inspect/path_analysis/impact_analysis），OpenCode/Claude Code 零配置对接 |
| **自动化测试** | 80 测试（73 单元 + 7 集成），本地素材自洽，`cargo test` 一键验证 |
| **噪声过滤** | 内置标定语料库 + 探针系统，自动计算噪声基线（mean+2.5σ），`check`/首次`index` 标定，搜索结果自动过滤低置信度条目 |

## 安装

### 离线安装包（推荐，无需网络）

1. 浏览器打开 [Gitee Releases](https://gitee.com/greengreensea/codeloom/releases)（国内快）或 [GitHub Releases](https://github.com/sherlock-bug/codeloom/releases)
2. 下载 `codeloom-vX.Y.Z-offline.tar.gz`
3. 解压并安装：

```bash
tar xzf codeloom-vX.Y.Z-offline.tar.gz
./install.sh
```

安装到 `~/.codeloom/bin/codeloom`，配置文件模板写入 `~/.codeloom/config.yaml`。

### 预编译二进制（在线）

```bash
# Linux / macOS（国内推荐 Gitee）
curl -sSL https://gitee.com/greengreensea/codeloom/raw/master/scripts/install.sh | bash

# 或 GitHub（海外）
curl -sSL https://raw.githubusercontent.com/sherlock-bug/codeloom/master/scripts/install.sh | bash
```

### 从源码编译

```bash
git clone https://gitee.com/greengreensea/codeloom.git  # 国内推荐
# 或 git clone https://github.com/sherlock-bug/codeloom.git
cd codeloom
cargo build --release
```

要求：Rust 1.80+。build.rs 自动编译 sqlite-vec 扩展（静态链接）。

## 5 分钟上手

```bash
# 1. 配置 Embedding API（语义搜索必需）
cat > ~/.codeloom/config.yaml << 'YAML'
embedding:
  api_base: "http://localhost:11434/v1"   # Ollama
  model: "bge-m3"
YAML

# 2. 索引 + 搜索
codeloom check                              # 检查环境 + 噪声标定（自动过滤低质量搜索结果）
codeloom index /path/to/your/cpp/repo       # 索引代码库 + 文档（首次自动标定）
codeloom status                             # 查看状态
codeloom search "auth token"                # FTS5 + 向量混合搜索（自动过滤噪声）

# 注册到 OpenCode
opencode mcp add codeloom -- codeloom mcp
```

OpenCode 里直接用：
```
/codeloom:overview
/codeloom:search "用户认证流程"
/codeloom:inspect      login
/codeloom:call-graph       login --direction callers
```

## 输出件存放

```
~/.codeloom/
├── config.yaml              # 项目配置（多仓路径 + embedding API）
├── <repo>.rag.db            # 知识图谱主文件（单文件 SQLite）
└── bin/codeloom             # 二进制 (~18MB)
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

15 个 MCP 工具，所有搜索/查询工具的 `branch` 参数必传。**打开仓库后第一步先调 `codeloom_list_repos` 获取可用仓库名。**

### 仓库与状态管理

| 工具 | 参数 | 说明 |
|------|------|------|
| `codeloom_list_repos` | 无 | **第一步调用**：列出所有已索引的仓库名。 |
| `codeloom_list_branches` | `repo` | 列出指定仓库的所有已索引分支及符号数。 |
| `codeloom_status` | `branch`, `repo` | 查看索引状态：符号数、边数、文档数、DB 大小。 |
| `codeloom_index` | `path`, `branch`, `repo?` | ⚠️ 不通过 MCP 执行索引（需 CLI）。调用前先用 `codeloom_list_repos` 检查是否已索引。 |

### 符号查询

| 工具 | 参数 | 说明 |
|------|------|------|
| `codeloom_search` | `query`, `branch`, `limit?`, `repo` | **首选搜索工具**：同时理解精确命名和中文/英文功能意图。自动融合关键词 BM25 和语义向量，返回统一排序。 |
| `codeloom_list_symbols` | `pattern`, `branch`, `limit?`, `repo` | 模糊搜索符号名。C++ 类方法用 `ClassName::methodName` 格式。 |
| `codeloom_inspect` | `name`, `branch`, `repo` | 查看节点全部信息：定义、注释、所有关联边（调用/继承/参数/返回/字段）。**先 search 再 inspect。** |

### 关系与分析

| 工具 | 参数 | 说明 |
|------|------|------|
| `codeloom_get_call_graph` | `name`, `branch`, `direction?`, `max_depth?`, `repo` | 分析函数的调用者/被调用者（grep 无法获取调用关系）。`direction="callers"` / `"callees"`。 |
| `codeloom_path_analysis` | `source`, `target`, `branch`, `repo`, `mode?`, `edge_filter?`, `max_paths?`, `direction?` | 两点间路径分析。`mode="shortest"` 最短路径 / `"all"` 全部路径，支持按边类型过滤（`edge_filter=["calls","inherits"]`）。 |
| `codeloom_impact_analysis` | `symbol`, `branch`, `repo`, `radius?`, `direction?` | 影响分析：修改某个符号会影响哪些其他符号。`direction="reverse"` 查谁依赖它 / `"forward"` 查它依赖谁。 |
| `codeloom_neighbor_graph` | `symbol`, `branch`, `repo`, `depth?`, `direction?` | 符号邻里图：查看某个符号周围的直接关联，按边类型分组（calls/returns/param_type/inherits 等）。 |
| `codeloom_inheritance_tree` | `symbol`, `branch`, `repo`, `direction?`, `max_depth?` | 类继承树。`direction="down"` 查子类 / `"up"` 查父类，含虚函数 override 信息。 |

### 文档与元数据

| 工具 | 参数 | 说明 |
|------|------|------|
| `codeloom_get_doc` | `doc_id`, `branch`, `repo` | 获取文档节点完整内容及嵌入图片。 |
| `codeloom_query_excel` | `doc_id`, `branch`, `repo`, `mode?`, `filter?`, `limit?` | Excel 结构化查询：row/column/filter/auto 四种模式。 |
| `codeloom_schema` | 无 | 元数据工具：返回所有节点类型（15 种）和边类型（10 种）的定义及方向语义。

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

### Embedding API（语义搜索必需）

语义搜索（向量召回）通过 OpenAI 兼容的 `/v1/embeddings` 接口调用嵌入模型。不配置时仅能使用 FTS5 关键词搜索。

```yaml
# ~/.codeloom/config.yaml
embedding:
  api_base: "http://localhost:11434/v1"   # 必填，OpenAI 兼容 API 地址
  model: "bge-m3"                         # 必填，模型名
  api_key: "not-needed"                   # 可选，Bearer Token
  dimension: 1024                         # 可选，向量维度（自动检测）
  batch_size: 64                          # 可选，批量大小（默认 64）
  text_limit: 300                         # 可选，单文本截断长度（默认 300）
  max_chars_per_batch: 90000              # 可选，单批最大字符数（默认 90000）
```

**支持的嵌入服务：**

| 服务 | api_base 示例 | 推荐模型 | 维度 |
|------|--------------|---------|------|
| Ollama | `http://localhost:11434/v1` | `bge-m3` | 1024 |
| LM Studio | `http://localhost:1234/v1` | `text-embedding-nomic-embed-text-v1.5` | 768 |
| Xinference | `http://localhost:9997/v1` | `bge-m3` | 1024 |
| OpenAI | `https://api.openai.com/v1` | `text-embedding-3-small` | 1536 |
| 内网部署 | `http://your-server:port/v1` | 任意兼容模型 | — |

**验证连通性：**

```bash
codeloom check    # 会尝试调一次 /v1/embeddings，报告状态
```

### 日志

日志默认开启。在 `config.yaml` 中配置后，CodeLoom 将关键操作（索引、搜索、错误）记录到文件：

```yaml
# ~/.codeloom/config.yaml
logging:
  enabled: true           # 默认 true
  level: "info"           # error | warn | info | debug
  max_file_size_mb: 50    # 单文件大小上限
  max_files: 10           # 保留文件数上限
```

**日志位置**：`~/.codeloom/logs/codeloom_{YYYYMMDD-HHMMSS}_{毫秒}.log`

**日志格式**：`2026-05-10 15:30:12.345 [PID:TID] LEVEL module: message`

**快速查看**：
```bash
# 最新日志
cat $(ls -t ~/.codeloom/logs/*.log | head -1)

# 只看错误
grep ERROR ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)

# 按模块过滤
grep "mcp" ~/.codeloom/logs/$(ls -t ~/.codeloom/logs/ | head -1)
```

日志文件超过 `max_file_size_mb` 时自动绕接到新文件，超过 `max_files` 时删除最旧文件。

### 多仓配置

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
                    embedding/ ──→ OpenAI 兼容 API (`/v1/embeddings`)
                         │
                    doc/ ──→ Markdown 解析 + 术语表
                         │
                    mcp/ ──→ 15 个 JSON-RPC 工具 → OpenCode
```

## 开发

```bash
cargo test                    # 73 测试 (< 3s)
cargo test --test integration  # 7 集成测试 (~180s)
```

测试素材：`tests/fixtures/leveldb` (132 C++ 文件) + `tests/fixtures/docs` (102 .md 文件)。

## 许可证

MIT

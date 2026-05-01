# CodeLoom

> 团队代码知识管理工具 — 为 LLM Agent 编织代码库知识图谱

## 是什么

CodeLoom 把零散的代码、文档、业务知识编织成一张可查询的知识图谱，让 OpenCode 中的 LLM 能理解百万行级别的多代码仓项目。

## 能力

- **代码知识图谱**：tree-sitter 解析 C++/Python/Java/TypeScript/Go，提取符号定义、调用图、继承链
- **Git 驱动增量**：自动跟踪 commit，`git diff` 只扫变更文件；新分支从父分支继承符号
- **文档自动关联**：bge-small-zh embedding 自动建立文档段落与代码符号的对应关系
- **分支术语**：`## 23B (release/xxx)` 格式自动映射惯用叫法到实际分支名
- **多仓支持**：前后端独立索引，跨仓依赖自动识别
- **团队共享**：CI 产出单 SQLite DB，路径无关，维护者 push / 成员 pull

## 安装

```bash
# Linux / macOS
curl -sSL https://get.codeloom.dev | bash

# Windows
irm https://get.codeloom.dev/windows | iex
```

## 快速上手

```bash
# 首次安装后检查环境
codeloom check

# 索引当前项目
codeloom index

# 查看状态
codeloom status

# 语义搜索
codeloom semantic-search "用户认证流程"

# 注册到 OpenCode
opencode mcp add codeloom
```

## 输出件存放

CodeLoom 的所有数据存放在 `~/.codeloom/` 下：

```
~/.codeloom/
├── config.yaml              # 项目配置（多仓、路径、阈值）
├── project.rag.db           # 知识图谱主文件（team-shared）
├── project.rag.overlay.db   # 本地增量层（个人分支修改，不共享）
└── bin/
    └── codeloom             # 二进制
```

- `project.rag.db` — 这是你要共享给团队的文件。单文件 SQLite，百万行代码约 200MB。
- `project.rag.overlay.db` — 本地个人修改，**不共享**。feature 分支的增量存这里。
- 文件名可自定义，取决于 config.yaml 中的配置。

## 团队共享配置

### 第一步：配置项目

创建 `~/.codeloom/config.yaml`：

```yaml
projects:
  my-project:                          # 项目组名
    repos:                              # 多仓定义
      backend:
        root: /home/alice/work/backend  # Alice 的路径
      frontend:
        root: /home/alice/work/frontend
      shared-lib:
        root: /home/alice/work/shared-lib
    base_db: s3://team-rag/my-project.rag.db   # 共享 DB 地址
    similarity_threshold: 0.75          # 文档-代码关联阈值
```

### 第二步：各仓独立索引

```bash
codeloom index --repo backend   --branch main
codeloom index --repo frontend  --branch main
codeloom index --repo shared-lib --branch main
```

所有仓的符号写入同一个 `project.rag.db`，跨仓依赖自动识别。

### 第三步：CI 自动产出共享 DB

```yaml
# .github/workflows/codeloom.yml
- name: Build knowledge graph
  run: |
    codeloom index --repo backend  --branch main
    codeloom index --repo frontend --branch main
    codeloom db upload --source s3://team-rag/
```

### 第四步：团队成员拉取

```bash
# 首次加入
codeloom pull --source s3://team-rag/my-project.rag.db

# 日常同步（拉取最新的团队共享 DB）
codeloom pull
```

## 团队协作模式

### 角色分工

```
维护者（1-2 人）                    普通成员（其他人）
                                    
git checkout main                   git checkout feature-x
git pull                            写代码...
codeloom index --branch main        本地增量索引（只写 overlay）
codeloom push                       codeloom pull      ← 拉维护者更新的共享库
  └→ 上传 base DB 到团队存储          codeloom pull      ← 定期同步
```

### 关键设计

- `codeloom index` 只写本地 overlay DB (`project.rag.overlay.db`)，**不污染团队库**
- `codeloom push` 只推 base DB (`project.rag.db`)，overlay 永不离机
- `codeloom pull` 拉取 base DB，覆盖本地共享层，不影响个人 overlay

```bash
# 正常分支日常：拉共享库 + 本地增量
codeloom index                    # commit 不变自动跳过
codeloom index --branch my-fix    # 本地索引当前分支（只写 overlay）

# 新分支：显式指定源分支跳过检测
codeloom index --parent develop   # 从 develop 继承符号 + 只扫 fork 后变更
codeloom index --parent main      # 从 main 继承

# 维护者合并代码后：更新共享库
git checkout main && git pull
codeloom index                    # 全量索引 main
codeloom status                   # 确认无误
codeloom push                     # 上传到团队共享存储
```

## 分支惯用叫法

分支名可能很长（如 `release/2023-B-sprint-patch-4`），团队习惯叫"23B"。两种方式建立映射：

### 方式一：直接命令行（适合临时添加）

```bash
# 添加单条映射
codeloom branch set-alias 23B release/2023-B-sprint-patch-4 \
    --desc "2023年B版本补丁系列" --repo my-project

# 查看所有映射
codeloom branch list-aliases --repo my-project
```

### 方式二：术语文档（适合批量维护）

创建 `docs/branch-glossary.md`，格式为 `## 惯用叫法 (实际分支名)`：

```markdown
# 分支术语表

## 23B (release/2023-B-sprint-patch-4)
2023年B版本补丁系列，包含安全修复和性能优化。

## 24A / release/2024-A-mainline
2024年A版本主线，新增结算引擎模块。

## hotfix-login → release/2025-hotfix-login-v3
登录模块紧急修复，解决OAuth2超时问题。
```

支持三种格式：`别名 (分支名)`、`别名 / 分支名`、`别名 → 分支名`。

```bash
codeloom index docs/branch-glossary.md --repo my-project
```

> `codeloom index` 一条命令搞定一切 — 自动识别文件类型（代码走 tree-sitter，文档走 Markdown 解析），支持文件和目录两种参数。

## 路径无关化

数据库内**不存绝对路径**，存的是项目根目录的**相对路径**：

```
DB 中存储:     src/core.cpp
Alice 解析:    /home/alice/work/backend/src/core.cpp  ← config 中 root 拼接
Bob 解析:      /home/bob/dev/backend/src/core.cpp     ← config 中 root 拼接
```

校验机制：符号定义文本的 SHA256 哈希存 DB，查询时对比磁盘文件的当前哈希。不匹配 → 提示"该符号已过期，建议 codeloom index"。

## 查询时按仓过滤

所有 MCP 工具支持 `repo` 参数（默认 `all`）：

```
# 只看前端仓
codeloom get-definition login --repo frontend

# 跨仓完整调用链
codeloom get-call-graph login
  → frontend/login → apiClient → [→ backend] AuthController::login → ...
```

## 状态检查

```bash
$ codeloom status

  backend (main):         12,000 符号
  backend (feature-pmt):  +1,200 符号
  frontend (main):         8,500 符号
  shared-lib (main):       3,200 符号
  ──────────────────────────────
  总计:                   24,900 符号
  跨仓边:                  1,240 条
  存储:                    180 MB
```

## 从源码构建

需要 Rust 1.80+：

```bash
git clone https://github.com/xxx/codeloom
cd codeloom
cargo build --release
```

## 许可证

MIT

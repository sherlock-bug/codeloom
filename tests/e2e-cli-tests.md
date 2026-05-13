# CodeLoom CLI 端到端手工测试用例

> **版本**: codeloom 1.0.0
> **二进制**: `/mnt/d/RagMcpHermes/codeloom/target/debug/codeloom`
> **测试仓库**: leveldb（唯一基准，~133 C++ 文件，真实项目）
> **leveldb 路径**: `/mnt/d/code/leveldb`
> **分支**: `master`
> **测试前准备**: 见下方「测试编排」脚本，自动完成 leveldb 全量清理+索引

---

## 测试编排

所有 CLI 测试基于 leveldb。每次执行前必须运行此编排：

```bash
# 别名
alias cl=/mnt/d/RagMcpHermes/codeloom/target/debug/codeloom

# 1. 清理 leveldb 旧数据，全新开始
rm -f ~/.codeloom/leveldb.rag.db
cl clean --repo leveldb 2>/dev/null

# 2. 全量索引 leveldb（所有后续测试的基础）
cl index /mnt/d/code/leveldb --repo leveldb --branch master

# 3. 验证索引成功
cl status --repo leveldb
# 预期: Symbols > 3000 (实测 3359), Edges > 6000 (实测 6487, 解析率 97%)
#       Docs = 63, FTS5 > 3500, DB size ~14MB
```

编排只用 leveldb，不涉及其他仓库。所有测试用例默认 --repo leveldb --branch master。

---

## 模块 A：索引类（Index）

---

### CLI-1 全量索引验证

| 字段 | 内容 |
|------|------|
| **名称** | 全量索引 — 对 leveldb 做首次全量索引，验证完整的索引管线（Clang C++ 解析、符号提取、边提取、FTS5、向量化、噪声标定） |
| **前置条件** | 无（编排脚本已清理旧库） |
| **步骤** | ① 执行「测试编排」中的索引命令<br>② 索引完成后验证 |
| **预期结果** | 索引完成，输出包含符号数 > 3000、边数 > 6000，无 error 日志 |
| **验证方法** | ① `cl status --repo leveldb` 输出 Symbols > 3000, Edges > 6000, DB size ~14MB<br>② 确认 FTS5 entries > 3500<br>③ 确认向量化完成（symbols indexed > 3000） |

### CLI-2 增量索引（重复索引验证去重）

| 字段 | 内容 |
|------|------|
| **名称** | 增量索引 — 对已索引的 leveldb 重新执行 index，验证 upsert 逻辑正确（不重复插入符号） |
| **前置条件** | leveldb 已索引（CLI-1 后） |
| **步骤** | ① `cl index /mnt/d/code/leveldb --repo leveldb --branch master` |
| **预期结果** | 索引快速完成，符号数不变或仅合理增加（如文档分块稍有不同），不报重复键错误 |
| **验证方法** | 第二次索引后 `cl status --repo leveldb` 的 Symbols 数相比 CLI-1 变化 < 5% |

### CLI-3 非 Git 无编译命令索引

| 字段 | 内容 |
|------|------|
| **名称** | 边界索引 — 对无 `compile_commands.json` 的纯 `.h` 目录索引，验证优雅降级 |
| **前置条件** | leveldb include/ 目录存在 |
| **步骤** | ① `cl index /mnt/d/code/leveldb/include --repo leveldb-headers --branch master`<br>② `cl status --repo leveldb-headers` |
| **预期结果** | 索引正常完成（头文件无 .cpp TU，Clang 回退到单文件模式），符号数可能为 0 或少量 |
| **验证方法** | ① 退出码 0，输出包含 "Done" 且无 panic<br>② 完成后 `cl clean --repo leveldb-headers` 清理 |

### CLI-4 索引缺参数/错误路径

| 字段 | 内容 |
|------|------|
| **名称** | 索引边界 — 缺 path、不存在的路径 |
| **步骤** | ① `cl index`（无参数）<br>② `cl index /nonexistent/path --repo bad-repo --branch master` |
| **预期结果** | ① 报错提示需要 path<br>② 优雅处理：输出 "Done: 0 files, 0 symbols"，exit 0 |
| **验证方法** | ① 命令返回非零退出码，stderr 包含明确错误信息<br>② 命令退出码 0，输出提示无文件可索引 |

---

## 模块 B：查询类（Query）

---

### CLI-6 Status 查看索引状态

| 字段 | 内容 |
|------|------|
| **名称** | status — 查看 leveldb 索引状态 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl status --repo leveldb` |
| **预期结果** | 输出包含：Symbols: 3359, Edges: 6487, Docs: 63, Vectors, FTS5, DB size: 13.7 MB（解析率 97%） |
| **验证方法** | 所有核心指标（Symbols/Edges/Docs/FTS5/DB size）均显示且值 > 0 |

### CLI-7 BM25 关键词搜索

| 字段 | 内容 |
|------|------|
| **名称** | search — BM25 精确关键词搜索 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl search "compaction" --repo leveldb --branch master --limit 5` |
| **预期结果** | 返回 5 条结果，包含 Compaction 类、CompactionState 等与 compaction 相关的符号 |
| **验证方法** | 结果列表每项包含 `[类型] 符号名 @ 文件路径`，数量 <= limit |

### CLI-8 Search 按类型过滤

| 字段 | 内容 |
|------|------|
| **名称** | search — 使用 `--kind` 按符号类型过滤 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl search "DB" --repo leveldb --branch master --kind method --limit 3` |
| **预期结果** | 仅返回 method 类型的 DB::Open、DB::Get 等方法符号，不含 class 类型 |
| **验证方法** | 所有结果行首均为 `[method]` |

### CLI-9 语义搜索

| 字段 | 内容 |
|------|------|
| **名称** | semantic — 自然语言语义搜索（INT8 量化，4× 压缩） |
| **前置条件** | leveldb 已索引，向量模型已加载 |
| **步骤** | `cl semantic "键值对写入操作" --repo leveldb --branch master --limit 5` |
| **预期结果** | 返回与写入操作相关的符号，如 DB::Put、WriteBatch::Put、DBImpl::Write 等。相似度分数因 INT8 量化可能有 ±0.02 的精度损失，但对排序无影响 |
| **验证方法** | 结果按语义相关性排序，top-3 应包含 Put/Write 相关符号 |

### CLI-10 Overview 架构全貌

| 字段 | 内容 |
|------|------|
| **名称** | overview — 查看仓库符号分布全景 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl overview --repo leveldb --branch master` |
| **预期结果** | 输出带框线标题，列出所有符号类型的数量及百分比，如 method: 1324 (39.4%)、function: 848 (25.2%) 等 |
| **验证方法** | 12 种符号类型全部列出，百分比之和约为 100% |

### CLI-11 List-Symbols 模糊搜索

| 字段 | 内容 |
|------|------|
| **名称** | list-symbols — LIKE 模式匹配搜索符号名 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl list-symbols "DB::" --repo leveldb --branch master --limit 8` |
| **预期结果** | 返回 DB::Open、DB::Get、DB::Put、DB::Delete 等以 DB:: 开头的方法 |
| **验证方法** | 所有结果符号名均以 "DB::" 开头，数量 <= 8 |

### CLI-12 List-Symbols 无匹配

| 字段 | 内容 |
|------|------|
| **名称** | list-symbols — 模式无匹配时的行为 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl list-symbols "ZZZNotExistZZZ" --repo leveldb --branch master --limit 5` |
| **预期结果** | 输出空结果提示（如 "No symbols found" 或空列表），不报错 |
| **验证方法** | 退出码 0，无 panic 或 crash |

### CLI-13 Inspect 节点详情

| 字段 | 内容 |
|------|------|
| **名称** | inspect — 查看符号节点的完整定义和关联边 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl inspect "Compaction" --repo leveldb --branch master` |
| **预期结果** | 框线格式输出：Kind、Language、Namespace、File 路径、Documentation、Edges 列表 |
| **验证方法** | 输出以 ╔══/║/╚══ 框线装饰，包含至少 Kind 和 File 字段 |

### CLI-14 Call-Graph 被调用者（callees）

| 字段 | 内容 |
|------|------|
| **名称** | call-graph — 查看函数调用了谁（callees 方向） |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl call-graph "DB::Get" --repo leveldb --branch master --direction callees --max-depth 3` |
| **预期结果** | 树状结构输出 DB::Get 内部调用的函数链 |
| **验证方法** | 输出包含 "Call graph for 'DB::Get'" 标题，展示递归调用链 |

### CLI-15 Call-Graph 调用者（callers）

| 字段 | 内容 |
|------|------|
| **名称** | call-graph — 查看谁调用了该函数（callers 方向） |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl call-graph "DB::Open" --repo leveldb --branch master --direction callers --max-depth 2` |
| **预期结果** | 树状结构展示调用 DB::Open 的上级函数 |
| **验证方法** | direction 切换后输出与 callees 方向不同 |

### CLI-16 空结果搜索

| 字段 | 内容 |
|------|------|
| **名称** | search/semantic — 无匹配关键词/不存在的仓库 |
| **前置条件** | 任意已索引仓库 |
| **步骤** | ① `cl search "ZZZZNoMatchZZZZ" --repo leveldb --branch master`<br>② `cl search "compaction" --repo nonexistent-repo --branch main` |
| **预期结果** | ① 返回空结果("(no results)")，exit 0<br>② 优雅提示："Repo 'nonexistent-repo' not found."，exit 0 |
| **验证方法** | ① 退出码 0，输出 "(no results)" 或 "(none)"<br>② 退出码 0，输出仓库不存在提示 |

---

## 模块 C：管理类（Admin）

---

### CLI-17 List-Repos 列仓库

| 字段 | 内容 |
|------|------|
| **名称** | list-repos — 列出所有已索引仓库 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl list-repos` |
| **预期结果** | 输出 "Indexed repos:" 标题，每个仓库一行（含大小），如 `leveldb  35.6 MB` |
| **验证方法** | 包含至少一个仓库名（如 leveldb），总大小合理 |

### CLI-18 List-Branches 列分支

| 字段 | 内容 |
|------|------|
| **名称** | list-branches — 列出指定仓库的所有已索引分支 |
| **前置条件** | leveldb 已索引（至少 2 个分支：`__builtin__` 和 `master`） |
| **步骤** | `cl list-branches --repo leveldb` |
| **预期结果** | 列出所有分支名及符号数，如 `__builtin__  141 symbols`、`master  N symbols` |
| **验证方法** | 至少包含 master 分支，符号数与 status 一致 |


### CLI-21 Clean — 删除仓库

| 字段 | 内容 |
|------|------|
| **名称** | clean — 删除指定仓库的全部数据 |
| **前置条件** | leveldb-headers 已索引（CLI-3） |
| **步骤** | ① `cl clean --repo leveldb-headers`<br>② `cl status --repo leveldb-headers` |
| **预期结果** | ① 清理成功提示<br>② status 报错：仓库不存在（因其数据已删除） |
| **验证方法** | 清理后 `cl list-repos` 中不再有 leveldb-headers |

### CLI-22 Clean — 删除分支

| 字段 | 内容 |
|------|------|
| **名称** | clean — 删除指定仓库的特定分支 |
| **前置条件** | leveldb 有 master 分支（`__builtin__` 为 auto-generated，不可手动删除） |
| **步骤** | ① 创建一个测试分支：`cl index /mnt/d/RagMcpHermes/codeloom/tests/fixtures/branch_filter --repo leveldb --branch test-clean-branch`<br>② `cl clean --repo leveldb --branch test-clean-branch`<br>③ `cl list-branches --repo leveldb` |
| **预期结果** | ① 索引成功<br>② 清理成功提示<br>③ master 分支仍在，test-clean-branch 分支消失 |
| **验证方法** | list-branches 输出不再包含 test-clean-branch |

### CLI-23 Clean — 清空全部（确认提示）

| 字段 | 内容 |
|------|------|
| **名称** | clean — 清空所有数据（需确认） |
| **前置条件** | 至少有一个已索引仓库 |
| **步骤** | ① `echo 'y' | cl clean --all`<br>② `cl list-repos` |
| **预期结果** | ① 提示确认，输入 y 后清除所有数据<br>② 输出 "No indexed repos" |
| **验证方法** | clean 后重新索引 leveldb 以便后续测试 |

⚠️ **注意**：此测试会破坏所有索引数据。执行后务必重新索引：
```bash
cl index /mnt/d/code/leveldb --repo leveldb --branch master
```

### CLI-24 Check 环境检查

| 字段 | 内容 |
|------|------|
| **名称** | check — 检查运行环境和版本 |
| **前置条件** | 无 |
| **步骤** | `cl check` |
| **预期结果** | 输出版本号（codeloom 1.0.0）、二进制路径、运行环境等信息 |
| **验证方法** | 包含 "codeloom" 和版本号 |

### CLI-25 Completion 补全脚本生成

| 字段 | 内容 |
|------|------|
| **名称** | completion — 为 bash/zsh/fish 生成 Shell 补全 |
| **前置条件** | 无 |
| **步骤** | ① `cl completion bash`<br>② `cl completion zsh`<br>③ `cl completion fish`<br>④ `cl completion powershell` |
| **预期结果** | 每个命令输出对应 shell 的补全函数定义，包含子命令补全逻辑 |
| **验证方法** | bash 补全输出包含 `_codeloom()` 函数定义；zsh/fish 输出各自 shell 格式；不报错 |

### CLI-26 MCP Stdio 模式

| 字段 | 内容 |
|------|------|
| **名称** | mcp — stdio 模式启动 MCP JSON-RPC 服务 |
| **前置条件** | leveldb 已索引 |
| **步骤** | ① 在后台启动: `cl mcp`（等待就绪）<br>② 发送 JSON-RPC 请求: `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | cl mcp` |
| **预期结果** | ① 启动成功，无 error 日志<br>② 返回 MCP 工具列表，包含 codeloom_search、codeloom_overview 等 |
| **验证方法** | 响应包含 `"jsonrpc":"2.0"` 和工具名称数组 |

### CLI-27 MCP HTTP 模式

| 字段 | 内容 |
|------|------|
| **名称** | mcp — HTTP 模式启动 MCP 服务 |
| **前置条件** | leveldb 已索引 |
| **步骤** | ① 后台启动: `cl mcp --http 8080 &`<br>② `curl -X POST http://localhost:8080 -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'`<br>③ 测试完 kill 进程 |
| **预期结果** | ① 启动日志显示 HTTP listening on :8080<br>② curl 返回 MCP 工具列表 JSON<br>③ kill 后进程正常退出 |
| **验证方法** | HTTP 响应包含 `"jsonrpc":"2.0"` 和工具列表 |

### CLI-28 Update（可选，需网络）

| 字段 | 内容 |
|------|------|
| **名称** | update — GitHub 自动更新（标记为可选） |
| **前置条件** | 需能访问 github.com |
| **步骤** | `cl update` |
| **预期结果** | 检查最新 Release，如有更新则下载；如已最新则提示 "Already up to date" |
| **验证方法** | 输出包含检查版本的信息；失败时不 panic（如无网络则优雅提示） |

---

## 模块 D：端到端流程（E2E Flow）

---

### CLI-29 全流程：index → status → search → inspect → call-graph

| 字段 | 内容 |
|------|------|
| **名称** | 端到端 — 用 leveldb 完整走一遍核心管线 |
| **前置条件** | 执行「测试编排」，leveldb 已索引 |

**步骤与预期结果：**

| # | 命令 | 预期结果 |
|---|------|---------|
| ① | `cl status --repo leveldb` | Symbols > 6000, Edges > 6000, Docs = 63 |
| ② | `cl search "compaction" --repo leveldb --branch master --limit 5` | 返回 Compaction 类、CompactionState 等 |
| ③ | `cl list-symbols "DB::" --repo leveldb --branch master --limit 8` | 返回 DB::Open、DB::Get、DB::Put 等 |
| ④ | `cl overview --repo leveldb --branch master` | 12 种符号类型分布，百分比之和 ≈ 100% |
| ⑤ | `cl inspect "Version" --repo leveldb --branch master` | 框线格式输出，含 Kind、File、Edges |
| ⑥ | `cl call-graph "DB::Get" --repo leveldb --branch master --direction callees` | 树状展示 DB::Get 内部调用链 |

**验证方法：**
- 步骤①-⑥全部成功，无 panic 或 crash
- 步骤②的搜索结果应包含 `[class] Compaction` 或 `[method] CompactionState::NewBoundary`
- 步骤③的所有结果均以 `DB::` 开头
- 步骤⑥输出包含 "Call graph for 'DB::Get'" 标题

---

### CLI-30 leveldb 多分支搜索对比

| 字段 | 内容 |
|------|------|
| **名称** | 多分支 — leveldb master 分支不同维度搜索对比 |
| **前置条件** | leveldb 已索引 |

**步骤与预期结果：**

| # | 命令 | 预期结果 |
|---|------|---------|
| ① | `cl search "write" --repo leveldb --branch master --limit 5 --kind method` | 返回 Write、WriteBatch::Put 等 method |
| ② | `cl search "write" --repo leveldb --branch master --limit 5 --kind class` | 返回 WriteBatch 等 class 类型，与①不同 |
| ③ | `cl semantic "键值对存储" --repo leveldb --branch master --limit 5` | 返回 DB::Put、WriteBatch、DBImpl::Write 等 |

**验证方法：**
- ①和②结果不同，体现 kind 过滤效果
- ③的语义搜索结果与关键词搜索结果有差异，体现向量搜索的优势

---

### CLI-31 include 边验证

| 字段 | 内容 |
|------|------|
| **名称** | include — 验证 leveldb 索引中 `#include` 边的正确性 |
| **前置条件** | leveldb 已索引 |

**步骤与预期结果：**

| # | 命令 | 预期结果 |
|---|------|---------|
| ① | `cl status --repo leveldb` | 输出中包含 `Includes: N edges` 且 N > 0 |

**验证方法：**
- status 输出显示 `Includes: 778 edges`（leveldb 的 #include 关系数量）
- 确保所有 #include 边正常入库，无 panic

---

## 附录：清理脚本

```bash
# 清理本次测试创建的临时索引
cl clean --repo leveldb-headers
```

## 测试记录表 (INT8 迁移验证 — 2026-05-13)

|| CLI-1 | ✅ | 2026-05-13 | 梦璃 | 全量索引 + INT8 表验证 |
|| CLI-2 | ✅ | 2026-05-13 | 梦璃 | 增量索引 |
|| CLI-3 | ✅ | 2026-05-13 | 梦璃 | parent 继承 |
|| CLI-4 | ✅ | 2026-05-13 | 梦璃 | 非 Git 目录 + INT8 语义搜索 |
|| CLI-5 | ✅ | 2026-05-13 | 梦璃 | 5a CWD索引 ✓ 5b 不存在路径优雅处理 ✓ |
|| CLI-6 | ✅ | 2026-05-13 | 梦璃 | status (3364/3177/63 ✅) |
|| CLI-7 | ✅ | 2026-05-13 | 梦璃 | search compaction |
|| CLI-8 | ✅ | 2026-05-13 | 梦璃 | kind=method 过滤正确 |
|| CLI-9 | ✅ | 2026-05-13 | 梦璃 | 语义搜索 INT8 (WriteBatch 0.7165 ✅) |
|| CLI-10 | ✅ | 2026-05-13 | 梦璃 | overview (12 types) |
|| CLI-11 | ✅ | 2026-05-13 | 梦璃 | list-symbols DB:: |
|| CLI-12 | ✅ | 2026-05-13 | 梦璃 | 无匹配优雅处理 |
|| CLI-13 | ✅ | 2026-05-13 | 梦璃 | inspect Compaction 框线格式 |
|| CLI-14 | ❌ | 2026-05-13 | 梦璃 | --depth 不被 CLI call-graph 支持 |
|| CLI-15 | ❌ | 2026-05-13 | 梦璃 | --depth 不被 CLI call-graph 支持 |
|| CLI-17 | ✅ | 2026-05-13 | 梦璃 | list-repos |
|| CLI-18 | ✅ | 2026-05-13 | 梦璃 | list-branches |
|| CLI-21 | ✅ | 2026-05-13 | 梦璃 | clean --repo |
|| CLI-22 | ✅ | 2026-05-13 | 梦璃 | clean --branch |
|| CLI-24 | ✅ | 2026-05-13 | 梦璃 | check |
|| CLI-25 | ✅ | 2026-05-13 | 梦璃 | completion bash |
|| CLI-26 | ✅ | 2026-05-13 | 梦璃 | MCP stdio (tools list) |
|| CLI-27 | ✅ | 2026-05-13 | 梦璃 | MCP HTTP (启动正常) |
|| CLI-28 | ⬜ | | | 可选，需网络 |
|| CLI-29 | ✅ | 2026-05-13 | 梦璃 | E2E 完整流程 |
|| CLI-30 | ✅ | 2026-05-13 | 梦璃 | 跨仓搜索 |

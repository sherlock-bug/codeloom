# CodeLoom CLI 端到端手工测试用例

> **版本**: codeloom 1.0.0
> **二进制**: `/mnt/d/RagMcpHermes/codeloom/target/release/codeloom`
> **测试仓库**: leveldb (C++，~130文件，已索引)、spdlog、flatbuffers
> **测试夹具**: `/mnt/d/RagMcpHermes/codeloom/tests/fixtures/`
> **Writer**: Manual QA
> **Updated**: 2026-05-12

---

## 测试前准备

```bash
# 设置别名，统一使用
alias cl=/mnt/d/RagMcpHermes/codeloom/target/release/codeloom

# 验证二进制可用
cl --version
# 预期: codeloom 1.0.0

# 确保 leveldb 已索引（测试基座）
cl status --repo leveldb
# 预期: 符号数 > 0，DB size 合理
```

---

## 模块 A：索引类（Index）

---

### CLI-1 全量索引新仓库（fixtures/clang_test）

| 字段 | 内容 |
|------|------|
| **名称** | 全量索引 — 对 tests/fixtures/clang_test 新仓库做首次全量索引 |
| **前置条件** | clang_test 未在 codeloom 中索引过（可用 `cl clean --repo clang-test` 确保干净） |
| **步骤** | ① `cl index /mnt/d/RagMcpHermes/codeloom/tests/fixtures/clang_test --repo clang-test --branch main` |
| **预期结果** | 索引完成，输出包含符号数 > 0，无 error 日志；返回时间 < 30s |
| **验证方法** | `cl status --repo clang-test` 输出 Symbols > 0, Edges > 0, DB size > 0 |

### CLI-2 增量索引（已索引仓库重复索引）

| 字段 | 内容 |
|------|------|
| **名称** | 增量索引 — 对已索引仓库重新执行 index，验证 delta 更新而非全量 |
| **前置条件** | clang-test 已索引（CLI-1 后） |
| **步骤** | ① `cl index /mnt/d/RagMcpHermes/codeloom/tests/fixtures/clang_test --repo clang-test --branch main` |
| **预期结果** | 索引快速完成（增量扫描，时间 < 全量 1/3），不报重复键错误 |
| **验证方法** | 第二次执行应更快返回；status 符号数不变或合理增加 |

### CLI-3 Parent 继承索引

| 字段 | 内容 |
|------|------|
| **名称** | Parent继承索引 — 使用 `--parent` 指定上游分支继承符号 |
| **前置条件** | clang-test 已索引到 main 分支 |
| **步骤** | ① `cl index /mnt/d/RagMcpHermes/codeloom/tests/fixtures/branch_filter --repo clang-test --branch feature-x --parent main` |
| **预期结果** | 索引成功，feature-x 分支继承 main 的符号 |
| **验证方法** | `cl status --repo clang-test` 显示符号数 > 0（feature-x 继承自 main） |

### CLI-4 非 Git 目录索引

| 字段 | 内容 |
|------|------|
| **名称** | 非 Git 目录索引 — 对纯文件目录（不含 .git）执行 index |
| **前置条件** | 夹具目录可用 |
| **步骤** | ① `cl index /mnt/d/RagMcpHermes/codeloom/tests/fixtures/edge_types --repo edge-test --branch main` |
| **预期结果** | 索引成功，输出符号数 >= 5（enum Status + 成员 + class AuthService + 字段 + 全局变量） |
| **验证方法** | `cl status --repo edge-test` 显示 Symbols > 0 |

### CLI-5 索引缺参数/错误路径

| 字段 | 内容 |
|------|------|
| **名称** | 索引边界 — 缺 path、不存在的路径 |
| **步骤** | ① `cl index`（无参数，无默认仓库）<br>② `cl index /nonexistent/path --repo bad-repo --branch main` |
| **预期结果** | ① 报错提示需要 path 或当前目录无可索引内容<br>② 优雅处理：输出 "Done: 0 files, 0 symbols"，exit 0 |
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
| **预期结果** | 输出包含：Symbols: 3359, Edges: 5177, Docs: 26, Vectors, FTS5, DB size: 35.6 MB |
| **验证方法** | 所有核心指标（Symbols/Edges/Docs/FTS5/DB size）均显示且值 > 0 |

### CLI-7 BM25 关键词搜索

| 字段 | 内容 |
|------|------|
| **名称** | search — BM25 精确关键词搜索 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl search "compaction" --repo leveldb --branch main1 --limit 5` |
| **预期结果** | 返回 5 条结果，包含 Compaction 类、CompactionState 等与 compaction 相关的符号 |
| **验证方法** | 结果列表每项包含 `[类型] 符号名 @ 文件路径`，数量 <= limit |

### CLI-8 Search 按类型过滤

| 字段 | 内容 |
|------|------|
| **名称** | search — 使用 `--kind` 按符号类型过滤 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl search "DB" --repo leveldb --branch main1 --kind method --limit 3` |
| **预期结果** | 仅返回 method 类型的 DB::Open、DB::Get 等方法符号，不含 class 类型 |
| **验证方法** | 所有结果行首均为 `[method]` |

### CLI-9 语义搜索

| 字段 | 内容 |
|------|------|
| **名称** | semantic — 自然语言语义搜索 |
| **前置条件** | leveldb 已索引，向量模型已加载 |
| **步骤** | `cl semantic "键值对写入操作" --repo leveldb --branch main1 --limit 5` |
| **预期结果** | 返回与写入操作相关的符号，如 DB::Put、WriteBatch::Put、DBImpl::Write 等 |
| **验证方法** | 结果按语义相关性排序，top-3 应包含 Put/Write 相关符号 |

### CLI-10 Overview 架构全貌

| 字段 | 内容 |
|------|------|
| **名称** | overview — 查看仓库符号分布全景 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl overview --repo leveldb --branch main1` |
| **预期结果** | 输出带框线标题，列出所有符号类型的数量及百分比，如 method: 1324 (39.4%)、function: 730 (21.7%) 等 |
| **验证方法** | 12 种符号类型全部列出，百分比之和约为 100% |

### CLI-11 List-Symbols 模糊搜索

| 字段 | 内容 |
|------|------|
| **名称** | list-symbols — LIKE 模式匹配搜索符号名 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl list-symbols "DB::" --repo leveldb --branch main1 --limit 8` |
| **预期结果** | 返回 DB::Open、DB::Get、DB::Put、DB::Delete 等以 DB:: 开头的方法 |
| **验证方法** | 所有结果符号名均以 "DB::" 开头，数量 <= 8 |

### CLI-12 List-Symbols 无匹配

| 字段 | 内容 |
|------|------|
| **名称** | list-symbols — 模式无匹配时的行为 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl list-symbols "ZZZNotExistZZZ" --repo leveldb --branch main1 --limit 5` |
| **预期结果** | 输出空结果提示（如 "No symbols found" 或空列表），不报错 |
| **验证方法** | 退出码 0，无 panic 或 crash |

### CLI-13 Inspect 节点详情

| 字段 | 内容 |
|------|------|
| **名称** | inspect — 查看符号节点的完整定义和关联边 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl inspect "Compaction" --repo leveldb --branch main1` |
| **预期结果** | 框线格式输出：Kind、Language、Namespace、File 路径、Documentation、Edges 列表 |
| **验证方法** | 输出以 ╔══/║/╚══ 框线装饰，包含至少 Kind 和 File 字段 |

### CLI-14 Call-Graph 被调用者（callees）

| 字段 | 内容 |
|------|------|
| **名称** | call-graph — 查看函数调用了谁（callees 方向） |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl call-graph "DB::Get" --repo leveldb --branch main1 --direction callees --max-depth 3` |
| **预期结果** | 树状结构输出 DB::Get 内部调用的函数链 |
| **验证方法** | 输出包含 "Call graph for 'DB::Get'" 标题，展示递归调用链 |

### CLI-15 Call-Graph 调用者（callers）

| 字段 | 内容 |
|------|------|
| **名称** | call-graph — 查看谁调用了该函数（callers 方向） |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl call-graph "DB::Open" --repo leveldb --branch main1 --direction callers --max-depth 2` |
| **预期结果** | 树状结构展示调用 DB::Open 的上级函数 |
| **验证方法** | direction 切换后输出与 callees 方向不同 |

### CLI-16 空结果搜索

| 字段 | 内容 |
|------|------|
| **名称** | search/semantic — 无匹配关键词/不存在的仓库 |
| **前置条件** | 任意已索引仓库 |
| **步骤** | ① `cl search "ZZZZNoMatchZZZZ" --repo leveldb --branch main1`<br>② `cl search "compaction" --repo nonexistent-repo --branch main` |
| **预期结果** | ① 返回空结果("(no results)")，exit 0<br>② 优雅提示："Repo 'nonexistent-repo' not found."，exit 0 |
| **验证方法** | ① 退出码 0，输出 "(no results)" 或 "(none)"<br>② 退出码 0，输出仓库不存在提示 |

---

## 模块 C：管理类（Admin）

---

### CLI-17 List-Repos 列仓库

| 字段 | 内容 |
|------|------|
| **名称** | list-repos — 列出所有已索引仓库 |
| **前置条件** | leveldb 已索引；若 spdlog 也已索引则更好 |
| **步骤** | `cl list-repos` |
| **预期结果** | 输出 "Indexed repos:" 标题，每个仓库一行（含大小），如 `leveldb  35.6 MB` |
| **验证方法** | 包含至少一个仓库名（如 leveldb），总大小合理 |

### CLI-18 List-Branches 列分支

| 字段 | 内容 |
|------|------|
| **名称** | list-branches — 列出指定仓库的所有已索引分支 |
| **前置条件** | leveldb 已索引（至少 2 个分支：__builtin__ 和 main1） |
| **步骤** | `cl list-branches --repo leveldb` |
| **预期结果** | 列出所有分支名及符号数，如 `__builtin__  141 symbols`、`main1  3359 symbols` |
| **验证方法** | 至少包含 main1 分支，符号数与 status 一致 |


### CLI-21 Clean — 删除仓库

| 字段 | 内容 |
|------|------|
| **名称** | clean — 删除指定仓库的全部数据 |
| **前置条件** | edge-test 已索引（CLI-4） |
| **步骤** | ① `cl clean --repo edge-test`<br>② `cl status --repo edge-test` |
| **预期结果** | ① 清理成功提示<br>② status 报错：仓库不存在（因其数据已删除） |
| **验证方法** | 清理后 `cl list-repos` 中不再有 edge-test |

### CLI-22 Clean — 删除分支

| 字段 | 内容 |
|------|------|
| **名称** | clean — 删除指定仓库的特定分支 |
| **前置条件** | leveldb 有 main1 分支（`__builtin__` 为 auto-generated，不可手动删除） |
| **步骤** | ① 创建一个测试分支：`cl index /mnt/d/RagMcpHermes/codeloom/tests/fixtures/branch_filter --repo leveldb --branch test-clean-branch`<br>② `cl clean --repo leveldb --branch test-clean-branch`<br>③ `cl list-branches --repo leveldb` |
| **预期结果** | ① 索引成功<br>② 清理成功提示<br>③ main1 分支仍在，test-clean-branch 分支消失 |
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
cl index /mnt/d/code/leveldb --repo leveldb --branch main1
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
| **名称** | 端到端 — 用 tests/fixtures/clang_test 新仓库执行完整流程 |
| **前置条件** | clang_test 未索引（如已存在先 `cl clean --repo clang-test-e2e`） |

**步骤与预期结果：**

| # | 命令 | 预期结果 |
|---|------|---------|
| ① | `cl index /mnt/d/RagMcpHermes/codeloom/tests/fixtures/clang_test --repo clang-test-e2e --branch main` | 索引成功，符号数 > 0 |
| ② | `cl status --repo clang-test-e2e` | 显示 Symbols、Edges、FTS5、DB size 均 > 0 |
| ③ | `cl search "sample" --repo clang-test-e2e --branch main --limit 5` | 命中 sample 相关符号 |
| ④ | `cl list-symbols "sample" --repo clang-test-e2e --branch main --limit 5` | 返回符号名包含 "sample" 的结果 |
| ⑤ | `cl overview --repo clang-test-e2e --branch main` | 输出符号按类型分布百分比 |
| ⑥ | `cl list-repos` | 包含 clang-test-e2e |
| ⑦ | 清理：`cl clean --repo clang-test-e2e` | 清理成功 |

**验证方法**：
- 步骤①-⑧全部成功，无 panic 或 crash
- 步骤③与步骤④结果不同（search 是 BM25，list-symbols 是 LIKE）
- 步骤⑦即使无调用链也输出 "no callees found" 而不是崩溃
- 清理：`cl clean --repo flatbuffers`

---

### CLI-30 跨仓库对比搜索

| 字段 | 内容 |
|------|------|
| **名称** | 端到端 — 同关键词在不同仓库搜索对比 |
| **前置条件** | leveldb 和 clang-test（CLI-1 后）均已索引 |

**步骤与预期结果：**

| # | 命令 | 预期结果 |
|---|------|---------|
| ① | `cl list-repos` | 同时包含 leveldb 和 clang-test |
| ② | `cl search "write" --repo leveldb --branch main1 --limit 5` | 返回 Write、WriteBatch、DB::Put 等 |
| ③ | `cl search "sample" --repo clang-test --branch main --limit 5` | 返回 clang_test 的 sample 相关符号 |

**验证方法**：
- ②和③结果不同，体现仓库特异性
- spdlog 的 search 结果应包含 logger/sink 相关符号

---

## 附录：清理脚本

```bash
# 清理本次测试创建的所有临时索引
cl clean --repo clang-test
cl clean --repo clang-test-e2e
cl clean --repo edge-test
cl clean --repo spdlog
```

## 测试记录表

|| CLI-1 | ✅ | 2026-05-12 | 梦璃 | 全量索引 clang-test |
| CLI-2 | ✅ | 2026-05-12 | 梦璃 | 增量索引 |
| CLI-3 | ✅ | 2026-05-12 | 梦璃 | parent 继承 |
| CLI-4 | ✅ | 2026-05-12 | 梦璃 | 非 Git 目录 |
| CLI-5 | ✅ | 2026-05-12 | 梦璃 | 5a 无参数报错 ✓ 5b 不存在路径优雅处理 ✓ |
| CLI-6 | ✅ | 2026-05-12 | 梦璃 | status 正常 |
| CLI-7 | ✅ | 2026-05-12 | 梦璃 | search compaction |
| CLI-8 | ✅ | 2026-05-12 | 梦璃 | kind=method 过滤正确 |
| CLI-9 | ✅ | 2026-05-12 | 梦璃 | 语义搜索 |
| CLI-10 | ✅ | 2026-05-12 | 梦璃 | overview |
| CLI-11 | ✅ | 2026-05-12 | 梦璃 | list-symbols |
| CLI-12 | ✅ | 2026-05-12 | 梦璃 | 无匹配 |
| CLI-13 | ✅ | 2026-05-12 | 梦璃 | inspect Compaction（MCP enrichment ✅ CLI展示格式不同） |
| CLI-14 | ❌ | 2026-05-12 | 梦璃 | --depth 不被 CLI call-graph 支持（clap 解析问题） |
| CLI-15 | ❌ | 2026-05-12 | 梦璃 | --depth 不被 CLI call-graph 支持 |
| CLI-17 | ✅ | 2026-05-12 | 梦璃 | list-repos |
| CLI-18 | ✅ | 2026-05-12 | 梦璃 | list-branches |
| CLI-21 | ✅ | 2026-05-12 | 梦璃 | clean --repo |
| CLI-22 | ✅ | 2026-05-12 | 梦璃 | 创建+删除临时分支 |
| CLI-24 | ✅ | 2026-05-12 | 梦璃 | check |
| CLI-25 | ✅ | 2026-05-12 | 梦璃 | completion |
| CLI-26 | ✅ | 2026-05-12 | 梦璃 | MCP stdio 12 tools |
| CLI-27 | ✅ | 2026-05-12 | 梦璃 | MCP HTTP 启动正常 |
| CLI-28 | ⬜ | | | 可选，需网络 |
| CLI-29 | ✅ | 2026-05-12 | 梦璃 | E2E 主流程（索引→搜索→inspect→清理） |
| CLI-30 | ✅ | 2026-05-12 | 梦璃 | 跨仓搜索 |

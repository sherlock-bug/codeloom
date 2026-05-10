# Design: Clang 核心解析器

## Context

CodeLoom 当前使用 tree-sitter 解析 C/C++ 代码，存在根本性局限：
- **无法处理宏展开**：`#define` 宏定义的函数/常量不会被识别
- **无法处理模板实例化**：`std::vector<int>` 等实例化的成员函数调用无法追踪
- **无法消歧重载**：`foo(int)` 和 `foo(double)` 被当作同一个符号
- **无法跨文件解析类型**：类型推断依赖当前文件上下文
- **调用边精度极低**：目标符号在 DB 内的比例仅 0-14%

Clang 作为编译器前端（执行完整的词法分析→语法分析→语义分析→AST 构建）可以完整解决这些问题。

当前技术栈：Rust（codeloom 本体）、SQLite（存储，含 FTS5/vec0）、tree-sitter（解析引擎，将被替换）。

## Goals / Non-Goals

**Goals:**
- 用 Clang 子进程替换 tree-sitter 解析 C/C++ 代码，获得完整的语义信息（宏展开、模板实例化、重载消歧、跨文件类型解析）
- codeloom 本体 musl 静态编译，无 libclang 链接依赖
- 扩展数据模型以承载 Clang 提供的丰富信息
- 外部符号按需创建 stub 节点

**Non-Goals:**
- 不同时支持 tree-sitter 和 Clang（直接替换，新分支独立开发）
- 不做 C/C++ 以外语言（Python/Java 等保持 tree-sitter）
- 不做增量索引（本阶段全量重建）
- 不做 LSP 集成
- 不做存储优化（int8 量化等，放后续阶段）

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  CodeLoom (musl 静态编译, ~6MB)  │
│                                                  │
│  ┌──────────────┐    ┌──────────────────────┐   │
│  │ indexer/mod  │───▶│  indexer/clang/       │   │
│  │ (语言路由)    │    │  ├── mod.rs           │   │
│  └──────────────┘    │  │   (spawn clang,    │   │
│                       │  │    parse JSON AST) │   │
│  ┌──────────────┐    │  ├── ast.rs           │   │
│  │ storage/     │◀───│  │   (AST JSON →      │   │
│  │ (写入 DB)    │    │  │    Symbol + Edge)   │   │
│  └──────────────┘    │  └── compile_cmds.rs   │   │
│                       │     (compile_commands  │   │
│                       │      发现与解析)        │   │
│                       └──────────────────────┘   │
│                              │                   │
│                              ▼ (子进程)          │
│                       ┌──────────────┐           │
│                       │   clang       │           │
│                       │ -fsyntax-only │           │
│                       │ -ast-dump=json│           │
│                       └──────────────┘           │
└─────────────────────────────────────────────────┘
```

核心流程：
1. 收集 C/C++ 源文件列表
2. 发现/加载 `compile_commands.json`，解析每个翻译单元的编译参数
3. 对每个翻译单元执行 `clang -fsyntax-only -ast-dump=json -- <编译参数> <源文件>`
4. 用 `serde_json` 流式解析 JSON AST，遍历 TranslationUnitDecl 节点
5. 提取：FunctionDecl、CXXMethodDecl、CXXRecordDecl、EnumDecl、FieldDecl、VarDecl、NamespaceDecl、ClassTemplateDecl、FunctionTemplateDecl、ClassTemplateSpecializationDecl
6. 从每个声明节点提取：调用关系（CallExpr→DeclRefExpr）、继承关系（CXXRecordDecl::bases）、类型关系（ParmVarDecl::type、FunctionDecl::returnType）、模板实例化关系、include 关系
7. 外部分辨率：遇到非本项目的符号引用时，标记 is_external=TRUE，kind 存真实类型
8. 写入 SQLite（symbols + edges）

## Data Model

### 节点类型（15 种）

| 序号 | kind | 来源 | 说明 |
|------|------|------|------|
| 1 | function | 原有 | 自由函数 |
| 2 | method | 原有 | 类成员函数 |
| 3 | class | 原有 | 类声明 |
| 4 | struct | 原有 | 结构体声明 |
| 5 | enum | 原有 | 枚举声明 |
| 6 | enum_value | 原有 | 枚举值 |
| 7 | field | 原有 | 类/结构体字段 |
| 8 | global | 原有 | 全局变量 |
| 9 | static_var | 原有 | 静态变量 |
| — | variable | **删除** | 局部变量对调用图无价值 |
| 11 | template_function | 原有 | 函数模板 |
| 12 | macro | 原有 | 宏定义（Clang 可提取展开前的宏名） |
| 13 | namespace | **新增** | 具名命名空间 |
| 14 | template_instance | **新增** | 本项目模板的实例化（如 `MyVec<int>`） |
| — | external_symbol | **删除** | 改用 `is_external` 列 + 复用 kind |

### 符号表新增列（6 列）

| 列名 | 类型 | 说明 |
|------|------|------|
| sid | TEXT | 稳定唯一ID（SHA256前16位），跨重建不变 |
| access | TEXT | public / protected / private（仅 method、field 有值，其余 NULL） |
| is_virtual | BOOL | 虚函数标记（仅 method 有值，其余 0） |
| is_definition | BOOL | TRUE=定义（有函数体），FALSE=纯声明 |
| is_external | BOOL | TRUE=外部符号，复用 kind 存真实类型 |
| template_args | TEXT | 模板参数 JSON 数组（仅 template_function 和 template_instance 有值） |

已有列说明：
- `namespace`（TEXT）：Clang 填充完整命名空间路径
- `signature`（TEXT）：包含 const/static/explicit 等修饰符
- `doc_comment`（TEXT）：声明和定义合并时拼接，注释不丢失

删掉的列及其替代：
- `parent_class` → `contains:` 边（class→method/field）
- `is_const` / `is_static` / `is_explicit` → signature 已包含
- `return_type` → `return_type:` 边

### 边类型（11 种）

| 序号 | edge_type 前缀 | 来源 | 说明 |
|------|---------------|------|------|
| 1 | calls: | 原有 | A 调用 B（外部通过 target.is_external 区分） |
| 2 | inherits: | 原有 | A 继承 B |
| 3 | overrides: | **合并** | A 覆写 B 的虚函数 |
| 4 | instantiates: | **新增** | template_instance → 主模板 |
| 5 | param_type: | **新增** | 函数参数 → 参数类型 |
| 6 | return_type: | **新增** | 函数 → 返回值类型 |
| 7 | includes: | **新增** | 文件 include 关系 |
| 8 | uses_type: | **新增** | 变量/字段 → 其声明类型 |
| 9 | contains: | **新增** | class/struct → 其方法/字段（替代 parent_class 列） |
| 10 | aliases: | **新增** | typedef → 底层类型 |
| 11 | uses: | **恢复** | function→global/static_var/enum_value/string_literal 依赖 |

删掉的边：`uses:`（旧版）、`references:`、`dataflow:`、`calls_external:`。

## Decisions

### Decision 1: 子进程方案（B1）

**选择**：`clang -fsyntax-only -ast-dump=json` 子进程

**拒绝的方案**：
- **libclang FFI**：需要链接 libclang.so，glibc 版本依赖，静态链接使二进制膨胀至 100MB+
- **捆绑 musl-clang**：clang 二进制本身 100MB+，打包后 codeloom 不再是小工具

**理由**：
- 生产环境已有 clang 19.1.7，无需额外安装
- 子进程通过 `compile_commands.json` 获取编译参数，零运行时依赖
- codeloom 本体保持 musl 静态编译 (~6MB)，纯净无链接依赖
- `-ast-dump=json` 输出结构稳定（Clang 15+ 版本基本不变）

### Decision 2: musl 静态编译

**选择**：codeloom 本体用 musl 静态编译

**理由**：
- 不依赖系统 glibc 版本（公司环境 glibc 2.28，老旧）
- 单一二进制文件，部署简单
- `clang` 子进程不受影响（它用自己的动态链接）

### Decision 3: compile_commands.json 自动发现

**选择**：三级搜索，用户可显式指定覆盖

**搜索顺序**：
1. 用户通过 `--compile-commands <path>` 显式指定
2. 项目根目录下的 `compile_commands.json`
3. `build/compile_commands.json`
4. `out/compile_commands.json`

**理由**：
- CMake 默认生成在 `build/`，Bazel 通过工具生成，Meson 在 `builddir/`
- 覆盖大多数构建系统
- 显式指定路径给非标准布局

### Decision 4: AST JSON 流式解析

**选择**：用 `serde_json::StreamDeserializer` 逐 token 解析，不完全加载整个 JSON 树

**理由**：
- LevelDB 的 AST JSON 单文件可达 50MB+，全量加载会 OOM
- 流式解析只保留当前正在处理的节点上下文
- 延迟构建符号：先在内存中积累到一个事务边界再批量写入

### Decision 5: 外部符号用 is_external 列标识

**选择**：外部符号不创建独立节点类型，用 `is_external` 布尔列 + 复用 `kind` 存真实类型

**判定逻辑**：
- DeclRefExpr 指向的 Decl 所在文件不在 `compile_commands.json` 覆盖范围内 → is_external=TRUE
- kind 存 Clang 原始类型（function/class/struct/template_function 等）
- file_path 留空，line_start/line_end=0，content_hash="ext:<hash>"
- 按 (name, namespace, kind) 去重

**理由**：
- 外部函数和外部类不再混在同一个 external_symbol kind 下
- 查询"所有外部类"：`WHERE kind='class' AND is_external=1`，无需新工具
- 外部调用用普通 `calls:` 边，通过 target.is_external 区分

### Decision 6: 模板实例化收敛

**选择**：只对**本项目定义的模板**的实例化创建 `template_instance` 节点

**实现**：
- AST 中遇到 `ClassTemplateSpecializationDecl` 时，检查主模板是否在本项目的源文件中定义
- 是 → 创建 `template_instance` 节点 + `instantiates:` 边指向主模板
- 否（如 `std::vector<int>`）→ 跳过，通过 calls: 边 + target.is_external=1 处理

### Decision 7: 声明与定义合并为单节点

**选择**：同一符号的声明（头文件）和定义（源文件）合并为同一个 symbols 行，不拆分。注释合并，is_definition 以定义为准（TRUE）。

**被拒绝的方案**：拆成两个节点 + resolves_to 边。

**合并逻辑**：
1. 匹配条件：`name` + `namespace` + `signature`（或 `name` + `namespace` + `kind`）相同
2. 先到声明 → 创建节点（is_definition=FALSE，保留声明注释）
3. 后到定义 → 更新同一行：`is_definition=TRUE`，`file_path`/`line_start`/`line_end` 更新为定义位置，`doc_comment` 拼接（声明注释 + 定义注释，用换行分隔）
4. 先到定义 → 创建节点（is_definition=TRUE），后到声明 → 仅拼接注释，不更新位置
5. `content_hash` 每次更新后重算

**理由**：
- CodeLoom 核心场景是 LLM 查调用图，非 IDE 导航。"跳转到声明"是 IDE 的事
- 减少节点冗余（一个符号两行 → 一行），查询更简单
- 减少边种类（少一个 resolves_to）
- 注释合并保证了声明处的文档注释不丢失

### Decision 8: 稳定唯一 ID（sid）

**选择**：新增 `sid TEXT` 列，值为 `SHA256(repo + name + signature + namespace + kind)` 前 16 字符

**理由**：
- 自增 `id` 在重新索引后变化，同名符号无法被 MCP 工具精确引用
- sid 跨重建稳定，可写入 LLM 上下文作为引用锚点
- MCP 工具返回 sid 后，LLM 用 sid 精确查询单符号

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 大型项目 AST JSON 解析内存峰值可能达数百 MB | 流式解析 + 批量提交，实测后加内存限制 |
| `clang` 版本差异导致 AST JSON 字段变化 | 仅依赖 Clang 15+ 的稳定字段，必要时加版本探测 |
| 无 `compile_commands.json` 的项目无法索引 | 单文件模式降级：`clang -fsyntax-only file.cpp --`（无额外编译参数） |
| 子进程 I/O 开销（spawn clang × N 文件） | 按翻译单元批量处理，一个编译命令覆盖多个源文件 |
| 新分支 schema 与旧版不兼容 | 独立分支开发，不做向后兼容迁移 |

## Migration Plan

1. 从 master 创建 `clang-parser` 分支
2. 新增 `src/indexer/clang/` 模块，实现子进程调用 + AST 解析
3. 修改 `src/storage/schema.rs`：新增列、新增表（`clang_meta` 等）
4. 修改 `src/indexer/mod.rs`：C/C++ 路由到 clang 模块
5. 移除 `src/indexer/queries/cpp.rs`（tree-sitter C++ 查询）
6. 更新 `src/mcp/mod.rs`：工具描述中的符号/边类型
7. 测试 + 验证 → 合并到 master（届时 master 的 tree-sitter 路径已被移除）

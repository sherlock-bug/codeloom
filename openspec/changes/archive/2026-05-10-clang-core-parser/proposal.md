# Proposal: Clang 核心解析器

## Why

当前 CodeLoom 使用 tree-sitter 解析 C/C++ 代码，无法处理宏展开、模板实例化、重载消歧，导致调用边精度极低（目标在 DB 内的比例仅 0-14%），且无法区分声明/定义、跨文件解析类型、识别外部库符号。Clang 作为编译器前端可以完整解决这些问题，使 CodeLoom 的符号表和调用图从"猜测"升级到"确定"。

## What Changes

- **替换** C/C++ 解析器：tree-sitter → Clang 子进程（`clang -fsyntax-only -ast-dump=json`），不再保留 tree-sitter C/C++ 路径
- **新增** `compile_commands.json` 自动发现（项目根目录 → build/ → out/ 逐级搜索）
- **扩展** 数据模型：新增 4 种节点（namespace, template_instance, typedef, string_literal）；删 variable；删 parent_class 列（用 contains: 边）；删 external_symbol 节点（用 is_external 列）；新增 sid 列
- **精简** 边体系：删 references/dataflow/calls_external；新增 instantiates/param_type/return_type/includes/uses_type/contains/aliases；恢复 uses（函数→全局/静态/枚举/字符串依赖）；合并 overrides
- **扩展** 符号表：新增 6 列（sid, access, is_virtual, is_definition, is_external, template_args）；删 parent_class
- **新增** 外部符号按需 stub 策略：遇到本项目引用的外部符号自动创建轻量节点，按 (name, namespace) 去重
- **新增** 独立分支开发，数据库 schema 重新设计，不兼容旧版
- **目标**：codeloom 本体 musl 静态编译（6MB），运行时调系统 `clang` 子进程。无需链接 libclang

## Capabilities

### New Capabilities

- `clang-subprocess-parser`: 调用系统 `clang` 子进程，解析 `-ast-dump=json` 输出，提取 C/C++ 符号和关系边
- `compile-commands-discovery`: 自动搜索项目中的 `compile_commands.json`，支持默认路径和显式指定
- `extended-node-types`: 新增 namespace/template_instance/typedef/string_literal 四种节点；删 variable；外部符号用 is_external 列
- `extended-edge-types`: 删 references/dataflow/calls_external；新增 instantiates/param_type/return_type/includes/uses_type/contains/aliases；恢复 uses（函数依赖全局/静态/枚举/字符串）；overrides 合并
- `external-symbol-stub`: 外部符号用 is_external 列标识，复用 kind 存真实类型，按 (name,namespace) 去重

### Modified Capabilities

- 无。新分支独立开发，不修改现有 master 的 specs。

## Impact

- 核心模块：`src/indexer/` 新增 `clang/` 子模块，移除 C/C++ 的 tree-sitter 解析路径
- 存储模块：`src/storage/symbols.rs` schema 重新设计，`src/storage/mod.rs` 新增边类型
- MCP 模块：工具描述更新以反映新数据类型
- **BREAKING**：全新 schema，与旧版 .rag.db 不兼容
- 运行时依赖：系统 PATH 中需有 `clang` 命令（codeloom 本体 musl 编译，无链接依赖）

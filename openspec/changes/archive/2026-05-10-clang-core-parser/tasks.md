# Tasks: Clang 核心解析器

## 1. 分支与模块搭建

- [x] 1.1 从 master 创建 `clang-parser` 分支
- [x] 1.2 创建 `src/indexer/clang/` 模块目录（含 mod.rs、ast.rs、compile_cmds.rs）
- [x] 1.3 在 Cargo.toml 中确认 serde_json 依赖（流式解析所需）

## 2. compile_commands.json 发现与解析

- [x] 2.1 实现 `compile_cmds.rs`：三级搜索（显式指定→项目根→build/→out/）
- [x] 2.2 解析 JSON 数组，提取 directory、command/arguments 字段
- [x] 2.3 过滤编译参数：去除编译器名、`-c`、`-o`，保留 `-I`、`-D`、`-std=` 等
- [x] 2.4 降级模式：无 compile_commands.json 时 `clang -fsyntax-only -- <file>`
- [x] 2.5 翻译单元按编译参数分组

## 3. Clang 子进程调用

- [x] 3.1 实现 `spawn_clang()` 函数，用 `std::process::Command` 执行 clang 子进程
- [x] 3.2 捕获 stdout（JSON AST）和 stderr（警告/错误）
- [x] 3.3 clang 不可用时返回明确错误："请安装 Clang 15+"
- [x] 3.4 源文件编译失败时记录错误但继续处理其他文件
- [x] 3.5 添加超时机制（单文件 > 60s 跳过）

## 4. AST JSON 流式解析

- [x] 4.1 定义 Clang AST JSON 节点结构（TranslationUnitDecl、FunctionDecl 等）
- [x] 4.2 实现 `serde_json::StreamDeserializer` 流式解析
- [x] 4.3 按 TranslationUnitDecl 遍历，分发到各类声明处理器
- [x] 4.4 添加大文件内存测试

## 5. 符号提取（15 种 kind，不含 variable）

- [x] 5.1 FunctionDecl / CXXMethodDecl：提取 name、signature、namespace、is_definition、access、is_virtual。生成 sid=SHA256(repo+name+sig+ns+kind)前16位。声明与定义合并：name+ns+sig 匹配时更新已有符号，拼接 doc_comment
- [x] 5.2 CXXRecordDecl：提取 name、kind(class/struct)、bases(继承)、namespace。不存 parent_class
- [x] 5.3 EnumDecl / EnumConstantDecl：提取 enum 名和 enum_value 列表
- [x] 5.4 FieldDecl：提取 name、type、access。不存 parent_class
- [x] 5.5 NamespaceDecl：提取具名命名空间，嵌套 NS 用 `::` 连接
- [x] 5.6 宏识别：从预处理展开前的宏名提取 macro 符号
- [x] 5.7 TypedefDecl / TypeAliasDecl：提取 name、底层类型。创建 kind=typedef 符号 + aliases: 边
- [x] 5.8 StringLiteral：提取字面量值存入 name 列（超长截断前128字符），按 name 去重，创建 kind=string_literal 符号
- [x] 5.9 跳过局部变量

## 6. 边提取（11 种）

- [x] 6.1 calls: CallExpr→DeclRefExpr，外部调用也用 calls: 边（不区分边类型）
- [x] 6.2 inherits: CXXRecordDecl::bases 提取继承边
- [x] 6.3 overrides: 检测 override/final 标记，覆写→被覆写
- [x] 6.4 instantiates: template_instance → 主模板
- [x] 6.5 param_type: ParmVarDecl→RecordType
- [x] 6.6 return_type: FunctionDecl→returnType
- [x] 6.7 includes: 预处理指令提取文件级 include 边
- [x] 6.8 uses_type: 变量/字段→其声明类型
- [x] 6.9 contains: class/struct→method/field
- [x] 6.10 aliases: TypedefDecl/TypeAliasDecl→底层类型
- [x] 6.11 uses: function/method→global/static_var/enum_value/string_literal

## 7. 模板实例化

- [x] 7.1 ClassTemplateSpecializationDecl：检查主模板是否在本项目源文件中
- [x] 7.2 是→创建 template_instance 节点 + template_args 列存参数 JSON + instantiates: 边
- [x] 7.3 否（STL 等）→跳过，外部调用通过 calls: + is_external=1 处理

## 8. 外部符号（is_external 列）

- [x] 8.1 声明归属判断：Decl 所在文件是否在项目 compile_commands.json 覆盖范围内
- [x] 8.2 外部符号：kind 存 Clang 原始类型（function/class/struct等），is_external=TRUE，file_path=""，content_hash="ext:<hash>"
- [x] 8.3 (name, namespace, kind) 去重
- [x] 8.4 外部符号与 builtin 共存（不同 repo：项目 repo vs __builtin__）

## 9. Schema 迁移

- [x] 9.1 symbols 表新增 6 列：sid TEXT、access TEXT、is_virtual BOOL、is_definition BOOL、is_external BOOL、template_args TEXT
- [x] 9.2 symbols 表删除 parent_class 列（ALTER TABLE DROP COLUMN 或重建表）
- [x] 9.3 新增 UNIQUE 约束或索引 (sid)
- [x] 9.4 在 schema.rs 中登记 15 种 kind（删 variable/external_symbol，+namespace/template_instance/typedef/string_literal）
- [x] 9.5 在 schema.rs 中登记 11 种边类型
- [x] 9.6 更新 FTS5 索引以包含新列（DROP + 重建 fts5_sym）

## 10. 语言路由与 tree-sitter 清理

- [x] 10.1 修改 `src/indexer/mod.rs`：C/C++ 文件路由到 `clang/` 模块
- [x] 10.2 保留 Python/Java 等语言的 tree-sitter 路径不变
- [x] 10.3 移除 `src/indexer/queries/cpp.rs`
- [x] 10.4 清理 Cargo.toml 中的 tree-sitter-cpp 依赖

## 11. MCP 工具更新

- [x] 11.1 更新 `codeloom_schema` 工具：15 种 kind + 11 种边类型
- [x] 11.2 所有 MCP 工具返回结果中包含 sid 字段
- [x] 11.3 新增或修改 MCP 工具支持按 sid 精确查询单符号
- [x] 11.4 更新 `codeloom_search` 的 kind 过滤和 is_external 过滤

## 12. CLI 命令更新

- [x] 12.1 `index` 命令新增 `--compile-commands <path>` 参数
- [x] 12.2 `index` 命令报告：内部符号数、外部符号数、template_instance 数
- [x] 12.3 `check` 命令新增 Clang 版本检测

## 13. 测试

- [x] 13.1 创建测试 fixture：含函数、类、模板、继承、namespace 的最小 C++ 项目
- [x] 13.2 单元测试：compile_commands.json 解析
- [x] 13.3 单元测试：符号提取（每种 kind + 声明合并 + 外部符号 is_external 标记）
- [x] 13.4 单元测试：边提取（每种 edge_type，含 contains: 正确性）
- [x] 13.5 集成测试：完整索引→调用图查询→验证边精度
- [x] 13.6 集成测试：sid 跨重建稳定性
- [x] 13.7 测试：Clang 不可用时的错误处理
- [x] 13.8 验证 `cargo test` 全部通过

## 14. 验证与清理

- [x] 14.1 对 leveldb 重新索引，验证搜索精度 vs tree-sitter
- [x] 14.2 运行 `openspec validate clang-core-parser --strict --json` 确保合规
- [x] 14.3 git commit 按约定式提交规范
- [x] 14.4 更新 README.md

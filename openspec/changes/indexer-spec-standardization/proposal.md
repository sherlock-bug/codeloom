# Proposal: 索引器符号与边规格标准化

## Why

当前索引器存在三类问题：①项目头文件中的类/结构体被过滤器误判为系统符号而丢弃（DB、Status 等缺失）；②边类型定义不统一（instantiates 方向、uses_type 覆盖范围等规格与实际有偏差）；③系统符号噪声过滤标准不明确（哪些 C 库/STL 符号该丢、哪些该留）。需要一个全面的规格对齐来消除这些偏差。

## Scope

In scope:
- 明确哪些符号类型需要入库，哪些需要丢弃
- 统一所有边类型的定义、方向、创建条件
- 定义外部存根的创建规则（何时建、何时跳过）
- 定义系统符号的过滤标准（C 库函数、STL 内部细节、编译内建）
- 定义模板实例的处理规则（命名、保留条件、噪声去除）
- 定义项目头文件类/结构体的保库机制
- 更新断言测试用例覆盖以上规格

Out of scope:
- 非 C++ 语言（Python/Java/TypeScript）的索引行为
- 文档索引（doc-indexing）相关逻辑
- 搜索排名/排序（search-ranking）相关逻辑
- CLI 和 MCP 工具的接口设计

## Approach

分四步实施：
1. 先建立完整的规格文档（delta specs），明确每条规则
2. 更新断言测试用例使其与新规格对齐
3. 分模块修改代码实现（filter → ast.rs → mod.rs → 验证）
4. 验证通过后归档

## Capabilities

### New Capabilities
- `symbol-indexing-rules`: 定义哪些符号入库、哪些丢弃的完整规则集
- `edge-type-rules`: 定义所有边类型的方向、创建条件和 stub 规则
- `system-symbol-filter`: 定义系统符号过滤的层级、标准和保留条件

### Modified Capabilities
- `template-parsing`: 更新模板实例保留条件（仅含项目类型参数的 STL 容器实例 + 项目自有模板）、命名规则（实例用完整类型名）
- `extended-edge-types`: 更新边类型列表，修正 instantiates 方向（实例→主模板）、增加 uses_type 覆盖范围
- `external-symbol-stub`: 明确外部存根仅在 uses_type/inherits/param_type/return_type 时创建，calls 时不创建
- `clang-subprocess-parser`: 更新 filter 行为（is_system 判断依据、路径注入范围、includedFrom 使用策略）
- `code-indexing`: 更新项目头文件类/结构体的保库机制

## Impact

- `scripts/clang_filter.py`: is_system() 逻辑调整，路径注入范围调整
- `src/indexer/clang/ast.rs`: 模板实例命名、uses_type 覆盖范围、外部符号处理
- `src/indexer/clang/mod.rs`: 边插入逻辑（stub 创建条件）
- `tests/run-spec-assertions.py`: 新增断言测试覆盖规格
- `tests/BUG_INVENTORY.md`: 部分 bug 可能因规格对齐而修复

# Design: 索引器符号与边规格标准化

## Context

当前 CodeLoom 索引器存在三类规格偏差：
1. Python filter `clang_filter.py` 的 `is_system()` 在无 `loc.file` 时直接判系统，导致项目头文件类被丢弃
2. 边类型（instantiates 方向、uses_type 覆盖范围、overrides 检测机制）与最新规格不一致
3. 模板实例的保留标准和命名规则未明确

## Goals

- 实现完整的符号入库规则（项目符号 vs 系统符号的明确界限）
- 统一所有边类型的定义和方向
- 消除项目头文件类丢失 bug
- 减少系统符号噪声

## Decisions

### Decision: Filter 层双通道处理
对 FunctionDecl 和 CXXRecordDecl 采用不同的处理策略：
- FunctionDecl：无 `loc.file` → 系统（解决 35K C 库函数问题）
- CXXRecordDecl/RecordDecl/EnumDecl：无 `loc.file` 时通过 `includedFrom` 判断是否项目头文件

理由：C 库函数和项目头文件类在 Clang AST 中表现相同（无 `loc.file`），但语义截然不同。需要按节点类型区分处理。

### Decision: includedFrom 使用但不注入
`includedFrom` 仅用于 `is_system()` 的判断上下文，不作为节点自己的 `loc.file` 注入。

### Decision: 外部函数调用不建边
`calls:` 边指向的外部函数不建边、不建 stub。其他边类型指向外部符号时创建外部存根。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| system-symbol-filter 的 includedFrom 回退仍会让部分系统头文件类漏过 | 知名 STL 容器白名单机制，非知名容器的系统类被丢弃 |
| 模板实例保留规则的知名容器列表可能遗漏 | 可通过扩展列表解决，非破坏性变更 |

## Implementation Plan

### Phase 1: 规格与用例先行
1. 更新现有主 specs（sync delta specs）
2. 更新断言测试用例，覆盖所有新规格

### Phase 2: Python filter 修改
1. clang_filter.py 双通道处理（FunctionDecl vs CXXRecordDecl）
2. is_system() 逻辑调整
3. 路径注入范围收窄（仅 BODY_KINDS/DeclRefExpr）

### Phase 3: Rust 层修改
1. ast.rs：模板实例命名规则、uses_type 覆盖范围、外部符号处理
2. mod.rs：calls 边外部 stub 不创建

### Phase 4: 验证
1. spec 断言测试通过
2. leveldb 全量索引不超时
3. 项目头文件类全部入库

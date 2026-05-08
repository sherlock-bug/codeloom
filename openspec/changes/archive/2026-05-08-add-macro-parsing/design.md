# Design: 宏解析

## Context

CodeLoom 的 C++ 索引器基于 tree-sitter 解析 AST。当前 `walk_children` 对所有 `preproc_*` 节点返回 `{}`（跳过），意味着预处理器宏定义和条件编译块内的代码统统不参与索引。

这与用户期望不符——项目中存在大量有语义的宏（配置开关、导出标注），LLM 需要能搜索到它们。

约束：
- 不改数据库 schema（macro 复用 `symbols` 表）
- 不引入 clang（保持纯 tree-sitter）
- 不改变已有 75 个测试的行为

## Goals / Non-Goals

**Goals:**
- 提取有语义的宏定义为 `kind: "macro"` 符号
- 过滤 include guard / 平台宏 / 编译器内置宏等噪音
- 遍历条件编译块内部，不遗漏宏

**Non-Goals:**
- 宏展开（需要 clang，不做）
- 宏替换值的语义分析
- 宏调用分析（如注册宏 entrypoint 边，经讨论后移除）
- Python/Java 等语言的宏（它们基本没有预处理器）

## Decisions

### Decision 1: 用 tree-sitter 而非 clang
选择 tree-sitter 的 `preproc_def` / `preproc_function_def` 节点提取宏，而非引入 clang。

- tree-sitter 已在项目中，零新增依赖
- 宏定义本身（`#define X Y`）tree-sitter 足够处理
- clang 能展开宏但不适合提取"宏定义符号"这一语义
- clang 引入后会显著增加编译时间和二进制体积

### Decision 2: 噪音宏过滤用硬编码规则
`is_noise_macro()` 用名称模式匹配而非语义分析。

过滤规则：
| 模式 | 匹配 | 原因 |
|------|------|------|
| `*_H` / `*_H_` | `MY_HEADER_H`, `FOO_H` | include guard |
| `*INCLUDED*` | `FOO_INCLUDED` | 另一种 include guard 风格 |
| `__*` | `__GNUC__`, `__cplusplus` | 编译器/平台内置宏 |
| 精确匹配 | `NDEBUG`, `_WIN32`, `_MSC_VER`, `_GLIBCXX_`, `LEVELDB_EXPORT` | 无搜索价值的常见宏 |

选择硬编码而非 ML 分类的原因：噪音宏的模式极其稳定，ML 过度设计。

### Decision 3: 条件编译块递归遍历（不通过 body 字段）
`preproc_if` / `preproc_ifdef` 节点没有 `body` 字段（与 `template_declaration` 不同），因此 `walk_children` 对它们直接遍历 `child.children()`，不通过 `node.field_name()` 访问子字段。

这与 `walk_children` 中其他节点的处理方式一致（内部 match 用 `child.kind()` 而非 `node.field_name()` 分派）。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 噪音过滤误杀有语义的宏（如项目自定义的 `__MY_CUSTOM__`） | 双下划线前缀的宏绝大多数是系统内置；若用户反馈误杀，增加白名单配置 |
| 条件编译块遍历增加索引时间 | 只多了子节点遍历，不增加 I/O；实测 leveldb 索引符号数不变（所有宏都是噪音被跳过） |

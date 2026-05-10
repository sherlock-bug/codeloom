# Design: 符号使用边扩展

## Context
当前 C++ 索引器在函数体扫描环节已实现调用提取（`extract_calls_with_types`），但仅覆盖函数调用关系。三类「使用」关系缺失：
- **枚举值使用**：`return Status::OK` 不产生任何边
- **全局变量引用**：`g_config.reload()` 不追踪
- **字符串字面量**：`LOG("error msg")` 的字面量与函数无关联

## Goals / Non-Goals
**Goals**：在函数体 tree-sitter walk 中新增三类边的提取逻辑，实现搜索「谁用了 X」的能力。
**Non-Goals**：不提取变量读写区分（read vs write），统一用 `references` 边；不跨翻译单元解析枚举值。

## Decisions

### Decision 1: 枚举值检测 — AST 模式匹配 `qualified_identifier` + `field_expression`
函数体中最常见的枚举引用是 `EnumName::Value` 形式，在 tree-sitter 中为 `qualified_identifier` 节点。也有 `ClassName::EnumName::Value` 三级形式。检测后查符号表确认该名称确实为 `enum_value` 类型。

选择方案：基于 tree-sitter 节点类型检测，而非正则匹配源码文本。理由：AST 结构更可靠，不受注释/字符串内容干扰。

### Decision 2: 全局变量引用 — 变量名查符号表
函数体中所有 `identifier` 节点，排除本地声明的变量名和参数名后，若命中符号表中 `kind=global` 或 `kind=static_var` 的符号，则创建 `references:var_name` 边。

选择方案：只查符号表种类过滤（`kind=global|static_var`），不做读写语义区分。理由：简单可靠，搜索「谁用过 X」即可满足需求。

### Decision 3: 字符串字面量归属 — 在函数体扫描时记录 node id
字符串字面量已在索引阶段提取为 `string_literal` 符号。在函数体扫描时检测到 `string_literal` / `raw_string_literal` 节点，取其文本内容，通过符号表的 name 字段匹配已有字面量符号，创建 `uses:content` 边。

选择方案：通过符号名称匹配（而非 node id），因为字面量符号已在上一阶段去重入表。理由：避免在扫描阶段二次创建符号，复用已有索引。

## Implementation

三者在同一个 tree-sitter walk 循环中实现（`extract_calls_with_types` 同级，或直接在该函数内扩展）。核心循环不变，在每个 `cur.kind()` 分支中增加三个匹配臂：

```
loop { cur = cursor.node()
  match cur.kind() {
    "call_expression"        → 已有逻辑
    "qualified_identifier"   → 枚举值检测
    "field_expression"       → 枚举值/成员检测（含全局变量）
    "identifier"             → 全局变量引用检测
    "string_literal" | "raw_string_literal" | ... → 字面量关联
    _ => {}
  }
}
```

## Risks / Trade-offs
| 风险 | 缓解 |
|------|------|
| 枚举值 false positive（同名不同枚举） | 当前同一 repo 下枚举值名称唯一（含父枚举名），误匹配概率极低 |
| 全局变量误判（头文件中声明的 extern 变量） | extern 变量也是 `global` 种类，引用边有意义 |
| 字符串字面量匹配失败（含转义引号） | 用 tree-sitter 节点的 `utf8_text` 获取原始文本，与索引时的去重逻辑一致 |

## Migration Plan
1. 新增三个匹配臂到 `extract_calls_with_types` 函数
2. 添加辅助函数：`is_enum_value(name, symbols)`、`is_global_var(name, symbols)`
3. `cargo test` 全量通过
4. 对 leveldb 重建索引验证边数量增长
5. 更新 SDD 产物后归档

# fix-inheritance-kind-check

## Purpose
CodeLoom 继承树类型校验修复：inherits 边查询增加 class/struct kind 检查，消除假阳性子类/父类。

## Requirements

### Requirement: 继承树查询校验类型
系统 SHALL 在 inheritance_tree 的 `inherits` 边遍历中对 source 和 target 符号增加 kind 校验，仅当目标符号类型为 `class` 或 `struct` 时才纳入继承树。

#### Scenario: 假阳性子类被过滤
- GIVEN DBImpl（class）有一条 `inherits` 边指向 `FLAGS_use_existing_db = false`（string_literal）
- WHEN 查询 `codeloom_inheritance_tree(symbol="DBImpl", direction="down")`
- THEN children 列表不含 `FLAGS_use_existing_db = false`

#### Scenario: 正常继承关系不变
- GIVEN 类 B 继承自类 A（两者皆 class，边类型为 `inherits:ClassName`）
- WHEN 查询 `codeloom_inheritance_tree(symbol="A", direction="down")`
- THEN B 正常出现在 children 列表中

#### Scenario: 父类查询同样校验
- GIVEN 类 C 有一条 `inherits` 边指向类型为 `string_literal` 的符号 F
- WHEN 查询 `codeloom_inheritance_tree(symbol="C", direction="up")`
- THEN parents 列表不含 F

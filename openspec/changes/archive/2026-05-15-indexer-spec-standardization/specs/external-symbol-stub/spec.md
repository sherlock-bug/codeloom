# Delta for external-symbol-stub

## MODIFIED Requirements

### Requirement: 外部符号创建条件（修正）
原规格：为所有被引用的外部符号创建存根。
新规格：系统 SHALL 仅为以下场景的外部符号创建存根：
- `uses_type:` 边的目标类型
- `inherits:` 边的目标基类
- `param_type:` 边的目标参数类型
- `return_type:` 边的目标返回类型
系统 SHALL **不**为 `calls:` 边的目标外部函数创建存根或边。

#### Scenario: calls 不创建 stub（新增）
- GIVEN 项目函数体内调用了 `memcpy`
- WHEN 索引器处理该调用
- THEN `memcpy` 不在 `name_to_id` 中
- AND 不创建 `memcpy` 的外部存根
- AND 不建立 `calls:` 边

#### Scenario: uses_type 创建 stub（修正）
- GIVEN 项目变量类型为 `std::unique_ptr<Iterator>`，`Iterator` 在项目中存在
- WHEN 解析 uses_type 边，`unique_ptr` 不在已索引符号中
- THEN 创建 `unique_ptr` 的外部存根（`is_external=true`）
- AND 建立 `uses_type:` 边指向该存根

#### Scenario: inherits 创建 stub（修正）
- GIVEN 项目类继承自不在项目中的基类
- WHEN 解析 inherits 边
- THEN 创建基类的外部存根
- AND 建立 `inherits:` 边

### Requirement: 外部符号去重（修正）
原规格：按 `(name, namespace, kind)` 去重。
新规格：系统 SHALL 按 `(name, namespace, kind, repo)` 四元组对外部符号去重。

#### Scenario: 跨仓去重（新增）
- GIVEN 两个仓库都引用了 `std::string`
- WHEN 各自创建外部存根
- THEN 每个仓库各自有一个 `std::string` 存根，互不冲突

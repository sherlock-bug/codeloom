# type-aware-call-resolution

## Purpose
CodeLoom 类型感知调用解析——通过扫描局部变量声明和函数参数建立变量到类型的映射，成员调用 obj->method() 解析为 TypeName::method，支持 this 指针用 parent_class 限定。

## Requirements

### Requirement: 成员调用通过变量类型解析
当 tree-sitter 解析到 `call_expression` 且其 `function` 子节点为 `field_expression`（如 `obj->method()`）时，系统 SHALL 扫描当前函数体的局部声明和参数声明，建立变量到类型的映射，并据此将调用边解析为限定形式 `calls:TypeName::methodName`。

#### Scenario: 指针成员调用
- GIVEN 函数内有 `MyClass* obj = new MyClass();`
- WHEN 解析到 `obj->method()`
- THEN 生成的调用边 SHALL 为 `calls:MyClass::method`

#### Scenario: 值对象成员调用
- GIVEN 函数内有 `MyClass obj;`
- WHEN 解析到 `obj.method()`
- THEN 生成的调用边 SHALL 为 `calls:MyClass::method`

#### Scenario: 函数参数成员调用
- GIVEN 函数签名 `void f(std::vector<int>* v)`
- WHEN 解析到 `v->push_back(1)`
- THEN 生成的调用边 SHALL 为 `calls:std::vector::push_back`

#### Scenario: 自由函数调用不受影响
- GIVEN 函数 `void free_func()`
- WHEN 解析到 `free_func()`
- THEN 生成的调用边 SHALL 为 `calls:free_func`（不变）

#### Scenario: 未查到类型时降级
- GIVEN 变量 `x` 的类型在当前函数内无法解析（如跨函数传入）
- WHEN 解析到 `x->method()`
- THEN 生成的调用边 SHALL 降级为 `calls:method`（保持现有行为）

### Requirement: this 指针调用用 parent_class 限定
当 tree-sitter 解析到 `field_expression` 且对象为 `this` 关键字时，系统 SHALL 直接使用当前函数的 `parent_class` 作为限定类名。

#### Scenario: 类方法中 this 调用
- GIVEN 类 `MyClass` 的方法 `void MyClass::DoAll()` 内有 `this->DoWork()`
- WHEN 解析该调用
- THEN 生成的调用边 SHALL 为 `calls:MyClass::DoWork`

### Requirement: 自由函数调用不改动
当 `call_expression` 的 `function` 子节点不是 `field_expression` 时，系统 SHALL 保持现有纯名称提取行为。

#### Scenario: 命名空间限定调用
- GIVEN 调用 `std::find(vec.begin(), vec.end(), x)`
- WHEN 解析该调用
- THEN 生成的调用边 SHALL 为 `calls:std::find`（不变）


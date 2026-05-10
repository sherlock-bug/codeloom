# virtual-dispatch

## Purpose
CodeLoom 虚函数调度展开——当成员调用解析到基类方法时，检测 override 边并额外生成指向所有派生类 override 的调用边（calls_override），调用图遍历同时匹配 calls 和 calls_override 边类型。

## Requirements

### Requirement: 虚函数展开生成 override 调用边
当成员调用解析到基类方法时，系统 SHALL 检查该方法是否有 `overrides` 边。若有，SHALL 额外生成指向所有 override 版本的调用边，边类型为 `calls_override`。

#### Scenario: 基类虚函数调用展开
- GIVEN `DB` 类有虚方法 `virtual Status Get(...)`，`DBImpl` 继承 `DB` 并 override `Get`
- WHEN 解析 `db->Get(options, key, value)` 且 `db` 类型为 `DB*`
- THEN 系统 SHALL 生成 `calls:DB::Get` 和 `calls_override:DBImpl::Get`

#### Scenario: 无 override 时不生成额外边
- GIVEN `DBImpl::MaybeScheduleCompaction` 没有被任何派生类 override
- WHEN 解析 `this->MaybeScheduleCompaction()`
- THEN 系统 SHALL 仅生成 `calls:DBImpl::MaybeScheduleCompaction`，不生成 `calls_override` 边

#### Scenario: 多重 override 全部展开
- GIVEN 基类方法被 `DerivedA` 和 `DerivedB` 两个类 override
- WHEN 解析对该方法的调用
- THEN 系统 SHALL 生成两条 `calls_override` 边，分别指向 `DerivedA::method` 和 `DerivedB::method`

### Requirement: call_graph 遍历包含 override 边
`call_graph.rs` 的 `traverse_calls` SHALL 同时匹配 `calls:%` 和 `calls_override:%` 边类型。

#### Scenario: 调用图显示 override
- GIVEN `DB` 的 `Get` 被 `DBImpl::Get` override
- WHEN 查询 `DB::Get` 的 callees（含 override）
- THEN 调用图 SHALL 显示 `DBImpl::Get` 作为 override 节点


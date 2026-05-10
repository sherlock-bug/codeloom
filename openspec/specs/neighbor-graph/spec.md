# neighbor-graph

## Purpose
CodeLoom 邻里图：查看符号周围的直接关联符号，按边类型分组展示，支持方向和深度控制。

## Requirements

### Requirement: 单符号邻域分析
系统 SHALL 提供单符号 1-2 跳邻域分析，支持方向控制，按边类型分组返回。

#### Scenario: 查询类的双向邻居
- GIVEN 类 `DBImpl` 与 DB（继承）、`DBImpl::Get`（包含）、`Env::NewSequentialFile`（调用）、`Status::OK`（使用）有边
- WHEN 调用 `codeloom_neighbor_graph` 查询 `symbol=DBImpl direction=both depth=1`
- THEN 返回按方向+边类型分组的邻居：`forward: {contains: [DBImpl::Get,...]}` + `backward: {inherits: [DB], called_by: [...]}`

#### Scenario: 仅正向查询
- GIVEN 函数 `init` 调用了多个函数
- WHEN 查询 `symbol=init direction=forward depth=1`
- THEN 仅返回正向邻居，不含 `called_by` 等反向信息

#### Scenario: 查询枚举值的邻居
- GIVEN 枚举值 `Status::OK` 被 3 个函数使用
- WHEN 调用邻域图 `direction=reverse`
- THEN 返回 `backward: {used_by: [validate, process_request, check_status]}`

### Requirement: 输入参数定义
系统 SHALL 接受 `symbol`、`repo`、`branch`、`direction`（默认 "both"）、`depth`（默认 1）参数。

#### Scenario: 默认方向为双向
- GIVEN 用户指定 `symbol=DBImpl` 但不指定 direction
- WHEN 工具执行
- THEN 使用默认 direction="both"，返回正向和反向邻居

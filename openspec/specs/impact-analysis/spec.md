# impact-analysis

## Purpose
CodeLoom 影响分析：基于图遍历的传递闭包，评估修改某符号的影响范围，支持方向控制和半径限制。

## Requirements

### Requirement: 传递闭包影响分析
系统 SHALL 提供从指定符号出发的 N 跳传递闭包分析，支持方向控制，覆盖所有边类型。

#### Scenario: 修改全局变量的影响范围（反向分析）
- GIVEN 全局变量 `g_config` 被 `process_request` 引用，`process_request` 又被 `main` 调用
- WHEN 调用 `codeloom_impact_analysis` 查询 `symbol=g_config direction=reverse radius=3`
- THEN 返回 `process_request`（distance=1 via references）、`main`（distance=2 via calls）

#### Scenario: 正向分析某函数影响哪些对象
- GIVEN 函数 `init` 调用了 `g_config` 和 `db_open`
- WHEN 查询 `symbol=init direction=forward radius=1`
- THEN 返回 `g_config`（via references）、`db_open`（via calls）

#### Scenario: 超半径限制时不扩展
- GIVEN `symbol=A radius=2`，A 影响传递 4 跳才到 Z
- WHEN 工具执行
- THEN Z 不在结果中（距离 4 > radius 2）

### Requirement: 输入参数定义
系统 SHALL 接受 `symbol`、`repo`、`branch`、`direction`（默认 "reverse"）、`radius`（默认 3）参数。

#### Scenario: 默认方向为反向
- GIVEN 用户指定 `symbol=g_config` 但不指定 direction
- WHEN 工具执行
- THEN 使用默认 direction="reverse"，查询谁引用了 g_config

# inheritance-tree

## Purpose
CodeLoom 继承树分析：递归构建类的继承层次，direction down 查子类 / up 查父类，含虚函数 override 信息。

## Requirements

### Requirement: 继承体系树形分析
系统 SHALL 提供指定类的继承树分析，向上查祖先，向下查子孙，含虚拟方法分发。

#### Scenario: 向下展开子类树
- GIVEN 类 `DB` 被 `DBImpl` 继承，`DBImpl` 被 `LevelDBImpl` 继承
- WHEN 调用 `codeloom_inheritance_tree` 查询 `symbol=DB direction=down`
- THEN 返回嵌套树：`DB → children: [DBImpl → children: [LevelDBImpl]]`

#### Scenario: 向上展开祖先链
- GIVEN 类 `LevelDBImpl` 继承自 `DBImpl`，`DBImpl` 继承自 `DB`
- WHEN 查询 `symbol=LevelDBImpl direction=up`
- THEN 返回祖先链：`LevelDBImpl → inherits: DBImpl → inherits: DB`

#### Scenario: 含虚拟方法分发
- GIVEN `DB::Write` 是虚函数，`DBImpl::Write` 和 `LevelDBImpl::Write` override 了它
- WHEN 查询 `symbol=DB direction=down max_depth=5`
- THEN 返回的每个子类包含 `overrides` 字段列出覆写的方法名

#### Scenario: 叶子节点无子类
- GIVEN `LevelDBImpl` 没有子类
- WHEN 查询 `symbol=LevelDBImpl direction=down`
- THEN 返回 `{"symbol": "LevelDBImpl", "children": []}`

### Requirement: 输入参数定义
系统 SHALL 接受 `symbol`、`repo`、`branch`、`direction`（"up"/"down"/"both"）、`max_depth`（默认 5）参数。

#### Scenario: 默认方向为双向
- GIVEN `symbol=DBImpl`
- WHEN 不指定 direction
- THEN 默认 direction="both"，返回祖先和子孙

# graph-visualization

## Purpose

定义图分析页面的交互规范，包括调用链树形图、继承树 UML 类图和调用路径链路图的展示方式、参数控制和交互反馈。

## Requirements

### Requirement: 调用链树形图

调用链分析 SHALL 以可折叠树形图展示函数调用关系，根节点为当前分析符号，子节点按层次展开。
每个节点 SHALL 显示：符号名、符号类型（kind）、文件路径、行号。当前分析符号 SHALL 高亮。

#### Scenario: 查看调用链
- GIVEN 用户在"调用链分析"标签页
- WHEN 输入函数名并选择方向(callees/callers)和深度后点击"分析"
- THEN 树形图 SHALL 渲染在可视化区域
- AND 每个节点 SHALL 可点击展开/折叠子节点
- AND 深度滑块 SHALL 实时影响树的最大深度

### Requirement: UML 类图继承树

继承树分析 SHALL 以 UML 类图风格展示，每个类为一个卡片，显示类名和方法列表。
当前分析的类 SHALL 高亮，继承关系用箭头连接。虚方法 SHALL 用斜体标注，重写方法 SHALL 用特殊标记。

#### Scenario: 查看继承树
- GIVEN 用户在"继承树"标签页
- WHEN 输入类名并选择方向后点击分析
- THEN UML 类图 SHALL 渲染在可视化区域
- AND 当前类卡片 SHALL 有蓝色边框高亮
- AND 父类向上排列，子类向下排列
- AND 每个类卡片 SHALL 显示其方法列表

### Requirement: 调用路径分析

路径分析 SHALL 以链路图展示从起点符号到终点符号的调用路径，每个节点为函数符号，连线为调用边。
最短路径模式 SHALL 只显示最优路径，所有路径模式 SHALL 显示多条可选路径。

#### Scenario: 查看调用路径
- GIVEN 用户在"调用路径分析"标签页
- WHEN 输入起点和终点符号名后点击分析
- THEN 链路图 SHALL 渲染为横向节点链
- AND 终点节点 SHALL 用不同颜色标注
- AND 边类型过滤复选框 SHALL 控制显示哪些类型的边

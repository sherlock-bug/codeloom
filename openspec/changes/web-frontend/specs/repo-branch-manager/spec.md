# repo-branch-manager

## Purpose

定义仓库与分支管理页面的交互规范，包括仓库 CRUD、分支列表展示、索引触发和状态反馈。用户通过此界面管理 CodeLoom 已注册的代码仓库，不再需要敲 CLI 命令。

## Requirements

### Requirement: 仓库列表

仓库列表 SHALL 以表格形式展示，包含列：仓库名、路径、分支数、符号数、最后索引时间、状态、操作。
状态 SHALL 包含：正常（绿色）、索引中（橙色）、失败（红色）、未索引（灰色）。

#### Scenario: 仓库状态展示
- GIVEN 仓库列表页面已加载
- WHEN 查看仓库行
- THEN 状态列 SHALL 使用对应颜色的 tag 标签
- AND 操作列 SHALL 根据状态提供不同按钮（正常→索引/详情/删除，索引中→禁用索引，失败→重试/删除）

### Requirement: 分支展开

点击仓库行 SHALL 展开该仓库的分支列表，展示每个分支的名称、符号数、最后索引时间和操作按钮。

#### Scenario: 查看分支
- GIVEN 仓库列表页面
- WHEN 用户点击仓库行
- THEN 该行下方 SHALL 展开分支详情卡片
- AND 卡片包含分支列表表格和分支级操作按钮

### Requirement: 添加仓库

点击"添加仓库"按钮 SHALL 弹出模态对话框，包含：仓库路径输入、仓库名输入、可选初始分支。
确认后 SHALL 立即注册仓库并触发首次索引。

#### Scenario: 添加新仓库
- GIVEN 仓库管理页面
- WHEN 点击"添加仓库"按钮
- THEN 模态对话框 SHALL 弹出
- AND 用户填写路径和名称后确认
- THEN 仓库 SHALL 出现在列表中，状态为"索引中"

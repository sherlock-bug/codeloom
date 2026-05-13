# search-interface

## Purpose

定义搜索页面的交互规范，包括精确搜索（BM25 FTS5）和模糊搜索（语义向量）两种模式、结果展示、过滤筛选和结果跳转功能。

## Requirements

### Requirement: 双模式搜索

搜索页面 SHALL 提供两种搜索模式切换：精确搜索（BM25关键词）和模糊搜索（语义向量）。两种模式使用同一搜索输入框，但结果显示的 score 含义不同。

#### Scenario: 切换搜索模式
- GIVEN 用户在搜索页面
- WHEN 点击"模糊搜索"模式按钮
- THEN 搜索模式 SHALL 切换为语义搜索
- AND 搜索结果中的 score SHALL 表示语义相似度而非关键词匹配度

### Requirement: 结果展示

搜索结果项 SHALL 包含：符号名、类型标签（class/function/method/struct/enum）、文件路径、行号、score 值、代码片段。
搜索词在代码片段中 SHALL 高亮显示。

#### Scenario: 展示搜索结果
- GIVEN 搜索已返回结果
- WHEN 渲染结果列表
- THEN 每项 SHALL 显示符号名（粗体）
- AND 类型标签 SHALL 使用对应颜色的 tag
- AND 代码片段中的搜索词 SHALL 用 `<em>` 高亮

### Requirement: 结果筛选

搜索页面 SHALL 在结果列表上方提供过滤条件：仓库选择、分支选择、符号类型选择。
切换过滤条件 SHALL 自动重新搜索。

#### Scenario: 按类型过滤
- GIVEN 搜索结果列表已展示
- WHEN 用户选择"class"类型过滤
- THEN 结果列表 SHALL 只显示 class 类型的符号

### Requirement: 结果跳转

点击搜索结果项 SHALL 跳转到文件浏览页面，并定位到对应文件的行号位置。

#### Scenario: 点击结果跳转
- GIVEN 搜索结果列表已展示
- WHEN 用户点击某个结果项
- THEN 页面 SHALL 切换到文件浏览标签页
- AND 文件树 SHALL 展开到对应文件
- AND 代码预览 SHALL 滚动到目标行并高亮

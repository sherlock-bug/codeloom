# frontend-overview

## Purpose

定义 CodeLoom Web 前端的整体架构规范，包括页面路由、组件树、数据流和状态管理的约定。本 spec SHALL 作为前端开发的导航入口，确保各页面风格统一、交互一致。

## Requirements

### Requirement: SPA 路由约定

前端 SHALL 使用基于 hash 的路由（`#/dashboard`, `#/repos`, `#/search` 等），无需服务端路由支持。

#### Scenario: 路由切换
- GIVEN 用户在搜索页面浏览
- WHEN 点击侧边栏"仓库管理"
- THEN URL hash 变更为 `#/repos`
- AND 主内容区切换为仓库管理页面

### Requirement: 主题风格

前端 SHALL 使用深色主题（dark mode），与 Codeloom 的技术工具属性匹配。配色方案：
- 主背景: `#0d1117`（GitHub Dark 风格）
- 卡片背景: `#161b22`
- 边框: `#30363d`
- 主色调: `#58a6ff`（蓝色）
- 成功: `#3fb950`
- 警告: `#d29922`
- 错误: `#f85149`
- 文字主色: `#e6edf3`
- 文字次要: `#8b949e`

#### Scenario: 深色主题一致性
- GIVEN 用户打开任意页面
- WHEN 检查页面背景和文字颜色
- THEN 所有页面 SHALL 使用统一的暗色调色板

### Requirement: 响应式布局

前端 SHALL 在 1280px+ 分辨率下全功能可用，在 768px-1279px 下侧边栏自动折叠。

#### Scenario: 窗口缩放
- GIVEN 窗口宽度为 1024px
- WHEN 页面加载
- THEN 侧边栏 SHALL 折叠为图标模式，悬停展开文字标签

### Requirement: 错误与空状态

所有数据展示区域 SHALL 具备三种状态：加载中（骨架屏/spinner）、数据正常（内容展示）、无数据/错误（提示与重试按钮）。

#### Scenario: 搜索无结果
- GIVEN 用户输入一个不存在的符号名
- WHEN 搜索执行返回空结果
- THEN 页面 SHALL 显示"未找到匹配结果"提示，并建议调整搜索词或切换搜索模式

### Requirement: 全局搜索入口

前端 SHALL 在顶部导航栏固定位置提供全局搜索输入框，支持快捷键 `Ctrl+K` 聚焦。

#### Scenario: 触发全局搜索
- GIVEN 用户在仓库管理页面
- WHEN 按下 `Ctrl+K`
- THEN 全局搜索输入框 SHALL 获得焦点
- AND 输入框展开为完整搜索模式

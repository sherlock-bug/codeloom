# Tasks: web-frontend

## Phase 1: 基础框架

- [ ] **T1.1** 创建 `src/web/` 模块，添加 `Cargo.toml` 依赖（axum, tower-http, rust-embed, tokio）
- [ ] **T1.2** 实现 `codeloom serve` CLI 子命令（端口可配，默认 8080）
- [ ] **T1.3** 实现 HTTP 服务器启动逻辑（`src/web/server.rs`）
- [ ] **T1.4** 创建 SPA HTML 骨架（侧边栏 + 导航 + 路由框架）
- [ ] **T1.5** 实现静态资源嵌入（rust-embed）
- [ ] **T1.6** 实现基础 REST API 路由注册（`/api/status` + `/api/repos` 桩）

## Phase 2: 核心功能

- [ ] **T2.1** 实现仓库管理 REST API（list / add / delete / branch list）
- [ ] **T2.2** 实现仓库管理前端页面（列表、添加对话框、删除确认）
- [ ] **T2.3** 实现搜索 REST API（精确 + 模糊，参数：q, repo, branch, kind, limit）
- [ ] **T2.4** 实现搜索前端页面（搜索框、模式切换、结果列表、高亮）
- [ ] **T2.5** 实现文件浏览 REST API（目录列表、文件内容、符号列表）
- [ ] **T2.6** 实现文件浏览前端页面（树形目录 + 代码预览 + 符号侧栏）

## Phase 3: 图分析

- [ ] **T3.1** 实现图分析 REST API 路由（call-graph / inheritance / path）
- [ ] **T3.2** 实现调用链树形可视化（D3.js 可折叠树）
- [ ] **T3.3** 实现继承树 UML 类图可视化
- [ ] **T3.4** 实现路径分析链路图可视化
- [ ] **T3.5** 实现图分析前端页面（三标签布局 + 参数面板 + 可视化区域）

## Phase 4: 高级功能

- [ ] **T4.1** 实现代码组管理 REST API + 前端页面
- [ ] **T4.2** 实现批量操作执行（pull / index / checkout）与进度 SSE
- [ ] **T4.3** 实现文件上传解析 API + 前端上传组件
- [ ] **T4.4** 实现仪表盘页面（系统概览、仓库状态卡片、最近活动）
- [ ] **T4.5** 实现日志查看 API + 前端日志面板
- [ ] **T4.6** 完善错误处理、加载状态、空状态等 UI 细节
- [ ] **T4.7** 更新 README.md 和帮助文档

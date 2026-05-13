# Proposal: web-frontend

## Intent

为 CodeLoom 提供一套 Web 管理界面，让用户通过浏览器完成代码库的索引管理、搜索浏览、知识图谱分析等操作，替代纯 CLI/MCP 交互方式，降低使用门槛。

## Scope

In scope:
- 嵌入 HTTP 服务器到 CodeLoom 二进制中，提供 REST API
- 单页 Web 前端（HTML + CSS + JS），零构建依赖
- 代码仓与分支管理界面（列表、添加、删除、索引触发）
- 代码组批量管理（一组代码仓关联操作：拉代码 → index → 切分支）
- 精确搜索（BM25 FTS5）结果展示
- 模糊搜索（语义向量）结果展示
- 文件管理（查看、浏览、切分支关联）
- 调用链分析（树形可视化）
- 类分析（继承树 + UML 类图展示）
- 调用路径分析（链路图可视化）
- 文件上传解析（上传源码/文档文件触发 index）
- 与现有 SQLite 数据库/MCP 工具共享数据

Out of scope:
- 用户认证与权限管理（单用户模式，本地使用）
- 多人协作编辑
- 大规模集群部署
- 完整的 IDE 功能（代码编辑/调试）
- 替代现有 CLI 和 MCP 功能（保持共存）

## Approach

**架构分层：**

```
浏览器 (SPA)
    ↓ HTTP REST / SSE
CodeLoom Web Server (axum/actix-web)
    ↓ 内部调用
CodeLoom Core (indexer / query / search / storage)
```

**后端：** 在现有 CodeLoom 二进制中嵌入一个可选的 HTTP 服务器模块（axum），作为子命令 `codeloom serve` 启动。该服务器暴露 REST API 给前端，底层调用现有的 query、search、storage 模块。

**前端：** 纯静态 SPA，打包为嵌入式资源（`rust-embed`）放进二进制，`codeloom serve` 启动后浏览器打开 `http://localhost:PORT` 即可访问。

**技术选型：**
- 后端框架: axum（轻量，tokio 原生，社区活跃）
- 前端: 原生 HTML + CSS + JS（零构建工具依赖），配合 D3.js 或 vis-network 做图可视化
- 数据流: REST JSON + Server-Sent Events（后台任务进度推送）
- 打包: rust-embed 将前端静态资源编译进二进制

**页面结构：**
1. 仪表盘 — 系统概览、仓库健康状态、索引进度
2. 仓库管理 — 仓库/分支列表、添加、删除、索引触发
3. 代码组管理 — 创建组、批量操作（拉取/index/切分支）
4. 搜索 — 搜索栏 + 结果列表（精确/模糊切换）
5. 文件浏览器 — 树形文件导航 + 文件内容 + 符号列表
6. 图分析 — 调用链、继承树、路径分析三大可视化面板
7. 上传解析 — 文件上传 + 解析进度 + 结果查看

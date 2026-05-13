# Design: web-frontend

## 1. 系统架构

```
┌─────────────────────────────────────────────┐
│                 浏览器 (SPA)                   │
│  HTML + CSS + JS + D3.js + vis-network      │
└──────────────────┬──────────────────────────┘
                   │ HTTP REST + SSE
┌──────────────────▼──────────────────────────┐
│          CodeLoom Web Server (axum)           │
│  /api/* 路由 → handler → core 模块调用       │
│  /static/* → rust-embed 静态资源              │
│  /api/events → SSE 任务进度流                │
└──────────────────┬──────────────────────────┘
                   │ 内部函数调用
┌──────────────────▼──────────────────────────┐
│            CodeLoom Core 模块                  │
│  indexer / query / search / storage / mcp    │
└──────────────────┬──────────────────────────┘
                   │ SQL + SQLite
┌──────────────────▼──────────────────────────┐
│              SQLite 数据库                      │
│  symbols / edges / FTS5 / vec0 / files       │
└─────────────────────────────────────────────┘
```

## 2. REST API 设计

### 2.1 仓库管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/repos | 列出所有仓库 |
| POST | /api/repos | 添加新仓库 |
| DELETE | /api/repos/:name | 删除仓库 |
| GET | /api/repos/:name/branches | 列出分支 |
| POST | /api/repos/:name/index | 触发索引 |
| POST | /api/repos/:name/branches/:branch/index | 指定分支索引 |
| DELETE | /api/repos/:name/branches/:branch | 删除分支索引 |

### 2.2 代码组管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/groups | 列出所有组 |
| POST | /api/groups | 创建组 |
| PUT | /api/groups/:id | 更新组 |
| DELETE | /api/groups/:id | 删除组 |
| POST | /api/groups/:id/repos | 向组添加仓库 |
| DELETE | /api/groups/:id/repos/:repo | 从组移除仓库 |
| POST | /api/groups/:id/execute | 执行批量操作 |

**批量操作类型：**
- `pull` — 拉取最新代码（所有仓库）
- `index` — 执行索引
- `checkout` — 切换分支
- `pull+index` — 拉取后索引

### 2.3 搜索

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/search?q=...&mode=exact|fuzzy&repo=...&branch=...&kind=...&limit=... | 精确/模糊搜索 |

**返回格式：**
```json
{
  "results": [
    {
      "id": 123,
      "name": "AuthService",
      "kind": "class",
      "file": "src/auth.rs",
      "line": 42,
      "signature": "class AuthService",
      "score": 0.85,
      "snippet": "pub struct AuthService { ... }",
      "repo": "myproject",
      "branch": "master"
    }
  ],
  "total": 42,
  "query": "AuthService",
  "mode": "exact|fuzzy"
}
```

### 2.4 文件浏览

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/files?repo=...&branch=...&path=... | 列出目录 |
| GET | /api/files/content?repo=...&branch=...&path=... | 文件内容 |
| GET | /api/files/symbols?repo=...&branch=...&path=... | 文件中符号 |

### 2.5 图分析

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/graph/call-graph?name=...&repo=...&branch=...&direction=...&depth=... | 调用链 |
| GET | /api/graph/inheritance?name=...&repo=...&branch=...&direction=...&depth=... | 继承树 |
| GET | /api/graph/path?source=...&target=...&repo=...&branch=...&mode=... | 路径分析 |
| GET | /api/graph/neighbor?name=...&repo=...&branch=...&direction=... | 邻里图 |
| GET | /api/graph/impact?name=...&repo=...&branch=...&direction=...&radius=... | 影响分析 |

### 2.6 文件上传与解析

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/upload | 上传文件(源码/文档) |
| GET | /api/upload/:task-id/status | 解析进度(SSE) |

### 2.7 系统状态

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/status | 系统概览(仓库数/符号数/磁盘占用) |
| GET | /api/logs?level=...&module=...&limit=... | 日志查看 |

## 3. 前端页面与组件

### 3.1 布局结构

```
┌──────────┬────────────────────────────────────┐
│          │           头部导航栏                  │
│  侧边栏  ├────────────────────────────────────┤
│          │                                    │
│  导航    │         主内容区域                    │
│  图标    │                                    │
│  文本    │   （根据路由切换页面内容）              │
│          │                                    │
│  用户    │                                    │
│  头像    │                                    │
└──────────┴────────────────────────────────────┘
```

侧边栏导航项：
1. 🏠 仪表盘
2. 📦 仓库管理
3. 📂 代码组
4. 🔍 搜索
5. 📄 文件浏览
6. 🌐 图分析
7. 📤 上传解析

### 3.2 仪表盘页面

- 总仓库数 / 总符号数 / 总边数 / 数据库大小
- 最近索引活动（时间线）
- 各仓库健康状态卡片

### 3.3 仓库管理页面

- 仓库列表表格（名称 / 分支数 / 符号数 / 最近索引 / 状态）
- 操作列：索引 / 删除 / 查看详情
- 添加仓库对话框（输入仓库路径、名称）
- 点击仓库行展开分支列表

### 3.4 代码组管理页面

- 组列表（名称 / 包含仓库数 / 状态）
- 创建/编辑组表单
- 组详情 → 仓库列表 → 批量操作按钮
- 批量操作进度展示（每个仓库的操作状态）

### 3.5 搜索页面

- 搜索框（支持 Enter 搜索，带清空按钮）
- 模式切换：精确搜索 / 模糊搜索
- 过滤条件：仓库、分支、符号类型
- 搜索结果列表（每项展示名称、类型、文件路径、行号、score、snippet）
- 结果项点击 → 跳转到文件浏览页面定位到对应行
- 分页/滚动加载更多
- 高亮匹配关键词

### 3.6 文件浏览页面

- 左侧树形文件目录
- 右侧代码预览（语法高亮）
- 符号列表侧边栏（文件中符号一览，点击跳转）
- 分支切换下拉菜单
- 文件路径面包屑导航

### 3.7 图分析页面

三栏式布局（或标签页切换）：

**调用链分析（树形）：**
- 搜索框输入函数名
- 方向切换：callers / callees
- 深度滑块
- 树形可视化（D3.js 可折叠树）
- 节点点击查看详情

**继承树分析（UML 类图）：**
- 搜索框输入类名
- 方向：向上/向下/双向
- UML 类图可视化（带基类/派生类、虚方法标注）
- 继承关系箭头标注

**路径分析（链路图）：**
- 起点输入框 + 终点输入框
- 模式：最短路径 / 所有路径
- 边类型过滤勾选框
- 图可视化（节点 + 箭头连线布局）

### 3.8 上传解析页面

- 文件拖拽/选择上传区域
- 上传进度条
- 解析状态轮询展示
- 解析完成后跳转到搜索结果

## 4. 技术依赖

### 新增 Rust 依赖
- `axum` — HTTP 服务器框架
- `tower-http` — CORS、静态文件服务中间件
- `serde_json` — JSON 序列化（已有）
- `tokio` — 异步运行时（已有）
- `rust-embed` — 静态资源嵌入

### 前端依赖（CDN 加载）
- D3.js v7 — 图数据可视化
- vis-network — 网络图可视化（可选，作为 D3 补充）
- highlight.js — 代码语法高亮
- 无构建工具依赖

## 5. 目录结构变更

```
src/
├── web/                        # 新增：Web 服务器模块
│   ├── mod.rs                  # 模块入口
│   ├── server.rs               # HTTP 服务器启动
│   ├── routes/                 # 路由处理
│   │   ├── mod.rs
│   │   ├── repos.rs            # 仓库管理 API
│   │   ├── groups.rs           # 代码组 API
│   │   ├── search.rs           # 搜索 API
│   │   ├── files.rs            # 文件浏览 API
│   │   ├── graph.rs            # 图分析 API
│   │   ├── upload.rs           # 上传解析 API
│   │   └── status.rs           # 系统状态 API
│   └── static/                 # 前端静态资源
│       ├── index.html          # SPA 入口
│       ├── css/
│       │   └── app.css         # 全局样式
│       ├── js/
│       │   ├── app.js          # 路由/状态管理
│       │   ├── pages/          # 各页面组件
│       │   ├── components/     # 通用 UI 组件
│       │   └── d3-vis.js       # D3 图可视化
│       └── assets/
├── cli/                        # 现有：CLI 子命令
│   └── serve.rs                # 新增：serve 子命令
└── ...（其他现有模块）
```

## 6. 实施阶段

### Phase 1: 基础框架（本 SDD）
- 创建 `src/web/` 模块框架
- 实现 `codeloom serve` 子命令
- 嵌入静态 SPA
- 基础 REST API 路由

### Phase 2: 核心功能
- 仓库/分支 API
- 搜索 API（精确+模糊）
- 文件浏览 API
- 前端页面实现（仓库管理、搜索、文件浏览）

### Phase 3: 图分析
- 调用链 API + 可视化
- 继承树 API + UML 类图
- 路径分析 API + 链路图
- 邻里图、影响分析

### Phase 4: 高级功能
- 代码组管理
- 文件上传解析
- 仪表盘
- 系统状态/日志
- SSE 进度推送

## 7. 与现有架构的关系

- 不修改现有 indexer/query/search/storage 核心模块（开闭原则）
- Web 服务器作为可选组件，不启动时原有功能不变
- REST API 复用现有 query/search 模块函数，不重复实现逻辑
- 不修改现有 MCP 工具（保持两种访问通道共存）

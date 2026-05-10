# multi-repo

## Purpose
CodeLoom multi-repo 功能域。本规范描述此功能的需求和行为。

## Purpose
支持多代码仓项目组，独立索引增量更新，自动识别跨仓依赖关系（include/import/API 端点），按仓过滤查询。

## Requirements

### Requirement: 多代码仓配置
系统 SHALL 支持在 config.yaml 中配置多个代码仓，每个仓各有独立的 root 路径，共享同一个 SQLite 知识库。

#### Scenario: 配置前后端两个仓
- GIVEN config.yaml 配置了 backend 和 frontend 两个 repo
- WHEN 执行 rag status
- THEN 分别显示两个仓的符号数量、索引时间和分支状态
- AND 总计显示跨仓统计

### Requirement: 独立索引增量更新
系统 SHALL 允许对单个代码仓执行索引和增量更新，不影响其他仓的符号。

#### Scenario: 只更新 frontend 仓
- GIVEN backend 和 frontend 均已全量索引
- AND 用户只修改了 frontend 的 3 个文件
- WHEN 执行 rag index --repo frontend
- THEN 仅扫描 frontend 仓的文件
- AND backend 仓的符号完全不重新处理
- AND 索引时间与 frontend 仓大小成正比，不受 backend 影响

### Requirement: 跨仓依赖自动识别
系统 SHALL 在索引时自动检测跨代码仓的依赖关系：include/import 引用、HTTP API 端点调用、共享类型定义等，建立跨仓的依赖边。

#### Scenario: backend 调用 shared-lib
- GIVEN shared-lib 仓已索引，含 Validator::check 符号
- AND backend 仓包含 #include "shared/validator.h" 且调用了 Validator::check()
- WHEN 索引 backend 仓
- THEN 自动创建 calls 类型的跨仓边
- AND 边的 source_repo='backend', target_repo='shared-lib'

#### Scenario: frontend 调用 backend API
- GIVEN frontend 仓含 fetch('/api/auth/login')
- AND backend 仓已索引，含路由 POST /api/auth/login → AuthController::login
- WHEN 索引 frontend 仓
- THEN 自动建立跨仓依赖边：frontend/apiClient → backend/AuthController::login
- AND 边类型为 calls

### Requirement: 符号归属标识
系统 SHALL 在每个符号上标注所属的代码仓，查询结果中自动带出 repo 字段。

#### Scenario: 查询符号自动显示仓名
- GIVEN 多仓均已索引
- WHEN 调用 rag_list_symbols("login")
- THEN 结果中每个符号标注 repo（如 frontend:login, backend:login）
- AND 不混淆来自不同仓的同名符号

### Requirement: 按仓过滤查询
系统 SHALL 在所有 MCP 查询工具上支持 repo 可选参数，允许限定查询范围到单个代码仓，默认 'all' 查询所有仓。

#### Scenario: 只看前端仓的符号
- GIVEN rag_list_symbols("render", repo="frontend")
- THEN 只返回 frontend 仓下的 render 相关符号
- AND 不返回 backend 或其他仓的 render

#### Scenario: 跨仓完整调用链
- GIVEN rag_get_call_graph("handleLogin") 未指定 repo
- THEN 返回完整调用链，跨仓边用 [→ repo名] 标记
- AND 显示从前端到后端到数据库的完整链路

### Requirement: 多仓共享 DB
系统 SHALL 对多仓项目组使用同一个 SQLite 数据库文件，各仓按需写入，团队通过拉取此 DB 获取所有仓的知识。

#### Scenario: 新成员拉取全量知识库
- GIVEN CI 产出了含 5 个仓的 shared DB
- WHEN 新成员执行 rag pull
- THEN 下载单个 DB 文件即获得所有仓的完整知识图谱
- AND 无需对每个仓分别拉取

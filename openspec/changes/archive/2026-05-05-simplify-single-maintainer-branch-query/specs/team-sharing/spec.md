# Delta for team-sharing

## REMOVED Requirements

### Requirement: 路径无关化存储
**Reason**: 项目改为单人维护模式，不再需要支持多机器、多路径的团队协作场景。
**Migration**: 代码层面保留路径无关化逻辑（不影响单人使用），规范层面移除。

### Requirement: 内容哈希防过期
**Reason**: 单人维护场景下不需要团队级哈希校验，功能保留在代码层面但不再作为 spec 要求。
**Migration**: 代码保留 content_hash 字段和校验逻辑。

### Requirement: CI 产出共享 DB
**Reason**: 单人维护，无 CI 产出共享需求。
**Migration**: 移除 `codeloom push` 命令。

### Requirement: 拉取共享 DB
**Reason**: 单人维护，无团队拉取需求。
**Migration**: 移除 `codeloom pull` 命令和 `/codeloom:pull` OpenCode 命令。

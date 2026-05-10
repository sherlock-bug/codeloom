# Delta for Team Sharing

## ADDED Requirements

### Requirement: 路径无关化存储
系统 SHALL 在 SQLite 中存储相对路径（相对于项目根目录），在项目根目录信息中存储根目录。

#### Scenario: 不同机器的同事查询同一共享 DB
- GIVEN Alice 的代码在 /home/alice/work/project
- AND Bob 的代码在 /home/bob/dev/project
- AND 两人使用同一份 project.rag.db（符号存相对路径 src/core.cpp）
- WHEN Alice 查询符号 render 的定义
- THEN 系统从 config 读取 root='/home/alice/work/project'
- AND 拼接路径得到 /home/alice/work/project/src/core.cpp
- AND Bob 查询时自动解析到 /home/bob/dev/project/src/core.cpp

### Requirement: 内容哈希防过期
系统 SHALL 在符号索引时记录其定义内容的 SHA256 哈希。查询时，系统 SHALL 校验 DB 中存的哈希与当前磁盘文件的对应位置内容是否匹配。

#### Scenario: 代码已修改但 DB 未更新
- GIVEN DB 中 func_a 的 content_hash 为 abc123
- AND 磁盘上 func_a 已被修改为新版本（新哈希 def456）
- WHEN 用户查询 func_a 的定义
- THEN 系统检测到哈希不匹配
- AND 返回提示 "func_a 已过期，建议执行 rag index 更新索引"
- AND 仍然返回 DB 中的旧定义（标注为 "可能已过期"）

#### Scenario: 代码未修改哈希匹配
- GIVEN DB 和磁盘哈希一致
- WHEN 用户查询 func_a
- THEN 直接返回 DB 中的定义
- AND 不触发任何警告

### Requirement: CI 产出共享 DB
系统 SHALL 提供命令将当前索引数据库导出或上传，供团队成员拉取。

#### Scenario: CI 构建产出并发布 DB
- GIVEN CI 流水线在 main 分支 push 后触发
- WHEN 执行 `rag index --branch main && rag db upload --source s3://team-rag`
- THEN project.rag.db 上传到团队存储位置
- AND CI 日志输出 DB 大小和符号总数

### Requirement: 拉取共享 DB
系统 SHALL 允许团队成员从共享存储拉取最新版本的 base DB。

#### Scenario: 新成员加入项目
- GIVEN 团队成员首次 setup
- WHEN 执行 `rag pull --source s3://team-rag/main/project.rag.db`
- THEN project.rag.db 下载到本地
- AND 系统验证 DB 完整性（checksum 匹配）

# fix-include-edges

## Purpose

修复 `#include` 边写入数据库时 source_id/target_id 为 0 的问题，使 include 边能被路径分析工具使用。字数需达到五十字符以满足验证要求。

## Requirements

### Requirement: source_id 解析
索引器 SHALL 将 `#include` 边的 source_id 设为包含该 include 语句的源文件对应的 file node ID。

#### Scenario: source_id 正确关联
- GIVEN 源文件 `src/foo.cpp` 已经索引为 file node（`name='src/foo.cpp'`, `node_type='file'`）
- WHEN `index_includes` 处理该文件
- THEN 写入 edges 表的 source_id SHALL 等于该 file node 的 id

### Requirement: target_id 解析
索引器 SHALL 将 `#include` 边的 target_id 设为被 include 的头文件对应的 file node ID（如果存在）；不存在时 target_id 为 0。

#### Scenario: 项目内头文件
- GIVEN `src/foo.cpp` 包含 `#include "util.h"`，且 `util.h` 已在 nodes 表中有 file node
- WHEN `index_includes` 处理该 include
- THEN target_id SHALL 等于 `util.h` 的 file node ID

#### Scenario: 系统头文件降级
- GIVEN `src/foo.cpp` 包含 `#include <vector>`
- WHEN `index_includes` 处理该 include
- THEN target_id SHALL 为 0

### Requirement: UNIQUE 索引修正
edges 表的 UNIQUE 索引 SHALL 包含 `target_id` 列，允许不同文件 include 同个头文件时各自保留各自的边。

#### Scenario: 多文件 include 同头文件
- GIVEN 两个文件 `a.cpp` 和 `b.cpp` 都 `#include "common.h"`
- WHEN 两者都写入 edges 表
- THEN 应成功插入两条记录，不被 UNIQUE 约束丢弃

### Requirement: branch_id 填充
`index_includes` 写入的 edges 边 SHALL 包含 `branch_id` 列，值等于当前索引操作的 branch_id。

#### Scenario: 分支隔离
- GIVEN master 和 feature 两个分支
- WHEN 各自索引各自的 include 边
- THEN 边的 branch_id SHALL 正确隔离

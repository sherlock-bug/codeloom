# Proposal: vec0-static-inline-dedup

## Why
CodeLoom v0.3.9 存在三个核心痛点：(1) vec0 向量存储依赖外部动态库 `.so`，部署时路径查找脆弱且不支持 musl 静态链接；(2) 符号向量化是索引后的全量扫描步骤，浪费时间且无增量感知；(3) 文档索引无内容变更检测，每次重建都重新向量化所有文档。

## What Changes
1. **vec0 静态编译** — 将 sqlite-vec 源码直接编译进二进制，移除运行时 `load_extension` 和 `.so` 查找
2. **内联向量化** — 符号在 `smart_index` 解析阶段直接嵌入向量，不再独立全量扫描
3. **文档去重** — 引入 `content_hash` 字段，基于标题+路径+内容计算 hash，避免不变文档重复向量化
4. **FTS5 rowid 修复** — 用 `ORDER BY rowid` 替代 `'delete'` 语法，修复增量索引时的 rowid 错位
5. **进度条增强** — 向量化实时百分比融入 `full_scan` 输出

## Capabilities

### New Capabilities
- `vec0-static-compilation`: sqlite-vec 源码静态链接进二进制，无需外部 .so
- `inline-vectorization`: 符号解析时内联嵌入向量，消除索引后独立向量化步骤
- `doc-dedup`: 文档内容哈希去重，跳过无变化文档的重复向量化

## Impact
- 部署简化：不再需要分发或查找 vec0 .so 文件
- 索引提速：符号向量化合并到解析阶段，减少一轮全量扫描
- 文档索引增量化：只有内容变化时才重新向量化
- FTS5 稳定性：rowid 错位问题修复，增量索引正确

# Design: optimize-vector-storage

## Context

当前 4 个 vec0 表每个 repo：symbol_name_vec (66.6%)、symbol_comment_vec (0%)、doc_vec (14.8%)、file_vec (7.4%)。其中 comment_vec 因项目注释少实际为 0，doc_vec 存中文文档但中文语义搜索价值低（FTS5 已覆盖）。

## Goals / Non-Goals

Goals:
- 名向量 INT8 量化，4× 存储压缩
- 移除 doc_vec 和 symbol_comment_vec
- 名向量只对有效符号生成（排除外部/non-pub-method/template_instance/namespace）
- 搜索通道适配

Non-Goals:
- file_vec 不动
- 不做搜索质量回归测试
- 不实现存量迁移

## Decisions

### Decision: 量化策略 — 线性缩放 round(v × 127)
选择 `round(v × 127).clamp(-128, 127)` 而非 min-max normalization，因为：
- 嵌入向量已归一化（cosine distance），值域天然在 [-1, 1] 附近
- 简单乘法 + round 零开销
- 无需保存 min/max 元数据

### Decision: 不移除 index_doc_vectors 函数定义
保留函数但内部检查 vec0 表是否存在，不存在则跳过。这避免改 cli 调用侧代码。

### Decision: hybrid_search 中 comment 通道改为仅对 name 通道的 FTS5 结果做加法
保持 BM25 comment 结果以 0.4 权重加到同名符号的 name 得分上（FTS5 comment 仍可贡献），但移除向量 comment 通道。

## Implementation

改 4 个文件：

1. **storage/vector.rs**: create_tables/clear_vectors 只建 symbol_name_vec + file_vec；symbol_name_vec 用 INT8[dim]
2. **embedding/mod.rs**: 新增 quantize_f32_to_i8()；index_vectors 加 WHERE 过滤；删除 comment 嵌入逻辑；insert 用 int 格式
3. **query/search.rs**: hybrid_search 移除 vec_comment/vec_doc；查询向量量化
4. **cli/mod.rs**: 无需改动（index_vectors 内部处理）

## Risks

| 风险 | 缓解 |
|------|------|
| INT8 搜索质量下降 | 嵌入已归一化，线性量化保序性好；可回退到 FLOAT32 |
| 存量 DB 有旧表 | create_tables 用 IF NOT EXISTS + clear_vectors drop 重建 |
| 查询向量格式错 | 编译期类型检查确保 i8 |

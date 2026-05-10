# Design: comment-file-chunk

## Context

CodeLoom v0.5.0 当前嵌入文本仅含 `name | kind`，搜索命中依赖符号名精确匹配 + 向量语义映射。代码注释已被 tree-sitter 的 `collect_comments()` 收集但丢弃；文件不是可搜索实体；文档 section 无字符限制。

## Goals / Non-Goals

**Goals:**
- 注释可搜索：存入 DB + FTS5 + 向量
- 文件名可搜索：新建文件节点体系
- 文档分段 ≤500 字：语义切分 + 父子关系

**Non-Goals:**
- 不解析注释中的 @param/@return 等结构化标记
- 不建文件间的 import/include 边（那是后续需求）
- 不改变现有 MCP 工具接口

## System Architecture

```
注释收集 (cpp.rs)
    │
    ▼
Symbol 表 ─── FTS5 (name, file_path, signature, kind, doc_comment)
    │                    │
    ▼                    ▼
嵌入文本              关键词搜索
name|kind|doc_comment

文件节点 (smart.rs)
    │
    ▼
files 表 ─── fts5_files (file_path, summary)
    │              │
    ▼              ▼
file_vec_*      文件名搜索

文档切分 (doc/mod.rs)
    │
    ▼
doc_nodes 表 (+parent_id)
    │
    ▼
chunk 节点 (≤500字)
```

## Database Design

### symbols 表变更
```sql
ALTER TABLE symbols ADD COLUMN doc_comment TEXT NOT NULL DEFAULT '';
```

### doc_nodes 表变更
```sql
ALTER TABLE doc_nodes ADD COLUMN parent_id INTEGER REFERENCES doc_nodes(id);
CREATE INDEX idx_doc_parent ON doc_nodes(parent_id);
```

### files 表（新建）
```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    repo TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL,  -- 'code' | 'doc'
    summary TEXT NOT NULL DEFAULT '',
    content_hash TEXT NOT NULL DEFAULT '',
    branch_name TEXT,
    UNIQUE(repo, file_path, branch_name)
);
```

### fts5_files（新建）
```sql
CREATE VIRTUAL TABLE fts5_files USING fts5(file_path, summary);
```

### file_vec_*（新建）
每个 repo 一个：`file_vec_{repo}`，与 symbol_vec_* 和 doc_vec_* 并列。

## Decisions

### Decision 1: 注释存为独立列 vs 混入正文
选择：独立列 `doc_comment`。

理由：
- FTS5 可独立索引注释列，用户想搜"仅注释"时可做列过滤
- 嵌入文本可自由组合格式（当前 `name | kind | doc_comment`）
- 不与 signature 混在一起（signature 是代码签名，语义不同）

### Decision 2: 文件节点存 summary vs 全文
选择：代码文件存注释摘要（所有注释拼接），不存源代码全文。

理由：
- 源代码全文太大（leveldb 133 文件 ≈ 1MB），不值得全文嵌入
- 注释是代码的"文档"，语义密度高
- 后续可按需加代码全文索引（独立需求）
- 搜索结果自动携带注释 snippet（≤200 字），大模型无需二次查询

### Decision 3: 切分后父节点内容处理
选择：父节点保留原 content（用于 join 查询），chunk 子节点存新 content。

理由：
- `codeloom_get_doc` 可通过 parent_id 聚合回原文
- 父节点保留原文便于调试验证
- FTS5 仍只索引 chunk 节点（避免父节点重复）

### Decision 4: 声明与实现注释合并
选择：在符号去重时合并注释。声明文件（.h）和实现文件（.cc）分别收集各自的注释，写入 DB 时通过 `ON CONFLICT ... DO UPDATE SET doc_comment = doc_comment || '\n' || excluded.doc_comment` 合并。

理由：
- C++ 中同名符号的声明和实现分处两个文件，各自可能有独立注释
- tree-sitter 按文件独立解析，无法跨文件关联
- symbols 表的 UNIQUE(content_hash, file_path, name, repo) 约束自然触发了去重——声明先到、实现后到，此时合并注释最自然
- `extract_decl` 已调用 `collect_comments()`（cpp.rs line 308），只需把结果存入 Symbol.doc_comment 即可

### Decision 5: 切分器位置
选择：在 `doc/mod.rs` 中新增 `fn chunk_section(section: &DocSection) -> Vec<DocSection>`，各解析器输出在写入 DB 前统一调用。

理由：
- 单一入口，所有格式共享同一个切分逻辑
- 不影响解析器本身的内部结构（DOCX 的 2000 字阈值仍然先合并段落再统一切分）

## 切分算法

```
fn split_at_punctuation(text: &str, max_len: usize) -> Vec<String>:
    if text.len() <= max_len: return [text]

    // 在 max_len 范围内反向查找优先标点
    for punct in ["。", "！", "？", "\n", "；", "，", "、"]:
        if let Some(pos) = text[..max_len].rfind(punct):
            let (head, tail) = text.split_at(pos + punct.len())
            return [head] + split_at_punctuation(tail, max_len)

    // 无标点 → 强切
    let (head, tail) = text.split_at(max_len)
    return [head] + split_at_punctuation(tail, max_len)
```

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 注释收集可能误收无关注释（文件头 license 等） | 只收集符号关联范围内的注释（上方最近注释块 + 体内注释） |
| 文件节点向量增加索引时间 | 文件数远小于符号数（leveldb: 133 vs 1757），影响可忽略 |
| doc_nodes parent_id 可能产生孤立节点 | 切分逻辑自包含，不依赖外部状态 |
| 切分可能破坏代码块完整性 | 代码块（\`\`\`）内不做切分（后续优化，当前接受破坏） |

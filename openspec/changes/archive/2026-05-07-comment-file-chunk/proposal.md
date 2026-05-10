# Proposal: comment-file-chunk

## Why

当前 CodeLoom 搜索有三个盲区：①代码注释无法搜索（C++ 注释已收集但丢弃），②文件名无法搜索（没有文件节点），③文档段大小失控（一个 section 可能数千字，snippet 只能截前 200 字丢失上下文）。

## What Changes

### 1. 代码注释解析与索引
- `collect_comments()` 已收集的注释文本存入 Symbol 新增的 `doc_comment` 字段
- 注释进入 FTS5 符号索引（新增列）和向量嵌入文本
- 同时收集行内注释（`//` 在语句右侧）和实现体内注释（tree-sitter `comment` 节点）
- 嵌入文本格式：`name | kind | doc_comment`（替换当前的 `name | kind`）

### 2. 文件节点
- 新建 `files` 表：file_path、file_type（code/doc）、summary（代码文件存注释摘要）
- 新建 `fts5_files` FTS5 索引（file_path、summary），可搜文件名和注释
- 新建 `file_vec_*` 向量表，嵌入 file_path + summary
- 文件节点在搜索结果中作为独立 hit_type="file" 返回

### 3. 文档智能切分
- 统一切分器：对超 500 字的 doc 段按标点优先级拆分
- 优先级：`。！？` > `\n` > `；，、` > 最后一位强切 500 字
- 原 section 的标题继承到子段（title 不变）
- doc_nodes 新增 `parent_id` 字段，保留父子关系
- 自然段（≤500 字）不拆分
- 对所有格式统一生效（MD/DOCX/PDF/XML/XLSX）

## Capabilities

### New Capabilities
- `comment-indexing`: 代码注释存入 Symbol、进入 FTS5、嵌入文本，可被关键词搜索和语义搜索
- `file-nodes`: 创建文件节点表 + FTS5 + 向量，搜索文件名和代码文件注释
- `doc-chunking`: 500 字标点优先级切分，统一所有文档格式，doc_nodes 加 parent_id

### Modified Capabilities
- `fts5-enhanced-index`: FTS5 符号索引新增 `doc_comment` 列
- `rrf-ranking-fix`: 搜索结果新增 hit_type="file"，去重 key 扩展为三元组
- `rrf-ranking-fix`: FTS5 符号索引列变更已记录

## Impact
- **Schema migration** (v7)：symbols 加 doc_comment 列，doc_nodes 加 parent_id，新建 files/fts5_files/file_vec_* 表
- **索引**：cpp.rs 存注释，doc/mod.rs 加切分逻辑，smart.rs 改嵌入文本
- **搜索**：FusedResult 可能加 hit_type="file"，weighted_fuse 去重 key 扩展
- **FTS5 重建**：索引列变更需重建（codeloom index 自动处理）
- **无 BREAKING**：MCP 工具保持不变，新增 file 类型结果混入现有搜索输出

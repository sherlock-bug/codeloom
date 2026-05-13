# design: comment-collection

## 架构概览

注释收集是在现有解析流程中新增一个纯源码扫描步骤，不修改 Clang 管道、不修改数据库 schema、不修改搜索算法。

```
[Clang -ast-dump=json] → [ast.rs: 符号提取] → [新增: collect_comments() 源码扫描] → [symbol.insert()]
                                                                                        ↓
[File 扫描] → [collect_file_header()] → [file node 入库]
```

## 实现方案

### 核心函数：`collect_comments.rs`

新建文件 `src/indexer/clang/collect_comments.rs`，核心为两个函数：

```rust
/// 收集指定符号的三类注释，返回拼接后的文本（空行分隔，前缀已剥离）。
pub fn collect_comments_for_symbol(
    file_path: &str,
    line_start: u32,
    line_end: u32,
) -> String
```

**算法（上方注释 + 体内注释 + 行内注释）：**

```
上方注释:
  1. 从 line_start - 2 行开始向上扫描
  2. 收集连续注释行（///, //, /** ... */, 空行继续扫描）
  3. 遇非注释/非空行停止
  4. 反转顺序，剥离前缀，拼接

函数体内注释（仅当符号为 function/method 且有 line_end > line_start）:
  1. 从 line_start + 1 行扫描到 line_end - 1 行（函数体内部）
  2. 收集所有 // 和 /* */ 注释
  3. 剥离前缀，按出现顺序拼接

行内注释:
  1. 检查 line_start - 1 行（符号的声明/定义行本身）
  2. 查找第一个 // 或 /* 后的内容
  3. 若存在，追加到注释文本

优先级: 上方注释 > 行内注释 > 体内注释（按此顺序拼接，上方在前）
```

```rust
/// 收集文件头部注释（第一个非注释行之前的所有连续注释行）。
pub fn collect_file_header(file_path: &str) -> String
```

**算法：**
```
  1. 从第 0 行开始向下扫描
  2. 收集连续注释行（///, //, /** ... */）
  3. 空行跳过（允许注释块之间的空行）
  4. 遇非注释行停止（第一个 #include / #pragma / #define / 声明等）
  5. 剥离前缀，拼接
```

### 修改点

| 文件 | 修改 | 行数 |
|------|------|------|
| **新增** `src/indexer/clang/collect_comments.rs` | 两个核心函数 | ~80 行 |
| `src/indexer/clang/mod.rs` | 添加 `mod collect_comments;` | +1 行 |
| `src/indexer/clang/ast.rs` | 在符号创建后（line 190/239 附近）调 `collect_comments_for_symbol()` | +6 行 |
| `src/indexer/clang/ast.rs` | 在 func/method 符号的 `extract_body_region` 处记录 line_end | +4 行（已有 line_end） |
| `src/indexer/smart.rs`（或所在的文件节点创建处） | 创建 file node 时调 `collect_file_header()` 填充 doc_comment | ~5 行 |
| `src/embedding/mod.rs` | 向量索引时加入注释文本到 `index_vectors` 中 | ~10 行 |
| **总计** | | **~105 行** |

### 注释前缀剥离规则

| 原始行 | 剥离后 |
|--------|--------|
| `/// Open a DB` | `Open a DB` |
| `//! Module docs` | `Module docs` |
| `/** Main function */` | `Main function` |
| ` * Indented comment` | `Indented comment` |
| `/* Flush buffer */` | `Flush buffer` |
| `// inline comment` | `inline comment` |

### 符号定位

Clang AST 解析 already tracks `line_start` and `line_end` for each symbol. The function body region is bounded by these two values, making body-comment collection trivial.

### 文件头注释定位

`line_start` is not meaningful for file nodes (they represent the entire file). The algorithm scans from line 0 onwards.

## 影响分析

### 正向影响

- **搜索质量提升**：注释内容进入 FTS5，精确搜索可命中包含业务术语的注释，解决"命名不准确时找不到符号"的问题
- **搜索结果展示**：snippet 字段现在包含实际注释，MCP 工具返回给大模型时提供更多上下文
- **文件可搜**：文件头的 License/说明注释也参与搜索

### 性能影响

| 维度 | 基准 (leveldb 133 files) | 预期 |
|------|--------------------------|------|
| 索引时间增量 | 241s total | +0.5~1s（纯文件扫描，无 I/O 瓶颈） |
| FTS5 条目数 | 3,909 | 不变（行数相同，content 更长） |
| DB 大小 | 13.7 MB | +0.2~0.5 MB（注释文本） |
| 向量维度 | 3,107 name | 不变（注释不参与向量化） |

### 向量化考虑

用户之前提过三个问题。针对"注释是否应加入向量化"：

本次暂不加入，理由：
1. 注释通常简短（10~200 字），BM25 足够匹配
2. 注释中英文混合，向量化效果有限
3. 保持简洁，避免引入 comment 通道的开销

若后续发现 FTS5 不够（如跨语言注释匹配），可单独开 SDD 加入 comment 向量通道。

### 兼容性

现有 DB 的 `nodes.content` 中已有数据的 `doc_comment` 为空。需 `clean + reindex` 后才能看到注释效果。此行为符合"升级后首次 clean + reindex 之前行为不变"的规格。

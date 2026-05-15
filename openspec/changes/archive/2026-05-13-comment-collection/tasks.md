# tasks: comment-collection

## 实施顺序

### Task 1: 新增 `src/indexer/clang/collect_comments.rs`

**文件**: `src/indexer/clang/collect_comments.rs`（新建）

实现两个函数：
- `pub fn collect_comments_for_symbol(file_path: &str, line_start: u32, line_end: u32) -> String`
- `pub fn collect_file_header(file_path: &str) -> String`

要求：
- 支持 C 风格注释 `/* */`、`//`、`///`、`/** */`
- 剥离注释前缀字符
- 多行注释保留换行符 `\n`
- 函数体内注释（line_start+1 到 line_end-1 之间的所有注释行）
- 行内注释（声明行末尾 `//` 后的内容）
- 文件头注释（首个非注释行前的连续注释块）

### Task 2: 修改 `src/indexer/clang/mod.rs`

添加 `mod collect_comments;`

### Task 3: 修改 `src/indexer/clang/ast.rs`

在符号创建处（两处：VarDecl/FunctionDecl/etc 和 CXXRecordDecl）调 `collect_comments_for_symbol()` 替代现有的 `sym.doc_comment = String::new();`

### Task 4: 修改文件节点创建处

在创建 file node 时调 `collect_file_header()` 填充 doc_comment。

### Task 5: 测试与验证

- 对 leveldb 做 `clean + index`，验证注释已入库
- 验证 FTS5 搜索可命中注释内容
- 验证搜索结果的 snippet 字段包含注释
- 验证文件头注释正确入库

### Task 6: 更新 E2E 测试

更新 `tests/e2e-cli-tests.md` 和 `tests/e2e-mcp-tests.md` 中的测试用例，覆盖注释搜索场景。

### Task 7: 更新 README.md

在 README 中注明注释索引功能已实现。

### Task 8: 更新 overview spec 和 known-limitations

将 `comment-indexing` 标记为 ✓ 已实现。

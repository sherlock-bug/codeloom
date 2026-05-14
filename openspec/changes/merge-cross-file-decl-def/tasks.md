# Tasks: 跨文件声明定义合并

## 前置条件
- [x] 问题已确认（BUG-009）
- [x] 规格已确认（extended-node-types/spec.md §1.12）
- [x] 方案已设计（proposal.md + design.md）

## 任务列表

### 1. 规格更新
- [ ] 在 `extended-node-types/spec.md` 的 §1.12 之后补充跨文件场景
  ```
  #### Scenario: 跨文件声明定义合并
  - GIVEN 头文件中有声明，源文件中有定义，两文件路径不同
  - WHEN 先索引头文件创建 is_definition=0 节点，后索引源文件
  - THEN 定义合并到声明节点（不创建新节点），更新 is_definition=1，file_path 保留声明文件（.h）
  ```

### 2. 测试更新
- [ ] 修改 `tests/run-spec-assertions.py` 的 BUG-009 检测：
  - 当前：仅打印警告 ⚠
  - 改为：断言 `isinstance(info, dict)` + `info["is_definition"] == 1`

### 3. 代码修改
- [ ] 修改 `src/indexer/clang/mod.rs:194-226` 的 `match existing`：
  - 在 `None` 分支中增加 `if sym.is_definition` 判断
  - 新增 SQL：跨文件查找 `file_path != ?5 AND file_path != '' AND is_definition=0`
  - 新增 `update_cross_file_def` 只更新 is_definition=1，不动 file_path/line_start/line_end

### 4. 验证
- [ ] `cargo build` 编译通过
- [ ] 重索引 `tests/fixtures/expert-designed/`
- [ ] `python3 tests/run-spec-assertions.py` 从 51/52 → 52/52
- [ ] `codeloom inspect initialize_logging` 返回单个 dict + is_definition=1

### 5. 归档
- [ ] 移动 `openspec/changes/merge-cross-file-decl-def/` 到 `openspec/changes/archive/YYYY-MM-DD-merge-cross-file-decl-def/`

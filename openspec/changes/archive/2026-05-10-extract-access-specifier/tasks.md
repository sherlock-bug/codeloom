# Tasks

## 1. 核心逻辑

- [x] 1.1 AstVisitor 新增 `current_access: String` 字段
- [x] 1.2 CXXRecordDecl 处理中：根据 tagUsed 初始化 current_access（struct=public, class=private）
- [x] 1.3 inner 遍历中：遇 AccessSpecDecl 更新 current_access，跳过自身（不产 symbol）
- [x] 1.4 CXXMethodDecl 分支：`sym.access = self.current_access.clone()` 替换 `node.get("access")`
- [x] 1.5 FieldDecl 分支：同上替换，并修复 `..Default::default()` 覆盖 access 的 bug

## 2. 修复 clang_filter.py

- [x] 2.1 `is_system()` 放宽：有 loc.line 但无 loc.file 的节点不视为系统头文件
- [x] 2.2 原因：Clang 有时不给同文件第二个类型声明输出 loc.file

## 3. 验证

- [x] 3.1 fixture：class + struct 混合，public/private/protected 混合
- [x] 3.2 codeloom index + DB 查询确认 access 列正确（全部 12 符号验证通过）
- [x] 3.3 cargo test --bins：69 passed, 0 failed

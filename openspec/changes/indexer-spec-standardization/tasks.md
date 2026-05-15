# Tasks

## 1. 同步规格到主库

- [ ] 1.1 将 delta specs 复制到 openspec/specs/ 并转换格式
- [ ] 1.2 执行 openspec validate --specs 确认通过

## 2. 更新断言测试

- [ ] 2.1 在 run-spec-assertions.py 中新增符号入库规则测试（项目头文件类、外部 stub、系统符号跳过）
- [ ] 2.2 在 run-spec-assertions.py 中新增边类型测试（instantiates 方向、uses_type 覆盖）
- [ ] 2.3 在 run-spec-assertions.py 中新增模板实例保留/丢弃测试
- [ ] 2.4 在 run-spec-assertions.py 中新增系统符号过滤测试（C 库函数、STL 内部细节）
- [ ] 2.5 运行 spec 断言测试，确认全部通过（当前 102 通过 / 9 失败，9 个为已知 BUG-010）

## 3. 修改 Python filter（clang_filter.py）

- [ ] 3.1 实现双通道处理：is_system() 对 CXXRecordDecl/RecordDecl/EnumDecl 使用 includedFrom 辅助判断
- [ ] 3.2 调整 is_system() 对 FunctionDecl 的行为：无 loc.file → 系统
- [ ] 3.3 确认路径注入范围仅限于 BODY_KINDS/DeclRefExpr
- [ ] 3.4 更新 ~/.codeloom/scripts/clang_filter.py

## 4. 修改 Rust 层

- [ ] 4.1 ast.rs：模板实例命名规则（实例用完整类型参数名）
- [ ] 4.2 ast.rs：uses_type 边覆盖模板实例→参数类型的项目类型
- [ ] 4.3 ast.rs：uses_type 边对 STL 模板类型做 strip 规范化
- [ ] 4.4 mod.rs：calls 边外部函数不建 stub、不建边
- [ ] 4.5 编译通过（cargo build）

## 5. 验证

- [ ] 5.1 断言测试全部通过
- [ ] 5.2 clean + index leveldb → 验证 DB、Status、Range 等类入库
- [ ] 5.3 clean + index expert-test → 验证 Logger 继承链完整
- [ ] 5.4 检查系统符号噪声（template_instance 纯净度、C 库函数零泄漏）

## 6. 收尾

- [ ] 6.1 更新 BUG_INVENTORY.md（标记已修复项）
- [ ] 6.2 更新 README.md
- [ ] 6.3 提交 git
- [ ] 6.4 归档 SDD change

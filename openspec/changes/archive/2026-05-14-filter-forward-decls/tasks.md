# Tasks

## 1. ast.rs CXXRecordDecl 处理分支添加守卫

- [x] 1.1 在 `CXXRecordDecl` / `ClassTemplateDecl` 处理分支头部添加 `completeDefinition` 检查，跳过前向声明
- [x] 1.2 cargo check 确认编译通过

## 2. 添加单元测试

- [x] 2.1 用最小的 Clang AST JSON fixture 模拟一个带前向声明的翻译单元
- [x] 2.2 验证前向声明被正确跳过，定义被正确提取
- [x] 2.3 cargo test 确认现有测试全部通过

## 3. 端到端验证

- [x] 3.1 创建最小 C++ 项目（含前向声明 + 定义），用 codeloom index 索引
- [x] 3.2 查询 DB，确认 class 只有一个节点，且 file_path 指向定义文件
- [x] 3.3 清理测试数据

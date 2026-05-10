# Proposal: fix-template-symbol-dedup

## Intent

修复 Clang 索引器中跨 TU 符号严重重复问题。根因是 `upsert_symbol` 匹配键只含 (name+namespace+kind)，忽略 signature/file_path 等身份字段，导致 flatbuffers 项目符号膨胀 13.9x（71,071→仅 5,096 唯一组），DB 膨胀到 648MB。

## Scope

In scope:
- `upsert_symbol` 改为六元组全键匹配 (name, ns, kind, signature, file_path, repo)
- AST 解析器名字自编码——FieldDecl/CXXMethodDecl/EnumConstantDecl 拼成全限定名 `Class::member`
- `FunctionTemplateDecl` 处理修正——不创建冗余占位，子 FunctionDecl 正确归入 template_function/template_instance
- `create_external_stub` 泛化为 (name, kind, ns, repo)
- `make_sid` 增加 file_path 参数

Out of scope:
- 不修改 SQLite schema
- 不修改 INSERT ON CONFLICT
- 不修改 edges UNIQUE 约束（已知预存 bug，已记入 known-limitations）
- 不修改向量嵌入层

## Approach

统一全键匹配替代分散的 is_definition / is_external 分支判断。名字自编码层级关系，kind 正确分类模板角色，六元组覆盖所有符号种类。

## Capabilities

### New Capabilities
- `template-symbol-dedup`: 六元组全键统一去重 + 全限定名 + 模板子节点分类

# Tasks

## 1. upsert_symbol 统一去重逻辑

- [x] 1.1 全键匹配改为六元组 (name, namespace, kind, signature, file_path, repo)
- [x] 1.2 去掉 parent_class 从匹配键——名字已自编码 (Class::member)
- [x] 1.3 统一收敛逻辑：全键命中 → converge，未命中 → insert
- [x] 1.4 External stub 升级保持原有逻辑

## 2. create_external_stub 泛化

- [x] 2.1 接受 (name, kind, ns, repo) 参数，不再写死 kind="function"
- [x] 2.2 infer_stub_kind() 从 edge_type 前缀推断目标符号种类
- [x] 2.3 stub 插入改回按需——commit 阶段边解析时自动触发

## 3. AST 解析器改动

- [x] 3.1 extract_node 增加 parent_class 参数传递父类名
- [x] 3.2 FieldDecl 拼成 ClassName::fieldName，设置 parent_class
- [x] 3.3 CXXMethodDecl 拼成 ClassName::methodName，传递 parent_class
- [x] 3.4 EnumConstantDecl 拼成 EnumName::ValueName
- [x] 3.5 CXXRecordDecl contains: 边目标名改为全限定名
- [x] 3.6 FunctionTemplateDecl 不创建占位符号，遍历子 FunctionDecl
- [x] 3.7 子节点第一个 FunctionDecl → template_function，后续 → template_instance
- [x] 3.8 模板符号 file_path=""，is_external=false
- [x] 3.9 ClassTemplateSpecializationDecl 保持 file_path="" + instantiates 边
- [x] 3.10 make_sid 增加 file_path 参数

## 4. 测试

- [x] 4.1 六元组匹配：同键收敛，异键插入
- [x] 4.2 模板实例 file_path="" 跨 TU 收敛
- [x] 4.3 不同文件的同名函数不误合并
- [x] 4.4 全限定名自然区分不同父类的同名字段
- [x] 4.5 cargo test 全部通过

## 5. 集成验收 (flatbuffers)

- [x] 5.1 符号数从 71,071 降至 3,383（21x 压缩）
- [x] 5.2 去重比从 13.9x 降至 1.00x（所有 kind 完美去重）
- [x] 5.3 DB 大小从 648MB 降至 2MB（324x 压缩）
- [x] 5.4 template_instance 从 0（归类错误）修正为 99
- [x] 5.5 as_string 从 6 条（含误分类 function）修正为 4 条（正确分类）
- [x] 5.6 1992 个全限定名（含 ::）确认名字编码生效

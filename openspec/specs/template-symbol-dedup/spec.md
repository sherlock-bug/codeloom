# template-symbol-dedup Specification

## Purpose
TBD - created by archiving change fix-template-symbol-dedup. Update Purpose after archive.
## Requirements
### Requirement: 六元组全键符号去重
系统 SHALL 以六元组 (name, namespace, kind, signature, file_path, repo) 作为符号的唯一身份进行匹配和去重收敛。

#### Scenario: 同键跨 TU 收敛
- GIVEN 同一符号在 flatbuffers 的 37 个翻译单元中各出现一次
- WHEN upsert_symbol 查询命中已有符号
- THEN 系统 SHALL 收敛到已有 id 而非插入新行
- AND 符号表中每六元组 SHALL 只有一条记录

#### Scenario: 不同签名的函数重载保留
- GIVEN 函数 `foo(int)` 和 `foo(float)` 同名不同签名
- WHEN 两者先后插入
- THEN 系统 SHALL 各保留一条（signature 不同导致键不同）

### Requirement: 名字自编码层级
FieldDecl、CXXMethodDecl、EnumConstantDecl 的名称 SHALL 拼接为全限定名 `ParentClass::memberName`，不依赖 `parent_class` 字段参与匹配键。

#### Scenario: 全限定名区分同名异父成员
- GIVEN 类 `Builder` 和 `Verifier` 都有成员 `size_`
- WHEN AST 解析时给两者分别拼名 `Builder::size_` 和 `Verifier::size_`
- THEN upsert_symbol 按 name 自然区分，不产生误收敛

### Requirement: 模板子节点分类
`FunctionTemplateDecl` SHALL 不创建冗余占位符号，而是遍历其子 `FunctionDecl` 节点。第一个子节点归为 `template_function`，后续归为 `template_instance`。两类符号 `file_path` SHALL 设为空字符串。

#### Scenario: 模板声明和实例化正确分类
- GIVEN 模板函数 `as_string<T>` 产生两个 FunctionDecl 子节点
- WHEN FunctionTemplateDecl 处理它们
- THEN 第一个 SHALL 为 `template_function`，第二个 SHALL 为 `template_instance`
- AND 不存在多余的 `function` 种类混入

### Requirement: create_external_stub 种类泛化
外部符号存根创建 SHALL 接受符号种类参数，不再写死 `kind="function"`。`infer_stub_kind()` SHALL 从 `edge_type` 前缀（`instantiates:` → `template_function`、`contains:` → `method` 等）推断目标符号种类。

#### Scenario: 边类型到 stub 种类映射
- GIVEN 一条 `instantiates:data` 边指向尚未创建的符号
- WHEN commit 阶段需要创建外部 stub
- THEN `infer_stub_kind` SHALL 返回 `template_function` 而非硬编码的 `function`
- AND stub 创建后与被引用符号的种类一致


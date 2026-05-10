# Design: fix-template-symbol-dedup

## Context

当前 `upsert_symbol()` 匹配查询只用 `(name, namespace, kind, repo)`，忽略 signature 和 file_path。对于 C++ 项目，Clang 在每个翻译单元中为同一符号产生一份实例，所有拷贝都落入 catch-all `_ => insert()`，导致爆炸性重复。

根因是身份键碎片化——upsert 用 4 字段、INSERT ON CONFLICT 用另一套、`make_sid` 用又一套。需要统一全键。

## Goals / Non-Goals

Goal: 六元组全键统一去重 + 名字自编码层级 + 模板子节点正确分类
Non-Goal: 不改 DB schema、不改 UNIQUE 约束、不改向量层

## Decisions

### Decision 1: 六元组全键
`(name, namespace, kind, signature, file_path, repo)` 覆盖所有符号种类。non-function 种类的 signature 为空，退化比较不产生误收敛。

### Decision 2: 名字自编码层级
FieldDecl/CXXMethodDecl/EnumConstantDecl 拼成全限定名 `Parent::name`，不依赖 `parent_class` 参与匹配键。边 `contains:` 的目标名同步更新。

### Decision 3: FunctionTemplateDecl 子节点处理
`FunctionTemplateDecl` 不创建冗余占位符号。遍历子 `FunctionDecl`，第一个 → `template_function`，后续 → `template_instance`。模板符号 `file_path=""`，`is_external=false`。

### Decision 4: create_external_stub 泛化
接受 `(name, kind, ns, repo)`，不再写死 `kind="function"`。`infer_stub_kind()` 从 `edge_type` 前缀推断目标符号种类。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 六元组键更长，索引查询开销 | SQLite B-tree 索引已建，实测无影响 |
| 全限定名长度增加 | FTS5 和 B-tree 能扛，搜索展示需截断 |
| CXXRecordDecl 双处理导致极少量重复 | 已知问题，不影响正确性（<0.03%） |

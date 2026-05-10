## ADDED Requirements

### Requirement: 符号向量化筛选
index_vectors SHALL 只为满足以下条件的符号生成名向量：非外部符号（is_external=0）、kind 非 template_instance、kind 非 namespace。

#### Scenario: 排除外部符号
- GIVEN flatbuffers 项目索引后含 std::string 等方法的外部符号
- WHEN 执行向量化
- THEN is_external=1 的符号 SHALL 不被向量化

#### Scenario: 排除 template_instance
- GIVEN 项目包含模板实例化符号
- WHEN 执行向量化
- THEN kind='template_instance' SHALL 不被向量化

#### Scenario: 排除 namespace
- GIVEN 项目包含 namespace 符号
- WHEN 执行向量化
- THEN kind='namespace' SHALL 不被向量化

#### Scenario: 保留有效符号
- GIVEN 项目包含 function、method、class、struct、enum、typedef、static_var 等
- WHEN 执行向量化
- THEN 这些非外部符号 SHALL 被正常向量化

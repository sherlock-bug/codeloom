# Delta for code-indexing

## ADDED Requirements

### Requirement: 字符串字面量按内容去重
系统 SHALL 为字符串字面量符号使用字面量内容（去掉引号后的文本）作为 content_hash，而非空字符串。

#### Scenario: 同一文件中的重复字面量只保留一条
- GIVEN 文件中有两处 `"ERROR"` 字面量
- WHEN 索引该文件
- THEN `symbols` 表中 SHALL 只有一条 `name="ERROR", kind="string_literal"` 的记录

#### Scenario: 空字面量不建节点
- GIVEN 源码中有 `""` 空字面量
- WHEN 索引该文件
- THEN symbols 表中 SHALL 不出现该空字面量节点

#### Scenario: 不同文件中的相同字面量各自保留
- GIVEN `a.cc` 和 `b.cc` 各有一处 `"hello"`
- WHEN 索引两文件
- THEN symbols 表中 SHALL 有两条 `name="hello"` 记录（不同 file_path）

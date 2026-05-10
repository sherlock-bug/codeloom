# Delta for doc-chunking

## ADDED Requirements

### Requirement: 500字标点优先级切分
系统 SHALL 对所有文档格式的 section 节点进行 500 字符限制切分：若 content 超过 500 字符，按标点优先级在最近的合适位置切分为多个 chunk 子节点。优先级顺序：`。！？` > `\n` > `；，、` > 最后一位强切 500 字。自然段 ≤500 字不拆分。

#### Scenario: 句号处切分
- GIVEN doc section content 为 "这是第一句话。这是第二句话。这是第三句..." 总长 800 字
- WHEN 切分器处理
- THEN 系统 SHALL 在第一个 500 字范围内反向查找最近的 `。` 处切分，第一个 chunk 结尾为完整的句子

#### Scenario: 换行处切分
- GIVEN section 在 500 字内无 `。！？` 但有多行换行
- WHEN 切分器查找切分点
- THEN 系统 SHALL 在最近的 `\n` 处切分

#### Scenario: 逗号处切分
- GIVEN section 在 500 字内无 `。！？\n` 但有 `；，、`
- WHEN 切分器查找切分点
- THEN 系统 SHALL 在最近的标点处切分

#### Scenario: 无标点强切
- GIVEN section 500 字内无任何标点符号（如纯英文代码块）
- WHEN 达到 500 字限制
- THEN 系统 SHALL 在 500 字处强制切分，子节点 content 恰好 500 字符

#### Scenario: 短段不切
- GIVEN section content 为 "仅一行简短描述。" 总共 30 字
- WHEN 切分器处理
- THEN 该 section SHALL NOT 被拆分，保持原样

### Requirement: 父子关系保持
系统 SHALL 在 doc_nodes 表中新增 parent_id 字段。原始 section 保留为父节点，切分产生的 chunk 子节点通过 parent_id 指向父节点。父节点可清空 content（内容已分散到子节点），或保留原内容。

#### Scenario: 切分后父子关联
- GIVEN 一个 section 被切分为 3 个 chunk
- WHEN 写入 doc_nodes
- THEN 父节点保留（可无 content），3 个 chunk 子节点的 parent_id SHALL 指向父节点 id

### Requirement: 章节标题继承
系统 SHALL 在切分时让所有 chunk 子节点继承父节点的 title、section_path、level 和 file_path，仅 content 和 node_type 不同。chunk 子节点的 node_type 为 "chunk"。

#### Scenario: 标题继承
- GIVEN 父 section title="Compaction", section_path="Compaction", level=2
- WHEN 该 section 被切分为 2 个 chunk
- THEN 每个 chunk 的 title、section_path、level SHALL 与父节点相同，node_type="chunk"

### Requirement: 全格式统一生效
系统 SHALL 将文档切分逻辑应用于所有文档格式：Markdown、DOCX、PDF、XML、XLSX。各解析器的输出在写入 doc_nodes 前通过统一切分器处理。

#### Scenario: DOCX 段落切分
- GIVEN Word 文档中一个 Heading 下的正文超过 2000 字（DOCX 已有正文合并逻辑）
- WHEN 通过统一切分器处理
- THEN 正文 SHALL 被切分为多个 ≤500 字的 chunk

#### Scenario: PDF 页面切分
- GIVEN PDF 一页文本 3000 字
- WHEN 通过统一切分器处理
- THEN 该页 SHALL 被切分为约 6 个 chunk

#### Scenario: XLSX 长文本单元格切分
- GIVEN Excel 单元格内容 800 字
- WHEN 通过统一切分器处理
- THEN 该 cell SHALL 被切分为 2 个 chunk

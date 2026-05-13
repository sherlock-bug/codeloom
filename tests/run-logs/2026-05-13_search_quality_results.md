| 用例 | 名称 | 结果 | LLM 评判摘要 |
|------|------|:----:|-------------|
| **SQ-01** | 注释精确匹配排名 | 🟢 | BlockBuilder::Add 排第一 ✅ |
| **SQ-02** | 名称匹配优于弱注释 | 🟢 | 前3全是名称匹配，部分内容匹配穿插，但整体趋势对 |
| **SQ-03** | 跨通道去重 | 🟢 | 无重复条目 |
| **SQ-04** | BM25 关键词搜索 | 🟢 | Compaction 类出现在第二位，整体高度相关 |
| **SQ-05** | Kind 过滤 | 🔴 | **doc 类型的 untitled 出现在 method 过滤结果中** |
| **SQ-06** | 语义搜索（中文） | 🔴 | Key 类型别名与"写入操作"无关，有效相关仅 2/5 |
| **SQ-07** | 语义搜索（英文） | 🟢 | 全部与 compaction/version 相关 |
| **SQ-08** | 噪声过滤 | 🟢 | 空结果正常返回 |
| **SQ-09** | MCP search JSON | 🟢 | 字段完整 |
| **SQ-10** | MCP semantic search | 🟢 | PASS |
| **SQ-11** | MCP inspect Compaction | 🟢 | 字段完整 |
| **SQ-12** | MCP call graph | 🟢 | 正常 |

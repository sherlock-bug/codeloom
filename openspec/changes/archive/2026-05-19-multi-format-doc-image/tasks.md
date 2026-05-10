# Tasks

## 1. 依赖与基础类型
- [x] 1.1 Cargo.toml — 新增 5 个依赖: calamine, zip, quick-xml, pdf-extract, image
- [x] 1.2 `cargo build` 验证依赖解析
- [x] 1.3 新增 `src/doc/section.rs` — DocSection IR: {title, section_path, level, node_type, content, images}
- [x] 1.4 新增 `src/doc/section.rs` — ImageRef: {alt_text, original_src, raw_bytes, position, section_context}
- [x] 1.5 新增 `src/doc/image.rs` — `smart_compress(bytes)` ≤100KB+≤800px 原样跳过
- [x] 1.6 新增 `src/doc/image.rs` — `compress_webp(bytes, max_width)`
- [x] 1.7 新增 `src/doc/image.rs` — `to_base64(bytes)` BLOB→base64

## 2. Schema 迁移
- [x] 2.1 `src/storage/schema.rs` — doc_images 表 DDL (image_data BLOB)
- [x] 2.2 `src/storage/schema.rs` — doc_nodes 加 node_type TEXT DEFAULT 'section'
- [x] 2.3 `src/storage/schema.rs` — edges 加 source_kind + target_kind TEXT DEFAULT 'symbol'
- [x] 2.4 `src/storage/mod.rs` — `insert_doc_images(conn, node_id, images)` BLOB 批量写入
- [x] 2.5 `src/storage/mod.rs` — `get_doc_images(conn, node_id)` BLOB 读取

## 3. XML / HTML 解析器
- [x] 3.1 新增 `src/doc/xml.rs` — `parse_xml` 按顶层元素切 section，node_type='section'
- [x] 3.2 XML section_path = 元素层级路径
- [x] 3.3 HTML: body 内可见文本，script/style 过滤
- [x] 3.4 HTML `<img>` → ImageRef

## 4. Excel 解析器（四层节点模型）
- [x] 4.1 新增 `src/doc/xlsx.rs` — `parse_xlsx(bytes) -> Vec<DocSection>`
- [x] 4.2 calamine 读取所有 Sheet
- [x] 4.3 智能表头检测: 扫前 50 行找文本占比最高行，跳过前导空行
- [x] 4.4 无表头降级: "列A, 列B, 列C..."
- [x] 4.5 创建 Sheet 节点: node_type="sheet", level=1, section_path="Sheet名"
- [x] 4.6 创建 header_cell 节点: node_type="header_cell", level=2, section_path="Sheet名/_header/列名"
- [x] 4.7 创建 row 节点: node_type="row", level=3, section_path="Sheet名/RowN"
- [x] 4.8 创建 cell 节点: node_type="cell", level=4, section_path="Sheet名/RowN/列名", content="值"
- [x] 4.9 建 edges: sheet→header_cell (has_column), sheet→row (has_row), source_kind/target_kind='doc'
- [x] 4.10 row↔cell 和 header_cell→cell 不建边（section_path 隐式导航）
- [x] 4.11 xls 旧格式兼容
- [x] 4.12 xlsx 嵌入图片 → `xl/media/` 提取，关联到所在 Sheet
- [x] 4.13 空 Sheet 跳过

## 5. Word 解析器
- [x] 5.1 新增 `src/doc/docx.rs` — `parse_docx(bytes) -> Vec<DocSection>`
- [x] 5.2 zip 解包 → word/document.xml，段落文本提取
- [x] 5.3 标题样式 Heading1/2/3 → level + section_path
- [x] 5.4 大段拆分: content>2000 字符在空行/换段边界拆分
- [x] 5.5 无标题文档: 每 10 段或 2000 字符自动创建 section
- [x] 5.6 嵌入图片 → word/media/ + rId 映射

## 6. PDF 解析器
- [x] 6.1 新增 `src/doc/pdf.rs` — pdf-extract 文本提取
- [x] 6.2 超 10MB 跳过，空 PDF 跳过
- [x] 6.3 PDF 图片暂不提取（TODO Phase 3）

## 7. Markdown 图片增强
- [x] 7.1 修改 `index_markdown` — 检测 `![alt](url)` 和 `<img>` → ImageRef
- [x] 7.2 本地图读取 raw_bytes，远程 URL image_type='linked'

## 8. 文档管线重构
- [x] 8.1 `parse_document(ext, path, bytes)` 格式路由
- [x] 8.2 路由表: md/rst→index_markdown; xlsx/xls→parse_xlsx; docx→parse_docx; pdf→parse_pdf; xml/html→parse_xml
- [x] 8.3 `write_doc_sections(conn, repo, path, file_format, sections)` 统一写入
- [x] 8.4 写入流程: doc_nodes INSERT → smart_compress → INSERT BLOB → edges INSERT (Excel)
- [x] 8.5 `index_docs` 扩展名: [md,rst] → [md,rst,xlsx,docx,pdf,xml,html]

## 9. MCP — codeloom_search 增强
- [x] 9.1 FusedResult 加 snippet（前 200 字符）+ node_type + has_images + image_count
- [x] 9.2 搜索不直接返回 base64 图片
- [x] 9.3 node_type + section_path 在结果中可见

## 10. MCP — codeloom_get_doc
- [x] 10.1 注册工具 (doc_id, repo, branch)
- [x] 10.2 handler: doc_nodes + JOIN doc_images
- [x] 10.3 图片 BLOB→base64
- [x] 10.4 无效 doc_id → "Document not found"

## 11. MCP — codeloom_query_excel
- [x] 11.1 注册工具 (doc_id, repo, branch, mode, filter, search, limit)
- [x] 11.2 mode 自动推断: cell→row, row→row, header_cell→column, sheet→filter
- [x] 11.3 mode="row": section_path 前缀查同行所有 cell → 格式化为 "列: 值"
- [x] 11.4 mode="column": section_path 列名查同 Sheet 下所有同列 cell
- [x] 11.5 mode="filter": 解析 filter 表达式 → SQL LIKE/GREATER → 返回匹配行
- [x] 11.6 search 实现: 所有列模糊搜索
- [x] 11.7 结果格式: 表头行 + 匹配数据行 + total_matches

## 12. MCP — codeloom_overview 增强
- [x] 12.1 image_count 统计 + per-format 文档统计

## 13. 测试素材
- [x] 13.1-13.8 同之前 (xml/html/xlsx/docx/pdf/md/png/large_xlsx)

## 14. 单元测试
- [x] 14.1-14.10 同之前
- [x] 14.11 xlsx_node_model — 四层节点正确创建，node_type 和 section_path 验证
- [x] 14.12 xlsx_edges — sheet→header_cell, sheet→row 边正确，无 row→cell 边
- [x] 14.13 xlsx_header_detect — 文本表头、空行跳过、降级
- [x] 14.14 query_excel_mode — row/column/filter 三种模式正确

## 15. 集成测试
- [x] 15.1-15.10 同之前
- [x] 15.11 xlsx_section_path_nav — LLM 从搜索结果 section_path 构造 SQL 查同行
- [x] 15.12 edges_doc_to_doc — source_kind/target_kind='doc' 的边能正确查询

## 16. 回归测试
- [x] 16.1 cargo test 全绿
- [x] 16.2 leveldb 端到端验证

## 17. 文档更新
- [x] 17.1-17.5 同之前

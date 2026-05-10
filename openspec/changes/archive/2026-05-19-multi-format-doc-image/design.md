# Design: 多格式文档解析 + 图片关联支持

## Context
CodeLoom 当前文档管线：`index_docs()` 遍历目录 → 过滤 `[md, rst]` → `read_file_smart` → `index_markdown()` → 按 heading 切段 → 写入 `doc_nodes`。图片完全忽略。需要扩展到 5 种新格式 + 图片全链路。

## Goals / Non-Goals
- ✅ xlsx/docx/PDF/XML/HTML 文本提取，存为 doc_nodes
- ✅ Markdown `![alt](url)` 和 HTML `<img>` 图片引用提取
- ✅ xlsx/docx/PDF 嵌入图片提取
- ✅ 图片压缩（WebP, quality 80%）+ 位置记录
- ✅ MCP 搜索命中文档时返回关联图片
- ✅ 新增 `codeloom_get_doc` MCP 工具
- ❌ `.doc` 旧格式（二进制，无纯 Rust 库）
- ❌ 图片内容理解/分析（留给 LLM 在消费端做）
- ❌ 远程 URL 图片自动下载（Phase 3，需网络依赖）

## System Architecture

```
                    index_docs(dir, repo)
                           │
                    ┌──────┴──────┐
                    │ ext→解析器  │
                    │ 路由表      │
                    └──────┬──────┘
       ┌─────────┬────────┼────────┬─────────┬─────────┐
       v         v        v        v         v         v
    md/rst    xlsx     docx      pdf       xml      html
   (已有)   calamine   zip+     pdf-    quick_xml  quick_xml
                      quick_xml extract
       │         │        │        │         │         │
       └─────────┴────────┴────────┴─────────┴─────────┘
                           │
                    ┌──────┴──────┐
                    │ DocSection  │  (统一 IR)
                    │ + Vec<Image>│
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              v            v            v
         doc_nodes    doc_images    (全在 SQLite
         INSERT       INSERT BLOB   一个 .db 文件)
```

**数据流**：解析 → DocSection{images: Vec<Vec<u8>>} → WebP 压缩 → `image_data BLOB` INSERT → MCP 查询时 BLOB → base64 → JSON

## Database Design

### 新增表: doc_images
```sql
CREATE TABLE doc_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    doc_node_id INTEGER NOT NULL REFERENCES doc_nodes(id) ON DELETE CASCADE,
    alt_text TEXT,              -- 图片alt/描述
    original_src TEXT,          -- 原始引用URL或文件名
    image_data BLOB,            -- 压缩后的 WebP 图片字节（直接存 DB，拎包就走）
    position INTEGER,           -- 在文档中的序号（第几张图）
    section_context TEXT,       -- 图片出现位置的文字上下文（前100后100字符）
    image_type TEXT DEFAULT 'inline',  -- inline/attachment/linked
    original_size INTEGER,      -- 原始文件大小（字节）
    compressed_size INTEGER,    -- 压缩后大小（字节）
    width INTEGER,              -- 像素宽（若有）
    height INTEGER               -- 像素高（若有）
);
```

**设计原则**：图片字节直接存入 SQLite BLOB，不落磁盘。整个知识库（代码+文档+图片）就是一个 `.db` 文件，拷贝即迁移，备份即全量。

### 新增列: doc_nodes.node_type
```sql
ALTER TABLE doc_nodes ADD COLUMN node_type TEXT NOT NULL DEFAULT 'section';
-- 取值: 'section'(md/docx/pdf/xml默认), 'sheet', 'header_cell', 'row', 'cell'
```

### 扩展列: edges.source_kind / target_kind
```sql
ALTER TABLE edges ADD COLUMN source_kind TEXT NOT NULL DEFAULT 'symbol';
ALTER TABLE edges ADD COLUMN target_kind TEXT NOT NULL DEFAULT 'symbol';
-- 取值: 'symbol'(指向symbols表), 'doc'(指向doc_nodes表)
-- 允许边连接任意组合: symbol↔symbol, doc↔doc(Excel), symbol↔doc(代码-文档关联)
```

## Decisions

### Decision 1: 统一 IR (DocSection) 而非各自写 DB
选择：所有解析器输出统一的 `DocSection { title, content, level, images }` 结构，由调用方统一写入 DB。
替代方案：各解析器自行写 DB → 拒绝，因违反 DRY，content_hash 和 ON CONFLICT 逻辑需重复。

### Decision 2: 图片存储位置
选择：图片压缩后以 **BLOB 存入 SQLite doc_images 表**，不落磁盘文件。
理由：CodeLoom 设计哲学是「一个 .db 文件拎包就走」——代码符号、文档、图片全在一个 SQLite 文件里，拷贝即迁移、备份即全量。BLOB 读写性能与文件系统持平（SQLite 对 1MB 以下 BLOB 优化极好）。
替代方案：磁盘文件 → 拒绝，违背单文件哲学，远程部署和团队共享需要额外同步目录。

### Decision 3: 图片压缩格式
选择：WebP (quality 80%, max 1920px 宽)，使用 `image` crate。
替代方案：PNG 无损 → 太大；JPEG → 不支持透明；不压缩 → 浪费空间。

### Decision 4: MCP 图片返回方式
选择：MCP 返回图片时使用 **base64 内联**。图片在磁盘存储为 WebP 压缩文件，MCP 响应时读取文件并 base64 编码嵌入 JSON。
理由：远程部署场景下，MCP 服务器的文件路径对 LLM 客户端不可访问。base64 内联保证任意部署拓扑都能用。
替代方案：文件路径 → 拒绝，远程部署不可用。

### Decision 5: 图片在 `codeloom_search` 中的附着时机
选择：搜索命中 `doc` 类型节点时，自动 JOIN `doc_images` 附带图片信息。
替代方案：只有 `codeloom_get_doc` 返回图片 → 不够便捷，LLM 需要二次调用。

## Crate 依赖

| 新增 crate | 用途 | 体积 |
|-----------|------|------|
| `calamine` | xlsx/xls/ods 读取 | ~200KB |
| `zip` | docx/xlsx 解压 | ~150KB |
| `quick-xml` | XML/HTML/docx 内部 XML | ~100KB |
| `pdf-extract` | PDF 文本提取 | ~300KB |
| `image` | WebP 编解码 + 尺寸检测 | ~400KB |

### Decision 6: base64 图片大小控制 + BLOB 读取
选择：`codeloom_search` 附带图片时最大 3 张，每张限制 200KB。MCP 响应时从 SQLite BLOB 直接读取 WebP 字节 → base64 编码 → 嵌入 JSON。
理由：图片全在 DB 里，不碰文件系统。搜索限 3 张防止 JSON 过大（3×200KB×1.33≈800KB 可控）。get_doc 不限张数。
替代方案：不限大小 / 磁盘文件 → 拒绝。

### Decision 7: 小图不压缩
选择：原始 ≤100KB 且宽度 ≤800px 的图片原样存储，不经过 WebP 压缩。
理由：小图（icon、logo、按钮截图）压缩收益近乎为零，WebP 重编码反而可能增大体积或降低锐度。原样保留对 LLM 识别更友好。
替代方案：所有图片一律压缩 → 拒绝，小图可能越压越大。

### Decision 8: Excel 四层节点模型 + section_path 隐式导航
选择：Excel 解析为四层 doc_nodes（sheet→header_cell→row→cell），node_type 区分层级。edges 仅建关键关系（sheet→header_cell, sheet→row），row↔cell 和 header_cell→cell 通过 section_path 前缀隐式表达。
理由：细粒度 cell 节点让 FTS5 精准命中值而非大块文本。section_path 编码层级（如 "销售数据/Row1/销售额"），LLM 无需图遍历就能定位同行同列。edges 不建 row→cell 避免万级节点爆炸（1000行×10列=10000条边省略）。
替代方案：全建 edges → 拒绝，边爆炸；粗粒度 500 行 chunk → 拒绝，搜索不精准。

### Decision 9: DOCX 大段自动拆分
选择：docx section 超过 2000 字符时按自然段边界自动拆分，section_path 附加序号。无标题文档每 10 段或每 2000 字符自动创建 section。
理由：即使有标题，某些章节可能长达 5000 字。拆分后搜索命中更精准，get_doc 返回量更可控。参照 markdown heading 拆分的成功经验。
替代方案：保留原始 section 不拆 → 拒绝，大段搜索精度差。

## MCP Tools

### codeloom_get_doc

```json
{
  "name": "codeloom_get_doc",
  "description": "获取指定文档节点的完整内容+关联图片（base64编码）。doc_id来自codeloom_search返回的doc类型结果。返回标题、章节路径、级别、文本内容、图片列表(base64+alt_text+尺寸)。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "doc_id": {"type": "integer"},
      "repo": {"type": "string"},
      "branch": {"type": "string"}
    },
    "required": ["doc_id", "repo", "branch"]
  }
}
```

### codeloom_query_excel

```json
{
  "name": "codeloom_query_excel",
  "description": "查询 Excel 表格数据。从搜索命中的任意 Excel 节点（sheet/header_cell/row/cell）进入，mode='row'返回整行、'column'返回整列、'filter'按条件过滤。自动推断 mode 基于 node_type。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "doc_id": {"type": "integer", "description": "codeloom_search 返回的 Excel 节点 ID"},
      "repo": {"type": "string"},
      "branch": {"type": "string"},
      "mode": {"type": "string", "enum": ["row", "column", "filter"], "description": "自动: cell→row, row→row, header_cell→column, sheet→filter"},
      "filter": {"type": "string", "description": "如 '销售额 > 5000'，仅 mode=filter"},
      "search": {"type": "string", "description": "所有列模糊搜索"},
      "limit": {"type": "integer", "default": 20}
    },
    "required": ["doc_id", "repo", "branch"]
  }
}
```

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| PDF 文本提取质量不稳定 | `pdf-extract` 对英文/简单排版好，中文可能乱序；标注为实验性，file_format='pdf' 让 LLM 知晓 |
| WebP 编码增加编译时间 | `image` crate 纯 Rust，不引入 C 依赖；首次编译 +30s |
| 大 PDF 索引慢 | 单文件 limit 10MB，超限跳过并警告 |
| docx 嵌入图片提取失败 | 静默跳过，不影响文本索引 |
| MCP base64 响应过大 | 搜索限 3 张×200KB；get_doc 不限但 LLM 通常只在需要时调用 |
| base64 编码 CPU 开销 | 读取+编码 <5ms 对 200KB 图片，可忽略 |
| Excel 大文件产生大量 doc_node | 10000 行≈10000 row nodes + 1 meta node，SQLite 轻松承载；level=5 不参与 FTS5 索引 |
| Excel 表头检测错误 | 降级为 "列A/列B..." 默认列名；LLM 可通过 get_doc 查看原始数据后纠偏 |
| DOCX 拆分边界不当 | 按自然空行拆分，保证不切断句子；docs 拆分粒度不影响可读性 |

## Migration Plan
无数据迁移——doc_nodes 表结构不变，新增 doc_images 表由 schema.rs 自动创建。

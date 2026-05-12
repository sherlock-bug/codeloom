# CodeLoom 索引能力完整规格报告

> 生成时间: 2026-05-12  
> 项目路径: `/mnt/d/RagMcpHermes/codeloom`  
> 目标: 分析 src/indexer/、src/storage/、src/embedding/、src/calib/、src/ignore/、src/doc/、src/linking/ 所有相关模块

---

## 1. 智能索引入口 (smart_index)

- **类型**: internal
- **状态**: active
- **源码位置**: `src/indexer/smart.rs` (390行)
- **用途**: 统一的索引编排入口，根据上次索引状态决定全量扫描 / Git 增量扫描 / 分支继承扫描
- **核心算法**:
  - 获取 HEAD commit hash
  - 查询 `git_index_state` 中该 branch 的上次 indexed commit
  - 如果 commit 没变 → 跳过 (already up to date)
  - 如果存在历史 commit → `git diff --name-only` 获取变更文件列表 → 逐文件 `index_one()`
  - 如果不存在历史 commit → 尝试找 parent branch 做 merge-base → 变更文件 delta
  - 以上都不行 → `full_scan()`: 收集所有文件逐文件解析
- **索引后处理**:
  1. `smart_index` 返回后，调用 `index_docs()` 索引文档
  2. 调用 `index_includes()` 提取 #include 关系
  3. 调用 `fill_all_fts()` 填充 FTS5
  4. 调用 `index_vectors()` 生成向量
  5. 调用 `index_file_vectors()` 生成文件向量
  6. 调用 `calibrate()` 做噪声标定
- **设计 vs 实现**: match — 完全按照增量/全量/继承三层降级策略实现

---

## 2. Git 增量索引

- **类型**: internal
- **状态**: active
- **源码位置**: `src/indexer/git.rs` (23行) + `src/indexer/smart.rs` 中的调用逻辑
- **用途**: 利用 Git diff 实现增量索引，避免全量重扫
- **核心算法**:
  - `head_commit()`: `git rev-parse HEAD`
  - `changed_files(from, to)`: `git diff --name-only <from> <to>`，只保留实际存在的文件
  - `merge_base(a, b)`: `git merge-base <a> <b>`
  - `is_ancestor(ancestor, descendant)`: `git merge-base --is-ancestor`
  - delta 模式下只重新 parse 变更文件 → 逐文件 delete+reinsert
- **局限性/已知问题**:
  - delta 不处理 rename detection — 重命名文件会被删除+新增处理
  - 变更文件列表用 `--name-only`，不含文件内容 hash，内容不变但文件名变也会触发 reindex
- **设计 vs 实现**: match — 简单可靠，无复杂 merge 逻辑

---

## 3. Clang C++ 解析 (Clang AST JSON)

- **类型**: internal
- **状态**: active (主力 C++ 解析器)
- **源码位置**: `src/indexer/clang/mod.rs` (459行) + `src/indexer/clang/ast.rs` (853行) + `src/indexer/clang/compile_cmds.rs` (130行)
- **用途**: C/C++ 符号提取的主引擎，使用 Clang 的 AST dump JSON 解析
- **支持格式/语言**: C, C++ (.cpp/.cc/.cxx/.c)，头文件 (.h/.hpp/.hxx) 通过 #include 纳入
- **核心算法**:
  1. `parse_file()`: 构建管道 `clang -fsyntax-only -Xclang -ast-dump=json <flags> -- <file> | python3 clang_filter.py <project_root>`
  2. Python filter 脚本 (~/.codeloom/scripts/clang_filter.py) 负责：strip 系统头文件 AST 节点、strip 函数体/表达式
  3. `extract_symbols_and_edges()`: 递归遍历 Clang JSON AST，提取 15 种节点类型和 11 种边类型
  4. `compile_cmds::discover()`: 搜索顺序：显式路径 → CODELOOM_COMPILE_COMMANDS 环境变量 → 项目根目录 `compile_commands.json` → `build/compile_commands.json` → `out/compile_commands.json` → 空 map (单文件回退)
- **符号类型 (15种)**:
  - `function` (FunctionDecl), `method` (CXXMethodDecl), `class`, `struct`, `enum`, `enum_value`, `field`, `global`, `static_var`, `template_function`, `template_instance`, `namespace`, `typedef`, `string_literal`
- **边类型 (11种)**:
  - `calls:<callee>` (DeclRefExpr), `inherits:<base>` (bases), `overrides:<base_method>` (referencedDecl), `contains:<member>` (class→method/field), `param_type:<type>`, `return_type:<type>`, `uses_type:<type>` (field/var type), `uses:<symbol>` (DeclRefExpr in function body), `instantiates:<template>` (template instance), `aliases:<underlying>` (typedef), `includes:<file>` (PreprocessedEntity/InclusionDirective)
- **关键特性**:
  - 只 parse translation unit (.cpp/.cc/.cxx/.c)，头文件通过 #include 自动纳入
  - `is_project_file()`: 相对/绝对路径解析后 check 是否在 project_root 内 + 不在 ignore 列表
  - `is_external`: 根据是否有 source location 和是否在 project 内判定
  - 外部符号 (stub) 自动创建：`create_external_stub()` 用 SHA256 hash 去重
  - 符号去重 upsert：full-key 匹配 (name, namespace, kind, signature, file_path, repo)
    - 外部 stub → 真正实现的升级 (is_external: 1→0)
    - 声明 → 定义的合并 (is_definition: false→true)
    - 模板实例 file_path="" 跨 TU 收敛
- **局限性/已知问题**:
  - 依赖外部 `clang` 命令和 Python3 — 需要环境预装
  - Python filter 脚本位置硬编码为 `~/.codeloom/scripts/clang_filter.py` — 不存在则管道失败
  - 模板实例去重依赖 `file_path=""` — 同名但不同模板参数的会去重丢失
  - `strip_cv_ref()` 简单处理 const/引用/指针 — 复杂模板类型可能匹配不全
  - `is_builtin_type()` 只包含常见基本类型 — 自定义类型的 builtin 无法区分
  - `FunctionTemplateDecl` 和 `ClassTemplateSpecializationDecl` 的特殊处理较重
  - 使用 `sh -c` 执行管道 — shell 注入风险 (但只有本地使用)
  - 单文件回退模式没有 compile_commands → 缺少 -I/-D flags → 解析不准确
- **设计 vs 实现**: match — 架构设计与实现完全一致，符号/边类型清单与 schema 严格对应

---

## 4. 文件收集与语言检测

- **类型**: internal
- **状态**: active
- **源码位置**: `src/indexer/tree_sitter.rs` (73行)
- **用途**: 文件发现、语言检测，C++ 路由到 Clang，其他语言由文件收集创建 file node
- **支持格式/语言**:
  - C/C++: `.cpp/.cc/.cxx/.hpp/.hxx/.c/.h` → 路由到 Clang
  - 其他扩展名 → 收集创建 file node，无符号提取
- **核心算法**:
  - `collect_files()`: WalkDir 遍历目录 → git ls-files 过滤 → .codeloomignore 过滤 → detect_language
  - `get_git()`: `git ls-files --cached --others --exclude-standard`
  - `detect_language()`: 基于文件扩展名映射
- **设计 vs 实现**: ✅ match

---

## 5. 文档索引

- **类型**: internal
- **状态**: active
- **源码位置**: `src/doc/mod.rs` (499行) + `src/doc/section.rs` (50行) + `src/doc/docx.rs` (195行) + `src/doc/xlsx.rs` (173行) + `src/doc/pdf.rs` (63行) + `src/doc/xml.rs` (121行) + `src/doc/image.rs` (64行) + `src/doc/glossary.rs` (17行)
- **用途**: 将各类文档解析为统一的 DocSection 结构并写入 nodes 表
- **支持格式/语言**: `md`, `rst`, `xlsx`/`xls`/`xlsm`, `docx`, `pdf`, `xml`
- **核心算法**:
  - `parse_document(ext, path, bytes)`: 按扩展名路由解析
    - **md/rst**: `parse_markdown()` — 按 `#/##/###` 分 section + 正则 `![alt](url)` 和 `<img>` 提取图片
    - **xlsx**: `parse_xlsx()` — calamine crate 读取 → 智能头检测 → 四层节点模型 (sheet → header_cell → row → cell)
    - **docx**: `parse_docx()` — zip 解压 → quick_xml 解析 word/document.xml → 按 heading 样式分节 + 嵌入图片提取
    - **pdf**: `parse_pdf()` — pdf-extract crate (临时文件) → 按分页符分节，10MB 上限
    - **xml/html**: `parse_xml()` — quick_xml 逐层遍历 → 每层一个 section + 跳过 script/style
  - `write_doc_sections()`: 写入 nodes 表 (node_type='section' 或 'chunk')
    - 长内容 (>500字) 自动分块：`split_at_punctuation()` 优先级 `。！？` > `\n` > `；，、` > 强制截断
    - 父 section + chunks 模型，通过 `contains:` edge 关联
  - 图片处理: `smart_compress()` — <100KB且≤800px 保持原样，否则 WebP 压缩 (quality 80%, max 1920px)
  - Glossary: `parse_branch_glossary()` — 从 md/rst 文件提取分支别名映射
- **设计 vs 实现**: match — 文档索引能力完整实现

---

## 6. FTS5 全文搜索

- **类型**: internal
- **状态**: active
- **源码位置**: `src/storage/fts.rs` (291行)
- **用途**: 统一全文搜索层，符号/文档/文件共用一个 FTS5 虚拟表
- **核心算法**:
  - **架构**: `fts5_all(rowid=node_id, name, content)` ↔ `nodes(id, ...)`
  - **填充时机**: 每次 `smart_index` + `index_docs` + `index_includes` 完成后调用 `fill_all_fts()`
  - `fill_all_fts()`: DELETE repo 旧数据 → INSERT INTO fts5_all SELECT id, name, content FROM nodes WHERE repo=?
  - **搜索**: 统一三路查询 (sym + doc + file)，BM25 排序，按 score 融合截断
    - Symbol: JOIN nodes + branches (branch 过滤) + kind 过滤 → content 加 kind 的 prefix strip
    - Doc: node_type='section'
    - File: node_type='file'
  - `build_column_query()`: 构造 `name:"term" OR name:term* OR content:"term" OR content:term*`
- **填充内容**:
  - Symbol: `content = "kind doc_comment"` (如 "class Handles authentication")
  - Doc section: `content = 原文内容`
  - File: `content = summary` (文件头部注释)
- **设计 vs 实现**: match — 统一 FTS5 架构完全实现

---

## 7. 向量化索引

- **类型**: internal
- **状态**: active (符号向量 + 文件向量)
- **源码位置**: `src/embedding/mod.rs` (370行) + `src/storage/vector.rs` (66行)
- **用途**: 为符号和文件生成语义向量，支持语义搜索
- **核心算法**:
  - **Embedder 接口**: `ApiEmbedder` — 通过 OpenAI-compatible `/v1/embeddings` API
    - 可配置: api_base, api_key, model, batch_size, text_limit, max_chars_per_batch
    - 批量嵌入: `embed_batch()` 支持
    - 重试: 最多 3 次，指数退避
  - **向量存储**: sqlite-vec (vec0 虚拟表, FLOAT32, cosine 距离)
    - `symbol_name_vec_<repo>`: 符号名向量
    - `file_vec_<repo>`: 文件向量
  - **`index_vectors()`**: 只索引非 external、非 template_instance、非 namespace 的符号
    - 文本 = `"name [kind] signature"` → 嵌入
    - 分批写入 (batch_size / max_chars_per_batch 控制)
  - **`index_file_vectors()`**: 索引所有 file node
    - 文本 = `"file_path | summary"` → 嵌入
  - **`index_doc_vectors()`**: 已移除 — 注释说明 FTS5 对中文文档足够
  - **初始化**: 全局静态 `EMBEDDER` (OnceLock) — 首次调用 `get_embedder()` 时初始化
  - **时机**: 每次索引完成后异步调用 (tokio::task::spawn_blocking)
- **局限性/已知问题**:
  - 依赖外部 embedding API — 无 API 配置则向量索引静默跳过
  - doc vectors 被移除 — 文档不支持语义搜索
  - vec0 表在 `clear_vectors()` 时 drop+recreate — 所有向量被清空
- **设计 vs 实现**: match — 向量化架构清晰实现

---

## 8. 噪声标定 (Calibration)

- **类型**: internal
- **状态**: active
- **源码位置**: `src/calib/mod.rs` (220行)
- **用途**: 每次索引后自动标定向量搜索的噪声基线，搜索时用 z-score 过滤低置信度结果
- **核心算法**:
  1. 用 5 条与代码无关的噪声探针 (中文/英文/随机字符) 嵌入
  2. 找符号数最多的仓库
  3. 对每条探针做 KNN Top 5 搜索
  4. 收集所有相似度 → 计算 mean, std_dev
  5. `noise_ceiling = mean + 2 * std_dev` (z-score = 2.0)
  6. 持久化到 `config.db` 的 `noise_profile` 表
- **噪声探针**: `"morning coffee afternoon tea"`, `"地铁换乘站周末出行"`, `"今天晚饭吃什么好呢"`, `"春天来了桃花开了"`, `"xyxxy foobarbaz quuxzot kzmwptn"`
- **局限性/已知问题**:
  - 只在索引后自动执行 — 如果无法连接 embedding API 会静默跳过
  - 使用符号数最多的仓库做标定 — 小仓库的分布可能与目标仓库不同
  - 5 条探针 × 5 个结果 = 最多 25 个样本 — 统计量可能不够稳定
- **设计 vs 实现**: match — 噪声标定完整实现

---

## 9. 文件过滤 (Ignore)

- **类型**: internal
- **状态**: active
- **源码位置**: `src/ignore.rs` (92行)
- **用途**: 通过 `.codeloomignore` 文件控制哪些文件/目录跳过索引
- **核心算法**:
  - `load_patterns(root)`: 读取项目根目录的 `.codeloomignore`
  - `is_ignored(path, patterns)`: 简单 glob 匹配
    - `dir/` → 目录内所有文件匹配
    - `*.ext` → 扩展名匹配
    - `prefix*` → 前缀匹配
    - `literal` → 子串匹配
  - **双重过滤**: `collect_files()` 和 `index_clang()` 都调用 `is_ignored()`
- **局限性/已知问题**:
  - 只支持简单 glob — 不支持 `.gitignore` 风格的全部模式 (如 `**/`, `!` 反转)
  - 文件路径用 `contains` 和 `starts_with` 做匹配 — 可能有误判
  - 只检查 `.codeloomignore` — 不自动遵循 `.gitignore`（git ls-files 已做一个层面的 gitignore 过滤）
- **设计 vs 实现**: match — 功能完整

---

## 10. 存储层 (Unified Node/Edge Schema)

- **类型**: internal
- **状态**: active
- **源码位置**: `src/storage/schema.rs` (95行) + `src/storage/nodes.rs` (109行) + `src/storage/symbols.rs` (272行) + `src/storage/files.rs` (34行) + `src/storage/dedup.rs` (4行) + `src/storage/mod.rs` (55行)
- **用途**: 统一存储层，所有实体 (符号/文档/文件) 共用 `nodes` 表
- **核心表结构**:
  - `nodes(id, repo, node_type, name, content, file_path, line_start, content_hash, branch_id, kind, attrs)` — 单表存储所有实体
    - `node_type`: `'sym'` | `'section'` | `'chunk'` | `'file'`
    - `attrs`: JSON 字段，按 node_type 不同存储不同元数据
  - `edges(id, source_id, target_id, edge_type, source_repo, target_repo, branch_id)` — 关系图
  - `branches(node_id, repo, branch_id, branch_name, override_def, override_hash)` — 分支符号关联
  - `branch_meta(id, repo, branch_name)` — 分支注册
  - `git_index_state(repo, branch_name, head_commit, parent_ref, indexed_files, indexed_at)` — 增量索引状态
  - `branch_glossary(id, repo, branch_name, alias, description, doc_path)` — 分支别名
  - `doc_images(id, doc_node_id, alt_text, original_src, image_data, position, section_context, image_type, original_size, compressed_size, width, height)` — 文档图片
  - `fts5_all(name, content)` — FTS5 虚拟表
- **Symbol 插入逻辑 (symbols.rs)**:
  - `insert()`: 按 `(content_hash, file_path, name, repo)` 去重
    - 存在 → UPDATE line_start, line_end, doc_comment
    - 不存在 → INSERT with attrs JSON (signature, namespace, access, is_virtual, is_definition, is_external, template_args, language, parent_class, line_end, sid)
  - `insert_builtin_symbols()`: 预置 C++ STL 符号 (std::vector/string/map/set/unique_ptr/shared_ptr 的方法 + 30+ 个自由函数)
  - `make_sid()`: SHA256(repo, name, sig, ns, kind, file_path) 前 16 hex chars
- **FileNode 插入**: `ON CONFLICT(repo, file_path, branch_id) DO UPDATE`
- **设计 vs 实现**: match — 统一 nodes 表 + JSON attrs 的设计完全实现

---

## 11. #include 关系索引

- **类型**: internal
- **状态**: active
- **源码位置**: `src/cli/mod.rs` (lines 864-886)
- **用途**: 从 C/C++ 源文件提取 `#include` 关系并存储为 edges
- **核心算法**: 正则 `#include\s*[<"]([^>"]+)[>"]` 扫描所有 `.h/.hpp/.hxx/.cpp/.cxx/.cc/.c` 文件
- **局限性/已知问题**:
  - 正则提取 — 不处理条件编译、宏展开
  - edge 的 source_id 和 target_id 固定为 0 — 无法关联到具体符号
  - 只做单次 regex 扫描，不使用预处理器
- **设计 vs 实现**: diverges — edges 写入但没有真正的符号关联 (source_id=0, target_id=0)

---

## 12. 文档-代码桥接 (Doc-Code Linking)

- **类型**: internal
- **状态**: removed（规格砍掉，非 bug）
- **源码位置**: `src/linking/mod.rs` — 仅注释 `// doc-code linking — stub`
- **备注**: 该功能已被用户从规格中移除，`src/linking/mod.rs` 的 stub 为残留代码。不是设计vs实现的偏差。
- **设计vs实现**: N/A（已从规格移除）

---

## 13. 分支继承 (Branch Inheritance)

- **类型**: internal
- **状态**: active
- **源码位置**: `src/indexer/smart.rs` (lines 96-118)
- **用途**: 新建分支索引时，从 parent branch 继承符号，只索引 delta 变更
- **核心算法**:
  - `find_parent()`: 遍历 `git_index_state` 其他 branch → 找 merge base → 按 commit time 选最新
  - 或者通过 CLI `--parent` 参数指定
  - 继承的符号数记录为 `symbols_inherited` (而非 `symbols_new`)
  - `git_index_state.parent_ref` 记录继承来源
- **设计 vs 实现**: match — 分支继承完整实现

---

## 总结：索引管线端到端流程

```
codeloom index <path> --repo <name> --branch <b>
  │
  ├─ 1. smart_index()
  │     ├─ head commit? → delta from git_index_state
  │     ├─ parent branch? → merge-base delta
  │     └─ no history → full_scan()
  │           ├─ C/C++ → clang -ast-dump=json → filter → extract
  │           └─ Other → skip (file node created)
  │
  ├─ 2. index_docs() → parse_document(ext) → sections + chunks
  │     ├─ md/rst → heading-based sections
  │     ├─ xlsx → 4-layer (sheet → header → row → cell)
  │     ├─ docx → heading styles + embedded images
  │     ├─ pdf → page-based sections
  │     └─ xml/html → element-based sections
  │
  ├─ 3. index_includes() → regex #include → edges (source=0 target=0)
  │
  ├─ 4. fill_all_fts() → DELETE + INSERT into fts5_all
  │
  ├─ 5. index_vectors() → symbol name → API embedder → vec0
  │     index_file_vectors() → file path|summary → API embedder → vec0
  │
  └─ 6. calibrate() → 5 noise probes × KNN Top 5 → noise profile
```

**关键发现**:
1. **Clang C++ 解析**是最完善的能力 — 15 种符号、11 种边类型、外部符号 stub、声明-定义合并
2. **文档索引**能力完整 — 6 种格式、图片压缩、中文文本分块
3. **#include 关系**的 edge 没有真正的符号关联 (source_id=0)
4. **增量索引**使用 `git diff --name-only` — 简单但可靠
5. **噪声标定**是独创的能力 — 用噪声探针自适应计算向量搜索阈值

# CodeLoom 内部查询能力规格

> 源码 `src/query/{search,graph,call_graph,repo}.rs`

---

## 1. FusedResult 结构体

- **源码**: `search.rs:9-33`
- **用途**: 混合搜索的统一结果载体
- **字段**: `id` · `score` · `name` · `hit_type` · `file_path` · `line_start` · `kind` · `signature` · `snippet` · `doc_id` + 10 个富化字段
- **富化字段**: `members` · `methods` · `values` · `parent_class` · `prev_section` · `next_section` · `prev_chunk` · `next_chunk` · `sections` · `parent_section`（逗号分隔字符串）
- **设计vs实现**: ✅ match

---

## 2. weighted_fuse

- **源码**: `search.rs:40`
- **签名**: `pub fn weighted_fuse(bm25_hits, vec_results, w_bm25, w_vec) -> Vec<FusedResult>`
- **逻辑**: max(归一化BM25, 余弦相似度) + 交叉通道奖励 0.1
- **归一化**: BM25 → `2.0/(1+|s|/8)`；向量 → `1.0/(1+dist)`
- **去重**: (name, file_path) 为 key
- **设计vs实现**: ✅ match

---

## 3. hybrid_search

- **源码**: `search.rs:177`
- **签名**: `pub fn hybrid_search(conn, query, repo, branch, limit, kind_filter)`
- **逻辑**: 4 BM25 通道（name×0.6 + content + doc×0.6 + file×0.5）+ 2 向量通道（name + file）
- **问题**: 注释提到硬阈值 0.5 但实际只 truncate，未真正过滤

---

## 4. bm25_precise_search

- **源码**: `search.rs:368`
- **签名**: `pub fn bm25_precise_search(conn, query, repo, branch, limit, kind_filter, skip_noise)`
- **逻辑**: 2 通道 BM25（name×0.7 + content×0.3）+ enrich_search_results
- **归一化**: 线性截断 `(-score/12).min(1).max(0)`，与 weighted_fuse 不同
- **降噪**: `skip_noise=false` 时过滤 score < 0.5
- **设计vs实现**: ✅ match

---

## 5. vector_semantic_search

- **源码**: `search.rs:434`
- **签名**: `pub fn vector_semantic_search(conn, query_emb, repo, branch, limit, skip_noise)`
- **逻辑**: vec0 KNN → 批量 JOIN 详情 → Z-score 噪声过滤
- **降噪**: 从 `noise_profile` 加载 μ, σ, 保留 z > 1.0
- **设计vs实现**: ✅ match

---

## 6. enrich_search_results

- **源码**: `search.rs:544`
- **签名**: `fn enrich_search_results(conn, repo, branch, results)`
- **逻辑**: 按 kind/hit_type 分支查询 edges/attrs，填充 FusedResult 富化字段
- **class/struct** → members + methods
- **enum** → values
- **function/method** → parent_class
- **doc** → prev_section / next_section
- **file** → sections
- **chunk** → parent_section + prev/next

---

## 7. get_edges

- **源码**: `graph.rs:23`
- **用途**: 获取符号的边（支持方向+边类型过滤）
- **设计vs实现**: ✅ match

---

## 8. resolve_symbol_id / resolve_symbol_id_or_name / symbol_name_by_id

- **源码**: `graph.rs:92 / 104 / 128`
- **用途**: 符号名↔ID 解析。优先 ID，回退名称

---

## 9. build_forward_adj / build_reverse_adj

- **源码**: `graph.rs:134 / 163`
- **用途**: 预加载全量邻接表（空间换时间，用于 BFS）
- **设计vs实现**: ✅ match

---

## 10. bfs_path_search

- **源码**: `graph.rs:197`
- **用途**: BFS 查找 source→target 路径
- **模式**: `shortest | all`，环路检测
- **设计vs实现**: ✅ match

---

## 11. transitive_closure

- **源码**: `graph.rs:267`
- **用途**: 有限半径 BFS 传递闭包（影响分析）
- **设计vs实现**: ✅ match

---

## 12. neighbor_map

- **源码**: `graph.rs:346`
- **用途**: 1 跳邻居（按边类型分组）
- **特殊**: class/struct 跳过 contains 边，改暴
露成员引用的外部符号
- **设计vs实现**: ✅ match

---

## 13. get_terminal_deps

- **源码**: `graph.rs:427`
- **用途**: 获取 `uses` / `references` / `string_literals` 三类终端依赖
- **设计vs实现**: ✅ match

---

## 14. get_call_graph

- **源码**: `call_graph.rs:7`
- **用途**: 构建调用图（递归 DFS，每次递归查 DB）
- **回退**: 精确匹配失败后 `LIKE '%name%'` 模糊匹配
- **设计vs实现**: ✅ match

---

## 15. list_repos

- **源码**: `repo.rs:5`
- **用途**: 扫描 `config_dir/*.rag.db` 列出仓库
- **设计vs实现**: ✅ match

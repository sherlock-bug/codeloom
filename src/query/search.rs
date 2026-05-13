// Search engine — BM25 FTS5 + vec0 vector semantic search
use crate::storage;
use rusqlite::Connection;
use std::collections::HashMap;

/// A fused search result from hybrid (BM25 + vector) search
#[derive(Debug, Clone, Default)]
pub struct FusedResult {
    pub id: i64,         // node rowid, 0 for fused/aggregated results
    pub score: f64, // weighted fusion score
    pub name: String,
    pub hit_type: String, // "code" or "doc"
    pub file_path: String,
    pub line_start: i64,
    pub kind: String,     // symbol kind (function/class/enum_value etc.), empty for docs
    pub signature: String, // function/method signature, empty for non-code or no-sig
    pub snippet: String,   // doc_comment for code, content for docs
    pub doc_id: i64,       // doc_nodes rowid (= doc_id for codeloom_get_doc), 0 for code results

    // Enrichment fields — populated by enrich_search_results(); leave as defaults otherwise
    pub members: String,       // class/struct: 字段名列表 "name,age,..."
    pub methods: String,       // class/struct: 方法名列表 "getUser,setName,..."
    pub values: String,        // enum: 枚举值列表 "RED,GREEN,BLUE"
    pub parent_class: String,  // function/method: 所属类名
    pub prev_section: String,  // section: 前一章节标题
    pub next_section: String,  // section: 后一章节标题
    pub prev_chunk: String,    // chunk: 前一 chunk 标题
    pub next_chunk: String,    // chunk: 后一 chunk 标题
    pub sections: String,      // file: 顶层 section 列表
    pub parent_section: String, // chunk: 所属 section 名
}
// ── BM25 Precise Search ─────────────────────────────────────────────────
// 纯 BM25 FTS5 搜索，名称+内容通道，取各符号最优原始BM25分数。
// skip_noise: true 时跳过噪音过滤（标定用）。

/// BM25 precise search — FTS5 only, name + content channels, raw BM25 take-best.
/// Returns FusedResult with single-channel raw BM25 scores.
pub fn bm25_precise_search(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
    kind_filter: Option<&str>,
    skip_noise: bool,
) -> anyhow::Result<Vec<FusedResult>> {
    let fetch_limit = (limit * 3).max(15);

    // ── Name channel ──
    let bm25_name = storage::fts::search_fts5_name(conn, query, repo, branch, fetch_limit, kind_filter)?;
    // ── Content channel ──
    let bm25_content = storage::fts::search_fts5_content(conn, query, repo, branch, fetch_limit, kind_filter)?;

    // Merge both channels into one pool: for each symbol keep its best (most negative) BM25 score.
    // Both channels query the same fts5_all table (same IDF space), so raw BM25 is directly comparable.
    let mut entries: HashMap<(String, String), FusedResult> = HashMap::new();
    for hits in [&bm25_name, &bm25_content] {
        for hit in hits {
            let key = (hit.name.clone(), hit.file_path.clone());
            let is_doc = matches!(hit.hit_type, storage::fts::HitType::Doc);
            let is_file = matches!(hit.hit_type, storage::fts::HitType::File);
            let ht = if is_doc { "doc" } else if is_file { "file" } else { "code" };
            entries
                .entry(key)
                .and_modify(|e| {
                    // Keep more negative BM25 (better match)
                    if hit.score < e.score { e.score = hit.score; }
                })
                .or_insert(FusedResult {
                    id: hit.rowid,
                    score: hit.score,
                    name: hit.name.clone(),
                    hit_type: ht.into(),
                    file_path: hit.file_path.clone(),
                    line_start: hit.line_start,
                    kind: hit.kind.clone(),
                    signature: hit.sig.clone(),
                    snippet: hit.snippet.clone(),
                    doc_id: if is_doc { hit.rowid } else { 0 },
                    ..Default::default()
                });
        }
    }

    let mut fused: Vec<FusedResult> = entries.into_values().collect();
    // Sort by raw BM25 ascending (more negative = better match in FTS5)
    fused.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

    // Debug: print raw results
    eprintln!("[DEBUG] bm25_precise_search: query='{}' repo={} branch={} name_hits={} content_hits={} fused={}",
        query, repo, branch, bm25_name.len(), bm25_content.len(), fused.len());
    for r in fused.iter().take(5) {
        eprintln!("[DEBUG]   BM25={:.2} type={} name={} snippet={}", r.score, r.hit_type, r.name, r.snippet.chars().take(40).collect::<String>());
    }
    for r in &fused {
        if r.name == "BlockBuilder::Add" || r.name == "TableBuilder::Add" {
            eprintln!("[DEBUG]   *** FOUND {} BM25={:.2} type={}", r.name, r.score, r.hit_type);
        }
    }
    for hit in bm25_content.iter().take(3) {
        eprintln!("[DEBUG]   content: name={} BM25={:.2} rowid={} file={}", hit.name, hit.score, hit.rowid, hit.file_path);
    }

    // Noise floor: BM25 > -5 (barely matches anything) = too weak
    if !skip_noise {
        fused.retain(|r| r.score <= -5.0);
    }

    fused.truncate(limit);
    enrich_search_results(conn, repo, branch, &mut fused);
    Ok(fused)
}

// ── Vector Semantic Search ──────────────────────────────────────────────
// 纯向量搜索，只搜符号名称通道，vec0 INT8 KNN。

/// Vector semantic search — vec0 KNN on symbol name channel only.
/// query_emb: pre-computed embedding for the search query.
/// skip_noise: true 时跳过噪音过滤（标定用）。。
pub fn vector_semantic_search(
    conn: &Connection,
    query_emb: &[f32],
    repo: &str,
    branch: &str,
    limit: usize,
    skip_noise: bool,
) -> anyhow::Result<Vec<FusedResult>> {
    if query_emb.is_empty() {
        return Ok(Vec::new());
    }
    if !crate::storage::vector::try_load(conn) {
        return Err(anyhow::anyhow!("vec0 extension not loaded — vector search unavailable"));
    }

    let fetch_limit = (limit * 2).max(10);
    let sym_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));

    // ── Vector: symbol name channel ──
    let rows = match crate::storage::vector::knn_search(conn, &sym_table, query_emb, fetch_limit) {
        Ok(r) => r,
        Err(e) => return Err(anyhow::anyhow!("KNN search failed: {}", e)),
    };

    let mut vec_results: Vec<FusedResult> = Vec::new();
    if !rows.is_empty() {
        let branch_id = crate::storage::resolve_branch_id(conn, repo, branch).unwrap_or(0);
        let rowids: Vec<i64> = rows.iter().map(|(id, _)| *id).collect();
        let placeholders: String = rowids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT n.id, n.name, n.kind, n.file_path, n.line_start, \
             COALESCE(n.content,''), COALESCE(json_extract(n.attrs,'$.signature'),'') \
             FROM nodes n \
             JOIN branches b ON b.node_id = n.id \
             WHERE n.id IN ({}) AND n.node_type='sym' AND b.branch_id=?",
            placeholders
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = rowids.iter()
            .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        params.push(Box::new(branch_id));

        let mut detail_map: HashMap<i64, (String, String, String, i64, String, String)> = HashMap::new();
        if let Ok(mut stmt) = conn.prepare(&sql) {
            let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            if let Ok(detail_rows) = stmt.query_map(refs.as_slice(), |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            }) {
                for dr in detail_rows.flatten() {
                    detail_map.insert(dr.0, (dr.1, dr.2, dr.3, dr.4, dr.5, dr.6));
                }
            }
        }

        for (rowid, dist) in &rows {
            if let Some((name, kind, file_path, line_start, doc_comment, signature)) = detail_map.get(rowid) {
                let sim = 1.0 / (1.0 + *dist as f64);
                let snippet: String = if !doc_comment.is_empty() {
                    doc_comment.chars().take(500).collect()
                } else if !signature.is_empty() {
                    signature.chars().take(500).collect()
                } else {
                    String::new()
                };
                vec_results.push(FusedResult {
                    id: *rowid,
                    score: sim,
                    name: name.clone(),
                    hit_type: "code".into(),
                    file_path: file_path.clone(),
                    line_start: *line_start,
                    kind: kind.clone(),
                    signature: signature.clone(),
                    snippet,
                    doc_id: 0,
                ..Default::default()                });
            }
        }
    }

    // ── Pure vector semantic results — sort by similarity descending ──
    vec_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Z-score noise filter for vector search (similarity distribution depends on model & index)
    // BM25 uses hard threshold 0.5 (absolute semantics); vector needs statistical calibration
    if !skip_noise {
        if let Some(p) = crate::calib::load_noise_profile() {
            if p.noise_std > 0.0 {
                vec_results.retain(|r| {
                    let z = (r.score - p.noise_mean) / p.noise_std;
                    z > 1.0
                });
            }
        }
    }

    vec_results.truncate(limit);
    enrich_search_results(conn, repo, branch, &mut vec_results);
    Ok(vec_results)
}

fn enrich_search_results(
    conn: &rusqlite::Connection,
    repo: &str,
    branch: &str,
    results: &mut [FusedResult],
) {
    let branch_id = crate::storage::resolve_branch_id(conn, repo, branch).unwrap_or(0);
    for r in results.iter_mut() {
        let is_doc = r.hit_type == "doc";
        let is_file = r.hit_type == "file";

        match r.kind.as_str() {
            "class" | "struct" => {
                // Members (fields)
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id \
                     WHERE e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'contains:' AND n.kind = 'field' \
                     ORDER BY n.id LIMIT 15"
                ) {
                    let names: Vec<String> = stmt.query_map([r.id, branch_id], |row| {
                        row.get::<_, String>(0)
                    }).ok().into_iter().flatten().filter_map(|r| r.ok()).collect();
                    if !names.is_empty() {
                        r.members = names.join(", ");
                    }
                }
                // Methods
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id \
                     WHERE e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'contains:' AND n.kind = 'method' \
                     ORDER BY n.id LIMIT 15"
                ) {
                    let names: Vec<String> = stmt.query_map([r.id, branch_id], |row| {
                        row.get::<_, String>(0)
                    }).ok().into_iter().flatten().filter_map(|r| r.ok()).collect();
                    if !names.is_empty() {
                        r.methods = names.join(", ");
                    }
                }
            }
            "enum" => {
                // Enum values
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id \
                     WHERE e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'contains:' AND n.kind = 'enum_value' \
                     ORDER BY n.id LIMIT 15"
                ) {
                    let names: Vec<String> = stmt.query_map([r.id, branch_id], |row| {
                        row.get::<_, String>(0)
                    }).ok().into_iter().flatten().filter_map(|r| r.ok()).collect();
                    if !names.is_empty() {
                        r.values = names.join(", ");
                    }
                }
            }
            "function" | "method" => {
                // Parent class from attrs
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT json_extract(attrs, '$.parent_class') FROM nodes WHERE id = ?1"
                ) {
                    if let Ok(parent) = stmt.query_row([r.id], |row| row.get::<_, Option<String>>(0)) {
                        if let Some(p) = parent {
                            if !p.is_empty() {
                                r.parent_class = p;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Section: prev/next sibling
        if is_doc {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT name FROM nodes WHERE file_path = ?1 AND node_type = 'section' AND id < ?2 ORDER BY id DESC LIMIT 1"
            ) {
                if let Ok(name) = stmt.query_row(rusqlite::params![&r.file_path, &r.id], |row| row.get::<_, String>(0)) {
                    r.prev_section = name;
                }
            }
            if let Ok(mut stmt) = conn.prepare(
                "SELECT name FROM nodes WHERE file_path = ?1 AND node_type = 'section' AND id > ?2 ORDER BY id ASC LIMIT 1"
            ) {
                if let Ok(name) = stmt.query_row(rusqlite::params![&r.file_path, &r.id], |row| row.get::<_, String>(0)) {
                    r.next_section = name;
                }
            }
        }

        // File: top-level sections
        if is_file {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT name FROM nodes WHERE file_path = ?1 AND node_type = 'section' ORDER BY id LIMIT 15"
            ) {
                let names: Vec<String> = stmt.query_map([&r.file_path], |row| {
                    row.get::<_, String>(0)
                }).ok().into_iter().flatten().filter_map(|r| r.ok()).collect();
                if !names.is_empty() {
                    r.sections = names.join(", ");
                }
            }
        }

        // Chunk: parent section + prev/next chunk
        if is_doc && r.doc_id > 0 {
            // Chunk is a doc with doc_id set; find parent section id first
            if let Ok(mut stmt) = conn.prepare(
                "SELECT e.source_id, n.name FROM edges e JOIN nodes n ON e.source_id = n.id \
                 WHERE e.target_id = ?1 AND e.edge_type = 'contains:' AND e.branch_id = ?2 LIMIT 1"
            ) {
                if let Ok((parent_id, parent_name)) = stmt.query_row([r.id, branch_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                }) {
                    r.parent_section = parent_name;
                    // prev chunk: same parent, lower idx
                    if let Ok(mut stmt2) = conn.prepare(
                        "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id \
                         WHERE e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'contains:' AND n.kind = 'chunk' AND n.id < ?3 \
                         ORDER BY n.id DESC LIMIT 1"
                    ) {
                        if let Ok(name) = stmt2.query_row([parent_id, branch_id, r.id], |row| row.get::<_, String>(0)) {
                            r.prev_chunk = name;
                        }
                    }
                    // next chunk
                    if let Ok(mut stmt3) = conn.prepare(
                        "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id \
                         WHERE e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'contains:' AND n.kind = 'chunk' AND n.id > ?3 \
                         ORDER BY n.id ASC LIMIT 1"
                    ) {
                        if let Ok(name) = stmt3.query_row([parent_id, branch_id, r.id], |row| row.get::<_, String>(0)) {
                            r.next_chunk = name;
                        }
                    }
                }
            }
        }
    }
}



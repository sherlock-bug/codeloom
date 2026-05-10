// Hybrid search — weighted fusion of FTS5 BM25 + vec0 vector ANN
use crate::storage;
use rusqlite::Connection;
use std::collections::HashMap;

/// A fused search result from hybrid (BM25 + vector) search
#[derive(Debug, Clone)]
pub struct FusedResult {
    pub score: f64, // weighted fusion score
    pub name: String,
    pub hit_type: String, // "code" or "doc"
    pub file_path: String,
    pub line_start: i64,
    pub kind: String,     // symbol kind (function/class/enum_value etc.), empty for docs
    pub signature: String, // function/method signature, empty for non-code or no-sig
    pub snippet: String,   // doc_comment for code, content for docs
    pub doc_id: i64,       // doc_nodes rowid (= doc_id for codeloom_get_doc), 0 for code results
}

/// Weighted fusion: normalize BM25 scores to (0,1] then combine with vector cosine similarity.
/// BM25 scores (from FTS5 bm25()) are ≤0, with 0 = best match. We use 1/(0.5+|score|)
/// so a strong BM25 hit (e.g. -0.5) normalizes to 1.0, outranking vector similarity.
/// Vector distances are converted to cosine similarity: 1/(1+distance), already in (0,1].
/// Final: max(w_bm25 * norm_bm25, w_vec * cosine_sim) — takes the stronger channel.
pub fn weighted_fuse(
    bm25_hits: &[storage::fts::SearchHit],
    vec_results: &[(f64, String, String, String, i64, String, String, String, i64)],
    w_bm25: f64,
    w_vec: f64,
) -> Vec<FusedResult> {
    // Key: (name, file_path) — prevents same-name sections from accumulating and
    // distinguishes same-named symbols from different files
    let mut entries: HashMap<(String, String), FusedResult> = HashMap::new();

    // BM25 side — normalize and insert
    for hit in bm25_hits {
        let key = (hit.name.clone(), hit.file_path.clone());
        let norm_bm25 = 1.0 / (0.5 + hit.score.abs());
        let score = w_bm25 * norm_bm25;

        let is_doc = matches!(hit.hit_type, storage::fts::HitType::Doc);
        let is_file = matches!(hit.hit_type, storage::fts::HitType::File);
        let ht = if is_doc { "doc" } else if is_file { "file" } else { "code" };
        let entry = entries.entry(key.clone()).or_insert(FusedResult {
            score: 0.0,
            name: hit.name.clone(),
            hit_type: ht.into(),
            file_path: hit.file_path.clone(),
            line_start: hit.line_start,
            kind: hit.kind.clone(),
            signature: hit.sig.clone(),
            snippet: hit.snippet.clone(),
            doc_id: if is_doc { hit.rowid } else { 0 },
        });
        // Take max — same name+file_path from multiple BM25 rows (multi-section doc) gets best score
        entry.score = entry.score.max(score);
    }

    // Vector side — normalize similarity and merge
    for (sim, name, hit_type, file_path, line_start, kind, sig, snippet, doc_id) in vec_results {
        let key = (name.clone(), file_path.clone());
        let score = w_vec * sim;

        if let Some(entry) = entries.get_mut(&key) {
            entry.score = entry.score.max(score);
            // If vec side has a real doc_id (non-zero), carry it forward
            if *doc_id != 0 {
                entry.doc_id = *doc_id;
            }
        } else {
            entries.insert(key, FusedResult {
                score,
                name: name.clone(),
                hit_type: hit_type.clone(),
                file_path: file_path.clone(),
                line_start: *line_start,
                kind: kind.clone(),
                signature: sig.clone(),
                snippet: snippet.clone(),
                doc_id: *doc_id,
            });
        }
    }

    // Sort by score descending
    let mut fused: Vec<FusedResult> = entries.into_values().collect();
    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    fused
}

/// Fuse a single channel (BM25 + vector) with given weights.
/// Returns vec of FusedResult with scores already weighted.
fn weighted_fuse_single(
    bm25_hits: &[storage::fts::SearchHit],
    vec_results: &[(f64, String, String, String, i64, String, String, String, i64)],
    default_hit_type: &str,
    w_bm25: f64,
    w_vec: f64,
) -> Vec<FusedResult> {
    let mut entries: HashMap<(String, String), FusedResult> = HashMap::new();

    // BM25 side
    for hit in bm25_hits {
        let key = (hit.name.clone(), hit.file_path.clone());
        let norm_bm25 = 1.0 / (0.5 + hit.score.abs());
        let score = w_bm25 * norm_bm25;
        let is_doc = matches!(hit.hit_type, storage::fts::HitType::Doc);
        let ht = if is_doc { "doc" } else { default_hit_type };
        let entry = entries.entry(key.clone()).or_insert(FusedResult {
            score: 0.0,
            name: hit.name.clone(),
            hit_type: ht.into(),
            file_path: hit.file_path.clone(),
            line_start: hit.line_start,
            kind: hit.kind.clone(),
            signature: hit.sig.clone(),
            snippet: hit.snippet.clone(),
            doc_id: if is_doc { hit.rowid } else { 0 },
        });
        entry.score = entry.score.max(score);
    }

    // Vector side
    for (sim, name, hit_type, file_path, line_start, kind, sig, snippet, doc_id) in vec_results {
        let key = (name.clone(), file_path.clone());
        let score = w_vec * sim;
        if let Some(entry) = entries.get_mut(&key) {
            entry.score = entry.score.max(score);
            if *doc_id != 0 { entry.doc_id = *doc_id; }
        } else {
            entries.insert(key, FusedResult {
                score,
                name: name.clone(),
                hit_type: hit_type.clone(),
                file_path: file_path.clone(),
                line_start: *line_start,
                kind: kind.clone(),
                signature: sig.clone(),
                snippet: snippet.clone(),
                doc_id: *doc_id,
            });
        }
    }

    let mut result: Vec<FusedResult> = entries.into_values().collect();
    result.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    result
}
/// Symbol name hits get weight 0.6, comment hits get 0.4. Doc/File get 0.5 each.
pub fn hybrid_search(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
    kind_filter: Option<&str>,
) -> anyhow::Result<Vec<FusedResult>> {
    let fetch_limit = (limit * 2).max(10);

    // FTS5: split name / comment / doc / file
    let bm25_name = storage::fts::search_symbols_name(conn, query, repo, branch, fetch_limit, kind_filter)?;
    let bm25_comment = storage::fts::search_symbols_comment(conn, query, repo, branch, fetch_limit, kind_filter)?;
    let bm25_docs = storage::fts::search_docs(conn, query, repo, fetch_limit)?;
    let bm25_files = storage::fts::search_files(conn, query, repo, fetch_limit)?;

    // Embed once — shared across all 4 vector channels
    let embedder = crate::embedding::get_embedder().ok();
    let query_emb = embedder.and_then(|e| e.embed(query).ok());

    // Vector: name + file channels (comment/doc removed — FTS5 sufficient)
    let vec_name = run_vector_search(conn, query_emb.as_deref(), repo, branch, fetch_limit, "name");
    let vec_file = run_vector_search(conn, query_emb.as_deref(), repo, branch, fetch_limit, "file");

    // Fuse each channel independently, then merge
    let mut entries: HashMap<(String, String), FusedResult> = HashMap::new();

    // Code name channel: weight 0.6
    let name_fused = weighted_fuse_single(&bm25_name, &vec_name, "code", 0.6, 0.6);
    for r in name_fused {
        let key = (r.name.clone(), r.file_path.clone());
        entries.entry(key).or_insert(r);
    }

    // Comment channel: BM25 comment results add score to matching name entries (no vector channel)

    // Doc channel: BM25-only (vector channel removed — FTS5 sufficient for Chinese)

    // File channel: weight 0.5
    let file_fused = weighted_fuse_single(&bm25_files, &vec_file, "file", 0.5, 0.5);
    for r in file_fused {
        let key = (r.name.clone(), r.file_path.clone());
        entries.entry(key)
            .and_modify(|e| { if r.score > e.score { e.score = r.score; } })
            .or_insert(r);
    }

    // Sort and filter noise
    let mut fused: Vec<FusedResult> = entries.into_values().collect();
    fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // z-score filter: drop results within 1σ of noise top1_mean (use bm25 baseline as hybrid)
    if let Some(profile) = crate::calib::noise_profile("bm25") {
        let threshold = 1.0;
        fused.retain(|r| (r.score - profile.top1_mean) / profile.top1_std.max(0.001) >= threshold);
    }

    fused.truncate(limit);
    Ok(fused)
}

/// Run vec0 vector search with a pre-computed embedding.
/// Returns (similarity, name, hit_type, file_path, line_start, kind, signature, snippet, doc_id).
/// doc_id is doc_nodes.rowid for doc results, 0 for code results.
/// When query_emb is None or empty, returns empty (no embedding available).
fn run_vector_search(
    conn: &Connection,
    query_emb: Option<&[f32]>,
    repo: &str,
    branch: &str,
    limit: usize,
    channel: &str,
) -> Vec<(f64, String, String, String, i64, String, String, String, i64)> {
    let query_emb = match query_emb {
        Some(e) if !e.is_empty() => e,
        _ => return Vec::new(),
    };
    if !crate::storage::vector::try_load(conn) {
        return Vec::new();
    }

    let mut results = Vec::new();

    // Symbol vector search — name channel (INT8)
    if channel == "name" {
        let sym_table = format!("symbol_{}_vec_{}", channel, repo.replace('-', "_"));
        let query_i8: Vec<i8> = crate::embedding::quantize_f32_to_i8(query_emb);
        if let Ok(rows) = crate::storage::vector::knn_search_int8(conn, &sym_table, &query_i8, limit) {
            if !rows.is_empty() {
                // Batch fetch: collect all rowids, query once with IN (...)
                let rowids: Vec<i64> = rows.iter().map(|(id, _)| *id).collect();
                let placeholders: String = rowids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sql = format!(
                    "SELECT s.rowid, s.name, s.kind, s.file_path, s.line_start, \
                     COALESCE(s.doc_comment,''), COALESCE(s.signature,'') \
                     FROM symbols s JOIN branches b ON b.symbol_id = s.id \
                     WHERE s.rowid IN ({}) AND b.branch_name=?",
                    placeholders
                );
                let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = rowids.iter()
                    .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
                    .collect();
                params.push(Box::new(branch.to_string()));
                if let Ok(mut stmt) = conn.prepare(&sql) {
                    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
                    let mut detail_map: HashMap<i64, (String, String, String, i64, String, String)> = HashMap::new();
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
                    // Re-join with knn results to preserve distance order
                    for (rowid, dist) in &rows {
                        if let Some((name, kind, file_path, line_start, doc_comment, signature)) = detail_map.get(rowid) {
                            let sim = 1.0 / (1.0 + *dist as f64);
                            let vsnip: String = if !doc_comment.is_empty() {
                                doc_comment.chars().take(500).collect()
                            } else if !signature.is_empty() {
                                signature.chars().take(500).collect()
                            } else {
                                String::new()
                            };
                            results.push((sim, name.clone(), "code".into(), file_path.clone(),
                                *line_start, kind.clone(), signature.clone(), vsnip, 0));
                        }
                    }
                }
            }
        }
    }

    // Doc vector search — REMOVED (channel skipped above)

    // File vector search (INT8)
    if channel == "file" {
        let file_table = format!("file_vec_{}", repo.replace('-', "_"));
        let query_i8: Vec<i8> = crate::embedding::quantize_f32_to_i8(query_emb);
        if let Ok(rows) = crate::storage::vector::knn_search_int8(conn, &file_table, &query_i8, limit) {
            if !rows.is_empty() {
                let rowids: Vec<i64> = rows.iter().map(|(id, _)| *id).collect();
                let placeholders: String = rowids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sql = format!(
                    "SELECT rowid, file_path, COALESCE(summary,'') FROM files WHERE rowid IN ({})",
                    placeholders
                );
                let params: Vec<Box<dyn rusqlite::types::ToSql>> = rowids.iter()
                    .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
                    .collect();
                let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
                let mut detail_map: HashMap<i64, (String, String)> = HashMap::new();
                if let Ok(mut stmt) = conn.prepare(&sql) {
                    if let Ok(dr) = stmt.query_map(refs.as_slice(), |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                    }) {
                        for d in dr.flatten() { detail_map.insert(d.0, (d.1, d.2)); }
                    }
                }
                for (rowid, dist) in &rows {
                    if let Some((file_path, summary)) = detail_map.get(rowid) {
                        let sim = 1.0 / (1.0 + *dist as f64);
                        results.push((sim, file_path.clone(), "file".into(), file_path.clone(), 0,
                            String::new(), String::new(), summary.clone(), 0));
                    }
                }
            }
        }
    }

    results
}

// ── BM25 Precise Search ─────────────────────────────────────────────────
// 纯 BM25 FTS5 搜索，不涉及向量。名称通道 ×0.7，内容通道 ×0.3。
// skip_noise: true 时跳过噪音过滤（标定用）

/// BM25 precise search — FTS5 only, name ×0.7, content ×0.3 across all node types.
/// Returns FusedResult with single-channel BM25 scores.
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

    // ── Symbol: name channel (×0.7) ──
    let bm25_sym_name = storage::fts::search_symbols_name(conn, query, repo, branch, fetch_limit, kind_filter)?;
    // ── Symbol: comment channel (×0.3) ──
    let bm25_sym_comment = storage::fts::search_symbols_comment(conn, query, repo, branch, fetch_limit, kind_filter)?;
    // ── Doc: title channel (×0.7) ──
    let bm25_doc_title = storage::fts::search_docs_title(conn, query, repo, fetch_limit)?;
    // ── Doc: content channel (×0.3) ──
    let bm25_doc_content = storage::fts::search_docs_content(conn, query, repo, fetch_limit)?;
    // ── File: name channel (×0.7) ──
    let bm25_file_name = storage::fts::search_files_name(conn, query, repo, fetch_limit)?;
    // ── File: content channel (×0.3) ──
    let bm25_file_summary = storage::fts::search_files_summary(conn, query, repo, fetch_limit)?;

    let mut entries: HashMap<(String, String), FusedResult> = HashMap::new();

    let weight_name = 0.7;
    let weight_content = 0.3;

    // Helper: insert hits with weight
    let insert_weighted = |entries: &mut HashMap<(String, String), FusedResult>,
                           hits: &[storage::fts::SearchHit],
                           weight: f64,
                           default_hit_type: &str| {
        for hit in hits {
            let key = (hit.name.clone(), hit.file_path.clone());
            let norm_bm25 = 1.0 / (0.5 + hit.score.abs());
            let score = weight * norm_bm25;
            let is_doc = matches!(hit.hit_type, storage::fts::HitType::Doc);
            let is_file = matches!(hit.hit_type, storage::fts::HitType::File);
            let ht = if is_doc { "doc" } else if is_file { "file" } else { default_hit_type };
            entries.entry(key)
                .and_modify(|e| { if score > e.score { e.score = score; } })
                .or_insert(FusedResult {
                    score,
                    name: hit.name.clone(),
                    hit_type: ht.into(),
                    file_path: hit.file_path.clone(),
                    line_start: hit.line_start,
                    kind: hit.kind.clone(),
                    signature: hit.sig.clone(),
                    snippet: hit.snippet.clone(),
                    doc_id: if is_doc { hit.rowid } else { 0 },
                });
        }
    };

    // Name channel results (×0.7) insert first — they'll "win" ties via earlier insertion
    insert_weighted(&mut entries, &bm25_sym_name, weight_name, "code");
    insert_weighted(&mut entries, &bm25_doc_title, weight_name, "doc");
    insert_weighted(&mut entries, &bm25_file_name, weight_name, "file");

    // Content channel results (×0.3) — only raise score if name channel didn't have it
    insert_weighted(&mut entries, &bm25_sym_comment, weight_content, "code");
    insert_weighted(&mut entries, &bm25_doc_content, weight_content, "doc");
    insert_weighted(&mut entries, &bm25_file_summary, weight_content, "file");

    let mut fused: Vec<FusedResult> = entries.into_values().collect();
    fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // BM25 noise filter (skip during calibration)
    if !skip_noise {
        if let Some(profile) = crate::calib::noise_profile("bm25") {
            let threshold = 1.0;
            fused.retain(|r| (r.score - profile.top1_mean) / profile.top1_std.max(0.001) >= threshold);
        }
    }

    fused.truncate(limit);
    Ok(fused)
}

// ── Vector Semantic Search ──────────────────────────────────────────────
// 纯向量搜索，只搜符号名称通道，vec0 INT8 KNN。

/// Vector semantic search — vec0 INT8 KNN on symbol name channel only.
/// query_emb: pre-computed embedding for the search query.
/// skip_noise: true 时跳过噪音过滤（标定用）。
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
    let query_i8: Vec<i8> = crate::embedding::quantize_f32_to_i8(query_emb);

    let rows = match crate::storage::vector::knn_search_int8(conn, &sym_table, &query_i8, fetch_limit) {
        Ok(r) => r,
        Err(e) => return Err(anyhow::anyhow!("KNN search failed: {}", e)),
    };

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    // Batch fetch details
    let rowids: Vec<i64> = rows.iter().map(|(id, _)| *id).collect();
    let placeholders: String = rowids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT s.rowid, s.name, s.kind, s.file_path, s.line_start, \
         COALESCE(s.doc_comment,''), COALESCE(s.signature,'') \
         FROM symbols s JOIN branches b ON b.symbol_id = s.id \
         WHERE s.rowid IN ({}) AND b.branch_name=?",
        placeholders
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = rowids.iter()
        .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    params.push(Box::new(branch.to_string()));

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

    let mut results: Vec<FusedResult> = Vec::new();
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
            results.push(FusedResult {
                score: sim,
                name: name.clone(),
                hit_type: "code".into(),
                file_path: file_path.clone(),
                line_start: *line_start,
                kind: kind.clone(),
                signature: signature.clone(),
                snippet,
                doc_id: 0,
            });
        }
    }

    // Vector noise filter (skip during calibration)
    if !skip_noise {
        if let Some(profile) = crate::calib::noise_profile("vector") {
            let threshold = 1.0;
            results.retain(|r| (r.score - profile.top1_mean) / profile.top1_std.max(0.001) >= threshold);
        }
    }

    results.truncate(limit);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    fn make_bm25_hit(name: &str, file_path: &str, score: f64, hit_type: storage::fts::HitType, rowid: i64) -> storage::fts::SearchHit {
        storage::fts::SearchHit {
            rowid,
            score,
            hit_type,
            name: name.into(),
            file_path: file_path.into(),
            line_start: 1,
            kind: String::new(),
            sig: String::new(),
            snippet: String::new(),
        }
    }

    #[test]
    fn test_weighted_fuse_both_sides() {
        let bm25 = vec![
            make_bm25_hit("AuthService", "src/auth.cpp", -2.5, storage::fts::HitType::CodeName, 0),
            make_bm25_hit("LoginManager", "src/auth.cpp", -1.8, storage::fts::HitType::CodeName, 0),
        ];

        let vec_results = vec![
            (0.91, "AuthService".into(), "code".into(), "src/auth.cpp".into(), 10, String::new(), String::new(), String::new(), 0),
            (0.85, "AuthenticateUser".into(), "code".into(), "src/auth.cpp".into(), 30, String::new(), String::new(), String::new(), 0),
        ];

        let fused = weighted_fuse_single(&bm25, &vec_results, "code", 0.5, 0.5);
        // AuthService should be top (appears in both lists, gets max of BM25 and vec)
        assert_eq!(fused[0].name, "AuthService");
        assert!(fused.len() >= 2);
    }

    #[test]
    fn test_weighted_fuse_one_side_empty() {
        let bm25 = vec![make_bm25_hit("OnlyBM25", "src/test.cpp", -2.5, storage::fts::HitType::CodeName, 0)];
        let vec_results: Vec<(f64, String, String, String, i64, String, String, String, i64)> = vec![];

        let fused = weighted_fuse_single(&bm25, &vec_results, "code", 0.5, 0.5);
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].name, "OnlyBM25");
    }

    #[test]
    fn test_weighted_fuse_sorting() {
        // "BothHave" appears in both lists → should rank higher than single-list entries
        let bm25 = vec![
            make_bm25_hit("BothHave", "a.cpp", -2.5, storage::fts::HitType::CodeName, 0),
            make_bm25_hit("OnlyBM25", "b.cpp", -1.8, storage::fts::HitType::CodeName, 0),
        ];

        let vec_results = vec![
            (0.9, "BothHave".into(), "code".into(), "a.cpp".into(), 1, String::new(), String::new(), String::new(), 0),
            (0.8, "OnlyVec".into(), "code".into(), "c.cpp".into(), 3, String::new(), String::new(), String::new(), 0),
        ];

        let fused = weighted_fuse_single(&bm25, &vec_results, "code", 0.5, 0.5);
        // "BothHave" (in both lists) should rank #1
        assert_eq!(fused[0].name, "BothHave");
        assert!(fused[0].score > fused[1].score);
    }

    #[test]
    fn test_no_duplicate_docs() {
        // Same name + file_path should produce only ONE result (take max score)
        let bm25 = vec![
            make_bm25_hit("impl", "./doc/impl.md", -3.0, storage::fts::HitType::Doc, 42),
            make_bm25_hit("impl", "./doc/impl.md", -5.0, storage::fts::HitType::Doc, 42),
        ];

        let vec_results = vec![
            (0.85, "impl".into(), "doc".into(), "./doc/impl.md".into(), 0, String::new(), String::new(), String::new(), 42),
        ];

        let fused = weighted_fuse_single(&bm25, &vec_results, "code", 0.5, 0.5);
        // Should have exactly 1 result for impl.md, not 3 separate ones
        assert_eq!(fused.len(), 1, "Expected 1 result for impl.md, got {}", fused.len());
        assert_eq!(fused[0].name, "impl");
        assert_eq!(fused[0].doc_id, 42);
    }

    #[test]
    fn test_exact_match_ranks_first() {
        // Exact function name match should score highest
        let bm25 = vec![
            make_bm25_hit("DoCompactionWork", "db/db_impl.cc", -0.5, storage::fts::HitType::CodeName, 0),
            make_bm25_hit("CompactPointer", "db/version_edit.cc", -8.0, storage::fts::HitType::CodeName, 0),
            make_bm25_hit("PrevLogNumber", "db/version_edit.cc", -10.0, storage::fts::HitType::CodeName, 0),
        ];

        let vec_results = vec![
            (0.92, "DoCompactionWork".into(), "code".into(), "db/db_impl.cc".into(), 898, String::new(), String::new(), String::new(), 0),
            (0.45, "CompactPointer".into(), "code".into(), "db/version_edit.cc".into(), 19, String::new(), String::new(), String::new(), 0),
        ];

        let fused = weighted_fuse_single(&bm25, &vec_results, "code", 0.5, 0.5);
        assert_eq!(fused[0].name, "DoCompactionWork");
        // Score gap should be significant (exact vs fuzzy)
        assert!(fused[0].score > fused[1].score + 0.1,
            "Expected significant gap, got: #1={} #2={}", fused[0].score, fused[1].score);
    }
}

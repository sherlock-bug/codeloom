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
    pub snippet: String,  // first 200 chars of definition, empty for docs without content
    pub doc_id: i64,      // doc_nodes rowid (= doc_id for codeloom_get_doc), 0 for code results
}

/// Weighted fusion: normalize BM25 scores to (0,1] then combine with vector cosine similarity.
/// BM25 scores (from FTS5 bm25()) are ≤0, with 0 = best match. We use 1/(0.5+|score|)
/// so a strong BM25 hit (e.g. -0.5) normalizes to 1.0, outranking vector similarity.
/// Vector distances are converted to cosine similarity: 1/(1+distance), already in (0,1].
/// Final: max(w_bm25 * norm_bm25, w_vec * cosine_sim) — takes the stronger channel.
pub fn weighted_fuse(
    bm25_hits: &[storage::fts::SearchHit],
    vec_results: &[(f64, String, String, String, i64, String, i64)],
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
            snippet: hit.snippet.clone(),
            doc_id: if is_doc { hit.rowid } else { 0 },
        });
        // Take max — same name+file_path from multiple BM25 rows (multi-section doc) gets best score
        entry.score = entry.score.max(score);
    }

    // Vector side — normalize similarity and merge
    for (sim, name, hit_type, file_path, line_start, snippet, doc_id) in vec_results {
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
                kind: String::new(),
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

/// Hybrid search: runs FTS5 BM25 + vec0 ANN in parallel, weighted fuses results.
pub fn hybrid_search(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
    kind_filter: Option<&str>,
) -> anyhow::Result<Vec<FusedResult>> {
    let fetch_limit = (limit * 2).max(10);

    // Run BM25 keyword search (FTS5)
    let bm25_symbols = storage::fts::search_symbols(conn, query, repo, branch, fetch_limit, kind_filter)?;
    let bm25_docs = storage::fts::search_docs(conn, query, repo, fetch_limit)?;
    let bm25_files = storage::fts::search_files(conn, query, repo, fetch_limit)?;

    // Run vec0 vector search
    let vec_results = run_vector_search(conn, query, repo, branch, fetch_limit);
    eprintln!("[DEBUG hybrid_search] bm25_sym={} bm25_doc={} bm25_file={} vec={}", 
        bm25_symbols.len(), bm25_docs.len(), bm25_files.len(), vec_results.len());

    // Combine bm25 symbols + docs + files
    let mut all_bm25: Vec<storage::fts::SearchHit> = Vec::new();
    all_bm25.extend(bm25_symbols);
    all_bm25.extend(bm25_docs);
    all_bm25.extend(bm25_files);

    // Weighted fuse (equal weights, fall back if one side is empty)
    let has_bm25 = !all_bm25.is_empty();
    let has_vec = !vec_results.is_empty();
    let (w_bm25, w_vec) = match (has_bm25, has_vec) {
        (true, true) => (0.5, 0.5),
        (true, false) => (1.0, 0.0),
        (false, true) => (0.0, 1.0),
        (false, false) => (0.0, 0.0),
    };

    let mut fused = weighted_fuse(&all_bm25, &vec_results, w_bm25, w_vec);

    // If query is a substring of the result name, boost score.
    // One simple rule for all types — no per-type special casing.
    let query_lower = query.to_lowercase();
    for r in &mut fused {
        if r.name.to_lowercase().contains(&query_lower) {
            r.score = (r.score + 0.3).min(1.0);
        }
    }
    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    fused.truncate(limit);
    Ok(fused)
}

/// Run vec0 vector search, returns (similarity, name, hit_type, file_path, line_start, snippet, doc_id).
/// doc_id is doc_nodes.rowid for doc results, 0 for code results.
fn run_vector_search(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
) -> Vec<(f64, String, String, String, i64, String, i64)> {
    let mut results = Vec::new();

    let Ok(embedder) = crate::embedding::get_embedder() else { return results; };
    let query_emb = match embedder.embed(query) {
        Ok(e) => e,
        Err(_) => return results,
    };

    if !crate::storage::vector::try_load(conn) {
        return results; // vec0 not loaded
    }

    let sym_table = format!("symbol_vec_{}", repo.replace('-', "_"));
    let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));

    // Symbol vector search
    if let Ok(rows) = crate::storage::vector::knn_search(conn, &sym_table, &query_emb, limit) {
        for (rowid, dist) in rows {
            if let Ok((name, kind, file_path, line_start)) = conn.query_row(
                "SELECT s.name, s.kind, s.file_path, s.line_start FROM symbols s \
                 JOIN branches b ON b.symbol_id = s.id \
                 WHERE s.rowid=?1 AND b.branch_name=?2",
                rusqlite::params![rowid, branch],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, i64>(3)?,
                    ))
                },
            ) {
                let sim = 1.0 / (1.0 + dist as f64);
                results.push((sim, name, "code".into(), file_path, line_start, String::new(), 0));
            }
        }
    }

    // Doc vector search
    if let Ok(rows) = crate::storage::vector::knn_search(conn, &doc_table, &query_emb, limit) {
        for (rowid, dist) in rows {
            if let Ok((title, file_path, content)) = conn.query_row(
                "SELECT title, file_path, COALESCE(content, '') FROM doc_nodes WHERE rowid=?1",
                rusqlite::params![rowid],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
            ) {
                let sim = 1.0 / (1.0 + dist as f64);
                let snippet: String = content.chars().take(200).collect();
                results.push((sim, title, "doc".into(), file_path, 0, snippet, rowid));
            }
        }
    }

    // File vector search
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));
    if let Ok(rows) = crate::storage::vector::knn_search(conn, &file_table, &query_emb, limit) {
        for (rowid, dist) in rows {
            if let Ok((file_path, summary)) = conn.query_row(
                "SELECT file_path, COALESCE(summary, '') FROM files WHERE rowid=?1",
                rusqlite::params![rowid],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            ) {
                let sim = 1.0 / (1.0 + dist as f64);
                results.push((sim, file_path.clone(), "file".into(), file_path, 0, summary, 0));
            }
        }
    }

    results
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
            snippet: String::new(),
        }
    }

    #[test]
    fn test_weighted_fuse_both_sides() {
        let bm25 = vec![
            make_bm25_hit("AuthService", "src/auth.cpp", -2.5, storage::fts::HitType::Code, 0),
            make_bm25_hit("LoginManager", "src/auth.cpp", -1.8, storage::fts::HitType::Code, 0),
        ];

        let vec_results = vec![
            (0.91, "AuthService".into(), "code".into(), "src/auth.cpp".into(), 10, String::new(), 0),
            (0.85, "AuthenticateUser".into(), "code".into(), "src/auth.cpp".into(), 30, String::new(), 0),
        ];

        let fused = weighted_fuse(&bm25, &vec_results, 0.5, 0.5);
        // AuthService should be top (appears in both lists, gets max of BM25 and vec)
        assert_eq!(fused[0].name, "AuthService");
        assert!(fused.len() >= 2);
    }

    #[test]
    fn test_weighted_fuse_one_side_empty() {
        let bm25 = vec![make_bm25_hit("OnlyBM25", "src/test.cpp", -2.5, storage::fts::HitType::Code, 0)];
        let vec_results: Vec<(f64, String, String, String, i64, String, i64)> = vec![];

        let fused = weighted_fuse(&bm25, &vec_results, 0.5, 0.5);
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].name, "OnlyBM25");
    }

    #[test]
    fn test_weighted_fuse_sorting() {
        // "BothHave" appears in both lists → should rank higher than single-list entries
        let bm25 = vec![
            make_bm25_hit("BothHave", "a.cpp", -2.5, storage::fts::HitType::Code, 0),
            make_bm25_hit("OnlyBM25", "b.cpp", -1.8, storage::fts::HitType::Code, 0),
        ];

        let vec_results = vec![
            (0.9, "BothHave".into(), "code".into(), "a.cpp".into(), 1, String::new(), 0),
            (0.8, "OnlyVec".into(), "code".into(), "c.cpp".into(), 3, String::new(), 0),
        ];

        let fused = weighted_fuse(&bm25, &vec_results, 0.5, 0.5);
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
            (0.85, "impl".into(), "doc".into(), "./doc/impl.md".into(), 0, String::new(), 42),
        ];

        let fused = weighted_fuse(&bm25, &vec_results, 0.5, 0.5);
        // Should have exactly 1 result for impl.md, not 3 separate ones
        assert_eq!(fused.len(), 1, "Expected 1 result for impl.md, got {}", fused.len());
        assert_eq!(fused[0].name, "impl");
        assert_eq!(fused[0].doc_id, 42);
    }

    #[test]
    fn test_exact_match_ranks_first() {
        // Exact function name match should score highest
        let bm25 = vec![
            make_bm25_hit("DoCompactionWork", "db/db_impl.cc", -0.5, storage::fts::HitType::Code, 0),
            make_bm25_hit("CompactPointer", "db/version_edit.cc", -8.0, storage::fts::HitType::Code, 0),
            make_bm25_hit("PrevLogNumber", "db/version_edit.cc", -10.0, storage::fts::HitType::Code, 0),
        ];

        let vec_results = vec![
            (0.92, "DoCompactionWork".into(), "code".into(), "db/db_impl.cc".into(), 898, String::new(), 0),
            (0.45, "CompactPointer".into(), "code".into(), "db/version_edit.cc".into(), 19, String::new(), 0),
        ];

        let fused = weighted_fuse(&bm25, &vec_results, 0.5, 0.5);
        assert_eq!(fused[0].name, "DoCompactionWork");
        // Score gap should be significant (exact vs fuzzy)
        assert!(fused[0].score > fused[1].score + 0.1,
            "Expected significant gap, got: #1={} #2={}", fused[0].score, fused[1].score);
    }
}

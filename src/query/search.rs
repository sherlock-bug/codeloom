// Hybrid search — RRF fusion of FTS5 BM25 + vec0 vector ANN
use crate::storage;
use rusqlite::Connection;
use std::collections::HashMap;

/// A fused search result from hybrid (BM25 + vector) search
#[derive(Debug, Clone)]
pub struct FusedResult {
    pub score: f64, // RRF fusion score
    pub name: String,
    pub hit_type: String, // "code" or "doc"
    pub file_path: String,
    pub line_start: i64,
}

/// Reciprocal Rank Fusion: merge BM25 and vector result lists.
/// k=60 is the standard value (used by Elasticsearch 8.x).
pub fn rrf_fuse(
    bm25: &[storage::fts::SearchHit],
    vec_results: &[(f64, String, String, String, i64)], // (similarity, name, hit_type, file_path, line_start)
    k: f64,
) -> Vec<FusedResult> {
    let mut scores: HashMap<String, (f64, String, String, i64, String)> = HashMap::new();

    // BM25 side
    for (rank, hit) in bm25.iter().enumerate() {
        let key = hit.name.clone();
        let score = 1.0 / (k + (rank as f64) + 1.0);
        let entry =
            scores
                .entry(key)
                .or_insert((0.0, hit.name.clone(), hit_type_to_str(&hit.hit_type), hit.line_start, hit.file_path.clone()));
        entry.0 += score;
    }

    // Vector side
    for (rank, (_, name, hit_type, file_path, line_start)) in vec_results.iter().enumerate() {
        let key = name.clone();
        let score = 1.0 / (k + (rank as f64) + 1.0);
        if let Some(entry) = scores.get_mut(&key) {
            entry.0 += score;
        } else {
            scores.insert(key, (score, name.clone(), hit_type.clone(), *line_start, file_path.clone()));
        }
    }

    // Sort by RRF score descending
    let mut fused: Vec<FusedResult> = scores
        .into_values()
        .map(|(s, name, ht, ls, fp)| FusedResult {
            score: s,
            name,
            hit_type: ht,
            file_path: fp,
            line_start: ls,
        })
        .collect();

    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    fused
}

fn hit_type_to_str(ht: &storage::fts::HitType) -> String {
    match ht {
        storage::fts::HitType::Code => "code".to_string(),
        storage::fts::HitType::Doc => "doc".to_string(),
    }
}

/// Hybrid search: runs FTS5 BM25 + vec0 ANN in parallel, RRF fuses results.
pub fn hybrid_search(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
) -> anyhow::Result<Vec<FusedResult>> {
    let fetch_limit = (limit * 2).max(10);

    // Run BM25 keyword search (FTS5)
    let bm25_symbols = storage::fts::search_symbols(conn, query, repo, branch, fetch_limit)?;
    let bm25_docs = storage::fts::search_docs(conn, query, repo, fetch_limit)?;

    // Run vec0 vector search
    let vec_results = run_vector_search(conn, query, repo, branch, fetch_limit);

    // Combine bm25 symbols + docs
    let mut all_bm25: Vec<storage::fts::SearchHit> = Vec::new();
    all_bm25.extend(bm25_symbols);
    all_bm25.extend(bm25_docs);

    // RRF fuse
    let mut fused = rrf_fuse(&all_bm25, &vec_results, 60.0);
    fused.truncate(limit);
    Ok(fused)
}

/// Run vec0 vector search, same as old semantic_search but returns structured results.
fn run_vector_search(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
) -> Vec<(f64, String, String, String, i64)> {
    let mut results = Vec::new();

    let embedder = crate::embedding::get_embedder();
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
                results.push((sim, name, "code".into(), file_path, line_start));
            }
        }
    }

    // Doc vector search
    if let Ok(rows) = crate::storage::vector::knn_search(conn, &doc_table, &query_emb, limit) {
        for (rowid, dist) in rows {
            if let Ok((title, file_path)) = conn.query_row(
                "SELECT title, file_path FROM doc_nodes WHERE rowid=?1",
                rusqlite::params![rowid],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            ) {
                let sim = 1.0 / (1.0 + dist as f64);
                results.push((sim, format!("📄 {}", title), "doc".into(), file_path, 0));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    #[test]
    fn test_rrf_fusion_both_sides() {
        // Create some BM25 hits
        let bm25 = vec![
            storage::fts::SearchHit {
                rowid: 1,
                score: 2.5,
                hit_type: storage::fts::HitType::Code,
                name: "AuthService".into(),
                file_path: "src/auth.cpp".into(),
                line_start: 10,
            },
            storage::fts::SearchHit {
                rowid: 2,
                score: 1.8,
                hit_type: storage::fts::HitType::Code,
                name: "LoginManager".into(),
                file_path: "src/auth.cpp".into(),
                line_start: 20,
            },
        ];

        // Create vec results: AuthService also appears, plus a vec-only hit
        let vec_results = vec![
            (0.91, "AuthService".into(), "code".into(), "src/auth.cpp".into(), 10),
            (0.85, "AuthenticateUser".into(), "code".into(), "src/auth.cpp".into(), 30),
        ];

        let fused = rrf_fuse(&bm25, &vec_results, 60.0);
        // AuthService should be top (appears in both lists)
        assert_eq!(fused[0].name, "AuthService");
        // Others follow
        assert!(fused.len() >= 2);
    }

    #[test]
    fn test_rrf_fusion_one_side_empty() {
        let bm25 = vec![storage::fts::SearchHit {
            rowid: 1,
            score: 2.5,
            hit_type: storage::fts::HitType::Code,
            name: "OnlyBM25".into(),
            file_path: "src/test.cpp".into(),
            line_start: 1,
        }];

        let vec_results: Vec<(f64, String, String, String, i64)> = vec![];

        let fused = rrf_fuse(&bm25, &vec_results, 60.0);
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].name, "OnlyBM25");
    }

    #[test]
    fn test_rrf_sorting() {
        // Doc that appears in both lists should rank higher than ones only in one
        let bm25 = vec![
            storage::fts::SearchHit {
                rowid: 1, score: 0.0,
                hit_type: storage::fts::HitType::Code,
                name: "BothHave".into(), file_path: "a.cpp".into(), line_start: 1,
            },
            storage::fts::SearchHit {
                rowid: 2, score: 0.0,
                hit_type: storage::fts::HitType::Code,
                name: "OnlyBM25".into(), file_path: "b.cpp".into(), line_start: 2,
            },
        ];

        let vec_results = vec![
            (0.9, "BothHave".into(), "code".into(), "a.cpp".into(), 1),
            (0.8, "OnlyVec".into(), "code".into(), "c.cpp".into(), 3),
        ];

        let fused = rrf_fuse(&bm25, &vec_results, 60.0);
        // "BothHave" (in both lists) should rank #1
        assert_eq!(fused[0].name, "BothHave");
        assert!(fused[0].score > fused[1].score);
    }
}
